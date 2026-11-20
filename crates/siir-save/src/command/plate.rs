use siir::{Document, Value};

use crate::error::Error;
use crate::names;
use crate::value::unquote;

pub(super) fn set_country(
    document: &mut Document<'_>,
    id: &str,
    country: &str,
) -> Result<(), Error> {
    if !valid_country(country) {
        return Err(Error::InvalidCountry {
            country: country.to_owned(),
        });
    }
    let unit = match document.get(id) {
        Some(unit) if matches!(unit.class(), names::VEHICLE | names::TRAILER) => unit,
        _ => return Err(Error::MissingUnit { id: id.to_owned() }),
    };
    let raw = unit
        .get(names::LICENSE_PLATE)
        .ok_or_else(|| Error::missing(unit.class(), names::LICENSE_PLATE))?;
    let body = match unquote(raw).rsplit_once('|') {
        Some((body, _)) => body.to_owned(),
        None => unquote(raw).to_owned(),
    };
    unit.set(
        names::LICENSE_PLATE,
        Value::string(format!("{body}|{country}")),
    )?;
    Ok(())
}

fn valid_country(country: &str) -> bool {
    !country.is_empty()
        && country
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/read.sii");

    #[test]
    fn replaces_the_country() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetLicensePlateCountry {
                id: "vehicle.one".into(),
                country: "poland".into(),
            },
        )
        .unwrap();
        assert_eq!(
            document
                .get("vehicle.one")
                .unwrap()
                .get(names::LICENSE_PLATE),
            Some("\"PL<offset>12|poland\"")
        );
    }

    #[test]
    fn appends_a_country_and_rejects_a_bad_one() {
        let mut document = Document::parse(SOURCE).unwrap();
        apply(
            &mut document,
            Game::Ets2,
            Command::SetLicensePlateCountry {
                id: "trailer.one".into(),
                country: "poland".into(),
            },
        )
        .unwrap();
        assert_eq!(
            document
                .get("trailer.one")
                .unwrap()
                .get(names::LICENSE_PLATE),
            Some("\"TEST|poland\"")
        );

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetLicensePlateCountry {
                id: "vehicle.one".into(),
                country: "Poland".into(),
            },
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "license plate country \"Poland\" is invalid"
        );
    }

    #[test]
    fn rejects_a_missing_unit() {
        let mut document = Document::parse(SOURCE).unwrap();
        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetLicensePlateCountry {
                id: "used.one".into(),
                country: "poland".into(),
            },
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "missing unit \"used.one\"");
    }
}
