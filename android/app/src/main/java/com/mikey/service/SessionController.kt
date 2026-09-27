package com.mikey.service

import android.content.Context
import android.os.Build
import android.os.SystemClock
import android.util.Log
import com.mikey.media.AudioCapture
import com.mikey.media.AudioFrame
import com.mikey.media.FrameJoiner
import com.mikey.media.OpusEncoder
import com.mikey.protocol.FrameType
import com.mikey.protocol.MediaHeader
import com.mikey.protocol.byePayload
import com.mikey.protocol.heartbeatPayload
import com.mikey.protocol.helloPayload
import com.mikey.protocol.parseReject
import com.mikey.protocol.parseWelcome
import com.mikey.protocol.readFrame
import com.mikey.protocol.writeFrame
import com.mikey.protocol.writeMediaFrame
import com.mikey.settings.PairedPc
import com.mikey.settings.Settings
import com.mikey.transport.Connection
import com.mikey.transport.Discovery
import com.mikey.transport.TransportManager
import com.mikey.transport.WifiLatencyLock
import org.json.JSONException
import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.io.Closeable
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.IOException
import java.net.ProtocolException
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean
import java.util.concurrent.atomic.AtomicReference
import kotlin.concurrent.thread

/**
 * Streams the mic to the PC: connect, handshake, send audio and heartbeats, and reconnect with
 * backoff when the link drops. While streaming, a second thread watches for a better level and
 * opens it first, so the audio moves over without a gap and the old link is closed after.
 *
 * The network runs on its own threads; the capture thread only drops frames into a small queue,
 * so recording never waits on the network. [onLink] and [onCableHint] are called on those threads.
 */
