//! Typed, read-only view of a save.

/// The game a save belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Game {
    /// Euro Truck Simulator 2.
    Ets2,
    /// American Truck Simulator.
    Ats,
}

/// Typed view of a save document. Rebuild it after every edit.
pub struct Save<'a> {
    /// Game the save was read as.
    pub game: Game,
    /// The profile's bank, when the save has one.
    pub bank: Option<Bank>,
    /// The profile's player, when the save has one.
    pub player: Option<Player<'a>>,
    /// Every `vehicle` unit, including AI and market trucks.
    pub trucks: Vec<Truck<'a>>,
    /// Every `trailer` unit.
    pub trailers: Vec<Trailer<'a>>,
    /// Every garage.
    pub garages: Vec<Garage<'a>>,
    /// Every AI driver.
    pub drivers: Vec<Driver<'a>>,
    /// Every company.
    pub companies: Vec<Company<'a>>,
    /// Every AI driver's current job.
    pub jobs: Vec<Job<'a>>,
    /// The player's current job, when there is one.
    pub player_job: Option<Job<'a>>,
    /// Every freight market offer.
    pub offers: Vec<Offer<'a>>,
    /// Every used-truck market listing.
    pub used_trucks: Vec<UsedTruck<'a>>,
    /// The profile's police offence log, when the save has one.
    pub offence: Option<Offence<'a>>,
}

impl<'a> Save<'a> {
    /// A truck by its vehicle unit id.
    pub fn truck(&self, id: &str) -> Option<&Truck<'a>> {
        self.trucks.iter().find(|truck| truck.id == id)
    }

    /// The truck in the player's assigned vehicle slot.
    pub fn current_truck(&self) -> Option<&Truck<'a>> {
        let slot = self.player.as_ref()?.assigned_vehicles?;
        self.trucks.iter().find(|truck| truck.slot == Some(slot))
    }

    /// A trailer by its unit id.
    pub fn trailer(&self, id: &str) -> Option<&Trailer<'a>> {
        self.trailers.iter().find(|trailer| trailer.id == id)
    }

    /// The trailer in the player's assigned vehicle slot.
    pub fn current_trailer(&self) -> Option<&Trailer<'a>> {
        self.trailer(self.current_truck()?.trailer?)
    }
}

/// One `vehicle` unit.
pub struct Truck<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Owning `player_vehicles` slot, when one references this truck.
    pub slot: Option<&'a str>,
    /// Trailer assigned to the owning slot, when there is one.
    pub trailer: Option<&'a str>,
    /// Model folder under `/def/vehicle/truck/`, for example `renault.etech_t`.
    pub model: Option<&'a str>,
    /// The model's `data.sii` path.
    pub data_path: Option<&'a str>,
    /// The engine's `data.sii` path.
    pub engine: Option<&'a str>,
    /// Engine wear, from 0 to 1.
    pub engine_wear: f32,
    /// Transmission wear, from 0 to 1.
    pub transmission_wear: f32,
    /// Cabin wear, from 0 to 1.
    pub cabin_wear: f32,
    /// Fuel level, from 0 to 1.
    pub fuel_relative: f32,
    /// Placement text from the owning slot, kept opaque.
    pub placement: Option<&'a str>,
}

/// One `trailer` unit.
pub struct Trailer<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Trailer definition the unit uses.
    pub definition: &'a str,
    /// Cargo mass in kilograms.
    pub cargo_mass: f32,
    /// Raw license plate text.
    pub license_plate: &'a str,
    /// Slave trailer unit id, when this trailer tows one.
    pub slave_trailer: Option<&'a str>,
    /// Body wear, from 0 to 1.
    pub body_wear: f32,
    /// Chassis wear, from 0 to 1.
    pub chassis_wear: f32,
}

/// One garage.
pub struct Garage<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Garage status the game stores.
    pub status: u64,
}

/// One AI driver.
pub struct Driver<'a> {
    /// Unit id.
    pub id: &'a str,
    /// The driver's experience points.
    pub experience_points: u64,
}

/// One company.
pub struct Company<'a> {
    /// Unit id.
    pub id: &'a str,
    /// The company prototype the unit is an instance of.
    pub permanent_data: &'a str,
}

/// A driver job or the player's current job.
pub struct Job<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Cargo definition, absent for an idle driver.
    pub cargo: Option<&'a str>,
    /// Source company, absent for an idle driver.
    pub source_company: Option<&'a str>,
    /// Target company, absent for an idle driver.
    pub target_company: Option<&'a str>,
    /// Planned road distance in kilometers.
    pub planned_distance_km: u64,
}

/// One freight market offer.
pub struct Offer<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Target city.
    pub target: &'a str,
    /// Cargo definition.
    pub cargo: &'a str,
    /// Shortest road distance in kilometers.
    pub shortest_distance_km: u64,
}

/// One used-truck market listing.
pub struct UsedTruck<'a> {
    /// Unit id.
    pub id: &'a str,
    /// The listed `vehicle` unit.
    pub truck: &'a str,
    /// Asking price.
    pub price: i64,
}

/// The profile's police offence log.
pub struct Offence<'a> {
    /// Unit id.
    pub id: &'a str,
}

/// The profile's bank.
pub struct Bank {
    /// Current account balance.
    pub money_account: i64,
}

/// The profile's player.
pub struct Player<'a> {
    /// Headquarters city.
    pub hq_city: &'a str,
    /// Owned `player_vehicles` slot ids.
    pub my_vehicles: Vec<&'a str>,
    /// Owned truck vehicle unit ids.
    pub trucks: Vec<&'a str>,
    /// Owned trailer unit ids.
    pub trailers: Vec<&'a str>,
    /// Assigned `player_vehicles` slot id, when there is one.
    pub assigned_vehicles: Option<&'a str>,
    /// The player's current `player_job` unit id, when there is one.
    pub current_job: Option<&'a str>,
}
