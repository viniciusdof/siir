use std::rc::Rc;

use gpui_kit::assets::IconName;
use gpui_kit::base::{Popover, Selectable};
use gpui_kit::prelude::*;
use gpui_kit::{
    Anchor, App, ElementId, FocusHandle, IntoElement, RenderOnce, Role, SharedString, Window, div,
    px,
};

use crate::ui::tokens::{self, RADIUS, ROW_HEIGHT, SPACE_1, SPACE_2, SPACE_3, TEXT_BODY};
use crate::ui::{Tone, button, icon};

#[derive(IntoElement)]
pub struct SelectField {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    focus: Option<FocusHandle>,
    open: bool,
}

pub fn select_field(
    id: impl Into<ElementId>,
    value: impl Into<SharedString>,
    label: impl Into<SharedString>,
) -> SelectField {
    SelectField {
        id: id.into(),
        value: value.into(),
        label: label.into(),
        focus: None,
        open: false,
    }
}

impl SelectField {
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }
}

impl Selectable for SelectField {
    fn selected(mut self, selected: bool) -> Self {
        self.open = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.open
    }
}

impl RenderOnce for SelectField {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let border = if self.open {
            palette.accent
        } else {
            palette.control_border
        };
        div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_between()
            .gap(px(SPACE_2))
            .h(px(ROW_HEIGHT))
            .px(px(SPACE_3))
            .w_full()
            .rounded(px(RADIUS))
            .bg(palette.surface)
            .border_1()
            .border_color(border)
            .text_size(px(TEXT_BODY))
            .text_color(palette.text)
            .role(Role::Button)
            .aria_label(self.label)
            .when_some(self.focus, |this, handle| this.track_focus(&handle))
            .cursor_pointer()
            .hover(|style| style.bg(palette.surface_hover))
            .focus_visible(|style| style.border_color(palette.accent))
            .child(div().min_w(px(0.0)).truncate().child(self.value))
            .child(icon(IconName::ChevronDown, 16.0, palette.muted))
    }
}

pub fn menu_popover(
    prefix: impl Into<SharedString>,
    anchor: Anchor,
    trigger: impl IntoElement + Selectable + 'static,
    options: Vec<(String, String)>,
    selected: Option<String>,
    on_select: impl Fn(String, &mut Window, &mut App) + 'static,
) -> Popover {
    let prefix = prefix.into();
    let palette = tokens::palette();
    let on_select = Rc::new(on_select);
    let list_id = SharedString::from(format!("{prefix}-list"));
    let option_prefix = prefix.clone();
    Popover::new(SharedString::from(format!("{prefix}-menu")))
        .anchor(anchor)
        .trigger(trigger)
        .content(move |_, _, cx| {
            let popover = cx.entity();
            let mut menu = div()
                .id(list_id.clone())
                .flex()
                .flex_col()
                .gap(px(SPACE_1))
                .p(px(SPACE_1))
                .max_h(px(260.0))
                .overflow_y_scroll()
                .on_scroll(|_, _, cx| cx.stop_propagation())
                .bg(palette.panel)
                .border_1()
                .border_color(palette.border)
                .rounded(px(RADIUS));
            for (value, label) in &options {
                let is_selected = selected.as_deref() == Some(value.as_str());
                let option_id = SharedString::from(format!("{option_prefix}-option-{value}"));
                let value = value.clone();
                let on_select = Rc::clone(&on_select);
                let popover = popover.clone();
                menu = menu.child(
                    button(option_id, label.clone())
                        .tone(Tone::Quiet)
                        .full_width()
                        .selected(is_selected)
                        .on_click(move |_, window, cx| {
                            if !is_selected {
                                on_select(value.clone(), window, cx);
                            }
                            popover.update(cx, |state, cx| state.dismiss(window, cx));
                        }),
                );
            }
            menu.into_any_element()
        })
}
