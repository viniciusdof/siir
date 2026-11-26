mod discovery;
mod economy;
mod overview;
mod profile;
mod shell;
mod trailers;
mod vehicles;

use std::path::{Path, PathBuf};

use gpui_kit::assets::IconName;
use gpui_kit::base::Popover;
use gpui_kit::base::input::InputState;
use gpui_kit::prelude::*;
use gpui_kit::{
    Context, Entity, FocusHandle, Focusable, IntoElement, PathPromptOptions, Render, Role,
    SharedString, Subscription, Window, div, px,
};

use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_gui::state::{Settings, State};
use siir_save::catalog::{Catalog, Currency, Levels};
use siir_save::{Command, Game};

use crate::ui::{self, Tone, tokens};

use self::discovery::HomeState;
use self::economy::EconomyState;
use self::profile::ProfileState;
use self::trailers::TrailersState;
use self::vehicles::VehiclesState;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Overview,
    Economy,
    Profile,
    Vehicles,
    Trailers,
}

struct LeaveDialogState {
    open: bool,
    close_after_discard: bool,
    dialog_focus: FocusHandle,
    keep_editing_focus: FocusHandle,
    switch_focus: FocusHandle,
}

impl LeaveDialogState {
    fn new(cx: &mut Context<'_, AppView>) -> Self {
        Self {
            open: false,
            close_after_discard: false,
            dialog_focus: cx.focus_handle(),
            keep_editing_focus: cx.focus_handle().tab_stop(true),
            switch_focus: cx.focus_handle().tab_stop(true),
        }
    }

    fn reset(&mut self) {
        self.open = false;
        self.close_after_discard = false;
    }
}

pub struct AppView {
    state: State,
    settings: Settings,
    section: Section,
    locale_focus: FocusHandle,
    home: HomeState,
    economy: EconomyState,
    profile: ProfileState,
    vehicles: VehiclesState,
    trailers: TrailersState,
    leave: LeaveDialogState,
    syncing: bool,
    catalog: Option<Catalog>,
    action_error: Option<&'static str>,
    _subscriptions: Vec<Subscription>,
}

