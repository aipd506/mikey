package com.mikey.media

import android.annotation.SuppressLint
import android.content.Context
import android.media.AudioFormat
import android.media.AudioManager
import android.media.AudioRecord
import android.media.MediaRecorder
import android.os.Process
import android.os.SystemClock
import android.util.Log

/** 10 ms of 48 kHz mono PCM s16le, as recorded. */
class AudioFrame(val seq: Int, val captureTimeUs: Long, val pcm: ByteArray)

/**
 * Records raw audio (48 kHz, mono, 16-bit) in 10 ms frames on its own high-priority thread:
 * with AAudio in low-latency mode where it works, with AudioRecord otherwise (media-pipeline.md).
 * No processing on the phone: noise suppression runs on the PC.
 * [onFrame] runs on the capture thread and must never block.
 */
class AudioCapture(private val context: Context, private val onFrame: (AudioFrame) -> Unit) {
    @Volatile private var running = false

    private var seq = 0

    fun start() {
        running = true
        Thread(::record, "mikey-capture").start()
    }

    /** Returns at once. The mic is released within one frame. */
    fun stop() {
        running = false
    }

    private fun record() {
        Process.setThreadPriority(Process.THREAD_PRIORITY_URGENT_AUDIO)
        val aaudio = AAudioInput.open(SAMPLE_RATE)
        if (aaudio != null) {
            Log.i(TAG, "Capturing with AAudio, low latency")
            val ended = aaudio.use { capture(it::read) }
            if (ended) return
            // AAudio stopped working, say because the device changed. AudioRecord takes over.
        }
        recordWithAudioRecord()
    }

    // MikeyService only runs after RECORD_AUDIO is granted.
    @SuppressLint("MissingPermission")
    private fun recordWithAudioRecord() {
        val bufferBytes = maxOf(AudioRecord.getMinBufferSize(SAMPLE_RATE, CHANNEL, ENCODING), FRAME_BYTES * 2)
        val recorder = try {
            AudioRecord(source(), SAMPLE_RATE, CHANNEL, ENCODING, bufferBytes)
        } catch (e: IllegalArgumentException) {
            Log.e(TAG, "Can't create the recorder", e)
            return
        }
        Log.i(TAG, "Capturing with AudioRecord")
        try {
            if (android.media.audiofx.AutomaticGainControl.isAvailable()) {
                try {
                    android.media.audiofx.AutomaticGainControl.create(recorder.audioSessionId)?.let { agc ->
                        agc.enabled = false
                        agc.release()
                    }
                } catch (e: Throwable) {
                    Log.w(TAG, "Could not disable system AGC", e)
                }
            }
            recorder.startRecording()
            capture { pcm -> recorder.read(pcm, 0, FRAME_BYTES).let { if (it < 0) it else it / 2 } }
        } catch (e: IllegalStateException) {
            // For example another app holds the mic.
            Log.e(TAG, "Can't record", e)
        } finally {
            recorder.release()
        }
    }

    /**
     * Reads frames with [read] (which returns samples read, or a negative error) until stopped.
     * Returns false when the source failed instead.
     */
    private fun capture(read: (ByteArray) -> Int): Boolean {
        while (running) {
            val pcm = ByteArray(FRAME_BYTES)
            val samples = read(pcm)
            if (samples < 0) {
                Log.e(TAG, "Recording stopped with error $samples")
                return false
            }
            if (samples == FRAME_SAMPLES) onFrame(AudioFrame(seq++, SystemClock.elapsedRealtimeNanos() / 1000, pcm))
        }
        return true
    }

    private fun source(): Int {
        val unprocessed = context.getSystemService(AudioManager::class.java)
            .getProperty(AudioManager.PROPERTY_SUPPORT_AUDIO_SOURCE_UNPROCESSED) == "true"
        // Never VOICE_COMMUNICATION: its own noise suppression and gain would fight the PC's.
        // Fall back to VOICE_RECOGNITION where UNPROCESSED is unsupported to avoid OEM AGC ducking on MIC.
        return if (unprocessed) MediaRecorder.AudioSource.UNPROCESSED else MediaRecorder.AudioSource.VOICE_RECOGNITION
    }

    private companion object {
        const val TAG = "AudioCapture"
        const val SAMPLE_RATE = 48_000
        const val CHANNEL = AudioFormat.CHANNEL_IN_MONO
        const val ENCODING = AudioFormat.ENCODING_PCM_16BIT
        const val FRAME_SAMPLES = SAMPLE_RATE / 100
        const val FRAME_BYTES = FRAME_SAMPLES * 2
    }
}
