fn main() {
    println!("cargo:rerun-if-changed=src/stdlib/crypto");
    println!("cargo:rerun-if-changed=src/stdlib/os");
    
    cc::Build::new()
        .file("src/stdlib/crypto/sha256.c")
        .file("src/stdlib/crypto/sha512.c")
        .file("src/stdlib/crypto/md5.c")
        .file("src/stdlib/crypto/aes.c")
        .file("src/stdlib/crypto/hmac.c")
        .file("src/stdlib/crypto/crypto.c")
        .file("src/stdlib/os/os.c")
        .include("src/stdlib/crypto")
        .include("src/stdlib/os")
        .compile("peel_native");
}
