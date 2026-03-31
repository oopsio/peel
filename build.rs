fn main() {
    cc::Build::new()
        .file("src/stdlib/crypto/sha256.c")
        .file("src/stdlib/crypto/sha512.c")
        .file("src/stdlib/crypto/md5.c")
        .file("src/stdlib/crypto/aes.c")
        .file("src/stdlib/crypto/hmac.c")
        .file("src/stdlib/crypto/crypto.c")
        .include("src/stdlib/crypto")
        .compile("peel_crypto");
}
