//! Cargo does not know that `rust-i18n` reads `locales/ui.yml` at compile time,
//! so this tells it to rebuild the crate when the copy changes. On Windows it
//! also embeds the app icon and version resource in the executable.

fn main() {
    println!("cargo:rerun-if-changed=locales/ui.yml");
    #[cfg(windows)]
    windows_resource();
}

#[cfg(windows)]
fn windows_resource() {
    const ICON: &str = "../../packaging/icons/siir.ico";
    println!("cargo:rerun-if-changed={ICON}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let mut resource = winresource::WindowsResource::new();
    resource
        .set_icon(ICON)
        .set("ProductName", "SIIR")
        .set("FileDescription", "SIIR");
    if let Err(error) = resource.compile() {
        panic!("cannot embed the Windows resource: {error}");
    }
}
