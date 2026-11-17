//! Tests the catalog over a hand-written `def/` tree.

use std::path::Path;

use siir_save::Game;
use siir_save::catalog::Catalog;

fn def() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/def")
}

#[test]
fn reads_trucks_parts_and_transmissions() {
    let catalog = Catalog::generate(&def(), Game::Ets2).unwrap();
    let trucks = catalog.trucks();
    assert_eq!(trucks.len(), 1);

    let truck = &trucks[0];
    assert_eq!(truck.brand, "test");
    assert_eq!(truck.model, "model");

    assert_eq!(truck.engines.len(), 2, "the broken file is skipped");
    assert_eq!(truck.engines[0].name, "Test 100");
    assert_eq!(truck.engines[0].power_hp, 136);
    assert_eq!(truck.engines[0].torque_nm, 500);
    assert_eq!(
        truck.engines[0].path,
        "/def/vehicle/truck/test.model/engine/100.sii"
    );
    assert_eq!(truck.engines[1].power_hp, 272);
    assert_eq!(truck.engines[1].torque_nm, 1200);

    assert_eq!(truck.transmissions.len(), 2);
    let twelve = &truck.transmissions[0];
    assert_eq!(twelve.name, "Twelve");
    assert_eq!(twelve.ratios.len(), 12);
    assert_eq!(twelve.ratios[0], 12.0);
    assert_eq!(twelve.ratios[11], 0.7);
    assert!(twelve.retarder);
    assert_eq!(
        twelve.path,
        "/def/vehicle/truck/test.model/transmission/12.sii"
    );
    let six = &truck.transmissions[1];
    assert_eq!(six.ratios.len(), 6);
    assert!(!six.retarder);

    assert_eq!(truck.cabins[0].name, "Space");
    assert_eq!(
        truck.cabins[0].path,
        "/def/vehicle/truck/test.model/cabin/space.sii"
    );
    assert_eq!(truck.chassis[0].name, "4x2");
    assert_eq!(truck.paint_jobs[0].name, "Red");
    assert_eq!(
        truck.paint_jobs[0].path,
        "/def/vehicle/truck/test.model/paint_job/color1.sii"
    );

    let cargo = &catalog.cargo()[0];
    assert_eq!(cargo.id, "cargo.stones");
    assert_eq!(cargo.name, "@@cn_stones@@");
    assert_eq!(cargo.mass, 2100.0);
    assert_eq!(cargo.volume, 1.0);
    assert_eq!(cargo.reward_per_km, 0.75);
    assert_eq!(cargo.body_types, ["dumper", "flatbed"]);

    let company = &catalog.companies()[0];
    assert_eq!(company.id, "company.permanent.testco");
    assert_eq!(company.name, "Test Co");

    let trailer = &catalog.trailers()[0];
    assert_eq!(trailer.id, "trailer_def.test.single_3");
    assert_eq!(trailer.body_type, "curtainside");
    assert_eq!(trailer.chain_type, "single");
    assert_eq!(trailer.volume, 87.0);
    assert_eq!(trailer.gross_weight_limit, 35_000);
    assert_eq!(trailer.chassis_mass, 6461);
    assert_eq!(trailer.axles, 3);
    assert!((trailer.length - 13.7).abs() < 0.001);
}
