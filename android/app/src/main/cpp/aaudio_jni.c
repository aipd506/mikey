// JNI side of com.mikey.media.AAudioInput. The handle is the AAudioStream pointer as a jlong.
#include <jni.h>
#include <stdint.h>
#include <aaudio/AAudio.h>

// Opens and starts a low-latency mono 16-bit input stream at sampleRate, or returns 0 when the
// device won't give exactly that (then AudioRecord takes over on the Kotlin side).
JNIEXPORT jlong JNICALL Java_com_mikey_media_AAudioInput_nativeOpen(JNIEnv *env, jclass clazz, jint sampleRate) {
    AAudioStreamBuilder *builder = NULL;
    if (AAudio_createStreamBuilder(&builder) != AAUDIO_OK) return 0;
    AAudioStreamBuilder_setDirection(builder, AAUDIO_DIRECTION_INPUT);
    AAudioStreamBuilder_setSharingMode(builder, AAUDIO_SHARING_MODE_SHARED);
    AAudioStreamBuilder_setPerformanceMode(builder, AAUDIO_PERFORMANCE_MODE_LOW_LATENCY);
    AAudioStreamBuilder_setSampleRate(builder, sampleRate);
    AAudioStreamBuilder_setChannelCount(builder, 1);
    AAudioStreamBuilder_setFormat(builder, AAUDIO_FORMAT_PCM_I16);
    if (__builtin_available(android 28, *)) {
        // Raw capture: no phone-side noise suppression or gain.
        AAudioStreamBuilder_setInputPreset(builder, AAUDIO_INPUT_PRESET_UNPROCESSED);
    }
    AAudioStream *stream = NULL;
    aaudio_result_t result = AAudioStreamBuilder_openStream(builder, &stream);
    if (result != AAUDIO_OK) {
        if (__builtin_available(android 26, *)) {
            // Fallback to VOICE_RECOGNITION if UNPROCESSED is unsupported by device HAL (disables AGC/ducking).
            AAudioStreamBuilder_setInputPreset(builder, AAUDIO_INPUT_PRESET_VOICE_RECOGNITION);
            result = AAudioStreamBuilder_openStream(builder, &stream);
        }
    }
    AAudioStreamBuilder_delete(builder);
    if (result != AAUDIO_OK) return 0;
    if (AAudioStream_getSampleRate(stream) != sampleRate || AAudioStream_getFormat(stream) != AAUDIO_FORMAT_PCM_I16 ||
        AAudioStream_getChannelCount(stream) != 1 || AAudioStream_requestStart(stream) != AAUDIO_OK) {
        AAudioStream_close(stream);
        return 0;
    }
    return (jlong) (intptr_t) stream;
}

// Blocks until frames samples are read or timeoutNs passes. Returns the samples read, or a negative AAudio error.
JNIEXPORT jint JNICALL Java_com_mikey_media_AAudioInput_nativeRead(
        JNIEnv *env, jclass clazz, jlong handle, jbyteArray pcm, jint frames, jlong timeoutNs) {
    jbyte *buffer = (*env)->GetByteArrayElements(env, pcm, NULL);
    aaudio_result_t read = AAudioStream_read((AAudioStream *) (intptr_t) handle, buffer, frames, timeoutNs);
    (*env)->ReleaseByteArrayElements(env, pcm, buffer, 0);
    return read;
}

JNIEXPORT void JNICALL Java_com_mikey_media_AAudioInput_nativeClose(JNIEnv *env, jclass clazz, jlong handle) {
    AAudioStream *stream = (AAudioStream *) (intptr_t) handle;
    AAudioStream_requestStop(stream);
    AAudioStream_close(stream);
}
