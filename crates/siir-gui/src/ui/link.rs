//! In-app link from a shown value to the screen that edits it.

use gpui_kit::base::Link;
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ClickEvent, ElementId, IntoElement, SharedString, Window, px, transparent_black,
};

use crate::ui::tokens::{self, RADIUS, SPACE_1};

/// `content` underlined on hover and focus, so it does not read as a link by
/// colour alone.
pub fn link(
    id: impl Into<ElementId>,
    accessible: impl Into<SharedString>,
    content: impl IntoElement,
    on_activate: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let palette = tokens::palette();
    Link::new(id)
        .accessibility_label(accessible)
        .on_activate(on_activate)
        .px(px(SPACE_1))
        .mx(px(-SPACE_1))
        .rounded(px(RADIUS))
        .border_1()
        .border_color(transparent_black())
        .cursor_pointer()
        .hover(|style| style.underline())
        .focus_visible(|style| style.underline().border_color(palette.accent))
        .child(content)
}
