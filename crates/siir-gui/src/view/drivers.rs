use std::ops::Range;

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{
    Anchor, AnyElement, Context, Div, ElementId, FocusHandle, FontWeight, IntoElement,
    ScrollStrategy, SharedString, UniformListScrollHandle, div, px, transparent_black,
    uniform_list,
};

use siir_gui::edit::{self, ADR_CLASSES, ADR_CODES};
use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_gui::view_data::{DriverRow, DriverStatus, GarageRow, SaveView};
use siir_save::Command;

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

const LIST_WIDTH: f32 = 290.0;
const ROW_HEIGHT: f32 = 54.0;
const SKILL_CELL: f32 = 210.0;
const ADR_CELL: f32 = 210.0;

const SKILLS: [(&str, &str); 5] = [
    ("driver-skill-long-dist", "skill.long_dist"),
    ("driver-skill-heavy", "skill.heavy"),
    ("driver-skill-fragile", "skill.fragile"),
    ("driver-skill-urgent", "skill.urgent"),
    ("driver-skill-mechanical", "skill.mechanical"),
];

pub(super) struct DriversState {
    pub(super) selected: Option<String>,
    pub(super) show_available: bool,
    pub(super) hire_garage: Option<String>,
    scroll: UniformListScrollHandle,
    skill_focus: [FocusHandle; 5],
    truck_focus: FocusHandle,
    hire_garage_focus: FocusHandle,
    pub(super) list_focus: FocusHandle,
}

impl DriversState {
    pub(super) fn new(cx: &mut Context<'_, AppView>) -> Self {
        Self {
            selected: None,
            show_available: false,
            hire_garage: None,
            scroll: UniformListScrollHandle::new(),
            skill_focus: std::array::from_fn(|_| cx.focus_handle().tab_stop(true)),
            truck_focus: cx.focus_handle().tab_stop(true),
            hire_garage_focus: cx.focus_handle().tab_stop(true),
            list_focus: cx.focus_handle().tab_stop(true),
        }
    }

    pub(super) fn reset(&mut self) {
        self.selected = None;
        self.show_available = false;
        self.hire_garage = None;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
    }
}

impl AppView {
    pub(super) fn default_driver(&self) -> Option<String> {
        let view = self.state.open()?.view();
        let show_available = self.drivers.show_available;
        view.drivers
            .iter()
            .find(|driver| matches_driver_filter(driver, show_available))
            .map(|driver| driver.id.clone())
    }

    pub(super) fn default_hire_garage(&self) -> Option<String> {
        let view = self.state.open()?.view();
        view.garages
            .iter()
            .find(|garage| garage.free_driver_slots() > 0)
            .map(|garage| garage.id.clone())
    }

    pub(super) fn sync_drivers(&mut self) {
        let keep_selected = self
            .drivers
            .selected
            .as_ref()
            .zip(self.state.open())
            .is_some_and(|(id, open)| {
                open.view().drivers.iter().any(|driver| {
                    &driver.id == id && matches_driver_filter(driver, self.drivers.show_available)
                })
            });
        if !keep_selected {
            self.drivers.selected = self.default_driver();
        }
        let keep_garage = self
            .drivers
            .hire_garage
            .as_ref()
            .zip(self.state.open())
            .is_some_and(|(id, open)| {
                open.view()
                    .garages
                    .iter()
                    .any(|garage| &garage.id == id && garage.free_driver_slots() > 0)
            });
        if !keep_garage {
            self.drivers.hire_garage = self.default_hire_garage();
        }
    }

    fn select_driver(&mut self, id: String, cx: &mut Context<'_, Self>) {
        self.drivers.selected = Some(id);
        cx.notify();
    }

    fn toggle_available_drivers(&mut self, cx: &mut Context<'_, Self>) {
        self.drivers.show_available = !self.drivers.show_available;
        self.drivers.selected = self.default_driver();
        self.drivers.scroll.scroll_to_item(0, ScrollStrategy::Top);
        cx.notify();
    }

    fn update_driver_skills(
        &mut self,
        id: String,
        mutate: impl FnOnce(&mut [u64; 5], &mut [bool; 6]),
        cx: &mut Context<'_, Self>,
    ) {
        let Some(current) = self
            .state
            .open()
            .and_then(|open| open.view().drivers.iter().find(|row| row.id == id))
            .map(|row| row.skills)
        else {
            return;
        };
        let mut levels = [
            current.long_dist,
            current.heavy,
            current.fragile,
            current.urgent,
            current.mechanical,
        ];
        let mut classes = std::array::from_fn(|bit| current.adr & (1u64 << bit) != 0);
        mutate(&mut levels, &mut classes);
        let skills = edit::skills(levels, classes, current.adr);
        if skills != current {
            self.apply(Command::SetDriverSkills { id, skills }, cx);
        }
    }

