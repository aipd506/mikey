package com.mikey.media

import android.content.Context
import android.graphics.ImageFormat
import android.graphics.Rect
import android.graphics.YuvImage
import android.os.Build
import android.os.Handler
import android.os.Looper
import android.os.PowerManager
import android.os.SystemClock
import android.util.Log
import android.util.Size
import android.view.OrientationEventListener
import android.view.Surface
import androidx.camera.core.AspectRatio
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageAnalysis
import androidx.camera.core.ImageProxy
import androidx.camera.core.resolutionselector.AspectRatioStrategy
import androidx.camera.core.resolutionselector.ResolutionSelector
import androidx.camera.core.resolutionselector.ResolutionStrategy
import androidx.camera.lifecycle.ProcessCameraProvider
import java.io.ByteArrayOutputStream
import java.util.concurrent.Executor
import java.util.concurrent.Executors

/** One camera image as JPEG, ready for a VIDEO frame. */
class VideoFrame(val seq: Int, val captureTimeUs: Long, val jpeg: ByteArray)

/**
 * Captures the camera as upright JPEG frames for the PC (media-pipeline.md). No preview: the
 * phone never shows the video. CameraX keeps only the latest image, so no backlog builds up,
 * and rotates it in native code to match how the phone is held, so the PC always gets an
 * upright picture. [onFrame] is called on the capture thread and must not block.
 */
class VideoCapture(private val context: Context, private val onFrame: (VideoFrame) -> Unit) {
    private val main = Handler(Looper.getMainLooper())
    private val mainExecutor = Executor { main.post(it) }
    private val worker = Executors.newSingleThreadExecutor { Thread(it, "mikey-video") }
    private val power = context.getSystemService(PowerManager::class.java)

    // Main thread only.
    private var lifecycle: ServiceLifecycle? = null
    private var provider: ProcessCameraProvider? = null
    private var analysis: ImageAnalysis? = null
    private var rotation = Surface.ROTATION_0

    @Volatile private var profile: CaptureProfile? = null

    @Volatile private var thermal = 0

    /** Set by the sender when frames pile up; the JPEG quality drops until it clears. */
    @Volatile var backlog = false

    // Capture thread only.
    private var seq = 0
    private var lastFrameNs = 0L
    private var nv21 = ByteArray(0)
    private var square = ByteArray(0)
    private val jpeg = ByteArrayOutputStream(256 * 1024)

    private val orientation = object : OrientationEventListener(context) {
        override fun onOrientationChanged(degrees: Int) {
            if (degrees == ORIENTATION_UNKNOWN) return
            val next = surfaceRotationFor(degrees, rotation)
            if (next != rotation) {
                rotation = next
                analysis?.targetRotation = next
            }
        }
    }

    private val thermalListener = PowerManager.OnThermalStatusChangedListener { thermal = it }

    /** Starts capturing with [profile], or moves to it (other lens, other size) if already running. Any thread. */
    fun start(profile: CaptureProfile) {
        main.post { bind(profile) }
    }

    fun stop() {
        main.post { unbind() }
    }

