//! Used-truck market commands.

use siir::{Document, Value};

use crate::error::Error;
use crate::names;

pub(super) fn set_price(document: &mut Document<'_>, id: &str, price: i64) -> Result<(), Error> {
    if price < 0 {
        return Err(Error::InvalidPrice { price });
    }
    super::edit(document, id, names::USED_TRUCK_OFFER)?.set("price", Value::Int(price))?;
    Ok(())
}

pub(super) fn set_expiration(
    document: &mut Document<'_>,
    id: &str,
    time: u64,
) -> Result<(), Error> {
    super::edit(document, id, names::USED_TRUCK_OFFER)?
        .set(names::EXPIRATION_GAME_TIME, Value::UInt(time))?;
    Ok(())
}

pub(super) fn set_assortment_generation(
    document: &mut Document<'_>,
    time: u64,
) -> Result<(), Error> {
    let id = document
        .units_of_class(names::USED_VEHICLE_ASSORTMENT)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or_else(|| Error::MissingUnit {
            id: names::USED_VEHICLE_ASSORTMENT.to_owned(),
        })?;
    let unit = document
        .get(&id)
        .ok_or_else(|| Error::MissingUnit { id: id.clone() })?;
    unit.set(names::NEXT_GENERATION_GAME_TIME, Value::UInt(time))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

    #[test]
    fn sets_the_price() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetUsedTruckPrice {
                id: "used.one".into(),
                price: 250,
            },
        )
        .unwrap();
        let unit = document.get("used.one").unwrap();
        assert_eq!(unit.get("price"), Some("250"));

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetUsedTruckPrice {
                id: "used_truck_offer".into(),
                price: 100,
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"used_truck_offer\"");
        assert_eq!(document.get("used.one").unwrap().get("price"), Some("250"));

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetUsedTruckPrice {
                id: "used.one".into(),
                price: -1,
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "used truck price -1 is negative");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetUsedTruckPrice {
                id: "used.missing".into(),
                price: 100,
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"used.missing\"");
    }

    #[test]
    fn sets_the_expiration_and_the_assortment_generation() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetUsedTruckExpiration {
                id: "used.one".into(),
                time: 900,
            },
        )
        .unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetUsedAssortmentNextGeneration(700),
        )
        .unwrap();
        assert_eq!(
            document
                .get("used.one")
                .unwrap()
                .get(names::EXPIRATION_GAME_TIME),
            Some("900")
        );
        assert_eq!(
            document
                .get("used_assortment.one")
                .unwrap()
                .get(names::NEXT_GENERATION_GAME_TIME),
            Some("700")
        );
    }
}
