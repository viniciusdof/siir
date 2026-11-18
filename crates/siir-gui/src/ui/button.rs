//! Action button and primary action.

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ClickEvent, ElementId, FontWeight, IntoElement, RenderOnce, SharedString, Toggled, Window,
    px, transparent_black,
};

use crate::ui::icon;
use crate::ui::tokens::{self, RADIUS, ROW_HEIGHT, SPACE_2, SPACE_3, TEXT_BODY};

type ClickHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// How much weight an action carries.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// Ordinary actions.
    Secondary,
    /// The one commit on the screen.
    Primary,
    /// A quiet choice in a group, such as the game switch.
    Quiet,
}

/// A labelled action with an optional leading icon.
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: SharedString,
    icon: Option<IconName>,
    tone: Tone,
    selected: bool,
    toggled: Option<bool>,
    width: Option<f32>,
    on_click: Option<ClickHandler>,
}

/// A labelled action in the secondary tone.
pub fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Button {
    Button {
        id: id.into(),
        label: label.into(),
        icon: None,
        tone: Tone::Secondary,
        selected: false,
        toggled: None,
        width: None,
        on_click: None,
    }
}

impl Button {
    /// Shows a leading Lucide icon.
    pub fn icon(mut self, name: IconName) -> Self {
        self.icon = Some(name);
        self
    }

    /// Sets the weight of the action.
    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// Marks this control as the persistent selection.
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Marks the control as a toggle button and exposes its on/off state.
    pub fn toggled(mut self, toggled: bool) -> Self {
        self.toggled = Some(toggled);
        self
    }

    /// Fixes the width so a changing label does not move its neighbours.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Runs on pointer click, Enter, and Space.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.selected;
        let primary = self.tone == Tone::Primary;
        let quiet = self.tone == Tone::Quiet;
        let (background, foreground, edge) = if primary {
            (palette.accent, palette.accent_text, palette.accent)
        } else if quiet {
            (
                if selected {
                    palette.surface_selected
                } else {
                    transparent_black()
                },
                if selected {
                    palette.text
                } else {
                    palette.muted
                },
                transparent_black(),
            )
        } else {
            (palette.surface, palette.text, palette.control_border)
        };
        let base = BaseButton::new(self.id)
            .h(px(ROW_HEIGHT))
            .px(px(SPACE_3))
            .gap(px(SPACE_2))
            .flex()
            .items_center()
            .rounded(px(RADIUS))
            .bg(background)
            .border_1()
            .border_color(edge)
            .text_size(px(TEXT_BODY))
            .font_weight(if selected {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::NORMAL
            })
            .text_color(foreground)
            .when_some(self.toggled, |this, toggled| {
                this.aria_toggled(if toggled {
                    Toggled::True
                } else {
                    Toggled::False
                })
            })
            .when_some(self.width, |this, width| this.w(px(width)))
            .accessibility_label(self.label.clone())
            .when_some(self.icon, |this, name| {
                this.child(icon(name, 16.0, foreground))
            })
            .child(self.label)
            .when_some(self.on_click, |this, handler| this.on_click(handler));
        if primary {
            base.hover(|style| style.bg(palette.accent_hover))
                .active(|style| style.bg(palette.accent))
                .focus_visible(|style| style.border_color(palette.accent_text))
        } else {
            base.hover(|style| style.bg(palette.surface_hover))
                .active(|style| style.bg(palette.surface_active))
                .focus_visible(|style| style.border_color(palette.accent))
        }
    }
}
