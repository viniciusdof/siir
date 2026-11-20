use std::rc::Rc;

use gpui_kit::base::{Dialog, DialogDescription, DialogPopup, DialogTitle};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, App, FocusHandle, FontWeight, IntoElement, RenderOnce, SharedString, Window, div,
    px,
};

use crate::ui::tokens::{self, RADIUS, SPACE_2, SPACE_3, SPACE_4, TEXT_BODY, TEXT_SECTION};

type CancelHandler = Rc<dyn Fn(&mut Window, &mut App) + 'static>;

const WIDTH: f32 = 440.0;

#[derive(IntoElement)]
pub struct Confirm {
    title: SharedString,
    message: SharedString,
    focus: FocusHandle,
    open: bool,
    actions: Vec<AnyElement>,
    on_cancel: Option<CancelHandler>,
}

pub fn confirm(
    title: impl Into<SharedString>,
    message: impl Into<SharedString>,
    focus: &FocusHandle,
) -> Confirm {
    Confirm {
        title: title.into(),
        message: message.into(),
        focus: focus.clone(),
        open: false,
        actions: Vec::new(),
        on_cancel: None,
    }
}

impl Confirm {
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.actions.push(action.into_any_element());
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Confirm {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let on_cancel = self.on_cancel;
        Dialog::new(cx)
            .open(self.open)
            .focus_handle(self.focus)
            .accessibility_label(self.title.clone())
            .close_on_backdrop_press(false)
            .on_ok(|_, _, cx| {
                cx.propagate();
                false
            })
            .on_cancel(move |_, window, cx| {
                if let Some(handler) = &on_cancel {
                    handler(window, cx);
                }
                true
            })
            .backdrop(div().absolute().inset_0().bg(palette.scrim))
            .popup(
                DialogPopup::new()
                    .w(px(WIDTH))
                    .flex()
                    .flex_col()
                    .bg(palette.panel)
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(RADIUS))
                    .overflow_hidden()
                    .child(
                        DialogTitle::new()
                            .px(px(SPACE_4))
                            .py(px(SPACE_3))
                            .bg(palette.surface)
                            .border_b_1()
                            .border_color(palette.border)
                            .text_size(px(TEXT_SECTION))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(self.title),
                    )
                    .child(
                        DialogDescription::new()
                            .p(px(SPACE_4))
                            .text_size(px(TEXT_BODY))
                            .text_color(palette.text)
                            .child(self.message),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .justify_end()
                            .gap(px(SPACE_2))
                            .px(px(SPACE_4))
                            .pb(px(SPACE_4))
                            .children(self.actions),
                    ),
            )
    }
}
