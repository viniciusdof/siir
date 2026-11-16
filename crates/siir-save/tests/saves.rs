//! Read model and commands over the real ETS2 and ATS fixtures.

use siir::Document;
use siir_save::{Command, Game, apply, read};

fn fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(path);
    std::fs::read(path).unwrap()
}

fn written(file: &siir::File, document: &Document<'_>) -> String {
    String::from_utf8(file.write(document).unwrap()).unwrap()
}

fn line_count(text: &str) -> usize {
    text.split_inclusive('\n').count()
}

fn changed_lines(before: &str, after: &str) -> Vec<(String, String)> {
    let before: Vec<&str> = before.split_inclusive('\n').collect();
    let after: Vec<&str> = after.split_inclusive('\n').collect();
    assert_eq!(before.len(), after.len());
    before
        .iter()
        .zip(&after)
        .filter(|(old, new)| old != new)
        .map(|(old, new)| (old.to_string(), new.to_string()))
        .collect()
}

struct Case {
    name: &'static str,
    fixture: &'static str,
    game: Game,
    money: i64,
    hq_city: &'static str,
    trucks: usize,
    garages: usize,
    drivers: usize,
    companies: usize,
    company: &'static str,
    offers: usize,
    target: &'static str,
    distance: u64,
    used: usize,
    price: i64,
    truck_id: &'static str,
    model: &'static str,
    data_path: &'static str,
    engine: &'static str,
    engine_wear: f32,
    fuel: f32,
    job_cargo: &'static str,
    job_km: u64,
}

const CASES: [Case; 2] = [
    Case {
        name: "ets2",
        fixture: "ets2/saves/1/game.sii",
        game: Game::Ets2,
        money: 2000,
        hq_city: "prague",
        trucks: 27,
        garages: 237,
        drivers: 354,
        companies: 1886,
        company: "company.permanent.medas",
        offers: 6499,
        target: "icaria.chania",
        distance: 27,
        used: 26,
        price: 122_007,
        truck_id: "_nameless.7f9f.ff91.ec30",
        model: "renault.etech_t",
        data_path: "/def/vehicle/truck/renault.etech_t/data.sii",
        engine: "/def/vehicle/truck/renault.etech_t/engine/490kw.sii",
        engine_wear: f32::from_bits(0x36c9_53b6),
        fuel: f32::from_bits(0x3f7d_cfde),
        job_cargo: "cargo.stones",
        job_km: 183,
    },
    Case {
        name: "ats",
        fixture: "ats/saves/1/game.sii",
        game: Game::Ats,
        money: 1500,
        hq_city: "seattle",
        trucks: 13,
        garages: 59,
        drivers: 354,
        companies: 885,
        company: "company.permanent.dri_oil_gst",
        offers: 2560,
        target: "ent_whs.los_angeles",
        distance: 282,
        used: 12,
        price: 102_999,
        truck_id: "_nameless.7fef.7fdc.1110",
        model: "volvo.vnr_e",
        data_path: "/def/vehicle/truck/volvo.vnr_e/data.sii",
        engine: "/def/vehicle/truck/volvo.vnr_e/engine/e340.sii",
        engine_wear: 0.0,
        fuel: 1.0,
        job_cargo: "cargo.mortar",
        job_km: 51,
    },
];

