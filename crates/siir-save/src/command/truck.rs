//! Truck commands.

use siir::{Document, Unit, Value};

use crate::error::Error;
use crate::names::{self, indexed};
use crate::value::{count, reference, unquote};

const FUEL: &str = "fuel_relative";
const ODOMETER: [&str; 3] = ["odometer", "integrity_odometer", "trip_distance_km"];

const WEAR: [&str; 6] = [
    "engine_wear",
    "transmission_wear",
    "cabin_wear",
    "engine_wear_unfixable",
    "transmission_wear_unfixable",
    "cabin_wear_unfixable",
];

pub(super) fn repair_truck(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    repair(truck(document, id)?)
}

pub(super) fn repair_all_trucks(document: &mut Document<'_>) -> Result<(), Error> {
    for id in owned_trucks(document) {
        if let Ok(unit) = truck(document, &id) {
            repair(unit)?;
        }
    }
    Ok(())
}

pub(super) fn set_fuel(document: &mut Document<'_>, id: &str, level: f32) -> Result<(), Error> {
    if !(0.0..=1.0).contains(&level) {
        return Err(Error::InvalidFuel { level });
    }
    truck(document, id)?.set(FUEL, Value::Float(level))?;
    Ok(())
}

pub(super) fn fill_all_trucks(document: &mut Document<'_>) -> Result<(), Error> {
    for id in owned_trucks(document) {
        if let Ok(unit) = truck(document, &id) {
            fill(unit)?;
        }
    }
    Ok(())
}

pub(super) fn set_infinite_fuel(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    fill(truck(document, id)?)
}

pub(super) fn set_odometer(document: &mut Document<'_>, id: &str, km: u64) -> Result<(), Error> {
    let unit = truck(document, id)?;
    for name in ODOMETER {
        unit.set(name, Value::UInt(km))?;
    }
    Ok(())
}

pub(super) fn set_license_plate(
    document: &mut Document<'_>,
    id: &str,
    plate: String,
) -> Result<(), Error> {
    truck(document, id)?.set("license_plate", Value::String(plate))?;
    Ok(())
}

fn repair(unit: &mut Unit<'_>) -> Result<(), Error> {
    for name in WEAR {
        unit.set(name, Value::Int(0))?;
    }
    Ok(())
}

fn fill(unit: &mut Unit<'_>) -> Result<(), Error> {
    unit.set(FUEL, Value::Float(1.0))?;
    Ok(())
}

fn truck<'a, 'd>(document: &'a mut Document<'d>, id: &str) -> Result<&'a mut Unit<'d>, Error> {
    match document.get(id) {
        Some(unit) if unit.class() == names::VEHICLE => Ok(unit),
        _ => Err(Error::MissingUnit { id: id.to_owned() }),
    }
}

fn owned_trucks(document: &Document<'_>) -> Vec<String> {
    let Some(player) = document.units_of_class(names::PLAYER).next() else {
        return Vec::new();
    };
    let mut trucks = Vec::new();
    if let Some(count) = player
        .get(names::TRUCKS)
        .and_then(|value| value.parse::<usize>().ok())
    {
        for nth in 0..count {
            if let Some(id) = player.get(&indexed(names::TRUCKS, nth)) {
                trucks.push(id.to_owned());
            }
        }
    }
    if let Some(count) = player
        .get(names::MY_VEHICLES)
        .and_then(|value| value.parse::<usize>().ok())
    {
        for nth in 0..count {
            let Some(slot_id) = player.get(&indexed(names::MY_VEHICLES, nth)) else {
                continue;
            };
            let Some(slot) = document
                .units_of_class(names::PLAYER_VEHICLES)
                .find(|unit| unit.id() == slot_id)
            else {
                continue;
            };
            if let Some(vehicle) = reference(slot, names::VEHICLE) {
                trucks.push(vehicle.to_owned());
            }
        }
    }
    trucks.sort();
    trucks.dedup();
    trucks
}

pub(super) fn remove_badge(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    let (total, kept) = {
        let unit = document
            .units_of_class(names::VEHICLE)
            .find(|unit| unit.id() == id)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        let total = count(unit, names::ACCESSORIES)? as usize;
        let mut kept = Vec::with_capacity(total);
        for nth in 0..total {
            let accessory = unit
                .get(&indexed(names::ACCESSORIES, nth))
                .ok_or_else(|| Error::missing(unit.class(), names::ACCESSORIES))?;
            let badged = document
                .units()
                .find(|unit| unit.id() == accessory)
                .and_then(|unit| unit.get(names::DATA_PATH))
                .is_some_and(|path| is_badge(unquote(path)));
            if !badged {
                kept.push(accessory.to_owned());
            }
        }
        if kept.len() == total {
            return Err(Error::NoBadge { id: id.to_owned() });
        }
        (total, kept)
    };

    let unit = truck(document, id)?;
    unit.set(names::ACCESSORIES, Value::UInt(kept.len() as u64))?;
    for (nth, accessory) in kept.iter().enumerate() {
        unit.set(&indexed(names::ACCESSORIES, nth), Value::token(accessory))?;
    }
    for nth in kept.len()..total {
        unit.remove_attr(&indexed(names::ACCESSORIES, nth))?;
    }
    Ok(())
}

fn is_badge(path: &str) -> bool {
    path.split('/')
        .any(|part| part == names::BADGE || part.ends_with(names::BADGE_SUFFIX))
}

