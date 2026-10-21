//! Embeds the application icon into the Windows executable (the old
//! PyInstaller build used `--icon logo/icon.ico`).

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=logo/icon.ico");

    // The release is built for x86_64-pc-windows-msvc; other targets (e.g. a
    // GNU cross-check from Linux) need no resource compiler.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if target_os == "windows" && target_env == "msvc" {
        let mut resource = winresource::WindowsResource::new();
        resource.set_icon("logo/icon.ico");
        if let Err(e) = resource.compile() {
            println!("cargo:warning=couldn't embed logo/icon.ico: {e}");
            std::process::exit(1);
        }
    }
}
