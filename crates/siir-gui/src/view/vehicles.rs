use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::base::input::{InputEvent, InputState, MaskPattern};
use gpui_kit::base::slider::SliderState;
use gpui_kit::prelude::*;
use gpui_kit::{
    Anchor, AnyElement, Context, ElementId, Entity, FocusHandle, Focusable, FontWeight,
    IntoElement, SharedString, Subscription, Window, div, px, transparent_black,
};

use siir_gui::edit;
use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_gui::view_data::VehicleRow;
use siir_save::Command;

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

const LIST_WIDTH: f32 = 290.0;

const PART_CELL: f32 =
    (tokens::FORM_WIDTH - 2.0 * tokens::SPACE_6 - 2.0 * tokens::SPACE_4 - tokens::SPACE_4) / 2.0;

const PART_COUNT: usize = PartKind::ALL.len();

pub(super) struct VehiclesState {
    pub(super) selected: Option<String>,
    pub(super) show_used_market: bool,
    odometer: Entity<InputState>,
    odometer_error: Option<&'static str>,
    plate: Entity<InputState>,
    plate_error: Option<&'static str>,
    used_price: Entity<InputState>,
    used_price_error: Option<&'static str>,
    pub(super) used_price_dirty: bool,
    fuel: Entity<SliderState>,
    fuel_focus: FocusHandle,
    country_focus: FocusHandle,
    parts_focus: Vec<FocusHandle>,
    pub(super) list_focus: FocusHandle,
}

impl VehiclesState {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<'_, AppView>,
        subscriptions: &mut Vec<Subscription>,
    ) -> Self {
        let odometer = AppView::number_input(window, cx, false);
        let fuel = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(0.0)
        });
        let fuel_focus = cx.focus_handle().tab_stop(true);
        let plate = AppView::input(window, cx);
        let used_price = AppView::number_input(window, cx, false);

        subscriptions.push(cx.observe(&odometer, |this, _, cx| {
            this.vehicles.odometer_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&odometer, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.apply_odometer_text(cx);
            }
        }));
        let odometer_focus = odometer.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&odometer_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&odometer_focus, window, |this, _, cx| {
            this.commit_odometer(cx)
        }));

        subscriptions.push(cx.observe(&fuel, |this, _, cx| {
            if !this.syncing {
                this.apply_fuel(cx);
            }
            cx.notify();
        }));
        subscriptions.push(cx.on_focus(&fuel_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&fuel_focus, window, |_, _, cx| cx.notify()));

        subscriptions.push(cx.observe(&plate, |this, _, cx| {
            this.vehicles.plate_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&plate, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.apply_plate_text(cx);
            }
        }));
        let plate_focus = plate.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&plate_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&plate_focus, window, |this, _, cx| this.commit_plate(cx)));

        subscriptions.push(cx.observe(&used_price, |this, _, cx| {
            this.vehicles.used_price_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.subscribe_in(
            &used_price,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Change)
                    && !this.syncing
                    && this.apply_used_price_text(cx) == Some(false)
                {
                    this.sync_used_price(window, cx);
                }
            },
        ));
        let used_price_focus = used_price.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&used_price_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&used_price_focus, window, |this, _, cx| {
            this.commit_used_price(cx)
        }));

        Self {
            selected: None,
            show_used_market: false,
            odometer,
            odometer_error: None,
            plate,
            plate_error: None,
            used_price,
            used_price_error: None,
            used_price_dirty: false,
            fuel,
            fuel_focus,
            country_focus: cx.focus_handle().tab_stop(true),
            parts_focus: (0..PART_COUNT)
                .map(|_| cx.focus_handle().tab_stop(true))
                .collect(),
            list_focus: cx.focus_handle().tab_stop(true),
        }
    }

    pub(super) fn clear_errors(&mut self) {
        self.odometer_error = None;
        self.plate_error = None;
        self.used_price_error = None;
        self.used_price_dirty = false;
    }

    pub(super) fn is_valid(&self) -> bool {
        self.odometer_error.is_none()
            && self.plate_error.is_none()
            && self.used_price_error.is_none()
    }

    pub(super) fn used_price_valid(&self) -> bool {
        self.used_price_error.is_none()
    }
}

