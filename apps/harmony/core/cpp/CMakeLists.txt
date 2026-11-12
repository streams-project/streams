# Links the core's static library into the one dynamic library ArkTS loads. The same shape as
# apps/android/core/src/main/cpp/CMakeLists.txt, with OpenHarmony's CPU names.
#
# CORE_FOLDER is the cargo target folder build-core.sh wrote into. OHOS_ARCH comes from the
# HarmonyOS SDK's own toolchain file.
cmake_minimum_required(VERSION 3.22)
project(streamscore C)

# Stops here rather than guessing: a wrong archive links and then crashes on a device, which is a
# worse hour than a build that says what it wanted. Whether this is the variable the HarmonyOS SDK's
# toolchain file sets is still OPEN QUESTION Q1 -- what the SDK calls the architecture it is building
# for was not read off the reference, only which architectures it will build for at all.
if(NOT OHOS_ARCH)
  message(FATAL_ERROR "OHOS_ARCH is not set, so this cannot tell which core archive to link")
endif()

# Two architectures, and there is no third. Read off the reference on 2026-11-12: the
# `externalNativeOptions.abiFilters` field of `build-profile.json5` takes exactly `arm64-v8a` and
# `x86_64`, and defaults to `arm64-v8a` alone when it is not set. There is NO 32-bit ARM value, so
# the `armeabi-v7a` branch this used to carry could never have been reached, and the
# `armv7-unknown-linux-ohos` core `build-core.sh` used to build had nothing to link into. Both are
# gone. An x86_64 emulator needs `x86_64` named in `abiFilters` explicitly, because the default does
# not include it.
if(OHOS_ARCH STREQUAL "x86_64")
  set(CORE_TARGET x86_64-unknown-linux-ohos)
elseif(OHOS_ARCH STREQUAL "arm64-v8a")
  set(CORE_TARGET aarch64-unknown-linux-ohos)
else()
  message(FATAL_ERROR "OHOS_ARCH is '${OHOS_ARCH}', which is neither arm64-v8a nor x86_64")
endif()

add_library(bridge_c STATIC IMPORTED)
set_target_properties(bridge_c PROPERTIES IMPORTED_LOCATION ${CORE_FOLDER}/${CORE_TARGET}/native/libbridge_c.a)

add_library(streamscore SHARED napi_bridge.c)
target_include_directories(streamscore PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../../../../platform/bridges/c)
# bridge_c first, which is the core itself: without it this library links with undefined references
# to core_abi, core_open, core_step, core_close, core_alloc and core_free, and the runtime would load
# a library with no core in it. libace_napi.z.so carries Node-API on HarmonyOS. dl and m are what the
# Rust standard library needs; the bridge writes no log line of its own, so hilog is not linked.
target_link_libraries(streamscore bridge_c libace_napi.z.so dl m)
