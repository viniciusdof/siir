//! Vehicle commands: wear, fuel, odometer, plate, and parts.

use siir::{Document, Unit, Value};

use crate::accessory::{Part, classify, write_accessories};
use crate::error::Error;
use crate::names::{self, indexed};
use crate::value::{length, reference, unquote};

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
/// Wear a mod's vehicle may omit, so it is reset only when the game wrote it.
const OPTIONAL_WEAR: [&str; 2] = [names::CHASSIS_WEAR, names::CHASSIS_WEAR_UNFIXABLE];
const WHEELS: [&str; 2] = ["wheels_wear", "wheels_wear_unfixable"];

pub(super) fn repair_vehicle(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    repair(vehicle(document, id)?)
}

pub(super) fn repair_all_vehicles(document: &mut Document<'_>) -> Result<(), Error> {
    for id in owned_vehicles(document)? {
        if let Ok(unit) = vehicle(document, &id) {
            repair(unit)?;
        }
    }
    Ok(())
}

pub(super) fn set_fuel(document: &mut Document<'_>, id: &str, level: f32) -> Result<(), Error> {
    if !(0.0..=1.0).contains(&level) {
        return Err(Error::InvalidFuel { level });
    }
    vehicle(document, id)?.set(FUEL, Value::Float(level))?;
    Ok(())
}

pub(super) fn fill_all_vehicles(document: &mut Document<'_>) -> Result<(), Error> {
    for id in owned_vehicles(document)? {
        if let Ok(unit) = vehicle(document, &id) {
            fill(unit)?;
        }
    }
    Ok(())
}

pub(super) fn set_infinite_fuel(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    fill(vehicle(document, id)?)
}

pub(super) fn set_odometer(document: &mut Document<'_>, id: &str, km: u64) -> Result<(), Error> {
    let unit = vehicle(document, id)?;
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
    vehicle(document, id)?.set("license_plate", Value::String(plate))?;
    Ok(())
}

fn repair(unit: &mut Unit<'_>) -> Result<(), Error> {
    for name in WEAR {
        if unit.get(name).is_none() {
            return Err(Error::missing(unit.class(), name));
        }
    }
    let mut wheels = Vec::new();
    for name in WHEELS {
        if unit.get(name).is_some() {
            wheels.push((name, length(unit, name)?));
        }
    }
    for name in WEAR {
        unit.set(name, Value::Int(0))?;
    }
    for name in OPTIONAL_WEAR {
        if unit.get(name).is_some() {
            unit.set(name, Value::Int(0))?;
        }
    }
    for (name, total) in wheels {
        for nth in 0..total {
            unit.set(&indexed(name, nth), Value::Int(0))?;
        }
    }
    Ok(())
}

fn fill(unit: &mut Unit<'_>) -> Result<(), Error> {
    unit.set(FUEL, Value::Float(1.0))?;
    Ok(())
}

fn vehicle<'a, 'd>(document: &'a mut Document<'d>, id: &str) -> Result<&'a mut Unit<'d>, Error> {
    match document.get(id) {
        Some(unit) if unit.class() == names::VEHICLE => Ok(unit),
        _ => Err(Error::MissingUnit { id: id.to_owned() }),
    }
}

fn owned_vehicles(document: &Document<'_>) -> Result<Vec<String>, Error> {
    let Some(player) = document.units_of_class(names::PLAYER).next() else {
        return Ok(Vec::new());
    };
    // Same rule as `Save::owns`: the player's truck, bus, and car lists, plus
    // any vehicle a `player_vehicles` slot points at.
    let mut vehicles = Vec::new();
    for list in [names::TRUCKS, names::BUSES, names::CARS] {
        if player.get(list).is_some() {
            let count = length(player, list)?;
            for nth in 0..count {
                if let Some(id) = player.get(&indexed(list, nth)) {
                    vehicles.push(id.to_owned());
                }
            }
        }
    }
    for slot in document.units_of_class(names::PLAYER_VEHICLES) {
        if let Some(vehicle) = reference(slot, names::VEHICLE) {
            vehicles.push(vehicle.to_owned());
        }
    }
    vehicles.sort();
    vehicles.dedup();
    Ok(vehicles)
}