pub(super) fn set_current_truck(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    if !document
        .units_of_class(names::VEHICLE)
        .any(|unit| unit.id() == id)
    {
        return Err(Error::MissingUnit { id: id.to_owned() });
    }
    let slot = document
        .units_of_class(names::PLAYER)
        .next()
        .and_then(|player| reference(player, names::ASSIGNED_VEHICLES))
        .map(str::to_owned)
        .ok_or(Error::NoCurrentTruck)?;
    let unit = document
        .get(&slot)
        .filter(|unit| unit.class() == names::PLAYER_VEHICLES)
        .ok_or_else(|| Error::MissingUnit { id: slot.clone() })?;
    unit.set(names::VEHICLE, Value::token(id))?;
    Ok(())
}

pub(super) fn set_hq_truck(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    let city = document
        .units_of_class(names::GARAGE)
        .find_map(|garage| {
            let count = garage.get(names::VEHICLES)?.parse::<usize>().ok()?;
            let holds =
                (0..count).any(|nth| garage.get(&indexed(names::VEHICLES, nth)) == Some(id));
            if !holds {
                return None;
            }
            garage
                .id()
                .strip_prefix(names::GARAGE_PREFIX)
                .map(str::to_owned)
        })
        .ok_or_else(|| Error::NoGarage { id: id.to_owned() })?;
    let player = document
        .units_of_class(names::PLAYER)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or(Error::NoCurrentTruck)?;
    document
        .get(&player)
        .ok_or(Error::MissingUnit { id: player })?
        .set(names::HQ_CITY, Value::token(city))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/truck.sii");

    fn source() -> Document<'static> {
        Document::parse(SOURCE).unwrap()
    }

    fn value<'a>(document: &'a Document<'_>, id: &str, name: &str) -> Option<&'a str> {
        document.units().find(|unit| unit.id() == id)?.get(name)
    }

    #[test]
    fn repairs_one_truck() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::RepairTruck("vehicle.one".into()),
        )
        .unwrap();
        for name in WEAR {
            assert_eq!(value(&document, "vehicle.one", name), Some("0"), "{name}");
        }
    }

    #[test]
    fn repairs_owned_trucks() {
        let mut document = source();
        apply(&mut document, Game::Ets2, Command::RepairAllTrucks).unwrap();
        assert_eq!(value(&document, "vehicle.one", "engine_wear"), Some("0"));
        assert_eq!(
            value(&document, "vehicle.one", "engine_wear_unfixable"),
            Some("0")
        );
        assert_eq!(value(&document, "vehicle.two", "engine_wear"), Some("0"));
    }

    #[test]
    fn fills_one_and_all_trucks() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetFuel {
                id: "vehicle.one".into(),
                level: 0.25,
            },
        )
        .unwrap();
        assert_eq!(value(&document, "vehicle.one", FUEL), Some("0.25"));
        apply(
            &mut document,
            Game::Ets2,
            Command::SetInfiniteFuel("vehicle.one".into()),
        )
        .unwrap();
        assert_eq!(value(&document, "vehicle.one", FUEL), Some("1.0"));

        let mut document = source();
        apply(&mut document, Game::Ets2, Command::FillAllTrucks).unwrap();
        assert_eq!(value(&document, "vehicle.one", FUEL), Some("1.0"));
        assert_eq!(value(&document, "vehicle.two", FUEL), Some("1.0"));
    }

    #[test]
    fn sets_odometer_and_plate() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetOdometer {
                id: "vehicle.one".into(),
                km: 1234,
            },
        )
        .unwrap();
        assert_eq!(value(&document, "vehicle.one", "odometer"), Some("1234"));
        assert_eq!(
            value(&document, "vehicle.one", "integrity_odometer"),
            Some("1234")
        );
        assert_eq!(
            value(&document, "vehicle.one", "trip_distance_km"),
            Some("1234")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::SetLicensePlate {
                id: "vehicle.one".into(),
                plate: "TEST-123".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "vehicle.one", "license_plate"),
            Some("\"TEST-123\"")
        );
    }

    #[test]
    fn removes_only_badge_accessories() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::RemoveBadge("vehicle.one".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "vehicle.one", names::ACCESSORIES),
            Some("2")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[0]"),
            Some("accessory.data")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[1]"),
            Some("accessory.wheel")
        );
        assert_eq!(value(&document, "vehicle.one", "accessories[2]"), None);
        assert!(
            !document
                .to_text()
                .contains("accessories[2]: accessory.wheel")
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::RemoveBadge("vehicle.one".into()),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "no badge on vehicle.one");
    }

    #[test]
    fn sets_current_truck_and_hq() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetCurrentTruck("vehicle.two".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "slot.one", names::VEHICLE),
            Some("vehicle.two")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::SetHqTruck("vehicle.one".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "player.one", names::HQ_CITY),
            Some("prague")
        );
    }

    #[test]
    fn reports_command_errors() {
        let mut document = source();
        let cases = [
            (Command::RepairTruck("nope".into()), "missing unit \"nope\""),
            (
                Command::SetFuel {
                    id: "vehicle.one".into(),
                    level: 2.0,
                },
                "fuel level 2 is not between 0 and 1",
            ),
            (
                Command::SetHqTruck("vehicle.two".into()),
                "no garage holds vehicle.two",
            ),
        ];
        for (command, expected) in cases {
            let error = apply(&mut document, Game::Ets2, command).unwrap_err();
            assert_eq!(error.to_string(), expected);
        }
    }
}
