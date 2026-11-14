//! Whole-file open and write.

use siir::container::Origin;
use siir::{Value, open};

fn fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures/ets2")
        .join(path);
    std::fs::read(path).unwrap()
}

#[test]
fn plain_text_round_trips() {
    let bytes = fixture("text/gearbox_layout_volvo_12.sii");
    let file = open(&bytes).unwrap();
    assert_eq!(file.origin(), Origin::Plain);

    let mut document = file.document().unwrap();
    document
        .get("direct_gearbox_config")
        .unwrap()
        .set("clutch_mode", Value::token("auto"))
        .unwrap();

    let out = file.write(&document).unwrap();
    assert!(
        String::from_utf8(out.clone())
            .unwrap()
            .contains("clutch_mode: auto")
    );
    let reopened = open(&out).unwrap();
    assert_eq!(reopened.origin(), Origin::Plain);
    assert_eq!(reopened.document().unwrap().to_text(), document.to_text());
}

#[test]
fn encrypted_text_writes_plain() {
    let bytes = fixture("profile.sii");
    let file = open(&bytes).unwrap();
    assert_eq!(file.origin(), Origin::EncryptedText);

    let mut document = file.document().unwrap();
    document
        .get("user_profile")
        .unwrap()
        .set("face", Value::Int(42))
        .unwrap();

    let out = file.write(&document).unwrap();
    assert!(out.starts_with(b"SiiNunit"));
    let reopened = open(&out).unwrap();
    assert_eq!(reopened.origin(), Origin::Plain);
    assert_eq!(reopened.document().unwrap().to_text(), document.to_text());
}

#[test]
fn invalid_utf8_reports_the_byte() {
    let error = match open(b"SiiN\xff") {
        Ok(_) => panic!("invalid UTF-8 must fail"),
        Err(error) => error,
    };
    assert_eq!(error.offset(), 4);
    assert_eq!(error.to_string(), "payload is not valid UTF-8 at offset 4");
}

#[test]
fn binary_save_opens_and_writes_plain() {
    let file = open(&fixture("saves/1/game.sii")).unwrap();
    assert_eq!(file.origin(), Origin::EncryptedBinary);

    let mut document = file.document().unwrap();
    assert_eq!(document.units_of_class("economy").count(), 1);
    assert!(document.units_of_class("vehicle").count() > 20);
    {
        let bank = document.get("bank").unwrap();
        assert_eq!(bank.get("money_account"), Some("2000"));
        assert_eq!(bank.get("coinsurance_ratio"), Some("&3dcccccd"));
    }

    let out = file.write(&document).unwrap();
    assert!(out.starts_with(b"SiiNunit"));
    let reopened = open(&out).unwrap();
    assert_eq!(reopened.origin(), Origin::Plain);
    assert_eq!(reopened.document().unwrap().to_text(), document.to_text());
}
