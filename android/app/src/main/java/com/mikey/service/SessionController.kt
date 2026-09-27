package com.mikey.service

import android.content.Context
import android.os.Build
import android.os.SystemClock
import android.util.Log
import com.mikey.media.AudioCapture
import com.mikey.media.AudioFrame
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
import com.mikey.transport.Discovery
import com.mikey.transport.TransportManager
import com.mikey.transport.WifiLatencyLock
import org.json.JSONException
import java.io.BufferedInputStream
import java.io.BufferedOutputStream
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.IOException
import java.net.ProtocolException
import java.net.Socket
import java.net.SocketTimeoutException
import java.util.concurrent.ArrayBlockingQueue
import java.util.concurrent.TimeUnit

/**
 * Streams the mic to the PC: connect, handshake, send audio and heartbeats, and reconnect with
 * backoff when the link drops. The network runs on its own thread; the capture thread only drops
 * frames into a small queue, so recording never waits on the network.
 *
 * [onLink] reports every change of [Link]. It is called on the session thread.
 */
class SessionController(context: Context, private val onLink: (Link) -> Unit) {
    private val settings = Settings(context)
    private val transports = TransportManager(settings, Discovery(settings.deviceId, Build.MODEL))
    private val wifiLock = WifiLatencyLock(context)
    private val frames = ArrayBlockingQueue<AudioFrame>(QUEUE_FRAMES)
    private val capture = AudioCapture(context) { frames.offerDroppingOldest(it) }
    private val thread = Thread(::sessionLoop, "mikey-session")

    @Volatile private var running = false

    fun start() {
        running = true
        capture.start()
        thread.start()
    }

    /** Releases the mic at once. The session thread then says BYE and closes on its own. */
    fun stop() {
        running = false
        capture.stop()
        thread.interrupt()
    }

    private fun sessionLoop() {
        var failures = 0
        while (running) {
            try {
                val connection = transports.open()
                val transport = connection.transport
                connection.socket.use { socket ->
                    val output = DataOutputStream(BufferedOutputStream(socket.getOutputStream()))
                    val input = DataInputStream(BufferedInputStream(socket.getInputStream()))
                    handshake(output, input, transport.level)
                    if (transport.level == 4) settings.lastPcAddress = transport.host
                    failures = 0
                    stream(socket, input, output, transport.level)
                }
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
            }
            if (running) pause(reconnectDelayMs(failures++))
        }
    }

    /** Sends HELLO and reads the PC's answer. Returns once we're accepted, throws otherwise. */
    private fun handshake(output: DataOutputStream, input: DataInputStream, level: Int) {
        val paired = settings.pairedPc
        output.writeFrame(FrameType.HELLO, helloPayload(settings.deviceId, Build.MODEL, level, paired?.token))
        output.flush()
        var approvalDeadlineMs = 0L
        while (true) {
            val frame = try {
                input.readFrame()
            } catch (e: SocketTimeoutException) {
                // The PC says nothing while it asks its user. Keep waiting, up to its 60 s prompt limit.
                if (!running || approvalDeadlineMs == 0L || SystemClock.elapsedRealtime() >= approvalDeadlineMs) throw e
                continue
            }
            when (frame.type) {
                FrameType.PENDING -> {
                    approvalDeadlineMs = SystemClock.elapsedRealtime() + APPROVAL_WAIT_MS
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
                    Log.i(TAG, "${if (welcome.resumed) "Resumed with" else "Connected to"} ${welcome.pcName} on level $level")
                    return
                }
                FrameType.REJECT -> throw RejectedException(parseReject(frame.payload))
                else -> Unit // Not for us. Unknown frames are skipped, as the spec says.
            }
        }
    }

    /** Sends audio and heartbeats until stopped (then says BYE), or throws when the link fails. */
    private fun stream(socket: Socket, input: DataInputStream, output: DataOutputStream, level: Int) {
        frames.clear() // Audio queued while we were offline is too old to play now.
        if (running) onLink(Link.Live(level))
        if (level == 4) wifiLock.hold()
        Thread({ receive(input, socket) }, "mikey-receive").start()
        try {
            var lastHeartbeatMs = 0L
            while (running) {
                val frame = try {
                    frames.poll(POLL_MS, TimeUnit.MILLISECONDS)
                } catch (e: InterruptedException) {
                    null
                }
                if (frame != null) {
                    output.writeMediaFrame(
                        FrameType.AUDIO,
                        frame.seq,
                        frame.captureTimeUs,
                        MediaHeader.CODEC_PCM_S16LE,
                        frame.pcm,
                        0,
                        frame.pcm.size,
                    )
                }
                val nowMs = SystemClock.elapsedRealtime()
                if (nowMs - lastHeartbeatMs >= HEARTBEAT_MS) {
                    output.writeFrame(FrameType.HEARTBEAT, heartbeatPayload(SystemClock.elapsedRealtimeNanos() / 1000))
                    lastHeartbeatMs = nowMs
                }
                output.flush()
            }
            sayBye(output)
        } finally {
            wifiLock.release()
            onLink(Link.Searching)
            socket.close()
        }
    }

    /** Any frame from the PC proves the link is alive. Six silent seconds or a BYE end it. */
    private fun receive(input: DataInputStream, socket: Socket) {
        try {
            do {
                val frame = input.readFrame()
            } while (frame.type != FrameType.BYE)
        } catch (e: IOException) {
            // Timed out, dropped, or closed by the sender.
        } finally {
            socket.close() // Makes the sender's next write fail, so it reconnects.
        }
    }

    private fun sayBye(output: DataOutputStream) {
        try {
            output.writeFrame(FrameType.BYE, byePayload("stop"))
            output.flush()
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

        /** The PC gives its user 60 s to answer the prompt. A little longer, so we never give up first. */
        const val APPROVAL_WAIT_MS = 65_000L
        const val REJECT_RETRY_MS = 5_000L
    }
}

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
