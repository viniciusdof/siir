use siir::{Document, Unit, Value};

use super::{first_id, push, tokens, values};
use crate::Skills;
use crate::error::Error;
use crate::names;

pub(super) fn set_skills(
    document: &mut Document<'_>,
    id: &str,
    skills: Skills,
) -> Result<(), Error> {
    super::write_skills(super::edit(document, id, names::DRIVER_AI)?, skills)
}

pub(super) fn hire(document: &mut Document<'_>, id: &str, garage: &str) -> Result<(), Error> {
    super::check(document, id, names::DRIVER_AI)?;
    super::check(document, garage, names::GARAGE)?;

    let slot = {
        let unit = document
            .units()
            .find(|unit| unit.id() == garage)
            .ok_or(Error::MissingUnit {
                id: garage.to_owned(),
            })?;
        values(unit, names::DRIVERS)?
            .iter()
            .position(|value| value == "null")
            .ok_or_else(|| Error::NoDriverSlot {
                id: garage.to_owned(),
            })?
    };

    let economy = first_id(document, names::ECONOMY)?;
    let player = first_id(document, names::PLAYER)?;
    let pool = values(unit_of(document, &economy)?, names::DRIVER_POOL)?;
    let dismissed = values(unit_of(document, &player)?, names::DISMISSED_DRIVERS)?;
    let in_pool = pool.iter().any(|value| value == id);
    let is_dismissed = dismissed.iter().any(|value| value == id);
    if !in_pool && !is_dismissed {
        return Err(Error::DriverNotAvailable { id: id.to_owned() });
    }

    super::edit(document, garage, names::GARAGE)?
        .set(&names::indexed(names::DRIVERS, slot), Value::token(id))?;

    if in_pool {
        let mut pool = pool;
        pool.retain(|value| value != id);
        rewrite(document, &economy, names::ECONOMY, names::DRIVER_POOL, pool)?;
    }
    if is_dismissed {
        let mut dismissed = dismissed;
        dismissed.retain(|value| value != id);
        rewrite(
            document,
            &player,
            names::PLAYER,
            names::DISMISSED_DRIVERS,
            dismissed,
        )?;
    }

    let already_employed = values(unit_of(document, &player)?, names::DRIVERS)?
        .iter()
        .any(|value| value == id);
    if !already_employed {
        let unit = super::edit(document, &player, names::PLAYER)?;
        for (name, value) in [
            (names::DRIVERS, id),
            (names::DRIVER_FLAGS, "0"),
            (names::DRIVER_READINESS_TIMER, "0"),
            (names::DRIVER_UNDRIVABLE_TRUCK_TIMERS, "0"),
            (names::DRIVER_QUIT_WARNED, "false"),
        ] {
            push(unit, name, value)?;
        }
    }

    super::edit(document, id, names::DRIVER_AI)?.set(names::STATE, Value::UInt(2))?;
    Ok(())
}

pub(super) fn fire(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    super::check(document, id, names::DRIVER_AI)?;
    let player = first_id(document, names::PLAYER)?;
    let index = {
        let unit = unit_of(document, &player)?;
        let drivers = values(unit, names::DRIVERS)?;
        let dismissed = values(unit, names::DISMISSED_DRIVERS)?;
        if dismissed.iter().any(|value| value == id) {
            return Ok(());
        }
        drivers
            .iter()
            .position(|value| value == id)
            .ok_or_else(|| Error::DriverNotEmployed { id: id.to_owned() })?
    };

    if let Some((garage, slot)) = driver_garage(document, id)? {
        super::edit(document, &garage, names::GARAGE)?
            .set(&names::indexed(names::DRIVERS, slot), Value::token("null"))?;
    }

    if active_job(document, id)? {
        let mut dismissed = values(unit_of(document, &player)?, names::DISMISSED_DRIVERS)?;
        dismissed.push(id.to_owned());
        rewrite(
            document,
            &player,
            names::PLAYER,
            names::DISMISSED_DRIVERS,
            dismissed,
        )?;
        return Ok(());
    }

    {
        let unit = super::edit(document, &player, names::PLAYER)?;
        for name in [
            names::DRIVERS,
            names::DRIVER_FLAGS,
            names::DRIVER_READINESS_TIMER,
            names::DRIVER_UNDRIVABLE_TRUCK_TIMERS,
            names::DRIVER_QUIT_WARNED,
        ] {
            let mut list = values(unit, name)?;
            if index < list.len() {
                list.remove(index);
            }
            super::rewrite(unit, name, tokens(list))?;
        }
    }

    let economy = first_id(document, names::ECONOMY)?;
    let mut pool = values(unit_of(document, &economy)?, names::DRIVER_POOL)?;
    pool.push(id.to_owned());
    rewrite(document, &economy, names::ECONOMY, names::DRIVER_POOL, pool)?;

    to_pool(super::edit(document, id, names::DRIVER_AI)?)
}

