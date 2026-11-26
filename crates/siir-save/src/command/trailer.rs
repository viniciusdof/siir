use siir::{Document, Unit, Value};

use crate::error::Error;
use crate::names::{self, indexed};
use crate::value::{length, reference, text};

const WEAR: [&str; 4] = [
    names::TRAILER_BODY_WEAR,
    names::TRAILER_BODY_WEAR_UNFIXABLE,
    names::CHASSIS_WEAR,
    names::CHASSIS_WEAR_UNFIXABLE,
];
const WHEELS: [&str; 2] = ["wheels_wear", "wheels_wear_unfixable"];

const CHAIN_LIMIT: usize = 20;

pub(super) fn repair_trailer(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    repair(document, id)
}

pub(super) fn repair_all_trailers(document: &mut Document<'_>) -> Result<(), Error> {
    for id in owned_trailers(document)? {
        if document
            .units_of_class(names::TRAILER)
            .any(|unit| unit.id() == id)
        {
            repair(document, &id)?;
        }
    }
    Ok(())
}

pub(super) fn set_cargo_mass(
    document: &mut Document<'_>,
    id: &str,
    mass: f32,
) -> Result<(), Error> {
    if !mass.is_finite() || mass < 0.0 {
        return Err(Error::invalid(
            names::TRAILER,
            names::CARGO_MASS,
            &mass.to_string(),
        ));
    }
    for id in chain(document, id)? {
        trailer(document, &id)?.set(names::CARGO_MASS, Value::Float(mass))?;
    }
    Ok(())
}

pub(super) fn set_license_plate(
    document: &mut Document<'_>,
    id: &str,
    plate: String,
) -> Result<(), Error> {
    for id in chain(document, id)? {
        trailer(document, &id)?.set(names::LICENSE_PLATE, Value::String(plate.clone()))?;
    }
    Ok(())
}

pub(super) fn unlock_in_restricted_areas(
    document: &mut Document<'_>,
    id: &str,
) -> Result<(), Error> {
    let definition = {
        let unit = document
            .units_of_class(names::TRAILER)
            .find(|unit| unit.id() == id)
            .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
        text(unit, names::TRAILER_DEFINITION)?.to_owned()
    };
    let count = {
        let unit = document
            .units()
            .find(|unit| unit.id() == definition && unit.class() == names::TRAILER_DEF)
            .ok_or_else(|| Error::MissingUnit {
                id: definition.clone(),
            })?;
        if unit.get(names::COUNTRY_VALIDITY).is_none() {
            None
        } else {
            Some(length(unit, names::COUNTRY_VALIDITY)?)
        }
    };
    let Some(count) = count else {
        return Ok(());
    };
    let unit = document
        .get(&definition)
        .filter(|unit| unit.class() == names::TRAILER_DEF)
        .ok_or_else(|| Error::MissingUnit {
            id: definition.clone(),
        })?;
    for nth in 0..count {
        unit.remove_attr(&indexed(names::COUNTRY_VALIDITY, nth))?;
    }
    unit.set(names::COUNTRY_VALIDITY, Value::Int(0))?;
    unit.set(names::SOURCE_NAME, Value::token("null"))?;
    Ok(())
}

pub(super) fn set_current_trailer(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    if !document
        .units_of_class(names::TRAILER)
        .any(|unit| unit.id() == id)
    {
        return Err(Error::MissingUnit { id: id.to_owned() });
    }
    let slot = document
        .units_of_class(names::PLAYER)
        .next()
        .and_then(|player| reference(player, names::ASSIGNED_VEHICLES))
        .map(str::to_owned)
        .ok_or(Error::NoCurrentTrailer)?;
    let unit = document
        .get(&slot)
        .filter(|unit| unit.class() == names::PLAYER_VEHICLES)
        .ok_or_else(|| Error::MissingUnit { id: slot.clone() })?;
    unit.set(names::TRAILER, Value::token(id))?;
    Ok(())
}

fn repair(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    for id in chain(document, id)? {
        let unit = trailer(document, &id)?;
        for name in WEAR {
            if unit.get(name).is_none() {
                return Err(Error::missing(unit.class(), name));
            }
        }
        let mut wheels = Vec::new();
        for name in WHEELS {
            wheels.push((name, length(unit, name)?));
        }
        for name in WEAR {
            unit.set(name, Value::Int(0))?;
        }
        for (name, total) in wheels {
            for nth in 0..total {
                unit.set(&indexed(name, nth), Value::Int(0))?;
            }
        }
    }
    Ok(())
}

