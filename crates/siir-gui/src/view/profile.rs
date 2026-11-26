use gpui_kit::assets::IconName;
use gpui_kit::base::input::{InputEvent, InputState};
use gpui_kit::prelude::*;
use gpui_kit::{Context, Entity, Focusable, IntoElement, Subscription, Window, div, px};

use siir_gui::edit;
use siir_gui::i18n::t;
use siir_save::Command;

use crate::ui::{self, tokens};
use crate::view::AppView;

pub(super) struct ProfileState {
    name: Entity<InputState>,
    name_error: Option<&'static str>,
}

impl ProfileState {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<'_, AppView>,
        subscriptions: &mut Vec<Subscription>,
    ) -> Self {
        let name = AppView::input(window, cx);
        subscriptions.push(cx.observe(&name, |this, _, cx| {
            this.profile.name_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&name, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.apply_profile_name_text(cx);
            }
        }));
        let focus = name.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&focus, window, |this, _, cx| this.commit_profile_name(cx)));
        Self {
            name,
            name_error: None,
        }
    }

    pub(super) fn clear_errors(&mut self) {
        self.name_error = None;
    }

    pub(super) fn is_valid(&self) -> bool {
        self.name_error.is_none()
    }
}

impl AppView {
    pub(super) fn sync_profile(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let Some(profile_name) = self
            .state
            .open()
            .map(|open| open.view().profile_name.clone())
        else {
            return;
        };
        self.profile.name.update(cx, |input, cx| {
            input.set_value(profile_name, window, cx);
        });
    }

    fn apply_profile_name_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some(open) = self.state.open().filter(|open| open.has_profile()) else {
            return true;
        };
        let current = open.view().profile_name.clone();
        let text = self.profile.name.read(cx).value().to_string();
        if text.trim() == current {
            self.profile.name_error = None;
            return true;
        }
        let Ok(Command::RenameProfile(name)) = edit::profile_name(&text) else {
            return false;
        };
        self.profile.name_error = None;
        if name != current {
            self.apply(Command::RenameProfile(name), cx);
        }
        true
    }

    pub(super) fn commit_profile_name(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_profile_name_text(cx) {
            self.profile.name_error = Some(edit::FieldError::Name.key());
            cx.notify();
        }
    }

    pub(crate) fn render_profile(&self) -> impl IntoElement {
        let has_profile = self.state.open().is_some_and(|open| open.has_profile());
        let body = if has_profile {
            ui::card()
                .child(ui::field(
                    t("field.profile_name"),
                    ui::text_field("profile-name", &self.profile.name, t("field.profile_name")),
                    self.profile.name_error.map(t),
                ))
                .into_any_element()
        } else {
            ui::card()
                .flex()
                .justify_center()
                .py(px(tokens::SPACE_6))
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
                    .gap(px(tokens::SPACE_4))
                    .child(body),
            )
    }
}
