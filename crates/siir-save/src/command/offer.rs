use siir::{Document, Value};

use crate::error::Error;
use crate::names;

pub(super) fn set_units(document: &mut Document<'_>, id: &str, units: u64) -> Result<(), Error> {
    super::edit(document, id, names::JOB_OFFER_DATA)?
        .set(names::UNITS_COUNT, Value::UInt(units))?;
    Ok(())
}

pub(super) fn set_cargo(document: &mut Document<'_>, id: &str, cargo: String) -> Result<(), Error> {
    super::edit(document, id, names::JOB_OFFER_DATA)?.set("cargo", Value::Token(cargo))?;
    Ok(())
}

pub(super) fn set_target(
    document: &mut Document<'_>,
    id: &str,
    target: String,
) -> Result<(), Error> {
    super::edit(document, id, names::JOB_OFFER_DATA)?.set("target", Value::String(target))?;
    Ok(())
}

pub(super) fn set_distance(document: &mut Document<'_>, id: &str, km: u64) -> Result<(), Error> {
    super::edit(document, id, names::JOB_OFFER_DATA)?
        .set("shortest_distance_km", Value::UInt(km))?;
    Ok(())
}

pub(super) fn set_fill(document: &mut Document<'_>, id: &str, fill: f32) -> Result<(), Error> {
    if !(0.0..=1.0).contains(&fill) {
        return Err(Error::InvalidFill { fill });
    }
    super::edit(document, id, names::JOB_OFFER_DATA)?.set(names::FILL_RATIO, Value::Float(fill))?;
    Ok(())
}

pub(super) fn set_urgency(
    document: &mut Document<'_>,
    id: &str,
    urgency: u64,
) -> Result<(), Error> {
    super::edit(document, id, names::JOB_OFFER_DATA)?.set(names::URGENCY, Value::UInt(urgency))?;
    Ok(())
}

pub(super) fn set_expiration(
    document: &mut Document<'_>,
    id: &str,
    time: u64,
) -> Result<(), Error> {
    super::edit(document, id, names::JOB_OFFER_DATA)?
        .set(names::EXPIRATION_TIME, Value::UInt(time))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

    fn value<'a>(document: &'a Document<'_>, name: &str) -> Option<&'a str> {
        document
            .units()
            .find(|unit| unit.id() == "offer.one")?
            .get(name)
    }

    #[test]
    fn sets_every_editable_field() {
        let mut document = Document::parse(SOURCE).unwrap();
        for command in [
            Command::SetOfferUnits {
                id: "offer.one".into(),
                units: 12,
            },
            Command::SetOfferCargo {
                id: "offer.one".into(),
                cargo: "cargo.plastic".into(),
            },
            Command::SetOfferTarget {
                id: "offer.one".into(),
                target: "transinet.ostrava".into(),
            },
            Command::SetOfferDistance {
                id: "offer.one".into(),
                km: 150,
            },
            Command::SetOfferFill {
                id: "offer.one".into(),
                fill: 0.5,
            },
            Command::SetOfferUrgency {
                id: "offer.one".into(),
                urgency: 1,
            },
            Command::SetOfferExpiration {
                id: "offer.one".into(),
                time: 900,
            },
        ] {
            apply(&mut document, Game::Ets2, command).unwrap();
        }
        assert_eq!(value(&document, names::UNITS_COUNT), Some("12"));
        assert_eq!(value(&document, "cargo"), Some("cargo.plastic"));
        assert_eq!(value(&document, "target"), Some("\"transinet.ostrava\""));
        assert_eq!(value(&document, "shortest_distance_km"), Some("150"));
        assert_eq!(value(&document, names::FILL_RATIO), Some("0.5"));
        assert_eq!(value(&document, names::URGENCY), Some("1"));
        assert_eq!(value(&document, names::EXPIRATION_TIME), Some("900"));
    }

    #[test]
    fn rejects_a_bad_fill_and_a_missing_offer() {
        let mut document = Document::parse(SOURCE).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetOfferFill {
                id: "offer.one".into(),
                fill: 1.5,
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "fill ratio 1.5 is not between 0 and 1");

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetOfferUnits {
                id: "driver.one".into(),
                units: 1,
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"driver.one\"");
    }
}
