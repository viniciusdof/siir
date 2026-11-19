//! Save commands that edit a document.

use siir::{Document, Unit, Value};

use crate::Game;
use crate::Skills;
use crate::error::Error;
use crate::names;
use crate::value::{length, optional_array};

mod bus;
mod car;
mod driver;
mod economy;
mod job;
mod offence;
mod offer;
mod oversize;
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
    /// Sets the player's experience points; the game stores them in 32 bits.
    SetExperience(u32),
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
    /// Hires an unemployed AI driver into a garage slot.
    HireDriver {
        /// Driver unit id.
        id: String,
        /// Garage unit id with a free driver slot.
        garage: String,
    },
    /// Fires a hired AI driver.
    FireDriver(String),
    /// Pairs a driver with a truck in the same garage.
    SetDriverTruck {
        /// Driver unit id.
        id: String,
        /// Vehicle unit id.
        truck: String,
    },
    /// Sets one oversize offer's required truck.
    SetOversizeTruck {
        /// Offer unit id.
        id: String,
        /// Truck definition the offer requires.
        truck: String,
    },
    /// Sets one oversize offer's expiration time.
    SetOversizeExpiration {
        /// Offer unit id.
        id: String,
        /// Game time the offer expires.
        time: u64,
    },
    /// Marks every bus stop discovered.
    DiscoverBusStops,
    /// Selects one bus line offer for the player.
    SetBusLineOffer(String),
    /// Adds one vehicle to the player's owned buses.
    SetOwnedBus(String),
    /// Adds one vehicle to the player's owned cars.
    SetOwnedCar(String),
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
        Command::HireDriver { id, garage } => driver::hire(document, &id, &garage),
        Command::FireDriver(id) => driver::fire(document, &id),
        Command::SetDriverTruck { id, truck } => driver::set_truck(document, &id, &truck),
        Command::SetOversizeTruck { id, truck } => oversize::set_truck(document, &id, &truck),
        Command::SetOversizeExpiration { id, time } => {
            oversize::set_expiration(document, &id, time)
        }
        Command::DiscoverBusStops => bus::discover(document),
        Command::SetBusLineOffer(id) => bus::set_offer(document, &id),
        Command::SetOwnedBus(id) => bus::set_owned(document, &id),
        Command::SetOwnedCar(id) => car::set_owned(document, &id),
    }
}

/// Checks that a unit exists with the expected class.
pub(super) fn check(document: &Document<'_>, id: &str, class: &str) -> Result<(), Error> {
    match document.units().find(|unit| unit.id() == id) {
        Some(unit) if unit.class() == class => Ok(()),
        _ => Err(Error::MissingUnit { id: id.to_owned() }),
    }
}

/// Points one player attribute at a unit of the expected class.
pub(super) fn set_player_ref(
    document: &mut Document<'_>,
    name: &str,
    id: &str,
    class: &str,
) -> Result<(), Error> {
    check(document, id, class)?;
    let player = player_id(document)?;
    edit(document, &player, names::PLAYER)?.set(name, Value::token(id))?;
    Ok(())
}

/// The first unit id of a class.
pub(super) fn first_id(document: &Document<'_>, class: &str) -> Result<String, Error> {
    document
        .units_of_class(class)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or_else(|| Error::MissingUnit {
            id: class.to_owned(),
        })
}

/// The player unit id.
pub(super) fn player_id(document: &Document<'_>) -> Result<String, Error> {
    first_id(document, names::PLAYER)
}

/// Adds one vehicle to an owned list after checking its plate type.
pub(super) fn add_owned(
    document: &mut Document<'_>,
    name: &str,
    id: &str,
    kind: &'static str,
) -> Result<(), Error> {
    {
        let unit = document
            .units()
            .find(|unit| unit.id() == id && unit.class() == names::VEHICLE)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        if unit.get("license_plate_type") != Some(kind) {
            return Err(Error::InvalidVehicleType {
                id: id.to_owned(),
                expected: kind,
            });
        }
    }
    let player = player_id(document)?;
    let unit = edit(document, &player, names::PLAYER)?;
    if values(unit, name)?.iter().any(|item| item == id) {
        return Ok(());
    }
    push(unit, name, id)
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

/// Replaces an implicit array, adding, resizing, or removing its count.
pub(super) fn rewrite(unit: &mut Unit<'_>, name: &str, values: Vec<Value>) -> Result<(), Error> {
    let old = match unit.get(name) {
        Some(_) => length(unit, name)?,
        None => 0,
    };
    let len = values.len();
    if unit.get(name).is_some() {
        unit.set(name, Value::UInt(len as u64))?;
    } else {
        unit.insert_attr(name, Value::UInt(len as u64))?;
    }
    for (nth, value) in values.into_iter().enumerate() {
        let attr = names::indexed(name, nth);
        if unit.get(&attr).is_some() {
            unit.set(&attr, value)?;
        } else {
            unit.insert_attr(&attr, value)?;
        }
    }
    for nth in len..old {
        let attr = names::indexed(name, nth);
        if unit.get(&attr).is_some() {
            unit.remove_attr(&attr)?;
        }
    }
    Ok(())
}

/// An implicit array as owned strings, empty when the attribute is absent.
pub(super) fn values(unit: &Unit<'_>, name: &str) -> Result<Vec<String>, Error> {
    Ok(optional_array(unit, name)?
        .into_iter()
        .map(str::to_owned)
        .collect())
}

pub(super) fn tokens(values: Vec<String>) -> Vec<Value> {
    values.into_iter().map(Value::Token).collect()
}

/// Appends one value to an implicit array.
pub(super) fn push(unit: &mut Unit<'_>, name: &str, value: &str) -> Result<(), Error> {
    let mut list = values(unit, name)?;
    list.push(value.to_owned());
    rewrite(unit, name, tokens(list))
}
