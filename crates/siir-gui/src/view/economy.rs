use gpui_kit::base::input::{InputEvent, InputState, MaskPattern};
use gpui_kit::base::slider::SliderState;
use gpui_kit::prelude::*;
use gpui_kit::{
    Context, Div, Entity, FocusHandle, Focusable, FontWeight, IntoElement, SharedString,
    Subscription, Window, div, px,
};

use siir_gui::edit::{self, ADR_CLASSES, ADR_CODES};
use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_save::{Command, Game};

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

const LEVEL_SLIDER_MAX: u32 = 100;

const SKILLS: [(&str, &str); 5] = [
    ("skill-long-dist", "skill.long_dist"),
    ("skill-heavy", "skill.heavy"),
    ("skill-fragile", "skill.fragile"),
    ("skill-urgent", "skill.urgent"),
    ("skill-mechanical", "skill.mechanical"),
];

const SKILL_CELL: f32 =
    (tokens::FORM_WIDTH - 2.0 * tokens::SPACE_6 - 2.0 * tokens::SPACE_4 - tokens::SPACE_4) / 2.0;
const ADR_CELL: f32 =
    (tokens::FORM_WIDTH - 2.0 * tokens::SPACE_6 - 2.0 * tokens::SPACE_4 - 2.0 * tokens::SPACE_2)
        / 3.0;

fn level_slider(max: u32) -> SliderState {
    SliderState::new()
        .min(0.0)
        .max(max as f32)
        .step(1.0)
        .default_value(0.0)
}

pub(super) struct EconomyState {
    pub(super) money: Entity<InputState>,
    money_error: Option<&'static str>,
    pub(super) money_dirty: bool,
    level: Entity<SliderState>,
    pub(super) level_focus: FocusHandle,
    pub(super) skill_focus: Vec<FocusHandle>,
    pub(super) adr_focus: FocusHandle,
}

impl EconomyState {
    pub(super) fn new(
        window: &mut Window,
        cx: &mut Context<'_, AppView>,
        subscriptions: &mut Vec<Subscription>,
    ) -> Self {
        let money = AppView::number_input(window, cx, true);
        let level = cx.new(|_| level_slider(LEVEL_SLIDER_MAX));
        let level_focus = cx.focus_handle().tab_stop(true);
        let skill_focus: Vec<FocusHandle> =
            (0..5).map(|_| cx.focus_handle().tab_stop(true)).collect();

        subscriptions.push(cx.observe(&money, |this, _, cx| {
            this.economy.money_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.subscribe_in(
            &money,
            window,
            |this, _, event: &InputEvent, window, cx| {
                if matches!(event, InputEvent::Change)
                    && !this.syncing
                    && this.apply_money_text(cx) == Some(false)
                {
                    this.sync_money(window, cx);
                }
            },
        ));
        let money_focus = money.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&money_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&money_focus, window, |this, _, cx| this.commit_money(cx)));

        subscriptions.push(cx.observe(&level, |this, _, cx| {
            if !this.syncing {
                this.apply_level(cx);
            }
            cx.notify();
        }));
        subscriptions.push(cx.on_focus(&level_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&level_focus, window, |_, _, cx| cx.notify()));

        Self {
            money,
            money_error: None,
            money_dirty: false,
            level,
            level_focus,
            skill_focus,
            adr_focus: cx.focus_handle().tab_stop(true),
        }
    }

    pub(super) fn clear_errors(&mut self) {
        self.money_error = None;
        self.money_dirty = false;
    }

    pub(super) fn is_valid(&self) -> bool {
        self.money_error.is_none()
    }
}

impl AppView {
    pub(super) fn sync_economy(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.sync_money(window, cx);
        self.sync_level(window, cx);
    }

    pub(super) fn sync_level(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let Some(experience) = self.state.open().map(|open| open.view().experience) else {
            return;
        };
        let level = self
            .levels()
            .map(|levels| levels.level(experience))
            .unwrap_or_default();
        self.syncing = true;
        self.economy.level.update(cx, |state, cx| {
            *state = level_slider(LEVEL_SLIDER_MAX.max(level));
            state.set_value(level as f32, window, cx);
        });
        self.syncing = false;
    }