impl AppView {
    pub fn new(
        settings: Settings,
        mut dirs: Vec<PathBuf>,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) -> Self {
        let game = settings.game().unwrap_or(Game::Ets2);
        dirs.extend(settings.dirs.iter().cloned());

        let mut subscriptions = Vec::new();
        let home = HomeState::new(window, cx, &mut subscriptions);
        let economy = EconomyState::new(window, cx, &mut subscriptions);
        let profile = ProfileState::new(window, cx, &mut subscriptions);
        let vehicles = VehiclesState::new(window, cx, &mut subscriptions);
        let trailers = TrailersState::new(window, cx, &mut subscriptions);
        let leave = LeaveDialogState::new(cx);

        let close = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            let asked = close
                .update(cx, |view, cx| {
                    let valid = view.commit_fields(cx);
                    if view.state.asks_before_close(valid) {
                        view.leave.close_after_discard = true;
                        view.ask_leave(window, cx);
                        true
                    } else {
                        false
                    }
                })
                .unwrap_or(false);
            if !asked {
                cx.quit();
            }
            false
        });

        Self {
            state: State::with_dirs(game, dirs),
            settings,
            section: Section::Overview,
            locale_focus: cx.focus_handle().tab_stop(true),
            home,
            economy,
            profile,
            vehicles,
            trailers,
            leave,
            syncing: false,
            catalog: None,
            action_error: None,
            _subscriptions: subscriptions,
        }
    }

    pub(super) fn input(window: &mut Window, cx: &mut Context<'_, Self>) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx))
    }

    pub(super) fn number_input(
        window: &mut Window,
        cx: &mut Context<'_, Self>,
        negative: bool,
    ) -> Entity<InputState> {
        cx.new(|cx| {
            InputState::new(window, cx)
                .validate(move |text, _| !text.contains('.') && (negative || !text.contains('-')))
        })
    }

    fn set_locale(&mut self, locale: &str, window: &mut Window, cx: &mut Context<'_, Self>) {
        i18n::set_locale(locale);
        self.settings.set_locale(locale);
        self.save_settings();
        self.home.search.update(cx, |search, cx| {
            search.set_placeholder(t("home.search"), window, cx)
        });
        cx.notify();
    }

    fn set_game(&mut self, game: Game, cx: &mut Context<'_, Self>) {
        tokens::apply_theme(cx, game);
        self.state.set_game(game);
        self.settings.set_game(game);
        self.save_settings();
        self.catalog = None;
        cx.notify();
    }

    fn save_settings(&mut self) {
        if self.settings.save().is_err() {
            self.action_error = Some("error.settings");
        } else if self.action_error == Some("error.settings") {
            self.action_error = None;
        }
    }

    fn set_section(&mut self, section: Section, cx: &mut Context<'_, Self>) {
        self.section = section;
        cx.notify();
    }

    fn follow_link(
        &mut self,
        section: Section,
        target: &FocusHandle,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.set_section(section, cx);
        window.focus(target, cx);
    }

    fn open_save(&mut self, ix: usize, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.state.open_save(ix);
        self.after_open(window, cx);
    }

    fn open_newest(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.state.open_newest();
        self.after_open(window, cx);
    }

    fn after_open(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.state.open().is_some() {
            self.catalog = Catalog::load(self.state.game()).ok();
            self.section = Section::Overview;
            self.vehicles.show_used_market = false;
            self.clear_errors();
            self.sync_forms(window, cx);
        }
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let valid = self.commit_fields(cx);
        if self.state.asks_before_close(valid) {
            self.ask_leave(window, cx);
            return;
        }
        self.close_save(window, cx);
    }

    fn ask_leave(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.action_error = None;
        self.leave.open = true;
        window.focus(&self.leave.keep_editing_focus, cx);
        cx.notify();
    }

    fn discard(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let closes = self.leave.close_after_discard;
        self.close_save(window, cx);
        if closes {
            cx.quit();
        }
    }

    fn keep_editing(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.leave.reset();
        self.action_error = None;
        window.focus(&self.leave.switch_focus, cx);
        cx.notify();
    }

    fn close_save(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        if let Some(ix) = self.state.selected_save()
            && let Some(row) = self.state.saves().get(ix)
            && let Some(profile) = self.state.profiles().get(row.profile)
        {
            self.home.collapsed.remove(&profile.path);
        }
        self.state.close();
        self.catalog = None;
        self.vehicles.show_used_market = false;
        self.clear_errors();
        window.focus(&self.home.search.read(cx).focus_handle(cx), cx);
        cx.notify();
    }

    fn dismiss_action(&mut self, cx: &mut Context<'_, Self>) {
        self.action_error = None;
        cx.notify();
    }

    fn dismiss_saved(&mut self, cx: &mut Context<'_, Self>) {
        self.state.clear_saved_backup();
        cx.notify();
    }

    fn dismiss_notice(&mut self, cx: &mut Context<'_, Self>) {
        self.state.clear_notice();
        cx.notify();
    }

    fn reveal_backup(&mut self, cx: &mut Context<'_, Self>) {
        let Some(path) = self.state.saved_backup().map(Path::to_path_buf) else {
            return;
        };
        if reveal(&path).is_err() {
            self.action_error = Some("error.reveal");
            cx.notify();
        }
    }

    fn save(&mut self, cx: &mut Context<'_, Self>) {
        if !self.commit_fields(cx) {
            return;
        }
        if !self.state.edited() {
            cx.notify();
            return;
        }
        self.action_error = match self.state.save() {
            Ok(_) => None,
            Err(_) => Some("error.save"),
        };
        self.leave.reset();
        cx.notify();
    }

    fn commit_fields(&mut self, cx: &mut Context<'_, Self>) -> bool {
        self.commit_money(cx);
        self.commit_profile_name(cx);
        self.commit_odometer(cx);
        self.commit_plate(cx);
        self.commit_used_price(cx);
        self.commit_trailer_plate(cx);
        self.commit_cargo_mass(cx);
        self.economy.is_valid()
            && self.profile.is_valid()
            && self.vehicles.is_valid()
            && self.trailers.is_valid()
    }

    pub(super) fn levels(&self) -> Option<&Levels> {
        self.catalog
            .as_ref()
            .map(Catalog::levels)
            .filter(|levels| !levels.steps.is_empty())
    }

    pub(super) fn currency(&self) -> Currency {
        let index = self.state.open().map(|open| open.currency()).unwrap_or(0);
        self.catalog
            .as_ref()
            .and_then(|catalog| catalog.currencies().get(index))
            .cloned()
            .unwrap_or_else(|| format::native_currency(self.state.game()))
    }

    pub(super) fn vehicle_name(&self, model: Option<&str>) -> String {
        model
            .map(|token| {
                self.catalog
                    .as_ref()
                    .and_then(|catalog| {
                        catalog.vehicle_name(token, i18n::current_locale().as_ref())
                    })
                    .unwrap_or(token)
                    .to_owned()
            })
            .unwrap_or_else(|| "—".to_owned())
    }

    pub(super) fn apply(&mut self, command: Command, cx: &mut Context<'_, Self>) {
        self.leave.reset();
        match self.state.apply(command) {
            Ok(()) => self.action_error = None,
            Err(_) => self.action_error = Some("error.apply"),
        }
        cx.notify();
    }

    fn clear_errors(&mut self) {
        self.economy.clear_errors();
        self.profile.clear_errors();
        self.vehicles.clear_errors();
        self.trailers.clear_errors();
        self.action_error = None;
        self.leave.reset();
    }

    fn sync_forms(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.sync_economy(window, cx);
        self.sync_profile(window, cx);
        self.vehicles.selected = self.default_vehicle();
        self.sync_vehicle(window, cx);
        self.trailers.selected = self.default_trailer();
        self.sync_trailer(window, cx);
    }

    fn add_dirs(&mut self, dirs: Vec<PathBuf>, cx: &mut Context<'_, Self>) {
        if !dirs.is_empty() {
            self.state.add_dirs(dirs.clone());
            self.settings.add_dirs(dirs);
            self.save_settings();
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

    fn locale_menu(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let current = i18n::current_locale();
        let label = i18n::language_name(current.as_ref());
        let accessible = format!("{}: {label}", t("field.language"));
        let view = cx.entity().downgrade();
        Popover::new("locale-menu")
            .anchor(gpui_kit::Anchor::TopRight)
            .trigger(
                ui::button("locale-trigger", label)
                    .aria_label(accessible)
                    .track_focus(&self.locale_focus)
                    .tone(Tone::Quiet)
                    .trailing_icon(IconName::ChevronDown),
            )
            .content(move |_, _window, cx| {
                let popover = cx.entity();
                let mut menu = div()
                    .flex()
                    .flex_col()
                    .gap(px(tokens::SPACE_1))
                    .p(px(tokens::SPACE_1))
                    .bg(palette.panel)
                    .border_1()
                    .border_color(palette.border)
                    .rounded(px(tokens::RADIUS));
                for locale in i18n::locales() {
                    let selected = locale.as_ref() == current.as_ref();
                    let tag = locale.into_owned();
                    let name = i18n::language_name(&tag);
                    let id = SharedString::from(format!("locale-option-{tag}"));
                    let view = view.clone();
                    let popover = popover.clone();
                    menu = menu.child(
                        ui::button(id, name)
                            .tone(Tone::Quiet)
                            .selected(selected)
                            .on_click(move |_, window, cx| {
                                view.update(cx, |view, cx| view.set_locale(&tag, window, cx))
                                    .ok();
                                popover.update(cx, |state, cx| state.dismiss(window, cx));
                            }),
                    );
                }
                menu.into_any_element()
            })
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
            let edited = self.state.edited();
            shell
                .child(ui::pill(
                    None,
                    t(i18n::game_short(self.state.game())),
                    palette.accent,
                    palette.accent_soft,
                ))
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
                    div()
                        .w(px(48.0))
                        .text_size(px(tokens::TEXT_LABEL))
                        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
                        .text_color(palette.accent)
                        .child(if edited {
                            t("editor.edited")
                        } else {
                            String::new()
                        }),
                )
                .child(
                    ui::button("switch-save", t("home.switch_save"))
                        .icon(IconName::List)
                        .track_focus(&self.leave.switch_focus)
                        .on_click(cx.listener(|this, _, window, cx| this.close(window, cx))),
                )
                .child(
                    ui::button("save", t("editor.save"))
                        .tone(Tone::Primary)
                        .icon(IconName::Save)
                        .disabled(!edited)
                        .on_click(cx.listener(|this, _, _, cx| this.save(cx))),
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
                                .tone(Tone::Quiet)
                                .width(72.0)
                                .selected(game == Game::Ets2)
                                .toggled(game == Game::Ets2)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.set_game(Game::Ets2, cx)),
                                ),
                        )
                        .child(
                            ui::button("game-ats", t(siir_gui::i18n::game_short(Game::Ats)))
                                .tone(Tone::Quiet)
                                .width(72.0)
                                .selected(game == Game::Ats)
                                .toggled(game == Game::Ats)
                                .on_click(
                                    cx.listener(|this, _, _, cx| this.set_game(Game::Ats, cx)),
                                ),
                        ),
                )
                .child(div().flex_1())
                .child(self.locale_menu(cx))
                .child(
                    ui::button("add-folder", t("discovery.add_folder"))
                        .icon(IconName::FolderOpen)
                        .on_click(cx.listener(|this, _, _, cx| this.pick_folder(cx))),
                )
        }
    }

    fn render_banners(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let mut banners = div().flex().flex_col();
        if let Some(key) = self.action_error {
            let message = t(key);
            banners = banners.child(
                div()
                    .id("action-error")
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
                    .child(div().flex_shrink_0().child(ui::icon(
                        IconName::CircleAlert,
                        16.0,
                        palette.danger_text,
                    )))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(0.0))
                            .truncate()
                            .text_size(px(tokens::TEXT_BODY))
                            .text_color(palette.danger_text)
                            .child(message),
                    )
                    .child(
                        ui::button("dismiss-error", t("banner.dismiss"))
                            .tone(Tone::Quiet)
                            .icon(IconName::X)
                            .on_click(cx.listener(|this, _, _, cx| this.dismiss_action(cx))),
                    ),
            );
        }
        if let Some(path) = self.state.saved_backup() {
            let message = t("editor.saved");
            let path = path.display().to_string();
            let label = format!("{message} {path}");
            banners = banners.child(
                div()
                    .id("saved")
                    .role(Role::Status)
                    .aria_label(label)
                    .flex()
                    .items_center()
                    .gap(px(tokens::SPACE_2))
                    .px(px(tokens::SPACE_4))
                    .py(px(tokens::SPACE_2))
                    .bg(palette.surface)
                    .border_b_1()
                    .border_color(palette.success)
                    .child(div().flex_shrink_0().child(ui::icon(
                        IconName::CircleCheck,
                        16.0,
                        palette.success,
                    )))
                    .child(
                        div()
                            .text_size(px(tokens::TEXT_BODY))
                            .text_color(palette.success)
                            .child(message),
                    )
                    .child(
                        div()
                            .id("reveal-backup")
                            .flex_1()
                            .min_w(px(0.0))
                            .truncate()
                            .text_size(px(tokens::TEXT_LABEL))
                            .text_color(palette.muted)
                            .cursor_pointer()
                            .on_click(cx.listener(|this, _, _, cx| this.reveal_backup(cx)))
                            .child(path),
                    )
                    .child(
                        ui::button("reveal-backup-button", t(i18n::reveal_label()))
                            .tone(Tone::Quiet)
                            .icon(IconName::FolderOpen)
                            .on_click(cx.listener(|this, _, _, cx| this.reveal_backup(cx))),
                    )
                    .child(
                        ui::button("dismiss-saved", t("banner.dismiss"))
                            .tone(Tone::Quiet)
                            .icon(IconName::X)
                            .on_click(cx.listener(|this, _, _, cx| this.dismiss_saved(cx))),
                    ),
            );
        }
        banners
    }

    fn render_screen(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let notice = self
            .state
            .notice()
            .map(|notice| (notice.key, notice.detail.clone()));
        let content = if self.state.open().is_some() {
            div()
                .flex()
                .flex_1()
                .min_w(px(0.0))
                .child(self.render_sidebar(cx))
                .child(self.render_section(cx))
                .into_any_element()
        } else {
            self.render_home(cx).into_any_element()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(palette.bg)
            .text_color(palette.text)
            .font_family(tokens::FONT_SANS)
            .child(self.render_header(cx))
            .when_some(notice, |this, (key, detail)| {
                this.child(self.render_notice(key, &detail, cx))
            })
            .child(self.render_banners(cx))
            .child(self.render_leave_dialog(cx))
            .child(
                div()
                    .w_full()
                    .flex_1()
                    .min_h(px(0.0))
                    .min_w(px(0.0))
                    .flex()
                    .child(content),
            )
    }

    fn render_leave_dialog(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let view = cx.entity().downgrade();
        ui::confirm(
            t("dialog.unsaved_title"),
            t("dialog.unsaved_message"),
            &self.leave.dialog_focus,
        )
        .open(self.leave.open)
        .on_cancel(move |window, cx| {
            view.update(cx, |view, cx| view.keep_editing(window, cx))
                .ok();
        })
        .action(
            ui::button("discard-changes", t("editor.discard"))
                .tone(Tone::Danger)
                .on_click(cx.listener(|this, _, window, cx| this.discard(window, cx))),
        )
        .action(
            ui::button("keep-editing", t("editor.keep_editing"))
                .track_focus(&self.leave.keep_editing_focus)
                .on_click(cx.listener(|this, _, window, cx| this.keep_editing(window, cx))),
        )
    }

    fn render_section(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        match self.section {
            Section::Overview => self.render_overview(cx).into_any_element(),
            Section::Economy => self.render_economy(cx).into_any_element(),
            Section::Profile => self.render_profile().into_any_element(),
            Section::Vehicles => self.render_vehicles(cx).into_any_element(),
            Section::Trailers => self.render_trailers(cx).into_any_element(),
        }
    }
}

fn reveal(path: &Path) -> std::io::Result<()> {
    let mut command = match std::env::consts::OS {
        "macos" => {
            let mut command = std::process::Command::new("open");
            command.arg("-R").arg(path);
            command
        }
        "windows" => {
            let mut command = std::process::Command::new("explorer");
            command.arg(format!("/select,{}", path.display()));
            command
        }
        _ => {
            let mut command = std::process::Command::new("xdg-open");
            command.arg(path.parent().unwrap_or(path));
            command
        }
    };
    command.spawn().map(|_| ())
}

impl Render for AppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<'_, Self>) -> impl IntoElement {
        self.render_screen(cx)
    }
}
