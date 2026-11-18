//! Profile discovery, the save list, and the one open save.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use siir_save::profiles::{self, Profile};
use siir_save::{Command, Error, Game, Origin, Session, Skills};

use crate::i18n;
use crate::view_data::SaveView;

/// What the app remembers between runs.
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Extra game folders added by the user.
    pub dirs: Vec<PathBuf>,
    game: Option<String>,
    locale: Option<String>,
}

impl Settings {
    /// Reads the settings file, or defaults when it is unusable.
    pub fn load() -> Self {
        settings_path()
            .map(|path| Self::load_from(&path))
            .unwrap_or_default()
    }

    /// Writes the settings file.
    pub fn save(&self) -> std::io::Result<()> {
        let path =
            settings_path().ok_or_else(|| std::io::Error::other("no OS config directory"))?;
        self.save_to(&path)
    }

    /// Game the app was showing.
    pub fn game(&self) -> Option<Game> {
        match self.game.as_deref() {
            Some("ets2") => Some(Game::Ets2),
            Some("ats") => Some(Game::Ats),
            _ => None,
        }
    }

    /// Remembers `game`.
    pub fn set_game(&mut self, game: Game) {
        self.game = Some(
            match game {
                Game::Ets2 => "ets2",
                Game::Ats => "ats",
            }
            .to_string(),
        );
    }

    /// Locale the app was showing.
    pub fn locale(&self) -> Option<&str> {
        self.locale.as_deref()
    }

    /// Remembers `locale`.
    pub fn set_locale(&mut self, locale: &str) {
        self.locale = Some(locale.to_string());
    }

    fn load_from(path: &Path) -> Self {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    fn save_to(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        let tmp = path.with_extension(format!("{}.tmp", std::process::id()));
        let result = fs::write(&tmp, json).and_then(|()| fs::rename(&tmp, path));
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
        }
        result
    }
}

fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("siir").join("settings.json"))
}

/// One profile group in the save list.
pub struct ProfileRow {
    /// Profile directory.
    pub path: PathBuf,
    /// `profile_name` from `profile.sii`.
    pub name: String,
    /// Translation key for the folder kind.
    pub kind_key: &'static str,
}

impl ProfileRow {
    fn new(profile: Profile) -> Self {
        Self {
            path: profile.path,
            name: profile.name,
            kind_key: i18n::root_kind(profile.kind),
        }
    }
}

/// One save directory in the save list.
pub struct SaveRow {
    /// Index of the owning [`ProfileRow`].
    pub profile: usize,
    /// Save directory.
    pub path: PathBuf,
    /// Save name from `info.sii`, or the directory name when it is unreadable.
    pub name: String,
    /// File time from `info.sii`, used to pick the newest save.
    pub file_time: u64,
}

impl SaveRow {
    fn new(profile: usize, path: &Path) -> Self {
        let info = profiles::info(path).ok();
        let name = info
            .as_ref()
            .map(|info| info.name.clone())
            .filter(|name| !name.is_empty())
            .unwrap_or_else(|| {
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned()
            });
        Self {
            profile,
            path: path.to_path_buf(),
            name,
            file_time: info.map(|info| info.file_time).unwrap_or_default(),
        }
    }
}

/// The save currently open for reading and editing.
pub struct Open {
    path: PathBuf,
    session: Session,
    view: SaveView,
    profile: Option<ProfileFile>,
}

/// The owning profile's `profile.sii`, edited beside the save.
struct ProfileFile {
    path: PathBuf,
    session: Session,
}

impl Open {
    /// Rebuilds the owned view from the document after a command that can
    /// change any field.
    fn refresh(&mut self) {
        let save_name = self.view.save_name.clone();
        let profile_name = self.view.profile_name.clone();
        if let Ok(save) = self.session.read() {
            self.view = SaveView::new(&save, save_name, profile_name);
        }
    }

    /// Format the file was read from.
    pub fn origin(&self) -> Origin {
        self.session.origin()
    }

    /// Owned values the overview renders.
    pub fn view(&self) -> &SaveView {
        &self.view
    }

    /// Whether the profile file was read, so the profile can be renamed.
    pub fn has_profile(&self) -> bool {
        self.profile.is_some()
    }

    /// Whether a command changed the save or the profile since open or save.
    pub fn edited(&self) -> bool {
        self.session.edited() || self.profile_edited()
    }

    fn profile_edited(&self) -> bool {
        self.profile
            .as_ref()
            .is_some_and(|profile| profile.session.edited())
    }
}

/// The view field a command can update without re-reading the document.
enum ViewUpdate {
    /// Written to the bank balance.
    Money(i64),
    /// Written to the total experience points.
    Experience(u64),
    /// Written to the player skill levels.
    Skills(Skills),
}

/// Why a write action did not finish.
pub enum ActionError {
    /// No save is open, so there is nothing to edit or save.
    NoOpenSave,
    /// The profile file could not be read, so the profile cannot be renamed.
    NoProfile,
    /// The save document rejected the change.
    Document(Error),
}

