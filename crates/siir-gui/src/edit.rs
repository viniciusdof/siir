//! Parses form text into save commands.

use siir_save::{Command, Skills};

/// Why a field's text is not accepted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldError {
    /// The text is empty, not a number, or out of range.
    Number,
    /// The text is blank or holds a character a SII string cannot carry.
    Name,
}

impl FieldError {
    /// Translation key shown next to the field.
    pub fn key(self) -> &'static str {
        match self {
            Self::Number => "error.number",
            Self::Name => "error.name",
        }
    }
}

/// Parses a profile name; surrounding spaces are dropped.
pub fn profile_name(text: &str) -> Result<Command, FieldError> {
    let name = text.trim();
    if name.is_empty() || name.contains(['"', '\n', '\r']) {
        return Err(FieldError::Name);
    }
    Ok(Command::RenameProfile(name.to_owned()))
}

/// Parses a bank balance.
pub fn money(text: &str) -> Result<Command, FieldError> {
    text.trim()
        .parse()
        .map(Command::SetMoney)
        .map_err(|_| FieldError::Number)
}

/// Parses total experience points.
pub fn experience(text: &str) -> Result<Command, FieldError> {
    text.trim()
        .parse()
        .map(Command::SetExperience)
        .map_err(|_| FieldError::Number)
}

/// Parses a vehicle's odometer in whole kilometers.
pub fn odometer(id: &str, text: &str) -> Result<Command, FieldError> {
    text.trim()
        .parse()
        .map(|km| Command::SetOdometer {
            id: id.to_owned(),
            km,
        })
        .map_err(|_| FieldError::Number)
}

/// Parses a trailer's cargo mass in whole kilograms.
pub fn cargo_mass(id: &str, text: &str) -> Result<Command, FieldError> {
    text.trim()
        .parse::<u32>()
        .map(|mass| Command::SetCargoMass {
            id: id.to_owned(),
            mass: mass as f32,
        })
        .map_err(|_| FieldError::Number)
}

/// Parses a trailer's plate as raw text; markup and country are dropped.
pub fn trailer_plate(id: &str, text: &str) -> Result<Command, FieldError> {
    let plate = text.trim();
    if plate.is_empty() || plate.contains(['"', '\n', '\r']) {
        return Err(FieldError::Name);
    }
    Ok(Command::SetTrailerLicensePlate {
        id: id.to_owned(),
        plate: plate.to_owned(),
    })
}

/// Sets a vehicle's fuel from a 0 to 100 slider position.
pub fn fuel(id: &str, percent: f32) -> Command {
    Command::SetFuel {
        id: id.to_owned(),
        level: (percent.round() / 100.0).clamp(0.0, 1.0),
    }
}

/// ADR class translation keys, least significant bit first.
pub const ADR_CLASSES: [&str; 6] = [
    "adr.explosives",
    "adr.gases",
    "adr.flammable_liquids",
    "adr.flammable_solids",
    "adr.toxic",
    "adr.corrosive",
];

/// Builds the player skills from five levels and the selected ADR classes.
///
/// The low six bits of `adr` are the ADR classes; higher bits are kept
/// untouched.
pub fn skills(levels: [u64; 5], classes: [bool; 6], adr: u64) -> Skills {
    let mut value = adr & !0x3f;
    for (ix, selected) in classes.into_iter().enumerate() {
        if selected {
            value |= 1 << ix;
        }
    }
    Skills {
        adr: value,
        long_dist: levels[0],
        heavy: levels[1],
        fragile: levels[2],
        urgent: levels[3],
        mechanical: levels[4],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_money_and_experience() {
        assert!(matches!(money(" 2000 "), Ok(Command::SetMoney(2000))));
        assert!(matches!(money("-5"), Ok(Command::SetMoney(-5))));
        assert!(matches!(experience("42"), Ok(Command::SetExperience(42))));
        assert!(matches!(experience("-1"), Err(FieldError::Number)));
        assert!(matches!(money(""), Err(FieldError::Number)));
        assert!(matches!(money("12.5"), Err(FieldError::Number)));
        assert!(matches!(
            money("99999999999999999999"),
            Err(FieldError::Number)
        ));
    }

    #[test]
    fn parses_odometers_and_fuel() {
        assert!(matches!(
            odometer("vehicle.one", " 1234 "),
            Ok(Command::SetOdometer { id, km: 1234 }) if id == "vehicle.one"
        ));
        assert!(matches!(odometer("v", "-1"), Err(FieldError::Number)));
        assert!(matches!(odometer("v", ""), Err(FieldError::Number)));
        assert!(matches!(
            fuel("v", 49.6),
            Command::SetFuel { level, .. } if level == 0.5
        ));
        assert!(matches!(fuel("v", 140.0), Command::SetFuel { level, .. } if level == 1.0));
        assert!(matches!(fuel("v", -3.0), Command::SetFuel { level, .. } if level == 0.0));
    }

    #[test]
    fn parses_trailer_mass_and_plate() {
        assert!(matches!(
            cargo_mass("t", " 1500 "),
            Ok(Command::SetCargoMass { mass, .. }) if mass == 1500.0
        ));
        assert!(matches!(cargo_mass("t", "-1"), Err(FieldError::Number)));
        assert!(matches!(cargo_mass("t", "1.5"), Err(FieldError::Number)));
        assert!(matches!(
            trailer_plate("t", " ABC 123 "),
            Ok(Command::SetTrailerLicensePlate { plate, .. }) if plate == "ABC 123"
        ));
        assert!(matches!(trailer_plate("t", ""), Err(FieldError::Name)));
        assert!(matches!(trailer_plate("t", "a\"b"), Err(FieldError::Name)));
    }

    #[test]
    fn parses_profile_names() {
        assert!(matches!(
            profile_name(" Road Runner "),
            Ok(Command::RenameProfile(name)) if name == "Road Runner"
        ));
        assert!(matches!(profile_name(""), Err(FieldError::Name)));
        assert!(matches!(profile_name("   "), Err(FieldError::Name)));
        assert!(matches!(profile_name("a\"b"), Err(FieldError::Name)));
        assert!(matches!(profile_name("a\nb"), Err(FieldError::Name)));
    }

    #[test]
    fn builds_skills_and_adr_classes() {
        assert_eq!(
            skills([1, 2, 3, 4, 5], [true, false, true, false, false, false], 0),
            Skills {
                adr: 0b000101,
                long_dist: 1,
                heavy: 2,
                fragile: 3,
                urgent: 4,
                mechanical: 5,
            }
        );
    }

    #[test]
    fn keeps_adr_bits_above_the_six_classes() {
        assert_eq!(skills([0; 5], [false; 6], 0b1100_0000).adr, 0b1100_0000);
    }
}
