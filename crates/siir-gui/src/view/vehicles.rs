use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::base::Popover;
use gpui_kit::prelude::*;
use gpui_kit::{
    Anchor, AnyElement, Context, ElementId, FocusHandle, FontWeight, IntoElement, SharedString,
    div, px, transparent_black,
};

use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_gui::view_data::VehicleRow;
use siir_save::{Command, Game};

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

const LIST_WIDTH: f32 = 300.0;

const PART_CELL: f32 = (tokens::FORM_WIDTH - 2.0 * tokens::SPACE_6 - tokens::SPACE_4) / 2.0;

pub(super) const PART_COUNT: usize = PartKind::ALL.len();

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
            .filter(|vehicle| vehicle.owned != self.show_used_market)
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
                    .when(!self.show_used_market, |this| {
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
                            t(if self.show_used_market {
                                "vehicles.show_owned"
                            } else {
                                "vehicles.show_used"
                            }),
                        )
                        .tone(Tone::Quiet)
                        .selected(self.show_used_market)
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

    fn vehicle_detail(&self, vehicle: &VehicleRow, cx: &mut Context<'_, Self>) -> AnyElement {
        let palette = tokens::palette();
        let fuel = self.fuel.read(cx).value().end().round();
        let country = self.country_menu(cx);
        let parts = (vehicle.listing.is_none() && self.has_parts(vehicle)).then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(tokens::SPACE_2))
                .child(ui::section_title(t("vehicles.parts")))
                .child(div().flex().flex_wrap().gap(px(tokens::SPACE_4)).children(
                    PartKind::ALL.into_iter().enumerate().map(|(ix, kind)| {
                        div().w(px(PART_CELL)).child(ui::field(
                            t(kind.key()),
                            self.part_menu(kind, vehicle, &self.parts_focus[ix], cx),
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
                                    .child(if vehicle.listing.is_some() {
                                        t("vehicles.used_listing")
                                    } else {
                                        vehicle.plate.clone()
                                    }),
                            ),
                    )
                    .when(vehicle.listing.is_none(), |this| {
                        this.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(tokens::SPACE_1))
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap(px(tokens::SPACE_1))
                                        .child(ui::field(
                                            t("field.plate"),
                                            ui::text_field("plate", &self.plate, t("field.plate")),
                                            self.plate_error.map(t),
                                        ))
                                        .child(
                                            div()
                                                .text_size(px(tokens::TEXT_LABEL))
                                                .text_color(palette.muted)
                                                .child(t("vehicles.plate_hint")),
                                        ),
                                )
                                .child(ui::field(self.plate_country_label(), country, None)),
                        )
                    })
                    .when(vehicle.listing.is_some(), |this| {
                        let price = self.currency();
                        let before = format!("{}{}", price.sign1, price.sign2);
                        let before = before.trim();
                        let after = price.sign3.trim();
                        this.child(ui::field(
                            t("field.price"),
                            ui::text_field("used-price", &self.used_price, t("field.price"))
                                .when(!before.is_empty(), |this| this.prefix(before.to_owned()))
                                .when(!after.is_empty(), |this| this.suffix(after.to_owned())),
                            self.used_price_error.map(t),
                        ))
                    })
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
                    .when_some(parts, |this, parts| this.child(parts))
                    .when(vehicle.listing.is_none(), |this| {
                        this.child(
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
                        )
                    }),
            )
            .into_any_element()
    }

    fn country_menu(&self, cx: &mut Context<'_, Self>) -> impl IntoElement + use<> {
        let palette = tokens::palette();
        let language = i18n::current_locale().into_owned();
        let current = self.selected_row().and_then(|row| row.country.clone());
        let label = current
            .as_deref()
            .and_then(|token| self.catalog_country_name(token, &language))
            .or_else(|| current.clone())
            .unwrap_or_else(|| t("plate.country_none"));
        let accessible = format!("{}: {label}", self.plate_country_label());
        let mut countries: Vec<(String, String)> = self
            .catalog
            .as_ref()
            .map(|catalog| {
                catalog
                    .countries()
                    .iter()
                    .map(|country| {
                        let name = catalog
                            .country_name(&country.token, &language)
                            .unwrap_or(&country.token)
                            .to_owned();
                        (country.token.clone(), name)
                    })
                    .collect()
            })
            .unwrap_or_default();
        countries.sort_by(|left, right| left.1.cmp(&right.1));
        let view = cx.entity().downgrade();
        Popover::new("plate-country-menu")
            .anchor(Anchor::BottomLeft)
            .trigger(
                ui::select_field("plate-country-trigger", label, accessible)
                    .track_focus(&self.country_focus),
            )
            .content(move |_, _, cx| {
                let popover = cx.entity();
                let mut menu = div()
                    .id("plate-country-list")
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(tokens::SPACE_1))
                    .p(px(tokens::SPACE_1))
                    .max_h(px(260.0))
                    .overflow_y_scroll()
                    .on_scroll(|_, _, cx| cx.stop_propagation())
                    .bg(palette.panel)
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(tokens::RADIUS));
                for (token, name) in &countries {
                    let selected = current.as_deref() == Some(token.as_str());
                    let id = SharedString::from(format!("plate-country-option-{token}"));
                    let token = token.clone();
                    let view = view.clone();
                    let popover = popover.clone();
                    menu = menu.child(
                        ui::button(id, name.clone())
                            .tone(Tone::Quiet)
                            .selected(selected)
                            .on_click(move |_, window, cx| {
                                view.update(cx, |view, cx| view.set_plate_country(&token, cx))
                                    .ok();
                                popover.update(cx, |state, cx| state.dismiss(window, cx));
                            }),
                    );
                }
                menu.into_any_element()
            })
    }

    fn plate_country_label(&self) -> String {
        match self.state.game() {
            Game::Ets2 => t("field.plate_country"),
            Game::Ats => t("field.plate_state"),
        }
    }

    fn catalog_country_name(&self, token: &str, language: &str) -> Option<String> {
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.country_name(token, language))
            .map(str::to_owned)
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
        let palette = tokens::palette();
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
        Popover::new(format!("{key}-menu"))
            .anchor(Anchor::BottomLeft)
            .trigger(
                ui::select_field(format!("{key}-trigger"), label, accessible).track_focus(focus),
            )
            .content(move |_, _, cx| {
                let popover = cx.entity();
                let mut menu = div()
                    .id(format!("{key}-list"))
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(tokens::SPACE_1))
                    .p(px(tokens::SPACE_1))
                    .max_h(px(260.0))
                    .overflow_y_scroll()
                    .on_scroll(|_, _, cx| cx.stop_propagation())
                    .bg(palette.panel)
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(tokens::RADIUS));
                for (path, name) in &options {
                    let selected = current.as_deref() == Some(path.as_str());
                    let path = path.clone();
                    let id = id.clone();
                    let view = view.clone();
                    let popover = popover.clone();
                    menu = menu.child(
                        ui::button(format!("{key}-option-{path}"), name.clone())
                            .tone(Tone::Quiet)
                            .selected(selected)
                            .on_click(move |_, window, cx| {
                                if !selected {
                                    view.update(cx, |view, cx| {
                                        view.apply(
                                            part.command(id.clone(), path.clone()),
                                            |_| {},
                                            cx,
                                        )
                                    })
                                    .ok();
                                }
                                popover.update(cx, |state, cx| state.dismiss(window, cx));
                            }),
                    );
                }
                menu.into_any_element()
            })
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
