//! Read model and commands over the real ETS2 and ATS fixtures.

use siir::{Document, Value};
use siir_save::{
    Command, Game, LicensePlate, Skills, apply, export_accessories, import_accessories, read,
};

fn fixture(path: &str) -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(path);
    std::fs::read(path).unwrap()
}

fn written(document: &Document<'_>) -> String {
    document.to_text()
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
    visited_city: &'static str,
    vehicles: usize,
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
    trailer_id: &'static str,
    trailer_definition: &'static str,
    trailer_mass: f32,
    trailer_plate: &'static str,
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
        visited_city: "prague",
        vehicles: 27,
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
        trailer_id: "_nameless.7f9f.ff9f.5450",
        trailer_definition: "trailer_def.wielton.scrapm.ch_3_10.scrapm",
        trailer_mass: 21_000.0,
        trailer_plate: "4A1<offset vshift=-8><img src=/material/ui/lp/czech/b.mat width=20 height=40 color=FFFFFFFF><offset vshift=8>8956|czech",
        job_cargo: "cargo.stones",
        job_km: 183,
    },
    Case {
        name: "ats",
        fixture: "ats/saves/1/game.sii",
        game: Game::Ats,
        money: 1500,
        hq_city: "seattle",
        visited_city: "seattle",
        vehicles: 13,
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
        trailer_id: "_nameless.7fef.7fdb.2310",
        trailer_definition: "trailer_def.scs.flatbed.single_45sp.curtain",
        trailer_mass: f32::from_bits(0x468e_4b34),
        trailer_plate: "75705TR|washington",
        job_cargo: "cargo.mortar",
        job_km: 51,
    },
];

