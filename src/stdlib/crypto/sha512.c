#include "sha512.h"
#include <string.h>

#define ROTR(x, n) (((x) >> (n)) | ((x) << (64 - (n))))
#define CH(x, y, z) (((x) & (y)) ^ (~(x) & (z)))
#define MAJ(x, y, z) (((x) & (y)) ^ ((x) & (z)) ^ ((y) & (z)))
#define EP0(x) (ROTR(x, 28) ^ ROTR(x, 34) ^ ROTR(x, 39))
#define EP1(x) (ROTR(x, 14) ^ ROTR(x, 18) ^ ROTR(x, 41))
#define SIG0(x) (ROTR(x, 1) ^ ROTR(x, 8) ^ ((x) >> 7))
#define SIG1(x) (ROTR(x, 19) ^ ROTR(x, 61) ^ ((x) >> 6))

static const uint64_t k[80] = {
	0x428a2f98d728ae22, 0x7137449123ef65cd, 0xb5c0fbcfec4d3b2f, 0xe9b5dba58189dbbc, 0x3956c25bf3456701, 0x59f111f15966510a, 0x923f82a4af194f9b, 0xab1c5ed5da6d8118,
	0xd807aa98a3030242, 0x12835b0145706fbe, 0x243185be4ee4b28c, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, 0x80deb1fe3b1696b1, 0x9bdc06a725c71235, 0xc19bf174cf692694,
	0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275, 0x4a7484aa6ea6e483, 0x5cb0a9dcbd41fbd4, 0x76ca1462b8a70650,
	0x983e515230606248, 0xa831c66d431d67c4, 0xb00327c8d30646c7, 0xbf597fc71c0f650a, 0xc6e00bf33572f10b, 0xd5a79147930aa725, 0x06ca6351e003826f, 0x142929670a0e6e70,
	0x27b70a8546d22ffc, 0x2e1b21385c26c926, 0x4d2c6dfc5ac42aed, 0x53380d139d95b3df, 0x650a73548baf63de, 0x766a0abb3c77b2a8, 0x81c2c92e47edaee6, 0x92722c851482353b,
	0xa2bfe8a14cf10364, 0xa81a664bbc423001, 0xc24b8b70d0f89791, 0xc76c51a30654be30, 0xd192e819d6ef5218, 0xd69906245565a910, 0xf40e35855771202a, 0x106aa07032bbd1b8,
	0x19a4c116b8d2d0c8, 0x1e376c085141ab53, 0x2748774cdf8eeb99, 0x34b0bcb5e19b48a8, 0x391c0cb3c5c95a63, 0x4ed8aa4ae3418acb, 0x5b9cca4f7763e373, 0x682e6ff3d6b2b8a3,
	0x748f82ee5defb2fc, 0x78a5636f43172f60, 0x84c87814a1f0ab72, 0x8cc702081a6439ec, 0x90befffa23631e28, 0xa4506ceb39709d6d, 0xbef9a3f75105a07e, 0xc67178f2ffd72895,
	0xd728ae22af723218, 0x106aa1062538fcc1, 0x23ef65cdb5c0fbcf, 0x3b2fe9b5dba58189, 0x45706fbe243185be, 0x550c7dc3d5ffb4e2, 0x72be5d74f27b896f, 0x80deb1fe3b1696b1,
	0x9bdc06a725c71235, 0xc19bf174cf692694, 0xe49b69c19ef14ad2, 0xefbe4786384f25e3, 0x0fc19dc68b8cd5b5, 0x240ca1cc77ac9c65, 0x2de92c6f592b0275, 0x4a7484aa6ea6e483
};

