//! Profile directories: discovery, save info, backup, restore, and clone.

use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::Game;
use crate::command::{Command, apply};
use crate::error::Error;
use crate::names;
use crate::value::{array, count, string, unquote};

const MAX_ENTRY: u64 = 512 * 1024 * 1024;
const MAX_TOTAL: u64 = 2 * 1024 * 1024 * 1024;

pub(crate) fn replace_file(tmp: &Path, dest: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    if dest.exists() {
        std::fs::remove_file(dest)?;
    }
    std::fs::rename(tmp, dest)
}

/// Metadata from a save's `info.sii`.
#[derive(Debug)]
pub struct SaveInfo {
    /// Save name shown by the game.
    pub name: String,
    /// In-game time the save was written.
    pub time: u64,
    /// File time the save was written.
    pub file_time: u64,
    /// Save structure version.
    pub version: u64,
    /// Info structure version.
    pub info_version: u64,
    /// DLC and mod dependencies.
    pub dependencies: Vec<String>,
}

/// Where a game folder came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootKind {
    /// The OS documents folder.
    Local,
    /// A Steam Cloud `userdata` folder.
    Steam,
    /// A Proton prefix documents folder.
    Proton,
    /// A caller-supplied folder.
    Custom,
}

/// One folder that can hold game profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameDir {
    /// Game folder itself.
    pub path: PathBuf,
    /// Where the folder came from.
    pub kind: RootKind,
}

/// A profile found under a game folder.
#[derive(Debug)]
pub struct Profile {
    /// `profiles*` folder holding the profile.
    pub path: PathBuf,
    /// `profile_name` from `profile.sii`.
    pub name: String,
    /// Newest `file_time` across the profile's saves.
    pub last_time: u64,
    /// Number of saves under the profile.
    pub saves: usize,
    /// Where the profile came from.
    pub kind: RootKind,
}

/// Profiles under a game's `profiles` directory.
pub fn find_profiles(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut profiles = Vec::new();
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.is_dir() && path.join(names::PROFILE_FILE).is_file() {
            profiles.push(path);
        }
    }
    profiles.sort();
    Ok(profiles)
}

/// Existing game folders with profiles, from the OS, Steam Cloud, and Proton.
pub fn game_dirs(game: Game) -> Vec<GameDir> {
    let mut dirs = Vec::new();
    if let Some(path) = local_game_dir(game) {
        dirs.push(GameDir {
            path,
            kind: RootKind::Local,
        });
    }
    for steam in steamlocate::locate_all().unwrap_or_default() {
        dirs.extend(userdata_dirs(steam.path(), game));
        if let Ok(libraries) = steam.library_paths() {
            for library in libraries {
                dirs.extend(proton_dirs(&library, game));
            }
        }
    }
    dirs.retain(|dir| dir.path.is_dir());
    dirs.sort_by(|left, right| left.path.cmp(&right.path));
    dirs.dedup_by(|left, right| left.path == right.path);
    dirs
}

/// Profiles under [`game_dirs`] and `extra`, sorted by path.
pub fn profiles(game: Game, extra: &[PathBuf]) -> Vec<Profile> {
    let mut roots = game_dirs(game);
    roots.extend(extra.iter().cloned().map(|path| GameDir {
        path,
        kind: RootKind::Custom,
    }));
    scan(&roots)
}

fn scan(roots: &[GameDir]) -> Vec<Profile> {
    let mut found = Vec::new();
    for root in roots {
        let Ok(entries) = fs::read_dir(&root.path) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir()
                || !entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with(names::PROFILES_PREFIX)
            {
                continue;
            }
            let Ok(profiles) = find_profiles(&path) else {
                continue;
            };
            for profile in profiles {
                if let Some(profile) = describe(profile, root.kind) {
                    found.push(profile);
                }
            }
        }
    }
    found.sort_by(|left, right| left.path.cmp(&right.path));
    found.dedup_by(|left, right| left.path == right.path);
    found
}

