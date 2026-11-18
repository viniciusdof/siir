//! Cargo does not know that `rust-i18n` reads `locales/ui.yml` at compile time,
//! so this tells it to rebuild the crate when the copy changes.

fn main() {
    println!("cargo:rerun-if-changed=locales/ui.yml");
}
