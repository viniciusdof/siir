use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{
    AnyElement, Context, FocusHandle, Focusable, FontWeight, IntoElement, div, px, relative,
};

use siir_gui::edit::{ADR_CLASSES, ADR_CODES};
use siir_gui::format;
use siir_gui::i18n::{self, t};

use crate::ui::{self, tokens};
use crate::view::{AppView, Section};

impl AppView {
    pub(crate) fn render_overview(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let view = open.view();
        let game = self.state.game();
        let money = view
            .money
            .map(|money| format::money(money, &self.currency(), game))
            .unwrap_or_else(|| "—".to_owned());
        let vehicle_name = self.vehicle_name(view.current_vehicle.as_deref());
        let current_vehicle = view.vehicles.iter().find(|row| row.current);
        let hq_city = self.city_name(&view.hq_city);
        let adr_text = ADR_CLASSES
            .iter()
            .enumerate()
            .filter(|(ix, _)| view.skills.adr & (1u64 << *ix) != 0)
            .map(|(_, key)| t(key))
            .collect::<Vec<_>>()
            .join(", ");
        let adr_text = if adr_text.is_empty() {
            t("adr.none")
        } else {
            adr_text
        };
        let money_focus = self.economy.money.read(cx).focus_handle(cx);
        let level_info = self.levels().map(|levels| {
            let level = levels.level(view.experience);
            let number = format::number(u64::from(level), game);
            let title = levels
                .title(level, i18n::current_locale().as_ref())
                .map(str::to_owned);
            let start = levels.experience(level).unwrap_or(0);
            let next = levels
                .experience(level.saturating_add(1))
                .filter(|next| *next <= u64::from(u32::MAX));
            let progress = match next {
                Some(next) if next > start => ((view.experience.saturating_sub(start)) as f32
                    / (next - start) as f32)
                    .clamp(0.0, 1.0),
                _ => 1.0,
            };
            let xp_detail = match next {
                Some(next) => format!(
                    "{} XP · {} XP {} {}",
                    format::number(view.experience, game),
                    format::number(next.saturating_sub(view.experience), game),
                    t("level.to_next"),
                    format::number(u64::from(level) + 1, game),
                ),
                None => format!(
                    "{} XP · {}",
                    format::number(view.experience, game),
                    t("level.max"),
                ),
            };
            (number, title, progress, xp_detail)
        });
        let economy_link =
            |field: &'static str, value: String, target: FocusHandle| -> AnyElement {
                let label = t(field);
                ui::link(
                    format!("overview-{field}"),
                    i18n::edit_link_label(&label, &value, "nav.economy"),
                    value,
                    cx.listener(move |this, _, window, cx| {
                        this.follow_link(Section::Economy, &target, window, cx)
                    }),
                )
                .into_any_element()
            };
        let skill_levels = [
            view.skills.long_dist,
            view.skills.heavy,
            view.skills.fragile,
            view.skills.urgent,
            view.skills.mechanical,
        ];
        let skill_keys = [
            "skill.long_dist",
            "skill.heavy",
            "skill.fragile",
            "skill.urgent",
            "skill.mechanical",
        ];