pub(super) fn remove_badge(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    let (total, kept) = {
        let unit = document
            .units_of_class(names::VEHICLE)
            .find(|unit| unit.id() == id)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        let total = length(unit, names::ACCESSORIES)?;
        let mut kept = Vec::with_capacity(total);
        for nth in 0..total {
            let accessory = unit
                .get(&indexed(names::ACCESSORIES, nth))
                .ok_or_else(|| Error::missing(unit.class(), names::ACCESSORIES))?;
            let badged = document
                .units()
                .find(|unit| unit.id() == accessory)
                .and_then(|unit| unit.get(names::DATA_PATH))
                .is_some_and(|path| crate::model::is_badge(unquote(path)));
            if !badged {
                kept.push(accessory.to_owned());
            }
        }
        if kept.len() == total {
            return Err(Error::NoBadge { id: id.to_owned() });
        }
        (total, kept)
    };

    let unit = vehicle(document, id)?;
    write_accessories(unit, &kept, total)
}

pub(super) fn set_engine(document: &mut Document<'_>, id: &str, path: &str) -> Result<(), Error> {
    if set_part(document, id, Part::Engine, path)? {
        Ok(())
    } else {
        Err(Error::NoEngine { id: id.to_owned() })
    }
}

pub(super) fn set_transmission(
    document: &mut Document<'_>,
    id: &str,
    path: &str,
) -> Result<(), Error> {
    if set_part(document, id, Part::Transmission, path)? {
        Ok(())
    } else {
        Err(Error::NoTransmission { id: id.to_owned() })
    }
}

pub(super) fn set_cabin(document: &mut Document<'_>, id: &str, path: &str) -> Result<(), Error> {
    if set_part(document, id, Part::Cabin, path)? {
        Ok(())
    } else {
        Err(Error::NoCabin { id: id.to_owned() })
    }
}

pub(super) fn set_chassis(document: &mut Document<'_>, id: &str, path: &str) -> Result<(), Error> {
    if set_part(document, id, Part::Chassis, path)? {
        Ok(())
    } else {
        Err(Error::NoChassis { id: id.to_owned() })
    }
}

pub(super) fn set_paint(document: &mut Document<'_>, id: &str, path: &str) -> Result<(), Error> {
    if set_part(document, id, Part::Paint, path)? {
        Ok(())
    } else {
        Err(Error::NoPaint { id: id.to_owned() })
    }
}

pub(super) fn set_accessory(
    document: &mut Document<'_>,
    id: &str,
    accessory: &str,
    path: &str,
) -> Result<(), Error> {
    let listed = {
        let unit = document
            .units_of_class(names::VEHICLE)
            .find(|unit| unit.id() == id)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        let total = length(unit, names::ACCESSORIES)?;
        (0..total).any(|nth| unit.get(&indexed(names::ACCESSORIES, nth)) == Some(accessory))
    };
    if !listed {
        return Err(Error::NoAccessory {
            id: id.to_owned(),
            accessory: accessory.to_owned(),
        });
    }
    let unit = document.get(accessory).ok_or_else(|| Error::MissingUnit {
        id: accessory.to_owned(),
    })?;
    unit.set(names::DATA_PATH, Value::String(path.to_owned()))?;
    Ok(())
}

pub(super) fn remove_accessory(
    document: &mut Document<'_>,
    id: &str,
    accessory: &str,
) -> Result<(), Error> {
    let (total, kept) = {
        let unit = document
            .units_of_class(names::VEHICLE)
            .find(|unit| unit.id() == id)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        let total = length(unit, names::ACCESSORIES)?;
        let mut kept = Vec::with_capacity(total);
        let mut removed = false;
        for nth in 0..total {
            let current = unit
                .get(&indexed(names::ACCESSORIES, nth))
                .ok_or_else(|| Error::missing(unit.class(), names::ACCESSORIES))?;
            if current == accessory {
                removed = true;
            } else {
                kept.push(current.to_owned());
            }
        }
        if !removed {
            return Err(Error::NoAccessory {
                id: id.to_owned(),
                accessory: accessory.to_owned(),
            });
        }
        (total, kept)
    };

    if let Some(part) = document
        .get(accessory)
        .and_then(|unit| unit.get(names::DATA_PATH))
        .map(unquote)
        .and_then(classify)
    {
        return Err(Error::CannotRemovePart { part: part.name() });
    }

    let unit = vehicle(document, id)?;
    write_accessories(unit, &kept, total)
}

