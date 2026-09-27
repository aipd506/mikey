// JNI side of com.mikey.media.OpusEncoder. The handle is the OpusEncoder pointer as a jlong.
#include <jni.h>
#include <stdint.h>
#include <opus.h>

JNIEXPORT jlong JNICALL Java_com_mikey_media_OpusEncoder_create(
        JNIEnv *env, jclass clazz, jint sampleRate, jint channels, jint application, jint bitrate) {
    int error = OPUS_OK;
    OpusEncoder *encoder = opus_encoder_create(sampleRate, channels, application, &error);
    if (error != OPUS_OK || encoder == NULL) return 0;
    opus_encoder_ctl(encoder, OPUS_SET_BITRATE(bitrate));
    return (jlong) (intptr_t) encoder;
}

// pcm holds s16le samples; every Android ABI is little-endian, so it can be read as opus_int16 as is.
// Returns the packet length, or a negative Opus error code.
JNIEXPORT jint JNICALL Java_com_mikey_media_OpusEncoder_encode(
        JNIEnv *env, jclass clazz, jlong handle, jbyteArray pcm, jint frameSamples, jbyteArray out) {
    jbyte *samples = (*env)->GetByteArrayElements(env, pcm, NULL);
    jbyte *packet = (*env)->GetByteArrayElements(env, out, NULL);
    jsize max = (*env)->GetArrayLength(env, out);
    int length = opus_encode((OpusEncoder *) (intptr_t) handle, (const opus_int16 *) samples, frameSamples,
                             (unsigned char *) packet, max);
    (*env)->ReleaseByteArrayElements(env, pcm, samples, JNI_ABORT);
    (*env)->ReleaseByteArrayElements(env, out, packet, 0);
    return length;
}

JNIEXPORT void JNICALL Java_com_mikey_media_OpusEncoder_destroy(JNIEnv *env, jclass clazz, jlong handle) {
    opus_encoder_destroy((OpusEncoder *) (intptr_t) handle);
}
