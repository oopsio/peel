// Standalone MD5 implementation.
#include "md5.h"
#include <string.h>

#define F(x, y, z) (((x) & (y)) | (~(x) & (z)))
#define G(x, y, z) (((x) & (z)) | ((y) & (~(z))))
#define H(x, y, z) ((x) ^ (y) ^ (z))
#define I(x, y, z) ((y) ^ ((x) | (~(z))))

#define ROT(x, n) (((x) << (n)) | ((x) >> (32 - (n))))

#define STEP(f, a, b, c, d, x, s, t) \
    (a) += f((b), (c), (d)) + (x) + (t); \
    (a) = ROT((a), (s)); \
    (a) += (b);

static const uint32_t T[] = {
    0xd76aa478, 0xe8c7b756, 0x242070db, 0xc1bdceee, 0xf57c0faf, 0x4787c62a, 0xa8304613, 0xfd469501,
    0x698098d8, 0x8b44f7af, 0xffff5bb1, 0x895cd7be, 0x6b901122, 0xfd987193, 0xa679438e, 0x49b40821,
    0xf61e2562, 0xc040b340, 0x265e5a51, 0xe9b6c7aa, 0xd62f105d, 0x02441453, 0xd8a1e681, 0xe7d3fbc8,
    0x21e1cde6, 0xc33707d6, 0xf4d50d87, 0x455a14ed, 0xa9e3e905, 0xfcefa3f8, 0x676f02d9, 0x8d2a4c8a,
    0xfffa3942, 0x8771f681, 0x6d9d6122, 0xfde5380c, 0xa4beea44, 0x4bdecfa9, 0xf6bb4b60, 0xbebfbc70,
    0x289b7ec6, 0xeaa127fa, 0xd4ef3085, 0x04881d05, 0xd9d4d039, 0xe6db99e5, 0x1fa27cf8, 0xc4ac5665,
    0xf4292244, 0x432aff97, 0xab9423a7, 0xfc93a039, 0x655b59c3, 0x8f0ccc92, 0xffeff47d, 0x85845dd1,
    0x6fa87e4f, 0xfe2ce6e0, 0xa3014314, 0x4e0811a1, 0xf7537e82, 0xbd3af235, 0x2ad7d2bb, 0xeb86d391
};