impl AppView {
    fn selected_row(&self) -> Option<&VehicleRow> {
        let id = self.vehicles.selected.as_ref()?;
        self.state
            .open()?
            .view()
            .vehicles
            .iter()
            .find(|vehicle| &vehicle.id == id)
    }

    fn select_vehicle(&mut self, id: String, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.vehicles.selected = Some(id);
        self.sync_vehicle(window, cx);
        cx.notify();
    }

    fn toggle_used_market(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.vehicles.show_used_market = !self.vehicles.show_used_market;
        let listed = self
            .selected_row()
            .is_some_and(|row| row.owned != self.vehicles.show_used_market);
        if !listed {
            self.vehicles.selected = self.default_vehicle();
            self.sync_vehicle(window, cx);
        }
        cx.notify();
    }

    pub(super) fn default_vehicle(&self) -> Option<String> {
        let vehicles = &self.state.open()?.view().vehicles;
        let listed = |vehicle: &&VehicleRow| vehicle.owned != self.vehicles.show_used_market;
        vehicles
            .iter()
            .filter(listed)
            .find(|vehicle| vehicle.current)
            .or_else(|| vehicles.iter().find(listed))
            .map(|vehicle| vehicle.id.clone())
    }

    pub(super) fn sync_vehicle(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some((odometer, fuel)) = self.selected_row().map(|row| (row.odometer, row.fuel)) else {
            return;
        };
        self.vehicles.odometer_error = None;
        let plate = self
            .selected_row()
            .map(|row| row.plate.clone())
            .unwrap_or_default();
        self.vehicles.plate_error = None;
        self.vehicles.plate.update(cx, |input, cx| {
            input.set_value(plate, window, cx);
        });
        self.vehicles.odometer.update(cx, |input, cx| {
            input.set_mask_pattern(
                MaskPattern::Number {
                    separator: Some(format::grouping(game)),
                    fraction: None,
                },
                window,
                cx,
            );
            input.set_value(
                format::number(odometer.unwrap_or_default(), game),
                window,
                cx,
            );
        });
        self.vehicles.used_price_error = None;
        self.vehicles.used_price_dirty = false;
        let currency = self.currency();
        let price = self
            .selected_row()
            .and_then(|row| row.listing.as_ref())
            .map(|listing| format::amount(currency.from_native(listing.price), game))
            .unwrap_or_default();
        self.syncing = true;
        self.vehicles.used_price.update(cx, |input, cx| {
            input.set_mask_pattern(
                MaskPattern::Number {
                    separator: Some(format::grouping(game)),
                    fraction: None,
                },
                window,
                cx,
            );
            input.set_value(price, window, cx);
        });
        self.vehicles
            .fuel
            .update(cx, |state, cx| state.set_value(fuel * 100.0, window, cx));
        self.syncing = false;
    }

    pub(super) fn sync_used_price(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some(price) = self
            .selected_row()
            .and_then(|row| row.listing.as_ref())
            .map(|listing| listing.price)
        else {
            return;
        };
        let shown = format::amount(self.currency().from_native(price), game);
        self.vehicles.used_price_error = None;
        self.vehicles.used_price_dirty = false;
        self.syncing = true;
        self.vehicles.used_price.update(cx, |input, cx| {
            input.set_value(shown, window, cx);
        });
        self.syncing = false;
    }

