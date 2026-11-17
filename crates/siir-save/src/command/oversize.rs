//! Oversize market edits.

use siir::{Document, Value};

use crate::error::Error;
use crate::names;

pub(super) fn set_truck(document: &mut Document<'_>, id: &str, truck: &str) -> Result<(), Error> {
    super::edit(document, id, names::OVERSIZE_OFFER)?.set(names::TRUCK, Value::string(truck))?;
    Ok(())
}

pub(super) fn set_expiration(
    document: &mut Document<'_>,
    id: &str,
    time: u64,
) -> Result<(), Error> {
    super::edit(document, id, names::OVERSIZE_OFFER)?.set(names::EXPIRATION, Value::UInt(time))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

    #[test]
    fn sets_the_truck_and_expiration() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetOversizeTruck {
                id: "oversize.one".into(),
                truck: "oversize/scania_8x4".into(),
            },
        )
        .unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetOversizeExpiration {
                id: "oversize.one".into(),
                time: 900,
            },
        )
        .unwrap();
        let unit = document.get("oversize.one").unwrap();
        assert_eq!(unit.get(names::TRUCK), Some("\"oversize/scania_8x4\""));
        assert_eq!(unit.get(names::EXPIRATION), Some("900"));
    }

    #[test]
    fn rejects_a_missing_offer() {
        let mut document = Document::parse(SOURCE).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetOversizeTruck {
                id: "offer.one".into(),
                truck: "oversize/scania_8x4".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"offer.one\"");
    }
}
