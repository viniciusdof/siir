use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{Div, FontWeight, Hsla, IntoElement, SharedString, div, px};

use crate::ui::icon;
use crate::ui::tokens::{self, CARD_RADIUS, SPACE_1, SPACE_2, SPACE_4, TEXT_LABEL};

pub fn card() -> Div {
    let palette = tokens::palette();
    div()
        .bg(palette.panel)
        .border_1()
        .border_color(palette.border)
        .rounded(px(CARD_RADIUS))
        .p(px(SPACE_4))
}

pub fn pill(
    icon_name: Option<IconName>,
    label: impl Into<SharedString>,
    foreground: Hsla,
    background: Hsla,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .gap(px(SPACE_1))
        .px(px(SPACE_2))
        .py(px(3.0))
        .rounded(px(999.0))
        .bg(background)
        .text_size(px(TEXT_LABEL))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(foreground)
        .when_some(icon_name, |this, name| {
            this.child(icon(name, 12.0, foreground))
        })
        .child(label.into())
}