/// A failure shown above the list.
pub struct Notice {
    /// Translation key for the message.
    pub key: &'static str,
    /// Path the failure is about.
    pub detail: String,
}

impl Notice {
    fn new(key: &'static str, detail: String) -> Self {
        Self { key, detail }
    }
}

/// What the window knows: game, profiles, saves, and the open save.
pub struct State {
    game: Game,
    custom_dirs: Vec<PathBuf>,
    profiles: Vec<ProfileRow>,
    saves: Vec<SaveRow>,
    selected_save: Option<usize>,
    open: Option<Open>,
    notice: Option<Notice>,
    saved_backup: Option<PathBuf>,
}

impl State {
    /// Discovers profiles and saves for `game` under the OS folders and `dirs`.
    pub fn with_dirs(game: Game, dirs: Vec<PathBuf>) -> Self {
        let mut state = Self {
            game,
            custom_dirs: dirs,
            profiles: Vec::new(),
            saves: Vec::new(),
            selected_save: None,
            open: None,
            notice: None,
            saved_backup: None,
        };
        state.discover();
        state
    }

    /// Game whose saves are listed.
    pub fn game(&self) -> Game {
        self.game
    }

    /// Profile groups, sorted by path.
    pub fn profiles(&self) -> &[ProfileRow] {
        &self.profiles
    }

    /// Every save under the listed profiles, grouped by profile.
    pub fn saves(&self) -> &[SaveRow] {
        &self.saves
    }

    /// Saves that belong to one profile group.
    pub fn saves_of(&self, profile: usize) -> impl Iterator<Item = &SaveRow> {
        self.saves.iter().filter(move |row| row.profile == profile)
    }

    /// Index of the last opened save.
    pub fn selected_save(&self) -> Option<usize> {
        self.selected_save
    }

    /// The save open for reading and editing.
    pub fn open(&self) -> Option<&Open> {
        self.open.as_ref()
    }

    /// Whether the open save has edits that are not on disk.
    pub fn edited(&self) -> bool {
        self.open.as_ref().is_some_and(Open::edited)
    }

    /// Backup written by the last successful save.
    pub fn saved_backup(&self) -> Option<&Path> {
        self.saved_backup.as_deref()
    }

    /// Hides the saved banner of the last write.
    pub fn clear_saved_backup(&mut self) {
        self.saved_backup = None;
    }

