//! AI driver edits.

use siir::Document;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

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
    fn rejects_a_missing_driver() {
        let mut document = Document::parse(SOURCE).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetDriverSkills {
                id: "job.one".into(),
                skills: Skills::default(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"job.one\"");
    }
}
