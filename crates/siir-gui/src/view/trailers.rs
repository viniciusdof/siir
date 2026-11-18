//! The Trailers screen: the trailer list and the selected trailer's detail.

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, SharedString, div, px,
    transparent_black,
};

use siir_gui::format;
use siir_gui::i18n::t;
use siir_gui::view_data::TrailerRow;
use siir_save::Command;

use crate::ui::{self, tokens};
use crate::view::AppView;

/// Width of the trailer list column.
const LIST_WIDTH: f32 = 300.0;

impl AppView {
    pub(crate) fn render_trailers(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let rows = &open.view().trailers;
        let selected = rows
            .iter()
            .find(|trailer| Some(&trailer.id) == self.selected_trailer.as_ref());
        let mut list = div()
            .id("trailer-list")
            .w(px(LIST_WIDTH))
            .flex_shrink_0()
            .h_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(palette.border);
        for (ix, trailer) in rows.iter().enumerate() {
            list = list.child(self.trailer_row(trailer, ix == 0, cx));
        }
        let detail = match selected {
            Some(trailer) => self.trailer_detail(trailer, cx).into_any_element(),
            None => div()
                .flex_1()
                .p(px(tokens::SPACE_6))
                .child(ui::empty_state(
                    IconName::Container,
                    t("empty.no_trailers"),
                    t("empty.no_trailers_hint"),
                ))
                .into_any_element(),
        };
        div()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(tokens::SPACE_2))
                    .px(px(tokens::SPACE_4))
                    .py(px(tokens::SPACE_2))
                    .border_b_1()
                    .border_color(palette.border)
                    .child(
                        ui::button("repair-all-trailers", t("trailers.repair_all"))
                            .icon(IconName::Wrench)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.trailer_command(Command::RepairAllTrailers, window, cx)
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .min_h(px(0.0))
                    .child(list)
                    .child(detail),
            )
            .into_any_element()
    }

    /// One list row: definition, plate, and a Current tag.
    fn trailer_row(
        &self,
        trailer: &TrailerRow,
        first: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.selected_trailer.as_ref() == Some(&trailer.id);
        let mut label = format!("{}, {}", trailer.definition, trailer.plate);
        if trailer.current {
            label.push_str(&format!(", {}", t("vehicles.current")));
        }
        let id = trailer.id.clone();
        BaseButton::new((
            ElementId::from("trailer"),
            SharedString::from(trailer.id.clone()),
        ))
        .when(first, |this| this.track_focus(&self.trailers_focus))
        .w_full()
        .min_h(px(tokens::SAVE_ROW_HEIGHT))
        .px(px(tokens::SPACE_3))
        .py(px(tokens::SPACE_1))
        .flex()
        .items_center()
        .gap(px(tokens::SPACE_2))
        .border_1()
        .border_color(transparent_black())
        .bg(if selected {
            palette.surface_selected
        } else {
            transparent_black()
        })
        .selected(selected)
        .accessibility_label(label)
        .hover(|style| style.bg(palette.surface_hover))
        .active(|style| style.bg(palette.surface_active))
        .focus_visible(|style| style.border_color(palette.accent))
        .child(
            div()
                .flex_1()
                .min_w(px(0.0))
                .flex()
                .flex_col()
                .child(
                    div()
                        .truncate()
                        .text_size(px(tokens::TEXT_BODY))
                        .font_weight(if selected {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(palette.text)
                        .child(trailer.definition.clone()),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(tokens::TEXT_LABEL))
                        .text_color(palette.muted)
                        .child(trailer.plate.clone()),
                ),
        )
        .when(trailer.current, |this| {
            this.child(
                div()
                    .flex_shrink_0()
                    .text_size(px(tokens::TEXT_LABEL))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.accent)
                    .child(t("vehicles.current")),
            )
        })
        .on_click(
            cx.listener(move |this, _, window, cx| this.select_trailer(id.clone(), window, cx)),
        )
    }

    /// The selected trailer: plate, cargo mass, wear, and its actions.
    fn trailer_detail(&self, trailer: &TrailerRow, cx: &mut Context<'_, Self>) -> AnyElement {
        let palette = tokens::palette();
        let id = trailer.id.clone();
        let action = |key: &'static str, icon: IconName, command: fn(String) -> Command| {
            let id = id.clone();
            ui::button(SharedString::from(format!("trailer-{key}")), t(key))
                .icon(icon)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.trailer_command(command(id.clone()), window, cx)
                }))
        };
        div()
            .id("trailer-detail")
            .flex_1()
            .min_w(px(0.0))
            .h_full()
            .overflow_y_scroll()
            .child(
                div()
                    .max_w(px(tokens::FORM_WIDTH))
                    .p(px(tokens::SPACE_6))
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_6))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_1))
                            .child(ui::section_title(trailer.definition.clone()))
                            .child(
                                div()
                                    .text_size(px(tokens::TEXT_BODY))
                                    .text_color(palette.muted)
                                    .child(trailer.id.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(tokens::SPACE_4))
                            .child(div().flex_1().min_w(px(0.0)).child(ui::field(
                                t("field.trailer_plate"),
                                ui::text_field(
                                    "trailer-plate",
                                    &self.trailer_plate,
                                    t("field.trailer_plate"),
                                ),
                                self.trailer_plate_error.map(t),
                            )))
                            .child(div().flex_1().min_w(px(0.0)).child(ui::field(
                                t("field.cargo_mass"),
                                ui::text_field(
                                    "cargo-mass",
                                    &self.cargo_mass,
                                    t("field.cargo_mass"),
                                ),
                                self.cargo_mass_error.map(t),
                            ))),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::section_title(t("vehicles.wear")))
                            .child(ui::info_row(
                                t("wear.body"),
                                format::percent(trailer.body_wear),
                            ))
                            .child(ui::info_row(
                                t("wear.chassis"),
                                format::percent(trailer.chassis_wear),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(tokens::SPACE_2))
                            .child(action(
                                "trailers.repair",
                                IconName::Wrench,
                                Command::RepairTrailer,
                            ))
                            .child(action(
                                "trailers.unlock",
                                IconName::LockOpen,
                                Command::UnlockInRestrictedAreas,
                            ))
                            .child(
                                action(
                                    "trailers.attach",
                                    IconName::Link,
                                    Command::SetCurrentTrailer,
                                )
                                .disabled(trailer.current),
                            ),
                    ),
            )
            .into_any_element()
    }
}
