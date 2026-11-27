//! Owned rows the views render, built from a borrowed [`Save`].

use std::collections::{BTreeMap, HashMap, HashSet};

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
    /// Every `driver_ai` unit, employed ones first.
    pub drivers: Vec<DriverRow>,
    /// Every `garage` unit, active ones first.
    pub garages: Vec<GarageRow>,
    /// Police offense log summary, when the save has one.
    pub offense: Option<OffenseView>,
}

/// Employment state of an AI driver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DriverStatus {
    /// Hired by the player and not dismissed.
    Employed,
    /// Fired while on a job; can be hired back into a free slot.
    Dismissed,
    /// Waiting in the recruitment pool.
    Available,
    /// Not listed in the player's roster, dismissed list, or pool.
    Other,
}

impl DriverStatus {
    /// Whether `Command::HireDriver` accepts this driver.
    pub fn can_hire(self) -> bool {
        matches!(self, Self::Available | Self::Dismissed)
    }

    fn sort_key(self) -> u8 {
        match self {
            Self::Employed => 0,
            Self::Dismissed => 1,
            Self::Available => 2,
            Self::Other => 3,
        }
    }
}

/// One `driver_ai` unit as the Drivers screen shows it.
pub struct DriverRow {
    /// Unit id, the key every driver command takes.
    pub id: String,
    /// Employment status from the player and economy rosters.
    pub status: DriverStatus,
    /// Hometown city token, empty when unassigned.
    pub hometown: String,
    /// Experience points.
    pub experience_points: u64,
    /// Skill levels and ADR bitmask.
    pub skills: Skills,
    /// Garage unit id holding this driver, when assigned to one.
    pub garage_id: Option<String>,
    /// Garage city token holding this driver, when assigned to one.
    pub garage_city: Option<String>,
    /// Truck unit id sharing this driver's garage slot, or `assigned_truck`.
    pub truck: Option<String>,
    /// Whether the driver's current job has an active cargo.
    pub on_job: bool,
}

/// One `garage` unit as the Garages screen shows it.
pub struct GarageRow {
    /// Unit id, the key `SetGarageStatus` and `HireDriver` take.
    pub id: String,
    /// City token stripped from `garage.<city>`.
    pub city: String,
    /// Garage status stored in the save (`0`, `1`, `2`, `3`, or `6`).
    pub status: u64,
    /// Vehicle unit ids by garage slot, `None` for an empty slot.
    pub vehicles: Vec<Option<String>>,
    /// Driver unit ids by garage slot, `None` for an empty slot.
    pub drivers: Vec<Option<String>>,
    /// Trailer unit ids by garage slot, `None` for an empty slot.
    pub trailers: Vec<Option<String>>,
}

impl GarageRow {
    /// Whether the garage holds any vehicle, driver, or trailer.
    pub fn occupied(&self) -> bool {
        self.vehicles.iter().any(Option::is_some)
            || self.drivers.iter().any(Option::is_some)
            || self.trailers.iter().any(Option::is_some)
    }

    /// Number of empty driver slots available for `Command::HireDriver`.
    ///
    /// Unowned garages (`status` 0 or 1) return 0 even if stale `null` slots
    /// remain in the save.
    pub fn free_driver_slots(&self) -> usize {
        if !matches!(self.status, 2 | 3 | 6) {
            return 0;
        }
        self.drivers.iter().filter(|slot| slot.is_none()).count()
    }
}

/// Police offense summary as the Offenses screen shows it.
pub struct OffenseView {
    /// Offense counts per category.
    pub counts: Vec<u64>,
    /// Fines per category, in the save's stored currency.
    pub fines: Vec<i64>,
    /// Number of detailed history entries.
    pub history_entries: u64,
}

impl OffenseView {
    /// Sum of offenses across every category.
    pub fn total_count(&self) -> u64 {
        self.counts.iter().copied().sum()
    }

    /// Sum of fines across every category.
    pub fn total_fines(&self) -> i64 {
        self.fines.iter().copied().sum()
    }

    /// Whether counts, fines, and history are all zero.
    pub fn is_clear(&self) -> bool {
        self.total_count() == 0 && self.total_fines() == 0 && self.history_entries == 0
    }
}

/// One `trailer` unit as the Trailers screen shows it.
pub struct TrailerRow {
    /// Unit id, the key every trailer command takes.
    pub id: String,
    /// Trailer definition the unit uses.
    pub definition: String,
    /// Plate text without markup or country.
    pub plate: String,
    /// Country after `|`, when the plate names one.
    pub country: Option<String>,
    /// Attached to the player's current truck.
    pub current: bool,
    /// Cargo mass in kilograms.
    pub cargo_mass: f32,
    /// `body_type` of a definition stored in the save.
    pub body_type: Option<String>,
    /// `source_name` of that definition.
    pub source: Option<String>,
    /// Whether the inline trailer definition restricts allowed countries.
    pub restricted: bool,
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
    /// Garage city holding this vehicle, when a garage lists it.
    pub garage: Option<String>,
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
            garage: vehicle.garage.map(str::to_owned),
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