    fn hire_driver(&mut self, id: String, garage: String, cx: &mut Context<'_, Self>) {
        self.apply(
            Command::HireDriver {
                id: id.clone(),
                garage,
            },
            cx,
        );
        if self.action_error.is_none() {
            self.drivers.show_available = false;
            self.drivers.selected = Some(id);
            self.sync_drivers();
            cx.notify();
        }
    }

    fn fire_driver(&mut self, id: String, cx: &mut Context<'_, Self>) {
        self.apply(Command::FireDriver(id), cx);
        if self.action_error.is_none() {
            self.sync_drivers();
            cx.notify();
        }
    }

    fn set_driver_truck(&mut self, id: String, truck: String, cx: &mut Context<'_, Self>) {
        self.apply(Command::SetDriverTruck { id, truck }, cx);
    }

    pub(crate) fn render_drivers(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let view = open.view();
        let show_available = self.drivers.show_available;
        let row_indices: Vec<usize> = view
            .drivers
            .iter()
            .enumerate()
            .filter(|(_, driver)| matches_driver_filter(driver, show_available))
            .map(|(ix, _)| ix)
            .collect();
        let selected = row_indices
            .iter()
            .map(|&ix| &view.drivers[ix])
            .find(|driver| Some(&driver.id) == self.drivers.selected.as_ref());

        let count = row_indices.len();
        let list = div()
            .w(px(LIST_WIDTH))
            .flex_shrink_0()
            .h_full()
            .flex()
            .flex_col()
            .p(px(tokens::SPACE_2))
            .border_r_1()
            .border_color(palette.border)
            .child(
                uniform_list(
                    "driver-list",
                    count,
                    cx.processor(move |this, range: Range<usize>, _window, cx| {
                        let Some(open) = this.state.open() else {
                            return Vec::new();
                        };
                        let drivers = &open.view().drivers;
                        range
                            .filter_map(|pos| {
                                let driver = drivers.get(*row_indices.get(pos)?)?;
                                Some(
                                    div()
                                        .w_full()
                                        .pb(px(tokens::SPACE_1))
                                        .child(this.driver_row(driver, pos == 0, cx)),
                                )
                            })
                            .collect()
                    }),
                )
                .track_scroll(&self.drivers.scroll)
                .flex_1()
                .w_full()
                .h_full(),
            );

        let detail = match selected {
            Some(driver) => self.driver_detail(driver, view, cx).into_any_element(),
            None => {
                let (title, hint) = if show_available {
                    (
                        t("empty.no_available_drivers"),
                        t("empty.no_available_drivers_hint"),
                    )
                } else {
                    (
                        t("empty.no_hired_drivers"),
                        t("empty.no_hired_drivers_hint"),
                    )
                };
                div()
                    .flex_1()
                    .p(px(tokens::SPACE_6))
                    .child(ui::empty_state(IconName::Users, title, hint))
                    .into_any_element()
            }
        };

        let toggle_label = if show_available {
            t("drivers.show_hired")
        } else {
            t("drivers.show_available")
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
                    .child(div().flex_1())
                    .child(
                        ui::button("toggle-available-drivers", toggle_label)
                            .tone(Tone::Quiet)
                            .selected(show_available)
                            .toggled(show_available)
                            .on_click(
                                cx.listener(|this, _, _, cx| this.toggle_available_drivers(cx)),
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

    fn driver_row(
        &self,
        driver: &DriverRow,
        first: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.drivers.selected.as_ref() == Some(&driver.id);
        let name = self.driver_name(&driver.id);
        let status_label = t(i18n::driver_status(driver.status));
        let subtitle = driver
            .garage_city
            .as_deref()
            .map(|city| self.city_name(city))
            .or_else(|| (!driver.hometown.is_empty()).then(|| self.city_name(&driver.hometown)))
            .unwrap_or_else(|| status_label.clone());
        let accessible = format!("{name}, {status_label}");
        let id = driver.id.clone();

        BaseButton::new((
            ElementId::from("driver"),
            SharedString::from(driver.id.clone()),
        ))
        .when(first, |this| this.track_focus(&self.drivers.list_focus))
        .w_full()
        .h(px(ROW_HEIGHT))
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
        .accessibility_label(accessible)
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
                        .when(driver.on_job, |this| {
                            this.child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(tokens::TEXT_LABEL))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(palette.accent)
                                    .child(t("drivers.on_job")),
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
                        .child(div().truncate().child(subtitle))
                        .when(driver.status == DriverStatus::Dismissed, |this| {
                            this.child(
                                div()
                                    .flex_shrink_0()
                                    .text_color(palette.danger_text)
                                    .child(status_label),
                            )
                        }),
                ),
        )
        .on_click(cx.listener(move |this, _, _, cx| this.select_driver(id.clone(), cx)))
    }

    fn driver_detail(
        &self,
        driver: &DriverRow,
        view: &SaveView,
        cx: &mut Context<'_, Self>,
    ) -> AnyElement {
        let palette = tokens::palette();
        let game = self.state.game();
        let (status_fg, status_bg) = match driver.status {
            DriverStatus::Employed => (palette.success, palette.success_soft),
            DriverStatus::Dismissed => (palette.danger_text, palette.surface),
            DriverStatus::Available => (palette.accent, palette.accent_soft),
            DriverStatus::Other => (palette.muted, palette.surface),
        };
        let xp_label = match self.levels() {
            Some(levels) => format!(
                "{} {} · {} XP",
                t("field.level"),
                format::number(u64::from(levels.level(driver.experience_points)), game),
                format::number(driver.experience_points, game),
            ),
            None => format!("{} XP", format::number(driver.experience_points, game)),
        };

        let skills_card = ui::card()
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_4))
            .child(ui::section_title(t("discovery.skills")))
            .child(self.driver_skill_grid(driver, cx))
            .child(
                div()
                    .pt(px(tokens::SPACE_3))
                    .border_t_1()
                    .border_color(palette.border)
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_2))
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_LABEL))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.muted)
                            .child(t("adr.classes")),
                    )
                    .child(self.driver_adr_grid(driver, cx)),
            );

        let action_card = match driver.status {
            DriverStatus::Employed => Some(self.employed_driver_card(driver, view, cx)),
            DriverStatus::Available | DriverStatus::Dismissed => {
                Some(self.hire_driver_card(driver, view, cx))
            }
            DriverStatus::Other => None,
        };

        div()
            .id("driver-detail")
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
                                    .gap(px(tokens::SPACE_1))
                                    .child(
                                        div()
                                            .text_size(px(tokens::TEXT_TITLE))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette.text)
                                            .child(self.driver_name(&driver.id)),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .flex_wrap()
                                            .items_center()
                                            .gap(px(tokens::SPACE_2))
                                            .text_size(px(tokens::TEXT_LABEL))
                                            .text_color(palette.muted)
                                            .child(xp_label)
                                            .when(!driver.hometown.is_empty(), |this| {
                                                this.child(format!(
                                                    "· {}: {}",
                                                    t("field.hometown"),
                                                    self.city_name(&driver.hometown)
                                                ))
                                            }),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(tokens::SPACE_2))
                                    .child(ui::pill(
                                        None,
                                        t(i18n::driver_status(driver.status)),
                                        status_fg,
                                        status_bg,
                                    ))
                                    .when(driver.on_job, |this| {
                                        this.child(ui::pill(
                                            Some(IconName::Truck),
                                            t("drivers.on_job"),
                                            palette.accent,
                                            palette.accent_soft,
                                        ))
                                    }),
                            ),
                    )
                    .child(skills_card)
                    .when_some(action_card, |this, card| this.child(card)),
            )
            .into_any_element()
    }

    fn driver_skill_grid(&self, driver: &DriverRow, cx: &mut Context<'_, Self>) -> Div {
        let palette = tokens::palette();
        let levels = [
            driver.skills.long_dist,
            driver.skills.heavy,
            driver.skills.fragile,
            driver.skills.urgent,
            driver.skills.mechanical,
        ];
        let mut cells: Vec<Div> = Vec::with_capacity(SKILLS.len());
        for (ix, (id, key)) in SKILLS.iter().enumerate() {
            let value = levels[ix];
            let driver_id = driver.id.clone();
            let view = cx.entity().downgrade();
            cells.push(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .w(px(SKILL_CELL))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(
                                div()
                                    .text_size(px(tokens::TEXT_BODY))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(palette.text)
                                    .child(t(key)),
                            )
                            .child(
                                div()
                                    .text_size(px(tokens::TEXT_LABEL))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(if value > 0 {
                                        palette.accent
                                    } else {
                                        palette.muted
                                    })
                                    .child(format!("{value} / 6")),
                            ),
                    )
                    .child(ui::skill_segments(
                        id,
                        t(key),
                        value,
                        &self.drivers.skill_focus[ix],
                        move |level, _, cx| {
                            let driver_id = driver_id.clone();
                            view.update(cx, |view, cx| {
                                view.update_driver_skills(
                                    driver_id,
                                    |levels, _| levels[ix] = level,
                                    cx,
                                )
                            })
                            .ok();
                        },
                    )),
            );
        }
        div()
            .flex()
            .flex_wrap()
            .gap(px(tokens::SPACE_4))
            .children(cells)
    }

    fn driver_adr_grid(&self, driver: &DriverRow, cx: &mut Context<'_, Self>) -> Div {
        let mut cells: Vec<Div> = Vec::with_capacity(ADR_CLASSES.len());
        for (ix, key) in ADR_CLASSES.iter().enumerate() {
            let checked = driver.skills.adr & (1u64 << ix) != 0;
            let driver_id = driver.id.clone();
            cells.push(
                div().w(px(ADR_CELL)).child(
                    ui::checkbox(("driver-adr-class", ix), t(key), checked)
                        .badge(ADR_CODES[ix])
                        .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                            this.update_driver_skills(
                                driver_id.clone(),
                                |_, classes| classes[ix] = *checked,
                                cx,
                            )
                        })),
                ),
            );
        }
        div()
            .flex()
            .flex_wrap()
            .gap(px(tokens::SPACE_2))
            .children(cells)
    }

    fn employed_driver_card(
        &self,
        driver: &DriverRow,
        view: &SaveView,
        cx: &mut Context<'_, Self>,
    ) -> AnyElement {
        let palette = tokens::palette();
        let garage = driver
            .garage_id
            .as_deref()
            .and_then(|id| view.garages.iter().find(|garage| garage.id == id));
        let garage_trucks: Vec<(String, String)> = garage
            .map(|garage| {
                garage
                    .vehicles
                    .iter()
                    .flatten()
                    .map(|vehicle_id| {
                        let label = self.vehicle_label(view, vehicle_id);
                        (vehicle_id.clone(), label)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let truck_control = if garage_trucks.is_empty() {
            div()
                .text_size(px(tokens::TEXT_LABEL))
                .text_color(palette.muted)
                .child(t("drivers.no_garage_trucks"))
                .into_any_element()
        } else {
            self.driver_truck_menu(driver, garage_trucks, view, cx)
                .into_any_element()
        };

        let fire_id = driver.id.clone();
        ui::card()
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_3))
            .child(ui::section_title(t("drivers.assignment")))
            .when_some(driver.garage_city.as_deref(), |this, city| {
                this.child(ui::field(
                    t("field.garage"),
                    div()
                        .text_size(px(tokens::TEXT_BODY))
                        .text_color(palette.text)
                        .child(self.city_name(city)),
                    None,
                ))
            })
            .child(ui::field(t("field.truck"), truck_control, None))
            .child(
                div()
                    .pt(px(tokens::SPACE_3))
                    .border_t_1()
                    .border_color(palette.border)
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(tokens::SPACE_2))
                    .when(driver.on_job, |this| {
                        this.child(
                            div()
                                .text_size(px(tokens::TEXT_LABEL))
                                .text_color(palette.muted)
                                .child(t("drivers.fire_on_job_hint")),
                        )
                    })
                    .child(
                        ui::button("fire-driver", t("drivers.fire"))
                            .tone(Tone::Danger)
                            .icon(IconName::UserMinus)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.fire_driver(fire_id.clone(), cx)
                            })),
                    ),
            )
            .into_any_element()
    }

    fn driver_truck_menu(
        &self,
        driver: &DriverRow,
        options: Vec<(String, String)>,
        view: &SaveView,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement + use<> {
        let current = driver.truck.clone();
        let label = current
            .as_deref()
            .map(|id| self.vehicle_label(view, id))
            .unwrap_or_else(|| t("drivers.truck_none"));
        let accessible = format!("{}: {label}", t("field.truck"));
        let driver_id = driver.id.clone();
        let entity = cx.entity().downgrade();
        ui::menu_popover(
            "driver-truck",
            Anchor::BottomLeft,
            ui::select_field("driver-truck-trigger", label, accessible)
                .track_focus(&self.drivers.truck_focus),
            options,
            current,
            move |truck_id, _, cx| {
                entity
                    .update(cx, |view, cx| {
                        view.set_driver_truck(driver_id.clone(), truck_id, cx)
                    })
                    .ok();
            },
        )
    }

    fn hire_driver_card(
        &self,
        driver: &DriverRow,
        view: &SaveView,
        cx: &mut Context<'_, Self>,
    ) -> AnyElement {
        let palette = tokens::palette();
        let free_garages: Vec<&GarageRow> = view
            .garages
            .iter()
            .filter(|garage| garage.free_driver_slots() > 0)
            .collect();
        let target = self
            .drivers
            .hire_garage
            .as_deref()
            .filter(|id| free_garages.iter().any(|garage| garage.id == *id))
            .map(str::to_owned)
            .or_else(|| free_garages.first().map(|garage| garage.id.clone()));

        if free_garages.is_empty() {
            return ui::card()
                .flex()
                .flex_col()
                .items_start()
                .gap(px(tokens::SPACE_3))
                .child(ui::section_title(t("drivers.hire_section")))
                .child(
                    div()
                        .text_size(px(tokens::TEXT_LABEL))
                        .text_color(palette.muted)
                        .child(t("drivers.no_free_garages")),
                )
                .child(
                    ui::button("hire-driver", t("drivers.hire"))
                        .icon(IconName::UserPlus)
                        .disabled(true),
                )
                .into_any_element();
        }

        let options: Vec<(String, String)> = free_garages
            .iter()
            .map(|garage| {
                let used = garage.drivers.len() - garage.free_driver_slots();
                let label = format!(
                    "{} ({used} / {})",
                    self.city_name(&garage.city),
                    garage.drivers.len()
                );
                (garage.id.clone(), label)
            })
            .collect();
        let selected_label = target
            .as_deref()
            .and_then(|id| {
                options
                    .iter()
                    .find(|(garage_id, _)| garage_id == id)
                    .map(|(_, label)| label.clone())
            })
            .unwrap_or_else(|| t("drivers.select_garage"));
        let accessible = format!("{}: {selected_label}", t("field.garage"));
        let entity = cx.entity().downgrade();
        let garage_menu = ui::menu_popover(
            "hire-garage",
            Anchor::BottomLeft,
            ui::select_field("hire-garage-trigger", selected_label, accessible)
                .track_focus(&self.drivers.hire_garage_focus),
            options,
            target.clone(),
            move |garage_id, _, cx| {
                entity
                    .update(cx, |view, cx| {
                        view.drivers.hire_garage = Some(garage_id);
                        cx.notify();
                    })
                    .ok();
            },
        );

        let driver_id = driver.id.clone();
        let hire_garage = target.clone();
        ui::card()
            .flex()
            .flex_col()
            .items_start()
            .gap(px(tokens::SPACE_3))
            .child(ui::section_title(t("drivers.hire_section")))
            .child(
                div()
                    .w(px(320.0))
                    .child(ui::field(t("field.garage"), garage_menu, None)),
            )
            .child(
                ui::button("hire-driver", t("drivers.hire"))
                    .icon(IconName::UserPlus)
                    .disabled(hire_garage.is_none())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if let Some(garage) = hire_garage.clone() {
                            this.hire_driver(driver_id.clone(), garage, cx);
                        }
                    })),
            )
            .into_any_element()
    }

    pub(super) fn driver_name(&self, id: &str) -> String {
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.driver_name(id))
            .unwrap_or(id)
            .to_owned()
    }

    pub(super) fn vehicle_label(&self, view: &SaveView, id: &str) -> String {
        let Some(vehicle) = view.vehicles.iter().find(|row| row.id == id) else {
            return id.to_owned();
        };
        let name = self.vehicle_name(vehicle.model.as_deref());
        if vehicle.plate.is_empty() {
            name
        } else {
            format!("{name} · {}", vehicle.plate)
        }
    }
}

fn matches_driver_filter(driver: &DriverRow, show_available: bool) -> bool {
    if show_available {
        driver.status.can_hire()
    } else {
        driver.status == DriverStatus::Employed
    }
}