        let money_card = ui::card()
            .flex_1()
            .min_w(px(340.0))
            .flex()
            .flex_col()
            .justify_between()
            .gap(px(tokens::SPACE_4))
            .child(
                div()
                    .flex()
                    .items_start()
                    .justify_between()
                    .gap(px(tokens::SPACE_3))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .items_start()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::pill(
                                Some(IconName::Wallet),
                                t("field.money"),
                                palette.success,
                                palette.success_soft,
                            ))
                            .child(
                                div()
                                    .text_size(px(tokens::TEXT_TITLE * 1.65))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(palette.text)
                                    .child(economy_link("field.money", money, money_focus)),
                            ),
                    )
                    .when_some(level_info.as_ref(), |this, (number, title, _, _)| {
                        let summary = match title {
                            Some(title) => format!("{number} · {title}"),
                            None => number.clone(),
                        };
                        let badge = format!("{} {number}", t("field.level"));
                        let target = self.economy.level_focus.clone();
                        this.child(ui::link(
                            "overview-field.level",
                            i18n::edit_link_label(&t("field.level"), &summary, "nav.economy"),
                            ui::pill(None, badge, palette.accent, palette.accent_soft),
                            cx.listener(move |this, _, window, cx| {
                                this.follow_link(Section::Economy, &target, window, cx)
                            }),
                        ))
                    }),
            )
            .when_some(level_info, |this, (_, title, progress, xp_detail)| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(tokens::SPACE_2))
                                .when_some(title, |row, title| {
                                    row.child(
                                        div()
                                            .text_size(px(tokens::TEXT_BODY))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_color(palette.text)
                                            .child(title),
                                    )
                                })
                                .child(
                                    div()
                                        .text_size(px(tokens::TEXT_LABEL))
                                        .text_color(palette.muted)
                                        .child(xp_detail),
                                ),
                        )
                        .child(
                            div()
                                .w_full()
                                .h(px(7.0))
                                .rounded(px(999.0))
                                .bg(palette.surface)
                                .overflow_hidden()
                                .child(
                                    div()
                                        .h_full()
                                        .w(relative(progress))
                                        .rounded(px(999.0))
                                        .bg(palette.accent),
                                ),
                        ),
                )
            });

        let vehicle_card = ui::card()
            .flex_1()
            .min_w(px(300.0))
            .flex()
            .flex_col()
            .justify_between()
            .gap(px(tokens::SPACE_3))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap(px(tokens::SPACE_2))
                    .child(ui::pill(
                        Some(IconName::Truck),
                        t("field.vehicle"),
                        palette.accent,
                        palette.accent_soft,
                    ))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(tokens::SPACE_1))
                            .text_size(px(tokens::TEXT_LABEL))
                            .text_color(palette.muted)
                            .child(ui::icon(IconName::MapPin, 12.0, palette.muted))
                            .child(format!("{}: {hq_city}", t("field.headquarters"))),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_start()
                    .gap(px(tokens::SPACE_2))
                    .child(
                        div()
                            .text_size(px(17.0))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.text)
                            .child(if current_vehicle.is_some() {
                                let target = self.vehicles.list_focus.clone();
                                ui::link(
                                    "overview-current-vehicle",
                                    i18n::edit_link_label(
                                        &t("field.vehicle"),
                                        &vehicle_name,
                                        "nav.vehicles",
                                    ),
                                    vehicle_name.clone(),
                                    cx.listener(move |this, _, window, cx| {
                                        this.follow_link(Section::Vehicles, &target, window, cx)
                                    }),
                                )
                                .into_any_element()
                            } else {
                                vehicle_name.into_any_element()
                            }),
                    )
                    .when_some(current_vehicle, |this, row| {
                        this.child(ui::plate_preview(&row.plate, row.country.as_deref(), game))
                    }),
            )
            .when_some(current_vehicle, |this, row| {
                this.child(
                    div()
                        .pt(px(tokens::SPACE_2))
                        .border_t_1()
                        .border_color(palette.border)
                        .flex()
                        .items_center()
                        .justify_between()
                        .gap(px(tokens::SPACE_3))
                        .text_size(px(tokens::TEXT_LABEL))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(tokens::SPACE_1))
                                .text_color(palette.muted)
                                .child(t("field.fuel"))
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(palette.text)
                                        .child(format::percent(row.fuel)),
                                ),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(tokens::SPACE_1))
                                .text_color(palette.muted)
                                .child(t("wear.engine"))
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(tokens::wear_color(row.engine_wear))
                                        .child(format::percent(row.engine_wear)),
                                ),
                        ),
                )
            });

        let metrics_strip = div()
            .flex()
            .flex_wrap()
            .gap(px(tokens::SPACE_3))
            .child(ui::metric(
                IconName::Truck,
                palette.accent,
                t("field.vehicles"),
                {
                    let label = t("field.vehicles");
                    let value = format::number(view.counts.vehicles as u64, game);
                    let target = self.vehicles.list_focus.clone();
                    ui::link(
                        "overview-field.vehicles",
                        i18n::edit_link_label(&label, &value, "nav.vehicles"),
                        value,
                        cx.listener(move |this, _, window, cx| {
                            this.follow_link(Section::Vehicles, &target, window, cx)
                        }),
                    )
                },
            ))
            .child(ui::metric(
                IconName::Container,
                palette.info,
                t("field.trailers"),
                {
                    let label = t("field.trailers");
                    let value = format::number(view.counts.trailers as u64, game);
                    let target = self.trailers.list_focus.clone();
                    ui::link(
                        "overview-field.trailers",
                        i18n::edit_link_label(&label, &value, "nav.trailers"),
                        value,
                        cx.listener(move |this, _, window, cx| {
                            this.follow_link(Section::Trailers, &target, window, cx)
                        }),
                    )
                },
            ))
            .child(ui::metric(
                IconName::Users,
                palette.purple,
                t("field.drivers"),
                format::number(view.counts.drivers as u64, game),
            ))
            .child(ui::metric(
                IconName::Warehouse,
                palette.success,
                t("field.garages"),
                format::number(view.counts.garages as u64, game),
            ))
            .child(ui::metric(
                IconName::ClipboardList,
                palette.cyan,
                t("field.offers"),
                format::number(view.counts.offers as u64, game),
            ));

        let adr_target = self.economy.adr_focus.clone();
        let skills_card = ui::card()
            .flex()
            .flex_col()
            .gap(px(tokens::SPACE_3))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap(px(tokens::SPACE_2))
                    .child(ui::section_title(t("discovery.skills")))
                    .child(ui::link(
                        "overview-skill.adr",
                        i18n::edit_link_label(&t("skill.adr"), &adr_text, "nav.economy"),
                        div()
                            .flex()
                            .items_center()
                            .gap(px(tokens::SPACE_2))
                            .child(
                                div()
                                    .text_size(px(tokens::TEXT_LABEL))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(palette.muted)
                                    .child(t("skill.adr")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(tokens::SPACE_1))
                                    .children(ADR_CODES.iter().enumerate().map(|(ix, code)| {
                                        let active = view.skills.adr & (1u64 << ix) != 0;
                                        div()
                                            .w(px(20.0))
                                            .h(px(20.0))
                                            .rounded(px(4.0))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .bg(if active {
                                                palette.accent_soft
                                            } else {
                                                palette.surface
                                            })
                                            .border_1()
                                            .border_color(if active {
                                                palette.accent
                                            } else {
                                                palette.border
                                            })
                                            .text_size(px(11.0))
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(if active {
                                                palette.accent
                                            } else {
                                                palette.muted
                                            })
                                            .child(*code)
                                    })),
                            ),
                        cx.listener(move |this, _, window, cx| {
                            this.follow_link(Section::Economy, &adr_target, window, cx)
                        }),
                    )),
            )
            .child(
                div().flex().flex_wrap().gap(px(tokens::SPACE_2)).children(
                    skill_keys
                        .into_iter()
                        .zip(skill_levels)
                        .zip(&self.economy.skill_focus)
                        .map(|((key, level), focus)| {
                            let label = t(key);
                            let value = format!("{level} / 6");
                            let target = focus.clone();
                            div()
                                .flex_1()
                                .min_w(px(150.0))
                                .bg(palette.surface)
                                .border_1()
                                .border_color(palette.border)
                                .rounded(px(tokens::RADIUS))
                                .p(px(tokens::SPACE_3))
                                .flex()
                                .flex_col()
                                .gap(px(tokens::SPACE_2))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .gap(px(tokens::SPACE_1))
                                        .child(
                                            div()
                                                .truncate()
                                                .text_size(px(tokens::TEXT_LABEL))
                                                .text_color(palette.muted)
                                                .child(label.clone()),
                                        )
                                        .child(
                                            div()
                                                .flex_shrink_0()
                                                .text_size(px(tokens::TEXT_LABEL))
                                                .font_weight(FontWeight::SEMIBOLD)
                                                .text_color(palette.text)
                                                .child(ui::link(
                                                    format!("overview-{key}"),
                                                    i18n::edit_link_label(
                                                        &label,
                                                        &value,
                                                        "nav.economy",
                                                    ),
                                                    value,
                                                    cx.listener(move |this, _, window, cx| {
                                                        this.follow_link(
                                                            Section::Economy,
                                                            &target,
                                                            window,
                                                            cx,
                                                        )
                                                    }),
                                                )),
                                        ),
                                )
                                .child(ui::skill_pips(level))
                        }),
                ),
            );

        div()
            .id("overview-scroll")
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
                    .max_w(px(tokens::CONTENT_WIDTH))
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_4))
                    .p(px(tokens::SPACE_6))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(tokens::SPACE_4))
                            .child(money_card)
                            .child(vehicle_card),
                    )
                    .child(metrics_strip)
                    .child(skills_card),
            )
            .into_any_element()
    }

    fn city_name(&self, token: &str) -> String {
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.city_name(token, i18n::current_locale().as_ref()))
            .unwrap_or(token)
            .to_owned()
    }
}
