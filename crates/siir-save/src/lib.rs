//! Typed read model and edits for SCS save documents.
//!
//! [`read`] borrows the document, [`apply`] edits it in place, and [`Session`]
//! owns one file for a front end.

use std::collections::HashMap;

use siir::{Document, Unit};

use accessory::{Part, classify};

pub mod accessory;
pub mod catalog;
mod color;
mod command;
pub mod config;
mod error;
mod model;
mod names;
mod plate;
pub mod profiles;
mod session;
mod value;

pub use accessory::{export_accessories, import_accessories};
pub use color::Color;
pub use command::{Command, apply};
pub use error::Error;
pub use model::{
    Accessory, Bank, BusJob, BusJobStage, BusLineOffer, BusStop, CarJob, CarOffer, Company, Driver,
    Game, Garage, Job, JobLog, Offence, Offer, OversizeOffer, OversizeRoute, Player, Save, Skills,
    Trailer, UsedAssortment, UsedTruck, Vehicle,
};
pub use plate::LicensePlate;
pub use session::Session;
pub use siir::container::Origin;

use value::{
    array, boolean, count, float, integer, optional_array, optional_count, reference, string, text,
    unquote,
};

/// Reads the typed view of a save document.
pub fn read<'a>(document: &'a Document<'_>, game: Game) -> Result<Save<'a>, Error> {
    let units: HashMap<&str, &Unit<'_>> = document.units().map(|unit| (unit.id(), unit)).collect();

    let economy = first(document, names::ECONOMY);
    let bank = linked(economy, names::BANK, &units).or_else(|| first(document, names::BANK));
    let player = linked(economy, names::PLAYER, &units).or_else(|| first(document, names::PLAYER));

    let (experience, skills, visited_cities, unlocked_dealers) = match economy {
        Some(unit) => (
            count(unit, names::EXPERIENCE_POINTS)?,
            read_skills(unit)?,
            array(unit, names::VISITED_CITIES)?,
            array(unit, names::UNLOCKED_DEALERS)?,
        ),
        None => (0, Skills::default(), Vec::new(), Vec::new()),
    };

    let assigned = player.and_then(|unit| reference(unit, names::ASSIGNED_VEHICLES));
    let mut slots: HashMap<&str, &Unit<'_>> = HashMap::new();
    for slot in document.units_of_class(names::PLAYER_VEHICLES) {
        let Some(vehicle) = reference(slot, names::VEHICLE) else {
            continue;
        };
        // Keep the assigned slot when several slots reference the same vehicle.
        if Some(slot.id()) == assigned || !slots.contains_key(vehicle) {
            slots.insert(vehicle, slot);
        }
    }

    let bank = match bank {
        Some(unit) => Some(Bank {
            money_account: integer(unit, "money_account")?,
        }),
        None => None,
    };

    let player = match player {
        Some(unit) => Some(Player {
            hq_city: text(unit, names::HQ_CITY)?,
            my_vehicles: array(unit, names::MY_VEHICLES)?,
            trucks: array(unit, names::TRUCKS)?,
            trailers: array(unit, names::TRAILERS)?,
            buses: optional_array(unit, names::BUSES)?,
            cars: optional_array(unit, names::CARS)?,
            assigned_vehicles: reference(unit, names::ASSIGNED_VEHICLES),
            current_job: reference(unit, "current_job"),
            current_bus_job: reference(unit, names::CURRENT_BUS_JOB),
            selected_bus_offer: reference(unit, names::SELECTED_BUS_OFFER),
            current_car_job: reference(unit, names::CURRENT_CAR_JOB),
        }),
        None => None,
    };

    let mut vehicles = Vec::new();
    for unit in document.units_of_class(names::VEHICLE) {
        let slot = slots.get(unit.id()).copied();
        let accessories = accessories(unit, &units)?;
        let parts = parts(&accessories);
        vehicles.push(Vehicle {
            id: unit.id(),
            slot: slot.map(|slot| slot.id()),
            trailer: slot.and_then(|slot| reference(slot, names::TRAILER)),
            accessories,
            model: parts.model,
            data_path: parts.data_path,
            engine: parts.engine,
            transmission: parts.transmission,
            cabin: parts.cabin,
            chassis: parts.chassis,
            paint_job: parts.paint_job,
            engine_wear: float(unit, "engine_wear")?,
            transmission_wear: float(unit, "transmission_wear")?,
            cabin_wear: float(unit, "cabin_wear")?,
            fuel_relative: float(unit, "fuel_relative")?,
            license_plate: string(unit, names::LICENSE_PLATE)?,
            placement: slot.and_then(|slot| slot.get(names::STORED_VEHICLE_PLACEMENT)),
        });
    }

    let mut trailers = Vec::new();
    for unit in document.units_of_class(names::TRAILER) {
        trailers.push(Trailer {
            id: unit.id(),
            definition: text(unit, names::TRAILER_DEFINITION)?,
            cargo_mass: float(unit, names::CARGO_MASS)?,
            license_plate: string(unit, names::LICENSE_PLATE)?,
            slave_trailer: reference(unit, names::SLAVE_TRAILER),
            body_wear: float(unit, names::TRAILER_BODY_WEAR)?,
            chassis_wear: float(unit, names::CHASSIS_WEAR)?,
        });
    }

    let mut garages = Vec::new();
    for unit in document.units_of_class(names::GARAGE) {
        garages.push(Garage {
            id: unit.id(),
            status: count(unit, names::STATUS)?,
        });
    }

    let mut drivers = Vec::new();
    for unit in document.units_of_class(names::DRIVER_AI) {
        drivers.push(Driver {
            id: unit.id(),
            experience_points: count(unit, "experience_points")?,
            skills: read_skills(unit)?,
            hometown: string(unit, names::HOMETOWN)?,
            assigned_truck: reference(unit, names::ASSIGNED_TRUCK),
            job: reference(unit, names::DRIVER_JOB),
            training_policy: count(unit, names::TRAINING_POLICY)?,
        });
    }

    let mut companies = Vec::new();
    for unit in document.units_of_class(names::COMPANY) {
        companies.push(Company {
            id: unit.id(),
            permanent_data: text(unit, "permanent_data")?,
        });
    }

    let mut jobs = Vec::new();
    for unit in document.units_of_class(names::JOB_INFO) {
        jobs.push(job(unit)?);
    }

    let player_job = first(document, names::PLAYER_JOB).map(job).transpose()?;

    let mut offers = Vec::new();
    for unit in document.units_of_class(names::JOB_OFFER_DATA) {
        offers.push(Offer {
            id: unit.id(),
            target: string(unit, "target")?,
            cargo: text(unit, "cargo")?,
            shortest_distance_km: count(unit, "shortest_distance_km")?,
            urgency: optional_count(unit, names::URGENCY)?,
            expiration_time: optional_count(unit, names::EXPIRATION_TIME)?,
            units_count: count(unit, names::UNITS_COUNT)?,
            fill_ratio: float(unit, names::FILL_RATIO)?,
            trailer_definition: text(unit, names::TRAILER_DEFINITION)?,
        });
    }

    let mut used_trucks = Vec::new();
    for unit in document.units_of_class(names::USED_TRUCK_OFFER) {
        used_trucks.push(UsedTruck {
            id: unit.id(),
            truck: text(unit, "truck")?,
            price: integer(unit, "price")?,
            expiration_game_time: count(unit, names::EXPIRATION_GAME_TIME)?,
        });
    }

    let used_assortment = match first(document, names::USED_VEHICLE_ASSORTMENT) {
        Some(unit) => Some(UsedAssortment {
            id: unit.id(),
            next_generation_game_time: count(unit, names::NEXT_GENERATION_GAME_TIME)?,
            trucks: array(unit, names::TRUCKS)?,
        }),
        None => None,
    };

    let offence = match first(document, names::POLICE_OFFENCE_LOG) {
        Some(unit) => Some(Offence {
            id: unit.id(),
            counts: numbers(unit, names::OFFENCE_TOTAL_COUNTS)?,
            fines: fines(unit, names::OFFENCE_TOTAL_FINES)?,
            history_entries: count(unit, names::DETAILED_HISTORY_ENTRIES)?,
        }),
        None => None,
    };

    let mut oversize_offers = Vec::new();
    for unit in document.units_of_class(names::OVERSIZE_OFFER) {
        oversize_offers.push(OversizeOffer {
            id: unit.id(),
            offer_data: text(unit, names::OFFER_DATA)?,
            truck: string(unit, names::TRUCK)?,
            expiration: count(unit, names::EXPIRATION)?,
        });
    }

    let mut oversize_routes = Vec::new();
    for unit in document.units_of_class(names::OVERSIZE_ROUTE_OFFERS) {
        oversize_routes.push(OversizeRoute {
            id: unit.id(),
            route: text(unit, names::ROUTE)?,
            offers: array(unit, names::OFFERS)?,
        });
    }

    // shortcut: bus, car, and log units are parsed strictly, so one malformed
    // unit fails the whole read; skip malformed units when older or modded
    // saves are supported.
    let mut bus_stops = Vec::new();
    for unit in document.units_of_class(names::BUS_STOP) {
        let city = unit
            .id()
            .strip_prefix("bus_stop.city.")
            .unwrap_or_else(|| unit.id());
        bus_stops.push(BusStop {
            id: unit.id(),
            city,
            discovered: boolean(unit, names::DISCOVERED)?,
            offers: array(unit, names::LINES_OFFER)?,
        });
    }

    let mut bus_offers = Vec::new();
    for unit in document.units_of_class(names::LINE_OFFER_DATA) {
        bus_offers.push(BusLineOffer {
            id: unit.id(),
            kind: text(unit, "type")?,
            source: text(unit, "source")?,
            targets: array(unit, names::SAVED_TARGETS)?
                .into_iter()
                .map(unquote)
                .collect(),
            distances: numbers(unit, names::TARGETS_DISTANCES)?,
            times: numbers(unit, names::TARGETS_TIMES)?,
            departure_time_of_day: count(unit, names::DEPARTURE_TIME_OF_DAY)?,
            bus: text(unit, names::BUS_NAME)?,
            expiration_time: count(unit, names::EXPIRATION_TIME)?,
        });
    }

    let bus_job = match first(document, names::BUS_JOB) {
        Some(unit) => Some(BusJob {
            id: unit.id(),
            kind: text(unit, "job_type")?,
            company_bus: reference(unit, "company_bus"),
            stages: array(unit, "stages")?
                .into_iter()
                .map(|id| {
                    let stage = units
                        .get(id)
                        .copied()
                        .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
                    bus_stage(stage)
                })
                .collect::<Result<_, _>>()?,
            stage_index: count(unit, "stage_idx")?,
            passengers: count(unit, "passengers")?,
        }),
        None => None,
    };

    let car_job = match first(document, names::CAR_JOB) {
        Some(unit) => Some(CarJob {
            id: unit.id(),
            job_type: text(unit, "job_type")?,
            source_company: text(unit, "source_company")?,
            target_company: text(unit, "target_company")?,
            planned_distance_km: count(unit, "planned_distance")?,
            urgency: count(unit, names::URGENCY)?,
            time_lower_limit: count(unit, "time_lower_limit")?,
            time_upper_limit: count(unit, "time_upper_limit")?,
            cargo: reference(unit, "cargo"),
            units_count: count(unit, names::UNITS_COUNT)?,
            travelled_distance: count(unit, "travelled_distance")?,
        }),
        None => None,
    };

    let mut car_offers = Vec::new();
    for unit in document.units_of_class(names::CAR_OFFER) {
        car_offers.push(CarOffer {
            id: unit.id(),
            source: string(unit, "source")?,
            target: string(unit, "target")?,
            expiration_time: optional_count(unit, names::EXPIRATION_TIME)?,
            urgency: optional_count(unit, names::URGENCY)?,
            job_type: text(unit, "job_type")?,
            shortest_distance_km: count(unit, "shortest_distance_km")?,
            job: reference(unit, "job"),
            cargo: reference(unit, "cargo"),
            company_car: string(unit, "company_car")?,
            units_count: count(unit, names::UNITS_COUNT)?,
        });
    }

    let bus_job_log = first(document, names::BUS_JOB_LOG)
        .map(job_log)
        .transpose()?;
    let car_job_log = first(document, names::CAR_JOB_LOG)
        .map(job_log)
        .transpose()?;

    Ok(Save {
        game,
        bank,
        experience,
        skills,
        visited_cities,
        unlocked_dealers,
        player,
        vehicles,
        trailers,
        garages,
        drivers,
        companies,
        jobs,
        player_job,
        offers,
        used_trucks,
        used_assortment,
        offence,
        oversize_offers,
        oversize_routes,
        bus_stops,
        bus_offers,
        bus_job,
        car_job,
        car_offers,
        bus_job_log,
        car_job_log,
    })
}

