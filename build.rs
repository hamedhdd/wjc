fn main() {
    // Embed manifest for Windows MSVC target
    if std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default() == "windows" {
        if std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default() == "msvc" {
            println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
            println!("cargo:rustc-link-arg=/MANIFESTINPUT:app.manifest");
        }
    }
}
