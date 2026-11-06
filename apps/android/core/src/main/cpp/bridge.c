/* The core's six functions, as Java sees them. Bytes go in and bytes come out, and no rules live here. */
#include <jni.h>
#include <stdint.h>
#include "core.h"

JNIEXPORT jint JNICALL Java_streams_core_NativeCore_abi(JNIEnv *env, jclass cls) {
    return (jint)core_abi();
}

JNIEXPORT jlong JNICALL Java_streams_core_NativeCore_open(JNIEnv *env, jclass cls, jbyteArray config) {
    jsize len = (*env)->GetArrayLength(env, config);
    jbyte *bytes = (*env)->GetByteArrayElements(env, config, NULL);
    CoreHandle *handle = core_open((const uint8_t *)bytes, (size_t)len);
    (*env)->ReleaseByteArrayElements(env, config, bytes, JNI_ABORT);
    return (jlong)(intptr_t)handle;
}

JNIEXPORT jbyteArray JNICALL Java_streams_core_NativeCore_step(JNIEnv *env, jclass cls, jlong handle, jbyteArray input) {
    jsize len = (*env)->GetArrayLength(env, input);
    jbyte *bytes = (*env)->GetByteArrayElements(env, input, NULL);
    uint8_t *out = NULL;
    size_t out_len = 0;
    int32_t status = core_step((CoreHandle *)(intptr_t)handle, (const uint8_t *)bytes, (size_t)len, &out, &out_len);
    (*env)->ReleaseByteArrayElements(env, input, bytes, JNI_ABORT);
    if (status != CORE_OK) {
        jclass thrown = (*env)->FindClass(env, "java/lang/IllegalStateException");
        (*env)->ThrowNew(env, thrown, "the core refused a step");
        return NULL;
    }
    jbyteArray answer = (*env)->NewByteArray(env, (jsize)out_len);
    (*env)->SetByteArrayRegion(env, answer, 0, (jsize)out_len, (const jbyte *)out);
    core_free(out, out_len);
    return answer;
}

JNIEXPORT void JNICALL Java_streams_core_NativeCore_close(JNIEnv *env, jclass cls, jlong handle) {
    core_close((CoreHandle *)(intptr_t)handle);
}