        let employed: HashSet<&str> = player
            .map(|player| player.drivers.iter().copied().collect())
            .unwrap_or_default();
        let dismissed: HashSet<&str> = player
            .map(|player| player.dismissed_drivers.iter().copied().collect())
            .unwrap_or_default();
        let pool: HashSet<&str> = save.driver_pool.iter().copied().collect();
        let active_jobs: HashSet<&str> = save
            .jobs
            .iter()
            .filter(|job| job.cargo.is_some())
            .map(|job| job.id)
            .collect();

        let mut driver_garages: HashMap<&str, (&str, &str, Option<&str>)> = HashMap::new();
        let mut garages: Vec<GarageRow> = save
            .garages
            .iter()
            .map(|garage| {
                let city = garage.id.strip_prefix("garage.").unwrap_or(garage.id);
                for (slot, driver) in garage.drivers.iter().enumerate() {
                    if let Some(driver_id) = driver {
                        let slot_truck = garage.vehicles.get(slot).copied().flatten();
                        driver_garages
                            .entry(driver_id)
                            .or_insert((garage.id, city, slot_truck));
                    }
                }
                GarageRow {
                    id: garage.id.to_owned(),
                    city: city.to_owned(),
                    status: garage.status,
                    vehicles: garage
                        .vehicles
                        .iter()
                        .map(|slot| slot.map(str::to_owned))
                        .collect(),
                    drivers: garage
                        .drivers
                        .iter()
                        .map(|slot| slot.map(str::to_owned))
                        .collect(),
                    trailers: garage
                        .trailers
                        .iter()
                        .map(|slot| slot.map(str::to_owned))
                        .collect(),
                }
            })
            .collect();
        garages.sort_by(|left, right| {
            (left.status == 0)
                .cmp(&(right.status == 0))
                .then_with(|| left.city.cmp(&right.city))
        });

