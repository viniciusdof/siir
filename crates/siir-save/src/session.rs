use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use siir::File;
use siir::container::Origin;

use crate::command::{Command, apply};
use crate::error::Error;
use crate::model::{Game, Save};
use crate::profiles::replace_file;

/// A save file open for reading and editing.
pub struct Session {
    game: Game,
    file: File,
    edited: bool,
}

impl Session {
    /// Opens a save from disk.
    pub fn open(game: Game, path: &Path) -> Result<Self, Error> {
        Self::from_bytes(game, fs::read(path)?)
    }

    /// Opens a save from bytes already in memory.
    pub fn from_bytes(game: Game, bytes: Vec<u8>) -> Result<Self, Error> {
        Ok(Self {
            game,
            file: siir::open(&bytes)?,
            edited: false,
        })
    }

    /// Reads the typed view of the current document.
    pub fn read(&self) -> Result<Save<'_>, Error> {
        crate::read(self.file.document(), self.game)
    }

    /// Applies one command, marking the session edited.
    pub fn apply(&mut self, command: Command) -> Result<(), Error> {
        let game = self.game;
        self.edited = true;
        self.file
            .with_document_mut(|document| apply(document, game, command))
    }

    /// Renders the current document as plain `SiiNunit` text.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.file.write()
    }

    /// Writes the document, keeping the old file as `<file>.bak`.
    pub fn save(&mut self, path: &Path) -> Result<(), Error> {
        write_atomic(path, &self.to_bytes())?;
        self.edited = false;
        Ok(())
    }

    /// Whether a command changed the document since open or save.
    pub fn edited(&self) -> bool {
        self.edited
    }

    /// Format the document was opened from.
    pub fn origin(&self) -> Origin {
        self.file.origin()
    }
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    static SERIAL: AtomicU64 = AtomicU64::new(0);
    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    let dir = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let serial = SERIAL.fetch_add(1, Ordering::Relaxed);
    let tmp = dir.join(format!(".{name}.{}.{serial}.tmp", std::process::id()));
    let result = (|| -> Result<(), Error> {
        let mut file = fs::File::create(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        if path.exists() {
            fs::copy(path, backup_path(path))?;
        }
        replace_file(&tmp, path)?;
        #[cfg(unix)]
        fs::File::open(dir)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".bak");
    PathBuf::from(name)
}
