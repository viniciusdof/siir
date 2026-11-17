//! Save commands that edit a document.

use siir::{Document, Unit};

use crate::Game;
use crate::Skills;
use crate::error::Error;
use crate::names;

mod driver;
mod economy;
mod job;
mod offence;
mod offer;
mod plate;
mod player;
mod profile;
mod trailer;
mod used;
mod vehicle;

/// A save edit applied through [`apply`].
pub enum Command {
    /// Repairs every wear attribute on one vehicle.
    RepairVehicle(String),
    /// Repairs every vehicle the player owns.
    RepairAllVehicles,
    /// Sets one vehicle's fuel level, from 0 to 1.
    SetFuel {
        /// Vehicle unit id.
        id: String,
        /// Fuel level, from 0 to 1.
        level: f32,
    },
    /// Fills every vehicle the player owns.
    FillAllVehicles,
    /// Fills one vehicle.
    SetInfiniteFuel(String),
    /// Sets one vehicle's odometer, in kilometers.
    SetOdometer {
        /// Vehicle unit id.
        id: String,
        /// Odometer in kilometers.
        km: u64,
    },
    /// Sets one vehicle's license plate to raw text.
    SetLicensePlate {
        /// Vehicle unit id.
        id: String,
        /// Raw plate text.
        plate: String,
    },
    /// Removes every badge accessory from one vehicle.
    RemoveBadge(String),
    /// Sets one vehicle's engine `data_path`.
    SetEngine {
        /// Vehicle unit id.
        id: String,
        /// Engine `data_path` from the catalog.
        path: String,
    },
    /// Sets one vehicle's transmission `data_path`.
    SetTransmission {
        /// Vehicle unit id.
        id: String,
        /// Transmission `data_path` from the catalog.
        path: String,
    },
    /// Sets one vehicle's cabin `data_path`.
    SetCabin {
        /// Vehicle unit id.
        id: String,
        /// Cabin `data_path` from the catalog.
        path: String,
    },
    /// Sets one vehicle's chassis `data_path`.
    SetChassis {
        /// Vehicle unit id.
        id: String,
        /// Chassis `data_path` from the catalog.
        path: String,
    },
    /// Sets one vehicle's paint job `data_path`.
    SetPaint {
        /// Vehicle unit id.
        id: String,
        /// Paint job `data_path` from the catalog.
        path: String,
    },
    /// Sets one accessory's `data_path`.
    SetAccessory {
        /// Vehicle unit id.
        id: String,
        /// Accessory unit id.
        accessory: String,
        /// Accessory `data_path`.
        path: String,
    },
    /// Removes one accessory from a vehicle's accessory list.
    RemoveAccessory {
        /// Vehicle unit id.
        id: String,
        /// Accessory unit id.
        accessory: String,
    },
    /// Sets one used-truck listing's price.
    SetUsedTruckPrice {
        /// Listing unit id.
        id: String,
        /// Asking price.
        price: i64,
    },
    /// Points the player's assigned vehicle slot at one vehicle.
    SetCurrentVehicle(String),
    /// Moves the player's headquarters to the garage city holding one vehicle.
    SetHqVehicle(String),
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
    /// Sets one freight market offer's cargo unit count.
    SetOfferUnits {
        /// Offer unit id.
        id: String,
        /// Units of cargo.
        units: u64,
    },
    /// Sets one freight market offer's cargo definition.
    SetOfferCargo {
        /// Offer unit id.
        id: String,
        /// Cargo `data_path` token, for example `cargo.plast_film`.
        cargo: String,
    },
    /// Sets one freight market offer's target company.
    SetOfferTarget {
        /// Offer unit id.
        id: String,
        /// Target company token, for example `transinet.ostrava`.
        target: String,
    },
    /// Sets one freight market offer's shortest distance.
    SetOfferDistance {
        /// Offer unit id.
        id: String,
        /// Shortest road distance in kilometers.
        km: u64,
    },
    /// Sets one freight market offer's fill ratio.
    SetOfferFill {
        /// Offer unit id.
        id: String,
        /// Trailer fill ratio, from 0 to 1.
        fill: f32,
    },
    /// Sets one freight market offer's urgency class.
    SetOfferUrgency {
        /// Offer unit id.
        id: String,
        /// Urgency class the game stores.
        urgency: u64,
    },
    /// Sets one freight market offer's expiration time.
    SetOfferExpiration {
        /// Offer unit id.
        id: String,
        /// Game time the offer expires.
        time: u64,
    },
    /// Sets one AI driver's skill levels.
    SetDriverSkills {
        /// Driver unit id.
        id: String,
        /// Skill levels to write.
        skills: Skills,
    },
    /// Zeroes the police offence counts and fines.
    ClearOffences,
    /// Sets the cargo a player's current job hauls.
    SetCurrentJobCargo(String),
    /// Sets a vehicle's or trailer's license plate country.
    SetLicensePlateCountry {
        /// Vehicle or trailer unit id.
        id: String,
        /// Country name after `|`.
        country: String,
    },
    /// Sets one used-truck listing's expiration time.
    SetUsedTruckExpiration {
        /// Listing unit id.
        id: String,
        /// Game time the listing expires.
        time: u64,
    },
    /// Sets when the next used-truck listing generation is due.
    SetUsedAssortmentNextGeneration(u64),
}