class SessionController(
    context: Context,
    private val onLink: (Link) -> Unit,
    onCableHint: (Boolean) -> Unit,
) {
    private val settings = Settings(context)
    private val transports = TransportManager(context, settings, Discovery(settings.deviceId, Build.MODEL), onCableHint)
    private val wifiLock = WifiLatencyLock(context)
    private val frames = ArrayBlockingQueue<AudioFrame>(QUEUE_FRAMES)
    private val capture = AudioCapture(context) { frames.offerDroppingOldest(it) }
    private val thread = Thread(::sessionLoop, "mikey-session")

    @Volatile private var running = false

    /** The link being greeted right now, so stop() can cut a long wait for approval short. */
    @Volatile private var greeting: Wire? = null

    /** What the PC said it can do in WELCOME, e.g. `opus`. */
    private var pcCaps: Set<String> = emptySet()

    fun start() {
        running = true
        transports.start()
        capture.start()
        thread.start()
    }

    /** Releases the mic at once. The session thread then says BYE and closes on its own. */
    fun stop() {
        running = false
        capture.stop()
        transports.stop()
        thread.interrupt()
        greeting?.close()
    }

    private fun sessionLoop() {
        var failures = 0
        while (running) {
            try {
                val wire = connect(transports.open(), silent = false)
                failures = 0
                stream(wire)
            } catch (e: RejectedException) {
                Log.i(TAG, "PC refused us: ${e.reason}")
                when (rejectPolicy(e.reason)) {
                    Reaction.RETRY -> Unit
                    Reaction.FORGET_AND_RETRY -> settings.forgetPc()
                    Reaction.GIVE_UP -> {
                        onLink(Link.Refused(e.reason))
                        waitUntilStopped()
                        return
                    }
                }
                if (running) pause(REJECT_RETRY_MS)
                continue
            } catch (e: IOException) {
                Log.i(TAG, "No link to the PC: $e")
                onLink(Link.Searching)
                transports.noteLevel(0)
            }
            if (running) pause(reconnectDelayMs(failures++))
        }
    }

    /** An open connection to the PC, with framed streams. */
    private class Wire(private val connection: Connection) : Closeable {
        val level = connection.level
        val host = connection.host
        val output = DataOutputStream(BufferedOutputStream(connection.output))
        val input = DataInputStream(BufferedInputStream(connection.input))

        /** No frame from the PC for this long means the link is dead (wire-protocol.md). */
        var readTimeoutMs: Int
            get() = connection.readTimeoutMs
            set(value) {
                connection.readTimeoutMs = value
            }

        init {
            readTimeoutMs = LINK_TIMEOUT_MS
        }

        override fun close() = connection.close()
    }

    /** Wraps and greets a fresh connection. Throws (and closes it) unless the PC accepts us. */
    private fun connect(connection: Connection, silent: Boolean): Wire {
        val wire = Wire(connection)
        greeting = wire
        try {
            handshake(wire, silent)
        } catch (e: Exception) {
            wire.close()
            throw e
        } finally {
            greeting = null
        }
        when (wire.level) {
            3 -> settings.pcBtAddress = wire.host
            4 -> settings.lastPcAddress = wire.host
        }
        return wire
    }

    /**
     * Sends HELLO and reads the PC's answer. Returns once we're accepted, throws otherwise.
     * A [silent] handshake is an upgrade while we're already streaming: it must not bother the
     * PC's user, so PENDING counts as a failure instead of a wait.
     */
    private fun handshake(wire: Wire, silent: Boolean) {
        val paired = settings.pairedPc
        wire.output.writeFrame(FrameType.HELLO, helloPayload(settings.deviceId, Build.MODEL, wire.level, paired?.token))
        wire.output.flush()
        while (true) {
            val frame = wire.input.readFrame()
            when (frame.type) {
                FrameType.PENDING -> {
                    if (silent) throw IOException("The PC would ask its user; keeping the current link")
                    // The PC says nothing while it asks its user. Wait it out, up to its 60 s prompt limit.
                    wire.readTimeoutMs = APPROVAL_WAIT_MS
                    onLink(Link.Waiting)
                }
                FrameType.WELCOME -> {
                    val welcome = try {
                        parseWelcome(frame.payload)
                    } catch (e: JSONException) {
                        throw ProtocolException("Bad WELCOME: ${e.message}")
                    }
                    if (paired?.id != welcome.pcId || paired.token != welcome.token) {
                        settings.pairedPc = PairedPc(welcome.pcId, welcome.pcName, welcome.token)
                    }
                    pcCaps = welcome.pcCaps
                    wire.readTimeoutMs = LINK_TIMEOUT_MS
                    Log.i(TAG, "${if (welcome.resumed) "Resumed with" else "Connected to"} ${welcome.pcName} on level ${wire.level}")
                    return
                }
                FrameType.REJECT -> throw RejectedException(parseReject(frame.payload))
                else -> Unit // Not for us. Unknown frames are skipped, as the spec says.
            }
        }
    }

    /**
     * Sends audio and heartbeats until stopped (then says BYE), or throws when the link fails.
     * Moves to a better link whenever the upgrade thread hands one over.
     */
    private fun stream(first: Wire) {
        var wire = first
        val current = AtomicReference(first)
        val better = AtomicReference<Wire?>()
        val active = AtomicBoolean(true)
        frames.clear() // Audio queued while we were offline is too old to play now.
        var sender = startSending(wire)
        val upgrader = thread(name = "mikey-upgrade") { upgradeLoop(current, better, active) }
        try {
            var lastHeartbeatMs = 0L
            while (running) {
                better.getAndSet(null)?.let { next ->
                    val old = wire
                    wire = next
                    current.set(next)
                    sender.close()
                    sender = startSending(next)
                    sayBye(old, "switch")
                    old.close()
                    Log.i(TAG, "Moved from level ${old.level} to level ${next.level}")
                }
                val frame = try {
                    frames.poll(POLL_MS, TimeUnit.MILLISECONDS)
                } catch (e: InterruptedException) {
                    null
                }
                if (frame != null) sender.send(frame, wire.output)
                val nowMs = SystemClock.elapsedRealtime()
                if (nowMs - lastHeartbeatMs >= HEARTBEAT_MS) {
                    wire.output.writeFrame(FrameType.HEARTBEAT, heartbeatPayload(SystemClock.elapsedRealtimeNanos() / 1000))
                    lastHeartbeatMs = nowMs
                }
                wire.output.flush()
            }
            sayBye(wire, "stop")
        } catch (e: IOException) {
            transports.markDead(wire.level)
            throw e
        } finally {
            active.set(false)
            transports.wake()
            upgrader.interrupt()
            better.getAndSet(null)?.close()
            sender.close()
            wifiLock.release()
            onLink(Link.Searching)
            transports.noteLevel(0)
            wire.close()
        }
    }

    /** Turns capture frames into AUDIO frames for one link: raw PCM, or Opus with the link's profile. */
    private class Sender(private val encoder: OpusEncoder?, private val joiner: FrameJoiner) : Closeable {
        fun send(frame: AudioFrame, output: DataOutputStream) {
            if (encoder == null) {
                output.writeMediaFrame(FrameType.AUDIO, frame.seq, frame.captureTimeUs, MediaHeader.CODEC_PCM_S16LE, frame.pcm, 0, frame.pcm.size)
                return
            }
            val packet = joiner.add(frame) ?: return
            val length = encoder.encode(packet.pcm)
            if (length > 0) {
                output.writeMediaFrame(FrameType.AUDIO, packet.seq, packet.captureTimeUs, MediaHeader.CODEC_OPUS, encoder.packet, 0, length)
            }
        }

        override fun close() {
            encoder?.close()
        }
    }

    /** Everything that depends on which link we send on: state, Wi-Fi lock, codec, and the reader. */
    private fun startSending(wire: Wire): Sender {
        if (running) onLink(Link.Live(wire.level))
        transports.noteLevel(wire.level)
        if (wire.level == 4) wifiLock.hold() else wifiLock.release()
        val codec = audioCodecFor(wire.level, settings.losslessWifi, pcHasOpus = "opus" in pcCaps)
        val profile = opusProfileFor(wire.level)
        val encoder = if (codec == AudioCodec.OPUS) OpusEncoder(profile.application, profile.bitrate) else null
        Log.i(TAG, "Level ${wire.level}: sending ${if (encoder == null) "raw PCM" else "Opus ${profile.bitrate / 1000} kbps, ${profile.framesPerPacket * 10} ms frames"}")
        thread(name = "mikey-receive") { receive(wire) }
        return Sender(encoder, FrameJoiner(profile.framesPerPacket))
    }

    /**
     * Waits for a chance at a better level, opens and greets it, and hands it to [stream].
     * A failed try just waits for the next chance; the current link keeps streaming meanwhile.
     */
    private fun upgradeLoop(current: AtomicReference<Wire>, better: AtomicReference<Wire?>, active: AtomicBoolean) {
        while (active.get() && running) {
            transports.waitForBetterChance(current.get().level)
            if (!active.get() || !running || better.get() != null) continue
            val level = current.get().level
            val next = try {
                connect(transports.openBetterThan(level), silent = true)
            } catch (e: IOException) {
                transports.noteLevel(level)
                continue
            } catch (e: RejectedException) {
                Log.i(TAG, "No upgrade: PC said ${e.reason}")
                continue
            }
            if (active.get()) better.set(next) else next.close()
        }
        better.getAndSet(null)?.close()
    }

    /** Any frame from the PC proves the link is alive. Six silent seconds or a BYE end it. */
    private fun receive(wire: Wire) {
        try {
            do {
                val frame = wire.input.readFrame()
            } while (frame.type != FrameType.BYE)
        } catch (e: IOException) {
            // Timed out, dropped, or closed by the sender.
        } finally {
            wire.close() // Makes the sender's next write fail, so it reconnects.
        }
    }

    private fun sayBye(wire: Wire, reason: String) {
        try {
            wire.output.writeFrame(FrameType.BYE, byePayload(reason))
            wire.output.flush()
        } catch (e: IOException) {
            // Best effort: the PC also notices the socket closing.
        }
    }

    /** After a final refusal we don't try again by ourselves, so the PC's user isn't asked over and over. */
    private fun waitUntilStopped() {
        while (running) pause(Long.MAX_VALUE)
    }

    private fun pause(ms: Long) {
        try {
            Thread.sleep(ms)
        } catch (e: InterruptedException) {
            // stop() wakes us so we can exit.
        }
    }

    private companion object {
        const val TAG = "SessionController"

        /** 200 ms of audio. Anything older would reach the PC too late to be played. */
        const val QUEUE_FRAMES = 20
        const val HEARTBEAT_MS = 2_000L
        const val POLL_MS = 100L
        const val LINK_TIMEOUT_MS = 6_000

        /** The PC gives its user 60 s to answer the prompt. A little longer, so we never give up first. */
        const val APPROVAL_WAIT_MS = 65_000
        const val REJECT_RETRY_MS = 5_000L
    }
}

