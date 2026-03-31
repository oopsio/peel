#ifndef PEEL_MD5_H
#define PEEL_MD5_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint64_t size;        // Size of input in bytes
    uint32_t buffer[4];   // Current accumulation of hash
    uint8_t input[64];    // Input to be used in the next step
    uint8_t digest[16];   // Result of algorithm
} PEEL_MD5_CTX;

void peel_md5_init(PEEL_MD5_CTX *ctx);
void peel_md5_update(PEEL_MD5_CTX *ctx, const uint8_t *input, size_t input_len);
void peel_md5_final(PEEL_MD5_CTX *ctx);

#endif
