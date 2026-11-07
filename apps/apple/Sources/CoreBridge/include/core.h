/* The six functions of the core. They carry UTF-8 JSON bytes and hold no rules.
 */
#ifndef CORE_H
#define CORE_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

#define CORE_ABI 1

#define CORE_OK 0
#define CORE_BAD_MESSAGE 1
#define CORE_PANICKED 2
#define CORE_BAD_CALL 3

typedef struct CoreHandle CoreHandle;

uint32_t core_abi(void);
uint8_t *core_alloc(size_t len);
void core_free(uint8_t *ptr, size_t len);
CoreHandle *core_open(const uint8_t *cfg_ptr, size_t cfg_len);
int32_t core_step(CoreHandle *handle, const uint8_t *in_ptr, size_t in_len, uint8_t **out_ptr_out, size_t *out_len_out);
void core_close(CoreHandle *handle);

#ifdef __cplusplus
}
#endif

#endif