internal enum class AudioCodec { PCM, OPUS }

/** Raw PCM on USB, where bandwidth is free, and when the user asked for lossless Wi-Fi. Opus elsewhere, if the PC can decode it. */
internal fun audioCodecFor(level: Int, losslessWifi: Boolean, pcHasOpus: Boolean): AudioCodec = when {
    !pcHasOpus || level <= 2 -> AudioCodec.PCM
    level == 4 && losslessWifi -> AudioCodec.PCM
    else -> AudioCodec.OPUS
}

/** How Opus is set up on a level. */
internal class OpusProfile(val application: OpusEncoder.Application, val bitrate: Int, val framesPerPacket: Int)

/**
 * Wi-Fi: music-grade low delay at 96 kbps, 10 ms frames. Bluetooth: speech mode at 48 kbps and
 * 20 ms frames, which suit its packet timing (media-pipeline.md). RFCOMM retransmits, so loss
 * never reaches Opus and FEC would only cost bits.
 */
internal fun opusProfileFor(level: Int): OpusProfile =
    if (level == 3) OpusProfile(OpusEncoder.Application.VOIP, 48_000, 2) else OpusProfile(OpusEncoder.Application.LOW_DELAY, 96_000, 1)

/** The PC answered HELLO with REJECT. */
private class RejectedException(val reason: String) : Exception("PC said $reason")

internal enum class Reaction { RETRY, FORGET_AND_RETRY, GIVE_UP }

/**
 * What to do about a REJECT. `timeout` (nobody answered the PC's prompt) and `busy` (another phone
 * is streaming) can clear up by themselves, so ask again after a while. `bad_token` means our pairing
 * is stale: drop it and introduce ourselves as new. Anything else, like `denied` or `version`, is
 * final until the user turns the mic off and on.
 */
internal fun rejectPolicy(reason: String): Reaction = when (reason) {
    "timeout", "busy" -> Reaction.RETRY
    "bad_token" -> Reaction.FORGET_AND_RETRY
    else -> Reaction.GIVE_UP
}

/** Reconnect waits 0.5 s, 1 s, 2 s, 4 s, then 5 s from then on. */
internal fun reconnectDelayMs(failures: Int): Long = minOf(500L shl failures.coerceAtMost(4), 5_000L)

/** Never blocks: when the queue is full, the oldest item makes room. Fresh audio beats complete audio. */
internal fun <T> ArrayBlockingQueue<T>.offerDroppingOldest(item: T) {
    while (!offer(item)) poll()
}
