use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::base::input::{InputEvent, InputState, MaskPattern};
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, Context, ElementId, Entity, FocusHandle, Focusable, FontWeight, IntoElement,
    SharedString, Subscription, Window, div, px, transparent_black,
};

use siir_gui::edit;
use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_gui::view_data::TrailerRow;
use siir_save::Command;

use crate::ui::{self, tokens};
use crate::view::AppView;

const LIST_WIDTH: f32 = 290.0;

pub(super) struct TrailersState {
    pub(super) selected: Option<String>,
    plate: Entity<InputState>,
    plate_error: Option<&'static str>,
    cargo_mass: Entity<InputState>,
    cargo_mass_error: Option<&'static str>,
    pub(super) list_focus: FocusHandle,
}

impl TrailersState {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<'_, AppView>,
        subscriptions: &mut Vec<Subscription>,
    ) -> Self {
        let plate = AppView::input(window, cx);
        let cargo_mass = AppView::number_input(window, cx, false);

        subscriptions.push(cx.observe(&plate, |this, _, cx| {
            this.trailers.plate_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.subscribe(&plate, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.apply_trailer_plate_text(cx);
            }
        }));
        let plate_focus = plate.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&plate_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&plate_focus, window, |this, _, cx| {
            this.commit_trailer_plate(cx)
        }));

        subscriptions.push(cx.observe(&cargo_mass, |this, _, cx| {
            this.trailers.cargo_mass_error = None;
            cx.notify();
        }));
        subscriptions.push(
            cx.subscribe(&cargo_mass, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.apply_cargo_mass_text(cx);
                }
            }),
        );
        let cargo_mass_focus = cargo_mass.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&cargo_mass_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&cargo_mass_focus, window, |this, _, cx| {
            this.commit_cargo_mass(cx)
        }));

        Self {
            selected: None,
            plate,
            plate_error: None,
            cargo_mass,
            cargo_mass_error: None,
            list_focus: cx.focus_handle().tab_stop(true),
        }
    }

    pub(super) fn clear_errors(&mut self) {
        self.plate_error = None;
        self.cargo_mass_error = None;
    }

    pub(super) fn is_valid(&self) -> bool {
        self.plate_error.is_none() && self.cargo_mass_error.is_none()
    }
}

impl AppView {
    fn selected_trailer_row(&self) -> Option<&TrailerRow> {
        let id = self.trailers.selected.as_ref()?;
        self.state
            .open()?
            .view()
            .trailers
            .iter()
            .find(|trailer| &trailer.id == id)
    }

    fn select_trailer(&mut self, id: String, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.trailers.selected = Some(id);
        self.sync_trailer(window, cx);
        cx.notify();
    }

    pub(super) fn default_trailer(&self) -> Option<String> {
        let trailers = &self.state.open()?.view().trailers;
        trailers
            .iter()
            .find(|trailer| trailer.current)
            .or_else(|| trailers.first())
            .map(|trailer| trailer.id.clone())
    }

    pub(super) fn sync_trailer(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some((plate, mass)) = self
            .selected_trailer_row()
            .map(|row| (row.plate.clone(), row.cargo_mass))
        else {
            return;
        };
        self.trailers.plate_error = None;
        self.trailers.cargo_mass_error = None;
        self.trailers.plate.update(cx, |input, cx| {
            input.set_value(plate, window, cx);
        });
        self.trailers.cargo_mass.update(cx, |input, cx| {
            input.set_mask_pattern(
                MaskPattern::Number {
                    separator: Some(format::grouping(game)),
                    fraction: None,
                },
                window,
                cx,
            );
            input.set_value(
                format::number(mass.max(0.0).round() as u64, game),
                window,
                cx,
            );
        });
    }

