use siir::{Document, Value};

use crate::error::Error;
use crate::names;

pub(super) fn rename(document: &mut Document<'_>, name: String) -> Result<(), Error> {
    let id = document
        .units_of_class(names::USER_PROFILE)
        .next()
        .map(|unit| unit.id().to_owned())
        .ok_or_else(|| Error::MissingUnit {
            id: names::USER_PROFILE.to_owned(),
        })?;
    document
        .get(&id)
        .filter(|unit| unit.class() == names::USER_PROFILE)
        .ok_or(Error::MissingUnit { id })?
        .set(names::PROFILE_NAME, Value::String(name))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/profile.sii");

    #[test]
    fn renames_the_profile() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::RenameProfile("new".into()),
        )
        .unwrap();
        let unit = document.units_of_class(names::USER_PROFILE).next().unwrap();
        assert_eq!(unit.get(names::PROFILE_NAME), Some("\"new\""));
    }
}