fn assert_case(case: &Case) {
    let bytes = fixture(case.fixture);
    let file = siir::open(&bytes).unwrap();
    let document = file.document().unwrap();
    let save = read(&document, case.game).unwrap();
    let name = case.name;

    assert_eq!(save.game, case.game, "{name}");
    assert_eq!(
        save.bank.as_ref().unwrap().money_account,
        case.money,
        "{name}"
    );

    let player = save.player.as_ref().unwrap();
    assert_eq!(player.hq_city, case.hq_city, "{name}");
    assert_eq!(player.my_vehicles.len(), 3, "{name}");
    assert!(player.assigned_vehicles.is_some(), "{name}");
    assert!(player.current_job.is_some(), "{name}");

    assert_eq!(save.trucks.len(), case.trucks, "{name}");
    assert_eq!(save.trailers.len(), 1, "{name}");
    assert_eq!(save.garages.len(), case.garages, "{name}");
    assert_eq!(
        save.garages
            .iter()
            .filter(|garage| garage.status == 6)
            .count(),
        1,
        "{name}"
    );
    assert_eq!(save.drivers.len(), case.drivers, "{name}");
    assert_eq!(save.companies.len(), case.companies, "{name}");
    assert_eq!(save.companies[0].permanent_data, case.company, "{name}");
    assert_eq!(save.jobs.len(), case.drivers, "{name}");
    assert_eq!(save.jobs[0].cargo, None, "{name}");
    assert_eq!(save.offers.len(), case.offers, "{name}");
    assert_eq!(save.offers[0].target, case.target, "{name}");
    assert_eq!(save.offers[0].shortest_distance_km, case.distance, "{name}");
    assert_eq!(save.used_trucks.len(), case.used, "{name}");
    assert_eq!(save.used_trucks[0].price, case.price, "{name}");
    assert!(save.offence.is_some(), "{name}");

    let truck = save.current_truck().unwrap();
    assert_eq!(truck.id, case.truck_id, "{name}");
    assert_eq!(truck.model, Some(case.model), "{name}");
    assert_eq!(truck.data_path, Some(case.data_path), "{name}");
    assert_eq!(truck.engine, Some(case.engine), "{name}");
    assert_eq!(truck.engine_wear, case.engine_wear, "{name}");
    assert_eq!(truck.fuel_relative, case.fuel, "{name}");
    assert!(truck.placement.unwrap().starts_with('('), "{name}");
    assert!(save.truck(truck.id).is_some(), "{name}");

    let job = save.player_job.as_ref().unwrap();
    assert_eq!(job.cargo, Some(case.job_cargo), "{name}");
    assert_eq!(job.planned_distance_km, case.job_km, "{name}");
}

#[test]
fn reads_ets2() {
    assert_case(&CASES[0]);
}

#[test]
fn reads_ats() {
    assert_case(&CASES[1]);
}

#[test]
fn ets2_commands_change_only_expected_lines() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let mut document = file.document().unwrap();
    let save = read(&document, Game::Ets2).unwrap();
    let id = save.current_truck().unwrap().id.to_owned();
    let target = save.used_trucks[0].truck.to_owned();

    let old = written(&file, &document);
    apply(&mut document, Game::Ets2, Command::RepairTruck(id.clone())).unwrap();
    let new = written(&file, &document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 4);
    for (old, new) in changed {
        let name = old.trim_start().split(':').next().unwrap();
        assert!(name.ends_with("_wear") || name.ends_with("_unfixable"));
        assert!(new.trim_start().ends_with(": 0\n"));
    }
    let unit = document.units().find(|unit| unit.id() == id).unwrap();
    for name in [
        "engine_wear",
        "transmission_wear",
        "cabin_wear",
        "engine_wear_unfixable",
        "transmission_wear_unfixable",
        "cabin_wear_unfixable",
    ] {
        assert_eq!(unit.get(name), Some("0"), "{name}");
    }

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetFuel {
            id: id.clone(),
            level: 0.5,
        },
    )
    .unwrap();
    let new = written(&file, &document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "fuel_relative: 0.5\n");

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetOdometer {
            id: id.clone(),
            km: 1234,
        },
    )
    .unwrap();
    let new = written(&file, &document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 3);
    for (old, new) in changed {
        assert!(matches!(
            old.trim_start().split(':').next().unwrap(),
            "odometer" | "integrity_odometer" | "trip_distance_km"
        ));
        assert!(new.trim_start().ends_with(": 1234\n"));
    }

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetLicensePlate {
            id: id.clone(),
            plate: "TEST-123".into(),
        },
    )
    .unwrap();
    let new = written(&file, &document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "license_plate: \"TEST-123\"\n");

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetCurrentTruck(target.clone()),
    )
    .unwrap();
    let new = written(&file, &document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), format!("vehicle: {target}\n"));

    let save = read(&document, Game::Ets2).unwrap();
    assert_eq!(save.current_truck().unwrap().id, target);

    let reopened = siir::open(new.as_bytes()).unwrap();
    assert_eq!(reopened.document().unwrap().to_text(), new);
}

