//! Styled single-line input frame.

use gpui_kit::base::input::{Input, InputBase, InputState};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ElementId, Entity, Focusable, IntoElement, RenderOnce, SharedString, Window, px,
};

use crate::ui::tokens::{self, RADIUS, ROW_HEIGHT, SPACE_3, TEXT_BODY};

/// A framed input around a retained [`InputState`].
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    state: Entity<InputState>,
    label: SharedString,
}

/// A single-line input that reports its current value through the state.
pub fn text_field(
    id: impl Into<ElementId>,
    state: &Entity<InputState>,
    label: impl Into<SharedString>,
) -> TextField {
    TextField {
        id: id.into(),
        state: state.clone(),
        label: label.into(),
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        InputBase::new(self.id)
            .h(px(ROW_HEIGHT))
            .px(px(SPACE_3))
            .flex()
            .items_center()
            .rounded(px(RADIUS))
            .bg(palette.surface)
            .border_1()
            .border_color(palette.control_border)
            .text_size(px(TEXT_BODY))
            .text_color(palette.text)
            .accessibility_label(self.label)
            .focused(focused)
            .styles(|styles| styles.focused(|style| style.border_color(palette.accent)))
            .child(Input::new(&self.state))
    }
}
