#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include "crypto.h"
#include "sha256.h"
#include "sha512.h"
#include "md5.h"
#include "hmac.h"
#include "aes.h"

// Helper to convert hash to hex string
static char* bytes_to_hex(const unsigned char* hash, size_t len) {
    char* hex = (char*)malloc(len * 2 + 1);
    if (!hex) return NULL;
    for (size_t i = 0; i < len; i++) {
        sprintf(hex + i * 2, "%02x", hash[i]);
    }
    hex[len * 2] = '\0';
    return hex;
}

// Simple Base64 encoding for AES output
static const char b64chars[] = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
static char* base64_encode(const unsigned char* data, size_t input_length) {
    size_t output_length = 4 * ((input_length + 2) / 3);
    char* encoded_data = (char*)malloc(output_length + 1);
    if (!encoded_data) return NULL;

    for (size_t i = 0, j = 0; i < input_length;) {
        uint32_t octet_a = i < input_length ? (unsigned char)data[i++] : 0;
        uint32_t octet_b = i < input_length ? (unsigned char)data[i++] : 0;
        uint32_t octet_c = i < input_length ? (unsigned char)data[i++] : 0;
        uint32_t triple = (octet_a << 0x10) + (octet_b << 0x08) + octet_c;
        encoded_data[j++] = b64chars[(triple >> 3 * 6) & 0x3F];
        encoded_data[j++] = b64chars[(triple >> 2 * 6) & 0x3F];
        encoded_data[j++] = b64chars[(triple >> 1 * 6) & 0x3F];
        encoded_data[j++] = b64chars[(triple >> 0 * 6) & 0x3F];
    }
    for (size_t i = 0; i < (3 - input_length % 3) % 3; i++) encoded_data[output_length - 1 - i] = '=';
    encoded_data[output_length] = '\0';
    return encoded_data;
}

// Simple Base64 decoding for AES input
static unsigned char* base64_decode(const char* data, size_t* output_length) {
    size_t input_length = strlen(data);
    if (input_length % 4 != 0) return NULL;
    *output_length = (input_length / 4) * 3;
    if (data[input_length - 1] == '=') (*output_length)--;
    if (data[input_length - 2] == '=') (*output_length)--;
    unsigned char* decoded_data = (unsigned char*)malloc(*output_length);
    if (!decoded_data) return NULL;

    static char decoding_table[256];
    static int table_init = 0;
    if (!table_init) {
        for (int i = 0; i < 64; i++) decoding_table[(unsigned char)b64chars[i]] = i;
        table_init = 1;
    }

    for (size_t i = 0, j = 0; i < input_length;) {
        uint32_t sextet_a = data[i] == '=' ? 0 : decoding_table[(unsigned char)data[i]]; i++;
        uint32_t sextet_b = data[i] == '=' ? 0 : decoding_table[(unsigned char)data[i]]; i++;
        uint32_t sextet_c = data[i] == '=' ? 0 : decoding_table[(unsigned char)data[i]]; i++;
        uint32_t sextet_d = data[i] == '=' ? 0 : decoding_table[(unsigned char)data[i]]; i++;
        uint32_t triple = (sextet_a << 3 * 6) + (sextet_b << 2 * 6) + (sextet_c << 1 * 6) + sextet_d;
        if (j < *output_length) decoded_data[j++] = (triple >> 2 * 8) & 0xFF;
        if (j < *output_length) decoded_data[j++] = (triple >> 1 * 8) & 0xFF;
        if (j < *output_length) decoded_data[j++] = (triple >> 0 * 8) & 0xFF;
    }
    return decoded_data;
}

char* peel_sha256(const char* data) {
    PEEL_SHA256_CTX ctx;
    uint8_t hash[32];
    peel_sha256_init(&ctx);
    peel_sha256_update(&ctx, (const uint8_t*)data, strlen(data));
    peel_sha256_final(&ctx, hash);
    return bytes_to_hex(hash, 32);
}

char* peel_sha512(const char* data) {
    PEEL_SHA512_CTX ctx;
    uint8_t hash[64];
    peel_sha512_init(&ctx);
    peel_sha512_update(&ctx, (const uint8_t*)data, strlen(data));
    peel_sha512_final(&ctx, hash);
    return bytes_to_hex(hash, 64);
}

char* peel_md5(const char* data) {
    PEEL_MD5_CTX ctx;
    peel_md5_init(&ctx);
    peel_md5_update(&ctx, (const uint8_t*)data, strlen(data));
    peel_md5_final(&ctx);
    return bytes_to_hex(ctx.digest, 16);
}

char* peel_hmac_sha256(const char* data, const char* key) {
    uint8_t hash[32];
    peel_hmac_sha256_internal((const uint8_t*)key, strlen(key), (const uint8_t*)data, strlen(data), hash);
    return bytes_to_hex(hash, 32);
}

char* peel_aes_256_cbc_encrypt(const char* data, const char* key, const char* iv) {
    struct AES_ctx ctx;
    size_t data_len = strlen(data);
    size_t padded_len = ((data_len + 15) / 16) * 16;
    uint8_t* buffer = (uint8_t*)calloc(1, padded_len);
    if (!buffer) return NULL;
    memcpy(buffer, data, data_len);
    // Simple PKCS7 padding
    uint8_t pad_val = padded_len - data_len;
    for (size_t i = data_len; i < padded_len; i++) buffer[i] = pad_val;

    AES_init_ctx_iv(&ctx, (const uint8_t*)key, (const uint8_t*)iv);
    AES_CBC_encrypt_buffer(&ctx, buffer, padded_len);
    char* b64 = base64_encode(buffer, padded_len);
    free(buffer);
    return b64;
}

char* peel_aes_256_cbc_decrypt(const char* ciphertext_b64, const char* key, const char* iv) {
    struct AES_ctx ctx;
    size_t dec_len;
    uint8_t* buffer = base64_decode(ciphertext_b64, &dec_len);
    if (!buffer) return NULL;

    AES_init_ctx_iv(&ctx, (const uint8_t*)key, (const uint8_t*)iv);
    AES_CBC_decrypt_buffer(&ctx, buffer, dec_len);
    
    // Remove padding
    uint8_t pad_val = buffer[dec_len - 1];
    if (pad_val > 16) pad_val = 0; // Invalid padding
    size_t plain_len = dec_len - pad_val;
    char* res = (char*)malloc(plain_len + 1);
    if (!res) { free(buffer); return NULL; }
    memcpy(res, buffer, plain_len);
    res[plain_len] = '\0';
    free(buffer);
    return res;
}

void peel_crypto_free(char* ptr) {
    if (ptr) free(ptr);
}
