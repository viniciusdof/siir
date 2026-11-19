//! A framed control that shows the chosen value of a list and a chevron.

use gpui_kit::assets::IconName;
use gpui_kit::base::Selectable;
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ElementId, FocusHandle, IntoElement, RenderOnce, Role, SharedString, Window, div, px,
};

use crate::ui::tokens::{self, RADIUS, ROW_HEIGHT, SPACE_2, SPACE_3, TEXT_BODY};

/// A full-width control that looks like a text field, carries the value of a
/// closed list, and paints an accent border while the list it opens is open.
///
/// It carries no click handler: it is meant as a popover's trigger, which
/// opens on its own. The popover marks it open through [`Selectable`]. With a
/// focus handle it is focusable, and the popover opens on the confirm key.
#[derive(IntoElement)]
pub struct SelectField {
    id: ElementId,
    value: SharedString,
    label: SharedString,
    focus: Option<FocusHandle>,
    open: bool,
}

/// A [`SelectField`] for `value`, named `label` for assistive technology.
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
    /// Uses `handle` for focus, so the keyboard can reach the list it opens.
    pub fn track_focus(mut self, handle: &FocusHandle) -> Self {
        self.focus = Some(handle.clone());
        self
    }
}

impl Selectable for SelectField {
    /// The popover reports its open state through `selected`.
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
            .child(div().min_w(px(0.0)).truncate().child(self.value))
            .child(IconName::ChevronDown)
    }
}
