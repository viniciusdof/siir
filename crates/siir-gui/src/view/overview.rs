//! The overview of the open save.

use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{AnyElement, Context, FocusHandle, Focusable, FontWeight, IntoElement, div, px};

use siir_gui::edit::ADR_CLASSES;
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
            .map(|money| format::money(money, game))
            .unwrap_or_else(|| "—".to_owned());
        let vehicle = view
            .current_vehicle
            .as_deref()
            .map(|token| {
                self.catalog
                    .as_ref()
                    .and_then(|catalog| {
                        catalog.vehicle_name(token, i18n::current_locale().as_ref())
                    })
                    .unwrap_or(token)
                    .to_owned()
            })
            .unwrap_or_else(|| "—".to_owned());
        let adr = ADR_CLASSES
            .iter()
            .enumerate()
            .filter(|(ix, _)| view.skills.adr & (1u64 << *ix) != 0)
            .map(|(_, key)| t(key))
            .collect::<Vec<_>>()
            .join(", ");
        let adr = if adr.is_empty() { t("adr.none") } else { adr };
        let money_focus = self.money.read(cx).focus_handle(cx);
        let experience_focus = self.experience.read(cx).focus_handle(cx);
        let economy = |field: &'static str, value: String, target: FocusHandle| -> AnyElement {
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
        let levels = [
            view.skills.long_dist,
            view.skills.heavy,
            view.skills.fragile,
            view.skills.urgent,
            view.skills.mechanical,
        ];
        let skills = std::iter::once(("skill.adr", adr, self.adr_focus.clone())).chain(
            [
                "skill.long_dist",
                "skill.heavy",
                "skill.fragile",
                "skill.urgent",
                "skill.mechanical",
            ]
            .into_iter()
            .zip(levels)
            .zip(&self.skill_focus)
            .map(|((key, level), focus)| (key, level.to_string(), focus.clone())),
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
                    .gap(px(tokens::SPACE_6))
                    .p(px(tokens::SPACE_6))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_4))
                            .pb(px(tokens::SPACE_6))
                            .border_b_1()
                            .border_color(palette.border)
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .items_end()
                                    .gap(px(tokens::SPACE_6))
                                    .child(stat(
                                        t("field.money"),
                                        economy("field.money", money, money_focus),
                                        tokens::TEXT_TITLE * 1.6,
                                        None,
                                    ))
                                    .child(stat(
                                        t("field.experience"),
                                        economy(
                                            "field.experience",
                                            format::number(view.experience, game),
                                            experience_focus,
                                        ),
                                        tokens::TEXT_TITLE * 1.1,
                                        None,
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(tokens::SPACE_6))
                                    .child(stat(
                                        t("field.headquarters"),
                                        view.hq_city.clone(),
                                        tokens::TEXT_BODY,
                                        Some(360.0),
                                    ))
                                    .child(stat(
                                        t("field.vehicle"),
                                        vehicle,
                                        tokens::TEXT_BODY,
                                        Some(360.0),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_3))
                            .child(ui::section_title(t("discovery.fleet")))
                            .child(
                                div()
                                    .flex()
                                    .flex_wrap()
                                    .gap(px(tokens::SPACE_6))
                                    .child(ui::metric(
                                        IconName::Truck,
                                        t("field.vehicles"),
                                        format::number(view.counts.vehicles as u64, game),
                                    ))
                                    .child(ui::metric(
                                        IconName::Container,
                                        t("field.trailers"),
                                        format::number(view.counts.trailers as u64, game),
                                    ))
                                    .child(ui::metric(
                                        IconName::Users,
                                        t("field.drivers"),
                                        format::number(view.counts.drivers as u64, game),
                                    ))
                                    .child(ui::metric(
                                        IconName::Warehouse,
                                        t("field.garages"),
                                        format::number(view.counts.garages as u64, game),
                                    )),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_3))
                            .child(ui::section_title(t("discovery.market")))
                            .child(div().flex().flex_wrap().gap(px(tokens::SPACE_6)).child(
                                ui::metric(
                                    IconName::ClipboardList,
                                    t("field.offers"),
                                    format::number(view.counts.offers as u64, game),
                                ),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::section_title(t("discovery.skills")))
                            .children(skills.into_iter().map(|(key, value, focus)| {
                                ui::info_row(t(key), economy(key, value, focus))
                            })),
                    ),
            )
            .into_any_element()
    }
}

fn stat(
    label: String,
    value: impl IntoElement,
    size: f32,
    max_width: Option<f32>,
) -> impl IntoElement {
    let palette = tokens::palette();
    div()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .child(
            div()
                .text_size(px(tokens::TEXT_LABEL))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.muted)
                .child(label),
        )
        .child(
            div()
                .text_size(px(size))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(palette.text)
                .when_some(max_width, |this, width| this.max_w(px(width)))
                .child(value),
        )
}
