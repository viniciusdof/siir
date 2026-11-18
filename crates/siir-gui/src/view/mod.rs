//! The one window: the save list, the editor shell, and the save flow.

mod discovery;
mod economy;
mod overview;
mod profile;
mod shell;
mod trailers;
mod vehicles;

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use gpui_kit::assets::IconName;
use gpui_kit::base::Popover;
use gpui_kit::base::input::{InputEvent, InputState, MaskPattern};
use gpui_kit::base::slider::SliderState;
use gpui_kit::prelude::*;
use gpui_kit::{
    Context, Entity, FocusHandle, Focusable, IntoElement, PathPromptOptions, Render, Role,
    SharedString, Subscription, Window, div, px,
};

use siir_gui::edit;
use siir_gui::format;
use siir_gui::i18n::{self, t};
use siir_gui::state::{Settings, State};
use siir_save::catalog::Catalog;
use siir_save::{Command, Game};

use crate::ui::{self, Tone, tokens};

/// One section of the editor shell.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Overview,
    Economy,
    Profile,
    Vehicles,
    Trailers,
}

/// The application window.
pub struct AppView {
    state: State,
    settings: Settings,
    section: Section,
    search: Entity<InputState>,
    money: Entity<InputState>,
    experience: Entity<InputState>,
    profile_name: Entity<InputState>,
    skills: Vec<Entity<SliderState>>,
    skill_focus: Vec<FocusHandle>,
    adr_focus: FocusHandle,
    selected_vehicle: Option<String>,
    show_all_vehicles: bool,
    odometer: Entity<InputState>,
    odometer_error: Option<&'static str>,
    fuel: Entity<SliderState>,
    fuel_focus: FocusHandle,
    vehicles_focus: FocusHandle,
    selected_trailer: Option<String>,
    trailer_plate: Entity<InputState>,
    cargo_mass: Entity<InputState>,
    trailer_plate_error: Option<&'static str>,
    cargo_mass_error: Option<&'static str>,
    trailers_focus: FocusHandle,
    adr: [bool; 6],
    syncing: bool,
    locale_open: bool,
    catalog: Option<Catalog>,
    money_error: Option<&'static str>,
    experience_error: Option<&'static str>,
    profile_name_error: Option<&'static str>,
    action_error: Option<&'static str>,
    confirm_leave: bool,
    close_after_discard: bool,
    leave_focus: FocusHandle,
    keep_editing_focus: FocusHandle,
    switch_focus: FocusHandle,
    _subscriptions: Vec<Subscription>,
    collapsed: HashSet<PathBuf>,
    hide_autosaves: bool,
}