    /// Last failure to show above the list.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
    }

    /// Hides the notice until the next discovery.
    pub fn clear_notice(&mut self) {
        self.notice = None;
    }

    /// The save offered by the Continue action.
    pub fn newest(&self) -> Option<&SaveRow> {
        self.newest_index().and_then(|ix| self.saves.get(ix))
    }

    /// Switches the game and rediscovers its saves.
    pub fn set_game(&mut self, game: Game) {
        if self.game == game {
            return;
        }
        self.game = game;
        self.discover();
    }

    /// Adds game folders and rediscovers.
    pub fn add_dirs(&mut self, dirs: impl IntoIterator<Item = PathBuf>) {
        for dir in dirs {
            if !self.custom_dirs.contains(&dir) {
                self.custom_dirs.push(dir);
            }
        }
        self.discover();
    }

    /// Opens one save from the list.
    pub fn open_save(&mut self, ix: usize) {
        let (path, name, profile, profile_dir) = match self.saves.get(ix) {
            Some(row) => {
                let (profile, profile_dir) = self
                    .profiles
                    .get(row.profile)
                    .map(|profile| (profile.name.clone(), Some(profile.path.clone())))
                    .unwrap_or_default();
                (
                    profiles::save_path(&row.path),
                    row.name.clone(),
                    profile,
                    profile_dir,
                )
            }
            None => return,
        };
        self.selected_save = Some(ix);
        self.open = None;
        let session = match Session::open(self.game, &path) {
            Ok(session) => session,
            Err(_) => {
                let detail = path.display().to_string();
                self.notice = Some(Notice::new("error.open_save", detail));
                return;
            }
        };
        let view = match session.read() {
            Ok(save) => SaveView::new(&save, name, profile),
            Err(_) => {
                let detail = path.display().to_string();
                self.notice = Some(Notice::new("error.open_save", detail));
                return;
            }
        };
        // An unreadable profile file only disables the rename; the save stays usable.
        let profile = profile_dir.and_then(|dir| {
            let path = profiles::profile_path(&dir);
            Session::open(self.game, &path)
                .ok()
                .map(|session| ProfileFile { path, session })
        });
        self.open = Some(Open {
            path,
            session,
            view,
            profile,
        });
        self.notice = None;
        self.saved_backup = None;
    }

    /// Applies one command; economy fields update in place, since live editing
    /// sends one per keystroke.
    pub fn apply(&mut self, command: Command) -> Result<(), ActionError> {
        let Some(open) = self.open.as_mut() else {
            return Err(ActionError::NoOpenSave);
        };
        if let Command::RenameProfile(name) = &command {
            let name = name.clone();
            let profile = open.profile.as_mut().ok_or(ActionError::NoProfile)?;
            profile
                .session
                .apply(command)
                .map_err(ActionError::Document)?;
            open.view.profile_name = name;
            self.saved_backup = None;
            return Ok(());
        }
        let update = match &command {
            Command::SetMoney(value) => Some(ViewUpdate::Money(*value)),
            Command::SetExperience(value) => Some(ViewUpdate::Experience(*value)),
            Command::SetSkills(skills) => Some(ViewUpdate::Skills(*skills)),
            _ => None,
        };
        let result = open.session.apply(command);
        match (result.as_ref(), update) {
            (Ok(()), Some(ViewUpdate::Money(value))) => open.view.money = Some(value),
            (Ok(()), Some(ViewUpdate::Experience(value))) => open.view.experience = value,
            (Ok(()), Some(ViewUpdate::Skills(skills))) => open.view.skills = skills,
            (Ok(()), None) | (Err(_), _) => open.refresh(),
        }
        self.saved_backup = None;
        result.map_err(ActionError::Document)
    }

    /// Writes the edited files of the open save and returns the backup path:
    /// the save's when it was written, otherwise the profile's.
    pub fn save(&mut self) -> Result<PathBuf, ActionError> {
        let Some(open) = self.open.as_mut() else {
            return Err(ActionError::NoOpenSave);
        };
        // With nothing edited the save is still rewritten, as before the profile file.
        let write_save = open.session.edited() || !open.profile_edited();
        if write_save {
            let path = open.path.clone();
            open.session.save(&path).map_err(ActionError::Document)?;
        }
        if let Some(profile) = open.profile.as_mut().filter(|p| p.session.edited()) {
            let path = profile.path.clone();
            profile.session.save(&path).map_err(ActionError::Document)?;
            let name = open.view.profile_name.clone();
            if let Some(row) = self
                .selected_save
                .and_then(|ix| self.saves.get(ix))
                .and_then(|save| self.profiles.get_mut(save.profile))
            {
                row.name = name;
            }
        }
        let backup = match open.profile.as_ref().filter(|_| !write_save) {
            Some(profile) => backup_path(&profile.path),
            None => backup_path(&open.path),
        };
        self.saved_backup = Some(backup.clone());
        Ok(backup)
    }

    /// Opens the newest save, the one the Continue action shows.
    pub fn open_newest(&mut self) {
        if let Some(ix) = self.newest_index() {
            self.open_save(ix);
        }
    }

    /// Returns to the save list.
    pub fn close(&mut self) {
        self.open = None;
        self.saved_backup = None;
    }

    fn newest_index(&self) -> Option<usize> {
        self.saves
            .iter()
            .enumerate()
            .max_by_key(|(_, row)| row.file_time)
            .map(|(ix, _)| ix)
    }

    fn discover(&mut self) {
        self.notice = None;
        self.open = None;
        self.selected_save = None;
        self.saved_backup = None;
        self.saves.clear();

        // shortcut: discovery scans the OS and Steam folders on the UI thread;
        // move it to the background executor when a cold scan is felt.
        self.profiles = profiles::profiles(self.game, &self.custom_dirs)
            .into_iter()
            .map(ProfileRow::new)
            .collect();
        for (ix, profile) in self.profiles.iter().enumerate() {
            match profiles::saves(&profile.path) {
                Ok(paths) => {
                    self.saves
                        .extend(paths.iter().map(|path| SaveRow::new(ix, path)));
                }
                Err(_) => {
                    let detail = profile.path.display().to_string();
                    self.notice = Some(Notice::new("error.read_saves", detail));
                }
            }
        }
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(".bak");
    PathBuf::from(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_the_settings_file() {
        let root = std::env::temp_dir().join(format!("siir-gui-settings-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let path = root.join("nested").join("settings.json");

        let missing = Settings::load_from(&path);
        assert!(missing.dirs.is_empty());
        assert_eq!(missing.game(), None);
        assert_eq!(missing.locale(), None);

        let mut settings = Settings::default();
        settings.dirs.push(PathBuf::from("/games/ets2"));
        settings.set_game(Game::Ats);
        settings.set_locale("pt_br");
        settings.save_to(&path).unwrap();

        let loaded = Settings::load_from(&path);
        assert_eq!(loaded.dirs, [PathBuf::from("/games/ets2")]);
        assert_eq!(loaded.game(), Some(Game::Ats));
        assert_eq!(loaded.locale(), Some("pt_br"));
        let leftovers = fs::read_dir(path.parent().unwrap()).unwrap().count();
        assert_eq!(
            leftovers, 1,
            "the temp file must be renamed, not left behind"
        );

        fs::write(&path, b"not json").unwrap();
        assert!(Settings::load_from(&path).dirs.is_empty());
        fs::remove_dir_all(&root).unwrap();
    }
}
