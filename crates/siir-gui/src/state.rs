//! Profile discovery, the save list, and the one open save.

use std::path::{Path, PathBuf};

use siir_save::profiles::{self, Profile};
use siir_save::{Command, Error, Game, Origin, Session, Skills};

use crate::i18n;
use crate::view_data::SaveView;

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

    /// Whether a command changed the document since open or save.
    pub fn edited(&self) -> bool {
        self.session.edited()
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
        let (path, name, profile) = match self.saves.get(ix) {
            Some(row) => {
                let profile = self
                    .profiles
                    .get(row.profile)
                    .map(|profile| profile.name.clone())
                    .unwrap_or_default();
                (profiles::save_path(&row.path), row.name.clone(), profile)
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
        self.open = Some(Open {
            path,
            session,
            view,
        });
        self.notice = None;
        self.saved_backup = None;
    }

    /// Applies one command and keeps the overview in step with the document.
    ///
    /// The economy commands update their one view field in place; rebuilding
    /// the whole view re-reads every unit, and live editing sends one command
    /// per keystroke.
    pub fn apply(&mut self, command: Command) -> Result<(), ActionError> {
        let Some(open) = self.open.as_mut() else {
            return Err(ActionError::NoOpenSave);
        };
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

    /// Writes the open save and returns the backup path.
    pub fn save(&mut self) -> Result<PathBuf, ActionError> {
        let Some(open) = self.open.as_mut() else {
            return Err(ActionError::NoOpenSave);
        };
        let path = open.path.clone();
        open.session.save(&path).map_err(ActionError::Document)?;
        let backup = backup_path(&path);
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
