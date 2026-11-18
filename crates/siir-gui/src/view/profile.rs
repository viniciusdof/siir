//! The profile form: the name stored in `profile.sii`.

use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{IntoElement, div, px};

use siir_gui::i18n::t;

use crate::ui::{self, tokens};
use crate::view::AppView;

impl AppView {
    pub(crate) fn render_profile(&self) -> impl IntoElement {
        let has_profile = self.state.open().is_some_and(|open| open.has_profile());
        let body = if has_profile {
            ui::field(
                t("field.profile_name"),
                ui::text_field("profile-name", &self.profile_name, t("field.profile_name")),
                self.profile_name_error.map(t),
            )
            .into_any_element()
        } else {
            div()
                .flex()
                .justify_center()
                .pt(px(tokens::SPACE_4))
                .child(ui::empty_state(
                    IconName::CircleAlert,
                    t("profile.unavailable"),
                    t("profile.unavailable_hint"),
                ))
                .into_any_element()
        };
        div()
            .id("profile-scroll")
            .w_full()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .items_start()
            .justify_center()
            .overflow_y_scroll()
            .child(
                div()
                    .w_full()
                    .max_w(px(tokens::FORM_WIDTH))
                    .p(px(tokens::SPACE_6))
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_6))
                    .child(body),
            )
    }
}
