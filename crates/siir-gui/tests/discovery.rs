//! Discovery through `State` over a fixture-derived game folder.

use std::fs;
use std::path::{Path, PathBuf};

use siir_gui::state::State;
use siir_save::{Game, Origin};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// Copies fixture saves into a temp game folder with one profile.
fn game_folder(name: &str, game: &str, saves: &[&str]) -> PathBuf {
    let root = std::env::temp_dir().join(format!("siir-gui-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let profile = root.join("profiles").join("AAA");
    fs::create_dir_all(&profile).unwrap();
    let source = fixtures().join(game).join("profile.sii");
    let source = if source.is_file() {
        source
    } else {
        fixtures().join("ets2/profile.sii")
    };
    fs::copy(source, profile.join("profile.sii")).unwrap();
    for save in saves {
        copy_tree(
            &fixtures().join(game).join("saves").join(save),
            &profile.join("save").join(save),
        );
    }
    root
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

fn save_dirs(game: &str) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(fixtures().join(game).join("saves"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn profile_in(state: &State, root: &Path) -> usize {
    let path = root.join("profiles").join("AAA");
    state
        .profiles()
        .iter()
        .position(|row| row.path == path)
        .expect("the fixture profile is discovered")
}

fn fixture_saves(state: &State, root: &Path) -> Vec<usize> {
    state
        .saves()
        .iter()
        .enumerate()
        .filter(|(_, row)| row.path.starts_with(root))
        .map(|(ix, _)| ix)
        .collect()
}

#[test]
fn lists_the_saves_of_each_profile() {
    for (game, game_dir) in [(Game::Ets2, "ets2"), (Game::Ats, "ats")] {
        let sources = save_dirs(game_dir);
        let names: Vec<&str> = sources.iter().map(String::as_str).collect();
        let root = game_folder(&format!("list-{game_dir}"), game_dir, &names);
        let state = State::with_dirs(game, vec![root.clone()]);

        let profile = profile_in(&state, &root);
        let listed: Vec<_> = state.saves_of(profile).collect();
        assert_eq!(listed.len(), sources.len());
        assert!(listed.iter().all(|row| !row.name.is_empty()));
        assert_eq!(fixture_saves(&state, &root).len(), sources.len());

        fs::remove_dir_all(&root).unwrap();
    }
}

#[test]
fn opens_a_fixture_save() {
    let root = game_folder("open", "ets2", &["1", "autosave"]);
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    assert!(state.open().is_none(), "the home opens before any save");

    let profile = profile_in(&state, &root);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == root.join("profiles/AAA/save/1"))
        .expect("the fixture save is listed");
    assert_eq!(state.saves()[ix].profile, profile);
    assert_eq!(state.saves_of(profile).count(), 2);

    let path = root.join("profiles/AAA/save/1/game.sii");
    let before = fs::read(&path).unwrap();
    state.open_save(ix);

    let open = state.open().expect("the fixture save opens");
    assert_eq!(open.origin(), Origin::EncryptedBinary);
    let view = open.view();
    assert_eq!(view.money, Some(2000));
    assert_eq!(view.hq_city, "prague");
    assert_eq!(view.counts.vehicles, 3);
    assert_eq!(view.counts.trailers, 0);
    assert_eq!(view.counts.garages, 1);
    assert_eq!(view.counts.drivers, 354);
    assert_eq!(view.counts.offers, 6499);
    assert_eq!(view.current_vehicle.as_deref(), Some("renault.etech_t"));
    assert!(state.notice().is_none());
    assert_eq!(fs::read(&path).unwrap(), before, "discovery writes nothing");

    state.close();
    assert!(state.open().is_none());
    assert_eq!(state.selected_save(), Some(ix));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn opens_every_fixture_save() {
    for (game, game_dir) in [(Game::Ets2, "ets2"), (Game::Ats, "ats")] {
        let sources = save_dirs(game_dir);
        let names: Vec<&str> = sources.iter().map(String::as_str).collect();
        let root = game_folder(&format!("all-{game_dir}"), game_dir, &names);
        let mut state = State::with_dirs(game, vec![root.clone()]);
        let saves = fixture_saves(&state, &root);
        assert_eq!(saves.len(), sources.len());

        for ix in saves {
            let path = state.saves()[ix].path.clone();
            state.open_save(ix);
            assert!(state.open().is_some(), "{} did not open", path.display());
            assert!(
                state.notice().is_none(),
                "{} reported an error",
                path.display()
            );
        }

        fs::remove_dir_all(&root).unwrap();
    }
}

#[test]
fn reports_a_missing_game_file() {
    let root = game_folder("missing", "ets2", &["1"]);
    fs::remove_file(root.join("profiles/AAA/save/1/game.sii")).unwrap();
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let ix = state
        .saves()
        .iter()
        .position(|row| row.path == root.join("profiles/AAA/save/1"))
        .expect("the save directory is still listed");
    state.open_save(ix);

    assert!(state.open().is_none());
    let notice = state.notice().expect("a missing game file is reported");
    assert_eq!(notice.key, "error.open_save");
    assert!(notice.detail.ends_with("game.sii"));

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn a_profile_without_saves_has_no_rows() {
    let root = game_folder("empty", "ets2", &[]);
    let state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    let profile = profile_in(&state, &root);

    assert_eq!(state.saves_of(profile).count(), 0);
    assert!(state.open().is_none());

    fs::remove_dir_all(&root).unwrap();
}

#[test]
fn switching_game_keeps_custom_folders() {
    let root = game_folder("game", "ets2", &["1"]);
    let mut state = State::with_dirs(Game::Ets2, vec![root.clone()]);
    state.set_game(Game::Ats);

    assert_eq!(state.game(), Game::Ats);
    profile_in(&state, &root);

    fs::remove_dir_all(&root).unwrap();
}
