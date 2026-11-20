//! Owned rows the views render, built from a borrowed [`Save`].

use std::collections::BTreeMap;

use siir_save::{LicensePlate, Save, Skills, UsedTruck, Vehicle};

/// Headline values and counts of one open save.
pub struct SaveView {
    /// Save name from `info.sii`.
    pub save_name: String,
    /// Name of the profile that holds the save.
    pub profile_name: String,
    /// Bank balance, when the save has a bank.
    pub money: Option<i64>,
    /// Total experience points.
    pub experience: u64,
    /// Player skill levels.
    pub skills: Skills,
    /// How many cities the player has visited.
    pub visited_cities: usize,
    /// How many dealerships the player has discovered.
    pub dealerships: usize,
    /// Headquarters city.
    pub hq_city: String,
    /// Model folder of the player's current vehicle, when it has a data path.
    pub current_vehicle: Option<String>,
    /// Owned fleet and market sizes.
    pub counts: Counts,
    /// Every `vehicle` unit, owned ones first.
    pub vehicles: Vec<VehicleRow>,
    /// Every `trailer` unit.
    pub trailers: Vec<TrailerRow>,
}

/// One `trailer` unit as the Trailers screen shows it.
pub struct TrailerRow {
    /// Unit id, the key every trailer command takes.
    pub id: String,
    /// Trailer definition the unit uses.
    pub definition: String,
    /// Plate text without markup or country.
    pub plate: String,
    /// Attached to the player's current truck.
    pub current: bool,
    /// Cargo mass in kilograms.
    pub cargo_mass: f32,
    /// `body_type` of a definition stored in the save.
    pub body_type: Option<String>,
    /// `source_name` of that definition.
    pub source: Option<String>,
    /// Body wear, from 0 to 1.
    pub body_wear: f32,
    /// Chassis wear, from 0 to 1.
    pub chassis_wear: f32,
}

/// One `vehicle` unit as the Vehicles screen shows it.
pub struct VehicleRow {
    /// Unit id, the key every vehicle command takes.
    pub id: String,
    /// Model folder, for the catalog's game name.
    pub model: Option<String>,
    /// Engine `data_path`.
    pub engine: Option<String>,
    /// Transmission `data_path`.
    pub transmission: Option<String>,
    /// Cabin `data_path`.
    pub cabin: Option<String>,
    /// Chassis `data_path`.
    pub chassis: Option<String>,
    /// Plate text without markup or country.
    pub plate: String,
    /// Country after `|`, when the plate names one.
    pub country: Option<String>,
    /// Owned by the player, so Repair all and Fill all reach it.
    pub owned: bool,
    /// The truck the player drives.
    pub current: bool,
    /// Engine wear, from 0 to 1.
    pub engine_wear: f32,
    /// Transmission wear, from 0 to 1.
    pub transmission_wear: f32,
    /// Cabin wear, from 0 to 1.
    pub cabin_wear: f32,
    /// Fuel level, from 0 to 1.
    pub fuel: f32,
    /// Odometer in kilometers, when the unit stores one.
    pub odometer: Option<u64>,
    /// Whether any accessory is a badge `RemoveBadge` would remove.
    pub badges: bool,
    /// The used-market listing that offers this vehicle, when one does.
    pub listing: Option<Listing>,
}

/// One used-market listing.
pub struct Listing {
    /// Listing unit id, the key `SetUsedTruckPrice` takes.
    pub id: String,
    /// Asking price.
    pub price: i64,
}

impl VehicleRow {
    fn new(vehicle: &Vehicle<'_>, owned: bool, current: bool, listing: Option<Listing>) -> Self {
        let plate = LicensePlate::parse(vehicle.license_plate);
        Self {
            id: vehicle.id.to_owned(),
            model: vehicle.model.map(str::to_owned),
            engine: vehicle.engine.map(str::to_owned),
            transmission: vehicle.transmission.map(str::to_owned),
            cabin: vehicle.cabin.map(str::to_owned),
            chassis: vehicle.chassis.map(str::to_owned),
            plate: plate.text,
            country: plate.country.map(str::to_owned),
            owned,
            current,
            engine_wear: vehicle.engine_wear,
            transmission_wear: vehicle.transmission_wear,
            cabin_wear: vehicle.cabin_wear,
            fuel: vehicle.fuel_relative,
            odometer: vehicle.odometer,
            badges: vehicle
                .accessories
                .iter()
                .any(|accessory| accessory.is_badge()),
            listing,
        }
    }
}

/// Owned fleet and market sizes shown on the overview.
#[derive(Clone, Copy, Default)]
pub struct Counts {
    /// Vehicles the player owns, by `Save::owns`: the same set the Vehicles
    /// screen lists as owned.
    pub vehicles: usize,
    /// Owned trailer unit ids.
    pub trailers: usize,
    /// Hired AI drivers.
    pub drivers: usize,
    /// Garages with a non-zero status.
    pub garages: usize,
    /// Freight market offers.
    pub offers: usize,
}

impl SaveView {
    /// Copies the values the overview shows out of the borrowed save.
    pub fn new(save: &Save<'_>, save_name: String, profile_name: String) -> Self {
        let player = save.player.as_ref();
        let current = save.current_vehicle().map(|vehicle| vehicle.id);
        let current_trailer = save.current_trailer().map(|trailer| trailer.id);
        let listings: BTreeMap<&str, &UsedTruck<'_>> = save
            .used_trucks
            .iter()
            .map(|listing| (listing.truck, listing))
            .collect();
        let mut vehicles: Vec<VehicleRow> = save
            .vehicles
            .iter()
            .map(|vehicle| {
                let listing = listings.get(vehicle.id).map(|listing| Listing {
                    id: listing.id.to_owned(),
                    price: listing.price,
                });
                VehicleRow::new(
                    vehicle,
                    save.owns(vehicle.id),
                    current == Some(vehicle.id),
                    listing,
                )
            })
            .collect();
        vehicles.sort_by_key(|vehicle| !vehicle.owned);
        let owned = vehicles.iter().filter(|vehicle| vehicle.owned).count();
        Self {
            save_name,
            profile_name,
            money: save.bank.as_ref().map(|bank| bank.money_account),
            experience: save.experience,
            skills: save.skills,
            visited_cities: save.visited_cities.len(),
            dealerships: save.unlocked_dealers.len(),
            hq_city: save
                .player
                .as_ref()
                .map(|player| player.hq_city.to_owned())
                .unwrap_or_default(),
            current_vehicle: save
                .current_vehicle()
                .and_then(|vehicle| vehicle.model)
                .map(str::to_owned),
            counts: Counts {
                vehicles: owned,
                trailers: player
                    .map(|player| player.trailers.len())
                    .unwrap_or_default(),
                drivers: save.drivers.len(),
                garages: save
                    .garages
                    .iter()
                    .filter(|garage| garage.status != 0)
                    .count(),
                offers: save.offers.len(),
            },
            vehicles,
            trailers: save
                .trailers
                .iter()
                .map(|trailer| TrailerRow {
                    id: trailer.id.to_owned(),
                    definition: trailer.definition.to_owned(),
                    cargo_mass: trailer.cargo_mass,
                    body_type: trailer.body_type.map(str::to_owned),
                    source: trailer.source.map(str::to_owned),
                    plate: LicensePlate::parse(trailer.license_plate).text,
                    current: current_trailer == Some(trailer.id),
                    body_wear: trailer.body_wear,
                    chassis_wear: trailer.chassis_wear,
                })
                .collect(),
        }
    }
}