void peel_sha512_transform(PEEL_SHA512_CTX *ctx, const uint8_t data[]) {
	uint64_t a, b, c, d, e, f, g, h, i, j, t1, t2, m[80];

	for (i = 0, j = 0; i < 16; ++i, j += 8)
		m[i] = ((uint64_t)data[j] << 56) | ((uint64_t)data[j + 1] << 48) | ((uint64_t)data[j + 2] << 40) | ((uint64_t)data[j + 3] << 32) |
		       ((uint64_t)data[j + 4] << 24) | ((uint64_t)data[j + 5] << 16) | ((uint64_t)data[j + 6] << 8) | (uint64_t)data[j + 7];
	for ( ; i < 80; ++i)
		m[i] = SIG1(m[i - 2]) + m[i - 7] + SIG0(m[i - 15]) + m[i - 16];

	a = ctx->state[0]; b = ctx->state[1]; c = ctx->state[2]; d = ctx->state[3];
	e = ctx->state[4]; f = ctx->state[5]; g = ctx->state[6]; h = ctx->state[7];

	for (i = 0; i < 80; ++i) {
		t1 = h + EP1(e) + CH(e, f, g) + k[i] + m[i];
		t2 = EP0(a) + MAJ(a, b, c);
		h = g; g = f; f = e;
		e = d + t1;
		d = c; c = b; b = a;
		a = t1 + t2;
	}

	ctx->state[0] += a; ctx->state[1] += b; ctx->state[2] += c; ctx->state[3] += d;
	ctx->state[4] += e; ctx->state[5] += f; ctx->state[6] += g; ctx->state[7] += h;
}

void peel_sha512_init(PEEL_SHA512_CTX *ctx) {
	ctx->datalen = 0; ctx->bitlen[0] = 0; ctx->bitlen[1] = 0;
	ctx->state[0] = 0x6a09e667f3bcc908; ctx->state[1] = 0xbb67ae8584caa73b; ctx->state[2] = 0x3c6ef372fe94f82b; ctx->state[3] = 0xa54ff53a5f1d36f1;
	ctx->state[4] = 0x510e527fade682d1; ctx->state[5] = 0x9b05688c2b3e6c1f; ctx->state[6] = 0x1f83d9abfb41bd6b; ctx->state[7] = 0x5be0cd19137e2179;
}

void peel_sha512_update(PEEL_SHA512_CTX *ctx, const uint8_t data[], size_t len) {
	for (size_t i = 0; i < len; ++i) {
		ctx->data[ctx->datalen] = data[i]; ctx->datalen++;
		if (ctx->datalen == 128) {
			peel_sha512_transform(ctx, ctx->data);
			if (ctx->bitlen[1] > 0xffffffffffffffff - 1024) ctx->bitlen[0]++;
			ctx->bitlen[1] += 1024; ctx->datalen = 0;
		}
	}
}

void peel_sha512_final(PEEL_SHA512_CTX *ctx, uint8_t hash[]) {
	uint32_t i = ctx->datalen;
	if (ctx->datalen < 112) {
		ctx->data[i++] = 0x80;
		while (i < 112) ctx->data[i++] = 0x00;
	} else {
		ctx->data[i++] = 0x80;
		while (i < 128) ctx->data[i++] = 0x00;
		peel_sha512_transform(ctx, ctx->data);
		memset(ctx->data, 0, 112);
	}

	if (ctx->bitlen[1] > 0xffffffffffffffff - (ctx->datalen * 8)) ctx->bitlen[0]++;
	ctx->bitlen[1] += ctx->datalen * 8;
	ctx->data[127] = ctx->bitlen[1]; ctx->data[126] = ctx->bitlen[1] >> 8; ctx->data[125] = ctx->bitlen[1] >> 16; ctx->data[124] = ctx->bitlen[1] >> 24;
	ctx->data[123] = ctx->bitlen[1] >> 32; ctx->data[122] = ctx->bitlen[1] >> 40; ctx->data[121] = ctx->bitlen[1] >> 48; ctx->data[120] = ctx->bitlen[1] >> 56;
	ctx->data[119] = ctx->bitlen[0]; ctx->data[118] = ctx->bitlen[0] >> 8; ctx->data[117] = ctx->bitlen[0] >> 16; ctx->data[116] = ctx->bitlen[0] >> 24;
	ctx->data[115] = ctx->bitlen[0] >> 32; ctx->data[114] = ctx->bitlen[0] >> 40; ctx->data[113] = ctx->bitlen[0] >> 48; ctx->data[112] = ctx->bitlen[0] >> 56;
	peel_sha512_transform(ctx, ctx->data);

	for (i = 0; i < 8; ++i) {
		hash[i] = (ctx->state[0] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 8] = (ctx->state[1] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 16] = (ctx->state[2] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 24] = (ctx->state[3] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 32] = (ctx->state[4] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 40] = (ctx->state[5] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 48] = (ctx->state[6] >> (56 - i * 8)) & 0x00000000000000ff;
		hash[i + 56] = (ctx->state[7] >> (56 - i * 8)) & 0x00000000000000ff;
	}
}
