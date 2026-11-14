//! Open a whole SII file and write it back as plain `SiiNunit` text.

use crate::container::{self, Origin};
use crate::error::Error;
use crate::model::Document;

/// An SII file opened to its typed document.
pub struct File {
    origin: Origin,
    payload: String,
}

/// Opens an SII file, unwrapping `ScsC` when present.
pub fn open(bytes: &[u8]) -> Result<File, Error> {
    let opened = container::open(bytes)?;
    match opened.origin {
        Origin::Plain | Origin::EncryptedText => {
            let payload = String::from_utf8(opened.payload)
                .map_err(|error| Error::invalid_encoding(error.utf8_error().valid_up_to()))?;
            Ok(File {
                origin: opened.origin,
                payload,
            })
        }
        Origin::Binary | Origin::EncryptedBinary => Err(Error::unsupported("BSII")),
    }
}

impl File {
    /// Where the payload came from.
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// Parses a fresh document from the payload; keep the returned value.
    pub fn document(&self) -> Result<Document<'_>, Error> {
        Document::parse(&self.payload)
    }

    /// Writes the document as plain `SiiNunit` text.
    pub fn write(&self, document: &Document<'_>) -> Result<Vec<u8>, Error> {
        Ok(document.to_text().into_bytes())
    }
}
