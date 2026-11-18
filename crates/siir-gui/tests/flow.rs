//! Write flow over a copied fixture save: open, apply, save, reopen.

use std::fs;
use std::path::{Path, PathBuf};

use siir_gui::edit;
use siir_gui::state::State;
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
        &fixtures().join("ets2/saves/1"),
        &profile.join("save").join("1"),
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

    assert!(state.apply(edit::money("12345").unwrap()).is_ok());
    assert!(state.apply(edit::experience("42").unwrap()).is_ok());
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
