//! Player position commands.

use siir::{Document, Value};

use crate::error::Error;
use crate::names::{self, indexed};
use crate::value::reference;

pub(super) fn set_position(
    document: &mut Document<'_>,
    location: &str,
    rotation: &str,
) -> Result<(), Error> {
    let placement = format!("{location} {rotation}");
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
    unit.set(
        names::STORED_VEHICLE_PLACEMENT,
        Value::Vector(placement.clone()),
    )?;
    if let Some(total) = unit
        .get(names::STORED_TRAILER_PLACEMENTS)
        .and_then(|value| value.parse::<usize>().ok())
    {
        for nth in 0..total {
            unit.set(
                &indexed(names::STORED_TRAILER_PLACEMENTS, nth),
                Value::Vector(placement.clone()),
            )?;
        }
        if total > 0 {
            unit.set(names::STORED_TRAILER_ATTACHED, Value::Bool(true))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/truck.sii");

    fn value<'a>(document: &'a Document<'_>, id: &str, name: &str) -> Option<&'a str> {
        document.units().find(|unit| unit.id() == id)?.get(name)
    }

    #[test]
    fn sets_the_vehicle_and_trailer_placement() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetPlayerPosition {
                location: "(10, 20, 30)".into(),
                rotation: "(1; 0, 0, 0)".into(),
            },
        )
        .unwrap();
        let placement = Some("(10, 20, 30) (1; 0, 0, 0)");
        assert_eq!(
            value(&document, "slot.one", names::STORED_VEHICLE_PLACEMENT),
            placement
        );
        assert_eq!(
            value(&document, "slot.one", "stored_trailer_placements[0]"),
            placement
        );
        assert_eq!(
            value(&document, "slot.one", names::STORED_TRAILER_ATTACHED),
            Some("true")
        );
    }
}