fn assert_case(case: &Case) {
    let bytes = fixture(case.fixture);
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, case.game).unwrap();
    let name = case.name;

    assert_eq!(save.game, case.game, "{name}");
    assert_eq!(
        save.bank.as_ref().unwrap().money_account,
        case.money,
        "{name}"
    );
    assert_eq!(save.experience, 0, "{name}");
    assert_eq!(save.skills, Skills::default(), "{name}");
    assert_eq!(save.visited_cities, [case.visited_city], "{name}");
    assert!(save.unlocked_dealers.is_empty(), "{name}");

    let player = save.player.as_ref().unwrap();
    assert_eq!(player.hq_city, case.hq_city, "{name}");
    assert_eq!(player.my_vehicles.len(), 3, "{name}");
    assert!(player.trucks.is_empty(), "{name}");
    assert!(player.trailers.is_empty(), "{name}");
    assert_eq!(player.drivers.len(), 1, "{name}");
    assert!(player.dismissed_drivers.is_empty(), "{name}");
    assert!(player.assigned_vehicles.is_some(), "{name}");
    assert!(player.current_job.is_some(), "{name}");

    assert_eq!(save.vehicles.len(), case.vehicles, "{name}");
    assert_eq!(save.trailers.len(), 1, "{name}");
    assert_eq!(save.garages.len(), case.garages, "{name}");
    let starter = save
        .garages
        .iter()
        .find(|garage| garage.status == 6)
        .unwrap();
    assert_eq!(starter.drivers.len(), 1, "{name}");
    assert_eq!(starter.vehicles.len(), 1, "{name}");
    assert_eq!(
        save.garages
            .iter()
            .filter(|garage| garage.status == 6)
            .count(),
        1,
        "{name}"
    );
    assert_eq!(save.drivers.len(), case.drivers, "{name}");
    assert_eq!(save.driver_pool.len(), case.drivers, "{name}");
    assert_eq!(save.drivers[0].state, Some(1), "{name}");
    assert_eq!(save.companies.len(), case.companies, "{name}");
    assert_eq!(save.companies[0].permanent_data, case.company, "{name}");
    assert_eq!(save.jobs.len(), case.drivers, "{name}");
    assert_eq!(save.jobs[0].cargo, None, "{name}");
    assert_eq!(save.offers.len(), case.offers, "{name}");
    assert_eq!(save.offers[0].target, case.target, "{name}");
    assert_eq!(save.offers[0].shortest_distance_km, case.distance, "{name}");
    assert_eq!(save.used_trucks.len(), case.used, "{name}");
    assert_eq!(save.used_trucks[0].price, case.price, "{name}");
    assert!(save.offense.is_some(), "{name}");
    assert!(save.bus_job.is_none(), "{name}");
    assert!(save.car_job.is_none(), "{name}");
    assert!(save.car_offers.is_empty(), "{name}");
    assert_eq!(save.bus_job_log.as_ref().unwrap().entries, 0, "{name}");
    assert_eq!(save.car_job_log.as_ref().unwrap().entries, 0, "{name}");

    let truck = save.current_vehicle().unwrap();
    assert_eq!(truck.id, case.truck_id, "{name}");
    assert_eq!(truck.trailer, Some(case.trailer_id), "{name}");
    assert_eq!(truck.model, Some(case.model), "{name}");
    assert_eq!(truck.data_path, Some(case.data_path), "{name}");
    assert_eq!(truck.engine, Some(case.engine), "{name}");
    assert_eq!(truck.engine_wear, case.engine_wear, "{name}");
    assert_eq!(truck.fuel_relative, case.fuel, "{name}");
    assert!(truck.placement.unwrap().starts_with('('), "{name}");
    assert!(save.vehicle(truck.id).is_some(), "{name}");

    let trailer = save.current_trailer().unwrap();
    assert_eq!(trailer.id, case.trailer_id, "{name}");
    assert_eq!(trailer.definition, case.trailer_definition, "{name}");
    assert_eq!(trailer.cargo_mass, case.trailer_mass, "{name}");
    assert_eq!(trailer.license_plate, case.trailer_plate, "{name}");
    assert_eq!(trailer.slave_trailer, None, "{name}");
    assert_eq!(trailer.body_wear, 0.0, "{name}");
    assert_eq!(trailer.chassis_wear, 0.0, "{name}");
    assert!(save.trailer(trailer.id).is_some(), "{name}");

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
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let save = read(&document, Game::Ets2).unwrap();
    let id = save.current_vehicle().unwrap().id.to_owned();
    let target = save.used_trucks[0].truck.to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::RepairVehicle(id.clone()),
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 8);
    for (old, new) in changed {
        let name = old.trim_start().split(':').next().unwrap();
        assert!(
            name.split('[').next().unwrap().ends_with("_wear")
                || name.split('[').next().unwrap().ends_with("_unfixable")
        );
        assert!(new.trim_start().ends_with(": 0\n"));
    }
    let unit = document.units().find(|unit| unit.id() == id).unwrap();
    for name in [
        "engine_wear",
        "transmission_wear",
        "cabin_wear",
        "chassis_wear",
        "engine_wear_unfixable",
        "transmission_wear_unfixable",
        "cabin_wear_unfixable",
        "chassis_wear_unfixable",
    ] {
        assert_eq!(unit.get(name), Some("0"), "{name}");
    }
    for name in ["wheels_wear", "wheels_wear_unfixable"] {
        for nth in 0..2 {
            let name = format!("{name}[{nth}]");
            assert_eq!(unit.get(&name), Some("0"), "{name}");
        }
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
    let new = written(&document);
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
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 3);
    for (old, new) in changed {
        assert!(matches!(
            old.trim_start().split(':').next().unwrap(),
            "odometer" | "integrity_odometer" | "trip_distance_km"
        ));
        assert!(new.trim_start().ends_with(": 1234\n"));
    }
    let save = read(&document, Game::Ets2).unwrap();
    assert_eq!(save.vehicle(&id).unwrap().odometer, Some(1234));

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
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "license_plate: \"TEST-123\"\n");

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetCurrentVehicle(target.clone()),
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), format!("vehicle: {target}\n"));

    let save = read(&document, Game::Ets2).unwrap();
    assert_eq!(save.current_vehicle().unwrap().id, target);

    let reopened = siir::open(new.as_bytes()).unwrap();
    assert_eq!(reopened.document().to_text(), new);
}

