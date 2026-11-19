//! Write flow over a copied fixture save: open, apply, save, reopen.

use std::fs;
use std::path::{Path, PathBuf};

use siir_gui::state::State;
use siir_gui::{edit, format};
use siir_save::catalog::Catalog;
use siir_save::{Command, Game, Origin, Skills};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

/// A temp game folder with one profile and one copied save.
fn game_folder(name: &str) -> PathBuf {
    game_folder_with(name, "1")
}

/// A temp game folder with one profile and the ETS2 fixture save `save`.
fn game_folder_with(name: &str, save: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("siir-gui-flow-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let profile = root.join("profiles").join("AAA");
    fs::create_dir_all(&profile).unwrap();
    fs::copy(
        fixtures().join("ets2/profile.sii"),
        profile.join("profile.sii"),
    )
    .unwrap();
    copy_tree(
        &fixtures().join("ets2/saves").join(save),
        &profile.join("save").join(save),
    );
    root
}

#[test]
fn applies_saves_and_reopens() {
    let root = game_folder("flow");
    let save_dir = root.join("profiles/AAA/save/1");
    let save = save_dir.join("game.sii");
    let original = fs::read(&save).unwrap();

    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);
    assert_eq!(state.open().unwrap().view().money, Some(2000));
    assert!(!state.edited());

    let euro = format::native_currency(Game::Ets2);
    assert!(state.apply(edit::money("12345", &euro).unwrap()).is_ok());
    assert!(state.apply(Command::SetExperience(42)).is_ok());
    assert!(
        state
            .apply(Command::SetSkills(edit::skills(
                [1, 2, 3, 4, 5],
                [true, false, false, false, false, false],
                0,
            )))
            .is_ok()
    );

    assert!(state.edited());
    assert_eq!(state.open().unwrap().view().money, Some(12_345));
    assert_eq!(state.open().unwrap().view().experience, 42);
    assert_eq!(
        state.open().unwrap().view().skills,
        Skills {
            adr: 1,
            long_dist: 1,
            heavy: 2,
            fragile: 3,
            urgent: 4,
            mechanical: 5,
        }
    );
    assert_eq!(
        fs::read(&save).unwrap(),
        original,
        "applying must not write the file"
    );

    let backup = match state.save() {
        Ok(path) => path,
        Err(_) => panic!("save failed"),
    };
    assert!(!state.edited());
    assert_eq!(state.saved_backup(), Some(backup.as_path()));
    assert_eq!(
        fs::read(&backup).unwrap(),
        original,
        "the backup holds the previous file"
    );
    assert_ne!(fs::read(&save).unwrap(), original, "the save is rewritten");
    assert!(save.strip_prefix(&root).unwrap().ends_with("game.sii"));

    state.close();
    state.open_save(ix);
    let open = state.open().expect("the saved file reopens");
    assert_eq!(open.origin(), Origin::Plain);
    assert_eq!(open.view().money, Some(12_345));
    assert_eq!(open.view().experience, 42);
    assert_eq!(open.view().skills.adr, 1);
    assert_eq!(open.view().skills.mechanical, 5);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn sets_experience_from_a_level_and_saves_it() {
    let root = game_folder("level");
    let save_dir = root.join("profiles/AAA/save/1");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);

    let catalog = Catalog::load(Game::Ets2).unwrap();
    let levels = catalog.levels();
    assert!(state.apply(edit::level(50, levels).unwrap()).is_ok());
    let experience = state.open().unwrap().view().experience;
    assert_eq!(Some(experience), levels.experience(50));
    assert_eq!(levels.level(experience), 50);
    assert!(edit::level(7_000_000, levels).is_none());

    assert!(state.save().is_ok());
    state.close();
    state.open_save(ix);
    assert_eq!(state.open().unwrap().view().experience, experience);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn maps_each_vehicle_to_its_used_market_listing() {
    let root = game_folder("listing");
    let save_dir = root.join("profiles/AAA/save/1");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);

    let listed: Vec<(String, String, i64)> = state
        .open()
        .unwrap()
        .view()
        .vehicles
        .iter()
        .filter_map(|row| {
            row.listing
                .as_ref()
                .map(|listing| (row.id.clone(), listing.id.clone(), listing.price))
        })
        .collect();
    assert_eq!(listed.len(), 26, "every listing is offered by one vehicle");
    for (vehicle, listing, _) in &listed {
        assert_ne!(vehicle, listing, "the listing id is not the vehicle id");
    }
    let (vehicle, listing, _) = listed[0].clone();
    assert!(
        state
            .open()
            .unwrap()
            .view()
            .vehicles
            .iter()
            .any(|row| row.id == vehicle && !row.owned),
        "a listed vehicle is not owned"
    );

    let euro = format::native_currency(Game::Ets2);
    assert!(
        state
            .apply(edit::used_price(&listing, "99500", &euro).unwrap())
            .is_ok()
    );
    let after = state
        .open()
        .unwrap()
        .view()
        .vehicles
        .iter()
        .filter_map(|row| {
            row.listing
                .as_ref()
                .map(|listing| (listing.id.clone(), listing.price))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        after.iter().filter(|(id, _)| id == &listing).count(),
        1,
        "the listing is still one row"
    );
    assert_eq!(
        after.iter().filter(|(_, price)| *price == 99_500).count(),
        1,
        "only the chosen listing changed"
    );

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn reads_the_profile_currency_and_converts_money() {
    let root = game_folder("currency");
    let save_dir = root.join("profiles/AAA/save/1");
    fs::write(
        root.join("profiles/AAA/config.cfg"),
        "uset g_lang \"pt_br\"\r\nuset g_currency \"2\"\r\n",
    )
    .unwrap();
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);
    let open = state.open().unwrap();
    assert_eq!(open.currency(), 2);

    let catalog = Catalog::load(Game::Ets2).unwrap();
    let koruna = &catalog.currencies()[open.currency()];
    assert_eq!(koruna.code, "CZK");
    assert_eq!(
        format::money(2000, koruna, Game::Ets2),
        format!("48 800{}", koruna.sign3)
    );
    assert!(state.apply(edit::money("24400", koruna).unwrap()).is_ok());
    assert_eq!(state.open().unwrap().view().money, Some(1000));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn falls_back_to_the_stored_currency_without_a_config() {
    let root = game_folder("no-config");
    let save_dir = root.join("profiles/AAA/save/1");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);
    assert_eq!(state.open().unwrap().currency(), 0);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn renames_the_profile_on_save() {
    let root = game_folder("profile");
    let save_dir = root.join("profiles/AAA/save/1");
    let save = save_dir.join("game.sii");
    let profile = root.join("profiles/AAA/profile.sii");
    let original_save = fs::read(&save).unwrap();
    let original_profile = fs::read(&profile).unwrap();

    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    let old_name = state.profiles()[state.saves()[ix].profile].name.clone();

    state.open_save(ix);
    assert!(state.open().unwrap().has_profile());
    assert!(state.apply(edit::profile_name("Renamed").unwrap()).is_ok());
    assert!(state.edited());
    assert_eq!(state.open().unwrap().view().profile_name, "Renamed");
    assert_eq!(fs::read(&profile).unwrap(), original_profile);

    state.close();
    assert_eq!(
        state.profiles()[state.saves()[ix].profile].name,
        old_name,
        "discarding keeps the listed name"
    );

    state.open_save(ix);
    assert_eq!(state.open().unwrap().view().profile_name, old_name);
    assert!(state.apply(edit::profile_name("Renamed").unwrap()).is_ok());
    let backup = match state.save() {
        Ok(path) => path,
        Err(_) => panic!("save failed"),
    };
    assert!(!state.edited());
    assert_eq!(backup, root.join("profiles/AAA/profile.sii.bak"));
    assert_eq!(fs::read(&backup).unwrap(), original_profile);
    assert_eq!(
        fs::read(&save).unwrap(),
        original_save,
        "an untouched save is not rewritten"
    );
    assert_eq!(state.profiles()[state.saves()[ix].profile].name, "Renamed");

    let state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    assert_eq!(state.profiles()[state.saves()[ix].profile].name, "Renamed");

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn applies_cities_and_dealerships() {
    let root = game_folder("cities");
    let save_dir = root.join("profiles/AAA/save/1");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);

    let open = state.open().expect("the fixture save opens");
    assert_eq!(open.view().visited_cities, 1);
    assert_eq!(open.view().dealerships, 0);

    assert!(state.apply(Command::SetVisitedCities(true)).is_ok());
    assert_eq!(state.open().unwrap().view().visited_cities, 388);

    assert!(state.apply(Command::SetDealershipsDiscovered(true)).is_ok());
    assert_eq!(state.open().unwrap().view().dealerships, 388);

    assert!(
        state
            .apply(Command::SetDealershipsDiscovered(false))
            .is_ok()
    );
    assert_eq!(state.open().unwrap().view().dealerships, 0);

    assert!(state.save().is_ok());
    state.close();
    state.open_save(ix);
    let open = state.open().expect("the saved file reopens");
    assert_eq!(open.view().visited_cities, 388);
    assert_eq!(open.view().dealerships, 0);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn asks_before_close_only_for_an_open_save_with_pending_work() {
    let root = game_folder("close");
    let save_dir = root.join("profiles/AAA/save/1");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    assert!(
        !state.asks_before_close(false),
        "the home has no form, so its empty fields never block closing"
    );

    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);
    assert!(!state.asks_before_close(true));
    assert!(state.asks_before_close(false), "an invalid field asks");

    let euro = format::native_currency(Game::Ets2);
    assert!(state.apply(edit::money("12345", &euro).unwrap()).is_ok());
    assert!(state.asks_before_close(true), "unsaved edits ask");
    assert!(state.save().is_ok());
    assert!(!state.asks_before_close(true));

    state.close();
    assert!(!state.asks_before_close(false));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn lists_and_edits_vehicles() {
    let root = game_folder_with("vehicles", "truck");
    let save_dir = root.join("profiles/AAA/save/truck");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);

    let vehicles = &state.open().unwrap().view().vehicles;
    let owned: Vec<String> = vehicles
        .iter()
        .take_while(|vehicle| vehicle.owned)
        .map(|vehicle| vehicle.id.clone())
        .collect();
    assert!(owned.len() >= 2, "the truck fixture owns two trucks");
    assert!(
        vehicles
            .iter()
            .skip(owned.len())
            .all(|vehicle| !vehicle.owned),
        "owned vehicles come first"
    );
    assert_eq!(vehicles.iter().filter(|vehicle| vehicle.current).count(), 1);
    let current = vehicles.iter().find(|vehicle| vehicle.current).unwrap();
    assert!(current.owned);
    let other = owned.iter().find(|id| **id != current.id).unwrap().clone();

    let row = |state: &State, id: &str| {
        let view = state.open().unwrap().view();
        let row = view
            .vehicles
            .iter()
            .find(|vehicle| vehicle.id == id)
            .unwrap();
        (row.fuel, row.odometer, row.engine_wear, row.current)
    };
    assert!(state.apply(edit::fuel(&other, 25.0)).is_ok());
    assert!(state.apply(edit::odometer(&other, "4321").unwrap()).is_ok());
    assert!(state.apply(Command::RepairVehicle(other.clone())).is_ok());
    assert!(
        state
            .apply(Command::SetCurrentVehicle(other.clone()))
            .is_ok()
    );
    assert_eq!(row(&state, &other), (0.25, Some(4321), 0.0, true));

    assert!(state.save().is_ok());
    state.close();
    state.open_save(ix);
    assert_eq!(row(&state, &other), (0.25, Some(4321), 0.0, true));

    assert!(state.apply(edit::plate(&other, "NEW 1").unwrap()).is_ok());
    assert!(
        state
            .apply(edit::plate_country(&other, "poland").unwrap())
            .is_ok()
    );
    let view = state.open().unwrap().view();
    let row = view.vehicles.iter().find(|row| row.id == other).unwrap();
    assert_eq!(row.plate, "NEW 1");
    assert_eq!(row.country.as_deref(), Some("poland"));
    assert!(state.apply(Command::FillAllVehicles).is_ok());
    let view = state.open().unwrap().view();
    assert!(
        view.vehicles
            .iter()
            .filter(|vehicle| vehicle.owned)
            .all(|vehicle| vehicle.fuel == 1.0),
        "Fill all reaches every vehicle the screen calls owned"
    );

    assert!(state.save().is_ok());
    state.close();
    state.open_save(ix);
    let view = state.open().unwrap().view();
    let saved = view.vehicles.iter().find(|row| row.id == other).unwrap();
    assert_eq!(saved.plate, "NEW 1");
    assert_eq!(saved.country.as_deref(), Some("poland"));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn lists_and_edits_trailers() {
    let root = game_folder_with("trailers", "1");
    let save_dir = root.join("profiles/AAA/save/1");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);
    let trailers = &state.open().unwrap().view().trailers;
    assert!(!trailers.is_empty(), "the fixture save has trailers");
    let id = trailers[0].id.clone();

    let row = |state: &State| {
        let view = state.open().unwrap().view();
        let row = view.trailers.iter().find(|row| row.id == id).unwrap();
        (
            row.plate.clone(),
            row.cargo_mass,
            row.body_wear,
            row.chassis_wear,
        )
    };
    assert!(
        state
            .apply(edit::trailer_plate(&id, "NEW 123").unwrap())
            .is_ok()
    );
    assert!(state.apply(edit::cargo_mass(&id, "1500").unwrap()).is_ok());
    assert!(state.apply(Command::RepairTrailer(id.clone())).is_ok());
    let (plate, mass, body, chassis) = row(&state);
    assert_eq!(plate, "NEW 123");
    assert_eq!(mass, 1500.0);
    assert_eq!((body, chassis), (0.0, 0.0));

    assert!(state.save().is_ok());
    state.close();
    state.open_save(ix);
    assert_eq!(row(&state).0, "NEW 123");
    assert_eq!(row(&state).1, 1500.0);

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn owns_every_listed_bus() {
    let root = game_folder_with("buses", "bus");
    let save_dir = root.join("profiles/AAA/save/bus");
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == save_dir)
        .expect("the copied save is listed");
    state.open_save(ix);

    let owned_models: Vec<Option<String>> = state
        .open()
        .unwrap()
        .view()
        .vehicles
        .iter()
        .filter(|vehicle| vehicle.owned)
        .map(|vehicle| vehicle.model.clone())
        .collect();
    for bus in ["volvo.9700_23", "man.lions_24"] {
        assert!(
            owned_models.contains(&Some(bus.to_owned())),
            "{bus} is owned, slot or not"
        );
    }

    fs::remove_dir_all(&root).unwrap();
}
