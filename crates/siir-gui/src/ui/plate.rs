use gpui_kit::prelude::*;
use gpui_kit::{FontWeight, IntoElement, div, px, rgb};
use siir_save::Game;

pub fn plate_preview(text: &str, country: Option<&str>, game: Game) -> impl IntoElement {
    let shown = if text.trim().is_empty() {
        "—".to_owned()
    } else {
        text.trim().to_owned()
    };
    match game {
        Game::Ets2 => {
            let code = country.map(eu_code).unwrap_or_default();
            div()
                .flex()
                .items_stretch()
                .min_w(px(164.0))
                .max_w(px(260.0))
                .h(px(36.0))
                .rounded(px(6.0))
                .overflow_hidden()
                .bg(rgb(0xefefe9))
                .border_2()
                .border_color(rgb(0x1c1e22))
                .child(
                    div()
                        .w(px(26.0))
                        .flex_shrink_0()
                        .bg(rgb(0x1c4aa6))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(1.0))
                        .child(
                            div()
                                .text_size(px(8.0))
                                .text_color(rgb(0xf5c542))
                                .child("★"),
                        )
                        .when(!code.is_empty(), |this| {
                            this.child(
                                div()
                                    .text_size(px(9.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xffffff))
                                    .child(code),
                            )
                        }),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .px(px(12.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .truncate()
                        .text_size(px(15.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x141619))
                        .child(shown),
                )
                .into_any_element()
        }
        Game::Ats => {
            let header = country
                .map(|token| token.replace('_', " ").to_ascii_uppercase())
                .unwrap_or_default();
            div()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .min_w(px(148.0))
                .max_w(px(240.0))
                .px(px(14.0))
                .py(px(4.0))
                .rounded(px(6.0))
                .bg(rgb(0xf3efe4))
                .border_2()
                .border_color(rgb(0x2b3a55))
                .when(!header.is_empty(), |this| {
                    this.child(
                        div()
                            .truncate()
                            .text_size(px(9.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xb8382c))
                            .child(header),
                    )
                })
                .child(
                    div()
                        .truncate()
                        .text_size(px(16.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(rgb(0x1b2a4a))
                        .child(shown),
                )
                .into_any_element()
        }
    }
}

fn eu_code(token: &str) -> String {
    let known = match token {
        "albania" => Some("AL"),
        "austria" => Some("A"),
        "belgium" => Some("B"),
        "bosnia" => Some("BIH"),
        "bulgaria" => Some("BG"),
        "croatia" => Some("HR"),
        "czech" => Some("CZ"),
        "denmark" => Some("DK"),
        "estonia" => Some("EST"),
        "finland" => Some("FIN"),
        "france" => Some("F"),
        "germany" => Some("D"),
        "greece" => Some("GR"),
        "hungary" => Some("H"),
        "italy" => Some("I"),
        "kosovo" => Some("RKS"),
        "latvia" => Some("LV"),
        "lithuania" => Some("LT"),
        "luxembourg" => Some("L"),
        "macedonia" => Some("NMK"),
        "montenegro" => Some("MNE"),
        "netherlands" => Some("NL"),
        "norway" => Some("N"),
        "poland" => Some("PL"),
        "portugal" => Some("P"),
        "romania" => Some("RO"),
        "russia" => Some("RUS"),
        "serbia" => Some("SRB"),
        "slovakia" => Some("SK"),
        "slovenia" => Some("SLO"),
        "spain" => Some("E"),
        "sweden" => Some("S"),
        "switzerland" => Some("CH"),
        "turkey" => Some("TR"),
        "uk" => Some("UK"),
        _ => None,
    };
    match known {
        Some(code) => code.to_owned(),
        None => token
            .chars()
            .take(2)
            .collect::<String>()
            .to_ascii_uppercase(),
    }
}

#[cfg(test)]
mod tests {
    use super::eu_code;

    #[test]
    fn maps_known_european_countries_and_falls_back() {
        assert_eq!(eu_code("germany"), "D");
        assert_eq!(eu_code("netherlands"), "NL");
        assert_eq!(eu_code("portugal"), "P");
        assert_eq!(eu_code("iceland"), "IC");
    }
}
