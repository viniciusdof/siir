//! Icon, label, and headline value for the overview and fleet counts.

use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{FontWeight, IntoElement, SharedString, div, px};

use crate::ui::icon;
use crate::ui::tokens::{self, SPACE_1, TEXT_LABEL, TEXT_METRIC};

/// One labelled number.
pub fn metric(
    name: IconName,
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex()
        .flex_col()
        .gap(px(SPACE_1))
        .min_w(px(72.0))
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(SPACE_1))
                .text_color(palette.muted)
                .text_size(px(TEXT_LABEL))
                .font_weight(FontWeight::SEMIBOLD)
                .child(icon(name, 14.0, palette.muted))
                .child(label.into()),
        )
        .child(
            div()
                .text_size(px(TEXT_METRIC))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text)
                .child(value.into()),
        )
}
