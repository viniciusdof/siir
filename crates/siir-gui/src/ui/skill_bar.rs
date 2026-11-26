use std::rc::Rc;

use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{App, FocusHandle, IntoElement, Role, Window, div, px};

use crate::ui::tokens::{self, SPACE_1};

pub fn skill_segments(
    id: &'static str,
    label: String,
    value: u64,
    focus: &FocusHandle,
    on_select: impl Fn(u64, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let palette = tokens::palette();
    let on_select = Rc::new(on_select);
    div()
        .id(id)
        .role(Role::Group)
        .aria_label(format!("{label}: {value} / 6"))
        .flex()
        .items_center()
        .gap(px(SPACE_1))
        .children((1u64..=6).map(move |step| {
            let filled = step <= value;
            let next = if value == step { step - 1 } else { step };
            let on_select = Rc::clone(&on_select);
            BaseButton::new((id, step as usize))
                .when(step == 1, |this| this.track_focus(focus))
                .flex_1()
                .h(px(22.0))
                .rounded(px(5.0))
                .border_1()
                .border_color(if filled {
                    palette.accent
                } else {
                    palette.control_border
                })
                .bg(if filled {
                    palette.accent
                } else {
                    palette.surface
                })
                .selected(filled)
                .accessibility_label(format!("{label}: {step} / 6"))
                .hover(move |style| {
                    if filled {
                        style.bg(palette.accent_hover)
                    } else {
                        style.bg(palette.surface_hover)
                    }
                })
                .focus_visible(|style| style.border_color(palette.text))
                .on_click(move |_, window, cx| on_select(next, window, cx))
        }))
}

pub fn skill_pips(value: u64) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex()
        .items_center()
        .gap(px(3.0))
        .w_full()
        .children((1u64..=6).map(move |step| {
            let filled = step <= value;
            div().flex_1().h(px(5.0)).rounded(px(999.0)).bg(if filled {
                palette.accent
            } else {
                palette.surface_selected
            })
        }))
}