    pub(super) fn sync_money(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some(open) = self.state.open() else {
            return;
        };
        let money = open.view().money.unwrap_or_default();
        let currency = self.currency();
        let mask = MaskPattern::Number {
            separator: Some(format::grouping(game)),
            fraction: None,
        };
        self.economy.money_error = None;
        self.economy.money_dirty = false;
        self.syncing = true;
        self.economy.money.update(cx, |input, cx| {
            input.set_mask_pattern(mask, window, cx);
            input.set_value(
                format::amount(currency.from_native(money), game),
                window,
                cx,
            );
        });
        self.syncing = false;
    }

    fn apply_money_text(&mut self, cx: &mut Context<'_, Self>) -> Option<bool> {
        let Some(current) = self.state.open().and_then(|open| open.view().money) else {
            return Some(false);
        };
        let text = self.economy.money.read(cx).unmask_value().to_string();
        let currency = self.currency();
        let Ok(Command::SetMoney(value)) = edit::money(&text, &currency) else {
            return None;
        };
        self.economy.money_error = None;
        if edit::same_shown_amount(current, value, &currency) {
            return Some(false);
        }
        self.economy.money_dirty = true;
        self.apply(Command::SetMoney(value), cx);
        Some(true)
    }

    pub(super) fn commit_money(&mut self, cx: &mut Context<'_, Self>) {
        if self.apply_money_text(cx).is_none() {
            self.economy.money_error = Some(edit::FieldError::Number.key());
            cx.notify();
        }
    }

    fn apply_level(&mut self, cx: &mut Context<'_, Self>) {
        let level = self.economy.level.read(cx).value().end().round().max(0.0) as u32;
        let Some(levels) = self.levels() else {
            return;
        };
        let Some(current) = self.state.open().map(|open| open.view().experience) else {
            return;
        };
        if levels.level(current) == level {
            return;
        }
        if let Some(command) = edit::level(level, levels) {
            self.apply(command, cx);
        }
    }

    fn skill_levels(&self) -> [u64; 5] {
        let skills = self
            .state
            .open()
            .map(|open| open.view().skills)
            .unwrap_or_default();
        [
            skills.long_dist,
            skills.heavy,
            skills.fragile,
            skills.urgent,
            skills.mechanical,
        ]
    }

    fn adr_classes(&self) -> [bool; 6] {
        let adr = self
            .state
            .open()
            .map(|open| open.view().skills.adr)
            .unwrap_or_default();
        std::array::from_fn(|ix| adr & (1 << ix) != 0)
    }

    fn set_skill(&mut self, ix: usize, level: u64, cx: &mut Context<'_, Self>) {
        let Some(current) = self.state.open().map(|open| open.view().skills) else {
            return;
        };
        let mut levels = self.skill_levels();
        if let Some(slot) = levels.get_mut(ix) {
            *slot = level.min(6);
        }
        let next = edit::skills(levels, self.adr_classes(), current.adr);
        if next != current {
            self.apply(Command::SetSkills(next), cx);
        }
    }

    fn set_adr(&mut self, ix: usize, selected: bool, cx: &mut Context<'_, Self>) {
        let Some(current) = self.state.open().map(|open| open.view().skills) else {
            return;
        };
        let mut classes = self.adr_classes();
        if let Some(slot) = classes.get_mut(ix) {
            *slot = selected;
        }
        let next = edit::skills(self.skill_levels(), classes, current.adr);
        if next != current {
            self.apply(Command::SetSkills(next), cx);
        }
    }

