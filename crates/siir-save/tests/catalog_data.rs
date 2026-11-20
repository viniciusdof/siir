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
    assert_eq!(data["game"], game, "{name}");
    assert!(
        !data["country_names"].as_array().unwrap().is_empty(),
        "{name}: no countries"
    );
    let steps = data["levels"]["steps"].as_array().unwrap();
    assert!(!steps.is_empty(), "{name}: no level steps");
    assert!(
        steps
            .iter()
            .all(|step| step.as_u64().is_some_and(|step| step > 0)),
        "{name}: a level step is not positive"
    );
    for section in [
        "vehicles",
        "cargo",
        "companies",
        "trailers",
        "trailer_body_types",
        "trailer_brands",
        "cities",
    ] {
        let items = data[section].as_array().unwrap_or_else(|| {
            panic!("{name}: {section} is not an array");
        });
        assert!(!items.is_empty(), "{name}: no {section}");
    }
    let vehicles = data["vehicles"].as_array().unwrap();
    assert!(
        vehicles.iter().any(|vehicle| {
            vehicle["engines"]
                .as_array()
                .is_some_and(|engines| !engines.is_empty())
        }),
        "{name}: no truck has an engine"
    );
    for vehicle in vehicles {
        let token = vehicle["token"].as_str().unwrap();
        assert!(token.contains('.'), "{name}: {token}");
        for engine in vehicle["engines"].as_array().into_iter().flatten() {
            assert!(
                engine["path"]
                    .as_str()
                    .unwrap()
                    .starts_with("/def/vehicle/"),
                "{name}"
            );
        }
    }
}

fn assert_load(game: Game, name: &str) {
    let data = snapshot(name);
    let catalog = Catalog::load(game).unwrap_or_else(|error| panic!("{name}: {error}"));
    for (section, len) in [
        ("vehicles", catalog.vehicles().len()),
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

    let ix = catalog
        .vehicles()
        .iter()
        .position(|vehicle| !vehicle.engines.is_empty())
        .unwrap();
    let truck = &catalog.vehicles()[ix];
    let json = &data["vehicles"][ix];
    assert_eq!(truck.token, json["token"].as_str().unwrap(), "{name}");
    let engine = &truck.engines[0];
    let json = &json["engines"][0];
    let english = json["names"]["en_us"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: an engine has no English name"));
    assert_eq!(engine.name("en_us"), Some(english), "{name}");
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

    let found = catalog
        .vehicle(&truck.token)
        .expect("the model token resolves");
    assert_eq!(found.engines.len(), truck.engines.len(), "{name}");
    assert_eq!(found.cabins.len(), truck.cabins.len(), "{name}");
    assert!(catalog.vehicle("no.such_model").is_none(), "{name}");

    let body_type = &catalog.body_types()[0];
    let json = &data["trailer_body_types"][0];
    assert_eq!(body_type.token, json["token"].as_str().unwrap(), "{name}");
    let english = json["names"]["en_us"]
        .as_str()
        .unwrap_or_else(|| panic!("{name}: a body type has no English name"));
    assert_eq!(
        catalog.body_type_name(&body_type.token, "en_us"),
        Some(english),
        "{name}"
    );

    for (token, section, resolved) in [
        (
            catalog.brands()[0].token.as_str(),
            "trailer_brands",
            catalog.brand_name(&catalog.brands()[0].token, "en_us"),
        ),
        (
            catalog.cities()[0].token.as_str(),
            "cities",
            catalog.city_name(&catalog.cities()[0].token, "en_us"),
        ),
    ] {
        let json = &data[section][0];
        assert_eq!(json["token"].as_str(), Some(token), "{name}");
        let english = json["names"]["en_us"]
            .as_str()
            .unwrap_or_else(|| panic!("{name}: {section} has no English name"));
        assert_eq!(resolved, Some(english), "{name}");
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

#[test]
fn levels_follow_each_game_table() {
    let ets2 = Catalog::load(Game::Ets2).unwrap();
    let levels = ets2.levels();
    assert_eq!(levels.experience(5), Some(3300));
    assert_eq!(levels.level(3300), 5);
    assert_eq!(levels.title(50, "pt_br"), Some("Lenda"));

    let ats = Catalog::load(Game::Ats).unwrap();
    let levels = ats.levels();
    assert_eq!(levels.experience(5), Some(3400));
    assert_eq!(levels.title(10, "en_us"), Some("Workhorse"));
}

#[test]
fn names_plate_countries_in_every_language() {
    let ets2 = Catalog::load(Game::Ets2).unwrap();
    assert_eq!(ets2.country_name("germany", "en_us"), Some("Germany"));
    assert_eq!(ets2.country_name("germany", "pt_br"), Some("Alemanha"));
    assert_eq!(ets2.country_name("poland", "en_us"), Some("Poland"));
    assert!(
        ets2.countries()
            .iter()
            .any(|country| country.token == "czech")
    );

    let ats = Catalog::load(Game::Ats).unwrap();
    assert_eq!(ats.country_name("texas", "en_us"), Some("Texas"));
    assert_eq!(ats.country_name("texas", "pt_br"), Some("Texas"));
}

#[test]
fn currencies_start_with_the_stored_one() {
    let ets2 = Catalog::load(Game::Ets2).unwrap();
    let first = &ets2.currencies()[0];
    assert_eq!((first.code.as_str(), first.ratio), ("EUR", 1.0));
    assert!(
        ets2.currencies()
            .iter()
            .any(|currency| currency.code == "GBP")
    );

    let ats = Catalog::load(Game::Ats).unwrap();
    let first = &ats.currencies()[0];
    assert_eq!((first.code.as_str(), first.ratio), ("USD", 1.0));
    assert!(ats.currencies().iter().all(|currency| currency.ratio > 0.0));
}
