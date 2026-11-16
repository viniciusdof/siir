//! Profile discovery, info, backup, restore, and clone.

use std::fs;
use std::path::{Path, PathBuf};

use siir_save::Game;
use siir_save::profiles::{
    backup, clone_profile, copy_settings, find_profiles, info, restore, saves,
};

#[cfg(unix)]
use std::os::unix::fs::symlink;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn temp(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("siir-save-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    path
}

fn files(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, Vec<u8>)>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path
                .file_name()
                .is_some_and(|name| name.to_string_lossy() == "album")
            {
                continue;
            }
            if path.is_dir() {
                walk(root, &path, out);
            } else if path.is_file() {
                out.push((
                    path.strip_prefix(root).unwrap().to_path_buf(),
                    fs::read(&path).unwrap(),
                ));
            }
        }
    }
    let mut out = Vec::new();
    walk(root, root, &mut out);
    out.sort_by(|left, right| left.0.cmp(&right.0));
    out
}

#[test]
fn finds_profiles_saves_and_info() {
    let profiles = find_profiles(&fixtures()).unwrap();
    assert_eq!(profiles, vec![fixtures().join("ets2")]);

    let saves = saves(&fixtures().join("ets2")).unwrap();
    assert_eq!(saves.len(), 3);
    assert!(saves.contains(&fixtures().join("ets2/saves/1")));

    let info = info(&fixtures().join("ets2/saves/1")).unwrap();
    assert_eq!(info.name, "teste");
    assert_eq!(info.version, 105);
    assert_eq!(info.info_version, 1);
    assert_eq!(info.dependencies.len(), 106);
}

#[test]
fn backup_and_restore_round_trip() {
    let temp = temp("backup");
    fs::create_dir_all(&temp).unwrap();
    let archive = temp.join("profile.zip");
    let restored = temp.join("restored");
    let source = fixtures().join("ets2");

    backup(&source, &archive).unwrap();
    restore(&archive, &restored).unwrap();
    assert_eq!(files(&source), files(&restored));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn clones_and_renames_a_profile() {
    let temp = temp("clone");
    fs::create_dir_all(&temp).unwrap();
    let archive = temp.join("profile.zip");
    let source = temp.join("source");
    backup(&fixtures().join("ets2"), &archive).unwrap();
    restore(&archive, &source).unwrap();

    let cloned = clone_profile(&source, "new", Game::Ets2).unwrap();
    assert_eq!(cloned.file_name().unwrap(), "6E6577");

    let bytes = fs::read(cloned.join("profile.sii")).unwrap();
    let file = siir::open(&bytes).unwrap();
    let document = file.document().unwrap();
    let unit = document.units_of_class("user_profile").next().unwrap();
    assert_eq!(unit.get("profile_name"), Some("\"new\""));

    let error = clone_profile(&source, "new", Game::Ets2).unwrap_err();
    assert!(error.to_string().contains("already exists"));

    let _ = fs::remove_dir_all(&temp);
}

#[cfg(unix)]
#[test]
fn backup_rejects_a_link_outside_the_profile() {
    let temp = temp("link");
    let profile = temp.join("profile");
    let outside = temp.join("outside");
    fs::create_dir_all(&profile).unwrap();
    fs::create_dir_all(&outside).unwrap();
    fs::write(profile.join("profile.sii"), b"profile").unwrap();
    symlink(&outside, profile.join("saves")).unwrap();

    let error = backup(&profile, &temp.join("profile.zip")).unwrap_err();
    assert!(error.to_string().contains("points outside the profile"));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn restore_refuses_a_non_empty_destination() {
    let temp = temp("restore");
    fs::create_dir_all(&temp).unwrap();
    let archive = temp.join("profile.zip");
    let destination = temp.join("destination");
    backup(&fixtures().join("ets2"), &archive).unwrap();
    fs::create_dir_all(&destination).unwrap();
    fs::write(destination.join("stale.sii"), b"stale").unwrap();

    let error = restore(&archive, &destination).unwrap_err();
    assert!(error.to_string().contains("already exists"));

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn info_unquotes_a_name_with_spaces() {
    let temp = temp("info");
    let save = temp.join("save");
    fs::create_dir_all(&save).unwrap();
    fs::write(
        save.join("info.sii"),
        concat!(
            "SiiNunit\n{\n",
            "save_container : sc {\n",
            "    name: \"My Save\"\n",
            "    time: 1\n",
            "    file_time: 2\n",
            "    version: 105\n",
            "    info_version: 1\n",
            "    dependencies: 0\n",
            "}\n}\n",
        ),
    )
    .unwrap();
    assert_eq!(info(&save).unwrap().name, "My Save");

    let _ = fs::remove_dir_all(&temp);
}

#[test]
fn copies_settings_files_that_exist() {
    let temp = temp("settings");
    let from = temp.join("from");
    let to = temp.join("to");
    fs::create_dir_all(&from).unwrap();
    fs::create_dir_all(&to).unwrap();
    fs::write(from.join("config.cfg"), b"config").unwrap();
    fs::write(from.join("controls.sii"), b"controls").unwrap();

    copy_settings(&from, &to).unwrap();
    assert_eq!(fs::read(to.join("config.cfg")).unwrap(), b"config");
    assert_eq!(fs::read(to.join("controls.sii")).unwrap(), b"controls");
    assert!(!to.join("config_local.cfg").exists());

    let _ = fs::remove_dir_all(&temp);
}
