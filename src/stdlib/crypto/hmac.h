#ifndef PEEL_HMAC_H
#define PEEL_HMAC_H

#include <stddef.h>
#include <stdint.h>
#include "sha256.h"

void peel_hmac_sha256_internal(const uint8_t *key, size_t key_len, const uint8_t *data, size_t data_len, uint8_t *hmac);

#endif
