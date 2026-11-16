//! Used-truck market commands.

use siir::{Document, Value};

use crate::error::Error;
use crate::names;

pub(super) fn set_price(document: &mut Document<'_>, id: &str, price: i64) -> Result<(), Error> {
    if price < 0 {
        return Err(Error::InvalidPrice { price });
    }
    let unit = document
        .get(id)
        .filter(|unit| unit.class() == names::USED_TRUCK_OFFER && unit.id() == id)
        .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })?;
    unit.set("price", Value::Int(price))?;
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
}
