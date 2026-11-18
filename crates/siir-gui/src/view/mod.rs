//! The one window: a save tree and the open save.

mod discovery;
mod overview;

use std::collections::HashSet;
use std::path::PathBuf;

use gpui_kit::assets::IconName;
use gpui_kit::base::input::InputState;
use gpui_kit::prelude::*;
use gpui_kit::{
    Context, Focusable, IntoElement, PathPromptOptions, Render, Role, Subscription, Window, div, px,
};

use siir_gui::i18n::t;
use siir_gui::state::State;
use siir_save::Game;

use crate::ui::{self, tokens};

/// The application window.
pub struct AppView {
    state: State,
    search: gpui_kit::Entity<InputState>,
    _search_changes: Subscription,
    _search_focus: Subscription,
    _search_blur: Subscription,
    collapsed: HashSet<PathBuf>,
    hide_autosaves: bool,
}

impl AppView {
    /// Discovers saves for the first game under the folders given on the
    /// command line.
    pub fn new(dirs: Vec<PathBuf>, window: &mut Window, cx: &mut Context<'_, Self>) -> Self {
        let search = cx.new(|cx| {
            let mut state = InputState::new(window, cx);
            state.set_placeholder(t("home.search"), window, cx);
            state
        });
        let subscription = cx.observe(&search, |_, _, cx| cx.notify());
        let handle = search.read(cx).focus_handle(cx);
        let focus = cx.on_focus(&handle, window, |_, _, cx| cx.notify());
        let blur = cx.on_blur(&handle, window, |_, _, cx| cx.notify());
        Self {
            state: State::with_dirs(Game::Ets2, dirs),
            search,
            _search_changes: subscription,
            _search_focus: focus,
            _search_blur: blur,
            collapsed: HashSet::new(),
            hide_autosaves: true,
        }
    }

    fn set_game(&mut self, game: Game, cx: &mut Context<'_, Self>) {
        self.state.set_game(game);
        cx.notify();
    }

    fn open_save(&mut self, ix: usize, cx: &mut Context<'_, Self>) {
        self.state.open_save(ix);
        cx.notify();
    }

    fn open_newest(&mut self, cx: &mut Context<'_, Self>) {
        self.state.open_newest();
        cx.notify();
    }

    fn close(&mut self, cx: &mut Context<'_, Self>) {
        if let Some(ix) = self.state.selected_save()
            && let Some(row) = self.state.saves().get(ix)
            && let Some(profile) = self.state.profiles().get(row.profile)
        {
            self.collapsed.remove(&profile.path);
        }
        self.state.close();
        cx.notify();
    }

    fn toggle_profile(&mut self, path: PathBuf, cx: &mut Context<'_, Self>) {
        if !self.collapsed.remove(&path) {
            self.collapsed.insert(path);
        }
        cx.notify();
    }

    fn toggle_autosaves(&mut self, cx: &mut Context<'_, Self>) {
        self.hide_autosaves = !self.hide_autosaves;
        cx.notify();
    }

    fn add_dirs(&mut self, dirs: Vec<PathBuf>, cx: &mut Context<'_, Self>) {
        if !dirs.is_empty() {
            self.state.add_dirs(dirs);
            cx.notify();
        }
    }

    fn pick_folder(&mut self, cx: &mut Context<'_, Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some(t("discovery.add_folder").into()),
        });
        cx.spawn(async move |view, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            view.update(cx, |view, cx| view.add_dirs(paths, cx)).ok();
        })
        .detach();
    }

    fn render_header(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let shell = div()
            .flex()
            .items_center()
            .gap(px(tokens::SPACE_3))
            .h(px(tokens::HEADER_HEIGHT))
            .px(px(tokens::SPACE_4))
            .bg(palette.panel)
            .border_b_1()
            .border_color(palette.border);
        if let Some(open) = self.state.open() {
            let view = open.view();
            shell
                .child(
                    div()
                        .flex_1()
                        .min_w(px(0.0))
                        .truncate()
                        .text_size(px(tokens::TEXT_TITLE))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .child(format!("{} - {}", view.profile_name, view.save_name)),
                )
                .child(
                    ui::button("switch-save", t("home.switch_save"))
                        .icon(IconName::List)
                        .on_click(cx.listener(|this, _, _, cx| this.close(cx))),
                )
        } else {
            let game = self.state.game();
            shell
                .child(
                    div()
                        .id("game-switch")
                        .role(Role::Group)
                        .aria_label(t("field.game"))
                        .flex()
                        .items_center()
                        .gap(px(tokens::SPACE_1))
                        .child(
                            ui::button("game-ets2", t(siir_gui::i18n::game_short(Game::Ets2)))
                                .tone(ui::Tone::Quiet)
                                .width(72.0)
                                .selected(game == Game::Ets2)
                                .toggled(game == Game::Ets2)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.set_game(Game::Ets2, cx)),
                                ),
                        )
                        .child(
                            ui::button("game-ats", t(siir_gui::i18n::game_short(Game::Ats)))
                                .tone(ui::Tone::Quiet)
                                .width(72.0)
                                .selected(game == Game::Ats)
                                .toggled(game == Game::Ats)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.set_game(Game::Ats, cx)),
                                ),
                        ),
                )
                .child(div().flex_1())
                .child(
                    ui::button("add-folder", t("discovery.add_folder"))
                        .icon(IconName::FolderOpen)
                        .on_click(cx.listener(|this, _, _, cx| this.pick_folder(cx))),
                )
        }
    }
}

impl Render for AppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let notice = self
            .state
            .notice()
            .map(|notice| (notice.key, notice.detail.clone()));
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.bg)
            .text_color(palette.text)
            .font_family(tokens::FONT_SANS)
            .child(self.render_header(cx))
            .when_some(notice, |this, (key, detail)| {
                this.child(self.render_notice(key, &detail))
            })
            .child(div().flex_1().min_h(px(0.0)).min_w(px(0.0)).flex().child(
                if self.state.open().is_some() {
                    self.render_overview().into_any_element()
                } else {
                    self.render_home(cx).into_any_element()
                },
            ))
    }
}
