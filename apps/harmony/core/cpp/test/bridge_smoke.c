/* The same bridge, against the real core instead of a recording one.
 *
 * `bridge_test.c` can make the core answer anything, which is what lets it drive every refusal; the
 * price is that nothing it does proves a byte ever reached the core that ships. This does: it links
 * `libbridge_c.a` built from this tree and opens, steps and closes a real core through the four
 * Node-API functions the bridge registers, with a HarmonyOS config.
 *
 * It is the twin of `apps/xbox/host/smoke_test.cpp`, which `README.md` records this family as not
 * having. It proves nothing about HarmonyOS: no SDK, no DevEco and no device are on this machine.
 * What it proves is that the bridge in this folder, as written, carries a config in and a
 * `core_out` back out.
 *
 *     cargo build --locked --profile native -p bridge-c --target <this machine's target>
 *     cc -std=c11 -Wall -Wextra -I platform/bridges/c -I apps/harmony/core/cpp/test \
 *         apps/harmony/core/cpp/napi_bridge.c apps/harmony/core/cpp/test/napi_stub.c \
 *         apps/harmony/core/cpp/test/bridge_smoke.c target/<target>/native/libbridge_c.a \
 *         -o smoke && ./smoke
 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "core.h"
#include "stub.h"

static int checks;
static int failures;

static void ok(int held, const char *what) {
    checks += 1;
    printf("%s %s\n", held ? "ok  " : "FAIL", what);
    if (!held) {
        failures += 1;
    }
}

/* The contract version the core in this tree speaks. A mismatch fails the first step, which is the
 * point: the host carries one version and the core refuses anything else. */
static const char *const WIRE = "2026-11-05";

/* A Huawei phone, as `core/Host.ets:322-336` builds it: app and os `harmony`, and the class and the
 * form the wiring hands in. `class` decides the floors the server gives this device and `form`
 * decides what it draws, so the pair is the one thing a host must get right before any screen
 * exists. */
static const char *const CONFIG =
        "{\"app\":\"harmony\",\"class\":\"phone\",\"contract\":\"2026-11-05\",\"form\":\"phone\","
        "\"mode\":\"local\",\"os\":\"harmony\","
        "\"seed\":\"00112233445566778899aabbccddeeff\","
        "\"type\":\"core_config\",\"version\":\"0.0.0\"}";

static napi_value call(const char *name, napi_value *args, size_t count) {
    napi_callback method = stub_method(name);
    if (method == NULL) {
        return NULL;
    }
    return stub_call(method, args, count);
}

int main(void) {
    stub_reset();
    stub_start();
    ok(stub_started() && stub_methods() == 4, "the module registers its four functions");

    {
        napi_value answer = call("abi", NULL, 0);
        ok(stub_is_number(answer) && stub_number_of(answer) == CORE_ABI,
           "the linked core carries the ABI this bridge was built for");
    }

    ok(strstr(CONFIG, WIRE) != NULL, "the config names the wire this test holds the answer to");

    napi_value handle = NULL;
    {
        napi_value args[1];
        args[0] = stub_say("not json");
        napi_value answer = call("open", args, 1);
        ok(stub_is_number(answer) && stub_number_of(answer) == 0,
           "a config that does not read gives no handle, and the host sees 0");
    }
    {
        napi_value args[1];
        args[0] = stub_say(CONFIG);
        handle = call("open", args, 1);
        ok(stub_is_number(handle) && stub_number_of(handle) != 0,
           "a phone's config opens a real core");
    }

    {
        napi_value args[2];
        args[0] = handle;
        args[1] = stub_say("{\"contract\":\"2026-11-05\",\"events\":[],\"mono_ms\":0,"
                           "\"now_ms\":1000,\"type\":\"core_in\"}");
        napi_value answer = call("step", args, 2);
        ok(stub_is_text(answer), "the first step runs and answers text");
        const char *head = "{\"contract\":\"2026-11-05\",\"type\":\"core_out\"";
        ok(stub_text_bytes(answer) != NULL && stub_text_len(answer) > strlen(head) &&
                   strncmp(stub_text_bytes(answer), head, strlen(head)) == 0,
           "and the text is a core_out message on the wire this bridge speaks");
    }
    {
        int before = stub_throws();
        napi_value args[2];
        args[0] = handle;
        args[1] = stub_say("{");
        napi_value answer = call("step", args, 2);
        ok(stub_throws() == before + 1,
           "a message the core cannot read is thrown rather than answered");
        ok(!stub_is_text(answer), "and nothing is handed back in its place");
    }
    {
        napi_value args[2];
        args[0] = handle;
        args[1] = stub_say("{\"contract\":\"2026-11-05\",\"events\":[],\"mono_ms\":5,"
                           "\"now_ms\":1005,\"type\":\"core_in\"}");
        napi_value answer = call("step", args, 2);
        ok(stub_is_text(answer), "and the core carries on after a refused message");
    }
    {
        napi_value args[1];
        args[0] = handle;
        napi_value answer = call("close", args, 1);
        ok(stub_is_undefined(answer), "closing answers nothing");
    }

    if (failures > 0) {
        printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    printf("%d of %d held\n", checks, checks);
    return 0;
}