#[test]
fn bulk_commands_reach_the_assigned_vehicle_and_trailer() {
    for (path, game) in [
        ("ets2/saves/1/game.sii", Game::Ets2),
        ("ats/saves/1/game.sii", Game::Ats),
    ] {
        let bytes = fixture(path);
        let file = siir::open(&bytes).unwrap();
        let source = file.document().to_text();
        let mut document = Document::parse(&source).unwrap();
        let (truck, trailer) = {
            let save = read(&document, game).unwrap();
            (
                save.current_vehicle().unwrap().id.to_owned(),
                save.current_trailer().unwrap().id.to_owned(),
            )
        };

        apply(
            &mut document,
            game,
            Command::SetFuel {
                id: truck.clone(),
                level: 0.25,
            },
        )
        .unwrap();
        let old = written(&document);
        apply(&mut document, game, Command::FillAllVehicles).unwrap();
        let new = written(&document);
        let changed = changed_lines(&old, &new);
        assert_eq!(changed.len(), 1, "{path}");
        assert_eq!(changed[0].1.trim_start(), "fuel_relative: 1.0\n", "{path}");

        document
            .get(&trailer)
            .unwrap()
            .set("trailer_body_wear", Value::Int(1))
            .unwrap();
        apply(&mut document, game, Command::RepairAllTrailers).unwrap();
        assert_eq!(
            document.get(&trailer).unwrap().get("trailer_body_wear"),
            Some("0"),
            "{path}"
        );

        document
            .get(&truck)
            .unwrap()
            .set("engine_wear", Value::Int(1))
            .unwrap();
        apply(&mut document, game, Command::RepairAllVehicles).unwrap();
        assert_eq!(
            document.get(&truck).unwrap().get("engine_wear"),
            Some("0"),
            "{path}"
        );
    }
}

#[test]
fn owned_buses_and_cars_count_and_get_filled() {
    for (path, game) in [
        ("ets2/saves/bus/game.sii", Game::Ets2),
        ("ats/saves/car/game.sii", Game::Ats),
    ] {
        let bytes = fixture(path);
        let file = siir::open(&bytes).unwrap();
        let source = file.document().to_text();
        let mut document = Document::parse(&source).unwrap();
        let listed: Vec<String> = {
            let save = read(&document, game).unwrap();
            let player = save.player.as_ref().unwrap();
            let listed: Vec<String> = player
                .buses
                .iter()
                .chain(&player.cars)
                .map(|id| id.to_string())
                .collect();
            assert!(listed.len() >= 2, "{path} lists more than one bus or car");
            assert!(
                listed
                    .iter()
                    .any(|id| save.vehicle(id).is_some_and(|v| v.slot.is_none())),
                "{path} has a listed vehicle outside any slot"
            );
            for id in &listed {
                assert!(save.owns(id), "{path}: {id} is owned");
            }
            listed
        };
        for id in &listed {
            apply(
                &mut document,
                game,
                Command::SetFuel {
                    id: id.clone(),
                    level: 0.25,
                },
            )
            .unwrap();
        }
        apply(&mut document, game, Command::FillAllVehicles).unwrap();
        let save = read(&document, game).unwrap();
        for id in &listed {
            assert_eq!(save.vehicle(id).unwrap().fuel_relative, 1.0, "{path}: {id}");
        }
    }
}

#[test]
fn visited_cities_and_dealers_handle_a_real_save() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();

    apply(&mut document, Game::Ets2, Command::SetVisitedCities(true)).unwrap();
    apply(
        &mut document,
        Game::Ets2,
        Command::SetDealershipsDiscovered(true),
    )
    .unwrap();
    let text = written(&document);
    assert!(text.contains("visited_cities: 388\n"));
    assert!(text.contains("unlocked_dealers: 388\n"));

    apply(&mut document, Game::Ets2, Command::SetVisitedCities(false)).unwrap();
    apply(
        &mut document,
        Game::Ets2,
        Command::SetDealershipsDiscovered(false),
    )
    .unwrap();
    let text = written(&document);
    assert!(text.contains("visited_cities: 0\n"));
    assert!(text.contains("unlocked_dealers: 0\n"));
    assert_eq!(
        siir::open(text.as_bytes()).unwrap().document().to_text(),
        text
    );
}

