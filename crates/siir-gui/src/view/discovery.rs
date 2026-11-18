//! The save tree and the error banner.

use gpui_kit::assets::IconName;
use gpui_kit::base::Button as BaseButton;
use gpui_kit::prelude::*;
use gpui_kit::{
    App, ClickEvent, Context, ElementId, FontWeight, IntoElement, Role, SharedString, Window, div,
    px, transparent_black,
};

use siir_gui::i18n::t;
use siir_gui::state::{ProfileRow, SaveRow};

use crate::ui::{self, Tone, tokens};
use crate::view::AppView;

const AUTOSAVES_WIDTH: f32 = 176.0;

impl AppView {
    pub(crate) fn render_home(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let mut page = div().flex().flex_col().gap(px(tokens::SPACE_6));

        if self.state.saves().is_empty() {
            page = page.child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(tokens::SPACE_4))
                    .pt(px(tokens::SPACE_6))
                    .child(ui::empty_state(
                        IconName::FolderSearch,
                        t("empty.no_saves"),
                        t("empty.no_saves_hint"),
                    ))
                    .child(
                        ui::button("empty-add-folder", t("discovery.add_folder"))
                            .tone(Tone::Primary)
                            .icon(IconName::FolderOpen)
                            .on_click(cx.listener(|this, _, _, cx| this.pick_folder(cx))),
                    ),
            );
        } else {
            let query = self.search.read(cx).value().to_lowercase();
            let searching = !query.is_empty();
            let mut tree = div().flex().flex_col();
            let mut groups = 0;

            for (ix, profile) in self.state.profiles().iter().enumerate() {
                let rows =
                    visible_rows(self.state.saves(), profile, ix, &query, self.hide_autosaves);
                if rows.is_empty() {
                    continue;
                }
                groups += 1;
                let expanded = searching || !self.collapsed.contains(&profile.path);
                tree = tree.child(group(self, profile, expanded, rows, cx));
            }

            page = page
                .child(self.continue_block(cx))
                .child(self.browser_toolbar(cx));
            if groups == 0 {
                let (icon, title, hint) = empty_state_for(searching, self.hide_autosaves);
                page = page.child(div().pt(px(tokens::SPACE_4)).child(ui::empty_state(
                    icon,
                    t(title),
                    t(hint),
                )));
            } else {
                page = page.child(tree);
            }
        }

        div()
            .id("home-scroll")
            .w_full()
            .flex_1()
            .min_h(px(0.0))
            .flex()
            .items_start()
            .justify_center()
            .overflow_y_scroll()
            .child(
                div()
                    .w(px(tokens::CONTENT_WIDTH))
                    .p(px(tokens::SPACE_6))
                    .child(page),
            )
    }

    fn browser_toolbar(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let (label, icon) = if self.hide_autosaves {
            (t("home.show_autosaves"), IconName::Eye)
        } else {
            (t("home.hide_autosaves"), IconName::EyeOff)
        };
        div()
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_2))
            .child(div().flex_1().min_w(px(0.0)).child(ui::text_field(
                "save-search",
                &self.search,
                t("home.search"),
            )))
            .child(
                ui::button("toggle-autosaves", label)
                    .icon(icon)
                    .width(AUTOSAVES_WIDTH)
                    .on_click(cx.listener(|this, _, _, cx| this.toggle_autosaves(cx))),
            )
    }

    fn continue_block(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let Some(row) = self.state.newest() else {
            return div().into_any_element();
        };
        let meta = self
            .state
            .profiles()
            .get(row.profile)
            .map(|profile| format!("{} · {}", profile.name, t(profile.kind_key)))
            .unwrap_or_default();
        div()
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_4))
            .p(px(tokens::SPACE_4))
            .rounded(px(tokens::RADIUS))
            .bg(palette.panel)
            .border_1()
            .border_color(palette.border)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.0))
                    .min_w(px(0.0))
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_LABEL))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(palette.accent)
                            .child(t("home.most_recent")),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(tokens::TEXT_TITLE))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(row.name.clone()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(px(tokens::TEXT_LABEL))
                            .text_color(palette.muted)
                            .child(meta),
                    ),
            )
            .child(div().flex_1())
            .child(
                ui::button("continue", t("home.continue"))
                    .tone(Tone::Primary)
                    .on_click(cx.listener(|this, _, _, cx| this.open_newest(cx))),
            )
            .into_any_element()
    }

    pub(crate) fn render_notice(&self, key: &'static str, detail: &str) -> impl IntoElement {
        let palette = tokens::palette();
        let message = t(key);
        div()
            .id("notice")
            .role(Role::Status)
            .aria_label(message.clone())
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_2))
            .px(px(tokens::SPACE_4))
            .py(px(tokens::SPACE_2))
            .bg(palette.surface)
            .border_b_1()
            .border_color(palette.danger)
            .child(
                div()
                    .flex_shrink_0()
                    .text_color(palette.danger_text)
                    .child(ui::icon(IconName::CircleAlert, 16.0, palette.danger_text)),
            )
            .child(
                div()
                    .text_size(px(tokens::TEXT_BODY))
                    .text_color(palette.danger_text)
                    .child(message),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(0.0))
                    .truncate()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(detail.to_owned()),
            )
    }
}

