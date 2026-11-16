//! Profile directories: discovery, save info, backup, restore, and clone.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::Game;
use crate::command::{Command, apply};
use crate::error::Error;
use crate::names;
use crate::value::{array, count, string, unquote};

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

/// Saves inside one profile.
pub fn saves(profile: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut saves = Vec::new();
    for entry in fs::read_dir(profile.join(names::SAVES_DIR))? {
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

/// Reads one save's `info.sii`.
pub fn info(save: &Path) -> Result<SaveInfo, Error> {
    let bytes = fs::read(save.join(names::INFO_FILE))?;
    let file = siir::open(&bytes)?;
    let document = file.document()?;
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
    let entries = walk(profile)?;
    // shortcut: archives are stored without compression; enable zip deflate
    // and allow the zlib license when backup size matters.
    let file = fs::File::create(archive)?;
    let mut zip = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    for entry in entries {
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
    Ok(())
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
    fs::create_dir_all(profile)?;
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
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut out = fs::File::create(&destination)?;
        std::io::copy(&mut entry, &mut out)?;
    }
    Ok(())
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
    let file = siir::open(&bytes)?;
    let mut document = file.document()?;
    apply(&mut document, game, Command::RenameProfile(name.to_owned()))?;
    fs::write(
        destination.join(names::PROFILE_FILE),
        file.write(&document)?,
    )?;
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
                // shortcut: album photos are skipped; include them when a full
                // profile copy is required.
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