impl AppView {
    /// Discovers saves for the remembered game and folders.
    pub fn new(mut dirs: Vec<PathBuf>, window: &mut Window, cx: &mut Context<'_, Self>) -> Self {
        let settings = Settings::load();
        if let Some(locale) = settings.locale() {
            i18n::set_locale(locale);
        }
        dirs.extend(settings.dirs.iter().cloned());
        let search = Self::input(window, cx);
        search.update(cx, |search, cx| {
            search.set_placeholder(t("home.search"), window, cx)
        });
        let money = Self::number_input(window, cx, true);
        let experience = Self::number_input(window, cx, false);
        let profile_name = Self::input(window, cx);
        let skills: Vec<Entity<SliderState>> = (0..5)
            .map(|_| {
                cx.new(|_| {
                    SliderState::new()
                        .min(0.0)
                        .max(6.0)
                        .step(1.0)
                        .default_value(0.0)
                })
            })
            .collect();
        let skill_focus: Vec<FocusHandle> =
            (0..5).map(|_| cx.focus_handle().tab_stop(true)).collect();
        let odometer = Self::number_input(window, cx, false);
        let fuel = cx.new(|_| {
            SliderState::new()
                .min(0.0)
                .max(100.0)
                .step(1.0)
                .default_value(0.0)
        });
        let fuel_focus = cx.focus_handle().tab_stop(true);
        let trailer_plate = Self::input(window, cx);
        let cargo_mass = Self::number_input(window, cx, false);

        let mut subscriptions = vec![
            cx.observe(&search, |_, _, cx| cx.notify()),
            cx.observe(&money, |this, _, cx| {
                this.money_error = None;
                cx.notify();
            }),
            cx.observe(&experience, |this, _, cx| {
                this.experience_error = None;
                cx.notify();
            }),
            cx.observe(&profile_name, |this, _, cx| {
                this.profile_name_error = None;
                cx.notify();
            }),
            cx.observe(&odometer, |this, _, cx| {
                this.odometer_error = None;
                cx.notify();
            }),
            cx.observe(&fuel, |this, _, cx| {
                if !this.syncing {
                    this.apply_fuel(cx);
                }
                cx.notify();
            }),
        ];
        subscriptions.push(cx.subscribe(&odometer, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.apply_odometer_text(cx);
            }
        }));
        let odometer_focus = odometer.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&odometer_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&odometer_focus, window, |this, _, cx| {
            this.commit_odometer(cx)
        }));
        subscriptions.push(cx.on_focus(&fuel_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&fuel_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.observe(&trailer_plate, |this, _, cx| {
            this.trailer_plate_error = None;
            cx.notify();
        }));
        subscriptions.push(cx.observe(&cargo_mass, |this, _, cx| {
            this.cargo_mass_error = None;
            cx.notify();
        }));
        subscriptions.push(
            cx.subscribe(&trailer_plate, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.apply_trailer_plate_text(cx);
                }
            }),
        );
        subscriptions.push(
            cx.subscribe(&cargo_mass, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.apply_cargo_mass_text(cx);
                }
            }),
        );
        let trailer_plate_focus = trailer_plate.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&trailer_plate_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&trailer_plate_focus, window, |this, _, cx| {
            this.commit_trailer_plate(cx)
        }));
        let cargo_mass_focus = cargo_mass.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&cargo_mass_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&cargo_mass_focus, window, |this, _, cx| {
            this.commit_cargo_mass(cx)
        }));
        subscriptions.push(cx.subscribe(&money, |this, _, event: &InputEvent, cx| {
            if matches!(event, InputEvent::Change) {
                this.apply_money_text(cx);
            }
        }));
        subscriptions.push(
            cx.subscribe(&experience, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.apply_experience_text(cx);
                }
            }),
        );
        subscriptions.push(
            cx.subscribe(&profile_name, |this, _, event: &InputEvent, cx| {
                if matches!(event, InputEvent::Change) {
                    this.apply_profile_name_text(cx);
                }
            }),
        );
        let search_focus = search.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&search_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&search_focus, window, |_, _, cx| cx.notify()));
        let money_focus = money.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&money_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&money_focus, window, |this, _, cx| this.commit_money(cx)));
        let experience_focus = experience.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&experience_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&experience_focus, window, |this, _, cx| {
            this.commit_experience(cx)
        }));
        let profile_name_focus = profile_name.read(cx).focus_handle(cx);
        subscriptions.push(cx.on_focus(&profile_name_focus, window, |_, _, cx| cx.notify()));
        subscriptions.push(cx.on_blur(&profile_name_focus, window, |this, _, cx| {
            this.commit_profile_name(cx)
        }));
        for handle in &skill_focus {
            subscriptions.push(cx.on_focus(handle, window, |_, _, cx| cx.notify()));
            subscriptions.push(cx.on_blur(handle, window, |_, _, cx| cx.notify()));
        }
        for state in &skills {
            subscriptions.push(cx.observe(state, |this, _, cx| {
                if !this.syncing {
                    this.apply_skills(cx);
                }
                cx.notify();
            }));
        }

        // The window is the whole app, so closing it quits instead of
        // destroying the window: on Touch Bar Macs, AppKit aborts when a GPUI
        // window is torn down (AccessKit#823), while quitting exits cleanly.
        let close = cx.entity().downgrade();
        window.on_window_should_close(cx, move |window, cx| {
            let asked = close
                .update(cx, |view, cx| {
                    let valid = view.commit_fields(cx);
                    if view.state.asks_before_close(valid) {
                        view.close_after_discard = true;
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
            state: State::with_dirs(settings.game().unwrap_or(Game::Ets2), dirs),
            settings,
            section: Section::Overview,
            search,
            money,
            experience,
            profile_name,
            skills,
            skill_focus,
            adr_focus: cx.focus_handle().tab_stop(true),
            selected_vehicle: None,
            show_all_vehicles: false,
            odometer,
            odometer_error: None,
            fuel,
            fuel_focus,
            vehicles_focus: cx.focus_handle().tab_stop(true),
            selected_trailer: None,
            trailer_plate,
            cargo_mass,
            trailer_plate_error: None,
            cargo_mass_error: None,
            trailers_focus: cx.focus_handle().tab_stop(true),
            adr: [false; 6],
            syncing: false,
            locale_open: false,
            catalog: None,
            money_error: None,
            experience_error: None,
            profile_name_error: None,
            action_error: None,
            confirm_leave: false,
            close_after_discard: false,
            leave_focus: cx.focus_handle(),
            keep_editing_focus: cx.focus_handle().tab_stop(true),
            switch_focus: cx.focus_handle().tab_stop(true),
            _subscriptions: subscriptions,
            collapsed: HashSet::new(),
            hide_autosaves: true,
        }
    }

    fn input(window: &mut Window, cx: &mut Context<'_, Self>) -> Entity<InputState> {
        cx.new(|cx| InputState::new(window, cx))
    }

    /// A whole-number field; `negative` also accepts a leading minus.
    fn number_input(
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
        self.search.update(cx, |search, cx| {
            search.set_placeholder(t("home.search"), window, cx)
        });
        cx.notify();
    }

    fn set_game(&mut self, game: Game, cx: &mut Context<'_, Self>) {
        self.state.set_game(game);
        self.settings.set_game(game);
        self.save_settings();
        self.catalog = None;
        cx.notify();
    }

    /// Writes the settings; a failure shows the banner.
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

    /// Opens `section` with focus on the control a link pointed at, since the
    /// link itself is no longer drawn.
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
        if self.state.open().is_some() {
            self.catalog = Catalog::load(self.state.game()).ok();
            self.locale_open = false;
            self.section = Section::Overview;
            self.clear_errors();
            self.sync_forms(window, cx);
        }
        cx.notify();
    }

    fn open_newest(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.state.open_newest();
        if self.state.open().is_some() {
            self.catalog = Catalog::load(self.state.game()).ok();
            self.locale_open = false;
            self.section = Section::Overview;
            self.clear_errors();
            self.sync_forms(window, cx);
        }
        cx.notify();
    }

    fn close(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        if self.state.edited() {
            self.ask_leave(window, cx);
            return;
        }
        self.close_save(window, cx);
    }

    /// Opens the unsaved-changes dialog with focus on the safe choice.
    fn ask_leave(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.action_error = None;
        self.confirm_leave = true;
        window.focus(&self.keep_editing_focus, cx);
        cx.notify();
    }

    /// Leaves the open save without writing it.
    fn discard(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let closes = self.close_after_discard;
        self.close_save(window, cx);
        if closes {
            cx.quit();
        }
    }

    /// Closes the dialog and returns focus to the Switch save trigger.
    fn keep_editing(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.confirm_leave = false;
        self.close_after_discard = false;
        self.action_error = None;
        window.focus(&self.switch_focus, cx);
        cx.notify();
    }

    /// Returns to the save list with focus on its search field, since the
    /// control that closed the save is gone.
    fn close_save(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        if let Some(ix) = self.state.selected_save()
            && let Some(row) = self.state.saves().get(ix)
            && let Some(profile) = self.state.profiles().get(row.profile)
        {
            self.collapsed.remove(&profile.path);
        }
        self.state.close();
        self.catalog = None;
        self.clear_errors();
        window.focus(&self.search.read(cx).focus_handle(cx), cx);
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
        self.confirm_leave = false;
        self.close_after_discard = false;
        cx.notify();
    }

    /// Applies the text of every form field and reports whether all parse.
    fn commit_fields(&mut self, cx: &mut Context<'_, Self>) -> bool {
        self.commit_money(cx);
        self.commit_experience(cx);
        self.commit_profile_name(cx);
        self.commit_odometer(cx);
        self.trailer_plate_error =
            (!self.apply_trailer_plate_text(cx)).then_some(edit::FieldError::Name.key());
        self.cargo_mass_error =
            (!self.apply_cargo_mass_text(cx)).then_some(edit::FieldError::Number.key());
        self.money_error.is_none()
            && self.experience_error.is_none()
            && self.profile_name_error.is_none()
            && self.odometer_error.is_none()
            && self.trailer_plate_error.is_none()
            && self.cargo_mass_error.is_none()
    }

    /// Applies the money text once it parses; reports whether it did.
    fn apply_money_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let text = self.money.read(cx).unmask_value().to_string();
        let Ok(Command::SetMoney(value)) = edit::money(&text) else {
            return false;
        };
        self.money_error = None;
        let current = self.state.open().and_then(|open| open.view().money);
        if current != Some(value) {
            self.apply(Command::SetMoney(value), |_| {}, cx);
        }
        true
    }

    /// Applies the experience text once it parses; reports whether it did.
    fn apply_experience_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let text = self.experience.read(cx).unmask_value().to_string();
        let Ok(Command::SetExperience(value)) = edit::experience(&text) else {
            return false;
        };
        self.experience_error = None;
        let current = self.state.open().map(|open| open.view().experience);
        if current != Some(value) {
            self.apply(Command::SetExperience(value), |_| {}, cx);
        }
        true
    }

    /// Applies the profile name once it is valid; reports whether it was.
    fn apply_profile_name_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some(open) = self.state.open().filter(|open| open.has_profile()) else {
            // Without a profile file the field is not shown, so nothing to check.
            return true;
        };
        let current = open.view().profile_name.clone();
        let text = self.profile_name.read(cx).value().to_string();
        let Ok(Command::RenameProfile(name)) = edit::profile_name(&text) else {
            return false;
        };
        self.profile_name_error = None;
        if name != current {
            self.apply(Command::RenameProfile(name), |_| {}, cx);
        }
        true
    }

    fn commit_profile_name(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_profile_name_text(cx) {
            self.profile_name_error = Some(edit::FieldError::Name.key());
            cx.notify();
        }
    }

    fn commit_money(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_money_text(cx) {
            self.money_error = Some("error.number");
            cx.notify();
        }
    }

    fn commit_experience(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_experience_text(cx) {
            self.experience_error = Some("error.number");
            cx.notify();
        }
    }

    /// The selected vehicle's row in the owned view.
    fn selected_row(&self) -> Option<&siir_gui::view_data::VehicleRow> {
        let id = self.selected_vehicle.as_ref()?;
        self.state
            .open()?
            .view()
            .vehicles
            .iter()
            .find(|vehicle| &vehicle.id == id)
    }

    /// The game's name for a model folder, or the folder when the catalog
    /// does not know it.
    fn vehicle_name(&self, model: Option<&str>) -> String {
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

    fn select_vehicle(&mut self, id: String, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.selected_vehicle = Some(id);
        self.sync_vehicle(window, cx);
        cx.notify();
    }

    fn toggle_all_vehicles(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.show_all_vehicles = !self.show_all_vehicles;
        if !self.show_all_vehicles && !self.selected_row().is_some_and(|row| row.owned) {
            self.selected_vehicle = self.default_vehicle();
            self.sync_vehicle(window, cx);
        }
        cx.notify();
    }

    /// The current truck, else the first listed vehicle.
    fn default_vehicle(&self) -> Option<String> {
        let vehicles = &self.state.open()?.view().vehicles;
        vehicles
            .iter()
            .find(|vehicle| vehicle.current)
            .or_else(|| {
                vehicles
                    .iter()
                    .find(|vehicle| self.show_all_vehicles || vehicle.owned)
            })
            .map(|vehicle| vehicle.id.clone())
    }

    /// Puts the selected vehicle's odometer and fuel into their controls.
    fn sync_vehicle(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some((odometer, fuel)) = self.selected_row().map(|row| (row.odometer, row.fuel)) else {
            return;
        };
        self.odometer_error = None;
        self.odometer.update(cx, |input, cx| {
            input.set_mask_pattern(
                MaskPattern::Number {
                    separator: Some(format::grouping(game)),
                    fraction: None,
                },
                window,
                cx,
            );
            input.set_value(
                format::number(odometer.unwrap_or_default(), game),
                window,
                cx,
            );
        });
        self.syncing = true;
        self.fuel
            .update(cx, |state, cx| state.set_value(fuel * 100.0, window, cx));
        self.syncing = false;
    }

    /// Applies the odometer text once it parses; reports whether it did.
    fn apply_odometer_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, Some(current))) = self
            .selected_row()
            .map(|row| (row.id.clone(), row.odometer))
        else {
            // Without a stored odometer the field is not shown.
            return true;
        };
        let text = self.odometer.read(cx).unmask_value().to_string();
        let Ok(command) = edit::odometer(&id, &text) else {
            return false;
        };
        self.odometer_error = None;
        if matches!(command, Command::SetOdometer { km, .. } if km != current) {
            self.apply(command, |_| {}, cx);
        }
        true
    }

    fn commit_odometer(&mut self, cx: &mut Context<'_, Self>) {
        if !self.apply_odometer_text(cx) {
            self.odometer_error = Some(edit::FieldError::Number.key());
            cx.notify();
        }
    }

    fn apply_fuel(&mut self, cx: &mut Context<'_, Self>) {
        let percent = self.fuel.read(cx).value().end();
        let Some((id, current)) = self.selected_row().map(|row| (row.id.clone(), row.fuel)) else {
            return;
        };
        if (current * 100.0).round() != percent.round() {
            self.apply(edit::fuel(&id, percent), |_| {}, cx);
        }
    }

    /// Applies a vehicle action, then refreshes the controls it can change.
    fn vehicle_command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.apply(command, |_| {}, cx);
        self.sync_vehicle(window, cx);
    }

    /// The selected trailer's row in the owned view.
    fn selected_trailer_row(&self) -> Option<&siir_gui::view_data::TrailerRow> {
        let id = self.selected_trailer.as_ref()?;
        self.state
            .open()?
            .view()
            .trailers
            .iter()
            .find(|trailer| &trailer.id == id)
    }

    fn select_trailer(&mut self, id: String, window: &mut Window, cx: &mut Context<'_, Self>) {
        self.selected_trailer = Some(id);
        self.sync_trailer(window, cx);
        cx.notify();
    }

    /// The attached trailer, else the first one.
    fn default_trailer(&self) -> Option<String> {
        let trailers = &self.state.open()?.view().trailers;
        trailers
            .iter()
            .find(|trailer| trailer.current)
            .or_else(|| trailers.first())
            .map(|trailer| trailer.id.clone())
    }

    /// Puts the selected trailer's plate and cargo mass into their fields.
    fn sync_trailer(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some((plate, mass)) = self
            .selected_trailer_row()
            .map(|row| (row.plate.clone(), row.cargo_mass))
        else {
            return;
        };
        self.trailer_plate_error = None;
        self.cargo_mass_error = None;
        self.trailer_plate.update(cx, |input, cx| {
            input.set_value(plate, window, cx);
        });
        self.cargo_mass.update(cx, |input, cx| {
            input.set_mask_pattern(
                MaskPattern::Number {
                    separator: Some(format::grouping(game)),
                    fraction: None,
                },
                window,
                cx,
            );
            input.set_value(
                format::number(mass.max(0.0).round() as u64, game),
                window,
                cx,
            );
        });
    }

    /// Applies the plate text once it is valid; reports whether it was.
    fn apply_trailer_plate_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, plate)) = self
            .selected_trailer_row()
            .map(|row| (row.id.clone(), row.plate.clone()))
        else {
            // Without a selected trailer the field is not shown.
            return true;
        };
        let text = self.trailer_plate.read(cx).value().to_string();
        let Ok(Command::SetTrailerLicensePlate { plate: next, .. }) =
            edit::trailer_plate(&id, &text)
        else {
            return false;
        };
        self.trailer_plate_error = None;
        if next != plate {
            self.apply(
                edit::trailer_plate(&id, &text).expect("parsed above"),
                |_| {},
                cx,
            );
        }
        true
    }

    /// Applies the cargo mass once it parses; reports whether it did.
    fn apply_cargo_mass_text(&mut self, cx: &mut Context<'_, Self>) -> bool {
        let Some((id, mass)) = self
            .selected_trailer_row()
            .map(|row| (row.id.clone(), row.cargo_mass))
        else {
            return true;
        };
        let text = self.cargo_mass.read(cx).unmask_value().to_string();
        let Ok(Command::SetCargoMass { mass: next, .. }) = edit::cargo_mass(&id, &text) else {
            return false;
        };
        self.cargo_mass_error = None;
        if next != mass {
            self.apply(
                edit::cargo_mass(&id, &text).expect("parsed above"),
                |_| {},
                cx,
            );
        }
        true
    }

    fn commit_trailer_plate(&mut self, cx: &mut Context<'_, Self>) {
        if self.selected_trailer_row().is_none() {
            return;
        }
        if !self.apply_trailer_plate_text(cx) {
            self.trailer_plate_error = Some(edit::FieldError::Name.key());
            cx.notify();
        }
    }

    fn commit_cargo_mass(&mut self, cx: &mut Context<'_, Self>) {
        if self.selected_trailer_row().is_none() {
            return;
        }
        if !self.apply_cargo_mass_text(cx) {
            self.cargo_mass_error = Some(edit::FieldError::Number.key());
            cx.notify();
        }
    }

    /// Applies a trailer action, then refreshes the fields it can change.
    fn trailer_command(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        self.apply(command, |_| {}, cx);
        self.sync_trailer(window, cx);
    }

    fn apply_skills(&mut self, cx: &mut Context<'_, Self>) {
        let levels: [u64; 5] = std::array::from_fn(|ix| {
            self.skills[ix]
                .read(cx)
                .value()
                .end()
                .round()
                .clamp(0.0, 6.0) as u64
        });
        let Some(current) = self.state.open().map(|open| open.view().skills) else {
            return;
        };
        let next = edit::skills(levels, self.adr, current.adr);
        if next != current {
            self.apply(Command::SetSkills(next), |_| {}, cx);
        }
    }

    fn set_adr(&mut self, ix: usize, selected: bool, cx: &mut Context<'_, Self>) {
        if let Some(slot) = self.adr.get_mut(ix) {
            *slot = selected;
        }
        self.apply_skills(cx);
    }

    fn apply(
        &mut self,
        command: Command,
        clear: impl FnOnce(&mut Self),
        cx: &mut Context<'_, Self>,
    ) {
        self.confirm_leave = false;
        self.close_after_discard = false;
        match self.state.apply(command) {
            Ok(()) => {
                clear(self);
                self.action_error = None;
            }
            Err(_) => self.action_error = Some("error.apply"),
        }
        cx.notify();
    }

    fn clear_errors(&mut self) {
        self.money_error = None;
        self.experience_error = None;
        self.profile_name_error = None;
        self.odometer_error = None;
        self.trailer_plate_error = None;
        self.cargo_mass_error = None;
        self.action_error = None;
        self.confirm_leave = false;
        self.close_after_discard = false;
    }

    fn sync_forms(&mut self, window: &mut Window, cx: &mut Context<'_, Self>) {
        let game = self.state.game();
        let Some(open) = self.state.open() else {
            return;
        };
        let view = open.view();
        let money = view.money.unwrap_or_default();
        let experience = view.experience;
        let profile_name = view.profile_name.clone();
        let adr = view.skills.adr;
        let skills = [
            view.skills.long_dist,
            view.skills.heavy,
            view.skills.fragile,
            view.skills.urgent,
            view.skills.mechanical,
        ];
        let mask = MaskPattern::Number {
            separator: Some(format::grouping(game)),
            fraction: None,
        };
        self.money.update(cx, |input, cx| {
            input.set_mask_pattern(mask.clone(), window, cx);
            input.set_value(format::amount(money, game), window, cx);
        });
        self.experience.update(cx, |input, cx| {
            input.set_mask_pattern(mask, window, cx);
            input.set_value(format::number(experience, game), window, cx);
        });
        self.profile_name.update(cx, |input, cx| {
            input.set_value(profile_name, window, cx);
        });
        self.adr = std::array::from_fn(|ix| adr & (1 << ix) != 0);
        self.syncing = true;
        for (state, level) in self.skills.iter().zip(skills) {
            state.update(cx, |state, cx| state.set_value(level as f32, window, cx));
        }
        self.syncing = false;
        self.selected_vehicle = self.default_vehicle();
        self.sync_vehicle(window, cx);
        self.selected_trailer = self.default_trailer();
        self.sync_trailer(window, cx);
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
            self.state.add_dirs(dirs.clone());
            self.settings.dirs.extend(dirs);
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

    /// The language dropdown over every shipped locale.
    fn locale_menu(&self, cx: &mut Context<'_, Self>) -> impl IntoElement {
        let palette = tokens::palette();
        let current = i18n::current_locale();
        let label = i18n::language_name(current.as_ref());
        let accessible = format!("{}: {label}", t("field.language"));
        let view = cx.entity().downgrade();
        Popover::new("locale-menu")
            .anchor(gpui_kit::Anchor::TopRight)
            .open(self.locale_open)
            .on_open_change({
                let view = view.clone();
                move |open, _, cx| {
                    view.update(cx, |view, cx| {
                        view.locale_open = *open;
                        cx.notify();
                    })
                    .ok();
                }
            })
            .trigger(
                ui::button("locale-trigger", label)
                    .aria_label(accessible)
                    .on_a11y_click({
                        let view = view.clone();
                        move |_, cx| {
                            view.update(cx, |view, cx| {
                                view.locale_open = !view.locale_open;
                                cx.notify();
                            })
                            .ok();
                        }
                    })
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
                        .track_focus(&self.switch_focus)
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
            &self.leave_focus,
        )
        .open(self.confirm_leave)
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
                .track_focus(&self.keep_editing_focus)
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

/// Shows `path` in the file manager of the running OS.
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
        // Linux file managers have no standard "select this file" argument.
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