fn first<'a, 'd>(document: &'a Document<'d>, class: &'a str) -> Option<&'a Unit<'d>> {
    document.units_of_class(class).next()
}

fn linked<'a, 'd>(
    unit: Option<&'a Unit<'d>>,
    name: &str,
    units: &HashMap<&'a str, &'a Unit<'d>>,
) -> Option<&'a Unit<'d>> {
    units.get(reference(unit?, name)?).copied()
}

fn read_skills(unit: &Unit<'_>) -> Result<Skills, Error> {
    Ok(Skills {
        adr: count(unit, names::ADR)?,
        long_dist: count(unit, names::LONG_DIST)?,
        heavy: count(unit, names::HEAVY)?,
        fragile: count(unit, names::FRAGILE)?,
        urgent: count(unit, names::URGENT)?,
        mechanical: count(unit, names::MECHANICAL)?,
    })
}

fn job<'a>(unit: &'a Unit<'_>) -> Result<Job<'a>, Error> {
    Ok(Job {
        id: unit.id(),
        cargo: reference(unit, "cargo"),
        source_company: reference(unit, "source_company"),
        target_company: reference(unit, "target_company"),
        planned_distance_km: count(unit, "planned_distance_km")?,
        urgency: optional_count(unit, names::URGENCY)?,
        units_count: count(unit, names::UNITS_COUNT)?,
        fill_ratio: float(unit, names::FILL_RATIO)?,
    })
}

