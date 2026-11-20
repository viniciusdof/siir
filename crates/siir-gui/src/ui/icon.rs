use gpui_kit::assets::IconName;
use gpui_kit::{Hsla, IntoElement, Styled, px, svg};

pub fn icon(name: IconName, size: f32, color: Hsla) -> impl IntoElement {
    svg()
        .path(name.path())
        .flex_shrink_0()
        .size(px(size))
        .text_color(color)
}
