use gpui_kit::prelude::*;
use gpui_kit::{FontWeight, IntoElement, SharedString, div, px};

use crate::ui::tokens::{self, TEXT_SECTION};

pub fn section_title(title: impl Into<SharedString>) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .truncate()
        .text_size(px(TEXT_SECTION))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(palette.text)
        .child(title.into())
}
