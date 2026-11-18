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

/// Switches the UI language, clamped to a shipped locale.
pub fn set_locale(locale: &str) {
    let locale = locales()
        .into_iter()
        .find(|shipped| shipped.as_ref() == locale)
        .unwrap_or(Cow::Borrowed(FALLBACK));
    rust_i18n::set_locale(&locale);
}

/// Starts in the OS language when `ui.yml` ships it, else in English.
pub fn init() {
    let locale = sys_locale::get_locale().and_then(|tag| match_locale(&tag));
    set_locale(locale.as_deref().unwrap_or(FALLBACK));
}

/// Shipped locale for an OS tag such as `pt-BR` or `pt_BR.UTF-8`: the exact
/// region first, then any region of the same language.
pub fn match_locale(tag: &str) -> Option<Cow<'static, str>> {
    let tag = tag
        .split(['.', '@'])
        .next()
        .unwrap_or_default()
        .replace('-', "_")
        .to_ascii_lowercase();
    let language = tag.split('_').next().unwrap_or_default();
    let shipped = locales();
    shipped
        .iter()
        .find(|locale| locale.as_ref() == tag)
        .or_else(|| {
            shipped
                .iter()
                .find(|locale| locale.split('_').next() == Some(language))
        })
        .cloned()
}

/// Accessible name of an Overview value that opens the screen editing it.
pub fn edit_link_label(label: &str, value: &str, screen: &str) -> String {
    rust_i18n::t!(
        "link.edit_in",
        label = label,
        value = value,
        screen = t(screen)
    )
    .to_string()
}

/// Label of the action that shows the backup in the system file manager.
pub fn reveal_label() -> &'static str {
    match std::env::consts::OS {
        "macos" => "banner.reveal_finder",
        "windows" => "banner.reveal_explorer",
        _ => "banner.reveal_folder",
    }
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
    fn matches_os_locales_to_shipped_ones() {
        let matched = |tag: &str| match_locale(tag).map(Cow::into_owned);
        assert_eq!(matched("pt-BR").as_deref(), Some("pt_br"));
        assert_eq!(matched("pt_BR.UTF-8").as_deref(), Some("pt_br"));
        assert_eq!(matched("pt-PT").as_deref(), Some("pt_br"));
        assert_eq!(matched("en-GB").as_deref(), Some("en_us"));
        assert_eq!(matched("en").as_deref(), Some("en_us"));
        assert_eq!(matched("de-DE"), None);
        assert_eq!(matched("C"), None);
        assert_eq!(matched(""), None);
    }

    #[test]
    fn names_an_edit_link_after_its_screen() {
        assert_eq!(
            edit_link_label("Money", "€2 000", "nav.economy"),
            "Money: €2 000, edit in Economy"
        );
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
}
