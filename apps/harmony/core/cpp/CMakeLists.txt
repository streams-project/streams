# Links the core's static library into the one dynamic library ArkTS loads. The same shape as
# apps/android/core/src/main/cpp/CMakeLists.txt, with OpenHarmony's CPU names.
#
# CORE_FOLDER is the cargo target folder build-core.sh wrote into. OHOS_ARCH comes from the
# HarmonyOS SDK's own toolchain file.
cmake_minimum_required(VERSION 3.22)
project(streamscore C)

# Stops here rather than guessing: a wrong archive links and then crashes on a device, which is a
# worse hour than a build that says what it wanted. Whether this is the variable the HarmonyOS SDK's
# toolchain file sets is OPEN QUESTION Q1.
if(NOT OHOS_ARCH)
  message(FATAL_ERROR "OHOS_ARCH is not set, so this cannot tell which core archive to link")
endif()

if(OHOS_ARCH STREQUAL "armeabi-v7a")
  set(CORE_TARGET armv7-unknown-linux-ohos)
elseif(OHOS_ARCH STREQUAL "x86_64")
  set(CORE_TARGET x86_64-unknown-linux-ohos)
else()
  set(CORE_TARGET aarch64-unknown-linux-ohos)
endif()

add_library(bridge_c STATIC IMPORTED)
set_target_properties(bridge_c PROPERTIES IMPORTED_LOCATION ${CORE_FOLDER}/${CORE_TARGET}/native/libbridge_c.a)

add_library(streamscore SHARED napi_bridge.c)
target_include_directories(streamscore PRIVATE ${CMAKE_CURRENT_SOURCE_DIR}/../../../../platform/bridges/c)
# libace_napi.z.so carries Node-API on HarmonyOS. dl and m are what the Rust standard library
# needs; the bridge writes no log line of its own, so hilog is not linked.
target_link_libraries(streamscore libace_napi.z.so dl m)
