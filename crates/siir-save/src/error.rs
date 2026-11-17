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
    /// The vehicle has no badge accessory.
    NoBadge {
        /// Vehicle unit id.
        id: String,
    },
    /// The vehicle has no engine accessory.
    NoEngine {
        /// Vehicle unit id.
        id: String,
    },
    /// The vehicle has no transmission accessory.
    NoTransmission {
        /// Vehicle unit id.
        id: String,
    },
    /// The vehicle has no cabin accessory.
    NoCabin {
        /// Vehicle unit id.
        id: String,
    },
    /// The vehicle has no chassis accessory.
    NoChassis {
        /// Vehicle unit id.
        id: String,
    },
    /// The vehicle has no paint job accessory.
    NoPaint {
        /// Vehicle unit id.
        id: String,
    },
    /// The vehicle does not reference the accessory.
    NoAccessory {
        /// Vehicle unit id.
        id: String,
        /// Accessory unit id.
        accessory: String,
    },
    /// A required vehicle part cannot be removed.
    CannotRemovePart {
        /// Part name, for example `engine`.
        part: &'static str,
    },
    /// A used-truck price is negative.
    InvalidPrice {
        /// Requested price.
        price: i64,
    },
    /// An imported block contains a unit that is not an accessory.
    InvalidAccessoryClass {
        /// Unit class found in the block.
        class: String,
    },
    /// An imported block is missing a required vehicle part.
    MissingAccessoryPart {
        /// Part name, for example `engine`.
        part: &'static str,
    },
    /// The vehicle is not assigned to any garage.
    NoGarage {
        /// Vehicle unit id.
        id: String,
    },
    /// The player has no assigned vehicle slot.
    NoCurrentVehicle,
    /// The player has no trailer in the assigned vehicle slot.
    NoCurrentTrailer,
    /// The player has no current job.
    NoCurrentJob,
    /// A fuel level is outside the range the game accepts.
    InvalidFuel {
        /// Requested level.
        level: f32,
    },
    /// A garage status is outside the set the game writes.
    InvalidGarageStatus {
        /// Requested status.
        status: u64,
    },
    /// A license plate country is not a lowercase country token.
    InvalidCountry {
        /// Requested country.
        country: String,
    },
    /// An offer fill ratio is outside the range the game accepts.
    InvalidFill {
        /// Requested ratio.
        fill: f32,
    },
    /// A garage still holds vehicles or drivers.
    GarageOccupied {
        /// Garage unit id.
        id: String,
    },
    /// A driver is not in the pool or the dismissed list.
    DriverNotAvailable {
        /// Driver unit id.
        id: String,
    },
    /// A driver is not in the player's roster.
    DriverNotEmployed {
        /// Driver unit id.
        id: String,
    },
    /// A garage has no free driver slot.
    NoDriverSlot {
        /// Garage unit id.
        id: String,
    },
    /// A driver and a truck are not paired in the same garage.
    NoSharedGarage {
        /// Driver unit id.
        driver: String,
        /// Vehicle unit id.
        truck: String,
    },
    /// The document rejected the edit.
    Document(siir::Error),
    /// A file operation failed.
    Io(std::io::Error),
    /// A profile archive could not be read or written.
    Zip(zip::result::ZipError),
    /// The archive contains a path that escapes the destination.
    InvalidArchive,
    /// An archive entry or the whole archive is too large to extract.
    ArchiveTooLarge,
    /// A profile directory with the requested name already exists.
    ProfileExists(String),
    /// A profile symlink points outside the profile directory.
    ExternalLink {
        /// Relative path of the symlink.
        path: String,
    },
    /// A required `config.cfg` setting is absent.
    MissingConfig {
        /// Setting name, for example `g_developer`.
        key: String,
    },
    /// `cams.txt` has no parseable placement.
    InvalidCamera,
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
            Self::NoEngine { id } => write!(f, "no engine on {id}"),
            Self::NoTransmission { id } => write!(f, "no transmission on {id}"),
            Self::NoCabin { id } => write!(f, "no cabin on {id}"),
            Self::NoChassis { id } => write!(f, "no chassis on {id}"),
            Self::NoPaint { id } => write!(f, "no paint job on {id}"),
            Self::NoAccessory { id, accessory } => {
                write!(f, "no accessory {accessory} on {id}")
            }
            Self::CannotRemovePart { part } => write!(f, "cannot remove the {part}"),
            Self::InvalidPrice { price } => {
                write!(f, "used truck price {price} is negative")
            }
            Self::InvalidAccessoryClass { class } => {
                write!(f, "invalid accessory class {class:?}")
            }
            Self::MissingAccessoryPart { part } => {
                write!(f, "accessory set is missing the {part}")
            }
            Self::NoGarage { id } => write!(f, "no garage holds {id}"),
            Self::NoCurrentVehicle => write!(f, "player has no assigned vehicle slot"),
            Self::NoCurrentTrailer => write!(f, "player has no current trailer"),
            Self::NoCurrentJob => write!(f, "player has no current job"),
            Self::InvalidFuel { level } => {
                write!(f, "fuel level {level} is not between 0 and 1")
            }
            Self::InvalidGarageStatus { status } => {
                write!(f, "garage status {status} is not 1, 2, 3, or 6")
            }
            Self::InvalidCountry { country } => {
                write!(f, "license plate country {country:?} is invalid")
            }
            Self::InvalidFill { fill } => {
                write!(f, "fill ratio {fill} is not between 0 and 1")
            }
            Self::GarageOccupied { id } => {
                write!(f, "garage {id} still holds vehicles, drivers, or trailers")
            }
            Self::DriverNotAvailable { id } => {
                write!(f, "driver {id:?} is not available to hire")
            }
            Self::DriverNotEmployed { id } => {
                write!(f, "driver {id:?} is not in the player's roster")
            }
            Self::NoDriverSlot { id } => {
                write!(f, "garage {id} has no free driver slot")
            }
            Self::NoSharedGarage { driver, truck } => {
                write!(
                    f,
                    "driver {driver:?} and truck {truck:?} are not in the same garage"
                )
            }
            Self::Document(error) => error.fmt(f),
            Self::Io(error) => error.fmt(f),
            Self::Zip(error) => error.fmt(f),
            Self::InvalidArchive => write!(f, "archive path escapes the destination"),
            Self::ArchiveTooLarge => write!(f, "archive entry is too large"),
            Self::ProfileExists(name) => write!(f, "profile {name:?} already exists"),
            Self::ExternalLink { path } => {
                write!(f, "profile link {path:?} points outside the profile")
            }
            Self::MissingConfig { key } => write!(f, "config setting {key:?} is missing"),
            Self::InvalidCamera => write!(f, "cams.txt has no parseable placement"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Document(error) => Some(error),
            Self::Io(error) => Some(error),
            Self::Zip(error) => Some(error),
            _ => None,
        }
    }
}

impl From<siir::Error> for Error {
    fn from(error: siir::Error) -> Self {
        Self::Document(error)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<zip::result::ZipError> for Error {
    fn from(error: zip::result::ZipError) -> Self {
        Self::Zip(error)
    }
}
