#ifndef PEEL_SHA512_H
#define PEEL_SHA512_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
    uint8_t  data[128];
    uint32_t datalen;
    uint64_t bitlen[2];
    uint64_t state[8];
} PEEL_SHA512_CTX;

void peel_sha512_init(PEEL_SHA512_CTX *ctx);
void peel_sha512_update(PEEL_SHA512_CTX *ctx, const uint8_t data[], size_t len);
void peel_sha512_final(PEEL_SHA512_CTX *ctx, uint8_t hash[]);

#endif
