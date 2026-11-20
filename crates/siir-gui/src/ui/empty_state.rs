use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{IntoElement, SharedString, TextAlign, div, px};

use crate::ui::icon;
use crate::ui::tokens::{self, SPACE_2, SPACE_3, TEXT_BODY, TEXT_SECTION};

pub fn empty_state(
    name: IconName,
    title: impl Into<SharedString>,
    hint: impl Into<SharedString>,
) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(SPACE_2))
        .p(px(SPACE_3))
        .max_w(px(280.0))
        .text_align(TextAlign::Center)
        .child(
            div()
                .text_color(palette.muted)
                .child(icon(name, 24.0, palette.muted)),
        )
        .child(
            div()
                .text_size(px(TEXT_SECTION))
                .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                .text_color(palette.text)
                .child(title.into()),
        )
        .child(
            div()
                .text_size(px(TEXT_BODY))
                .text_color(palette.muted)
                .child(hint.into()),
        )
}