    private fun bind(profile: CaptureProfile) {
        Log.d(TAG, "Bind ${profile.lens.name.lowercase()} ${profile.width}x${profile.height} at ${profile.fps} fps")
        this.profile = profile
        seq = 0
        val owner = lifecycle ?: ServiceLifecycle().also {
            lifecycle = it
            it.start()
            orientation.enable()
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                thermal = power.currentThermalStatus
                power.addThermalStatusListener(worker, thermalListener)
            }
        }
        val ready = ProcessCameraProvider.getInstance(context)
        ready.addListener({
            if (this.profile !== profile || lifecycle !== owner) return@addListener // Stopped or changed meanwhile.
            Log.d(TAG, "Binding ${profile.lens.name.lowercase()}")
            val provider = ready.get().also { provider = it }
            provider.unbindAll()
            val selector = ResolutionSelector.Builder()
                .setAspectRatioStrategy(
                    AspectRatioStrategy(if (profile.width * 3 == profile.height * 4) AspectRatio.RATIO_4_3 else AspectRatio.RATIO_16_9, AspectRatioStrategy.FALLBACK_RULE_AUTO),
                )
                .setResolutionStrategy(ResolutionStrategy(Size(profile.width, profile.height), ResolutionStrategy.FALLBACK_RULE_CLOSEST_HIGHER_THEN_LOWER))
                .build()
            val analysis = ImageAnalysis.Builder()
                .setBackpressureStrategy(ImageAnalysis.STRATEGY_KEEP_ONLY_LATEST)
                .setOutputImageRotationEnabled(true)
                .setResolutionSelector(selector)
                .setTargetRotation(rotation)
                .build()
            analysis.setAnalyzer(worker, ::analyze)
            try {
                provider.bindToLifecycle(owner, if (profile.lens == Lens.FRONT) CameraSelector.DEFAULT_FRONT_CAMERA else CameraSelector.DEFAULT_BACK_CAMERA, analysis)
                this.analysis = analysis
            } catch (e: IllegalArgumentException) {
                Log.e(TAG, "No usable ${profile.lens.name.lowercase()} camera", e)
            }
        }, mainExecutor)
    }

    /** Safe to call more than once: a session that ends stops the camera, and so does the service. */
    private fun unbind() {
        val owner = lifecycle ?: return
        Log.d(TAG, "Unbind")
        profile = null
        provider?.unbindAll()
        analysis = null
        owner.destroy()
        lifecycle = null
        orientation.disable()
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) power.removeThermalStatusListener(thermalListener)
    }

    private fun analyze(image: ImageProxy) {
        image.use {
            val profile = profile ?: return
            val nowNs = SystemClock.elapsedRealtimeNanos()
            // Frame rate cap: skip frames that come sooner than the wanted interval (with room for jitter).
            if (nowNs - lastFrameNs < 800_000_000L / profile.fps) return
            lastFrameNs = nowNs
            val width = image.width
            val height = image.height
            if (seq == 0) Log.d(TAG, "First frame ${width}x$height, chroma stride ${image.planes[1].rowStride}, pixel stride ${image.planes[1].pixelStride}")
            if (nv21.size != width * height * 3 / 2) nv21 = ByteArray(width * height * 3 / 2)
            val planes = image.planes
            yuv420ToNv21(planes[0].buffer, planes[1].buffer, planes[2].buffer, width, height, planes[0].rowStride, planes[1].rowStride, planes[1].pixelStride, nv21)
            var data = nv21
            var outWidth = width
            var outHeight = height
            if (profile.square) {
                val side = minOf(width, height)
                if (square.size != side * side * 3 / 2) square = ByteArray(side * side * 3 / 2)
                cropNv21Square(nv21, width, height, square)
                data = square
                outWidth = side
                outHeight = side
            }
            jpeg.reset()
            YuvImage(data, ImageFormat.NV21, outWidth, outHeight, null).compressToJpeg(Rect(0, 0, outWidth, outHeight), jpegQualityFor(backlog, thermal), jpeg)
            onFrame(VideoFrame(seq++, nowNs / 1000, jpeg.toByteArray()))
        }
    }

    private companion object {
        const val TAG = "VideoCapture"
    }
}

/**
 * The screen rotation that matches how the phone is held, from the orientation sensor's degrees.
 * A change needs to be at least 20° past the halfway point, so a phone held near a diagonal doesn't flicker.
 */
internal fun surfaceRotationFor(degrees: Int, current: Int): Int {
    val candidate = when (degrees) {
        in 45 until 135 -> Surface.ROTATION_270
        in 135 until 225 -> Surface.ROTATION_180
        in 225 until 315 -> Surface.ROTATION_90
        else -> Surface.ROTATION_0
    }
    if (candidate == current) return current
    val center = when (current) {
        Surface.ROTATION_270 -> 90
        Surface.ROTATION_180 -> 180
        Surface.ROTATION_90 -> 270
        else -> 0
    }
    val distance = minOf((degrees - center + 360) % 360, (center - degrees + 360) % 360)
    return if (distance > 65) candidate else current
}
