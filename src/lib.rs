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
