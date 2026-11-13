/* A core that records what the bridge did to it.
 *
 * The real core is linked by `bridge_smoke.c`, and that proves bytes really cross. It cannot prove
 * the other half: that every block the bridge took from `core_alloc` went back to `core_free` with
 * *its own size*, that the bytes handed to `core_step` are the string's bytes and all of them, and
 * that each refusal the core can answer is turned into a thrown error. Those need a core whose
 * every call is visible and whose answer can be chosen, which is this one.
 *
 * The size matters more than it looks. `platform/bridges/c/src/lib.rs`'s `core_free` rebuilds a
 * boxed slice from the pointer and the length it is given, so a block freed by the number of bytes
 * written into it rather than by the number asked for is undefined behaviour in release, and
 * silent. The bridge keeps `len` and `room` apart for exactly that reason, and the ledger here is
 * what holds it to it.
 */
#ifndef STREAMS_HARMONY_TEST_FAKE_CORE_H
#define STREAMS_HARMONY_TEST_FAKE_CORE_H

#include <stddef.h>
#include <stdint.h>

#include "core.h"

#ifdef __cplusplus
extern "C" {
#endif

enum { FakeRoom = 4096 };

struct FakeCore {
    /* The ledger. */
    int allocs;
    int frees;
    size_t outstanding;    /* blocks taken and not given back */
    int freed_wrong_size;  /* a block given back by a size it was not asked for */
    int freed_unknown;     /* a pointer given back that was never taken */

    /* What the bridge asked for. */
    int opens;
    int steps;
    int closes;
    char given[FakeRoom]; /* the bytes of the last core_step input */
    size_t given_len;
    char config[FakeRoom]; /* and of the last core_open config */
    size_t config_len;
    CoreHandle *stepped;
    CoreHandle *closed;

    /* What this core answers. */
    int refuse_open;    /* core_open gives no handle */
    int32_t status;     /* what core_step answers */
    int refuse_alloc;   /* core_alloc gives no room */
    char answer[FakeRoom]; /* the bytes core_step hands back on CORE_OK */
    size_t answer_len;
};

/* The one fake core. Reset it between checks. */
struct FakeCore *fake_core(void);
void fake_core_reset(void);

/* What this core will hand back from the next good step. */
void fake_core_answers(const char *bytes, size_t len);

/* The handle a good open gives, so a test can say what crossed. */
CoreHandle *fake_core_handle(void);

#ifdef __cplusplus
}
#endif

#endif /* STREAMS_HARMONY_TEST_FAKE_CORE_H */
