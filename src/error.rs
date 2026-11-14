//! Errors raised while reading or editing SII data.

use std::fmt;

/// A typed SIIR failure with the byte offset where it was detected.
#[derive(Debug)]
pub struct Error {
    kind: Kind,
    offset: usize,
}

#[derive(Debug)]
enum Kind {
    UnknownSignature([u8; 4]),
    Truncated { needed: usize, got: usize },
    BadCiphertext { len: usize },
    Decrypt,
    Decompress,
    Parse { expected: &'static str },
    MissingAttribute { name: String },
    MissingUnit { id: String },
    DuplicateUnit { id: String },
    InvalidName { reason: &'static str },
    InvalidValue { reason: &'static str },
    InvalidEncoding,
    Unsupported(&'static str),
}

impl Error {
    pub(crate) fn unknown_signature(signature: [u8; 4], offset: usize) -> Self {
        Self {
            kind: Kind::UnknownSignature(signature),
            offset,
        }
    }

    pub(crate) fn truncated(needed: usize, got: usize, offset: usize) -> Self {
        Self {
            kind: Kind::Truncated { needed, got },
            offset,
        }
    }

    pub(crate) fn bad_ciphertext(len: usize, offset: usize) -> Self {
        Self {
            kind: Kind::BadCiphertext { len },
            offset,
        }
    }

    pub(crate) fn decrypt(offset: usize) -> Self {
        Self {
            kind: Kind::Decrypt,
            offset,
        }
    }

    pub(crate) fn decompress(offset: usize) -> Self {
        Self {
            kind: Kind::Decompress,
            offset,
        }
    }

    pub(crate) fn parse(offset: usize, expected: &'static str) -> Self {
        Self {
            kind: Kind::Parse { expected },
            offset,
        }
    }

    pub(crate) fn missing_attribute(name: &str, offset: usize) -> Self {
        Self {
            kind: Kind::MissingAttribute {
                name: name.to_owned(),
            },
            offset,
        }
    }

    pub(crate) fn missing_unit(id: &str) -> Self {
        Self {
            kind: Kind::MissingUnit { id: id.to_owned() },
            offset: 0,
        }
    }

    pub(crate) fn duplicate_unit(id: &str) -> Self {
        Self {
            kind: Kind::DuplicateUnit { id: id.to_owned() },
            offset: 0,
        }
    }

    pub(crate) fn invalid_name(reason: &'static str) -> Self {
        Self {
            kind: Kind::InvalidName { reason },
            offset: 0,
        }
    }

    pub(crate) fn invalid_value(reason: &'static str, offset: usize) -> Self {
        Self {
            kind: Kind::InvalidValue { reason },
            offset,
        }
    }

    pub(crate) fn invalid_encoding(offset: usize) -> Self {
        Self {
            kind: Kind::InvalidEncoding,
            offset,
        }
    }

    pub(crate) fn unsupported(what: &'static str) -> Self {
        Self {
            kind: Kind::Unsupported(what),
            offset: 0,
        }
    }

    /// Byte offset into the input where the error was detected.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            Kind::MissingUnit { id } => return write!(f, "missing unit {id:?}"),
            Kind::DuplicateUnit { id } => return write!(f, "duplicate unit {id:?}"),
            Kind::InvalidName { reason } => return write!(f, "invalid name: {reason}"),
            Kind::Unsupported(what) => return write!(f, "unsupported: {what}"),
            _ => {}
        }
        match &self.kind {
            Kind::UnknownSignature(signature) => {
                let signature = String::from_utf8_lossy(signature);
                write!(f, "unknown signature {signature:?}")
            }
            Kind::Truncated { needed, got } => {
                write!(f, "truncated: need {needed} bytes, got {got}")
            }
            Kind::BadCiphertext { len } => write!(
                f,
                "ciphertext length {len} is not a multiple of the AES block size"
            ),
            Kind::Decrypt => write!(f, "AES-256-CBC decryption failed"),
            Kind::Decompress => write!(f, "zlib decompression failed"),
            Kind::Parse { expected } => write!(f, "expected {expected}"),
            Kind::MissingAttribute { name } => write!(f, "missing attribute {name:?}"),
            Kind::InvalidValue { reason } => write!(f, "invalid value: {reason}"),
            Kind::InvalidEncoding => write!(f, "payload is not valid UTF-8"),
            Kind::MissingUnit { .. }
            | Kind::DuplicateUnit { .. }
            | Kind::InvalidName { .. }
            | Kind::Unsupported(_) => unreachable!(),
        }?;
        write!(f, " at offset {}", self.offset)
    }
}

impl std::error::Error for Error {}
