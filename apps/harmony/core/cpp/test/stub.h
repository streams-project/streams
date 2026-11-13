/* What the test drives the bridge with: a runtime that answers the Node-API calls
 * `napi_bridge.c` makes, and a ledger of what it was asked.
 *
 * This is not an emulator of ArkTS and it does not pretend to be one. It is the smallest runtime
 * that can call the four functions the bridge registers and look at what came back, so the bridge's
 * own arithmetic -- how many arguments it counted, how many bytes it asked for, which block it gave
 * back and how big it said it was, whether it threw -- is run rather than read.
 *
 * What it cannot prove is in `napi/native_api.h`: that OpenHarmony's header declares these calls the
 * way the upstream Node-API headers do.
 */
#ifndef STREAMS_HARMONY_TEST_STUB_H
#define STREAMS_HARMONY_TEST_STUB_H

#include <stddef.h>
#include <stdint.h>

#include "napi/native_api.h"

#ifdef __cplusplus
extern "C" {
#endif

/* Forgets every value, every thrown error and every knob. Called between checks. */
void stub_reset(void);

/* The one environment. */
napi_env stub_env(void);

/* A string of exactly `len` bytes, which may hold a zero byte: a JS string can, and the bridge
 * must carry one by length rather than by looking for its end. */
napi_value stub_text(const char *bytes, size_t len);

/* A string from a C literal. */
napi_value stub_say(const char *text);

/* A number, which is how a handle crosses. */
napi_value stub_number(int64_t value);

/* What came back. */
int stub_is_undefined(napi_value value);
int stub_is_text(napi_value value);
int stub_is_number(napi_value value);
size_t stub_text_len(napi_value value);
const char *stub_text_bytes(napi_value value);
int64_t stub_number_of(napi_value value);

/* How many errors were thrown since the last reset, and what the last one said. */
int stub_throws(void);
const char *stub_thrown(void);

/* Knobs. Each one makes one Node-API call refuse once, so the bridge's unhappy paths are run and
 * not assumed. */
void stub_fail_sizing_a_string(void);   /* the first napi_get_value_string_utf8 refuses */
void stub_fail_reading_a_string(void);  /* the sizing call holds and the reading call refuses */
void stub_fail_making_a_string(void);   /* napi_create_string_utf8 refuses */

/* The module the bridge registered from its constructor, and the four functions it defined. */
napi_module *stub_module(void);
int stub_started(void);

/* Runs the module's register function against a fresh exports object. */
void stub_start(void);

/* One of the registered functions by the name it was registered under, or NULL. */
napi_callback stub_method(const char *name);

/* How many functions were registered. */
size_t stub_methods(void);

/* Calls a registered function with these arguments. */
napi_value stub_call(napi_callback method, napi_value *args, size_t count);

#ifdef __cplusplus
}
#endif

#endif /* STREAMS_HARMONY_TEST_STUB_H */
