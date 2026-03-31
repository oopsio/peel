#ifndef PEEL_SHA256_H
#define PEEL_SHA256_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t  data[64];
    uint32_t datalen;
    uint64_t bitlen;
    uint32_t state[8];
} PEEL_SHA256_CTX;

void peel_sha256_init(PEEL_SHA256_CTX *ctx);
void peel_sha256_update(PEEL_SHA256_CTX *ctx, const uint8_t data[], size_t len);
void peel_sha256_final(PEEL_SHA256_CTX *ctx, uint8_t hash[]);

#endif
