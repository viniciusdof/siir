//! Errors raised while opening a container.

use std::fmt;

/// A byte slice that is not a valid SII container.
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

    /// Byte offset into the input where the error was detected.
    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
        }?;
        write!(f, " at offset {}", self.offset)
    }
}

impl std::error::Error for Error {}
