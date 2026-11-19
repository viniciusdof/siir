//! The economy form: money, level and experience, skills, ADR classes,
//! cities, and dealerships.

use gpui_kit::prelude::*;
use gpui_kit::{Context, Div, FontWeight, IntoElement, SharedString, div, px};

use siir_gui::edit::ADR_CLASSES;
use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_save::Command;

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

/// The five skills that are levels from 0 to 6, in command order.
const SKILLS: [&str; 5] = [
    "skill.long_dist",
    "skill.heavy",
    "skill.fragile",
    "skill.urgent",
    "skill.mechanical",
];

/// Cell width in the two-column skill grid.
const SKILL_CELL: f32 = (tokens::FORM_WIDTH - 2.0 * tokens::SPACE_6 - tokens::SPACE_4) / 2.0;
/// Cell width in the three-column ADR grid.
const ADR_CELL: f32 = (tokens::FORM_WIDTH - 2.0 * tokens::SPACE_6 - 2.0 * tokens::SPACE_2) / 3.0;

impl AppView {
    pub(crate) fn render_economy(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let (visited_cities, dealerships) = self
            .state
            .open()
            .map(|open| (open.view().visited_cities, open.view().dealerships))
            .unwrap_or_default();
        div()
            .id("economy-scroll")
            .w_full()
            .flex_1()
            .min_w(px(0.0))
            .min_h(px(0.0))
            .flex()
            .items_start()
            .justify_center()
            .overflow_y_scroll()
            .child(
                div()
                    .w_full()
                    .max_w(px(tokens::FORM_WIDTH))
                    .p(px(tokens::SPACE_6))
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_6))
                    .child(self.money_field())
                    .child(self.level_row(cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_4))
                            .child(ui::section_title(t("discovery.skills")))
                            .child(self.skill_grid(cx))
                            .child(
                                div()
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
                                    .child(self.adr_grid(cx)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::section_title(t("economy.discovery")))
                            .child(Self::world_row(
                                "visited-cities",
                                t("world.visited_cities"),
                                visited_cities,
                                Command::SetVisitedCities,
                                cx,
                            ))
                            .child(Self::world_row(
                                "dealerships",
                                t("world.dealerships"),
                                dealerships,
                                Command::SetDealershipsDiscovered,
                                cx,
                            )),
                    ),
            )
    }

    /// The balance in the profile's currency, with the game's signs around it.
    fn money_field(&self) -> impl IntoElement {
        let currency = self.currency();
        let before = format!("{}{}", currency.sign1, currency.sign2);
        let before = before.trim();
        let after = currency.sign3.trim();
        ui::field(
            t("field.money"),
            ui::text_field("money", &self.money, t("field.money"))
                .when(!before.is_empty(), |this| this.prefix(before.to_owned()))
                .when(!after.is_empty(), |this| this.suffix(after.to_owned())),
            self.money_error.map(t),
        )
    }

    /// The level slider with its title, and the experience it stands for,
    /// shown but not edited. Without a level table only the experience shows.
    fn level_row(&self, cx: &mut Context<'_, Self>) -> Div {
        let palette = tokens::palette();
        let game = self.state.game();
        let experience = self
            .state
            .open()
            .map(|open| open.view().experience)
            .unwrap_or_default();
        let experience_text = format!("{} XP", format::number(experience, game));
        let Some(levels) = self.levels() else {
            return div().child(ui::field(
                t("field.experience"),
                div()
                    .text_size(px(tokens::TEXT_BODY))
                    .text_color(palette.text)
                    .child(experience_text),
                None,
            ));
        };
        let level = self.level.read(cx).value().end().round().max(0.0) as u32;
        let heading = match levels.title(level, i18n::current_locale().as_ref()) {
            Some(title) => format!("{} · {title}", format::number(u64::from(level), game)),
            None => format::number(u64::from(level), game),
        };
        let next = levels
            .experience(levels.level(experience).saturating_add(1))
            .filter(|next| *next <= u64::from(u32::MAX));
        let detail = match next {
            Some(next) => format!(
                "{experience_text} · {} XP {} {}",
                format::number(next - experience, game),
                t("level.to_next"),
                format::number(u64::from(levels.level(experience)) + 1, game),
            ),
            None => format!("{experience_text} · {}", t("level.max")),
        };
        div()
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_1))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_LABEL))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.muted)
                            .child(t("field.level")),
                    )
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_BODY))
                            .text_color(palette.text)
                            .child(heading),
                    ),
            )
            .child(ui::slider(&self.level, &self.level_focus, t("field.level")))
            .child(
                div()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(detail),
            )
    }

    /// One all-or-nothing list: its size, the action that fills it, and the
    /// action that empties it.
    fn world_row(
        id: &'static str,
        label: String,
        count: usize,
        command: fn(bool) -> Command,
        cx: &mut Context<'_, Self>,
    ) -> Div {
        let palette = tokens::palette();
        let discover = t("world.discover_all");
        let clear = t("world.clear");
        div()
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_2))
            .min_h(px(tokens::ROW_HEIGHT))
            .border_b_1()
            .border_color(palette.border)
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .truncate()
                    .text_size(px(tokens::TEXT_BODY))
                    .text_color(palette.text)
                    .child(label.clone()),
            )
            .child(
                div()
                    .flex_shrink_0()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(count.to_string()),
            )
            .child(
                // shortcut: the fill command is idempotent but always re-applies,
                // so a second click still marks the save edited; skip it once the
                // read model exposes the city total.
                ui::button(
                    SharedString::from(format!("{id}-discover")),
                    discover.clone(),
                )
                .aria_label(format!("{discover}: {label}"))
                .tone(Tone::Quiet)
                .on_click(cx.listener(move |this, _, _, cx| this.apply(command(true), |_| {}, cx))),
            )
            .child(
                ui::button(SharedString::from(format!("{id}-clear")), clear.clone())
                    .aria_label(format!("{clear}: {label}"))
                    .tone(Tone::Quiet)
                    .disabled(count == 0)
                    .on_click(
                        cx.listener(move |this, _, _, cx| this.apply(command(false), |_| {}, cx)),
                    ),
            )
    }

    /// Two sliders per row, each with its label and level.
    fn skill_grid(&self, cx: &mut Context<'_, Self>) -> Div {
        let mut cells: Vec<Div> = Vec::with_capacity(SKILLS.len());
        for (ix, key) in SKILLS.iter().enumerate() {
            cells.push(self.skill_cell(ix, key, cx));
        }
        div()
            .flex()
            .flex_wrap()
            .gap(px(tokens::SPACE_4))
            .children(cells)
    }

    fn skill_cell(&self, ix: usize, key: &'static str, cx: &mut Context<'_, Self>) -> Div {
        let palette = tokens::palette();
        let value = self.skills[ix].read(cx).value().end().round() as u64;
        div()
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_1))
            .w(px(SKILL_CELL))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_BODY))
                            .text_color(palette.text)
                            .child(t(key)),
                    )
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_LABEL))
                            .text_color(palette.muted)
                            .child(format!("{value} / 6")),
                    ),
            )
            .child(ui::slider(&self.skills[ix], &self.skill_focus[ix], t(key)))
    }

    /// The six ADR classes as independent checkboxes, three per row.
    fn adr_grid(&self, cx: &mut Context<'_, Self>) -> Div {
        let mut cells: Vec<Div> = Vec::with_capacity(ADR_CLASSES.len());
        for (ix, key) in ADR_CLASSES.iter().enumerate() {
            cells.push(
                div().w(px(ADR_CELL)).child(
                    ui::checkbox(("adr-class", ix), t(key), self.adr[ix])
                        .when(ix == 0, |this| this.track_focus(&self.adr_focus))
                        .on_change(cx.listener(move |this, checked: &bool, _, cx| {
                            this.set_adr(ix, *checked, cx)
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
}