pub(super) fn set_truck(document: &mut Document<'_>, id: &str, truck: &str) -> Result<(), Error> {
    super::check(document, id, names::DRIVER_AI)?;
    super::check(document, truck, names::VEHICLE)?;
    let player = first_id(document, names::PLAYER)?;
    if !values(unit_of(document, &player)?, names::DRIVERS)?
        .iter()
        .any(|value| value == id)
    {
        return Err(Error::DriverNotEmployed { id: id.to_owned() });
    }

    let Some((garage, driver_slot)) = driver_garage(document, id)? else {
        return Err(Error::NoSharedGarage {
            driver: id.to_owned(),
            truck: truck.to_owned(),
        });
    };
    let truck_slot = {
        let unit = unit_of(document, &garage)?;
        let vehicles = values(unit, names::VEHICLES)?;
        let Some(slot) = vehicles.iter().position(|value| value == truck) else {
            return Err(Error::NoSharedGarage {
                driver: id.to_owned(),
                truck: truck.to_owned(),
            });
        };
        slot
    };
    if driver_slot == truck_slot {
        return Ok(());
    }

    {
        let unit = super::edit(document, &garage, names::GARAGE)?;
        let mut vehicles = values(unit, names::VEHICLES)?;
        let current = vehicles[driver_slot].clone();
        vehicles[driver_slot] = truck.to_owned();
        vehicles[truck_slot] = current;
        super::rewrite(unit, names::VEHICLES, tokens(vehicles))?;
    }
    Ok(())
}

fn rewrite(
    document: &mut Document<'_>,
    id: &str,
    class: &str,
    name: &str,
    values: Vec<String>,
) -> Result<(), Error> {
    super::rewrite(super::edit(document, id, class)?, name, tokens(values))
}

fn unit_of<'a, 'd>(document: &'a Document<'d>, id: &str) -> Result<&'a Unit<'d>, Error> {
    document
        .units()
        .find(|unit| unit.id() == id)
        .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })
}

fn driver_garage(document: &Document<'_>, id: &str) -> Result<Option<(String, usize)>, Error> {
    for unit in document.units_of_class(names::GARAGE) {
        if let Some(slot) = values(unit, names::DRIVERS)?
            .iter()
            .position(|value| value == id)
        {
            return Ok(Some((unit.id().to_owned(), slot)));
        }
    }
    Ok(None)
}

fn active_job(document: &Document<'_>, id: &str) -> Result<bool, Error> {
    let driver = unit_of(document, id)?;
    let Some(job) = driver
        .get("driver_job")
        .filter(|job| *job != "null" && *job != "nil")
    else {
        return Ok(false);
    };
    let Some(job) = document.units().find(|unit| unit.id() == job) else {
        return Ok(false);
    };
    Ok(matches!(job.get("cargo"), Some(cargo) if cargo != "null" && cargo != "nil"))
}

