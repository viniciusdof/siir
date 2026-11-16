//! Signature detection and `ScsC` unwrapping.

use std::io::Read;

use aes::Aes256;
use aes::cipher::block_padding::Pkcs7;
use aes::cipher::{BlockDecryptMut, KeyIvInit};
use cbc::Decryptor;

use crate::error::Error;

const SIGNATURE_LEN: usize = 4;
const HMAC_LEN: usize = 32;
const IV_LEN: usize = 16;
const SIZE_LEN: usize = 4;
const AES_BLOCK_LEN: usize = 16;
const IV_START: usize = SIGNATURE_LEN + HMAC_LEN;
const IV_END: usize = IV_START + IV_LEN;
const HEADER_LEN: usize = IV_END + SIZE_LEN;

/// Fixed AES-256 key for `ScsC`.
const AES_KEY: [u8; 32] = [
    0x2a, 0x5f, 0xcb, 0x17, 0x91, 0xd2, 0x2f, 0xb6, 0x02, 0x45, 0xb3, 0xd8, 0x36, 0x9e, 0xd0, 0xb2,
    0xc2, 0x73, 0x71, 0x56, 0x3f, 0xbf, 0x1f, 0x3c, 0x9e, 0xdf, 0x6b, 0x11, 0x82, 0x5a, 0x5d, 0x0a,
];

/// Origin of an opened payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origin {
    /// Plain `SiiNunit` text.
    Plain,
    /// Plain `BSII` binary.
    Binary,
    /// `SiiNunit` text inside `ScsC`.
    EncryptedText,
    /// `BSII` binary inside `ScsC`.
    EncryptedBinary,
}

/// A container opened down to its payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Opened {
    /// Payload bytes: `SiiNunit` text or `BSII`, never `ScsC`.
    pub payload: Vec<u8>,
    /// Format that produced [`Opened::payload`].
    pub origin: Origin,
}

/// Detects the signature and unwraps `ScsC` to its payload.
pub fn open(bytes: &[u8]) -> Result<Opened, Error> {
    let Some(signature) = bytes.first_chunk::<SIGNATURE_LEN>() else {
        return Err(Error::truncated(SIGNATURE_LEN, bytes.len(), bytes.len()));
    };

    match signature {
        b"SiiN" => Ok(Opened {
            payload: bytes.to_vec(),
            origin: Origin::Plain,
        }),
        b"BSII" => Ok(Opened {
            payload: bytes.to_vec(),
            origin: Origin::Binary,
        }),
        b"ScsC" => unwrap_scsc(bytes),
        _ => Err(Error::unknown_signature(*signature, 0)),
    }
}

