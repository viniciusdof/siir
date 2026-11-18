//! Styled discrete slider.

use gpui_kit::base::slider::SliderState;
use gpui_kit::base::{Slider as BaseSlider, SliderIndicator, SliderThumb, SliderTrack};
use gpui_kit::prelude::*;
use gpui_kit::{
    App, Entity, FocusHandle, IntoElement, RenderOnce, SharedString, Window, div, px, relative,
    transparent_black,
};

use crate::ui::tokens::{self, RADIUS};

/// A single-value slider in the application colours.
#[derive(IntoElement)]
pub struct Slider {
    state: Entity<SliderState>,
    focus: FocusHandle,
    label: SharedString,
}

/// A slider around a retained [`SliderState`].
pub fn slider(
    state: &Entity<SliderState>,
    focus: &FocusHandle,
    label: impl Into<SharedString>,
) -> Slider {
    Slider {
        state: state.clone(),
        focus: focus.clone(),
        label: label.into(),
    }
}

impl RenderOnce for Slider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let focused = self.focus.is_focused(window);
        let percentage = self.state.read(cx).percentage().end;
        BaseSlider::new(&self.state)
            .track_focus(&self.focus)
            .accessibility_label(self.label)
            .flex()
            .items_center()
            .w_full()
            .h(px(32.0))
            .px(px(8.0))
            .rounded(px(RADIUS))
            .border_1()
            .border_color(if focused {
                palette.accent
            } else {
                transparent_black()
            })
            .child(
                SliderTrack::new(&self.state)
                    .flex()
                    .items_center()
                    .h(px(32.0))
                    .w_full()
                    .flex_shrink_0()
                    .child(
                        SliderIndicator::new(&self.state)
                            .relative()
                            .w_full()
                            .h(px(6.0))
                            .rounded(px(3.0))
                            .bg(palette.surface)
                            .child(
                                div()
                                    .absolute()
                                    .h_full()
                                    .left(px(0.0))
                                    .right(relative(1.0 - percentage))
                                    .rounded(px(3.0))
                                    .bg(palette.muted),
                            )
                            .child(
                                SliderThumb::new(&self.state)
                                    .absolute()
                                    .top(px(-4.0))
                                    .left(relative(percentage))
                                    .ml(px(-7.0))
                                    .size(px(14.0))
                                    .rounded(px(7.0))
                                    .bg(palette.accent)
                                    .border_2()
                                    .border_color(palette.bg),
                            ),
                    ),
            )
    }
}