fn bus_stage<'a>(unit: &'a Unit<'_>) -> Result<BusJobStage<'a>, Error> {
    Ok(BusJobStage {
        id: unit.id(),
        source: text(unit, "source_bus_stop")?,
        target: text(unit, "target_bus_stop")?,
        planned_distance: count(unit, "planned_distance")?,
        planned_departure_time: optional_count(unit, "planned_departure_time")?,
        planned_arrival_time: optional_count(unit, "planned_arrival_time")?,
        planned_stage_time: count(unit, "planned_stage_time")?,
        planned_ferry_time: count(unit, "planned_ferry_time")?,
        planned_entering_passengers: count(unit, "planned_entering_passenger_count")?,
        planned_leaving_passengers: count(unit, "planned_leaving_passenger_count")?,
        departure_time: optional_count(unit, "departure_time")?,
        arrival_time: optional_count(unit, "arrival_time")?,
        travelled_distance: count(unit, "travelled_distance")?,
        satisfaction: float(unit, "satisfaction")?,
    })
}

fn job_log(unit: &Unit<'_>) -> Result<JobLog, Error> {
    Ok(JobLog {
        version: count(unit, "version")?,
        entries: count(unit, "entries")?,
        cached_jobs: optional_count(unit, "cached_jobs_count")?,
    })
}

