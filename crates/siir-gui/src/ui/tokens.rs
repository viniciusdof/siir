use std::sync::atomic::{AtomicU8, Ordering};

use gpui_kit::base::{ColorTokens, TextStyleToken, Theme, ThemeAppearance, TypographyTokens};
use gpui_kit::{App, FontWeight, Hsla, hsla, px, rgb};
use siir_save::Game;

static ACTIVE_GAME: AtomicU8 = AtomicU8::new(0);

#[derive(Clone, Copy)]
pub struct Palette {
    pub bg: Hsla,
    pub panel: Hsla,
    pub surface: Hsla,
    pub surface_hover: Hsla,
    pub surface_active: Hsla,
    pub surface_selected: Hsla,
    pub border: Hsla,
    pub control_border: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub accent: Hsla,
    pub accent_hover: Hsla,
    pub accent_text: Hsla,
    pub accent_soft: Hsla,
    pub danger: Hsla,
    pub danger_text: Hsla,
    pub success: Hsla,
    pub success_soft: Hsla,
    pub info: Hsla,
    pub purple: Hsla,
    pub cyan: Hsla,
    pub scrim: Hsla,
}

pub fn set_game(game: Game) {
    let encoded = match game {
        Game::Ets2 => 0,
        Game::Ats => 1,
    };
    ACTIVE_GAME.store(encoded, Ordering::Relaxed);
}

pub fn active_game() -> Game {
    match ACTIVE_GAME.load(Ordering::Relaxed) {
        1 => Game::Ats,
        _ => Game::Ets2,
    }
}

pub fn palette() -> Palette {
    palette_for(active_game())
}

pub fn palette_for(game: Game) -> Palette {
    match game {
        Game::Ets2 => {
            let accent = color(0xf6b547);
            let success = color(0x5ad488);
            Palette {
                bg: color(0x0e1116),
                panel: color(0x151921),
                surface: color(0x1c222d),
                surface_hover: color(0x242c3a),
                surface_active: color(0x181e28),
                surface_selected: color(0x242d3c),
                border: color(0x283142),
                control_border: color(0x647188),
                text: color(0xeef2f8),
                muted: color(0x9ba7bc),
                accent,
                accent_hover: color(0xffc663),
                accent_text: color(0x14171c),
                accent_soft: accent.alpha(0.12),
                danger: color(0xd65a52),
                danger_text: color(0xf58b84),
                success,
                success_soft: success.alpha(0.12),
                info: color(0x60a5fa),
                purple: color(0xc084fc),
                cyan: color(0x38bdf8),
                scrim: hsla(0.0, 0.0, 0.0, 0.6),
            }
        }
        Game::Ats => {
            let accent = color(0xfa934d);
            let success = color(0x5ad488);
            Palette {
                bg: color(0x101014),
                panel: color(0x17171d),
                surface: color(0x1f2029),
                surface_hover: color(0x292a36),
                surface_active: color(0x1b1b23),
                surface_selected: color(0x2a2a36),
                border: color(0x2e2f3d),
                control_border: color(0x6b6d82),
                text: color(0xf2f1f7),
                muted: color(0xa1a1b5),
                accent,
                accent_hover: color(0xffa666),
                accent_text: color(0x141210),
                accent_soft: accent.alpha(0.10),
                danger: color(0xd65a52),
                danger_text: color(0xf58b84),
                success,
                success_soft: success.alpha(0.12),
                info: color(0x60a5fa),
                purple: color(0xc084fc),
                cyan: color(0x38bdf8),
                scrim: hsla(0.0, 0.0, 0.0, 0.6),
            }
        }
    }
}

pub fn wear_color(wear: f32) -> Hsla {
    let palette = palette();
    if wear >= 0.35 {
        palette.danger_text
    } else if wear >= 0.12 {
        palette.accent
    } else {
        palette.success
    }
}

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

pub const TEXT_TITLE: f32 = 20.0;
pub const TEXT_SECTION: f32 = 15.0;
pub const TEXT_BODY: f32 = 13.0;
pub const TEXT_LABEL: f32 = 12.0;
pub const TEXT_METRIC: f32 = 20.0;
pub const TEXT_ROW: f32 = 14.0;

pub const ROW_HEIGHT: f32 = 34.0;

pub const RADIUS: f32 = 8.0;
pub const CARD_RADIUS: f32 = 12.0;

pub const HEADER_HEIGHT: f32 = 56.0;
pub const CONTENT_WIDTH: f32 = 900.0;
pub const FORM_WIDTH: f32 = 760.0;
pub const SAVE_ROW_HEIGHT: f32 = 42.0;

pub const SPACE_1: f32 = 4.0;
pub const SPACE_2: f32 = 8.0;
pub const SPACE_3: f32 = 12.0;
pub const SPACE_4: f32 = 16.0;
pub const SPACE_6: f32 = 24.0;

pub const FONT_SANS: &str = ".SystemUIFont";