    fn apply_plate_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, plate, country)) = self
            .selected_row()
            .filter(|row| row.listing.is_none())
            .map(|row| (row.id.clone(), row.plate.clone(), row.country.clone()))
        else {
            return true;
        };
        let text = self.vehicles.plate.read(cx).value().to_string();
        if text.trim() == plate {
            self.vehicles.plate_error = None;
            return true;
        }
        let Ok(command) = edit::plate(&id, &text, country.as_deref()) else {
            return false;
        };
        self.vehicles.plate_error = None;
        self.apply(command, cx);
        true
    }

    pub(super) fn commit_plate(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_plate_text(cx) {
            self.vehicles.plate_error = Some(edit::FieldError::Name.key());
            cx.notify();
        }
    }

    fn apply_used_price_text(&mut self, cx: &mut Context<'_, Self>) -> Option<bool> {
        let Some((listing, price)) = self
            .selected_row()
            .and_then(|row| row.listing.as_ref())
            .map(|listing| (listing.id.clone(), listing.price))
        else {
            return Some(false);
        };
        let text = self.vehicles.used_price.read(cx).unmask_value().to_string();
        let currency = self.currency();
        let Ok(Command::SetUsedTruckPrice { price: next, .. }) =
            edit::used_price(&listing, &text, &currency)
        else {
            return None;
        };
        self.vehicles.used_price_error = None;
        if edit::same_shown_amount(price, next, &currency) {
            return Some(false);
        }
        self.vehicles.used_price_dirty = true;
        self.apply(
            Command::SetUsedTruckPrice {
                id: listing,
                price: next,
            },
            cx,
        );
        Some(true)
    }

    pub(super) fn commit_used_price(&mut self, cx: &mut Context<'_, Self>) {
        if self.apply_used_price_text(cx).is_none() {
            self.vehicles.used_price_error = Some(edit::FieldError::Number.key());
            cx.notify();
        }
    }

    fn apply_odometer_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, Some(current))) = self
            .selected_row()
            .map(|row| (row.id.clone(), row.odometer))
        else {
            return true;
        };
        let text = self.vehicles.odometer.read(cx).unmask_value().to_string();
        let Ok(command) = edit::odometer(&id, &text) else {
            return false;
        };
        self.vehicles.odometer_error = None;
        if matches!(command, Command::SetOdometer { km, .. } if km != current) {
            self.apply(command, cx);
        }
        true
    }

    pub(super) fn commit_odometer(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_odometer_text(cx) {
            self.vehicles.odometer_error = Some(edit::FieldError::Number.key());
            cx.notify();
        }
    }

    fn apply_fuel(&mut self, cx: &mut Context<'_, Self>) {
        let percent = self.vehicles.fuel.read(cx).value().end();
        let Some((id, current)) = self.selected_row().map(|row| (row.id.clone(), row.fuel)) else {
            return;
        };
        if (current * 100.0).round() != percent.round() {
            self.apply(edit::fuel(&id, percent), cx);
        }
    }

    fn vehicle_command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.apply(command, cx);
        self.sync_vehicle(window, cx);
    }

    pub(crate) fn render_vehicles(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let rows: Vec<&VehicleRow> = open
            .view()
            .vehicles
            .iter()
            .filter(|vehicle| vehicle.owned != self.vehicles.show_used_market)
            .collect();
        let selected = rows
            .iter()
            .find(|vehicle| Some(&vehicle.id) == self.vehicles.selected.as_ref())
            .copied();
        let mut list = div()
            .id("vehicle-list")
            .w(px(LIST_WIDTH))
            .flex_shrink_0()
            .h_full()
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_1))
            .p(px(tokens::SPACE_2))
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
                    .when(!self.vehicles.show_used_market, |this| {
                        this.child(
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
                    })
                    .child(div().flex_1())
                    .child(
                        ui::button(
                            "toggle-used-market",
                            t(if self.vehicles.show_used_market {
                                "vehicles.show_owned"
                            } else {
                                "vehicles.show_used"
                            }),
                        )
                        .tone(Tone::Quiet)
                        .selected(self.vehicles.show_used_market)
                        .on_click(
                            cx.listener(|this, _, window, cx| this.toggle_used_market(window, cx)),
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

    fn vehicle_row(
        &self,
        vehicle: &VehicleRow,
        first: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.vehicles.selected.as_ref() == Some(&vehicle.id);
        let name = self.vehicle_name(vehicle.model.as_deref());
        let mut label = format!("{name}, {}", vehicle.plate);
        if vehicle.current {
            label.push_str(&format!(", {}", t("vehicles.current")));
        }
        let id = vehicle.id.clone();
        let max_wear = vehicle
            .engine_wear
            .max(vehicle.transmission_wear)
            .max(vehicle.cabin_wear);
        BaseButton::new((
            ElementId::from("vehicle"),
            SharedString::from(vehicle.id.clone()),
        ))
        .when(first, |this| this.track_focus(&self.vehicles.list_focus))
        .w_full()
        .min_h(px(tokens::SAVE_ROW_HEIGHT))
        .px(px(tokens::SPACE_3))
        .py(px(tokens::SPACE_2))
        .rounded(px(tokens::RADIUS))
        .flex()
        .items_center()
        .gap(px(tokens::SPACE_2))
        .border_1()
        .border_color(if selected {
            palette.accent
        } else {
            transparent_black()
        })
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
                .gap(px(2.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(tokens::SPACE_2))
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
                        .when(vehicle.current, |this| {
                            this.child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(tokens::TEXT_LABEL))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(palette.accent)
                                    .child(t("vehicles.current")),
                            )
                        }),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(tokens::SPACE_2))
                        .text_size(px(tokens::TEXT_LABEL))
                        .text_color(palette.muted)
                        .child(div().truncate().child(vehicle.plate.clone()))
                        .child(
                            div()
                                .flex_shrink_0()
                                .flex()
                                .items_center()
                                .gap(px(5.0))
                                .child(
                                    div()
                                        .size(px(7.0))
                                        .rounded(px(999.0))
                                        .bg(tokens::wear_color(max_wear)),
                                )
                                .child(format::percent(vehicle.fuel)),
                        ),
                ),
        )
        .on_click(
            cx.listener(move |this, _, window, cx| this.select_vehicle(id.clone(), window, cx)),
        )
    }

    fn vehicle_detail(&self, vehicle: &VehicleRow, cx: &mut Context<'_, Self>) -> AnyElement {
        let palette = tokens::palette();
        let game = self.state.game();
        let hq_city = self
            .state
            .open()
            .map(|open| open.view().hq_city.clone())
            .unwrap_or_default();
        let is_hq = vehicle.garage.as_deref() == Some(hq_city.as_str()) && !hq_city.is_empty();
        let fuel = self.vehicles.fuel.read(cx).value().end().round();
        let live_plate = self.vehicles.plate.read(cx).value().to_string();
        let country = self.plate_country_menu(
            "plate",
            vehicle.id.clone(),
            vehicle.country.clone(),
            &self.vehicles.country_focus,
            cx,
        );
        let parts = (vehicle.listing.is_none() && self.has_parts(vehicle)).then(|| {
            ui::card()
                .flex()
                .flex_col()
                .gap(px(tokens::SPACE_3))
                .child(ui::section_title(t("vehicles.parts")))
                .child(div().flex().flex_wrap().gap(px(tokens::SPACE_4)).children(
                    PartKind::ALL.into_iter().enumerate().map(|(ix, kind)| {
                        div().w(px(PART_CELL)).child(ui::field(
                            t(kind.key()),
                            self.part_menu(kind, vehicle, &self.vehicles.parts_focus[ix], cx),
                            None,
                        ))
                    }),
                ))
        });
        let id = vehicle.id.clone();
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
                    .gap(px(tokens::SPACE_4))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap(px(tokens::SPACE_3))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_start()
                                    .gap(px(tokens::SPACE_1))
                                    .child(
                                        div()
                                            .text_size(px(tokens::TEXT_TITLE))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette.text)
                                            .child(self.vehicle_name(vehicle.model.as_deref())),
                                    )
                                    .when(vehicle.listing.is_none(), |this| {
                                        this.when_some(vehicle.garage.as_deref(), |this, garage| {
                                            this.child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap(px(tokens::SPACE_2))
                                                    .child(
                                                        div()
                                                            .flex()
                                                            .items_center()
                                                            .gap(px(tokens::SPACE_1))
                                                            .text_size(px(tokens::TEXT_LABEL))
                                                            .text_color(palette.muted)
                                                            .child(ui::icon(
                                                                IconName::MapPin,
                                                                12.0,
                                                                palette.muted,
                                                            ))
                                                            .child(self.city_name(garage)),
                                                    )
                                                    .when(is_hq, |this| {
                                                        this.child(ui::pill(
                                                            None,
                                                            t("vehicles.hq_badge"),
                                                            palette.accent,
                                                            palette.accent_soft,
                                                        ))
                                                    }),
                                            )
                                        })
                                    }),
                            )
                            .when(vehicle.listing.is_none(), |this| {
                                this.child(ui::plate_preview(
                                    &live_plate,
                                    vehicle.country.as_deref(),
                                    game,
                                ))
                            }),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_4))
                            .when(vehicle.listing.is_none(), |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap(px(tokens::SPACE_4))
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w(px(220.0))
                                                .flex()
                                                .flex_col()
                                                .gap(px(tokens::SPACE_1))
                                                .child(ui::field(
                                                    t("field.plate"),
                                                    ui::text_field(
                                                        "plate",
                                                        &self.vehicles.plate,
                                                        t("field.plate"),
                                                    ),
                                                    self.vehicles.plate_error.map(t),
                                                ))
                                                .child(
                                                    div()
                                                        .text_size(px(tokens::TEXT_LABEL))
                                                        .text_color(palette.muted)
                                                        .child(t("vehicles.plate_hint")),
                                                ),
                                        )
                                        .child(div().flex_1().min_w(px(220.0)).child(ui::field(
                                            self.plate_country_label(),
                                            country,
                                            None,
                                        ))),
                                )
                            })
                            .when(vehicle.listing.is_some(), |this| {
                                let price = self.currency();
                                let before = format!("{}{}", price.sign1, price.sign2);
                                let before = before.trim();
                                let after = price.sign3.trim();
                                this.child(ui::field(
                                    t("field.price"),
                                    ui::text_field(
                                        "used-price",
                                        &self.vehicles.used_price,
                                        t("field.price"),
                                    )
                                    .when(!before.is_empty(), |this| this.prefix(before.to_owned()))
                                    .when(!after.is_empty(), |this| this.suffix(after.to_owned())),
                                    self.vehicles.used_price_error.map(t),
                                ))
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(tokens::SPACE_4))
                                    .when(vehicle.odometer.is_some(), |this| {
                                        this.child(div().flex_1().min_w(px(220.0)).child(
                                            ui::field(
                                                t("field.odometer"),
                                                ui::text_field(
                                                    "odometer",
                                                    &self.vehicles.odometer,
                                                    t("field.odometer"),
                                                ),
                                                self.vehicles.odometer_error.map(t),
                                            ),
                                        ))
                                    })
                                    .child(
                                        div()
                                            .flex_1()
                                            .min_w(px(220.0))
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
                                                    .child(
                                                        div()
                                                            .text_color(palette.text)
                                                            .child(format::percent(fuel / 100.0)),
                                                    ),
                                            )
                                            .child(ui::slider(
                                                &self.vehicles.fuel,
                                                &self.vehicles.fuel_focus,
                                                t("field.fuel"),
                                            ))
                                            .child(
                                                div()
                                                    .flex()
                                                    .justify_between()
                                                    .px(px(tokens::SPACE_2))
                                                    .text_size(px(11.0))
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_color(palette.muted)
                                                    .child("E")
                                                    .child("½")
                                                    .child("F"),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_3))
                            .child(ui::section_title(t("vehicles.wear")))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(tokens::SPACE_3))
                                    .child(ui::wear_bar(t("wear.engine"), vehicle.engine_wear))
                                    .child(ui::wear_bar(
                                        t("wear.transmission"),
                                        vehicle.transmission_wear,
                                    ))
                                    .child(ui::wear_bar(t("wear.cabin"), vehicle.cabin_wear)),
                            )
                            .when(vehicle.listing.is_none(), |this| {
                                this.child(
                                    div()
                                        .pt(px(tokens::SPACE_2))
                                        .border_t_1()
                                        .border_color(palette.border)
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
                                        )
                                        .child(
                                            action(
                                                "vehicles.set_hq",
                                                IconName::MapPin,
                                                Command::SetHqVehicle,
                                            )
                                            .disabled(vehicle.garage.is_none() || is_hq),
                                        ),
                                )
                            }),
                    )
                    .when_some(parts, |this, parts| this.child(parts)),
            )
            .into_any_element()
    }

    fn has_parts(&self, vehicle: &VehicleRow) -> bool {
        vehicle
            .model
            .as_deref()
            .and_then(|token| self.model(token))
            .is_some_and(|model| !model.engines.is_empty())
    }

    fn part_menu(
        &self,
        part: PartKind,
        vehicle: &VehicleRow,
        focus: &FocusHandle,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement + use<> {
        let key = part.key();
        let current = part.current(vehicle).map(str::to_owned);
        let options = self.part_options(part, vehicle.model.as_deref());
        let label = current
            .as_deref()
            .map(|path| {
                options
                    .iter()
                    .find(|(option, _)| option == path)
                    .map(|(_, name)| name.clone())
                    .unwrap_or_else(|| part_label(path))
            })
            .unwrap_or_else(|| t("part.none"));
        let accessible = format!("{}: {label}", t(key));
        let id = vehicle.id.clone();
        let view = cx.entity().downgrade();
        ui::menu_popover(
            key,
            Anchor::BottomLeft,
            ui::select_field(format!("{key}-trigger"), label, accessible).track_focus(focus),
            options,
            current,
            move |path, _, cx| {
                view.update(cx, |view, cx| {
                    view.apply(part.command(id.clone(), path), cx)
                })
                .ok();
            },
        )
    }

    fn part_options(&self, part: PartKind, model: Option<&str>) -> Vec<(String, String)> {
        let Some(truck) = model.and_then(|token| self.model(token)) else {
            return Vec::new();
        };
        let language = i18n::current_locale();
        let mut options = Vec::new();
        let mut add = |path: &str, name: Option<&str>| {
            let name = name.map(str::to_owned).unwrap_or_else(|| part_label(path));
            options.push((path.to_owned(), name));
        };
        match part {
            PartKind::Engine => truck
                .engines
                .iter()
                .for_each(|part| add(&part.path, part.name(&language))),
            PartKind::Transmission => truck
                .transmissions
                .iter()
                .for_each(|part| add(&part.path, part.name(&language))),
            PartKind::Cabin => truck
                .cabins
                .iter()
                .for_each(|part| add(&part.path, part.name(&language))),
            PartKind::Chassis => truck
                .chassis
                .iter()
                .for_each(|part| add(&part.path, part.name(&language))),
        }
        disambiguate(&mut options);
        options
    }

    fn model(&self, token: &str) -> Option<&siir_save::catalog::VehicleModel> {
        self.catalog.as_ref()?.vehicle(token)
    }
}

