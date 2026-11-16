//! Save commands that edit a document.

use siir::Document;

use crate::Game;
use crate::error::Error;

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
    }
}
