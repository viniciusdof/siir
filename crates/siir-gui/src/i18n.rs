//! Translation lookup and the keys that name typed values.

use siir_save::Game;
use siir_save::profiles::RootKind;

/// Translation for `key` in the current locale.
pub fn t(key: &str) -> String {
    rust_i18n::t!(key).to_string()
}

/// Short game name used by the switch.
pub fn game_short(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "game.ets2_short",
        Game::Ats => "game.ats_short",
    }
}

/// Where a game folder came from.
pub fn root_kind(kind: RootKind) -> &'static str {
    match kind {
        RootKind::Local => "kind.local",
        RootKind::Steam => "kind.steam",
        RootKind::Proton => "kind.proton",
        RootKind::Custom => "kind.custom",
    }
}
