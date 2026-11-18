//! The application palette and the Base theme projection.

use gpui_kit::base::{ColorTokens, TextStyleToken, Theme, ThemeAppearance, TypographyTokens};
use gpui_kit::{App, FontWeight, Hsla, hsla, px, rgb};

/// The fixed dark palette.
#[derive(Clone, Copy)]
pub struct Palette {
    /// Window background.
    pub bg: Hsla,
    /// Sidebar and section panels.
    pub panel: Hsla,
    /// Rows, buttons, and raised areas.
    pub surface: Hsla,
    /// Hovered rows and buttons.
    pub surface_hover: Hsla,
    /// Pressed rows and buttons.
    pub surface_active: Hsla,
    /// Selected row.
    pub surface_selected: Hsla,
    /// Hairline dividers.
    pub border: Hsla,
    /// Edge of a control.
    pub control_border: Hsla,
    /// Primary text.
    pub text: Hsla,
    /// Labels and secondary text.
    pub muted: Hsla,
    /// Selection, primary action, and focus ring.
    pub accent: Hsla,
    /// Primary action hover.
    pub accent_hover: Hsla,
    /// Text on amber.
    pub accent_text: Hsla,
    /// Destructive fill and error edge.
    pub danger: Hsla,
    /// Error text.
    pub danger_text: Hsla,
    /// Saved state and confirmations.
    pub success: Hsla,
    /// Modal backdrop.
    pub scrim: Hsla,
}

/// The palette every view and wrapper reads.
pub fn palette() -> Palette {
    Palette {
        bg: color(0x14181e),
        panel: color(0x1d232b),
        surface: color(0x252d37),
        surface_hover: color(0x2e3844),
        surface_active: color(0x212932),
        surface_selected: color(0x303a47),
        border: color(0x3a4552),
        control_border: color(0x6e7782),
        text: color(0xe6e9ed),
        muted: color(0x9aa4b2),
        accent: color(0xe3a24a),
        accent_hover: color(0xf0b45f),
        accent_text: color(0x1a1d21),
        danger: color(0xc96a5b),
        danger_text: color(0xe2907f),
        success: color(0x7fb87f),
        scrim: hsla(0.0, 0.0, 0.0, 0.55),
    }
}

fn color(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Type roles in px.
pub const TEXT_TITLE: f32 = 20.0;
/// Section title.
pub const TEXT_SECTION: f32 = 15.0;
/// Body, rows, and inputs.
pub const TEXT_BODY: f32 = 13.0;
/// Labels and captions.
pub const TEXT_LABEL: f32 = 12.0;
/// Headline metric value.
pub const TEXT_METRIC: f32 = 18.0;
/// Save name in the home list.
pub const TEXT_ROW: f32 = 14.0;

/// Height of a list row and of a button.
pub const ROW_HEIGHT: f32 = 30.0;

/// One radius for every surface and control.
pub const RADIUS: f32 = 4.0;

/// Header strip height.
pub const HEADER_HEIGHT: f32 = 56.0;
/// Width of the content column on the home and overview.
pub const CONTENT_WIDTH: f32 = 880.0;
/// Width of the economy form.
pub const FORM_WIDTH: f32 = 700.0;
/// Height of a save row on the home screen.
pub const SAVE_ROW_HEIGHT: f32 = 40.0;

/// Spacing scale.
pub const SPACE_1: f32 = 4.0;
/// Spacing scale.
pub const SPACE_2: f32 = 8.0;
/// Spacing scale.
pub const SPACE_3: f32 = 12.0;
/// Spacing scale.
pub const SPACE_4: f32 = 16.0;
/// Spacing scale.
pub const SPACE_6: f32 = 24.0;

/// The UI typeface.
pub const FONT_SANS: &str = ".SystemUIFont";

/// Projects the palette into the Base theme so Base controls inherit it.
pub fn install(cx: &mut App) {
    let palette = palette();
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
    radius.lg = px(RADIUS);
    radius.xl = px(RADIUS);

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
    typography.lg = text_style(TEXT_METRIC, 24.0, FontWeight::SEMIBOLD);
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

    fn contrast(a: Hsla, b: Hsla) -> f32 {
        let (a, b) = (luminance(a), luminance(b));
        (a.max(b) + 0.05) / (a.min(b) + 0.05)
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
        let p = palette();
        for (name, background) in [
            ("bg", p.bg),
            ("panel", p.panel),
            ("surface", p.surface),
            ("surface_active", p.surface_active),
            ("surface_selected", p.surface_selected),
        ] {
            assert_ratio(p.text, background, 4.5, &format!("text on {name}"));
            assert_ratio(p.muted, background, 4.5, &format!("muted on {name}"));
            assert_ratio(
                p.danger_text,
                background,
                4.5,
                &format!("danger_text on {name}"),
            );
            assert_ratio(p.success, background, 4.5, &format!("success on {name}"));
            assert_ratio(p.accent, background, 3.0, &format!("accent on {name}"));
        }
        assert_ratio(p.accent_text, p.accent, 4.5, "accent_text on accent");
        assert_ratio(p.bg, p.danger, 4.5, "bg on danger");
    }

    #[test]
    fn control_edges_meet_wcag_aa() {
        let p = palette();
        assert_ratio(p.control_border, p.panel, 3.0, "control_border on panel");
        assert_ratio(
            p.control_border,
            p.surface,
            3.0,
            "control_border on surface",
        );
    }
}