#[test]
fn ats_commands_change_only_expected_lines() {
    let bytes = fixture("ats/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ats)
        .unwrap()
        .current_vehicle()
        .unwrap()
        .id
        .to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ats,
        Command::SetFuel {
            id: id.clone(),
            level: 0.25,
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "fuel_relative: 0.25\n");

    let before = new;
    apply(&mut document, Game::Ats, Command::RemoveBadge(id.clone())).unwrap();
    let after = written(&document);

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
    assert_eq!(reopened.document().to_text(), after);
}

#[test]
fn ets2_engine_and_transmission_survive_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_vehicle()
        .unwrap()
        .id
        .to_owned();

    let engine = "/def/vehicle/truck/renault.etech_t/engine/330kw.sii";
    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetEngine {
            id: id.clone(),
            path: engine.into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].1.trim_start(),
        format!("data_path: \"{engine}\"\n")
    );
    assert_eq!(
        read(&document, Game::Ets2)
            .unwrap()
            .current_vehicle()
            .unwrap()
            .engine,
        Some(engine)
    );

    let transmission = "/def/vehicle/truck/renault.etech_t/transmission/test.sii";
    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetTransmission {
            id: id.clone(),
            path: transmission.into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].1.trim_start(),
        format!("data_path: \"{transmission}\"\n")
    );

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    let truck = save.current_vehicle().unwrap();
    assert_eq!(truck.engine, Some(engine));
    assert_eq!(truck.transmission, Some(transmission));
}

#[test]
fn ets2_cabin_and_chassis_survive_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_vehicle()
        .unwrap()
        .id
        .to_owned();

    let cabin = "/def/vehicle/truck/renault.etech_t/cabin/test.sii";
    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetCabin {
            id: id.clone(),
            path: cabin.into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].1.trim_start(),
        format!("data_path: \"{cabin}\"\n")
    );

    let chassis = "/def/vehicle/truck/renault.etech_t/chassis/test.sii";
    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetChassis {
            id: id.clone(),
            path: chassis.into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].1.trim_start(),
        format!("data_path: \"{chassis}\"\n")
    );

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    let truck = save.current_vehicle().unwrap();
    assert_eq!(truck.cabin, Some(cabin));
    assert_eq!(truck.chassis, Some(chassis));
}

#[test]
fn ets2_paint_survives_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_vehicle()
        .unwrap()
        .id
        .to_owned();

    let paint = "/def/vehicle/truck/renault.etech_t/paint_job/test.sii";
    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetPaint {
            id,
            path: paint.into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].1.trim_start(),
        format!("data_path: \"{paint}\"\n")
    );

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    assert_eq!(save.current_vehicle().unwrap().paint_job, Some(paint));
}

#[test]
fn ets2_accessory_path_survives_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let (id, accessory_id) = {
        let save = read(&document, Game::Ets2).unwrap();
        let truck = save.current_vehicle().unwrap();
        let accessory = truck
            .accessories
            .iter()
            .find(|accessory| accessory.slot.is_some())
            .unwrap();
        (truck.id.to_owned(), accessory.id.to_owned())
    };

    let path = "/def/vehicle/truck/renault.etech_t/accessory/test.sii";
    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetAccessory {
            id: id.clone(),
            accessory: accessory_id.clone(),
            path: path.into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(
        changed[0].1.trim_start(),
        format!("data_path: \"{path}\"\n")
    );

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    let truck = save.current_vehicle().unwrap();
    let accessory = truck
        .accessories
        .iter()
        .find(|accessory| accessory.id == accessory_id)
        .unwrap();
    assert_eq!(accessory.path, Some(path));
}

fn accessories_export_and_import(path: &str, game: Game) {
    let bytes = fixture(path);
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let (source_id, source_count, target_id) = {
        let save = read(&document, game).unwrap();
        let source = save.current_vehicle().unwrap();
        let target = save
            .vehicles
            .iter()
            .find(|truck| truck.id != source.id)
            .unwrap();
        (
            source.id.to_owned(),
            source.accessories.len(),
            target.id.to_owned(),
        )
    };

    let block = export_accessories(&document, &source_id).unwrap();
    import_accessories(&mut document, &target_id, &block).unwrap();

    let save = read(&document, game).unwrap();
    let target = save.vehicle(&target_id).unwrap();
    assert_eq!(target.accessories.len(), source_count);
    assert!(
        target
            .accessories
            .iter()
            .any(|accessory| accessory.path.is_some())
    );

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, game).unwrap();
    assert_eq!(
        save.vehicle(&target_id).unwrap().accessories.len(),
        source_count
    );
    assert!(
        export_accessories(document, &target_id)
            .unwrap()
            .contains("vehicle_accessory")
    );
}

