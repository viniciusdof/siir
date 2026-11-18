//! Label and value with a hairline separator.

use gpui_kit::prelude::*;
use gpui_kit::{IntoElement, SharedString, div, px};

use crate::ui::tokens::{self, ROW_HEIGHT, SPACE_2, TEXT_BODY};

/// One row of read-only detail.
pub fn info_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(SPACE_2))
        .min_h(px(ROW_HEIGHT))
        .border_b_1()
        .border_color(palette.border)
        .child(
            div()
                .flex_shrink_0()
                .text_size(px(TEXT_BODY))
                .text_color(palette.muted)
                .child(label.into()),
        )
        .child(
            div()
                .max_w(px(360.0))
                .text_size(px(TEXT_BODY))
                .text_color(palette.text)
                .child(value.into()),
        )
}
