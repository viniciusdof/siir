use siir::{Document, Value};

use crate::error::Error;
use crate::names::{self, PLAYER_JOB};
use crate::value::reference;

pub(super) fn set_current_cargo(document: &mut Document<'_>, cargo: String) -> Result<(), Error> {
    let player = document
        .units_of_class(names::PLAYER)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or_else(|| Error::MissingUnit {
            id: names::PLAYER.to_owned(),
        })?;
    let job = document
        .get(&player)
        .and_then(|unit| reference(unit, "current_job"))
        .map(str::to_owned)
        .ok_or(Error::NoCurrentJob)?;
    document
        .get(&job)
        .filter(|unit| unit.class() == PLAYER_JOB && unit.id() == job.as_str())
        .ok_or(Error::MissingUnit { id: job.clone() })?
        .set("cargo", Value::Token(cargo))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

    #[test]
    fn sets_the_current_job_cargo() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetCurrentJobCargo("cargo.plastic".into()),
        )
        .unwrap();
        assert_eq!(
            document.get("player_job.one").unwrap().get("cargo"),
            Some("cargo.plastic")
        );
    }

    #[test]
    fn rejects_a_bad_cargo_token() {
        let mut document = Document::parse(SOURCE).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetCurrentJobCargo("cargo plastic".into()),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("invalid value: token has invalid characters")
        );
    }
}