fn numbers(unit: &Unit<'_>, name: &str) -> Result<Vec<u64>, Error> {
    array(unit, name)?
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| Error::invalid(unit.class(), name, value))
        })
        .collect()
}

fn fines(unit: &Unit<'_>, name: &str) -> Result<Vec<i64>, Error> {
    array(unit, name)?
        .into_iter()
        .map(|value| {
            value
                .parse()
                .map_err(|_| Error::invalid(unit.class(), name, value))
        })
        .collect()
}

/// The vehicle's accessory units, in `accessories[]` order.
fn accessories<'d>(
    unit: &Unit<'d>,
    units: &HashMap<&str, &'d Unit<'d>>,
) -> Result<Vec<Accessory<'d>>, Error> {
    let mut list = Vec::new();
    for id in array(unit, names::ACCESSORIES)? {
        let Some(accessory) = units.get(id).copied() else {
            continue;
        };
        let path = accessory.get(names::DATA_PATH).map(unquote);
        list.push(Accessory {
            id: accessory.id(),
            class: accessory.class(),
            path,
            slot: path.and_then(slot),
        });
    }
    Ok(list)
}

fn slot(path: &str) -> Option<&str> {
    let rest = path.split_once(names::ACCESSORY_DIR)?.1;
    let slot = rest.split('/').next()?;
    (!slot.is_empty()).then_some(slot)
}

