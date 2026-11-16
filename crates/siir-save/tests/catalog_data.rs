//! Tests the generated catalog snapshots.

use std::fs;
use std::path::Path;

use serde_json::Value;

fn snapshot(game: &str) -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join(format!("{game}.json"));
    let source = fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("{}: {error}", path.display());
    });
    serde_json::from_str(&source).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn assert_snapshot(game: &str, name: &str) {
    let data = snapshot(game);
    assert_eq!(data["format"], 1, "{name}");
    assert_eq!(data["game"], game, "{name}");
    let trucks = data["trucks"].as_array().unwrap_or_else(|| {
        panic!("{name}: trucks is not an array");
    });
    assert!(!trucks.is_empty(), "{name}: no trucks");
    assert!(
        trucks.iter().any(|truck| {
            truck["engines"]
                .as_array()
                .is_some_and(|engines| !engines.is_empty())
        }),
        "{name}: no truck has an engine"
    );
    for truck in trucks {
        let brand = truck["brand"].as_str().unwrap();
        let model = truck["model"].as_str().unwrap();
        assert!(!brand.is_empty() && !model.is_empty(), "{name}");
        for engine in truck["engines"].as_array().unwrap() {
            assert!(
                engine["path"]
                    .as_str()
                    .unwrap()
                    .starts_with("/def/vehicle/truck/"),
                "{name}"
            );
        }
    }
}

#[test]
fn ets2_snapshot_parses() {
    assert_snapshot("ets2", "ets2");
}

#[test]
fn ats_snapshot_parses() {
    assert_snapshot("ats", "ats");
}
