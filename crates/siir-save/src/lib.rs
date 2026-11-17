//! Typed read model and edits for SCS save documents.
//!
//! [`read`] borrows the document and [`apply`] edits it in place.

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
pub mod profiles;
mod value;

pub use accessory::{export_accessories, import_accessories};
pub use color::Color;
pub use command::{Command, apply};
pub use error::Error;
pub use model::{
    Accessory, Bank, Company, Driver, Game, Garage, Job, Offence, Offer, Player, Save, Skills,
    Trailer, Truck, UsedTruck,
};

use value::{array, count, float, integer, reference, string, text, unquote};

/// Reads the typed view of a save document.
pub fn read<'a>(document: &'a Document<'_>, game: Game) -> Result<Save<'a>, Error> {
    let units: HashMap<&str, &Unit<'_>> = document.units().map(|unit| (unit.id(), unit)).collect();

    let economy = first(document, names::ECONOMY);
    let bank = linked(economy, names::BANK, &units).or_else(|| first(document, names::BANK));
    let player = linked(economy, names::PLAYER, &units).or_else(|| first(document, names::PLAYER));

    let (experience, skills, visited_cities, unlocked_dealers) = match economy {
        Some(unit) => (
            count(unit, names::EXPERIENCE_POINTS)?,
            skills(unit)?,
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
            assigned_vehicles: reference(unit, names::ASSIGNED_VEHICLES),
            current_job: reference(unit, "current_job"),
        }),
        None => None,
    };

    let mut trucks = Vec::new();
    for unit in document.units_of_class(names::VEHICLE) {
        let slot = slots.get(unit.id()).copied();
        let accessories = accessories(unit, &units)?;
        let parts = parts(&accessories);
        trucks.push(Truck {
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
        });
    }

    let mut used_trucks = Vec::new();
    for unit in document.units_of_class(names::USED_TRUCK_OFFER) {
        used_trucks.push(UsedTruck {
            id: unit.id(),
            truck: text(unit, "truck")?,
            price: integer(unit, "price")?,
        });
    }

    let offence = first(document, names::POLICE_OFFENCE_LOG).map(|unit| Offence { id: unit.id() });

    Ok(Save {
        game,
        bank,
        experience,
        skills,
        visited_cities,
        unlocked_dealers,
        player,
        trucks,
        trailers,
        garages,
        drivers,
        companies,
        jobs,
        player_job,
        offers,
        used_trucks,
        offence,
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

fn skills(unit: &Unit<'_>) -> Result<Skills, Error> {
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
    })
}

/// The truck's accessory units, in `accessories[]` order.
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

        let truck = save.current_truck().unwrap();
        assert_eq!(truck.id, "vehicle.one");
        assert_eq!(truck.slot, Some("slot.one"));
        assert_eq!(truck.trailer, Some("trailer.one"));
        assert_eq!(truck.model, Some("test.model"));
        assert_eq!(
            truck.data_path,
            Some("/def/vehicle/truck/test.model/data.sii")
        );
        assert_eq!(
            truck.engine,
            Some("/def/vehicle/truck/test.model/engine/500.sii")
        );
        assert_eq!(
            truck.transmission,
            Some("/def/vehicle/truck/test.model/transmission/12.sii")
        );
        assert_eq!(
            truck.cabin,
            Some("/def/vehicle/truck/test.model/cabin/space.sii")
        );
        assert_eq!(
            truck.chassis,
            Some("/def/vehicle/truck/test.model/chassis/4x2.sii")
        );
        assert_eq!(
            truck.paint_job,
            Some("/def/vehicle/truck/test.model/paint_job/color1.sii")
        );
        let mirror = truck
            .accessories
            .iter()
            .find(|accessory| accessory.id == "accessory.mirror")
            .unwrap();
        assert_eq!(mirror.class, "vehicle_accessory");
        assert_eq!(mirror.slot, Some("mirror"));
        assert_eq!(truck.engine_wear, 1.0);
        assert_eq!(truck.transmission_wear, 0.5);
        assert_eq!(truck.cabin_wear, 0.0);
        assert_eq!(truck.fuel_relative, 1.0);
        assert_eq!(truck.placement, Some("(1, 2, 3) (1; 0, 0, 0)"));

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
        assert_eq!(save.companies[0].permanent_data, "company.permanent.test");
        assert_eq!(save.jobs[0].cargo, None);
        assert_eq!(save.player_job.as_ref().unwrap().cargo, Some("cargo.stone"));
        assert_eq!(save.player_job.as_ref().unwrap().planned_distance_km, 12);
        assert_eq!(save.offers[0].target, "city.name");
        assert_eq!(save.offers[0].shortest_distance_km, 7);
        assert_eq!(save.used_trucks[0].truck, "vehicle.one");
        assert_eq!(save.used_trucks[0].price, 100);
        assert_eq!(save.offence.as_ref().unwrap().id, "offences");
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
