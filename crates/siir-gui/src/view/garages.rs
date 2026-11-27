use std::ops::Range;

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::base::input::InputState;
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, Context, Div, ElementId, Entity, FocusHandle, FontWeight, IntoElement,
    ScrollStrategy, SharedString, Subscription, UniformListScrollHandle, Window, div, px,
    transparent_black, uniform_list,
};

use siir_gui::i18n::{self, t};
use siir_gui::view_data::{GarageRow, SaveView};
use siir_save::Command;

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

const LIST_WIDTH: f32 = 290.0;
const ROW_HEIGHT: f32 = 54.0;
const VALID_STATUSES: [u64; 4] = [1, 6, 2, 3];

pub(super) struct GaragesState {
    pub(super) selected: Option<String>,
    pub(super) show_all: bool,
    pub(super) search: Entity<InputState>,
    scroll: UniformListScrollHandle,
    pub(super) list_focus: FocusHandle,
}

impl GaragesState {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<'_, AppView>,
        subscriptions: &mut Vec<Subscription>,
    ) -> Self {
        let search = cx.new(|cx| InputState::new(window, cx).placeholder(t("garages.search")));
        subscriptions.push(cx.observe(&search, |this, _, cx| {
            this.sync_garages(cx);
            cx.notify();
        }));
        Self {
            selected: None,
            show_all: false,
            search,
            scroll: UniformListScrollHandle::new(),
            list_focus: cx.focus_handle().tab_stop(true),
        }
    }

    pub(super) fn reset(&mut self, window: &mut Window, cx: &mut Context<'_, AppView>) {
        self.selected = None;
        self.show_all = false;
        self.scroll.scroll_to_item(0, ScrollStrategy::Top);
        self.search
            .update(cx, |input, cx| input.set_value("", window, cx));
    }
}

impl AppView {
    pub(super) fn default_garage(&self, cx: &Context<'_, Self>) -> Option<String> {
        let view = self.state.open()?.view();
        let query = self.garages.search.read(cx).value().trim().to_lowercase();
        let show_all = self.garages.show_all;
        view.garages
            .iter()
            .find(|garage| self.matches_garage(garage, show_all, &query))
            .map(|garage| garage.id.clone())
    }

    pub(super) fn sync_garages(&mut self, cx: &Context<'_, Self>) {
        let query = self.garages.search.read(cx).value().trim().to_lowercase();
        let show_all = self.garages.show_all;
        let keep =
            self.garages
                .selected
                .as_ref()
                .zip(self.state.open())
                .is_some_and(|(id, open)| {
                    open.view().garages.iter().any(|garage| {
                        &garage.id == id && self.matches_garage(garage, show_all, &query)
                    })
                });
        if !keep {
            self.garages.selected = self.default_garage(cx);
        }
    }

    fn matches_garage(&self, garage: &GarageRow, show_all: bool, query: &str) -> bool {
        if !show_all && garage.status == 0 {
            return false;
        }
        if query.is_empty() {
            return true;
        }
        let localized = self.city_name(&garage.city).to_lowercase();
        localized.contains(query) || garage.city.to_lowercase().contains(query)
    }

    fn select_garage(&mut self, id: String, cx: &mut Context<'_, Self>) {
        self.garages.selected = Some(id);
        cx.notify();
    }

    fn toggle_all_garages(&mut self, cx: &mut Context<'_, Self>) {
        self.garages.show_all = !self.garages.show_all;
        self.garages.scroll.scroll_to_item(0, ScrollStrategy::Top);
        self.sync_garages(cx);
        cx.notify();
    }

    fn set_garage_status(&mut self, id: String, status: u64, cx: &mut Context<'_, Self>) {
        self.apply(Command::SetGarageStatus { id, status }, cx);
        if self.action_error.is_none() {
            self.sync_drivers();
            self.sync_garages(cx);
            cx.notify();
        }
    }

