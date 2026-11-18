//! Checkbox with a check mark and label.

use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox as BaseCheckbox, CheckboxIndicator, CheckboxState};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ClickEvent, ElementId, FocusHandle, IntoElement, RenderOnce, SharedString, Window, div,
    px, transparent_black,
};

use crate::ui::{icon, tokens};

type ChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

/// One independent choice with a visible check mark.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: SharedString,
    checked: bool,
    focus: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
}

/// A labelled checkbox.
pub fn checkbox(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    checked: bool,
) -> Checkbox {
    Checkbox {
        id: id.into(),
        label: label.into(),
        checked,
        focus: None,
        on_change: None,
    }
}

impl Checkbox {
    /// Uses `handle` for focus, so the owner can move focus to the checkbox.
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }

    /// Reports the next checked state on pointer, Space, or assistive activation.
    pub fn on_change(mut self, handler: impl Fn(&bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let checked = self.checked;
        let label = self.label;
        BaseCheckbox::new(self.id)
            .checked(checked)
            .accessibility_label(label.clone())
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_2))
            .px(px(tokens::SPACE_1))
            .py(px(2.0))
            .rounded(px(tokens::RADIUS))
            .border_1()
            .border_color(transparent_black())
            .hover(|style| style.bg(palette.surface_hover))
            .focus_visible(|style| style.border_color(palette.accent))
            .when_some(self.focus, |this, handle| this.track_focus(&handle))
            .child(
                CheckboxIndicator::new()
                    .checked(checked)
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(18.0))
                    .rounded(px(3.0))
                    .border_1()
                    .border_color(if checked {
                        palette.accent
                    } else {
                        palette.control_border
                    })
                    .bg(if checked {
                        palette.accent
                    } else {
                        transparent_black()
                    })
                    .when(checked, |this| {
                        this.child(icon(IconName::Check, 12.0, palette.accent_text))
                    }),
            )
            .child(
                div()
                    .text_size(px(tokens::TEXT_BODY))
                    .text_color(palette.text)
                    .child(label),
            )
            .when_some(self.on_change, |this, handler| {
                this.on_change(move |state: CheckboxState, _: &ClickEvent, window, cx| {
                    handler(&(state == CheckboxState::Checked), window, cx)
                })
            })
    }
}
