//! Economy, skills, cities, dealerships, and garages.

use siir::{Document, Unit, Value};

use crate::Skills;
use crate::error::Error;
use crate::names::{self, indexed};
use crate::value::length;

pub(super) fn set_money(document: &mut Document<'_>, money: i64) -> Result<(), Error> {
    first(document, names::BANK)?.set("money_account", Value::Int(money))?;
    Ok(())
}

pub(super) fn set_experience(document: &mut Document<'_>, experience: u64) -> Result<(), Error> {
    first(document, names::ECONOMY)?.set(names::EXPERIENCE_POINTS, Value::UInt(experience))?;
    Ok(())
}

pub(super) fn set_skills(document: &mut Document<'_>, skills: Skills) -> Result<(), Error> {
    let unit = first(document, names::ECONOMY)?;
    unit.set(names::ADR, Value::UInt(skills.adr))?;
    unit.set(names::LONG_DIST, Value::UInt(skills.long_dist))?;
    unit.set(names::HEAVY, Value::UInt(skills.heavy))?;
    unit.set(names::FRAGILE, Value::UInt(skills.fragile))?;
    unit.set(names::URGENT, Value::UInt(skills.urgent))?;
    unit.set(names::MECHANICAL, Value::UInt(skills.mechanical))?;
    Ok(())
}

pub(super) fn set_visited_cities(document: &mut Document<'_>, all: bool) -> Result<(), Error> {
    let cities = if all { cities(document)? } else { Vec::new() };
    let unit = first(document, names::ECONOMY)?;
    rewrite(
        unit,
        names::VISITED_CITIES,
        cities
            .iter()
            .map(|city| Value::token(city.as_str()))
            .collect(),
    )?;
    rewrite(
        unit,
        names::VISITED_CITIES_COUNT,
        vec![Value::Int(1); cities.len()],
    )
}

pub(super) fn set_dealerships(document: &mut Document<'_>, all: bool) -> Result<(), Error> {
    let cities = if all { cities(document)? } else { Vec::new() };
    let unit = first(document, names::ECONOMY)?;
    rewrite(
        unit,
        names::UNLOCKED_DEALERS,
        cities
            .iter()
            .map(|city| Value::token(city.as_str()))
            .collect(),
    )
}

pub(super) fn set_garage_status(
    document: &mut Document<'_>,
    id: &str,
    status: u64,
) -> Result<(), Error> {
    let slots = match status {
        1 => None,
        2 => Some(3),
        3 => Some(5),
        6 => Some(1),
        _ => return Err(Error::InvalidGarageStatus { status }),
    };
    let unit = match document.get(id) {
        Some(unit) if unit.class() == names::GARAGE => unit,
        _ => return Err(Error::MissingUnit { id: id.to_owned() }),
    };
    for name in [names::VEHICLES, names::DRIVERS, names::TRAILERS] {
        let current = if unit.get(name).is_some() {
            length(unit, name)?
        } else {
            0
        };
        for nth in 0..current {
            if let Some(value) = unit.get(&indexed(name, nth))
                && value != "null"
                && value != "nil"
            {
                return Err(Error::GarageOccupied { id: id.to_owned() });
            }
        }
    }
    if let Some(slots) = slots {
        for name in [names::VEHICLES, names::DRIVERS, names::TRAILERS] {
            rewrite(unit, name, vec![Value::token("null"); slots])?;
        }
    }
    unit.set(names::STATUS, Value::UInt(status))?;
    Ok(())
}

fn first<'a, 'd>(
    document: &'a mut Document<'d>,
    class: &'a str,
) -> Result<&'a mut Unit<'d>, Error> {
    let id = document
        .units_of_class(class)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or_else(|| Error::MissingUnit {
            id: class.to_owned(),
        })?;
    document.get(&id).ok_or(Error::MissingUnit { id })
}

fn cities(document: &Document<'_>) -> Result<Vec<String>, Error> {
    let economy = document
        .units_of_class(names::ECONOMY)
        .next()
        .ok_or_else(|| Error::MissingUnit {
            id: names::ECONOMY.to_owned(),
        })?;
    let total = length(economy, names::COMPANIES)?;
    let mut cities = Vec::new();
    for nth in 0..total {
        if let Some(id) = economy.get(&indexed(names::COMPANIES, nth))
            && let Some(city) = id.rsplit('.').next()
        {
            cities.push(city.to_owned());
        }
    }
    cities.sort();
    cities.dedup();
    Ok(cities)
}