fn set_part(document: &mut Document<'_>, id: &str, part: Part, path: &str) -> Result<bool, Error> {
    let accessories = {
        let unit = document
            .units_of_class(names::VEHICLE)
            .find(|unit| unit.id() == id)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        let total = length(unit, names::ACCESSORIES)?;
        let mut ids = Vec::with_capacity(total);
        for nth in 0..total {
            ids.push(
                unit.get(&indexed(names::ACCESSORIES, nth))
                    .ok_or_else(|| Error::missing(unit.class(), names::ACCESSORIES))?
                    .to_owned(),
            );
        }
        ids
    };

    let mut found = false;
    for accessory in accessories {
        let Some(unit) = document.get(&accessory) else {
            continue;
        };
        if unit
            .get(names::DATA_PATH)
            .is_some_and(|value| classify(unquote(value)) == Some(part))
        {
            unit.set(names::DATA_PATH, Value::String(path.to_owned()))?;
            found = true;
        }
    }
    Ok(found)
}

pub(super) fn set_current_vehicle(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
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
        .ok_or(Error::NoCurrentVehicle)?;
    let unit = document
        .get(&slot)
        .filter(|unit| unit.class() == names::PLAYER_VEHICLES)
        .ok_or_else(|| Error::MissingUnit { id: slot.clone() })?;
    unit.set(names::VEHICLE, Value::token(id))?;
    Ok(())
}