    pub(crate) fn render_economy(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let game = self.state.game();
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
                    .gap(px(tokens::SPACE_4))
                    .child(
                        ui::card()
                            .flex()
                            .flex_wrap()
                            .gap(px(tokens::SPACE_4))
                            .child(div().flex_1().min_w(px(240.0)).child(self.money_field()))
                            .child(div().flex_1().min_w(px(280.0)).child(self.level_row(cx))),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_4))
                            .child(ui::section_title(t("discovery.skills")))
                            .child(self.skill_grid(cx))
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
                                    .child(self.adr_grid(cx)),
                            ),
                    )
                    .child(
                        ui::card()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::section_title(t("economy.discovery")))
                            .child(Self::world_row(
                                "visited-cities",
                                t("world.visited_cities"),
                                visited_cities,
                                game,
                                Command::SetVisitedCities,
                                cx,
                            ))
                            .child(Self::world_row(
                                "dealerships",
                                t("world.dealerships"),
                                dealerships,
                                game,
                                Command::SetDealershipsDiscovered,
                                cx,
                            )),
                    ),
            )
    }

    fn money_field(&self) -> impl IntoElement {
        let currency = self.currency();
        let before = format!("{}{}", currency.sign1, currency.sign2);
        let before = before.trim();
        let after = currency.sign3.trim();
        ui::field(
            t("field.money"),
            ui::text_field("money", &self.economy.money, t("field.money"))
                .when(!before.is_empty(), |this| this.prefix(before.to_owned()))
                .when(!after.is_empty(), |this| this.suffix(after.to_owned())),
            self.economy.money_error.map(t),
        )
    }

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
        let level = self.economy.level.read(cx).value().end().round().max(0.0) as u32;
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
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.accent)
                            .child(heading),
                    ),
            )
            .child(ui::slider(
                &self.economy.level,
                &self.economy.level_focus,
                t("field.level"),
            ))
            .child(
                div()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(detail),
            )
    }

    fn world_row(
        id: &'static str,
        label: String,
        count: usize,
        game: Game,
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
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(palette.muted)
                    .child(format::number(count as u64, game)),
            )
            .child(
                ui::button(
                    SharedString::from(format!("{id}-discover")),
                    discover.clone(),
                )
                .aria_label(format!("{discover}: {label}"))
                .tone(Tone::Quiet)
                .on_click(cx.listener(move |this, _, _, cx| this.apply(command(true), cx))),
            )
            .child(
                ui::button(SharedString::from(format!("{id}-clear")), clear.clone())
                    .aria_label(format!("{clear}: {label}"))
                    .tone(Tone::Quiet)
                    .disabled(count == 0)
                    .on_click(cx.listener(move |this, _, _, cx| this.apply(command(false), cx))),
            )
    }

    fn skill_grid(&self, cx: &mut Context<'_, Self>) -> Div {
        let mut cells: Vec<Div> = Vec::with_capacity(SKILLS.len());
        for (ix, (id, key)) in SKILLS.iter().enumerate() {
            cells.push(self.skill_cell(ix, id, key, cx));
        }
        div()
            .flex()
            .flex_wrap()
            .gap(px(tokens::SPACE_4))
            .children(cells)
    }

    fn skill_cell(
        &self,
        ix: usize,
        id: &'static str,
        key: &'static str,
        cx: &mut Context<'_, Self>,
    ) -> Div {
        let palette = tokens::palette();
        let value = self.skill_levels()[ix];
        let view = cx.entity().downgrade();
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
                &self.economy.skill_focus[ix],
                move |level, _, cx| {
                    view.update(cx, |view, cx| view.set_skill(ix, level, cx))
                        .ok();
                },
            ))
    }

    fn adr_grid(&self, cx: &mut Context<'_, Self>) -> Div {
        let classes = self.adr_classes();
        let mut cells: Vec<Div> = Vec::with_capacity(ADR_CLASSES.len());
        for (ix, key) in ADR_CLASSES.iter().enumerate() {
            cells.push(
                div().w(px(ADR_CELL)).child(
                    ui::checkbox(("adr-class", ix), t(key), classes[ix])
                        .badge(ADR_CODES[ix])
                        .when(ix == 0, |this| this.track_focus(&self.economy.adr_focus))
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
