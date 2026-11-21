// Embeds the app icon and version details into the Windows .exe.
fn main() {
    println!("cargo:rerun-if-changed=assets/icon.ico");
    println!("cargo:rerun-if-changed=packaging/windows/stagecraft.rc");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os == "windows" {
        embed_resource::compile("packaging/windows/stagecraft.rc", embed_resource::NONE)
            .manifest_optional()
            .unwrap();
    }
}