#[test]
fn ets2_accessories_export_and_import_between_vehicles() {
    accessories_export_and_import("ets2/saves/1/game.sii", Game::Ets2);
}

#[test]
fn ats_accessories_export_and_import_between_vehicles() {
    accessories_export_and_import("ats/saves/1/game.sii", Game::Ats);
}

#[test]
fn ets2_used_truck_price_survives_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2).unwrap().used_trucks[0]
        .id
        .to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetUsedTruckPrice {
            id: id.clone(),
            price: 100_000,
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "price: 100000\n");

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    assert_eq!(save.used_trucks[0].price, 100_000);
}

#[test]
fn ets2_money_survives_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    apply(&mut document, Game::Ets2, Command::SetMoney(12_345)).unwrap();
    let out = document.to_text().into_bytes();

    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    assert_eq!(save.bank.unwrap().money_account, 12_345);
}

#[test]
fn owned_file_applies_several_commands_to_a_binary_save() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let mut file = siir::open(&bytes).unwrap();
    assert_eq!(
        file.origin(),
        siir::container::Origin::EncryptedBinary,
        "the fixture must exercise the binary path"
    );

    file.with_document_mut(|document| {
        apply(document, Game::Ets2, Command::SetMoney(12_345)).unwrap();
        apply(document, Game::Ets2, Command::SetExperience(42)).unwrap();
    });

    let out = file.write();
    let reopened = siir::open(&out).unwrap();
    let save = read(reopened.document(), Game::Ets2).unwrap();
    assert_eq!(save.bank.unwrap().money_account, 12_345);
    assert_eq!(save.experience, 42);
}

#[test]
fn ets2_trailer_commands_change_only_expected_lines() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_trailer()
        .unwrap()
        .id
        .to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::RepairTrailer(id.clone()),
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 6);
    for (old, new) in changed {
        assert!(old.trim_start().starts_with("wheels_wear"));
        assert!(new.trim_start().ends_with(": 0\n"));
    }

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetCargoMass {
            id: id.clone(),
            mass: 15_000.0,
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "cargo_mass: 15000.0\n");

    let old = new;
    apply(
        &mut document,
        Game::Ets2,
        Command::SetTrailerLicensePlate {
            id,
            plate: "NEW".into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "license_plate: \"NEW\"\n");

    let reopened = siir::open(new.as_bytes()).unwrap();
    assert_eq!(reopened.document().to_text(), new);
}

#[test]
fn ats_trailer_commands_change_only_expected_lines() {
    let bytes = fixture("ats/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ats)
        .unwrap()
        .current_trailer()
        .unwrap()
        .id
        .to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ats,
        Command::SetCargoMass {
            id: id.clone(),
            mass: 10_000.0,
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "cargo_mass: 10000.0\n");

    let old = new;
    apply(
        &mut document,
        Game::Ats,
        Command::SetTrailerLicensePlate {
            id,
            plate: "NEW".into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "license_plate: \"NEW\"\n");

    let reopened = siir::open(new.as_bytes()).unwrap();
    assert_eq!(reopened.document().to_text(), new);
}

#[test]
fn ets2_remove_badge_changes_only_the_accessory_list() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let before = written(&document);
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_vehicle()
        .unwrap()
        .id
        .to_owned();

    apply(&mut document, Game::Ets2, Command::RemoveBadge(id.clone())).unwrap();
    let after = written(&document);

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
    assert_eq!(reopened.document().to_text(), after);
}

#[test]
fn ets2_plate_country_changes_only_the_plate() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2)
        .unwrap()
        .current_vehicle()
        .unwrap()
        .id
        .to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetLicensePlateCountry {
            id,
            country: "poland".into(),
        },
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert!(changed[0].0.trim_start().starts_with("license_plate:"));
    assert!(changed[0].1.trim_end().ends_with("|poland\""));

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    let plate = LicensePlate::parse(save.current_vehicle().unwrap().license_plate);
    assert_eq!(plate.country, Some("poland"));
    assert!(!plate.text.is_empty());
}