    fn apply_trailer_plate_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, plate)) = self
            .selected_trailer_row()
            .map(|row| (row.id.clone(), row.plate.clone()))
        else {
            return true;
        };
        let text = self.trailers.plate.read(cx).value().to_string();
        if text.trim() == plate {
            self.trailers.plate_error = None;
            return true;
        }
        let Ok(command) = edit::trailer_plate(&id, &text) else {
            return false;
        };
        self.trailers.plate_error = None;
        if matches!(&command, Command::SetTrailerLicensePlate { plate: next, .. } if next != &plate)
        {
            self.apply(command, cx);
        }
        true
    }

    fn apply_cargo_mass_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, mass)) = self
            .selected_trailer_row()
            .map(|row| (row.id.clone(), row.cargo_mass))
        else {
            return true;
        };
        let text = self.trailers.cargo_mass.read(cx).unmask_value().to_string();
        let Ok(command) = edit::cargo_mass(&id, &text) else {
            return false;
        };
        self.trailers.cargo_mass_error = None;
        if matches!(&command, Command::SetCargoMass { mass: next, .. } if *next != mass) {
            self.apply(command, cx);
        }
        true
    }

    pub(super) fn commit_trailer_plate(&mut self, cx: &mut Context<'_, Self>) {
        if self.selected_trailer_row().is_none() {
            return;
        }
        if !self.apply_trailer_plate_text(cx) {
            self.trailers.plate_error = Some(edit::FieldError::Name.key());
            cx.notify();
        }
    }

    pub(super) fn commit_cargo_mass(&mut self, cx: &mut Context<'_, Self>) {
        if self.selected_trailer_row().is_none() {
            return;
        }
        if !self.apply_cargo_mass_text(cx) {
            self.trailers.cargo_mass_error = Some(edit::FieldError::Number.key());
            cx.notify();
        }
    }

    fn trailer_command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.apply(command, cx);
        self.sync_trailer(window, cx);
    }

    pub(crate) fn render_trailers(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let rows = &open.view().trailers;
        let selected = rows
            .iter()
            .find(|trailer| Some(&trailer.id) == self.trailers.selected.as_ref());
        let mut list = div()
            .id("trailer-list")
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

    fn trailer_row(
        &self,
        trailer: &TrailerRow,
        first: bool,
        cx: &mut Context<'_, Self>,
    ) -> impl IntoElement {
        let palette = tokens::palette();
        let selected = self.trailers.selected.as_ref() == Some(&trailer.id);
        let name = self.trailer_name(trailer);
        let mut label = format!("{}, {}", name, trailer.plate);
        if trailer.current {
            label.push_str(&format!(", {}", t("vehicles.current")));
        }
        let id = trailer.id.clone();
        let max_wear = trailer.body_wear.max(trailer.chassis_wear);
        BaseButton::new((
            ElementId::from("trailer"),
            SharedString::from(trailer.id.clone()),
        ))
        .when(first, |this| this.track_focus(&self.trailers.list_focus))
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
                        .when(trailer.current, |this| {
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
                        .child(div().truncate().child(trailer.plate.clone()))
                        .child(
                            div()
                                .flex_shrink_0()
                                .size(px(7.0))
                                .rounded(px(999.0))
                                .bg(tokens::wear_color(max_wear)),
                        ),
                ),
        )
        .on_click(
            cx.listener(move |this, _, window, cx| this.select_trailer(id.clone(), window, cx)),
        )
    }

    fn trailer_detail(&self, trailer: &TrailerRow, cx: &mut Context<'_, Self>) -> AnyElement {
        let palette = tokens::palette();
        let game = self.state.game();
        let live_plate = self.trailers.plate.read(cx).value().to_string();
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
                                            .child(self.trailer_name(trailer)),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(tokens::TEXT_LABEL))
                                            .text_color(palette.muted)
                                            .child(
                                                trailer
                                                    .source
                                                    .clone()
                                                    .unwrap_or_else(|| trailer.definition.clone()),
                                            ),
                                    ),
                            )
                            .child(ui::plate_preview(&live_plate, None, game)),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_wrap()
                            .gap(px(tokens::SPACE_4))
                            .child(div().flex_1().min_w(px(220.0)).child(ui::field(
                                t("field.trailer_plate"),
                                ui::text_field(
                                    "trailer-plate",
                                    &self.trailers.plate,
                                    t("field.trailer_plate"),
                                ),
                                self.trailers.plate_error.map(t),
                            )))
                            .child(div().flex_1().min_w(px(220.0)).child(ui::field(
                                t("field.cargo_mass"),
                                ui::text_field(
                                    "cargo-mass",
                                    &self.trailers.cargo_mass,
                                    t("field.cargo_mass"),
                                ),
                                self.trailers.cargo_mass_error.map(t),
                            ))),
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
                                    .child(ui::wear_bar(t("wear.body"), trailer.body_wear))
                                    .child(ui::wear_bar(t("wear.chassis"), trailer.chassis_wear)),
                            )
                            .child(
                                div()
                                    .pt(px(tokens::SPACE_2))
                                    .border_t_1()
                                    .border_color(palette.border)
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
                    ),
            )
            .into_any_element()
    }

    fn trailer_name(&self, trailer: &TrailerRow) -> String {
        let body_type = match trailer.body_type.as_deref() {
            Some(body_type) => Some(body_type),
            None => self
                .catalog
                .as_ref()
                .and_then(|catalog| catalog.trailer(&trailer.definition))
                .map(|definition| definition.body_type.as_str()),
        };
        let Some(body_type) = body_type else {
            return trailer.definition.clone();
        };
        let name = self
            .catalog
            .as_ref()
            .and_then(|catalog| catalog.body_type_name(body_type, i18n::current_locale().as_ref()))
            .unwrap_or(body_type);
        let id = trailer.source.as_deref().unwrap_or(&trailer.definition);
        let brand = trailer_brand(id).map(|token| {
            self.catalog
                .as_ref()
                .and_then(|catalog| catalog.brand_name(token, i18n::current_locale().as_ref()))
                .map(str::to_owned)
                .unwrap_or_else(|| brand_label(token))
        });
        match brand {
            Some(brand) => format!("{brand} · {name}"),
            None => name.to_owned(),
        }
    }
}

fn trailer_brand(id: &str) -> Option<&str> {
    id.strip_prefix("trailer_def.")?.split('.').next()
}

fn brand_label(brand: &str) -> String {
    if brand == "scs" {
        return "SCS".to_owned();
    }
    let mut chars = brand.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{brand_label, trailer_brand};

    #[test]
    fn names_the_brand_of_a_definition_id() {
        assert_eq!(
            trailer_brand("trailer_def.krone.profiliner.single_3sa.curtain"),
            Some("krone")
        );
        assert_eq!(trailer_brand("_nameless.6000.02b1.9810"), None);
        assert_eq!(trailer_brand("trailer_def"), None);
    }

    #[test]
    fn capitalizes_a_brand_token() {
        assert_eq!(brand_label("krone"), "Krone");
        assert_eq!(brand_label("schwmuller"), "Schwmuller");
        assert_eq!(brand_label("scs"), "SCS");
        assert_eq!(brand_label(""), "");
    }
}
