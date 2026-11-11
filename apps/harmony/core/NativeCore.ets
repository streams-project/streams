// The core's four functions, through core/cpp/napi_bridge.c. The same declaration as
// apps/android/core/src/main/kotlin/streams/core/NativeCore.kt, which reaches the same C through JNI.
//
// A handle is used from one thread at a time, and a HarmonyOS Worker has a VM of its own, so the
// module and every handle opened from it live on whichever thread imported this: OPEN QUESTION Q3.
//
// NOT COMPILED. The import name and what DevEco needs to resolve it are OPEN QUESTION Q1; the types
// it reads are in core/cpp/types/libstreamscore/index.d.ts.
import { abi, close, open, step } from 'libstreamscore.so';

export { abi, close, open, step };
