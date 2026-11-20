use gpui_kit::prelude::*;
use gpui_kit::{IntoElement, SharedString, div, px};

use crate::ui::tokens::{self, SPACE_1, TEXT_LABEL};

pub fn field(
    label: impl Into<SharedString>,
    control: impl IntoElement,
    error: Option<String>,
) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_1))
        .w_full()
        .child(
            div()
                .text_size(px(TEXT_LABEL))
                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                .text_color(palette.muted)
                .child(label.into()),
        )
        .child(control)
        .when_some(error, |this, error| {
            this.child(
                div()
                    .text_size(px(TEXT_LABEL))
                    .text_color(palette.danger_text)
                    .child(error),
            )
        })
}
