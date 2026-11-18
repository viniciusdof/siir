//! Profile discovery, the flat save list, and the one open save.

use std::path::{Path, PathBuf};

use siir_save::profiles::{self, Profile};
use siir_save::{Game, Origin, Session};

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

/// The save currently open for reading.
pub struct Open {
    session: Session,
    view: SaveView,
}

impl Open {
    /// Format the file was read from.
    pub fn origin(&self) -> Origin {
        self.session.origin()
    }

    /// Owned values the overview renders.
    pub fn view(&self) -> &SaveView {
        &self.view
    }
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

    /// The save open for reading.
    pub fn open(&self) -> Option<&Open> {
        self.open.as_ref()
    }

    /// Last failure to show above the list.
    pub fn notice(&self) -> Option<&Notice> {
        self.notice.as_ref()
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
        self.open = Some(Open { session, view });
        self.notice = None;
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
