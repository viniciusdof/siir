//! The overview of the open save.

use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{FontWeight, IntoElement, div, px};

use siir_gui::edit::ADR_CLASSES;
use siir_gui::i18n::{self, t};

use crate::ui::{self, tokens};
use crate::view::AppView;

impl AppView {
    pub(crate) fn render_overview(&self) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(open) = self.state.open() else {
            return div().into_any_element();
        };
        let view = open.view();
        let game = self.state.game();
        let money = view
            .money
            .map(|money| i18n::money(money, game))
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
                                        money,
                                        tokens::TEXT_TITLE * 1.6,
                                        None,
                                    ))
                                    .child(stat(
                                        t("field.experience"),
                                        view.experience.to_string(),
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
                                        view.counts.vehicles.to_string(),
                                    ))
                                    .child(ui::metric(
                                        IconName::Container,
                                        t("field.trailers"),
                                        view.counts.trailers.to_string(),
                                    ))
                                    .child(ui::metric(
                                        IconName::Users,
                                        t("field.drivers"),
                                        view.counts.drivers.to_string(),
                                    ))
                                    .child(ui::metric(
                                        IconName::Warehouse,
                                        t("field.garages"),
                                        view.counts.garages.to_string(),
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
                                    view.counts.offers.to_string(),
                                ),
                            )),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(tokens::SPACE_2))
                            .child(ui::section_title(t("discovery.skills")))
                            .child(ui::info_row(t("skill.adr"), adr))
                            .child(ui::info_row(
                                t("skill.long_dist"),
                                view.skills.long_dist.to_string(),
                            ))
                            .child(ui::info_row(
                                t("skill.heavy"),
                                view.skills.heavy.to_string(),
                            ))
                            .child(ui::info_row(
                                t("skill.fragile"),
                                view.skills.fragile.to_string(),
                            ))
                            .child(ui::info_row(
                                t("skill.urgent"),
                                view.skills.urgent.to_string(),
                            ))
                            .child(ui::info_row(
                                t("skill.mechanical"),
                                view.skills.mechanical.to_string(),
                            )),
                    ),
            )
            .into_any_element()
    }
}

fn stat(label: String, value: String, size: f32, max_width: Option<f32>) -> impl IntoElement {
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