fn describe(path: PathBuf, kind: RootKind) -> Option<Profile> {
    let name = profile_name(&path).ok()?;
    let saves = saves(&path).unwrap_or_default();
    let last_time = saves
        .iter()
        .filter_map(|save| info(save).ok())
        .map(|info| info.file_time)
        .max()
        .unwrap_or_default();
    Some(Profile {
        path,
        name,
        last_time,
        saves: saves.len(),
        kind,
    })
}

fn profile_name(profile: &Path) -> Result<String, Error> {
    let bytes = fs::read(profile.join(names::PROFILE_FILE))?;
    let file = siir::open(&bytes)?;
    let unit = file
        .document()
        .units_of_class(names::USER_PROFILE)
        .next()
        .ok_or_else(|| Error::MissingUnit {
            id: names::USER_PROFILE.to_owned(),
        })?;
    Ok(string(unit, names::PROFILE_NAME)?.to_owned())
}

fn local_game_dir(game: Game) -> Option<PathBuf> {
    #[cfg(windows)]
    let base = dirs::document_dir();
    #[cfg(not(windows))]
    let base = dirs::data_dir();
    base.map(|base| base.join(folder(game)))
}

fn userdata_dirs(steam: &Path, game: Game) -> Vec<GameDir> {
    let Ok(users) = fs::read_dir(steam.join("userdata")) else {
        return Vec::new();
    };
    users
        .flatten()
        .map(|user| GameDir {
            path: user.path().join(app_id(game)).join("remote"),
            kind: RootKind::Steam,
        })
        .collect()
}

fn proton_dirs(library: &Path, game: Game) -> Vec<GameDir> {
    let users = library
        .join("steamapps/compatdata")
        .join(app_id(game))
        .join("pfx/drive_c/users");
    let Ok(entries) = fs::read_dir(users) else {
        return Vec::new();
    };
    entries
        .flatten()
        .map(|user| GameDir {
            path: user.path().join("Documents").join(folder(game)),
            kind: RootKind::Proton,
        })
        .filter(|dir| dir.path.is_dir())
        .collect()
}

fn folder(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "Euro Truck Simulator 2",
        Game::Ats => "American Truck Simulator",
    }
}

fn app_id(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "227300",
        Game::Ats => "270880",
    }
}

/// Saves inside one profile.
pub fn saves(profile: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut saves = Vec::new();
    let entries = match fs::read_dir(profile.join(names::SAVES_DIR)) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(saves),
        Err(error) => return Err(error.into()),
    };
    for entry in entries {
        let path = entry?.path();
        if path.is_dir()
            && (path.join(names::INFO_FILE).is_file() || path.join(names::GAME_FILE).is_file())
        {
            saves.push(path);
        }
    }
    saves.sort();
    Ok(saves)
}

/// The `game.sii` file inside a save directory.
pub fn save_path(save: &Path) -> PathBuf {
    save.join(names::GAME_FILE)
}

/// The `profile.sii` file inside a profile directory.
pub fn profile_path(profile: &Path) -> PathBuf {
    profile.join(names::PROFILE_FILE)
}

/// Reads one save's `info.sii`.
pub fn info(save: &Path) -> Result<SaveInfo, Error> {
    let bytes = fs::read(save.join(names::INFO_FILE))?;
    let file = siir::open(&bytes)?;
    let document = file.document();
    let unit = document
        .units_of_class("save_container")
        .next()
        .ok_or_else(|| Error::MissingUnit {
            id: "save_container".to_owned(),
        })?;
    Ok(SaveInfo {
        name: string(unit, "name")?.to_owned(),
        time: count(unit, "time")?,
        file_time: count(unit, "file_time")?,
        version: count(unit, "version")?,
        info_version: count(unit, "info_version")?,
        dependencies: array(unit, "dependencies")?
            .into_iter()
            .map(|value| unquote(value).to_owned())
            .collect(),
    })
}

