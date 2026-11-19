//! Styled single-line input frame.

use gpui_kit::base::input::{Input, InputBase, InputState};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ElementId, Entity, Focusable, IntoElement, RenderOnce, SharedString, Window, div, px,
};

use crate::ui::tokens::{self, RADIUS, ROW_HEIGHT, SPACE_2, SPACE_3, TEXT_BODY};

/// A framed input around a retained [`InputState`].
#[derive(IntoElement)]
pub struct TextField {
    id: ElementId,
    state: Entity<InputState>,
    label: SharedString,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
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
        prefix: None,
        suffix: None,
    }
}

impl TextField {
    /// Shows a fixed symbol before the value, such as a currency.
    pub fn prefix(mut self, prefix: impl Into<SharedString>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Shows a fixed symbol after the value, such as a currency.
    pub fn suffix(mut self, suffix: impl Into<SharedString>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }
}

impl RenderOnce for TextField {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let focused = self.state.read(cx).focus_handle(cx).is_focused(window);
        let prefix = self.prefix.clone();
        let suffix = self.suffix.clone();
        let symbol =
            move |text: SharedString| div().flex_shrink_0().text_color(palette.muted).child(text);
        InputBase::new(self.id)
            .h(px(ROW_HEIGHT))
            .px(px(SPACE_3))
            .gap(px(SPACE_2))
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
            .when_some(prefix, |this, prefix| this.child(symbol(prefix)))
            .child(div().flex_1().min_w(px(0.0)).child(Input::new(&self.state)))
            .when_some(suffix, |this, suffix| this.child(symbol(suffix)))
    }
}
