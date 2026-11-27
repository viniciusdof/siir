use gpui_kit::assets::IconName;
use gpui_kit::prelude::*;
use gpui_kit::{Context, FontWeight, IntoElement, div, px};

use siir_gui::format;
use siir_gui::i18n::t;
use siir_save::Command;

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

impl AppView {
    pub(crate) fn render_offenses(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let game = self.state.game();
        let currency = self.currency();
        let offense = self
            .state
            .open()
            .and_then(|open| open.view().offense.as_ref());

        let content = match offense {
            None => ui::empty_state(
                IconName::ShieldAlert,
                t("empty.no_offenses"),
                t("empty.no_offenses_hint"),
            )
            .into_any_element(),
            Some(offense) => {
                let total_count = offense.total_count();
                let total_fines = offense.total_fines();
                let clear = offense.is_clear();
                let category_rows: Vec<(usize, u64, i64)> =
                    (0..offense.counts.len().max(offense.fines.len()))
                        .map(|ix| {
                            (
                                ix + 1,
                                offense.counts.get(ix).copied().unwrap_or(0),
                                offense.fines.get(ix).copied().unwrap_or(0),
                            )
                        })
                        .filter(|(_, count, fine)| *count > 0 || *fine != 0)
                        .collect();

                let metrics = div()
                    .flex()
                    .flex_wrap()
                    .gap(px(tokens::SPACE_3))
                    .child(ui::metric(
                        IconName::ShieldAlert,
                        if total_count > 0 {
                            palette.danger_text
                        } else {
                            palette.success
                        },
                        t("offenses.total_count"),
                        format::number(total_count, game),
                    ))
                    .child(ui::metric(
                        IconName::ReceiptText,
                        palette.accent,
                        t("offenses.total_fines"),
                        format::money(total_fines, &currency, game),
                    ))
                    .child(ui::metric(
                        IconName::ClipboardList,
                        palette.info,
                        t("offenses.history"),
                        format::number(offense.history_entries, game),
                    ));

                let summary_card =
                    ui::card()
                        .flex()
                        .flex_col()
                        .items_start()
                        .gap(px(tokens::SPACE_3))
                        .child(ui::section_title(t("offenses.summary")))
                        .when(clear, |this| {
                            this.child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .items_start()
                                    .gap(px(tokens::SPACE_1))
                                    .child(ui::pill(
                                        Some(IconName::CircleCheck),
                                        t("offenses.clean"),
                                        palette.success,
                                        palette.success_soft,
                                    ))
                                    .child(
                                        div()
                                            .text_size(px(tokens::TEXT_LABEL))
                                            .text_color(palette.muted)
                                            .child(t("offenses.clean_hint")),
                                    ),
                            )
                        })
                        .child(
                            ui::button("clear-offenses", t("offenses.clear"))
                                .tone(Tone::Danger)
                                .icon(IconName::Trash)
                                .disabled(clear)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.apply(Command::ClearOffenses, cx)
                                })),
                        );

                let breakdown_card = (!category_rows.is_empty()).then(|| {
                    let mut table = ui::card()
                        .flex()
                        .flex_col()
                        .gap(px(tokens::SPACE_2))
                        .child(ui::section_title(t("offenses.categories")))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(tokens::SPACE_3))
                                .pb(px(tokens::SPACE_2))
                                .border_b_1()
                                .border_color(palette.border)
                                .text_size(px(tokens::TEXT_LABEL))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(palette.muted)
                                .child(div().flex_1().child(t("offenses.category")))
                                .child(div().w(px(100.0)).text_right().child(t("offenses.count")))
                                .child(div().w(px(140.0)).text_right().child(t("offenses.fines"))),
                        );
                    for (index, count, fine) in category_rows {
                        table = table.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(tokens::SPACE_3))
                                .min_h(px(tokens::ROW_HEIGHT))
                                .border_b_1()
                                .border_color(palette.border)
                                .text_size(px(tokens::TEXT_BODY))
                                .child(
                                    div()
                                        .flex_1()
                                        .text_color(palette.text)
                                        .child(format!("#{index}")),
                                )
                                .child(
                                    div()
                                        .w(px(100.0))
                                        .text_right()
                                        .text_color(palette.text)
                                        .child(format::number(count, game)),
                                )
                                .child(
                                    div()
                                        .w(px(140.0))
                                        .text_right()
                                        .text_color(palette.muted)
                                        .child(format::money(fine, &currency, game)),
                                ),
                        );
                    }
                    table
                });

                div()
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_4))
                    .child(metrics)
                    .child(summary_card)
                    .when_some(breakdown_card, |this, card| this.child(card))
                    .into_any_element()
            }
        };

        div()
            .id("offenses-scroll")
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
                    .child(content),
            )
    }
}
