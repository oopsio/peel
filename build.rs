fn main() {
    println!("cargo:rerun-if-changed=src/stdlib/crypto");
    println!("cargo:rerun-if-changed=src/stdlib/os");
    println!("cargo:rerun-if-changed=src/stdlib/gui");
    
    // Download nuklear.h if missing
    let gui_dir = std::path::Path::new("src/stdlib/gui");
    let nuklear_path = gui_dir.join("nuklear.h");
    if !nuklear_path.exists() {
        println!("cargo:warning=Downloading nuklear.h...");
        std::process::Command::new("powershell")
            .args(&["-Command", "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/Immediate-Mode-UI/Nuklear/master/nuklear.h' -OutFile 'src/stdlib/gui/nuklear.h'"])
            .status()
            .expect("Failed to download nuklear.h");
    }

    let gdi_path = gui_dir.join("nuklear_gdi.h");
    if !gdi_path.exists() {
        println!("cargo:warning=Downloading nuklear_gdi.h...");
        std::process::Command::new("powershell")
            .args(&["-Command", "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/Immediate-Mode-UI/Nuklear/master/demo/gdi/nuklear_gdi.h' -OutFile 'src/stdlib/gui/nuklear_gdi.h'"])
            .status()
            .expect("Failed to download nuklear_gdi.h");
    }

    let x11_path = gui_dir.join("nuklear_xlib.h");
    if !x11_path.exists() {
        println!("cargo:warning=Downloading nuklear_xlib.h...");
        std::process::Command::new("powershell")
            .args(&["-Command", "Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/Immediate-Mode-UI/Nuklear/master/demo/x11/nuklear_xlib.h' -OutFile 'src/stdlib/gui/nuklear_xlib.h'"])
            .status()
            .expect("Failed to download nuklear_xlib.h");
    }

    let mut build = cc::Build::new();
    build.file("src/stdlib/crypto/sha256.c")
        .file("src/stdlib/crypto/sha512.c")
        .file("src/stdlib/crypto/md5.c")
        .file("src/stdlib/crypto/aes.c")
        .file("src/stdlib/crypto/hmac.c")
        .file("src/stdlib/crypto/crypto.c")
        .file("src/stdlib/os/os.c")
        .file("src/stdlib/gui/gui.c")
        .include("src/stdlib/crypto")
        .include("src/stdlib/os")
        .include("src/stdlib/gui");

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        println!("cargo:rustc-link-lib=gdi32");
    } else if target_os == "linux" {
        println!("cargo:rustc-link-lib=X11");
    }

    build.compile("peel_native");
}
