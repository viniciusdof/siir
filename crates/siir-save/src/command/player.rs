//! Player position commands.

use siir::{Document, Value};

use crate::error::Error;
use crate::names::{self, indexed};
use crate::value::{length, reference};

pub(super) fn set_position(
    document: &mut Document<'_>,
    location: &str,
    rotation: &str,
) -> Result<(), Error> {
    if !is_triple(location) || !is_quad(rotation) {
        return Err(Error::invalid(
            "placement",
            "position",
            &format!("{location} {rotation}"),
        ));
    }
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
    if unit.get(names::STORED_TRAILER_PLACEMENTS).is_some() {
        let total = length(unit, names::STORED_TRAILER_PLACEMENTS)?;
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

fn is_triple(text: &str) -> bool {
    !text.contains(';') && components(text).is_some_and(|parts| parts.len() == 3)
}

fn is_quad(text: &str) -> bool {
    text.matches(';').count() == 1 && components(text).is_some_and(|parts| parts.len() == 4)
}

fn components(text: &str) -> Option<Vec<&str>> {
    let inner = text.strip_prefix('(')?.strip_suffix(')')?;
    let parts: Vec<&str> = inner
        .split([';', ','])
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect();
    parts.iter().all(|part| is_number(part)).then_some(parts)
}

fn is_number(token: &str) -> bool {
    token.parse::<f32>().is_ok()
        || token
            .strip_prefix('&')
            .is_some_and(|hex| u32::from_str_radix(hex, 16).is_ok())
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

    #[test]
    fn rejects_a_malformed_placement() {
        let mut document = Document::parse(SOURCE).unwrap();
        for (location, rotation) in [("(1)", "(2)"), ("(a, b, c)", "(1; 0, 0, 0)")] {
            let error = apply(
                &mut document,
                Game::Ets2,
                Command::SetPlayerPosition {
                    location: location.into(),
                    rotation: rotation.into(),
                },
            )
            .unwrap_err();
            assert!(
                error.to_string().contains("invalid position value"),
                "{error}"
            );
        }

        apply(
            &mut document,
            Game::Ets2,
            Command::SetPlayerPosition {
                location: "(&462e851f, &42a4b610, &451ef723)".into(),
                rotation: "(&be3dd667; &bb9f085c, &bf7b8256, &3ca0be78)".into(),
            },
        )
        .unwrap();
    }
}
