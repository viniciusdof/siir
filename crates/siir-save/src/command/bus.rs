//! Bus stop edits.

use siir::{Document, Value};

use crate::error::Error;
use crate::names;

pub(super) fn discover(document: &mut Document<'_>) -> Result<(), Error> {
    let ids: Vec<String> = document
        .units_of_class(names::BUS_STOP)
        .map(|unit| unit.id().to_owned())
        .collect();
    if ids.is_empty() {
        return Err(Error::MissingUnit {
            id: names::BUS_STOP.to_owned(),
        });
    }
    for id in ids {
        let Some(unit) = document.get(&id) else {
            return Err(Error::MissingUnit { id });
        };
        if unit.get(names::DISCOVERED) == Some("true") {
            continue;
        }
        unit.set(names::DISCOVERED, Value::Bool(true))?;
    }
    Ok(())
}

pub(super) fn set_offer(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    super::set_player_ref(
        document,
        names::SELECTED_BUS_OFFER,
        id,
        names::LINE_OFFER_DATA,
    )
}

pub(super) fn set_owned(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    super::add_owned(document, names::BUSES, id, "bus")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/owned.sii");

    #[test]
    fn adds_a_bus_once() {
        let mut document = Document::parse(SOURCE).unwrap();
        for _ in 0..2 {
            apply(
                &mut document,
                Game::Ets2,
                Command::SetOwnedBus("bus.one".into()),
            )
            .unwrap();
        }
        let player = document.get("player.one").unwrap();
        assert_eq!(player.get(names::BUSES), Some("1"));
        assert_eq!(player.get("buses[0]"), Some("bus.one"));

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetOwnedBus("car.one".into()),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "vehicle \"car.one\" is not a bus");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetOwnedBus("player.one".into()),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"player.one\"");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetBusLineOffer("bus.one".into()),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"bus.one\"");
    }
}
