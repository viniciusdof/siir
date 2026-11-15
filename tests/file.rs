//! Whole-file open and write.

use siir::container::Origin;
use siir::{Value, open};

fn fixture(game: &str, path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(game)
        .join(path);
    std::fs::read(path).unwrap()
}

#[test]
fn plain_text_round_trips() {
    let bytes = fixture("ets2", "text/gearbox_layout_volvo_12.sii");
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
    let bytes = fixture("ets2", "profile.sii");
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
fn ats_info_is_encrypted_text() {
    let file = open(&fixture("ats", "saves/1/info.sii")).unwrap();
    assert_eq!(file.origin(), Origin::EncryptedText);
    assert!(file.document().is_ok());
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
fn binary_saves_open_and_write_plain() {
    for (game, money) in [("ets2", "2000"), ("ats", "1500")] {
        let file = open(&fixture(game, "saves/1/game.sii")).unwrap();
        assert_eq!(file.origin(), Origin::EncryptedBinary, "{game}");

        let mut document = file.document().unwrap();
        assert_eq!(document.units_of_class("economy").count(), 1, "{game}");
        {
            let bank = document.get("bank").unwrap();
            assert_eq!(bank.get("money_account"), Some(money), "{game}");
            if game == "ets2" {
                assert_eq!(bank.get("coinsurance_ratio"), Some("&3dcccccd"));
            }
        }
        if game == "ets2" {
            assert!(document.units_of_class("vehicle").count() > 20);
        }

        let out = file.write(&document).unwrap();
        assert!(out.starts_with(b"SiiNunit"));
        let reopened = open(&out).unwrap();
        assert_eq!(reopened.origin(), Origin::Plain);
        assert_eq!(reopened.document().unwrap().to_text(), document.to_text());
    }
}