fn group(
    view: &AppView,
    profile: &ProfileRow,
    expanded: bool,
    rows: Vec<(usize, String, Option<String>)>,
    cx: &mut Context<'_, AppView>,
) -> impl IntoElement {
    let palette = tokens::palette();
    let path = profile.path.clone();
    let count = rows.len();
    let mut rows_element = div().flex().flex_col().pl(px(28.0));
    for (save_ix, name, suffix) in rows {
        let row = &view.state.saves()[save_ix];
        rows_element = rows_element.child(save_row(
            (ElementId::from("save"), row.path.display().to_string()),
            format!("{} · {}", profile.name, row.name),
            view.state.selected_save() == Some(save_ix),
            cx.listener(move |this, _, _, cx| this.open_save(save_ix, cx)),
            name,
            suffix,
        ));
    }
    div()
        .flex()
        .flex_col()
        .child(
            BaseButton::new((
                ElementId::from("profile"),
                profile.path.display().to_string(),
            ))
            .w_full()
            .h(px(40.0))
            .px(px(tokens::SPACE_2))
            .gap(px(tokens::SPACE_2))
            .flex()
            .items_center()
            .rounded(px(tokens::RADIUS))
            .border_1()
            .border_color(transparent_black())
            .bg(transparent_black())
            .text_size(px(tokens::TEXT_BODY))
            .text_color(palette.text)
            .aria_expanded(expanded)
            .accessibility_label(profile.name.clone())
            .hover(|style| style.bg(palette.surface_hover))
            .active(|style| style.bg(palette.surface_active))
            .focus_visible(|style| style.border_color(palette.accent))
            .child(ui::icon(
                if expanded {
                    IconName::ChevronDown
                } else {
                    IconName::ChevronRight
                },
                16.0,
                palette.muted,
            ))
            .child(ui::section_title(profile.name.clone()))
            .child(
                div()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(t(profile.kind_key)),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(count.to_string()),
            )
            .on_click(cx.listener(move |this, _, _, cx| this.toggle_profile(path.clone(), cx))),
        )
        .when(expanded, |this| this.child(rows_element))
}

fn save_row(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    selected: bool,
    on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    name: String,
    suffix: Option<String>,
) -> impl IntoElement {
    let palette = tokens::palette();
    let row = BaseButton::new(id)
        .w_full()
        .h(px(tokens::SAVE_ROW_HEIGHT))
        .px(px(tokens::SPACE_3))
        .flex()
        .items_center()
        .gap(px(tokens::SPACE_3))
        .rounded(px(tokens::RADIUS))
        .border_1()
        .border_color(transparent_black())
        .bg(if selected {
            palette.surface_selected
        } else {
            transparent_black()
        })
        .text_size(px(tokens::TEXT_ROW))
        .text_color(palette.text)
        .font_weight(if selected {
            FontWeight::MEDIUM
        } else {
            FontWeight::NORMAL
        })
        .accessibility_label(label)
        .hover(|style| style.bg(palette.surface_hover))
        .active(|style| style.bg(palette.surface_active))
        .focus_visible(|style| style.border_color(palette.accent))
        .child(div().flex_1().min_w(px(0.0)).truncate().child(name))
        .when_some(suffix, |this, suffix| {
            this.child(
                div()
                    .text_size(px(tokens::TEXT_LABEL))
                    .text_color(palette.muted)
                    .child(suffix),
            )
        })
        .on_click(on_click);
    div().border_b_1().border_color(palette.border).child(row)
}

