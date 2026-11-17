//! Whole-file open and write.

use siir::container::Origin;
use siir::{Index, Value, open};

fn fixture(game: &str, path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(game)
        .join(path);
    std::fs::read(path).unwrap()
}

#[test]
fn plain_text_round_trips_after_several_edits() {
    let bytes = fixture("ets2", "text/gearbox_layout_volvo_12.sii");
    let mut file = open(&bytes).unwrap();
    assert_eq!(file.origin(), Origin::Plain);

    for mode in ["auto", "manual"] {
        file.with_document_mut(|document| {
            document
                .get("direct_gearbox_config")
                .unwrap()
                .set("clutch_mode", Value::token(mode))
                .unwrap();
        });
    }

    let out = file.write();
    assert!(
        String::from_utf8(out.clone())
            .unwrap()
            .contains("clutch_mode: manual")
    );
    let reopened = open(&out).unwrap();
    assert_eq!(reopened.origin(), Origin::Plain);
    assert_eq!(reopened.document().to_text(), file.document().to_text());
}

#[test]
fn encrypted_text_writes_plain() {
    let bytes = fixture("ets2", "profile.sii");
    let mut file = open(&bytes).unwrap();
    assert_eq!(file.origin(), Origin::EncryptedText);

    file.with_document_mut(|document| {
        document
            .get("user_profile")
            .unwrap()
            .set("face", Value::Int(42))
            .unwrap();
    });

    let out = file.write();
    assert!(out.starts_with(b"SiiNunit"));
    let reopened = open(&out).unwrap();
    assert_eq!(reopened.origin(), Origin::Plain);
    assert_eq!(reopened.document().to_text(), file.document().to_text());
}

#[test]
fn ats_info_is_encrypted_text() {
    let file = open(&fixture("ats", "saves/1/info.sii")).unwrap();
    assert_eq!(file.origin(), Origin::EncryptedText);
    assert!(file.document().units().next().is_some());
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
fn units_and_attrs_count_a_real_save() {
    use std::collections::HashSet;

    for (game, units, classes) in [("ets2", 22_498, 37), ("ats", 7_746, 35)] {
        let file = open(&fixture(game, "saves/1/game.sii")).unwrap();
        let document = file.document();

        let mut classes_seen = HashSet::new();
        let mut attrs = 0;
        for unit in document.units() {
            classes_seen.insert(unit.class());
            attrs += unit.attrs().count();
        }

        assert_eq!(document.units().count(), units, "{game}");
        assert_eq!(classes_seen.len(), classes, "{game}");
        assert!(attrs > units, "{game}");

        if game == "ets2" {
            let economy = document.units_of_class("economy").next().unwrap();
            assert!(
                economy
                    .attrs()
                    .any(|(name, index, _)| name == "companies" && index != Index::None)
            );
        }
    }
}

#[test]
fn binary_saves_open_and_write_plain() {
    for (game, money) in [("ets2", "2000"), ("ats", "1500")] {
        let mut file = open(&fixture(game, "saves/1/game.sii")).unwrap();
        assert_eq!(file.origin(), Origin::EncryptedBinary, "{game}");

        file.with_document_mut(|document| {
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
        });

        let out = file.write();
        assert!(out.starts_with(b"SiiNunit"));
        let reopened = open(&out).unwrap();
        assert_eq!(reopened.origin(), Origin::Plain);
        assert_eq!(reopened.document().to_text(), file.document().to_text());
    }
}

#[test]
fn file_is_send_and_static() {
    fn assert_send<T: Send>() {}
    fn assert_static<T: 'static>() {}
    assert_send::<siir::File>();
    assert_static::<siir::File>();
}

#[test]
fn malformed_text_fails_at_open() {
    let error = match open(b"SiiNunit\n{\n") {
        Ok(_) => panic!("a malformed document must fail"),
        Err(error) => error,
    };
    assert_eq!(error.offset(), 11);
}
