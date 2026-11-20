use siir::{Document, Value};

use crate::error::Error;
use crate::names::{
    self, DETAILED_HISTORY_ENTRIES, OFFENCE_TOTAL_COUNTS, OFFENCE_TOTAL_FINES, indexed,
};
use crate::value::length;

pub(super) fn clear(document: &mut Document<'_>) -> Result<(), Error> {
    let id = document
        .units_of_class(names::POLICE_OFFENCE_LOG)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or_else(|| Error::MissingUnit {
            id: names::POLICE_OFFENCE_LOG.to_owned(),
        })?;
    let unit = document
        .get(&id)
        .ok_or_else(|| Error::MissingUnit { id: id.clone() })?;
    for name in [OFFENCE_TOTAL_COUNTS, OFFENCE_TOTAL_FINES] {
        for nth in 0..length(unit, name)? {
            unit.set(&indexed(name, nth), Value::Int(0))?;
        }
    }
    unit.set(DETAILED_HISTORY_ENTRIES, Value::UInt(0))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

    #[test]
    fn zeroes_the_counts_the_fines_and_the_history() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(&mut document, Game::Ets2, Command::ClearOffences).unwrap();
        let log = document.get("offences").unwrap();
        assert_eq!(log.get(OFFENCE_TOTAL_COUNTS), Some("2"));
        assert_eq!(log.get(&indexed(OFFENCE_TOTAL_COUNTS, 0)), Some("0"));
        assert_eq!(log.get(&indexed(OFFENCE_TOTAL_COUNTS, 1)), Some("0"));
        assert_eq!(log.get(&indexed(OFFENCE_TOTAL_FINES, 0)), Some("0"));
        assert_eq!(log.get(&indexed(OFFENCE_TOTAL_FINES, 1)), Some("0"));
        assert_eq!(log.get(DETAILED_HISTORY_ENTRIES), Some("0"));
    }
}
