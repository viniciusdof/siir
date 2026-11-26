use gpui_kit::assets::IconName;
use gpui_kit::base::{Checkbox as BaseCheckbox, CheckboxIndicator, CheckboxState};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ClickEvent, ElementId, FocusHandle, FontWeight, IntoElement, RenderOnce, SharedString,
    Window, div, px, transparent_black,
};

use crate::ui::{icon, tokens};

type ChangeHandler = Box<dyn Fn(&bool, &mut Window, &mut App) + 'static>;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: SharedString,
    badge: Option<SharedString>,
    checked: bool,
    focus: Option<FocusHandle>,
    on_change: Option<ChangeHandler>,
}

pub fn checkbox(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    checked: bool,
) -> Checkbox {
    Checkbox {
        id: id.into(),
        label: label.into(),
        badge: None,
        checked,
        focus: None,
        on_change: None,
    }
}

impl Checkbox {
    pub fn badge(mut self, code: impl Into<SharedString>) -> Self {
        self.badge = Some(code.into());
        self
    }

    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }

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
        let badge = self.badge;
        BaseCheckbox::new(self.id)
            .checked(checked)
            .accessibility_label(label.clone())
            .w_full()
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_2))
            .px(px(tokens::SPACE_3))
            .py(px(tokens::SPACE_2))
            .rounded(px(tokens::RADIUS))
            .bg(if checked {
                palette.accent_soft
            } else {
                palette.surface
            })
            .border_1()
            .border_color(if checked {
                palette.accent
            } else {
                palette.border
            })
            .hover(|style| style.bg(palette.surface_hover))
            .focus_visible(|style| style.border_color(palette.text))
            .when_some(self.focus, |this, handle| this.track_focus(&handle))
            .child(
                CheckboxIndicator::new()
                    .checked(checked)
                    .flex()
                    .flex_shrink_0()
                    .items_center()
                    .justify_center()
                    .size(px(20.0))
                    .rounded(px(5.0))
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
                    .child(match badge {
                        Some(code) => div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(if checked {
                                palette.accent_text
                            } else {
                                palette.muted
                            })
                            .child(code)
                            .into_any_element(),
                        None => div()
                            .when(checked, |this| {
                                this.child(icon(IconName::Check, 12.0, palette.accent_text))
                            })
                            .into_any_element(),
                    }),
            )
            .child(
                div()
                    .min_w(px(0.0))
                    .truncate()
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
