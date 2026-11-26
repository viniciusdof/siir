use gpui_kit::prelude::*;
use gpui_kit::{FontWeight, IntoElement, SharedString, div, px, relative};

use siir_gui::format;

use crate::ui::tokens::{self, RADIUS, SPACE_2, SPACE_3, TEXT_BODY, TEXT_LABEL};

pub fn wear_bar(label: impl Into<SharedString>, wear: f32) -> impl IntoElement {
    let palette = tokens::palette();
    let clamped = wear.clamp(0.0, 1.0);
    let integrity = 1.0 - clamped;
    let bar_color = tokens::wear_color(clamped);
    div()
        .flex_1()
        .min_w(px(140.0))
        .bg(palette.surface)
        .border_1()
        .border_color(palette.border)
        .rounded(px(RADIUS))
        .px(px(SPACE_3))
        .py(px(SPACE_2))
        .flex()
        .flex_col()
        .gap(px(SPACE_2))
        .child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .gap(px(SPACE_2))
                .child(
                    div()
                        .text_size(px(TEXT_LABEL))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.muted)
                        .child(label.into()),
                )
                .child(
                    div()
                        .text_size(px(TEXT_BODY))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(bar_color)
                        .child(format::percent(clamped)),
                ),
        )
        .child(
            div()
                .w_full()
                .h(px(6.0))
                .rounded(px(999.0))
                .bg(palette.bg)
                .overflow_hidden()
                .child(
                    div()
                        .h_full()
                        .w(relative(integrity))
                        .rounded(px(999.0))
                        .bg(bar_color),
                ),
        )
}
