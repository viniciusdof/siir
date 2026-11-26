use gpui_kit::assets::IconName;
use gpui_kit::base::Selectable;
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ElementId, FocusHandle, IntoElement, RenderOnce, Role, SharedString, Window, div, px,
};

use crate::ui::icon;
use crate::ui::tokens::{self, RADIUS, ROW_HEIGHT, SPACE_2, SPACE_3, TEXT_BODY};

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