#[test]
fn ets2_used_market_expiration_and_assortment_survive_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2).unwrap().used_trucks[0]
        .id
        .to_owned();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetUsedTruckExpiration {
            id: id.clone(),
            time: 10_000,
        },
    )
    .unwrap();
    apply(
        &mut document,
        Game::Ets2,
        Command::SetUsedAssortmentNextGeneration(20_000),
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 2);
    assert!(
        changed
            .iter()
            .any(|(_, new)| new.trim_start() == "expiration_game_time: 10000\n")
    );
    assert!(
        changed
            .iter()
            .any(|(_, new)| new.trim_start() == "next_generation_game_time: 20000\n")
    );

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    let listing = save.used_trucks.iter().find(|used| used.id == id).unwrap();
    assert_eq!(listing.expiration_game_time, 10_000);
    let assortment = save.used_assortment.as_ref().unwrap();
    assert_eq!(assortment.next_generation_game_time, 20_000);
    assert!(!assortment.trucks.is_empty());
}

#[test]
fn ets2_offer_edits_change_only_expected_lines() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2).unwrap().offers[0].id.to_owned();

    let old = written(&document);
    for command in [
        Command::SetOfferUnits {
            id: id.clone(),
            units: 12,
        },
        Command::SetOfferCargo {
            id: id.clone(),
            cargo: "cargo.test_freight".into(),
        },
        Command::SetOfferTarget {
            id: id.clone(),
            target: "test.company".into(),
        },
        Command::SetOfferDistance {
            id: id.clone(),
            km: 1,
        },
        Command::SetOfferFill {
            id: id.clone(),
            fill: 0.25,
        },
        Command::SetOfferUrgency {
            id: id.clone(),
            urgency: 1,
        },
        Command::SetOfferExpiration {
            id: id.clone(),
            time: 99_999,
        },
    ] {
        apply(&mut document, Game::Ets2, command).unwrap();
    }
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 7);
    for (_, new) in changed {
        assert!(
            [
                "units_count: 12\n",
                "fill_ratio: 0.25\n",
                "urgency: 1\n",
                "expiration_time: 99999\n",
                "cargo: cargo.test_freight\n",
                "target: \"test.company\"\n",
                "shortest_distance_km: 1\n",
            ]
            .contains(&new.trim_start())
        );
    }

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    let offer = save.offers.iter().find(|offer| offer.id == id).unwrap();
    assert_eq!(offer.units_count, 12);
    assert_eq!(offer.fill_ratio, 0.25);
    assert_eq!(offer.urgency, Some(1));
    assert_eq!(offer.expiration_time, Some(99_999));
    assert_eq!(offer.cargo, "cargo.test_freight");
    assert_eq!(offer.target, "test.company");
    assert_eq!(offer.shortest_distance_km, 1);
}

#[test]
fn ets2_current_job_cargo_survives_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetCurrentJobCargo("cargo.plast_film".into()),
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert_eq!(changed[0].1.trim_start(), "cargo: cargo.plast_film\n");

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    assert_eq!(save.player_job.unwrap().cargo, Some("cargo.plast_film"));
}

#[test]
fn ets2_driver_skills_survive_reopen() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let id = read(&document, Game::Ets2).unwrap().drivers[0]
        .id
        .to_owned();
    let skills = Skills {
        adr: 6,
        long_dist: 5,
        heavy: 4,
        fragile: 3,
        urgent: 2,
        mechanical: 1,
    };

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetDriverSkills {
            id: id.clone(),
            skills,
        },
    )
    .unwrap();
    let new = written(&document);
    for (old, _) in changed_lines(&old, &new) {
        let name = old.trim_start().split(':').next().unwrap();
        assert!(
            [
                "adr",
                "long_dist",
                "heavy",
                "fragile",
                "urgent",
                "mechanical"
            ]
            .contains(&name)
        );
    }

    let out = document.to_text().into_bytes();
    let reopened = siir::open(&out).unwrap();
    let document = reopened.document();
    let save = read(document, Game::Ets2).unwrap();
    assert_eq!(
        save.drivers
            .iter()
            .find(|driver| driver.id == id)
            .unwrap()
            .skills,
        skills
    );
}