pub(super) fn set_hq_vehicle(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
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
        .ok_or(Error::NoCurrentVehicle)?;
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

    const SOURCE: &str = include_str!("../../testdata/vehicle.sii");

    fn source() -> Document<'static> {
        Document::parse(SOURCE).unwrap()
    }

    fn value<'a>(document: &'a Document<'_>, id: &str, name: &str) -> Option<&'a str> {
        document.units().find(|unit| unit.id() == id)?.get(name)
    }

    #[test]
    fn repairs_one_vehicle() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::RepairVehicle("vehicle.one".into()),
        )
        .unwrap();
        for name in WEAR {
            assert_eq!(value(&document, "vehicle.one", name), Some("0"), "{name}");
        }
        for name in OPTIONAL_WEAR {
            assert_eq!(value(&document, "vehicle.one", name), Some("0"), "{name}");
        }
        for name in WHEELS {
            assert_eq!(value(&document, "vehicle.one", name), Some("2"), "{name}");
            for nth in 0..2 {
                assert_eq!(
                    value(&document, "vehicle.one", &indexed(name, nth)),
                    Some("0"),
                    "{name}[{nth}]"
                );
            }
        }
    }

    #[test]
    fn repairs_owned_vehicles() {
        let mut document = source();
        apply(&mut document, Game::Ets2, Command::RepairAllVehicles).unwrap();
        assert_eq!(value(&document, "vehicle.one", "engine_wear"), Some("0"));
        assert_eq!(
            value(&document, "vehicle.one", "engine_wear_unfixable"),
            Some("0")
        );
        assert_eq!(value(&document, "vehicle.two", "engine_wear"), Some("0"));
    }

    #[test]
    fn fills_one_and_all_vehicles() {
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
        apply(&mut document, Game::Ets2, Command::FillAllVehicles).unwrap();
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
            Some("8")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[0]"),
            Some("accessory.data")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[1]"),
            Some("accessory.wheel")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[2]"),
            Some("accessory.engine")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[3]"),
            Some("accessory.transmission")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[4]"),
            Some("accessory.cabin")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[5]"),
            Some("accessory.chassis")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[6]"),
            Some("accessory.paint")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[7]"),
            Some("accessory.mirror")
        );
        assert_eq!(value(&document, "vehicle.one", "accessories[8]"), None);
        assert!(
            !document
                .to_text()
                .contains("accessories[1]: accessory.badge")
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
    fn sets_engine_and_transmission() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetEngine {
                id: "vehicle.one".into(),
                path: "/def/vehicle/truck/test.model/engine/600.sii".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "accessory.engine", names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/engine/600.sii\"")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::SetTransmission {
                id: "vehicle.one".into(),
                path: "/def/vehicle/truck/test.model/transmission/6.sii".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "accessory.transmission", names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/transmission/6.sii\"")
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetEngine {
                id: "vehicle.two".into(),
                path: "/def/vehicle/truck/test.model/engine/600.sii".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "no engine on vehicle.two");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetTransmission {
                id: "vehicle.two".into(),
                path: "/def/vehicle/truck/test.model/transmission/6.sii".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "no transmission on vehicle.two");
    }

    #[test]
    fn sets_cabin_and_chassis() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetCabin {
                id: "vehicle.one".into(),
                path: "/def/vehicle/truck/test.model/cabin/sleeper.sii".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "accessory.cabin", names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/cabin/sleeper.sii\"")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::SetChassis {
                id: "vehicle.one".into(),
                path: "/def/vehicle/truck/test.model/chassis/6x2.sii".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "accessory.chassis", names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/chassis/6x2.sii\"")
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetCabin {
                id: "vehicle.two".into(),
                path: "/def/vehicle/truck/test.model/cabin/sleeper.sii".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "no cabin on vehicle.two");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetChassis {
                id: "vehicle.two".into(),
                path: "/def/vehicle/truck/test.model/chassis/6x2.sii".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "no chassis on vehicle.two");
    }

    #[test]
    fn sets_paint() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetPaint {
                id: "vehicle.one".into(),
                path: "/def/vehicle/truck/test.model/paint_job/color2.sii".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "accessory.paint", names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/paint_job/color2.sii\"")
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetPaint {
                id: "vehicle.two".into(),
                path: "/def/vehicle/truck/test.model/paint_job/color2.sii".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "no paint job on vehicle.two");
    }

    #[test]
    fn sets_and_removes_an_accessory() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetAccessory {
                id: "vehicle.one".into(),
                accessory: "accessory.mirror".into(),
                path: "/def/vehicle/truck/test.model/accessory/mirror/shape2.sii".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "accessory.mirror", names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/accessory/mirror/shape2.sii\"")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::RemoveAccessory {
                id: "vehicle.one".into(),
                accessory: "accessory.mirror".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "vehicle.one", names::ACCESSORIES),
            Some("8")
        );
        assert_eq!(
            value(&document, "vehicle.one", "accessories[7]"),
            Some("accessory.paint")
        );
        assert_eq!(value(&document, "vehicle.one", "accessories[8]"), None);

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::RemoveAccessory {
                id: "vehicle.one".into(),
                accessory: "accessory.mirror".into(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "no accessory accessory.mirror on vehicle.one"
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetAccessory {
                id: "vehicle.one".into(),
                accessory: "accessory.missing".into(),
                path: "/def/vehicle/truck/test.model/accessory/mirror/shape2.sii".into(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "no accessory accessory.missing on vehicle.one"
        );
    }

    #[test]
    fn sets_current_vehicle_and_hq() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetCurrentVehicle("vehicle.two".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "slot.one", names::VEHICLE),
            Some("vehicle.two")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::SetHqVehicle("vehicle.one".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "player.one", names::HQ_CITY),
            Some("prague")
        );
    }

    #[test]
    fn repair_leaves_a_partial_vehicle_unchanged() {
        let source = "SiiNunit\n{\nvehicle : vehicle.one {\n    engine_wear: &3f800000\n    transmission_wear: &3f800000\n    cabin_wear: 0\n    engine_wear_unfixable: 1\n    transmission_wear_unfixable: 1\n}\n}\n";
        let mut document = Document::parse(source).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::RepairVehicle("vehicle.one".into()),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "missing attribute \"cabin_wear_unfixable\" on vehicle"
        );
        assert_eq!(document.to_text(), source);
    }

    #[test]
    fn rejects_removing_a_required_part() {
        let mut document = source();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::RemoveAccessory {
                id: "vehicle.one".into(),
                accessory: "accessory.engine".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "cannot remove the engine");
        assert_eq!(
            value(&document, "vehicle.one", names::ACCESSORIES),
            Some("9")
        );
    }

    #[test]
    fn reports_command_errors() {
        let mut document = source();
        let cases = [
            (
                Command::RepairVehicle("nope".into()),
                "missing unit \"nope\"",
            ),
            (
                Command::SetFuel {
                    id: "vehicle.one".into(),
                    level: 2.0,
                },
                "fuel level 2 is not between 0 and 1",
            ),
            (
                Command::SetHqVehicle("vehicle.two".into()),
                "no garage holds vehicle.two",
            ),
        ];
        for (command, expected) in cases {
            let error = apply(&mut document, Game::Ets2, command).unwrap_err();
            assert_eq!(error.to_string(), expected);
        }
    }
}