/// Applies one command to a save document.
pub fn apply(document: &mut Document<'_>, _game: Game, command: Command) -> Result<(), Error> {
    match command {
        Command::RepairVehicle(id) => vehicle::repair_vehicle(document, &id),
        Command::RepairAllVehicles => vehicle::repair_all_vehicles(document),
        Command::SetFuel { id, level } => vehicle::set_fuel(document, &id, level),
        Command::FillAllVehicles => vehicle::fill_all_vehicles(document),
        Command::SetInfiniteFuel(id) => vehicle::set_infinite_fuel(document, &id),
        Command::SetOdometer { id, km } => vehicle::set_odometer(document, &id, km),
        Command::SetLicensePlate { id, plate } => vehicle::set_license_plate(document, &id, plate),
        Command::RemoveBadge(id) => vehicle::remove_badge(document, &id),
        Command::SetEngine { id, path } => vehicle::set_engine(document, &id, &path),
        Command::SetTransmission { id, path } => vehicle::set_transmission(document, &id, &path),
        Command::SetCabin { id, path } => vehicle::set_cabin(document, &id, &path),
        Command::SetChassis { id, path } => vehicle::set_chassis(document, &id, &path),
        Command::SetPaint { id, path } => vehicle::set_paint(document, &id, &path),
        Command::SetAccessory {
            id,
            accessory,
            path,
        } => vehicle::set_accessory(document, &id, &accessory, &path),
        Command::RemoveAccessory { id, accessory } => {
            vehicle::remove_accessory(document, &id, &accessory)
        }
        Command::SetUsedTruckPrice { id, price } => used::set_price(document, &id, price),
        Command::SetCurrentVehicle(id) => vehicle::set_current_vehicle(document, &id),
        Command::SetHqVehicle(id) => vehicle::set_hq_vehicle(document, &id),
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
        Command::SetOfferUnits { id, units } => offer::set_units(document, &id, units),
        Command::SetOfferCargo { id, cargo } => offer::set_cargo(document, &id, cargo),
        Command::SetOfferTarget { id, target } => offer::set_target(document, &id, target),
        Command::SetOfferDistance { id, km } => offer::set_distance(document, &id, km),
        Command::SetOfferFill { id, fill } => offer::set_fill(document, &id, fill),
        Command::SetOfferUrgency { id, urgency } => offer::set_urgency(document, &id, urgency),
        Command::SetOfferExpiration { id, time } => offer::set_expiration(document, &id, time),
        Command::SetDriverSkills { id, skills } => driver::set_skills(document, &id, skills),
        Command::ClearOffences => offence::clear(document),
        Command::SetCurrentJobCargo(cargo) => job::set_current_cargo(document, cargo),
        Command::SetLicensePlateCountry { id, country } => {
            plate::set_country(document, &id, &country)
        }
        Command::SetUsedTruckExpiration { id, time } => used::set_expiration(document, &id, time),
        Command::SetUsedAssortmentNextGeneration(time) => {
            used::set_assortment_generation(document, time)
        }
    }
}

/// Finds one editable unit by id and class.
fn edit<'a, 'd>(
    document: &'a mut Document<'d>,
    id: &str,
    class: &str,
) -> Result<&'a mut Unit<'d>, Error> {
    match document.get(id) {
        Some(unit) if unit.class() == class && unit.id() == id => Ok(unit),
        _ => Err(Error::MissingUnit { id: id.to_owned() }),
    }
}

/// Writes the six skill attributes.
fn write_skills(unit: &mut Unit<'_>, skills: Skills) -> Result<(), Error> {
    unit.set(names::ADR, siir::Value::UInt(skills.adr))?;
    unit.set(names::LONG_DIST, siir::Value::UInt(skills.long_dist))?;
    unit.set(names::HEAVY, siir::Value::UInt(skills.heavy))?;
    unit.set(names::FRAGILE, siir::Value::UInt(skills.fragile))?;
    unit.set(names::URGENT, siir::Value::UInt(skills.urgent))?;
    unit.set(names::MECHANICAL, siir::Value::UInt(skills.mechanical))?;
    Ok(())
}