fn rewrite(unit: &mut Unit<'_>, name: &str, values: Vec<Value>) -> Result<(), Error> {
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
        let attr = indexed(name, nth);
        if unit.get(&attr).is_some() {
            unit.set(&attr, value)?;
        } else {
            unit.insert_attr(&attr, value)?;
        }
    }
    for nth in len..old {
        let attr = indexed(name, nth);
        if unit.get(&attr).is_some() {
            unit.remove_attr(&attr)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/economy.sii");

    fn source() -> Document<'static> {
        Document::parse(SOURCE).unwrap()
    }

    fn value<'a>(document: &'a Document<'_>, id: &str, name: &str) -> Option<&'a str> {
        document.units().find(|unit| unit.id() == id)?.get(name)
    }

    #[test]
    fn sets_money_experience_and_skills() {
        let mut document = source();
        apply(&mut document, Game::Ets2, Command::SetMoney(500)).unwrap();
        apply(&mut document, Game::Ets2, Command::SetExperience(1000)).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetSkills(Skills {
                adr: 1,
                long_dist: 2,
                heavy: 3,
                fragile: 4,
                urgent: 5,
                mechanical: 6,
            }),
        )
        .unwrap();
        assert_eq!(value(&document, "bank.one", "money_account"), Some("500"));
        assert_eq!(
            value(&document, "economy.one", names::EXPERIENCE_POINTS),
            Some("1000")
        );
        assert_eq!(value(&document, "economy.one", names::ADR), Some("1"));
        assert_eq!(
            value(&document, "economy.one", names::MECHANICAL),
            Some("6")
        );
    }

    #[test]
    fn sets_visited_cities_and_dealers() {
        let mut document = source();
        apply(&mut document, Game::Ets2, Command::SetVisitedCities(true)).unwrap();
        assert_eq!(
            value(&document, "economy.one", names::VISITED_CITIES),
            Some("2")
        );
        assert_eq!(
            value(&document, "economy.one", "visited_cities[0]"),
            Some("brno")
        );
        assert_eq!(
            value(&document, "economy.one", "visited_cities[1]"),
            Some("prague")
        );
        assert_eq!(
            value(&document, "economy.one", names::VISITED_CITIES_COUNT),
            Some("2")
        );
        assert_eq!(
            value(&document, "economy.one", "visited_cities_count[1]"),
            Some("1")
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::SetDealershipsDiscovered(true),
        )
        .unwrap();
        assert_eq!(
            value(&document, "economy.one", names::UNLOCKED_DEALERS),
            Some("2")
        );
        assert_eq!(
            value(&document, "economy.one", "unlocked_dealers[0]"),
            Some("brno")
        );

        apply(&mut document, Game::Ets2, Command::SetVisitedCities(false)).unwrap();
        assert_eq!(
            value(&document, "economy.one", names::VISITED_CITIES),
            Some("0")
        );
        assert_eq!(value(&document, "economy.one", "visited_cities[0]"), None);
        assert!(Document::parse(&document.to_text()).is_ok());
    }

    #[test]
    fn sets_garage_status() {
        let mut document = source();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetGarageStatus {
                id: "garage.one".into(),
                status: 2,
            },
        )
        .unwrap();
        assert_eq!(value(&document, "garage.one", names::STATUS), Some("2"));
        assert_eq!(value(&document, "garage.one", names::VEHICLES), Some("3"));
        assert_eq!(value(&document, "garage.one", "vehicles[2]"), Some("null"));
        assert_eq!(value(&document, "garage.one", names::DRIVERS), Some("3"));
        assert_eq!(value(&document, "garage.one", names::TRAILERS), Some("3"));
        assert_eq!(value(&document, "garage.one", "trailers[2]"), Some("null"));

        apply(
            &mut document,
            Game::Ets2,
            Command::SetGarageStatus {
                id: "garage.two".into(),
                status: 1,
            },
        )
        .unwrap();
        assert_eq!(value(&document, "garage.two", names::STATUS), Some("1"));
        assert_eq!(value(&document, "garage.two", names::VEHICLES), Some("1"));
    }

    #[test]
    fn reports_invalid_garage_status_and_occupied_garage() {
        let mut document = source();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetGarageStatus {
                id: "garage.one".into(),
                status: 5,
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "garage status 5 is not 1, 2, 3, or 6");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetGarageStatus {
                id: "garage.three".into(),
                status: 2,
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "garage garage.three still holds vehicles, drivers, or trailers"
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetGarageStatus {
                id: "garage.three".into(),
                status: 1,
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "garage garage.three still holds vehicles, drivers, or trailers"
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetGarageStatus {
                id: "garage.four".into(),
                status: 1,
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "garage garage.four still holds vehicles, drivers, or trailers"
        );
    }
}