fn part_label(path: &str) -> String {
    path.rsplit('/')
        .next()
        .unwrap_or(path)
        .trim_end_matches(".sii")
        .to_owned()
}

fn disambiguate(options: &mut [(String, String)]) {
    let repeated: Vec<bool> = options
        .iter()
        .map(|(_, name)| options.iter().filter(|(_, other)| other == name).count() > 1)
        .collect();
    for ((path, name), repeated) in options.iter_mut().zip(repeated) {
        if repeated {
            *name = format!("{name} ({})", part_label(path));
        }
    }
}

#[derive(Clone, Copy)]
enum PartKind {
    Engine,
    Transmission,
    Cabin,
    Chassis,
}

impl PartKind {
    const ALL: [Self; 4] = [Self::Engine, Self::Transmission, Self::Cabin, Self::Chassis];

    fn key(self) -> &'static str {
        match self {
            Self::Engine => "field.engine",
            Self::Transmission => "field.transmission",
            Self::Cabin => "field.cabin",
            Self::Chassis => "field.chassis",
        }
    }

    fn current(self, vehicle: &VehicleRow) -> Option<&str> {
        match self {
            Self::Engine => vehicle.engine.as_deref(),
            Self::Transmission => vehicle.transmission.as_deref(),
            Self::Cabin => vehicle.cabin.as_deref(),
            Self::Chassis => vehicle.chassis.as_deref(),
        }
    }

    fn command(self, id: String, path: String) -> Command {
        match self {
            Self::Engine => Command::SetEngine { id, path },
            Self::Transmission => Command::SetTransmission { id, path },
            Self::Cabin => Command::SetCabin { id, path },
            Self::Chassis => Command::SetChassis { id, path },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::disambiguate;

    #[test]
    fn adds_the_file_stem_only_to_repeated_names() {
        let mut options = vec![
            ("/def/cabin/l2h3.sii".to_owned(), "XXL".to_owned()),
            ("/def/cabin/l2h3_aero.sii".to_owned(), "XXL".to_owned()),
            ("/def/cabin/l2h1.sii".to_owned(), "Sleeper".to_owned()),
        ];
        disambiguate(&mut options);
        let names: Vec<&str> = options.iter().map(|(_, name)| name.as_str()).collect();
        assert_eq!(names, ["XXL (l2h3)", "XXL (l2h3_aero)", "Sleeper"]);
    }
}
