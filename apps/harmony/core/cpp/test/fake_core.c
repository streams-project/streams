#include "fake_core.h"

#include <stdlib.h>
#include <string.h>

enum { Blocks = 64 };

static struct FakeCore core;

/* Every block taken from core_alloc and the size it was asked for. */
static struct {
    uint8_t *at;
    size_t len;
} taken[Blocks];
static int takenCount;

/* A handle is a pointer the host never reads. This one is a real address so that the number it
 * crosses as is a plausible one, and so that nothing compares it against a small integer by luck. */
static int theCore;

struct FakeCore *fake_core(void) { return &core; }

CoreHandle *fake_core_handle(void) { return (CoreHandle *)&theCore; }

void fake_core_reset(void) {
    for (int at = 0; at < takenCount; at++) {
        free(taken[at].at);
    }
    takenCount = 0;
    memset(&core, 0, sizeof(core));
    core.status = CORE_OK;
}

void fake_core_answers(const char *bytes, size_t len) {
    if (len > FakeRoom) {
        len = FakeRoom;
    }
    memcpy(core.answer, bytes, len);
    core.answer_len = len;
}

uint32_t core_abi(void) { return CORE_ABI; }

uint8_t *core_alloc(size_t len) {
    if (core.refuse_alloc || takenCount >= Blocks) {
        return NULL;
    }
    uint8_t *room = (uint8_t *)malloc(len == 0 ? 1 : len);
    if (room == NULL) {
        return NULL;
    }
    taken[takenCount].at = room;
    taken[takenCount].len = len;
    takenCount += 1;
    core.allocs += 1;
    core.outstanding += 1;
    return room;
}

void core_free(uint8_t *ptr, size_t len) {
    if (ptr == NULL) {
        return;
    }
    core.frees += 1;
    for (int at = 0; at < takenCount; at++) {
        if (taken[at].at != ptr) {
            continue;
        }
        if (taken[at].len != len) {
            /* The quiet one: `core_free` in the real bridge rebuilds a boxed slice of this length. */
            core.freed_wrong_size += 1;
        }
        free(ptr);
        taken[at] = taken[takenCount - 1];
        takenCount -= 1;
        core.outstanding -= 1;
        return;
    }
    core.freed_unknown += 1;
}

CoreHandle *core_open(const uint8_t *cfg_ptr, size_t cfg_len) {
    core.opens += 1;
    core.config_len = cfg_len > FakeRoom ? FakeRoom : cfg_len;
    if (cfg_ptr != NULL && core.config_len > 0) {
        memcpy(core.config, cfg_ptr, core.config_len);
    }
    return core.refuse_open ? NULL : fake_core_handle();
}

int32_t core_step(CoreHandle *handle, const uint8_t *in_ptr, size_t in_len, uint8_t **out_ptr_out,
                  size_t *out_len_out) {
    core.stepped = handle;
    core.steps += 1;
    core.given_len = in_len > FakeRoom ? FakeRoom : in_len;
    if (in_ptr != NULL && core.given_len > 0) {
        memcpy(core.given, in_ptr, core.given_len);
    }
    if (core.status != CORE_OK) {
        return core.status;
    }
    uint8_t *out = core_alloc(core.answer_len);
    if (out == NULL) {
        return CORE_PANICKED;
    }
    memcpy(out, core.answer, core.answer_len);
    *out_ptr_out = out;
    *out_len_out = core.answer_len;
    return CORE_OK;
}

void core_close(CoreHandle *handle) {
    core.closes += 1;
    core.closed = handle;
}
