//! `siir::version` reports the version declared in `Cargo.toml`.

#[test]
fn version_is_public() {
    assert_eq!(siir::version(), env!("CARGO_PKG_VERSION"));
}