#[test]
fn ats_commands_change_only_expected_lines() {
    let bytes = fixture("ats/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let mut document = file.document().unwrap();
    let id = read(&document, Game::Ats)
        .unwrap()
        .current_truck()
        .unwrap()
        .id
        .to_owned();

    let old = written(&file, &document);
    apply(
        &mut document,
        Game::Ats,
        Command::SetFuel {
            id: id.clone(),
            level: 0.25,
        },
    )
    .unwrap();
    let new = written(&file, &document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "fuel_relative: 0.25\n");

    let before = new;
    apply(&mut document, Game::Ats, Command::RemoveBadge(id.clone())).unwrap();
    let after = written(&file, &document);

    assert_eq!(line_count(&after), line_count(&before) - 1);
    let start = before.find(&format!("vehicle : {id} {{")).unwrap();
    let end = start + before[start..].find("\n}").unwrap();
    let after_start = after.find(&format!("vehicle : {id} {{")).unwrap();
    let after_end = after_start + after[after_start..].find("\n}").unwrap();
    assert_eq!(&before[..start], &after[..after_start]);
    assert_eq!(&before[end..], &after[after_end..]);

    let block = &after[after_start..after_end];
    assert!(
        block
            .lines()
            .any(|line| line.trim_start() == "accessories: 46")
    );
    assert_eq!(
        block
            .lines()
            .filter(|line| line.trim_start().starts_with("accessories["))
            .count(),
        46
    );
    assert!(!block.contains("/badge/"));

    let reopened = siir::open(after.as_bytes()).unwrap();
    assert_eq!(reopened.document().unwrap().to_text(), after);
}

#[test]
fn ets2_remove_badge_changes_only_the_accessory_list() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let mut document = file.document().unwrap();
    let before = written(&file, &document);
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_truck()
        .unwrap()
        .id
        .to_owned();

    apply(&mut document, Game::Ets2, Command::RemoveBadge(id.clone())).unwrap();
    let after = written(&file, &document);

    assert_eq!(line_count(&after), line_count(&before) - 2);
    let start = before.find(&format!("vehicle : {id} {{")).unwrap();
    let end = start + before[start..].find("\n}").unwrap();
    let after_start = after.find(&format!("vehicle : {id} {{")).unwrap();
    let after_end = after_start + after[after_start..].find("\n}").unwrap();
    assert_eq!(&before[..start], &after[..after_start]);
    assert_eq!(&before[end..], &after[after_end..]);

    let block = &after[after_start..after_end];
    assert!(
        block
            .lines()
            .any(|line| line.trim_start() == "accessories: 47")
    );
    assert_eq!(
        block
            .lines()
            .filter(|line| line.trim_start().starts_with("accessories["))
            .count(),
        47
    );
    assert!(!block.contains("s_badge") && !block.contains("f_badge"));

    let others = |block: &str| -> Vec<String> {
        block
            .lines()
            .filter(|line| !line.trim_start().starts_with("accessories"))
            .map(str::to_owned)
            .collect()
    };
    assert_eq!(others(&before[start..end]), others(block));

    let unit = document.units().find(|unit| unit.id() == id).unwrap();
    for nth in 0..47 {
        let accessory = unit.get(&format!("accessories[{nth}]")).unwrap();
        let path = document
            .units()
            .find(|unit| unit.id() == accessory)
            .unwrap()
            .get("data_path")
            .unwrap();
        assert!(!path.contains("badge"), "{path}");
    }

    let reopened = siir::open(after.as_bytes()).unwrap();
    assert_eq!(reopened.document().unwrap().to_text(), after);
}