pub fn apply_theme(cx: &mut App, game: Game) {
    set_game(game);
    let palette = palette_for(game);
    let mut theme = Theme::default();
    theme.appearance = ThemeAppearance::Dark;

    let mut colors = ColorTokens::dark();
    colors.background = palette.bg;
    colors.foreground = palette.text;
    colors.surface = palette.panel;
    colors.surface_foreground = palette.text;
    colors.primary = palette.accent;
    colors.primary_foreground = palette.accent_text;
    colors.secondary = palette.surface;
    colors.secondary_foreground = palette.text;
    colors.muted = palette.surface_active;
    colors.muted_foreground = palette.muted;
    colors.accent = palette.surface_selected;
    colors.accent_foreground = palette.text;
    colors.destructive = palette.danger;
    colors.destructive_foreground = palette.bg;
    colors.border = palette.border;
    colors.input = palette.control_border;
    colors.ring = palette.accent;
    colors.selection = palette.accent.alpha(0.3);
    theme.tokens.colors = colors;

    let radius = &mut theme.tokens.radius;
    radius.sm = px(RADIUS);
    radius.md = px(RADIUS);
    radius.lg = px(CARD_RADIUS);
    radius.xl = px(CARD_RADIUS);

    let spacing = &mut theme.tokens.spacing;
    spacing.xxs = px(SPACE_1);
    spacing.xs = px(SPACE_1);
    spacing.sm = px(SPACE_2);
    spacing.md = px(SPACE_3);
    spacing.lg = px(SPACE_4);
    spacing.xl = px(SPACE_6);
    spacing.xxl = px(32.0);

    let mut typography = TypographyTokens::default();
    typography.sans = FONT_SANS.into();
    typography.xs = text_style(TEXT_LABEL, 16.0, FontWeight::SEMIBOLD);
    typography.sm = text_style(TEXT_BODY, 20.0, FontWeight::NORMAL);
    typography.md = text_style(TEXT_SECTION, 20.0, FontWeight::SEMIBOLD);
    typography.lg = text_style(TEXT_METRIC, 26.0, FontWeight::SEMIBOLD);
    typography.xl = text_style(TEXT_TITLE, 26.0, FontWeight::SEMIBOLD);
    theme.tokens.typography = typography;

    cx.set_global(theme);
}

fn text_style(size: f32, line_height: f32, weight: FontWeight) -> TextStyleToken {
    let mut style = TextStyleToken::default();
    style.size = px(size);
    style.line_height = px(line_height);
    style.weight = weight;
    style
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_kit::Rgba;

    fn luminance(color: Hsla) -> f32 {
        let rgba = Rgba::from(color);
        fn channel(value: f32) -> f32 {
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * channel(rgba.r) + 0.7152 * channel(rgba.g) + 0.0722 * channel(rgba.b)
    }

    fn contrast_luminance(a: f32, b: f32) -> f32 {
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
    }

    fn contrast(a: Hsla, b: Hsla) -> f32 {
        contrast_luminance(luminance(a), luminance(b))
    }

    fn blend_luminance(overlay: Hsla, base: Hsla) -> f32 {
        let alpha = overlay.a.clamp(0.0, 1.0);
        alpha * luminance(overlay) + (1.0 - alpha) * luminance(base)
    }

    fn assert_ratio(foreground: Hsla, background: Hsla, minimum: f32, name: &str) {
        let ratio = contrast(foreground, background);
        assert!(
            ratio >= minimum,
            "{name} is {ratio:.2}:1, below {minimum}:1"
        );
    }

    #[test]
    fn text_pairs_meet_wcag_aa() {
        for game in [Game::Ets2, Game::Ats] {
            let p = palette_for(game);
            for (name, background) in [
                ("bg", p.bg),
                ("panel", p.panel),
                ("surface", p.surface),
                ("surface_active", p.surface_active),
                ("surface_selected", p.surface_selected),
            ] {
                assert_ratio(p.text, background, 4.5, &format!("{game:?} text on {name}"));
                assert_ratio(
                    p.muted,
                    background,
                    4.5,
                    &format!("{game:?} muted on {name}"),
                );
                assert_ratio(
                    p.danger_text,
                    background,
                    4.5,
                    &format!("{game:?} danger_text on {name}"),
                );
                assert_ratio(
                    p.success,
                    background,
                    4.5,
                    &format!("{game:?} success on {name}"),
                );
                assert_ratio(
                    p.accent,
                    background,
                    4.5,
                    &format!("{game:?} accent on {name}"),
                );
            }
            let accent_pill =
                contrast_luminance(luminance(p.accent), blend_luminance(p.accent_soft, p.panel));
            assert!(
                accent_pill >= 4.5,
                "{game:?} accent on accent_soft/panel is {accent_pill:.2}:1, below 4.5:1"
            );
            let success_pill = contrast_luminance(
                luminance(p.success),
                blend_luminance(p.success_soft, p.panel),
            );
            assert!(
                success_pill >= 4.5,
                "{game:?} success on success_soft/panel is {success_pill:.2}:1, below 4.5:1"
            );
            assert_ratio(
                p.accent_text,
                p.accent,
                4.5,
                &format!("{game:?} accent_text on accent"),
            );
            assert_ratio(p.bg, p.danger, 4.5, &format!("{game:?} bg on danger"));
        }
    }

    #[test]
    fn control_edges_meet_wcag_aa() {
        for game in [Game::Ets2, Game::Ats] {
            let p = palette_for(game);
            assert_ratio(
                p.control_border,
                p.panel,
                3.0,
                &format!("{game:?} control_border on panel"),
            );
            assert_ratio(
                p.control_border,
                p.surface,
                3.0,
                &format!("{game:?} control_border on surface"),
            );
        }
    }
}
