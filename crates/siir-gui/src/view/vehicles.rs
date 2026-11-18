//! The Vehicles screen: the vehicle list and the selected vehicle's detail.

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, SharedString, div, px,
    transparent_black,
};

use siir_gui::format;
use siir_gui::i18n::t;
use siir_gui::view_data::VehicleRow;
use siir_save::Command;

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

/// Width of the vehicle list column.
const LIST_WIDTH: f32 = 300.0;

impl AppView {
    pub(crate) fn render_vehicles(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let rows: Vec<&VehicleRow> = open
            .view()
            .vehicles
            .iter()
            .filter(|vehicle| self.show_all_vehicles || vehicle.owned)
            .collect();
        let selected = rows
            .iter()
            .find(|vehicle| Some(&vehicle.id) == self.selected_vehicle.as_ref())
            .copied();
        let mut list = div()
            .id("vehicle-list")
            .w(px(LIST_WIDTH))
            .flex_shrink_0()
            .h_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .border_r_1()
            .border_color(palette.border);
        for (ix, vehicle) in rows.iter().enumerate() {
            list = list.child(self.vehicle_row(vehicle, ix == 0, cx));
        }
        let detail = match selected {
            Some(vehicle) => self.vehicle_detail(vehicle, cx).into_any_element(),
            None => div()
                .flex_1()
                .p(px(tokens::SPACE_6))
                .child(ui::empty_state(
                    IconName::Truck,
                    t("empty.no_vehicles"),
                    t("empty.no_vehicles_hint"),
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
                        ui::button("repair-all", t("vehicles.repair_all"))
                            .icon(IconName::Wrench)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.vehicle_command(Command::RepairAllVehicles, window, cx)
                            })),
                    )
                    .child(
                        ui::button("fill-all", t("vehicles.fill_all"))
                            .icon(IconName::Fuel)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.vehicle_command(Command::FillAllVehicles, window, cx)
                            })),
                    )
                    .child(div().flex_1())
                    .child(
                        ui::button(
                            "toggle-all-vehicles",
                            t(if self.show_all_vehicles {
                                "vehicles.show_owned"
                            } else {
                                "vehicles.show_all"
                            }),
                        )
                        .tone(Tone::Quiet)
                        .icon(IconName::ListFilter)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.toggle_all_vehicles(window, cx)),
                        ),
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

    /// One list row: the model's game name, its plate, and a Current tag.
    fn vehicle_row(
        &self,
        vehicle: &VehicleRow,
        first: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.selected_vehicle.as_ref() == Some(&vehicle.id);
        let name = self.vehicle_name(vehicle.model.as_deref());
        let mut label = format!("{name}, {}", vehicle.plate);
        if vehicle.current {
            label.push_str(&format!(", {}", t("vehicles.current")));
        }
        let id = vehicle.id.clone();
        BaseButton::new((
            ElementId::from("vehicle"),
            SharedString::from(vehicle.id.clone()),
        ))
        .when(first, |this| this.track_focus(&self.vehicles_focus))
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
                        .child(name),
                )
                .child(
                    div()
                        .truncate()
                        .text_size(px(tokens::TEXT_LABEL))
                        .text_color(palette.muted)
                        .child(vehicle.plate.clone()),
                ),
        )
        .when(vehicle.current, |this| {
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
            cx.listener(move |this, _, window, cx| this.select_vehicle(id.clone(), window, cx)),
        )
    }

    /// The selected vehicle: odometer, fuel, wear, and its actions.
    fn vehicle_detail(&self, vehicle: &VehicleRow, cx: &mut Context<'_, Self>) -> AnyElement {
        let palette = tokens::palette();
        let fuel = self.fuel.read(cx).value().end().round();
        let id = vehicle.id.clone();
        // `Command` is not `Clone`, so each click builds its command from the id.
        let action = |key: &'static str, icon: IconName, command: fn(String) -> Command| {
            let id = id.clone();
            ui::button(SharedString::from(format!("vehicle-{key}")), t(key))
                .icon(icon)
                .on_click(cx.listener(move |this, _, window, cx| {
                    this.vehicle_command(command(id.clone()), window, cx)
                }))
        };
        div()
            .id("vehicle-detail")
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
                            .child(ui::section_title(
                                self.vehicle_name(vehicle.model.as_deref()),
                            ))
                            .child(
                                div()
                                    .text_size(px(tokens::TEXT_BODY))
                                    .text_color(palette.muted)
                                    .child(vehicle.plate.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(tokens::SPACE_4))
                            .child(div().flex_1().min_w(px(0.0)).when(
                                vehicle.odometer.is_some(),
                                |this| {
                                    this.child(ui::field(
                                        t("field.odometer"),
                                        ui::text_field(
                                            "odometer",
                                            &self.odometer,
                                            t("field.odometer"),
                                        ),
                                        self.odometer_error.map(t),
                                    ))
                                },
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w(px(0.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(tokens::SPACE_1))
                                    .child(
                                        div()
                                            .flex()
                                            .justify_between()
                                            .text_size(px(tokens::TEXT_LABEL))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette.muted)
                                            .child(t("field.fuel"))
                                            .child(format::percent(fuel / 100.0)),
                                    )
                                    .child(ui::slider(
                                        &self.fuel,
                                        &self.fuel_focus,
                                        t("field.fuel"),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::section_title(t("vehicles.wear")))
                            .child(ui::info_row(
                                t("wear.engine"),
                                format::percent(vehicle.engine_wear),
                            ))
                            .child(ui::info_row(
                                t("wear.transmission"),
                                format::percent(vehicle.transmission_wear),
                            ))
                            .child(ui::info_row(
                                t("wear.cabin"),
                                format::percent(vehicle.cabin_wear),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(tokens::SPACE_2))
                            .child(action(
                                "vehicles.repair",
                                IconName::Wrench,
                                Command::RepairVehicle,
                            ))
                            .child(action(
                                "vehicles.fill",
                                IconName::Fuel,
                                Command::SetInfiniteFuel,
                            ))
                            .child(
                                action(
                                    "vehicles.remove_badges",
                                    IconName::Eraser,
                                    Command::RemoveBadge,
                                )
                                .disabled(!vehicle.badges),
                            )
                            .child(
                                action(
                                    "vehicles.set_current",
                                    IconName::KeyRound,
                                    Command::SetCurrentVehicle,
                                )
                                .disabled(vehicle.current || !vehicle.owned),
                            ),
                    ),
            )
            .into_any_element()
    }
}
