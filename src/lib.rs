//! Read, edit, and write SCS Software SII files.

pub mod container;

mod error;

pub use error::Error;

/// Crate version, from `Cargo.toml`.
pub const fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn version_matches_manifest() {
        assert_eq!(crate::version(), env!("CARGO_PKG_VERSION"));
    }
}