    pub(crate) fn render_garages(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let view = open.view();
        let query = self.garages.search.read(cx).value().trim().to_lowercase();
        let show_all = self.garages.show_all;
        let row_indices: Vec<usize> = view
            .garages
            .iter()
            .enumerate()
            .filter(|(_, garage)| self.matches_garage(garage, show_all, &query))
            .map(|(ix, _)| ix)
            .collect();
        let selected = row_indices
            .iter()
            .map(|&ix| &view.garages[ix])
            .find(|garage| Some(&garage.id) == self.garages.selected.as_ref());

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
                    "garage-list",
                    count,
                    cx.processor(move |this, range: Range<usize>, _window, cx| {
                        let Some(open) = this.state.open() else {
                            return Vec::new();
                        };
                        let view = open.view();
                        range
                            .filter_map(|pos| {
                                let garage = view.garages.get(*row_indices.get(pos)?)?;
                                Some(
                                    div()
                                        .w_full()
                                        .pb(px(tokens::SPACE_1))
                                        .child(this.garage_row(garage, view, pos == 0, cx)),
                                )
                            })
                            .collect()
                    }),
                )
                .track_scroll(&self.garages.scroll)
                .flex_1()
                .w_full()
                .h_full(),
            );

        let detail = match selected {
            Some(garage) => self.garage_detail(garage, view, cx).into_any_element(),
            None => div()
                .flex_1()
                .p(px(tokens::SPACE_6))
                .child(ui::empty_state(
                    IconName::Warehouse,
                    t("empty.no_garages"),
                    t("empty.no_garages_hint"),
                ))
                .into_any_element(),
        };

        let toggle_label = if show_all {
            t("garages.show_active")
        } else {
            t("garages.show_all")
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
                    .child(div().w(px(240.0)).child(ui::text_field(
                        "garage-search",
                        &self.garages.search,
                        t("garages.search"),
                    )))
                    .child(div().flex_1())
                    .child(
                        ui::button("toggle-all-garages", toggle_label)
                            .tone(Tone::Quiet)
                            .selected(show_all)
                            .toggled(show_all)
                            .on_click(cx.listener(|this, _, _, cx| this.toggle_all_garages(cx))),
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

    fn garage_row(
        &self,
        garage: &GarageRow,
        view: &SaveView,
        first: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.garages.selected.as_ref() == Some(&garage.id);
        let city = self.city_name(&garage.city);
        let status_label = t(i18n::garage_status(garage.status));
        let is_hq = !garage.city.is_empty() && garage.city == view.hq_city;
        let used_drivers = garage.drivers.iter().filter(|slot| slot.is_some()).count();
        let total_slots = garage.drivers.len();
        let accessible = format!("{city}, {status_label}");
        let id = garage.id.clone();

        BaseButton::new((
            ElementId::from("garage"),
            SharedString::from(garage.id.clone()),
        ))
        .when(first, |this| this.track_focus(&self.garages.list_focus))
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
                                .child(city),
                        )
                        .when(is_hq, |this| {
                            this.child(
                                div()
                                    .flex_shrink_0()
                                    .text_size(px(tokens::TEXT_LABEL))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(palette.accent)
                                    .child(t("vehicles.hq_badge")),
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
                        .child(div().truncate().child(status_label))
                        .when(total_slots > 0, |this| {
                            this.child(
                                div()
                                    .flex_shrink_0()
                                    .child(format!("{used_drivers} / {total_slots}")),
                            )
                        }),
                ),
        )
        .on_click(cx.listener(move |this, _, _, cx| this.select_garage(id.clone(), cx)))
    }

    fn garage_detail(
        &self,
        garage: &GarageRow,
        view: &SaveView,
        cx: &mut Context<'_, Self>,
    ) -> AnyElement {
        let palette = tokens::palette();
        let city = self.city_name(&garage.city);
        let is_hq = !garage.city.is_empty() && garage.city == view.hq_city;
        let occupied = garage.occupied();
        let (status_fg, status_bg) = match garage.status {
            2 | 3 | 6 => (palette.success, palette.success_soft),
            1 => (palette.accent, palette.accent_soft),
            _ => (palette.muted, palette.surface),
        };

        let mut status_buttons = div().flex().flex_wrap().gap(px(tokens::SPACE_2));
        for status in VALID_STATUSES {
            let active = garage.status == status;
            let id = garage.id.clone();
            status_buttons = status_buttons.child(
                ui::button(
                    SharedString::from(format!("garage-status-{status}")),
                    t(i18n::garage_status(status)),
                )
                .selected(active)
                .toggled(active)
                .disabled(occupied)
                .on_click(cx.listener(move |this, _, _, cx| {
                    if !active {
                        this.set_garage_status(id.clone(), status, cx);
                    }
                })),
            );
        }

        let vehicle_slots: Vec<Option<String>> = garage
            .vehicles
            .iter()
            .map(|slot| slot.as_deref().map(|id| self.vehicle_label(view, id)))
            .collect();
        let driver_slots: Vec<Option<String>> = garage
            .drivers
            .iter()
            .map(|slot| {
                slot.as_deref().map(|id| {
                    if id.starts_with("driver_player.") {
                        t("garages.player_driver")
                    } else {
                        self.driver_name(id)
                    }
                })
            })
            .collect();
        let trailer_slots: Vec<Option<String>> = garage
            .trailers
            .iter()
            .map(|slot| {
                slot.as_deref().map(|id| {
                    view.trailers
                        .iter()
                        .find(|trailer| trailer.id == id)
                        .map(|trailer| {
                            if trailer.plate.is_empty() {
                                trailer.definition.clone()
                            } else {
                                trailer.plate.clone()
                            }
                        })
                        .unwrap_or_else(|| id.to_owned())
                })
            })
            .collect();

        div()
            .id("garage-detail")
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
                                            .child(city),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(tokens::TEXT_LABEL))
                                            .text_color(palette.muted)
                                            .child(garage.id.clone()),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(tokens::SPACE_2))
                                    .when(is_hq, |this| {
                                        this.child(ui::pill(
                                            Some(IconName::MapPin),
                                            t("vehicles.hq_badge"),
                                            palette.accent,
                                            palette.accent_soft,
                                        ))
                                    })
                                    .child(ui::pill(
                                        None,
                                        t(i18n::garage_status(garage.status)),
                                        status_fg,
                                        status_bg,
                                    )),
                            ),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_3))
                            .child(ui::section_title(t("garages.status")))
                            .child(status_buttons)
                            .when(occupied, |this| {
                                this.child(
                                    div()
                                        .text_size(px(tokens::TEXT_LABEL))
                                        .text_color(palette.muted)
                                        .child(t("garages.occupied_hint")),
                                )
                            }),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_4))
                            .child(ui::section_title(t("garages.slots")))
                            .child(slot_group(t("field.vehicles"), &vehicle_slots))
                            .child(slot_group(t("field.drivers"), &driver_slots))
                            .child(slot_group(t("field.trailers"), &trailer_slots)),
                    ),
            )
            .into_any_element()
    }
}

fn slot_group(label: String, slots: &[Option<String>]) -> Div {
    let palette = tokens::palette();
    let used = slots.iter().filter(|slot| slot.is_some()).count();
    let mut list = div().flex().flex_wrap().gap(px(tokens::SPACE_2));
    for (ix, slot) in slots.iter().enumerate() {
        let (text, fg) = match slot {
            Some(name) => (format!("#{} · {name}", ix + 1), palette.text),
            None => (
                format!("#{} · {}", ix + 1, t("garages.slot_empty")),
                palette.muted,
            ),
        };
        list = list.child(
            div()
                .px(px(tokens::SPACE_3))
                .py(px(6.0))
                .rounded(px(tokens::RADIUS))
                .bg(palette.surface)
                .border_1()
                .border_color(palette.border)
                .text_size(px(tokens::TEXT_LABEL))
                .text_color(fg)
                .child(text),
        );
    }
    div()
        .flex()
        .flex_col()
        .gap(px(tokens::SPACE_2))
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
                        .child(label),
                )
                .child(
                    div()
                        .text_size(px(tokens::TEXT_LABEL))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(palette.muted)
                        .child(format!("{used} / {}", slots.len())),
                ),
        )
        .when(!slots.is_empty(), |this| this.child(list))
}
