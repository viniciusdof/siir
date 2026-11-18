//! Translation lookup and the keys that name typed values.

use std::borrow::Cow;

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

/// Locale used until the user picks another one.
pub const FALLBACK: &str = "en_us";

/// Every locale `ui.yml` ships.
pub fn locales() -> Vec<Cow<'static, str>> {
    rust_i18n::available_locales!()
}

/// Locale the UI is showing, clamped to a shipped one.
pub fn current_locale() -> Cow<'static, str> {
    let current = rust_i18n::locale();
    locales()
        .into_iter()
        .find(|locale| locale.as_ref() == &*current)
        .unwrap_or(Cow::Borrowed(FALLBACK))
}

/// Autonym of a shipped locale, for the language selector.
pub fn language_name(locale: &str) -> String {
    t(&format!("language.{locale}"))
}

/// Switches the UI language for the rest of the session.
pub fn set_locale(locale: &str) {
    rust_i18n::set_locale(locale);
}

/// Starts in the fallback locale until a settings store remembers the choice.
pub fn init() {
    if !locales()
        .into_iter()
        .any(|locale| locale.as_ref() == &*rust_i18n::locale())
    {
        rust_i18n::set_locale(FALLBACK);
    }
}

/// Label of the action that shows the backup in the system file manager.
pub fn reveal_label() -> &'static str {
    match std::env::consts::OS {
        "macos" => "banner.reveal_finder",
        "windows" => "banner.reveal_explorer",
        _ => "banner.reveal_folder",
    }
}

/// Currency symbol the game uses for money.
pub fn currency(game: Game) -> &'static str {
    match game {
        Game::Ets2 => "€",
        Game::Ats => "$",
    }
}

/// Thousands separator of the game's number format.
pub fn grouping(game: Game) -> char {
    match game {
        Game::Ets2 => ' ',
        Game::Ats => ',',
    }
}

/// An integer with the game's thousands separators.
pub fn amount(value: i64, game: Game) -> String {
    let separator = grouping(game);
    let digits = value.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if value < 0 {
        out.push('-');
    }
    for (ix, digit) in digits.chars().enumerate() {
        if ix > 0 && (digits.len() - ix).is_multiple_of(3) {
            out.push(separator);
        }
        out.push(digit);
    }
    out
}

/// Money as the game shows it: currency symbol and grouped digits.
pub fn money(value: i64, game: Game) -> String {
    format!("{}{}", currency(game), amount(value, game))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_carries_every_shipped_locale() {
        let source = include_str!("../locales/ui.yml");
        let locales = locales();
        let mut current: Option<&str> = None;
        let mut tags: Vec<&str> = Vec::new();
        let check = |key: &str, tags: &[&str]| {
            if key.starts_with("language.") {
                return;
            }
            for locale in &locales {
                assert!(tags.contains(&locale.as_ref()), "{key} lacks {locale}");
            }
        };
        for line in source.lines() {
            if let Some(key) = line
                .strip_suffix(':')
                .filter(|key| !key.starts_with(' ') && !key.starts_with('_'))
            {
                if let Some(previous) = current {
                    check(previous, &tags);
                }
                current = Some(key);
                tags.clear();
            } else if let Some(tag) = line
                .strip_prefix("  ")
                .and_then(|rest| rest.split_once(':'))
                .map(|(tag, _)| tag)
                .filter(|tag| !tag.contains(' '))
            {
                tags.push(tag);
            }
        }
        if let Some(last) = current {
            check(last, &tags);
        }
    }

    #[test]
    fn every_locale_has_an_autonym() {
        for locale in locales() {
            let key = format!("language.{locale}");
            assert_ne!(t(&key), key, "{key} is missing from ui.yml");
        }
    }

    #[test]
    fn labels_the_file_manager_of_each_platform() {
        for key in [
            "banner.reveal_finder",
            "banner.reveal_explorer",
            "banner.reveal_folder",
        ] {
            assert_ne!(t(key), key, "{key} is missing from ui.yml");
        }
        assert_eq!(t("banner.no_such_key"), "banner.no_such_key");
    }

    #[test]
    fn groups_amounts_per_game() {
        assert_eq!(amount(99_738_872, Game::Ets2), "99 738 872");
        assert_eq!(amount(99_738_872, Game::Ats), "99,738,872");
        assert_eq!(amount(2_000, Game::Ats), "2,000");
        assert_eq!(amount(-1_234, Game::Ats), "-1,234");
        assert_eq!(money(2_000, Game::Ets2), "€2 000");
        assert_eq!(money(2_000, Game::Ats), "$2,000");
    }
}