fn to_pool(unit: &mut Unit<'_>) -> Result<(), Error> {
    for (name, value) in [
        ("hometown", Value::string("")),
        ("current_city", Value::string("")),
        (names::STATE, Value::UInt(1)),
        ("on_duty_timer", Value::UInt(0)),
        ("extra_maintenance", Value::UInt(0)),
        (names::TRAINING_POLICY, Value::UInt(0)),
        ("adopted_truck", Value::token("null")),
        ("assigned_truck", Value::token("null")),
        ("assigned_truck_efficiency", Value::UInt(1)),
        ("assigned_truck_axle_count", Value::UInt(2)),
        ("assigned_truck_mass", Value::UInt(10000)),
        ("slot_truck_efficiency", Value::UInt(1)),
        ("slot_truck_axle_count", Value::UInt(2)),
        ("slot_truck_mass", Value::UInt(10000)),
        ("adopted_trailer", Value::token("null")),
        ("assigned_trailer", Value::token("null")),
        ("old_hometown", Value::string("")),
    ] {
        if unit.get(name).is_some() {
            unit.set(name, value)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");
    const DRIVERS: &str = include_str!("../../testdata/drivers.sii");

    fn value<'a>(document: &'a Document<'_>, id: &str, name: &str) -> Option<&'a str> {
        document.units().find(|unit| unit.id() == id)?.get(name)
    }

    #[test]
    fn sets_the_driver_skills() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetDriverSkills {
                id: "driver.one".into(),
                skills: Skills {
                    adr: 6,
                    long_dist: 5,
                    heavy: 4,
                    fragile: 3,
                    urgent: 2,
                    mechanical: 1,
                },
            },
        )
        .unwrap();
        let driver = document.get("driver.one").unwrap();
        assert_eq!(driver.get(names::ADR), Some("6"));
        assert_eq!(driver.get(names::LONG_DIST), Some("5"));
        assert_eq!(driver.get(names::MECHANICAL), Some("1"));
    }

    #[test]
    fn hires_into_a_free_garage_slot() {
        let mut document = Document::parse(DRIVERS).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::HireDriver {
                id: "driver.one".into(),
                garage: "garage.prague".into(),
            },
        )
        .unwrap();
        assert_eq!(value(&document, "player.one", names::DRIVERS), Some("2"));
        assert_eq!(
            value(&document, "player.one", "drivers[1]"),
            Some("driver.one")
        );
        assert_eq!(
            value(&document, "player.one", names::DRIVER_FLAGS),
            Some("2")
        );
        assert_eq!(
            value(&document, "player.one", names::DRIVER_READINESS_TIMER),
            Some("2")
        );
        assert_eq!(
            value(
                &document,
                "player.one",
                names::DRIVER_UNDRIVABLE_TRUCK_TIMERS
            ),
            Some("2")
        );
        assert_eq!(
            value(&document, "player.one", names::DRIVER_QUIT_WARNED),
            Some("2")
        );
        assert_eq!(
            value(&document, "economy.one", names::DRIVER_POOL),
            Some("1")
        );
        assert_eq!(
            value(&document, "economy.one", "driver_pool[0]"),
            Some("driver.two")
        );
        assert_eq!(
            value(&document, "garage.prague", "drivers[1]"),
            Some("driver.one")
        );
        assert_eq!(value(&document, "driver.one", names::STATE), Some("2"));
    }

    #[test]
    fn fires_an_idle_driver_back_to_the_pool() {
        let mut document = Document::parse(DRIVERS).unwrap();
        for command in [
            Command::HireDriver {
                id: "driver.one".into(),
                garage: "garage.prague".into(),
            },
            Command::FireDriver("driver.one".into()),
        ] {
            apply(&mut document, Game::Ets2, command).unwrap();
        }
        assert_eq!(value(&document, "player.one", names::DRIVERS), Some("1"));
        assert_eq!(
            value(&document, "player.one", names::DRIVER_READINESS_TIMER),
            Some("1")
        );
        assert_eq!(
            value(&document, "economy.one", "driver_pool[1]"),
            Some("driver.one")
        );
        assert_eq!(
            value(&document, "garage.prague", "drivers[1]"),
            Some("null")
        );
        assert_eq!(value(&document, "driver.one", names::STATE), Some("1"));
        assert_eq!(value(&document, "driver.one", "hometown"), Some("\"\""));
    }

    #[test]
    fn fires_a_driver_on_a_job_into_the_dismissed_list() {
        let mut document = Document::parse(DRIVERS).unwrap();
        for command in [
            Command::HireDriver {
                id: "driver.two".into(),
                garage: "garage.prague".into(),
            },
            Command::FireDriver("driver.two".into()),
        ] {
            apply(&mut document, Game::Ets2, command).unwrap();
        }
        assert_eq!(value(&document, "player.one", names::DRIVERS), Some("2"));
        assert_eq!(
            value(&document, "player.one", names::DISMISSED_DRIVERS),
            Some("1")
        );
        assert_eq!(
            value(&document, "player.one", "dismissed_drivers[0]"),
            Some("driver.two")
        );
        assert_eq!(
            value(&document, "garage.prague", "drivers[1]"),
            Some("null")
        );
        assert_eq!(value(&document, "driver.two", names::STATE), Some("2"));
    }

    #[test]
    fn rehires_a_dismissed_driver_without_duplicating_the_roster() {
        let mut document = Document::parse(DRIVERS).unwrap();
        for command in [
            Command::HireDriver {
                id: "driver.two".into(),
                garage: "garage.prague".into(),
            },
            Command::FireDriver("driver.two".into()),
            Command::HireDriver {
                id: "driver.two".into(),
                garage: "garage.prague".into(),
            },
        ] {
            apply(&mut document, Game::Ets2, command).unwrap();
        }
        assert_eq!(value(&document, "player.one", names::DRIVERS), Some("2"));
        assert_eq!(
            value(&document, "player.one", names::DRIVER_FLAGS),
            Some("2")
        );
        assert_eq!(
            value(&document, "player.one", names::DISMISSED_DRIVERS),
            Some("0")
        );
        assert_eq!(
            value(&document, "garage.prague", "drivers[1]"),
            Some("driver.two")
        );
    }

    #[test]
    fn pairs_a_driver_with_a_truck_in_its_garage() {
        let mut document = Document::parse(DRIVERS).unwrap();
        for command in [
            Command::HireDriver {
                id: "driver.one".into(),
                garage: "garage.prague".into(),
            },
            Command::SetDriverTruck {
                id: "driver.one".into(),
                truck: "vehicle.one".into(),
            },
        ] {
            apply(&mut document, Game::Ets2, command).unwrap();
        }
        assert_eq!(
            value(&document, "garage.prague", "vehicles[0]"),
            Some("vehicle.two")
        );
        assert_eq!(
            value(&document, "garage.prague", "vehicles[1]"),
            Some("vehicle.one")
        );
        assert_eq!(
            value(&document, "garage.prague", "drivers[1]"),
            Some("driver.one")
        );
    }

    #[test]
    fn rejects_missing_and_unavailable_drivers() {
        let mut document = Document::parse(DRIVERS).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::HireDriver {
                id: "driver_player.one".into(),
                garage: "garage.prague".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"driver_player.one\"");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::FireDriver("driver.one".into()),
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "driver \"driver.one\" is not in the player's roster"
        );

        apply(
            &mut document,
            Game::Ets2,
            Command::HireDriver {
                id: "driver.one".into(),
                garage: "garage.prague".into(),
            },
        )
        .unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::HireDriver {
                id: "driver.two".into(),
                garage: "garage.prague".into(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "garage garage.prague has no free driver slot"
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetDriverTruck {
                id: "driver.one".into(),
                truck: "vehicle.three".into(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "driver \"driver.one\" and truck \"vehicle.three\" are not in the same garage"
        );
    }
}
