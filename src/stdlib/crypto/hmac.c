#include "hmac.h"
#include <string.h>

void peel_hmac_sha256_internal(const uint8_t *key, size_t key_len, const uint8_t *data, size_t data_len, uint8_t *hmac) {
    PEEL_SHA256_CTX ctx;
    uint8_t k_ipad[64], k_opad[64], tk[32];
    int i;

    if (key_len > 64) {
        peel_sha256_init(&ctx);
        peel_sha256_update(&ctx, key, key_len);
        peel_sha256_final(&ctx, tk);
        key = tk;
        key_len = 32;
    }

    memset(k_ipad, 0, sizeof(k_ipad));
    memset(k_opad, 0, sizeof(k_opad));
    memcpy(k_ipad, key, key_len);
    memcpy(k_opad, key, key_len);

    for (i = 0; i < 64; i++) {
        k_ipad[i] ^= 0x36;
        k_opad[i] ^= 0x5c;
    }

    peel_sha256_init(&ctx);
    peel_sha256_update(&ctx, k_ipad, 64);
    peel_sha256_update(&ctx, data, data_len);
    peel_sha256_final(&ctx, hmac);

    peel_sha256_init(&ctx);
    peel_sha256_update(&ctx, k_opad, 64);
    peel_sha256_update(&ctx, hmac, 32);
    peel_sha256_final(&ctx, hmac);
}
