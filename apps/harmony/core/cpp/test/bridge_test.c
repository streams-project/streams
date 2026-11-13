/* Does the Node-API bridge carry bytes the way it says it does?
 *
 * The first test this family has ever had. `README.md` records that nothing here had been compiled
 * and that the C glue had only been syntax checked against a hand written header, which "catches a
 * typo and proves nothing". This runs it: the real `napi_bridge.c`, registered the way the runtime
 * registers it, driven through the stub runtime in `napi_stub.c`, against the recording core in
 * `fake_core.c`.
 *
 * What it proves is the bridge's own arithmetic and its refusals -- how many arguments it counted,
 * how many bytes it asked for, which block it gave back and with what size, and that each thing the
 * core can refuse becomes a thrown error. What it does not prove is that OpenHarmony's runtime
 * behaves like the stub, or that its `napi/native_api.h` declares these calls the way the upstream
 * Node-API headers do; both are stated in `napi/native_api.h` and in `OPEN-QUESTIONS.md` Q1.
 *
 *     cc -std=c11 -Wall -Wextra -I platform/bridges/c -I apps/harmony/core/cpp/test \
 *         apps/harmony/core/cpp/napi_bridge.c apps/harmony/core/cpp/test/napi_stub.c \
 *         apps/harmony/core/cpp/test/fake_core.c apps/harmony/core/cpp/test/bridge_test.c \
 *         -o bridge && ./bridge
 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "fake_core.h"
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

/* Every check starts from a clean runtime, a clean core and a freshly registered module. */
static void begin(void) {
    stub_reset();
    fake_core_reset();
    stub_start();
}

/* The ledger, which has to hold after every one of them. */
static int settled(void) {
    const struct FakeCore *core = fake_core();
    return core->outstanding == 0 && core->allocs == core->frees && core->freed_wrong_size == 0 &&
           core->freed_unknown == 0;
}

