//! Session open, read, edit, and atomic save.

use std::fs;
use std::path::{Path, PathBuf};

use siir::container::Origin;
use siir_save::profiles::{RootKind, profiles};
use siir_save::{Command, Game, Session};

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(path)
}

fn temp(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("siir-session-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn entries(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn reads_edits_and_reopens_from_bytes() {
    let bytes = fs::read(fixture("ets2/saves/1/game.sii")).unwrap();
    let mut session = Session::from_bytes(Game::Ets2, bytes).unwrap();
    assert_eq!(session.origin(), Origin::EncryptedBinary);
    assert!(!session.edited());
    assert_eq!(
        session.read().unwrap().bank.as_ref().unwrap().money_account,
        2000
    );

    session.apply(Command::SetMoney(12_345)).unwrap();
    assert_eq!(
        session.read().unwrap().bank.as_ref().unwrap().money_account,
        12_345
    );
    session.apply(Command::SetExperience(42)).unwrap();
    assert_eq!(session.read().unwrap().experience, 42);
    assert!(session.edited());

    let bytes = session.to_bytes();
    assert!(session.edited(), "to_bytes must not clear the dirty flag");

    let reopened = Session::from_bytes(Game::Ets2, bytes).unwrap();
    let save = reopened.read().unwrap();
    assert_eq!(save.bank.as_ref().unwrap().money_account, 12_345);
    assert_eq!(save.experience, 42);
}

#[test]
fn failed_apply_keeps_the_session_edited() {
    let bytes = fs::read(fixture("ets2/saves/1/game.sii")).unwrap();
    let mut session = Session::from_bytes(Game::Ets2, bytes).unwrap();
    assert!(
        session
            .apply(Command::RepairVehicle("missing".into()))
            .is_err()
    );
    assert!(
        session.edited(),
        "a failed command may have changed the document"
    );
}

#[test]
fn save_round_trips_with_a_backup() {
    let dir = temp("save");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.sii");
    let original = fs::read(fixture("ets2/saves/1/game.sii")).unwrap();
    fs::write(&path, &original).unwrap();

    let mut session = Session::open(Game::Ets2, &path).unwrap();
    session.apply(Command::SetMoney(12_345)).unwrap();
    session.save(&path).unwrap();
    assert!(!session.edited());

    assert_eq!(fs::read(dir.join("game.sii.bak")).unwrap(), original);
    let written = fs::read(&path).unwrap();
    assert!(written.starts_with(b"SiiNunit"));
    assert_eq!(entries(&dir), ["game.sii", "game.sii.bak"]);

    let reopened = Session::open(Game::Ets2, &path).unwrap();
    assert_eq!(
        reopened
            .read()
            .unwrap()
            .bank
            .as_ref()
            .unwrap()
            .money_account,
        12_345
    );

    session.apply(Command::SetMoney(7)).unwrap();
    session.save(&path).unwrap();
    assert_eq!(
        fs::read(dir.join("game.sii.bak")).unwrap(),
        written,
        "the second save must back up the first result"
    );
    assert_eq!(entries(&dir), ["game.sii", "game.sii.bak"]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn save_to_a_new_path_writes_no_backup() {
    let dir = temp("new");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.sii");
    let bytes = fs::read(fixture("ets2/saves/1/game.sii")).unwrap();

    let mut session = Session::from_bytes(Game::Ets2, bytes).unwrap();
    session.apply(Command::SetMoney(1)).unwrap();
    session.save(&path).unwrap();

    assert!(path.is_file());
    assert!(!dir.join("game.sii.bak").exists());
    assert_eq!(entries(&dir), ["game.sii"]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn failed_save_keeps_the_original_and_the_dirty_flag() {
    let dir = temp("fail");
    fs::create_dir_all(&dir).unwrap();
    let path = dir.join("game.sii");
    let original = fs::read(fixture("ets2/saves/1/game.sii")).unwrap();
    fs::write(&path, &original).unwrap();
    fs::create_dir(dir.join("game.sii.bak")).unwrap();

    let mut session = Session::open(Game::Ets2, &path).unwrap();
    session.apply(Command::SetMoney(12_345)).unwrap();
    assert!(session.save(&path).is_err());
    assert!(session.edited());
    assert_eq!(fs::read(&path).unwrap(), original);
    assert_eq!(entries(&dir), ["game.sii", "game.sii.bak"]);

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn session_is_send_and_static() {
    fn assert_send<T: Send>() {}
    fn assert_static<T: 'static>() {}
    assert_send::<Session>();
    assert_static::<Session>();
}

/// The flow a GUI drives: discover, open, read, edit, save, reopen.
#[test]
fn contract_flow_discover_open_edit_save_reopen() {
    let dir = temp("contract");
    let game = dir.join("game");
    let profile = game.join("profiles/abcdef");
    let save_dir = profile.join("saves/1");
    fs::create_dir_all(&save_dir).unwrap();
    fs::copy(fixture("ets2/profile.sii"), profile.join("profile.sii")).unwrap();
    fs::copy(fixture("ets2/saves/1/info.sii"), save_dir.join("info.sii")).unwrap();
    fs::copy(fixture("ets2/saves/1/game.sii"), save_dir.join("game.sii")).unwrap();

    let found: Vec<_> = profiles(Game::Ets2, std::slice::from_ref(&game))
        .into_iter()
        .filter(|profile| profile.path.starts_with(&game))
        .collect();
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "sirr-no-cloud");
    assert_eq!(found[0].saves, 1);
    assert_eq!(found[0].kind, RootKind::Custom);

    let save = save_dir.join("game.sii");
    let original = fs::read(&save).unwrap();
    let mut session = Session::open(Game::Ets2, &save).unwrap();
    assert_eq!(session.origin(), Origin::EncryptedBinary);
    assert!(!session.edited());
    assert_eq!(
        session.read().unwrap().bank.as_ref().unwrap().money_account,
        2000
    );

    session.apply(Command::SetMoney(12_345)).unwrap();
    assert!(session.edited());
    session.save(&save).unwrap();
    assert!(!session.edited());

    let mut backup = save.clone().into_os_string();
    backup.push(".bak");
    assert_eq!(fs::read(PathBuf::from(backup)).unwrap(), original);

    let reopened = Session::open(Game::Ets2, &save).unwrap();
    assert_eq!(reopened.origin(), Origin::Plain);
    assert_eq!(
        reopened
            .read()
            .unwrap()
            .bank
            .as_ref()
            .unwrap()
            .money_account,
        12_345
    );

    let _ = fs::remove_dir_all(&dir);
}