/// Writes a profile directory to a zip archive.
pub fn backup(profile: &Path, archive: &Path) -> Result<(), Error> {
    let name = archive
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| "profile.zip".to_owned());
    let tmp = archive.with_file_name(format!("{name}.{}.tmp", std::process::id()));
    let _ = fs::remove_file(&tmp);
    let entries = walk(profile)?;
    let exclude = fs::canonicalize(archive).ok();
    let result = (|| -> Result<(), Error> {
        let file = fs::File::create(&tmp)?;
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for entry in entries {
            if exclude.as_deref() == fs::canonicalize(&entry.source).ok().as_deref() {
                continue;
            }
            let name = entry.relative.to_string_lossy().replace('\\', "/");
            if entry.dir {
                zip.add_directory(format!("{name}/"), options)?;
            } else {
                zip.start_file(name, options)?;
                let mut source = fs::File::open(&entry.source)?;
                std::io::copy(&mut source, &mut zip)?;
            }
        }
        zip.finish()?;
        replace_file(&tmp, archive)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

/// Extracts a profile archive into an empty or new directory.
pub fn restore(archive: &Path, profile: &Path) -> Result<(), Error> {
    if let Ok(metadata) = fs::symlink_metadata(profile) {
        if metadata.file_type().is_symlink() {
            return Err(Error::ProfileExists(profile.display().to_string()));
        }
        if !metadata.is_dir() || fs::read_dir(profile)?.next().is_some() {
            return Err(Error::ProfileExists(profile.display().to_string()));
        }
    }
    let file = fs::File::open(archive)?;
    let mut zip = ZipArchive::new(file)?;
    let result = (|| -> Result<(), Error> {
        fs::create_dir_all(profile)?;
        let mut total = 0u64;
        for index in 0..zip.len() {
            let mut entry = zip.by_index(index)?;
            let Some(relative) = entry.enclosed_name() else {
                return Err(Error::InvalidArchive);
            };
            let destination = profile.join(relative);
            if entry.is_dir() {
                fs::create_dir_all(&destination)?;
                continue;
            }
            if entry.size() > MAX_ENTRY {
                return Err(Error::ArchiveTooLarge);
            }
            total = total.saturating_add(entry.size());
            if total > MAX_TOTAL {
                return Err(Error::ArchiveTooLarge);
            }
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut out = fs::File::create(&destination)?;
            let copied = std::io::copy(&mut (&mut entry).take(MAX_ENTRY + 1), &mut out)?;
            if copied > MAX_ENTRY {
                return Err(Error::ArchiveTooLarge);
            }
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(profile);
    }
    result
}

/// Copies a profile to a sibling directory named after `name` and renames it.
pub fn clone_profile(profile: &Path, name: &str, game: Game) -> Result<PathBuf, Error> {
    let entries = walk(profile)?;
    let parent = profile.parent().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "profile has no parent",
        ))
    })?;
    let destination = parent.join(profile_dir_name(name));
    if destination.exists() {
        return Err(Error::ProfileExists(name.to_owned()));
    }
    let result = (|| -> Result<(), Error> {
        fs::create_dir_all(&destination)?;
        for entry in entries {
            let target = destination.join(&entry.relative);
            if entry.dir {
                fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&entry.source, &target)?;
            }
        }
        let bytes = fs::read(destination.join(names::PROFILE_FILE))?;
        let mut file = siir::open(&bytes)?;
        file.with_document_mut(|document| {
            apply(document, game, Command::RenameProfile(name.to_owned()))
        })?;
        fs::write(destination.join(names::PROFILE_FILE), file.write())?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_dir_all(&destination);
    }
    result?;
    Ok(destination)
}

/// Copies the settings files the reference editors carry between profiles.
pub fn copy_settings(from: &Path, to: &Path) -> Result<(), Error> {
    for name in names::SETTINGS_FILES {
        let source = from.join(name);
        if source.is_file() {
            fs::copy(source, to.join(name))?;
        }
    }
    Ok(())
}