#[test]
fn reads_bus_stops_and_line_offers() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, Game::Ets2).unwrap();

    assert_eq!(save.bus_stops.len(), 322);
    let stop = &save.bus_stops[0];
    assert_eq!(stop.id, "bus_stop.city.tekirdag");
    assert_eq!(stop.city, "tekirdag");
    assert!(!stop.discovered);
    assert_eq!(stop.offers.len(), 8);

    assert_eq!(save.bus_offers.len(), 2326);
    let offer = &save.bus_offers[0];
    assert_eq!(offer.kind, "regular");
    assert_eq!(offer.source, "bus_stop.city.tekirdag");
    assert_eq!(offer.targets, ["city.thessaloniki", "city.larissa"]);
    assert_eq!(offer.distances, [490, 135]);
    assert_eq!(offer.times, [485, 145]);
    assert_eq!(offer.departure_time_of_day, 250);
    assert_eq!(offer.bus, "scania_touring_23_129h4_450_2");
    assert_eq!(offer.expiration_time, 910);

    let player = save.player.as_ref().unwrap();
    assert!(player.buses.is_empty());
    assert!(player.cars.is_empty());
    assert_eq!(player.current_bus_job, None);
    assert_eq!(player.selected_bus_offer, None);
    assert_eq!(player.current_car_job, None);
}

#[test]
fn ats_has_no_bus_data() {
    let bytes = fixture("ats/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, Game::Ats).unwrap();
    assert!(save.bus_stops.is_empty());
    assert!(save.bus_offers.is_empty());
}

#[test]
fn discover_bus_stops_changes_only_the_stops() {
    let bytes = fixture("ets2/saves/1/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let missing = read(&document, Game::Ets2)
        .unwrap()
        .bus_stops
        .iter()
        .filter(|stop| !stop.discovered)
        .count();

    let old = written(&document);
    apply(&mut document, Game::Ets2, Command::DiscoverBusStops).unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), missing);
    for (old, new) in changed {
        assert!(old.trim_start().starts_with("discovered: false"));
        assert!(new.trim_start().starts_with("discovered: true"));
    }

    apply(&mut document, Game::Ets2, Command::DiscoverBusStops).unwrap();
    assert_eq!(written(&document), new);

    let reopened = siir::open(new.as_bytes()).unwrap();
    let reopened = reopened.document();
    let save = read(reopened, Game::Ets2).unwrap();
    assert_eq!(save.bus_stops.len(), 322);
    assert!(save.bus_stops.iter().all(|stop| stop.discovered));
}

#[test]
fn reads_a_bus_job() {
    let bytes = fixture("ets2/saves/bus/game.sii");
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, Game::Ets2).unwrap();

    let job = save.bus_job.as_ref().unwrap();
    assert_eq!(job.kind, "regular");
    assert_eq!(job.company_bus, None);
    assert_eq!(job.stage_index, 0);
    assert_eq!(job.passengers, 32);
    assert_eq!(job.stages.len(), 2);
    let stage = &job.stages[0];
    assert_eq!(stage.source, "bus_stop.city.prague");
    assert_eq!(stage.target, "bus_stop.city.nurnberg");
    assert_eq!(stage.planned_distance, 256);
    assert_eq!(stage.planned_departure_time, Some(2655));
    assert_eq!(stage.planned_arrival_time, None);
    assert_eq!(stage.planned_stage_time, 225);
    assert_eq!(stage.planned_ferry_time, 0);
    assert_eq!(stage.planned_entering_passengers, 32);
    assert_eq!(stage.planned_leaving_passengers, 18);
    assert_eq!(stage.departure_time, Some(2655));
    assert_eq!(stage.arrival_time, None);
    assert_eq!(stage.traveled_distance, 0);
    assert_eq!(stage.satisfaction, -1.0);

    assert_eq!(
        save.current_vehicle().and_then(|vehicle| vehicle.model),
        Some("volvo.9700_23")
    );
    let player = save.player.as_ref().unwrap();
    assert_eq!(player.buses.len(), 2);
    assert_eq!(player.current_bus_job, Some(job.id));
    assert_eq!(save.bus_job_log.as_ref().unwrap().version, 0);
    assert_eq!(save.bus_job_log.as_ref().unwrap().cached_jobs, None);
}

