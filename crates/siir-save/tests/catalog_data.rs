//! Tests the generated catalog snapshots.

use std::fs;
use std::path::Path;

use serde_json::Value;
use siir_save::Game;
use siir_save::catalog::Catalog;

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
    assert_eq!(data["format"], 2, "{name}");
    assert_eq!(data["game"], game, "{name}");
    for section in ["trucks", "cargo", "companies", "trailers", "models"] {
        let items = data[section].as_array().unwrap_or_else(|| {
            panic!("{name}: {section} is not an array");
        });
        assert!(!items.is_empty(), "{name}: no {section}");
    }
    let trucks = data["trucks"].as_array().unwrap();
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

fn assert_load(game: Game, name: &str) {
    let data = snapshot(name);
    let catalog = Catalog::load(game).unwrap_or_else(|error| panic!("{name}: {error}"));
    for (section, len) in [
        ("trucks", catalog.trucks().len()),
        ("cargo", catalog.cargo().len()),
        ("companies", catalog.companies().len()),
        ("trailers", catalog.trailers().len()),
    ] {
        assert!(len > 0, "{name}: no {section}");
        assert_eq!(
            len,
            data[section].as_array().unwrap().len(),
            "{name}: {section}"
        );
    }

    assert_eq!(
        catalog.vehicle_names().len(),
        data["models"].as_array().unwrap().len(),
        "{name}: models"
    );

    let truck = &catalog.trucks()[0];
    let json = &data["trucks"][0];
    assert_eq!(truck.brand, json["brand"].as_str().unwrap(), "{name}");
    assert_eq!(truck.model, json["model"].as_str().unwrap(), "{name}");
    let engine = &truck.engines[0];
    let json = &json["engines"][0];
    assert_eq!(engine.name, json["name"].as_str().unwrap(), "{name}");
    assert_eq!(
        engine.power_hp as u64,
        json["power_hp"].as_u64().unwrap(),
        "{name}"
    );
    assert_eq!(
        engine.torque_nm as u64,
        json["torque_nm"].as_u64().unwrap(),
        "{name}"
    );
    assert_eq!(engine.path, json["path"].as_str().unwrap(), "{name}");

    let cargo = &catalog.cargo()[0];
    let json = &data["cargo"][0];
    assert_eq!(cargo.id, json["id"].as_str().unwrap(), "{name}");
    assert_eq!(cargo.name, json["name"].as_str().unwrap(), "{name}");
    assert_eq!(cargo.mass, json["mass"].as_f64().unwrap() as f32, "{name}");
    assert_eq!(
        cargo.volume,
        json["volume"].as_f64().unwrap() as f32,
        "{name}"
    );
    assert_eq!(
        cargo.reward_per_km,
        json["reward_per_km"].as_f64().unwrap() as f32,
        "{name}"
    );
    assert_eq!(
        cargo.body_types.len(),
        json["body_types"].as_array().unwrap().len(),
        "{name}"
    );

    let company = &catalog.companies()[0];
    let json = &data["companies"][0];
    assert_eq!(company.id, json["id"].as_str().unwrap(), "{name}");
    assert_eq!(company.name, json["name"].as_str().unwrap(), "{name}");

    let trailer = &catalog.trailers()[0];
    let json = &data["trailers"][0];
    assert_eq!(trailer.id, json["id"].as_str().unwrap(), "{name}");
    assert_eq!(
        trailer.body_type,
        json["body_type"].as_str().unwrap(),
        "{name}"
    );
    assert_eq!(
        trailer.chain_type,
        json["chain_type"].as_str().unwrap(),
        "{name}"
    );
    assert_eq!(
        trailer.volume,
        json["volume"].as_f64().unwrap() as f32,
        "{name}"
    );
    assert_eq!(
        trailer.gross_weight_limit,
        json["gross_weight_limit"].as_u64().unwrap(),
        "{name}"
    );
    assert_eq!(
        trailer.chassis_mass,
        json["chassis_mass"].as_u64().unwrap(),
        "{name}"
    );
    assert_eq!(trailer.axles, json["axles"].as_u64().unwrap(), "{name}");
    assert_eq!(
        trailer.length,
        json["length"].as_f64().unwrap() as f32,
        "{name}"
    );
}

#[test]
fn ets2_snapshot_parses() {
    assert_snapshot("ets2", "ets2");
}

#[test]
fn ats_snapshot_parses() {
    assert_snapshot("ats", "ats");
}

#[test]
fn ets2_snapshot_loads() {
    assert_load(Game::Ets2, "ets2");
}

#[test]
fn ats_snapshot_loads() {
    assert_load(Game::Ats, "ats");
}

#[test]
fn names_vehicles_in_every_language() {
    let ets2 = Catalog::load(Game::Ets2).unwrap();
    assert_eq!(
        ets2.vehicle_name("volvo.9700_23", "en_us"),
        Some("Volvo 9700")
    );
    assert_eq!(
        ets2.vehicle_name("volvo.9700_23", "pt_br"),
        Some("Volvo 9700")
    );
    assert_eq!(
        ets2.vehicle_name("volvo.9700_23", "de_de"),
        Some("Volvo 9700")
    );
    assert_eq!(
        ets2.vehicle_name("volvo.9700_23", "xx_yy"),
        Some("Volvo 9700")
    );
    assert_eq!(ets2.vehicle_name("renault.t", "en_us"), Some("Renault T"));
    assert_eq!(ets2.vehicle_name("no.such.model", "en_us"), None);

    let ats = Catalog::load(Game::Ats).unwrap();
    assert_eq!(
        ats.vehicle_name("peterbilt.579", "en_us"),
        Some("Peterbilt 579")
    );
}