fn unwrap_scsc(bytes: &[u8]) -> Result<Opened, Error> {
    if bytes.len() < HEADER_LEN {
        return Err(Error::truncated(HEADER_LEN, bytes.len(), bytes.len()));
    }

    let size = u32::from_le_bytes(
        bytes[IV_END..HEADER_LEN]
            .try_into()
            .map_err(|_| Error::truncated(SIZE_LEN, bytes.len(), IV_END))?,
    ) as usize;

    let iv = &bytes[IV_START..IV_END];
    let ciphertext = &bytes[HEADER_LEN..];
    if ciphertext.is_empty() || !ciphertext.len().is_multiple_of(AES_BLOCK_LEN) {
        return Err(Error::bad_ciphertext(ciphertext.len(), HEADER_LEN));
    }

    let mut buffer = ciphertext.to_vec();
    let decryptor = Decryptor::<Aes256>::new_from_slices(&AES_KEY, iv)
        .map_err(|_| Error::decrypt(HEADER_LEN))?;
    let compressed = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| Error::decrypt(HEADER_LEN))?;

    let mut payload = Vec::new();
    flate2::read::ZlibDecoder::new(compressed)
        .take(size as u64 + 1)
        .read_to_end(&mut payload)
        .map_err(|_| Error::decompress(HEADER_LEN))?;

    if payload.len() != size {
        return Err(Error::decompress(HEADER_LEN));
    }

    let Some(signature) = payload.first_chunk::<SIGNATURE_LEN>() else {
        return Err(Error::truncated(SIGNATURE_LEN, payload.len(), HEADER_LEN));
    };
    let origin = match signature {
        b"SiiN" => Origin::EncryptedText,
        b"BSII" => Origin::EncryptedBinary,
        _ => return Err(Error::unknown_signature(*signature, HEADER_LEN)),
    };

    Ok(Opened { payload, origin })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use aes::cipher::{BlockEncryptMut, KeyIvInit};

    use super::*;

    fn scsc(payload: &[u8]) -> Vec<u8> {
        let mut encoder =
            flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(payload).unwrap();
        scsc_from_compressed(&encoder.finish().unwrap(), payload.len())
    }

    /// Wraps already-compressed bytes, so a test can feed invalid zlib.
    fn scsc_from_compressed(compressed: &[u8], payload_len: usize) -> Vec<u8> {
        let mut buffer = compressed.to_vec();

        let iv = [0x5a; IV_LEN];
        let message_len = buffer.len();
        buffer.resize(message_len + AES_BLOCK_LEN, 0);
        let ciphertext = cbc::Encryptor::<Aes256>::new_from_slices(&AES_KEY, &iv)
            .unwrap()
            .encrypt_padded_mut::<Pkcs7>(&mut buffer, message_len)
            .unwrap()
            .to_vec();

        let mut bytes = Vec::with_capacity(HEADER_LEN + ciphertext.len());
        bytes.extend_from_slice(b"ScsC");
        bytes.extend_from_slice(&[0; HMAC_LEN]);
        bytes.extend_from_slice(&iv);
        bytes.extend_from_slice(&(payload_len as u32).to_le_bytes());
        bytes.extend_from_slice(&ciphertext);
        bytes
    }

    #[test]
    fn unwraps_encrypted_text() {
        let payload = b"SiiNunit\n{\n}\n";
        let opened = open(&scsc(payload)).unwrap();
        assert_eq!(opened.origin, Origin::EncryptedText);
        assert_eq!(opened.payload, payload);
    }

    #[test]
    fn unwraps_encrypted_binary() {
        let payload = b"BSII\x01\x00\x00\x00";
        let opened = open(&scsc(payload)).unwrap();
        assert_eq!(opened.origin, Origin::EncryptedBinary);
        assert_eq!(opened.payload, payload);
    }

    #[test]
    fn corrupt_ciphertext_fails_decryption() {
        let mut bytes = scsc(b"SiiNunit\n");
        let last = bytes.len() - 1;
        bytes[last] ^= 0xff;
        let error = open(&bytes).unwrap_err();
        assert_eq!(error.offset(), HEADER_LEN);
        assert_eq!(
            error.to_string(),
            format!("AES-256-CBC decryption failed at offset {HEADER_LEN}")
        );
    }

    #[test]
    fn ciphertext_not_a_block_multiple_fails() {
        let mut bytes = scsc(b"SiiNunit\n");
        bytes.pop();
        let error = open(&bytes).unwrap_err();
        assert_eq!(error.offset(), HEADER_LEN);
        assert!(error.to_string().contains("not a multiple"));
    }

    #[test]
    fn invalid_zlib_fails_decompression() {
        let error = open(&scsc_from_compressed(b"not zlib data", 0)).unwrap_err();
        assert_eq!(error.offset(), HEADER_LEN);
        assert_eq!(
            error.to_string(),
            format!("zlib decompression failed at offset {HEADER_LEN}")
        );
    }

    #[test]
    fn declared_size_mismatch_fails() {
        let mut bytes = scsc(b"SiiNunit\n");
        bytes[IV_END..HEADER_LEN].copy_from_slice(&1u32.to_le_bytes());
        let error = open(&bytes).unwrap_err();
        assert_eq!(error.offset(), HEADER_LEN);
        assert_eq!(
            error.to_string(),
            format!("zlib decompression failed at offset {HEADER_LEN}")
        );
    }

    #[test]
    fn payload_with_unknown_signature_fails() {
        let error = open(&scsc(b"nope, not sii")).unwrap_err();
        assert_eq!(error.offset(), HEADER_LEN);
        assert_eq!(
            error.to_string(),
            format!("unknown signature \"nope\" at offset {HEADER_LEN}")
        );
    }

    #[test]
    fn payload_shorter_than_signature_is_truncated() {
        let error = open(&scsc(b"Sii")).unwrap_err();
        assert_eq!(error.offset(), HEADER_LEN);
        assert_eq!(
            error.to_string(),
            format!("truncated: need 4 bytes, got 3 at offset {HEADER_LEN}")
        );
    }
}