int main(void) {
    /* ------------------------------------------------------------- what the runtime is handed */
    ok(stub_module() != NULL,
       "the module registers itself when the library loads, with no call from the runtime");
    if (stub_module() != NULL) {
        ok(stub_module()->nm_version == 1 && stub_module()->nm_flags == 0,
           "at module version 1 and no flags");
        ok(stub_module()->nm_modname != NULL && strcmp(stub_module()->nm_modname, "streamscore") == 0,
           "under the name the ArkTS side imports");
        ok(stub_module()->nm_register_func != NULL && stub_module()->nm_priv == NULL,
           "with a register function and nothing else");
    }

    begin();
    ok(stub_started(), "registering hands the exports object back, which is what a module must do");
    ok(stub_methods() == 4, "and defines four functions on it");
    ok(stub_method("abi") != NULL && stub_method("open") != NULL && stub_method("step") != NULL &&
               stub_method("close") != NULL,
       "named abi, open, step and close, and nothing else is on the boundary");

    /* --------------------------------------------------------------------------------- abi */
    {
        begin();
        napi_value answer = stub_call(stub_method("abi"), NULL, 0);
        ok(stub_is_number(answer) && stub_number_of(answer) == CORE_ABI,
           "abi answers the number the core was built with");
        ok(stub_throws() == 0, "and throws nothing");
    }

    /* -------------------------------------------------------------------------------- open */
    {
        begin();
        napi_value answer = stub_call(stub_method("open"), NULL, 0);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "open takes one config") == 0,
           "open with no config refuses in the viewer's runtime rather than reading an argument "
           "that is not there");
        ok(!stub_is_number(answer), "and answers no handle");
        ok(fake_core()->opens == 0, "and the core is never opened");
    }
    {
        begin();
        const char *config = "{\"type\":\"core_config\"}";
        napi_value args[1];
        args[0] = stub_say(config);
        napi_value answer = stub_call(stub_method("open"), args, 1);
        ok(fake_core()->opens == 1, "a config opens the core once");
        ok(fake_core()->config_len == strlen(config) &&
                   memcmp(fake_core()->config, config, strlen(config)) == 0,
           "and the core is handed the config's own bytes, all of them and no terminator");
        ok(stub_is_number(answer) &&
                   (CoreHandle *)(intptr_t)stub_number_of(answer) == fake_core_handle(),
           "and the handle crosses to ArkTS as the number of its own address");
        ok(settled(), "and the block the config was read into went back with its own size");
        ok(stub_throws() == 0, "and nothing was thrown");
    }
    {
        begin();
        fake_core()->refuse_open = 1;
        napi_value args[1];
        args[0] = stub_say("{}");
        napi_value answer = stub_call(stub_method("open"), args, 1);
        ok(stub_is_number(answer) && stub_number_of(answer) == 0,
           "a core that refuses to open is the number 0, which is what the host checks for");
        ok(stub_throws() == 0,
           "and is not thrown, because which of the two it is, is the host's to say");
        ok(settled(), "and the config was still given back");
    }
    {
        begin();
        stub_fail_sizing_a_string();
        napi_value args[1];
        args[0] = stub_say("{}");
        stub_call(stub_method("open"), args, 1);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "the config did not read") == 0,
           "a config whose length cannot be read is refused");
        ok(fake_core()->allocs == 0 && fake_core()->opens == 0,
           "with nothing taken from the core and nothing opened");
    }
    {
        begin();
        stub_fail_reading_a_string();
        napi_value args[1];
        args[0] = stub_say("{}");
        stub_call(stub_method("open"), args, 1);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "the config did not read") == 0,
           "a config that sizes and then will not read is refused as well");
        ok(fake_core()->allocs == 1 && settled(),
           "and the room taken for it goes back by the size it was asked for, which is the one "
           "free in this file that nothing else would notice");
        ok(fake_core()->opens == 0, "and the core is not opened on a message it never saw");
    }

    /* -------------------------------------------------------------------------------- step */
    {
        begin();
        napi_value args[1];
        args[0] = stub_number(1);
        napi_value answer = stub_call(stub_method("step"), args, 1);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "step takes a handle and a message") == 0,
           "step with a handle and no message is refused");
        ok(!stub_is_text(answer) && fake_core()->steps == 0, "and the core is not stepped");
    }
    {
        /* A message with a character outside ASCII and a zero byte in the middle of it. Both are
         * legal in a JS string, and both are what a copy that stopped at a terminator or counted
         * characters rather than bytes would lose. The bytes are written out rather than typed as
         * letters so that what is being counted is on the line: U+0645 U+0631 (two bytes each) and
         * U+1F3AC (four). */
        const char message[] = {'{', '"', 'a', '"', ':', '"', (char)0xd9, (char)0x85,
                                (char)0xd8, (char)0xb1, (char)0xf0, (char)0x9f, (char)0x8e,
                                (char)0xac, '\0', 'z', '"', '}'};
        const size_t length = sizeof(message);

        begin();
        fake_core_answers("", 0);
        napi_value args[2];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        args[1] = stub_text(message, length);
        stub_call(stub_method("step"), args, 2);
        ok(fake_core()->steps == 1 && fake_core()->stepped == fake_core_handle(),
           "the handle ArkTS held crosses back as the pointer the core gave out");
        ok(fake_core()->given_len == length && length == 18,
           "a message is measured in bytes, so these fourteen characters cross as eighteen bytes "
           "and not as fourteen");
        ok(memcmp(fake_core()->given, message, length) == 0,
           "and every byte of it reaches the core, including the zero one in the middle");
        ok(settled(), "and the room it was read into went back");
    }
    {
        /* And the same in the other direction: an answer is copied out by its length. */
        const char answerBytes[] = {'o', 'k', '\0', (char)0xd8, (char)0xb3, '!'};
        const size_t length = sizeof(answerBytes);

        begin();
        fake_core_answers(answerBytes, length);
        napi_value args[2];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        args[1] = stub_say("{}");
        napi_value answer = stub_call(stub_method("step"), args, 2);
        ok(stub_is_text(answer) && stub_text_len(answer) == length,
           "what the core answers crosses back as a string of exactly as many bytes");
        ok(stub_text_bytes(answer) != NULL &&
                   memcmp(stub_text_bytes(answer), answerBytes, length) == 0,
           "and with the same bytes in it");
        ok(settled(), "and the core's own buffer was given back by the length the core gave");
        ok(stub_throws() == 0, "and nothing was thrown on a step that worked");
    }
    {
        begin();
        fake_core()->status = CORE_BAD_MESSAGE;
        napi_value args[2];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        args[1] = stub_say("{");
        napi_value answer = stub_call(stub_method("step"), args, 2);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "the core refused a step") == 0,
           "a message the core will not read is thrown into ArkTS and not swallowed");
        ok(!stub_is_text(answer), "and no answer is invented for it");
        ok(settled(), "and the message's room is given back on the refusal path too");
    }
    {
        begin();
        fake_core()->status = CORE_PANICKED;
        napi_value args[2];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        args[1] = stub_say("{}");
        stub_call(stub_method("step"), args, 2);
        ok(stub_throws() == 1,
           "a core that has panicked is thrown as well: this file does not decide what a host does "
           "about it");
        ok(settled(), "and still gives the message's room back");
    }
    {
        begin();
        fake_core_answers("{}", 2);
        stub_fail_making_a_string();
        napi_value args[2];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        args[1] = stub_say("{}");
        napi_value answer = stub_call(stub_method("step"), args, 2);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "the answer did not fit") == 0,
           "an answer the runtime will not take is refused");
        ok(!stub_is_text(answer), "and nothing is handed back as if it had");
        ok(settled(),
           "and the core's buffer is freed before the refusal, which is the leak this check exists "
           "for");
    }
    {
        begin();
        fake_core_answers("", 0);
        napi_value args[2];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        args[1] = stub_say("");
        napi_value answer = stub_call(stub_method("step"), args, 2);
        ok(fake_core()->given_len == 0 && stub_is_text(answer) && stub_text_len(answer) == 0,
           "an empty message and an empty answer are carried as themselves and not as a refusal");
        ok(settled(), "and the empty block is still given back by the size it was asked for");
    }

    /* ------------------------------------------------------------------------------- close */
    {
        begin();
        napi_value answer = stub_call(stub_method("close"), NULL, 0);
        ok(stub_throws() == 1 && strcmp(stub_thrown(), "close takes a handle") == 0,
           "close with no handle is refused");
        ok(fake_core()->closes == 0, "and nothing is closed");
        ok(stub_is_undefined(answer) || answer == NULL,
           "and what comes back is nothing, which is all close ever answers");
    }
    {
        begin();
        napi_value args[1];
        args[0] = stub_number((int64_t)(intptr_t)fake_core_handle());
        napi_value answer = stub_call(stub_method("close"), args, 1);
        ok(fake_core()->closes == 1 && fake_core()->closed == fake_core_handle(),
           "a handle closes the core it came from");
        ok(stub_is_undefined(answer), "and answers nothing");
        ok(stub_throws() == 0, "and throws nothing");
    }
    {
        /* A pointer with its high bits set, which is the one thing the handle's comment promises:
         * a number holds whole values to 2^53 and a user pointer is smaller, so nothing is lost. */
        begin();
        const int64_t high = (int64_t)0x00007ffeefbff000;
        napi_value args[1];
        args[0] = stub_number(high);
        stub_call(stub_method("close"), args, 1);
        ok((int64_t)(intptr_t)fake_core()->closed == high,
           "a handle at a high address crosses both ways without losing a bit");
    }
    {
        begin();
        napi_value args[1];
        args[0] = stub_number(0);
        stub_call(stub_method("close"), args, 1);
        ok(fake_core()->closes == 1 && fake_core()->closed == NULL,
           "and a handle of 0 reaches the core as a null pointer, which it is built to take");
    }

    if (failures > 0) {
        printf("%d of %d failed\n", failures, checks);
        return 1;
    }
    printf("%d of %d held\n", checks, checks);
    return 0;
}