#[test]
fn reads_car_offers_and_a_car_job() {
    let bytes = fixture("ats/saves/car/game.sii");
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, Game::Ats).unwrap();

    assert_eq!(save.car_offers.len(), 1000);
    let empty = &save.car_offers[0];
    assert_eq!(empty.source, "");
    assert_eq!(empty.job, None);
    assert_eq!(empty.cargo, None);
    assert_eq!(empty.expiration_time, None);
    assert_eq!(empty.urgency, None);

    let offer = save
        .car_offers
        .iter()
        .find(|offer| offer.job == Some("car_job.psmpl_0"))
        .unwrap();
    assert_eq!(offer.source, "cm_min_plnt.seattle");
    assert_eq!(offer.target, "pt_trk_dlr.tacoma");
    assert_eq!(offer.expiration_time, Some(1498));
    assert_eq!(offer.urgency, Some(0));
    assert_eq!(offer.job_type, "delivery");
    assert_eq!(offer.shortest_distance_km, 50);
    assert_eq!(offer.cargo, Some("car_cargo.prod_smpl"));
    assert_eq!(offer.company_car, "ford_victoria_06_black");
    assert_eq!(offer.units_count, 1);

    let job = save.car_job.as_ref().unwrap();
    assert_eq!(job.job_type, "delivery");
    assert_eq!(job.source_company, "company.volatile.cm_min_plnt.seattle");
    assert_eq!(
        job.target_company,
        "company.volatile.ftf_food_pln.bellingham"
    );
    assert_eq!(job.planned_distance_km, 181);
    assert_eq!(job.urgency, 0);
    assert_eq!(job.time_lower_limit, 700);
    assert_eq!(job.time_upper_limit, 1100);
    assert_eq!(job.cargo, Some("car_cargo.pdoc"));
    assert_eq!(job.units_count, 1);
    assert_eq!(job.traveled_distance, 0);

    let player = save.player.as_ref().unwrap();
    assert_eq!(player.cars.len(), 3);
    assert_eq!(player.current_car_job, Some(job.id));
    assert_eq!(save.car_job_log.as_ref().unwrap().version, 1);
    assert_eq!(save.car_job_log.as_ref().unwrap().cached_jobs, Some(0));
}

#[test]
fn sets_the_selected_bus_offer() {
    let bytes = fixture("ets2/saves/bus/game.sii");
    let file = siir::open(&bytes).unwrap();
    let source = file.document().to_text();
    let mut document = Document::parse(&source).unwrap();
    let offer = {
        let save = read(&document, Game::Ets2).unwrap();
        save.bus_offers[0].id.to_owned()
    };

    let old = written(&document);
    apply(
        &mut document,
        Game::Ets2,
        Command::SetBusLineOffer(offer.clone()),
    )
    .unwrap();
    let new = written(&document);
    let changed = changed_lines(&old, &new);
    assert_eq!(changed.len(), 1);
    assert!(
        changed[0]
            .0
            .trim_start()
            .starts_with("selected_bus_offer: null")
    );
    assert_eq!(
        changed[0].1.trim_start(),
        format!("selected_bus_offer: {offer}\n")
    );

    let reopened = siir::open(new.as_bytes()).unwrap();
    let reopened = reopened.document();
    let save = read(reopened, Game::Ets2).unwrap();
    assert_eq!(
        save.player.as_ref().unwrap().selected_bus_offer,
        Some(offer.as_str())
    );
}

#[test]
fn truck_saves_have_owned_vehicles_and_no_side_job() {
    let bytes = fixture("ets2/saves/truck/game.sii");
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, Game::Ets2).unwrap();
    assert!(save.bus_job.is_none());
    assert_eq!(save.player.as_ref().unwrap().buses.len(), 2);
    assert_eq!(save.bus_stops.len(), 322);

    let bytes = fixture("ats/saves/truck/game.sii");
    let file = siir::open(&bytes).unwrap();
    let document = file.document();
    let save = read(document, Game::Ats).unwrap();
    assert!(save.car_job.is_none());
    assert_eq!(save.player.as_ref().unwrap().cars.len(), 3);
    assert_eq!(save.car_offers.len(), 1000);
}