fn folder(row: &SaveRow) -> String {
    row.path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

fn is_autosave(name: &str) -> bool {
    name.to_lowercase().starts_with("autosave")
}

fn matches(profile: &ProfileRow, row: &SaveRow, folder: &str, query: &str) -> bool {
    format!("{} {} {}", profile.name, row.name, folder)
        .to_lowercase()
        .contains(query)
}

fn empty_state_for(
    searching: bool,
    hide_autosaves: bool,
) -> (IconName, &'static str, &'static str) {
    if searching {
        (IconName::SearchX, "empty.no_match", "empty.no_match_hint")
    } else if hide_autosaves {
        (
            IconName::EyeOff,
            "empty.autosaves_hidden",
            "empty.autosaves_hidden_hint",
        )
    } else {
        (
            IconName::FileSearch,
            "empty.no_match",
            "empty.no_match_hint",
        )
    }
}

fn visible_rows(
    saves: &[SaveRow],
    profile: &ProfileRow,
    profile_ix: usize,
    query: &str,
    hide_autosaves: bool,
) -> Vec<(usize, String, Option<String>)> {
    saves
        .iter()
        .enumerate()
        .filter(|(_, row)| row.profile == profile_ix)
        .filter_map(|(save_ix, row)| {
            let folder = folder(row);
            if hide_autosaves && (is_autosave(&row.name) || is_autosave(&folder)) {
                return None;
            }
            if !query.is_empty() && !matches(profile, row, &folder, query) {
                return None;
            }
            let suffix = (folder != row.name).then_some(folder);
            Some((save_ix, row.name.clone(), suffix))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn profile() -> ProfileRow {
        ProfileRow {
            path: PathBuf::from("/profiles/AAA"),
            name: "AAA".to_owned(),
            kind_key: "kind.custom",
        }
    }

    fn row(name: &str, folder: &str) -> SaveRow {
        SaveRow {
            profile: 0,
            path: PathBuf::from(format!("/profiles/AAA/save/{folder}")),
            name: name.to_owned(),
            file_time: 0,
        }
    }

    fn rows() -> Vec<SaveRow> {
        vec![
            row("test-truck", "truck"),
            row("teste", "1"),
            row("autosave", "autosave"),
            row("autosave_drive", "autosave_drive"),
        ]
    }

    #[test]
    fn empty_message_explains_the_filter() {
        assert_eq!(empty_state_for(true, true).1, "empty.no_match");
        assert_eq!(empty_state_for(false, true).1, "empty.autosaves_hidden");
        assert_eq!(empty_state_for(false, false).1, "empty.no_match");
    }

    #[test]
    fn hides_autosaves_by_folder_or_name() {
        let found = visible_rows(&rows(), &profile(), 0, "", true);
        let names: Vec<_> = found.iter().map(|(_, name, _)| name.as_str()).collect();
        assert_eq!(names, ["test-truck", "teste"]);
    }

    #[test]
    fn searches_name_folder_and_profile() {
        let rows = rows();
        let profile = profile();
        let names = |query: &str| -> Vec<String> {
            visible_rows(&rows, &profile, 0, query, false)
                .into_iter()
                .map(|(_, name, _)| name)
                .collect()
        };
        assert_eq!(names("truck"), ["test-truck"]);
        assert_eq!(names("1"), ["teste"]);
        assert_eq!(names("autosave"), ["autosave", "autosave_drive"]);
        assert_eq!(names("aaa").len(), 4);
        assert!(names("nothing").is_empty());
    }

    #[test]
    fn keeps_the_folder_name_when_it_differs() {
        let found = visible_rows(&rows(), &profile(), 0, "teste", false);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].2.as_deref(), Some("1"));
    }
}
