//! Public behavior of `container::open`

use std::fs;

use siir::container::{Origin, open};

const SCSC_HEADER_LEN: usize = 56;
const FIXTURES: &str = "fixtures/ets2";

fn fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(FIXTURES)
        .join(path);
    fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
}

#[test]
fn plain_text_passes_through() {
    let bytes = b"SiiNunit\n{\n}\n";
    let opened = open(bytes).unwrap();
    assert_eq!(opened.origin, Origin::Plain);
    assert_eq!(opened.payload, bytes);
}

#[test]
fn plain_binary_passes_through() {
    let bytes = b"BSII\x01\x00\x00\x00";
    let opened = open(bytes).unwrap();
    assert_eq!(opened.origin, Origin::Binary);
    assert_eq!(opened.payload, bytes);
}

#[test]
fn unknown_signature_reports_offset_zero() {
    let error = open(b"nope").unwrap_err();
    assert_eq!(error.offset(), 0);
    assert_eq!(error.to_string(), "unknown signature \"nope\" at offset 0");
}

#[test]
fn empty_input_is_truncated() {
    let error = open(b"").unwrap_err();
    assert_eq!(error.offset(), 0);
    assert_eq!(
        error.to_string(),
        "truncated: need 4 bytes, got 0 at offset 0"
    );
}

#[test]
fn short_input_is_truncated() {
    let error = open(b"Sii").unwrap_err();
    assert_eq!(error.offset(), 3);
    assert_eq!(
        error.to_string(),
        "truncated: need 4 bytes, got 3 at offset 3"
    );
}

#[test]
fn signature_only_encrypted_input_is_truncated() {
    let error = open(b"ScsC").unwrap_err();
    assert_eq!(error.offset(), 4);
    assert_eq!(
        error.to_string(),
        "truncated: need 56 bytes, got 4 at offset 4"
    );
}

#[test]
fn partial_encrypted_header_is_truncated() {
    let mut bytes = b"ScsC".to_vec();
    bytes.resize(SCSC_HEADER_LEN - 1, 0);
    let error = open(&bytes).unwrap_err();
    assert_eq!(error.offset(), SCSC_HEADER_LEN - 1);
    assert_eq!(
        error.to_string(),
        "truncated: need 56 bytes, got 55 at offset 55"
    );
}

#[test]
fn encrypted_header_without_ciphertext_fails() {
    let mut bytes = b"ScsC".to_vec();
    bytes.resize(SCSC_HEADER_LEN, 0);
    let error = open(&bytes).unwrap_err();
    assert_eq!(error.offset(), SCSC_HEADER_LEN);
    assert_eq!(
        error.to_string(),
        "ciphertext length 0 is not a multiple of the AES block size at offset 56"
    );
}

#[test]
fn encrypted_ciphertext_off_block_size_fails() {
    let mut bytes = b"ScsC".to_vec();
    bytes.resize(SCSC_HEADER_LEN + 17, 0);
    let error = open(&bytes).unwrap_err();
    assert_eq!(error.offset(), SCSC_HEADER_LEN);
    assert_eq!(
        error.to_string(),
        "ciphertext length 17 is not a multiple of the AES block size at offset 56"
    );
}

#[test]
fn fixtures_open_to_expected_payload() {
    let cases: [(&str, Origin, &[u8]); 15] = [
        ("saves/1/game.sii", Origin::EncryptedBinary, b"BSII\x03"),
        (
            "saves/autosave/game.sii",
            Origin::EncryptedBinary,
            b"BSII\x03",
        ),
        (
            "saves/autosave_job/game.sii",
            Origin::EncryptedBinary,
            b"BSII\x03",
        ),
        ("saves/1/info.sii", Origin::EncryptedText, b"SiiNunit"),
        (
            "saves/autosave/info.sii",
            Origin::EncryptedText,
            b"SiiNunit",
        ),
        (
            "saves/autosave_job/info.sii",
            Origin::EncryptedText,
            b"SiiNunit",
        ),
        ("profile.sii", Origin::EncryptedText, b"SiiNunit"),
        ("text/controls_osx.sii", Origin::Plain, b"SiiNunit"),
        (
            "text/gearbox_layout_scania_12.sii",
            Origin::Plain,
            b"SiiNunit",
        ),
        (
            "text/gearbox_layout_scania_12_2.sii",
            Origin::Plain,
            b"SiiNunit",
        ),
        (
            "text/gearbox_layout_volvo_12.sii",
            Origin::Plain,
            b"SiiNunit",
        ),
        (
            "text/gearbox_layout_volvo_12_2.sii",
            Origin::Plain,
            b"SiiNunit",
        ),
        ("text/gearbox_layout_zf_12.sii", Origin::Plain, b"SiiNunit"),
        ("text/gearbox_layout_zf_16.sii", Origin::Plain, b"SiiNunit"),
        ("text/tutorial_hint_data.sii", Origin::Plain, b"SiiNunit"),
    ];

    for (path, origin, prefix) in cases {
        let opened = open(&fixture(path)).unwrap_or_else(|e| panic!("open {path}: {e}"));
        assert_eq!(opened.origin, origin, "{path}");
        assert!(opened.payload.starts_with(prefix), "{path}");
    }
}

#[test]
#[ignore = "needs SIIR_FIXTURE_DIR pointing at a local save slot"]
fn real_save_opens() {
    let dir = std::env::var("SIIR_FIXTURE_DIR").expect("SIIR_FIXTURE_DIR is not set");
    let path = std::path::Path::new(&dir).join("game.sii");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let opened = open(&bytes).unwrap();
    assert!(matches!(
        opened.origin,
        Origin::EncryptedText | Origin::EncryptedBinary
    ));
    assert!(opened.payload.starts_with(b"SiiN") || opened.payload.starts_with(b"BSII"));
}