static void md5_transform(uint32_t state[4], const uint8_t *block) {
    uint32_t a = state[0], b = state[1], c = state[2], d = state[3], x[16];
    for (int i = 0; i < 16; i++) x[i] = ((uint32_t)block[i*4+3] << 24) | ((uint32_t)block[i*4+2] << 16) | ((uint32_t)block[i*4+1] << 8) | (uint32_t)block[i*4];

    STEP(F, a, b, c, d, x[0], 7, T[0])  STEP(F, d, a, b, c, x[1], 12, T[1]) STEP(F, c, d, a, b, x[2], 17, T[2]) STEP(F, b, c, d, a, x[3], 22, T[3])
    STEP(F, a, b, c, d, x[4], 7, T[4])  STEP(F, d, a, b, c, x[5], 12, T[5]) STEP(F, c, d, a, b, x[6], 17, T[6]) STEP(F, b, c, d, a, x[7], 22, T[7])
    STEP(F, a, b, c, d, x[8], 7, T[8])  STEP(F, d, a, b, c, x[9], 12, T[9]) STEP(F, c, d, a, b, x[10], 17, T[10]) STEP(F, b, c, d, a, x[11], 22, T[11])
    STEP(F, a, b, c, d, x[12], 7, T[12]) STEP(F, d, a, b, c, x[13], 12, T[13]) STEP(F, c, d, a, b, x[14], 17, T[14]) STEP(F, b, c, d, a, x[15], 22, T[15])

    STEP(G, a, b, c, d, x[1], 5, T[16]) STEP(G, d, a, b, c, x[6], 9, T[17]) STEP(G, c, d, a, b, x[11], 14, T[18]) STEP(G, b, c, d, a, x[0], 20, T[19])
    STEP(G, a, b, c, d, x[5], 5, T[20]) STEP(G, d, a, b, c, x[10], 9, T[21]) STEP(G, c, d, a, b, x[15], 14, T[22]) STEP(G, b, c, d, a, x[4], 20, T[23])
    STEP(G, a, b, c, d, x[9], 5, T[24]) STEP(G, d, a, b, c, x[14], 9, T[25]) STEP(G, c, d, a, b, x[3], 14, T[26]) STEP(G, b, c, d, a, x[8], 20, T[27])
    STEP(G, a, b, c, d, x[13], 5, T[28]) STEP(G, d, a, b, c, x[2], 9, T[29]) STEP(G, c, d, a, b, x[7], 14, T[30]) STEP(G, b, c, d, a, x[12], 20, T[31])

    STEP(H, a, b, c, d, x[5], 4, T[32]) STEP(H, d, a, b, c, x[8], 11, T[33]) STEP(H, c, d, a, b, x[11], 16, T[34]) STEP(H, b, c, d, a, x[14], 23, T[35])
    STEP(H, a, b, c, d, x[1], 4, T[36]) STEP(H, d, a, b, c, x[4], 11, T[37]) STEP(H, c, d, a, b, x[7], 16, T[38])  STEP(H, b, c, d, a, x[10], 23, T[39])
    STEP(H, a, b, c, d, x[13], 4, T[40]) STEP(H, d, a, b, c, x[0], 11, T[41]) STEP(H, c, d, a, b, x[3], 16, T[42])  STEP(H, b, c, d, a, x[6], 23, T[43])
    STEP(H, a, b, c, d, x[9], 4, T[44])  STEP(H, d, a, b, c, x[12], 11, T[45]) STEP(H, c, d, a, b, x[15], 16, T[46]) STEP(H, b, c, d, a, x[2], 23, T[47])

    STEP(I, a, b, c, d, x[0], 6, T[48]) STEP(I, d, a, b, c, x[7], 10, T[49]) STEP(I, c, d, a, b, x[14], 15, T[50]) STEP(I, b, c, d, a, x[5], 21, T[51])
    STEP(I, a, b, c, d, x[12], 6, T[52]) STEP(I, d, a, b, c, x[3], 10, T[53]) STEP(I, c, d, a, b, x[10], 15, T[54]) STEP(I, b, c, d, a, x[1], 21, T[55])
    STEP(I, a, b, c, d, x[8], 6, T[56])  STEP(I, d, a, b, c, x[15], 10, T[57]) STEP(I, c, d, a, b, x[6], 15, T[58])  STEP(I, b, c, d, a, x[13], 21, T[59])
    STEP(I, a, b, c, d, x[4], 6, T[60])  STEP(I, d, a, b, c, x[11], 10, T[61]) STEP(I, c, d, a, b, x[2], 15, T[62])  STEP(I, b, c, d, a, x[9], 21, T[63])

    state[0] += a; state[1] += b; state[2] += c; state[3] += d;
}

void peel_md5_init(PEEL_MD5_CTX *ctx) {
    ctx->size = 0;
    ctx->buffer[0] = 0x67452301; ctx->buffer[1] = 0xefcdab89; ctx->buffer[2] = 0x98badcfe; ctx->buffer[3] = 0x10325476;
}

void peel_md5_update(PEEL_MD5_CTX *ctx, const uint8_t *input, size_t input_len) {
    uint32_t input_index = (uint32_t)(ctx->size % 64);
    ctx->size += input_len;
    uint32_t part_len = 64 - input_index;
    uint32_t i = 0;
    if (input_len >= part_len) {
        memcpy(&ctx->input[input_index], input, part_len);
        md5_transform(ctx->buffer, ctx->input);
        for (i = part_len; i + 63 < input_len; i += 64) md5_transform(ctx->buffer, &input[i]);
        input_index = 0;
    }
    memcpy(&ctx->input[input_index], &input[i], input_len - i);
}

void peel_md5_final(PEEL_MD5_CTX *ctx) {
    uint8_t padding[64] = {0x80};
    uint8_t bits[8];
    uint64_t bit_size = ctx->size * 8;
    for (int i = 0; i < 8; i++) bits[i] = (uint8_t)(bit_size >> (i * 8));
    uint32_t input_index = (uint32_t)(ctx->size % 64);
    uint32_t pad_len = (input_index < 56) ? (56 - input_index) : (120 - input_index);
    peel_md5_update(ctx, padding, pad_len);
    peel_md5_update(ctx, bits, 8);
    for (int i = 0; i < 4; i++) {
        ctx->digest[i*4] = (uint8_t)(ctx->buffer[i]);
        ctx->digest[i*4+1] = (uint8_t)(ctx->buffer[i] >> 8);
        ctx->digest[i*4+2] = (uint8_t)(ctx->buffer[i] >> 16);
        ctx->digest[i*4+3] = (uint8_t)(ctx->buffer[i] >> 24);
    }
}