/// The part paths selected by the accessory list.
fn parts<'a>(accessories: &[Accessory<'a>]) -> Parts<'a> {
    let mut data_path = None;
    let mut engine = None;
    let mut transmission = None;
    let mut cabin = None;
    let mut chassis = None;
    let mut paint_job = None;
    for accessory in accessories {
        let Some(path) = accessory.path else {
            continue;
        };
        match classify(path) {
            Some(Part::Data) => data_path = Some(path),
            Some(Part::Engine) => engine = Some(path),
            Some(Part::Transmission) => transmission = Some(path),
            Some(Part::Cabin) => cabin = Some(path),
            Some(Part::Chassis) => chassis = Some(path),
            Some(Part::Paint) => paint_job = Some(path),
            None => {}
        }
    }
    let model = data_path.and_then(|path| {
        path.strip_prefix(names::TRUCK_DIR)
            .and_then(|path| path.strip_suffix(names::DATA_SUFFIX))
    });
    Parts {
        model,
        data_path,
        engine,
        transmission,
        cabin,
        chassis,
        paint_job,
    }
}

struct Parts<'a> {
    model: Option<&'a str>,
    data_path: Option<&'a str>,
    engine: Option<&'a str>,
    transmission: Option<&'a str>,
    cabin: Option<&'a str>,
    chassis: Option<&'a str>,
    paint_job: Option<&'a str>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../testdata/read.sii");

    #[test]
    fn reads_every_modeled_class() {
        let document = Document::parse(SOURCE).unwrap();
        let save = read(&document, Game::Ets2).unwrap();

        assert_eq!(save.bank.as_ref().unwrap().money_account, 500);
        assert_eq!(save.experience, 100);
        assert_eq!(
            save.skills,
            Skills {
                adr: 1,
                long_dist: 2,
                heavy: 3,
                fragile: 4,
                urgent: 5,
                mechanical: 6,
            }
        );
        assert_eq!(save.visited_cities, ["prague"]);
        assert!(save.unlocked_dealers.is_empty());

        let player = save.player.as_ref().unwrap();
        assert_eq!(player.hq_city, "prague");
        assert_eq!(player.my_vehicles, ["slot.one"]);
        assert_eq!(player.trucks, ["vehicle.one"]);
        assert_eq!(player.trailers, ["trailer.one"]);
        assert_eq!(player.assigned_vehicles, Some("slot.one"));
        assert_eq!(player.current_job, Some("player_job.one"));
        assert!(player.buses.is_empty());
        assert!(player.cars.is_empty());
        assert_eq!(player.current_bus_job, None);
        assert_eq!(player.selected_bus_offer, None);
        assert_eq!(player.current_car_job, None);

        let vehicle = save.current_vehicle().unwrap();
        assert_eq!(vehicle.id, "vehicle.one");
        assert_eq!(vehicle.slot, Some("slot.one"));
        assert_eq!(vehicle.trailer, Some("trailer.one"));
        assert_eq!(vehicle.model, Some("test.model"));
        assert_eq!(
            vehicle.data_path,
            Some("/def/vehicle/truck/test.model/data.sii")
        );
        assert_eq!(
            vehicle.engine,
            Some("/def/vehicle/truck/test.model/engine/500.sii")
        );
        assert_eq!(
            vehicle.transmission,
            Some("/def/vehicle/truck/test.model/transmission/12.sii")
        );
        assert_eq!(
            vehicle.cabin,
            Some("/def/vehicle/truck/test.model/cabin/space.sii")
        );
        assert_eq!(
            vehicle.chassis,
            Some("/def/vehicle/truck/test.model/chassis/4x2.sii")
        );
        assert_eq!(
            vehicle.paint_job,
            Some("/def/vehicle/truck/test.model/paint_job/color1.sii")
        );
        let mirror = vehicle
            .accessories
            .iter()
            .find(|accessory| accessory.id == "accessory.mirror")
            .unwrap();
        assert_eq!(mirror.class, "vehicle_accessory");
        assert_eq!(mirror.slot, Some("mirror"));
        assert_eq!(vehicle.engine_wear, 1.0);
        assert_eq!(vehicle.transmission_wear, 0.5);
        assert_eq!(vehicle.cabin_wear, 0.0);
        assert_eq!(vehicle.fuel_relative, 1.0);
        assert_eq!(vehicle.license_plate, "PL<offset>12|germany");
        assert_eq!(vehicle.placement, Some("(1, 2, 3) (1; 0, 0, 0)"));

        let trailer = save.current_trailer().unwrap();
        assert_eq!(trailer.id, "trailer.one");
        assert_eq!(trailer.definition, "trailer_def.test");
        assert_eq!(trailer.cargo_mass, 1000.0);
        assert_eq!(trailer.license_plate, "TEST");
        assert_eq!(trailer.slave_trailer, None);
        assert_eq!(trailer.body_wear, 1.0);
        assert_eq!(trailer.chassis_wear, 0.5);
        assert_eq!(save.garages[0].status, 2);
        assert_eq!(save.drivers[0].experience_points, 10);
        let driver = &save.drivers[0];
        assert_eq!(
            driver.skills,
            Skills {
                adr: 0,
                long_dist: 1,
                heavy: 2,
                fragile: 3,
                urgent: 4,
                mechanical: 5,
            }
        );
        assert_eq!(driver.hometown, "prague");
        assert_eq!(driver.assigned_truck, Some("vehicle.one"));
        assert_eq!(driver.job, Some("job.one"));
        assert_eq!(driver.training_policy, 2);
        assert_eq!(save.companies[0].permanent_data, "company.permanent.test");
        assert_eq!(save.jobs[0].cargo, None);
        assert_eq!(save.jobs[0].urgency, Some(0));
        assert_eq!(save.player_job.as_ref().unwrap().cargo, Some("cargo.stone"));
        assert_eq!(save.player_job.as_ref().unwrap().planned_distance_km, 12);
        assert_eq!(save.player_job.as_ref().unwrap().units_count, 20);
        assert_eq!(save.player_job.as_ref().unwrap().fill_ratio, 0.5);
        assert_eq!(save.offers[0].target, "city.name");
        assert_eq!(save.offers[0].shortest_distance_km, 7);
        assert_eq!(save.offers[0].urgency, Some(2));
        assert_eq!(save.offers[0].expiration_time, Some(500));
        assert_eq!(save.offers[0].units_count, 30);
        assert_eq!(save.offers[0].fill_ratio, 0.75);
        assert_eq!(save.offers[0].trailer_definition, "trailer_def.test");
        assert_eq!(save.used_trucks[0].truck, "vehicle.one");
        assert_eq!(save.used_trucks[0].price, 100);
        assert_eq!(save.used_trucks[0].expiration_game_time, 800);
        let assortment = save.used_assortment.as_ref().unwrap();
        assert_eq!(assortment.id, "used_assortment.one");
        assert_eq!(assortment.next_generation_game_time, 600);
        assert_eq!(assortment.trucks, ["used.one"]);
        let offence = save.offence.as_ref().unwrap();
        assert_eq!(offence.id, "offences");
        assert_eq!(offence.counts, [4, 5]);
        assert_eq!(offence.fines, [600, 700]);
        assert_eq!(offence.history_entries, 3);
        assert_eq!(save.oversize_offers[0].id, "oversize.one");
        assert_eq!(save.oversize_offers[0].offer_data, "spec_offer.ovesize_3");
        assert_eq!(
            save.oversize_offers[0].truck,
            "oversize/mercedes_actros_8x4"
        );
        assert_eq!(save.oversize_offers[0].expiration, 6121);
        assert_eq!(save.oversize_routes[0].route, "route_data.koln_amster1");
        assert_eq!(save.oversize_routes[0].offers, ["oversize.one"]);
    }

    #[test]
    fn reports_a_missing_attribute() {
        let source = SOURCE.replace("    engine_wear: &3f800000\n", "");
        let document = Document::parse(&source).unwrap();
        let error = match read(&document, Game::Ets2) {
            Ok(_) => panic!("missing attribute must fail"),
            Err(error) => error,
        };
        assert_eq!(
            error.to_string(),
            "missing attribute \"engine_wear\" on vehicle"
        );
    }

    #[test]
    fn reports_an_invalid_value() {
        let source = SOURCE.replace("engine_wear: &3f800000", "engine_wear: nope");
        let document = Document::parse(&source).unwrap();
        let error = match read(&document, Game::Ets2) {
            Ok(_) => panic!("invalid value must fail"),
            Err(error) => error,
        };
        assert_eq!(
            error.to_string(),
            "invalid engine_wear value \"nope\" on vehicle"
        );
    }
}
