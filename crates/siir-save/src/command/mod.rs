//! Save commands that edit a document.

use siir::Document;

use crate::Game;
use crate::Skills;
use crate::error::Error;

mod economy;
mod player;
mod profile;
mod trailer;
mod truck;

/// A save edit applied through [`apply`].
pub enum Command {
    /// Repairs every wear attribute on one truck.
    RepairTruck(String),
    /// Repairs every truck the player owns.
    RepairAllTrucks,
    /// Sets one truck's fuel level, from 0 to 1.
    SetFuel {
        /// Truck unit id.
        id: String,
        /// Fuel level, from 0 to 1.
        level: f32,
    },
    /// Fills every truck the player owns.
    FillAllTrucks,
    /// Fills one truck.
    SetInfiniteFuel(String),
    /// Sets one truck's odometer, in kilometers.
    SetOdometer {
        /// Truck unit id.
        id: String,
        /// Odometer in kilometers.
        km: u64,
    },
    /// Sets one truck's license plate to raw text.
    SetLicensePlate {
        /// Truck unit id.
        id: String,
        /// Raw plate text.
        plate: String,
    },
    /// Removes every badge accessory from one truck.
    RemoveBadge(String),
    /// Points the player's assigned vehicle slot at one truck.
    SetCurrentTruck(String),
    /// Moves the player's headquarters to the garage city holding one truck.
    SetHqTruck(String),
    /// Repairs every wear attribute on one trailer and its slaves.
    RepairTrailer(String),
    /// Repairs every trailer the player owns.
    RepairAllTrailers,
    /// Sets cargo mass on one trailer and its slaves.
    SetCargoMass {
        /// Trailer unit id.
        id: String,
        /// Cargo mass in kilograms.
        mass: f32,
    },
    /// Sets one trailer's license plate text on it and its slaves.
    SetTrailerLicensePlate {
        /// Trailer unit id.
        id: String,
        /// Raw plate text.
        plate: String,
    },
    /// Removes the country restrictions from one trailer's definition.
    UnlockInRestrictedAreas(String),
    /// Points the player's assigned vehicle slot at one trailer.
    SetCurrentTrailer(String),
    /// Sets the bank balance.
    SetMoney(i64),
    /// Sets the player's experience points.
    SetExperience(u64),
    /// Sets the player's skill levels.
    SetSkills(Skills),
    /// Marks every city visited, or clears the visited list.
    SetVisitedCities(bool),
    /// Marks every dealership discovered, or clears the list.
    SetDealershipsDiscovered(bool),
    /// Sets one garage's status, expanding or clearing its slots.
    SetGarageStatus {
        /// Garage unit id.
        id: String,
        /// Status the game writes: 1, 2, 3, or 6.
        status: u64,
    },
    /// Sets the assigned vehicle slot's placement.
    SetPlayerPosition {
        /// Location vector text, parentheses included.
        location: String,
        /// Rotation vector text, parentheses included.
        rotation: String,
    },
    /// Renames the profile in a `profile.sii` document.
    RenameProfile(String),
}

/// Applies one command to a save document.
pub fn apply(document: &mut Document<'_>, _game: Game, command: Command) -> Result<(), Error> {
    match command {
        Command::RepairTruck(id) => truck::repair_truck(document, &id),
        Command::RepairAllTrucks => truck::repair_all_trucks(document),
        Command::SetFuel { id, level } => truck::set_fuel(document, &id, level),
        Command::FillAllTrucks => truck::fill_all_trucks(document),
        Command::SetInfiniteFuel(id) => truck::set_infinite_fuel(document, &id),
        Command::SetOdometer { id, km } => truck::set_odometer(document, &id, km),
        Command::SetLicensePlate { id, plate } => truck::set_license_plate(document, &id, plate),
        Command::RemoveBadge(id) => truck::remove_badge(document, &id),
        Command::SetCurrentTruck(id) => truck::set_current_truck(document, &id),
        Command::SetHqTruck(id) => truck::set_hq_truck(document, &id),
        Command::RepairTrailer(id) => trailer::repair_trailer(document, &id),
        Command::RepairAllTrailers => trailer::repair_all_trailers(document),
        Command::SetCargoMass { id, mass } => trailer::set_cargo_mass(document, &id, mass),
        Command::SetTrailerLicensePlate { id, plate } => {
            trailer::set_license_plate(document, &id, plate)
        }
        Command::UnlockInRestrictedAreas(id) => trailer::unlock_in_restricted_areas(document, &id),
        Command::SetCurrentTrailer(id) => trailer::set_current_trailer(document, &id),
        Command::SetMoney(money) => economy::set_money(document, money),
        Command::SetExperience(experience) => economy::set_experience(document, experience),
        Command::SetSkills(skills) => economy::set_skills(document, skills),
        Command::SetVisitedCities(all) => economy::set_visited_cities(document, all),
        Command::SetDealershipsDiscovered(all) => economy::set_dealerships(document, all),
        Command::SetGarageStatus { id, status } => {
            economy::set_garage_status(document, &id, status)
        }
        Command::SetPlayerPosition { location, rotation } => {
            player::set_position(document, &location, &rotation)
        }
        Command::RenameProfile(name) => profile::rename(document, name),
    }
}
