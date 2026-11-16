//! Errors raised while reading a save document.

use std::fmt;

/// A save read failure naming the unit class and attribute involved.
#[derive(Debug)]
pub enum Error {
    /// A required attribute is absent.
    MissingAttribute {
        /// Unit class being read.
        class: String,
        /// Attribute name that is absent.
        name: String,
    },
    /// An attribute holds a value the reader cannot decode.
    InvalidValue {
        /// Unit class being read.
        class: String,
        /// Attribute name that is malformed.
        name: String,
        /// Text the attribute holds.
        value: String,
    },
    /// A unit the command needs is absent.
    MissingUnit {
        /// Unit id that is absent.
        id: String,
    },
    /// The truck has no badge accessory.
    NoBadge {
        /// Truck unit id.
        id: String,
    },
    /// The truck is not assigned to any garage.
    NoGarage {
        /// Truck unit id.
        id: String,
    },
    /// The player has no assigned vehicle slot.
    NoCurrentTruck,
    /// The player has no trailer in the assigned vehicle slot.
    NoCurrentTrailer,
    /// A fuel level is outside the range the game accepts.
    InvalidFuel {
        /// Requested level.
        level: f32,
    },
    /// The document rejected the edit.
    Document(siir::Error),
}

impl Error {
    pub(crate) fn missing(class: &str, name: &str) -> Self {
        Self::MissingAttribute {
            class: class.to_owned(),
            name: name.to_owned(),
        }
    }

    pub(crate) fn invalid(class: &str, name: &str, value: &str) -> Self {
        Self::InvalidValue {
            class: class.to_owned(),
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingAttribute { class, name } => {
                write!(f, "missing attribute {name:?} on {class}")
            }
            Self::InvalidValue { class, name, value } => {
                write!(f, "invalid {name} value {value:?} on {class}")
            }
            Self::MissingUnit { id } => write!(f, "missing unit {id:?}"),
            Self::NoBadge { id } => write!(f, "no badge on {id}"),
            Self::NoGarage { id } => write!(f, "no garage holds {id}"),
            Self::NoCurrentTruck => write!(f, "player has no current truck"),
            Self::NoCurrentTrailer => write!(f, "player has no current trailer"),
            Self::InvalidFuel { level } => {
                write!(f, "fuel level {level} is not between 0 and 1")
            }
            Self::Document(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Document(error) => Some(error),
            _ => None,
        }
    }
}

impl From<siir::Error> for Error {
    fn from(error: siir::Error) -> Self {
        Self::Document(error)
    }
}