pub(super) fn chain(document: &Document<'_>, id: &str) -> Result<Vec<String>, Error> {
    let mut ids = Vec::new();
    let mut current = id.to_owned();
    for _ in 0..CHAIN_LIMIT {
        let unit = document
            .units_of_class(names::TRAILER)
            .find(|unit| unit.id() == current)
            .ok_or_else(|| Error::MissingUnit {
                id: current.clone(),
            })?;
        ids.push(current.clone());
        let Some(next) = reference(unit, names::SLAVE_TRAILER) else {
            break;
        };
        current = next.to_owned();
    }
    Ok(ids)
}

fn owned_trailers(document: &Document<'_>) -> Result<Vec<String>, Error> {
    let Some(player) = document.units_of_class(names::PLAYER).next() else {
        return Ok(Vec::new());
    };
    let mut trailers = Vec::new();
    if player.get(names::TRAILERS).is_some() {
        let count = length(player, names::TRAILERS)?;
        for nth in 0..count {
            if let Some(id) = player.get(&indexed(names::TRAILERS, nth)) {
                trailers.push(id.to_owned());
            }
        }
    }
    for slot in document.units_of_class(names::PLAYER_VEHICLES) {
        if let Some(trailer) = reference(slot, names::TRAILER) {
            trailers.push(trailer.to_owned());
        }
    }
    trailers.sort();
    trailers.dedup();
    Ok(trailers)
}

fn trailer<'a, 'd>(document: &'a mut Document<'d>, id: &str) -> Result<&'a mut Unit<'d>, Error> {
    match document.get(id) {
        Some(unit) if unit.class() == names::TRAILER => Ok(unit),
        _ => Err(Error::MissingUnit { id: id.to_owned() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/trailer.sii");

    fn source() -> Document<'static> {
        Document::parse(SOURCE).unwrap()
    }

    fn value<'a>(document: &'a Document<'_>, id: &str, name: &str) -> Option<&'a str> {
        document.units().find(|unit| unit.id() == id)?.get(name)
    }

    #[test]
    fn repairs_the_slave_chain() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::RepairTrailer("trailer.one".into()),
        )
        .unwrap();
        for id in ["trailer.one", "trailer.two"] {
            for name in WEAR {
                assert_eq!(value(&document, id, name), Some("0"), "{id} {name}");
            }
            assert_eq!(value(&document, id, "wheels_wear[0]"), Some("0"));
            assert_eq!(value(&document, id, "wheels_wear_unfixable[0]"), Some("0"));
        }
    }

    #[test]
    fn repairs_owned_trailers() {
        let mut document = source();
        apply(&mut document, Game::Ets2, Command::RepairAllTrailers).unwrap();
        assert_eq!(
            value(&document, "trailer.one", names::TRAILER_BODY_WEAR),
            Some("0")
        );
        assert_eq!(
            value(&document, "trailer.two", names::TRAILER_BODY_WEAR),
            Some("0")
        );
    }

    #[test]
    fn sets_cargo_mass_on_the_slave_chain() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetCargoMass {
                id: "trailer.one".into(),
                mass: 1500.0,
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "trailer.one", names::CARGO_MASS),
            Some("1500.0")
        );
        assert_eq!(
            value(&document, "trailer.two", names::CARGO_MASS),
            Some("1500.0")
        );
    }

    #[test]
    fn sets_the_plate_on_the_slave_chain() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetTrailerLicensePlate {
                id: "trailer.one".into(),
                plate: "NEW".into(),
            },
        )
        .unwrap();
        assert_eq!(
            value(&document, "trailer.one", names::LICENSE_PLATE),
            Some("\"NEW\"")
        );
        assert_eq!(
            value(&document, "trailer.two", names::LICENSE_PLATE),
            Some("\"NEW\"")
        );
    }

    #[test]
    fn unlocks_the_trailer_definition() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::UnlockInRestrictedAreas("trailer.one".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "trailer_def.test", names::COUNTRY_VALIDITY),
            Some("0")
        );
        assert_eq!(
            value(&document, "trailer_def.test", names::SOURCE_NAME),
            Some("null")
        );
        assert_eq!(
            value(&document, "trailer_def.test", "country_validity[0]"),
            None
        );
    }

    #[test]
    fn rejects_negative_cargo_mass() {
        let mut document = source();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetCargoMass {
                id: "trailer.one".into(),
                mass: -1.0,
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "invalid cargo_mass value \"-1\" on trailer"
        );
    }

    #[test]
    fn sets_the_current_trailer() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetCurrentTrailer("trailer.two".into()),
        )
        .unwrap();
        assert_eq!(
            value(&document, "slot.one", names::TRAILER),
            Some("trailer.two")
        );
    }
}