struct Entry {
    relative: PathBuf,
    source: PathBuf,
    dir: bool,
}

fn walk(root: &Path) -> Result<Vec<Entry>, Error> {
    let root = fs::canonicalize(root)?;
    let mut entries = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![(root.clone(), PathBuf::new())];
    while let Some((dir, relative)) = stack.pop() {
        let canonical = fs::canonicalize(&dir)?;
        if !visited.insert(canonical) {
            continue;
        }
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let relative = relative.join(entry.file_name());
            if relative
                .components()
                .next()
                .is_some_and(|part| part.as_os_str().to_string_lossy().as_ref() == names::ALBUM_DIR)
            {
                continue;
            }
            let file_type = entry.file_type()?;
            let (dir, file) = if file_type.is_symlink() {
                let target = fs::canonicalize(&path)?;
                if !target.starts_with(&root) {
                    return Err(Error::ExternalLink {
                        path: relative.display().to_string(),
                    });
                }
                (target.is_dir(), target.is_file())
            } else {
                (file_type.is_dir(), file_type.is_file())
            };
            if dir {
                entries.push(Entry {
                    relative: relative.clone(),
                    source: path.clone(),
                    dir: true,
                });
                stack.push((path, relative));
            } else if file {
                entries.push(Entry {
                    relative,
                    source: path,
                    dir: false,
                });
            }
        }
    }
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    Ok(entries)
}

fn profile_dir_name(name: &str) -> String {
    name.bytes().map(|byte| format!("{byte:02X}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_the_game_file() {
        assert_eq!(
            save_path(Path::new("/saves/1")),
            Path::new("/saves/1/game.sii")
        );
    }

    #[test]
    fn joins_the_profile_file() {
        assert_eq!(
            profile_path(Path::new("/profiles/AAA")),
            Path::new("/profiles/AAA/profile.sii")
        );
    }

    #[test]
    fn proton_dirs_find_the_prefix_documents_folder() {
        let root = std::env::temp_dir().join(format!("siir-proton-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let documents = root.join(
            "steamapps/compatdata/227300/pfx/drive_c/users/steamuser/Documents/Euro Truck Simulator 2",
        );
        fs::create_dir_all(&documents).unwrap();
        assert_eq!(
            proton_dirs(&root, Game::Ets2),
            vec![GameDir {
                path: documents,
                kind: RootKind::Proton,
            }]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn userdata_dirs_find_the_steam_cloud_folder() {
        let root = std::env::temp_dir().join(format!("siir-userdata-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let remote = root.join("userdata/123/227300/remote");
        fs::create_dir_all(&remote).unwrap();
        assert_eq!(
            userdata_dirs(&root, Game::Ets2),
            vec![GameDir {
                path: remote,
                kind: RootKind::Steam,
            }]
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn scan_keeps_the_root_kind() {
        let root = std::env::temp_dir().join(format!("siir-scan-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let profile = include_str!("../testdata/profile.sii");
        for name in ["local", "custom"] {
            let dir = root.join(name).join("profiles/abc");
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join("profile.sii"), profile).unwrap();
        }

        let roots = [
            GameDir {
                path: root.join("local"),
                kind: RootKind::Local,
            },
            GameDir {
                path: root.join("custom"),
                kind: RootKind::Custom,
            },
        ];
        let found = scan(&roots);
        assert_eq!(found.len(), 2);
        for profile in &found {
            assert_eq!(profile.name, "old");
            assert_eq!(profile.saves, 0);
            assert_eq!(profile.last_time, 0);
        }
        let local = found
            .iter()
            .find(|profile| profile.kind == RootKind::Local)
            .unwrap();
        assert_eq!(local.path, root.join("local/profiles/abc"));
        let custom = found
            .iter()
            .find(|profile| profile.kind == RootKind::Custom)
            .unwrap();
        assert_eq!(custom.path, root.join("custom/profiles/abc"));
        fs::remove_dir_all(&root).unwrap();
    }
}
