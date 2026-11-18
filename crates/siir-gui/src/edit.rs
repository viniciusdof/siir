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
