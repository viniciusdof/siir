//! Read, edit, and write SCS Software SII files.

pub mod container;

mod binary;
mod error;
mod file;
mod model;
mod text;

pub use error::Error;
pub use file::{File, open};
pub use model::{Document, Unit, Value};

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
