#ifndef PEEL_CRYPTO_H
#define PEEL_CRYPTO_H

#include <stddef.h>

char* peel_sha256(const char* data);
char* peel_sha512(const char* data);
char* peel_md5(const char* data);
char* peel_hmac_sha256(const char* data, const char* key);
char* peel_aes_256_cbc_encrypt(const char* data, const char* key, const char* iv);
char* peel_aes_256_cbc_decrypt(const char* ciphertext, const char* key, const char* iv);
void peel_crypto_free(char* ptr);

#endif
