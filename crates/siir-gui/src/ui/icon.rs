//! Lucide icon at a fixed size and colour.

use gpui_kit::assets::IconName;
use gpui_kit::{Hsla, IntoElement, Styled, px, svg};

/// One icon from the bundled set. GPUI paints an SVG only with an explicit text
/// colour, so the caller passes it instead of relying on inheritance.
pub fn icon(name: IconName, size: f32, color: Hsla) -> impl IntoElement {
    svg()
        .path(name.path())
        .flex_shrink_0()
        .size(px(size))
        .text_color(color)
}