        let mut drivers: Vec<DriverRow> = save
            .drivers
            .iter()
            .map(|driver| {
                let status = if dismissed.contains(driver.id) {
                    DriverStatus::Dismissed
                } else if employed.contains(driver.id) {
                    DriverStatus::Employed
                } else if pool.contains(driver.id) {
                    DriverStatus::Available
                } else {
                    DriverStatus::Other
                };
                let (garage_id, garage_city, slot_truck) = driver_garages
                    .get(driver.id)
                    .copied()
                    .map(|(id, city, truck)| (Some(id.to_owned()), Some(city.to_owned()), truck))
                    .unwrap_or((None, None, None));
                let truck = if garage_id.is_some() {
                    slot_truck
                } else {
                    driver.assigned_truck
                }
                .map(str::to_owned);
                let on_job = driver.job.is_some_and(|job| active_jobs.contains(job));
                DriverRow {
                    id: driver.id.to_owned(),
                    status,
                    hometown: driver.hometown.to_owned(),
                    experience_points: driver.experience_points,
                    skills: driver.skills,
                    garage_id,
                    garage_city,
                    truck,
                    on_job,
                }
            })
            .collect();
        drivers.sort_by_key(|driver| driver.status.sort_key());
        let hired_drivers = drivers
            .iter()
            .filter(|driver| driver.status == DriverStatus::Employed)
            .count();
        let active_garages = garages.iter().filter(|garage| garage.status != 0).count();
        let offense = save.offense.as_ref().map(|offense| OffenseView {
            counts: offense.counts.clone(),
            fines: offense.fines.clone(),
            history_entries: offense.history_entries,
        });

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
                drivers: hired_drivers,
                garages: active_garages,
                offers: save.offers.len(),
            },
            vehicles,
            trailers: save
                .trailers
                .iter()
                .map(|trailer| {
                    let plate = LicensePlate::parse(trailer.license_plate);
                    TrailerRow {
                        id: trailer.id.to_owned(),
                        definition: trailer.definition.to_owned(),
                        cargo_mass: trailer.cargo_mass,
                        body_type: trailer.body_type.map(str::to_owned),
                        source: trailer.source.map(str::to_owned),
                        restricted: !trailer.country_validity.is_empty(),
                        plate: plate.text,
                        country: plate.country.map(str::to_owned),
                        current: current_trailer == Some(trailer.id),
                        body_wear: trailer.body_wear,
                        chassis_wear: trailer.chassis_wear,
                    }
                })
                .collect(),
            drivers,
            garages,
            offense,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use siir_save::{Game, Session};

    #[test]
    fn builds_driver_garage_and_offense_rows() {
        let source = include_str!("../../siir-save/testdata/read.sii")
            .replace(
                "    unlocked_dealers: 0\n",
                "    unlocked_dealers: 0\n    driver_pool: 1\n    driver_pool[0]: driver.pool\n",
            )
            .replace(
                "    assigned_vehicles: slot.one\n",
                "    assigned_vehicles: slot.one\n    drivers: 2\n    drivers[0]: driver_player.one\n    drivers[1]: driver.one\n    dismissed_drivers: 0\n",
            )
            .replace(
                "garage : garage.one {\n    status: 2\n}",
                "garage : garage.prague {\n    status: 2\n    vehicles: 2\n    vehicles[0]: vehicle.one\n    vehicles[1]: null\n    drivers: 2\n    drivers[0]: driver.one\n    drivers[1]: null\n    trailers: 2\n    trailers[0]: null\n    trailers[1]: null\n}\ngarage : garage.brno {\n    status: 0\n    vehicles: 0\n    drivers: 0\n    trailers: 0\n}",
            )
            .replace(
                "    assigned_truck: vehicle.one\n}",
                "    assigned_truck: vehicle.one\n    state: 2\n}\ndriver_ai : driver.pool {\n    experience_points: 0\n    adr: 0\n    long_dist: 0\n    heavy: 0\n    fragile: 0\n    urgent: 0\n    mechanical: 0\n    hometown: \"\"\n    assigned_truck: null\n    driver_job: null\n    training_policy: 0\n    state: 1\n}",
            );
        let session = Session::from_bytes(Game::Ets2, source.into_bytes()).unwrap();
        let save = session.read().unwrap();
        let view = SaveView::new(&save, "save".into(), "profile".into());

        assert_eq!(view.counts.drivers, 1);
        assert_eq!(view.counts.garages, 1);

        assert_eq!(view.drivers.len(), 2);
        assert_eq!(view.drivers[0].id, "driver.one");
        assert_eq!(view.drivers[0].status, DriverStatus::Employed);
        assert_eq!(view.drivers[0].garage_id.as_deref(), Some("garage.prague"));
        assert_eq!(view.drivers[0].garage_city.as_deref(), Some("prague"));
        assert_eq!(view.drivers[0].truck.as_deref(), Some("vehicle.one"));
        assert!(!view.drivers[0].on_job);

        assert_eq!(view.drivers[1].id, "driver.pool");
        assert_eq!(view.drivers[1].status, DriverStatus::Available);
        assert!(view.drivers[1].status.can_hire());
        assert_eq!(view.drivers[1].garage_id, None);

        assert_eq!(view.garages.len(), 2);
        assert_eq!(view.garages[0].city, "prague");
        assert!(view.garages[0].occupied());
        assert_eq!(view.garages[0].free_driver_slots(), 1);
        assert_eq!(view.garages[1].city, "brno");
        assert!(!view.garages[1].occupied());
        assert_eq!(view.garages[1].free_driver_slots(), 0);

        let buyable = GarageRow {
            id: "garage.buyable".into(),
            city: "buyable".into(),
            status: 1,
            vehicles: vec![None],
            drivers: vec![None],
            trailers: vec![None],
        };
        assert_eq!(buyable.free_driver_slots(), 0);

        let offense = view.offense.as_ref().unwrap();
        assert_eq!(offense.total_count(), 9);
        assert_eq!(offense.total_fines(), 1300);
        assert_eq!(offense.history_entries, 3);
        assert!(!offense.is_clear());
    }

    #[test]
    fn empty_garage_slot_does_not_fall_back_to_assigned_truck() {
        let source = include_str!("../../siir-save/testdata/read.sii")
            .replace(
                "    assigned_vehicles: slot.one\n",
                "    assigned_vehicles: slot.one\n    drivers: 2\n    drivers[0]: driver_player.one\n    drivers[1]: driver.one\n    dismissed_drivers: 0\n",
            )
            .replace(
                "garage : garage.one {\n    status: 2\n}",
                "garage : garage.prague {\n    status: 2\n    vehicles: 2\n    vehicles[0]: null\n    vehicles[1]: vehicle.one\n    drivers: 2\n    drivers[0]: driver.one\n    drivers[1]: null\n    trailers: 2\n    trailers[0]: null\n    trailers[1]: null\n}",
            );
        let session = Session::from_bytes(Game::Ets2, source.into_bytes()).unwrap();
        let save = session.read().unwrap();
        let view = SaveView::new(&save, "save".into(), "profile".into());

        let driver = view.drivers.iter().find(|d| d.id == "driver.one").unwrap();
        assert_eq!(driver.garage_id.as_deref(), Some("garage.prague"));
        assert_eq!(driver.truck, None);
    }
}
