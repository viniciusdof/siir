//! Car edits.

use siir::Document;

use crate::error::Error;
use crate::names;

pub(super) fn set_owned(document: &mut Document<'_>, id: &str) -> Result<(), Error> {
    super::add_owned(document, names::CARS, id, "car")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Game;
    use crate::command::{Command, apply};

    const SOURCE: &str = include_str!("../../testdata/owned.sii");

    #[test]
    fn adds_a_car_once() {
        let mut document = Document::parse(SOURCE).unwrap();
        for _ in 0..2 {
            apply(
                &mut document,
                Game::Ets2,
                Command::SetOwnedCar("car.one".into()),
            )
            .unwrap();
        }
        let player = document.get("player.one").unwrap();
        assert_eq!(player.get(names::CARS), Some("1"));
        assert_eq!(player.get("cars[0]"), Some("car.one"));

        let error = apply(
            &mut document,
            Game::Ets2,
            Command::SetOwnedCar("truck.one".into()),
        )
        .unwrap_err();
        assert_eq!(error.to_string(), "vehicle \"truck.one\" is not a car");
    }
}
