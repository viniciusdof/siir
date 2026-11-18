//! The editor sidebar.

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{Context, FontWeight, IntoElement, Toggled, div, px, transparent_black};

use siir_gui::i18n::t;

use crate::ui::{self, tokens};
use crate::view::{AppView, Section};

impl AppView {
    pub(crate) fn render_sidebar(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        div()
            .w(px(200.0))
            .h_full()
            .flex_shrink_0()
            .bg(palette.panel)
            .border_r_1()
            .border_color(palette.border)
            .p(px(tokens::SPACE_2))
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_1))
            .child(self.nav_item(
                Section::Overview,
                "nav-overview",
                IconName::Gauge,
                t("nav.overview"),
                cx,
            ))
            .child(self.nav_item(
                Section::Economy,
                "nav-economy",
                IconName::Wallet,
                t("nav.economy"),
                cx,
            ))
            .child(self.nav_item(
                Section::Profile,
                "nav-profile",
                IconName::CircleUser,
                t("nav.profile"),
                cx,
            ))
    }

    fn nav_item(
        &self,
        section: Section,
        id: &'static str,
        name: IconName,
        label: String,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let active = self.section == section;
        let color = if active { palette.text } else { palette.muted };
        BaseButton::new(id)
            .w_full()
            .h(px(36.0))
            .px(px(tokens::SPACE_2))
            .gap(px(tokens::SPACE_2))
            .flex()
            .items_center()
            .justify_start()
            .rounded(px(tokens::RADIUS))
            .border_1()
            .border_color(transparent_black())
            .bg(if active {
                palette.surface_selected
            } else {
                transparent_black()
            })
            .text_size(px(tokens::TEXT_BODY))
            .font_weight(if active {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(color)
            .aria_toggled(if active {
                Toggled::True
            } else {
                Toggled::False
            })
            .accessibility_label(label.clone())
            .hover(|style| style.bg(palette.surface_hover))
            .active(|style| style.bg(palette.surface_active))
            .focus_visible(|style| style.border_color(palette.accent))
            .child(ui::icon(name, 16.0, color))
            .child(label)
            .on_click(cx.listener(move |this, _, _, cx| this.set_section(section, cx)))
    }
}
