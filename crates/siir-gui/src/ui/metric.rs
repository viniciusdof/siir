use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{FontWeight, Hsla, IntoElement, SharedString, div, px};

use crate::ui::icon;
use crate::ui::tokens::{self, CARD_RADIUS, RADIUS, SPACE_3, TEXT_LABEL, TEXT_METRIC};

pub fn metric(
    name: IconName,
    tint: Hsla,
    label: impl Into<SharedString>,
    value: impl IntoElement,
) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex_1()
        .min_w(px(140.0))
        .bg(palette.panel)
        .border_1()
        .border_color(palette.border)
        .rounded(px(CARD_RADIUS))
        .p(px(SPACE_3))
        .flex()
        .items_center()
        .gap(px(SPACE_3))
        .child(
            div()
                .flex_shrink_0()
                .size(px(36.0))
                .rounded(px(RADIUS))
                .bg(palette.surface)
                .flex()
                .items_center()
                .justify_center()
                .child(icon(name, 18.0, tint)),
        )
        .child(
            div()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(TEXT_METRIC))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.text)
                        .child(value),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(TEXT_LABEL))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.muted)
                        .child(label.into()),
                ),
        )
}
