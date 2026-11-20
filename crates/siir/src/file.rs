use self_cell::self_cell;

use crate::container::{self, Origin};
use crate::error::Error;
use crate::model::Document;

self_cell!(
    struct Inner {
        owner: String,
        #[covariant]
        dependent: Document,
    }
);

/// An SII file opened to its typed document.
pub struct File {
    origin: Origin,
    inner: Inner,
}

/// Opens an SII file, unwrapping `ScsC` when present.
pub fn open(bytes: &[u8]) -> Result<File, Error> {
    let opened = container::open(bytes)?;
    let payload = match opened.origin {
        Origin::Plain | Origin::EncryptedText => String::from_utf8(opened.payload)
            .map_err(|error| Error::invalid_encoding(error.utf8_error().valid_up_to()))?,
        Origin::Binary | Origin::EncryptedBinary => crate::binary::decode(&opened.payload)?,
    };
    let inner = Inner::try_new(payload, |payload| Document::parse(payload))?;
    Ok(File {
        origin: opened.origin,
        inner,
    })
}

impl File {
    /// Where the payload came from.
    pub fn origin(&self) -> Origin {
        self.origin
    }

    /// The parsed document.
    pub fn document(&self) -> &Document<'_> {
        self.inner.borrow_dependent()
    }

    /// Edits the parsed document in place.
    pub fn with_document_mut<R>(&mut self, edit: impl FnOnce(&mut Document<'_>) -> R) -> R {
        self.inner.with_dependent_mut(|_, document| edit(document))
    }

    /// Writes the document as plain `SiiNunit` text.
    pub fn write(&self) -> Vec<u8> {
        self.document().to_text().into_bytes()
    }
}
