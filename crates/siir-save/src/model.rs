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
    /// Total experience points.
    pub experience: u64,
    /// Player skill levels.
    pub skills: Skills,
    /// Cities the player has visited.
    pub visited_cities: Vec<&'a str>,
    /// Dealerships discovered, as city names.
    pub unlocked_dealers: Vec<&'a str>,
    /// The profile's player, when the save has one.
    pub player: Option<Player<'a>>,
    /// Every `vehicle` unit, including buses, cars, and AI vehicles.
    pub vehicles: Vec<Vehicle<'a>>,
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
    /// The used-truck assortment behind the listings, when the save has one.
    pub used_assortment: Option<UsedAssortment<'a>>,
    /// The profile's police offence log, when the save has one.
    pub offence: Option<Offence<'a>>,
    /// Every oversize market offer.
    pub oversize_offers: Vec<OversizeOffer<'a>>,
    /// Every oversize route and the offers it holds.
    pub oversize_routes: Vec<OversizeRoute<'a>>,
}

impl<'a> Save<'a> {
    /// A vehicle by its unit id.
    pub fn vehicle(&self, id: &str) -> Option<&Vehicle<'a>> {
        self.vehicles.iter().find(|vehicle| vehicle.id == id)
    }

    /// The vehicle in the player's assigned slot.
    pub fn current_vehicle(&self) -> Option<&Vehicle<'a>> {
        let slot = self.player.as_ref()?.assigned_vehicles?;
        self.vehicles
            .iter()
            .find(|vehicle| vehicle.slot == Some(slot))
    }

    /// A trailer by its unit id.
    pub fn trailer(&self, id: &str) -> Option<&Trailer<'a>> {
        self.trailers.iter().find(|trailer| trailer.id == id)
    }

    /// The trailer in the player's assigned vehicle slot.
    pub fn current_trailer(&self) -> Option<&Trailer<'a>> {
        self.trailer(self.current_vehicle()?.trailer?)
    }
}

/// One accessory referenced by a vehicle.
pub struct Accessory<'a> {
    /// Accessory unit id.
    pub id: &'a str,
    /// Unit class, for example `vehicle_accessory`.
    pub class: &'a str,
    /// `data_path`, unquoted.
    pub path: Option<&'a str>,
    /// Slot under `/accessory/`, when the path names one.
    pub slot: Option<&'a str>,
}

/// One `vehicle` unit.
pub struct Vehicle<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Owning `player_vehicles` slot, when one references this vehicle.
    pub slot: Option<&'a str>,
    /// Trailer assigned to the owning slot, when there is one.
    pub trailer: Option<&'a str>,
    /// Accessories referenced by this vehicle.
    pub accessories: Vec<Accessory<'a>>,
    /// Model folder under `/def/vehicle/truck/`, for example `renault.etech_t`.
    pub model: Option<&'a str>,
    /// The model's `data.sii` path.
    pub data_path: Option<&'a str>,
    /// The engine's `data.sii` path.
    pub engine: Option<&'a str>,
    /// The transmission's `data.sii` path.
    pub transmission: Option<&'a str>,
    /// The cabin's `data.sii` path.
    pub cabin: Option<&'a str>,
    /// The chassis's `data.sii` path.
    pub chassis: Option<&'a str>,
    /// The paint job's `data.sii` path.
    pub paint_job: Option<&'a str>,
    /// Engine wear, from 0 to 1.
    pub engine_wear: f32,
    /// Transmission wear, from 0 to 1.
    pub transmission_wear: f32,
    /// Cabin wear, from 0 to 1.
    pub cabin_wear: f32,
    /// Fuel level, from 0 to 1.
    pub fuel_relative: f32,
    /// Raw license plate text, markup included.
    pub license_plate: &'a str,
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
    /// The driver's skill levels.
    pub skills: Skills,
    /// Hometown city, empty for a driver hired in place.
    pub hometown: &'a str,
    /// Truck unit the driver is assigned to.
    pub assigned_truck: Option<&'a str>,
    /// Current `job_info` unit.
    pub job: Option<&'a str>,
    /// Training policy the game stores.
    pub training_policy: u64,
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
    /// Urgency class the game stores, absent when the job has none.
    pub urgency: Option<u64>,
    /// Units of cargo.
    pub units_count: u64,
    /// Trailer fill ratio, from 0 to 1.
    pub fill_ratio: f32,
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
    /// Urgency class the game stores, absent when the offer has none.
    pub urgency: Option<u64>,
    /// Game time the offer expires, absent when it does not expire.
    pub expiration_time: Option<u64>,
    /// Units of cargo offered.
    pub units_count: u64,
    /// Trailer fill ratio, from 0 to 1.
    pub fill_ratio: f32,
    /// Trailer definition the offer requires.
    pub trailer_definition: &'a str,
}

/// One used-truck market listing.
pub struct UsedTruck<'a> {
    /// Unit id.
    pub id: &'a str,
    /// The listed `vehicle` unit.
    pub truck: &'a str,
    /// Asking price.
    pub price: i64,
    /// Game time the listing expires.
    pub expiration_game_time: u64,
}

/// The used-truck assortment that rotates the listings.
pub struct UsedAssortment<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Game time the next listing generation is due.
    pub next_generation_game_time: u64,
    /// Listing unit ids in the assortment.
    pub trucks: Vec<&'a str>,
}

/// The profile's police offence log.
pub struct Offence<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Offence counts per category.
    pub counts: Vec<u64>,
    /// Fines per category.
    pub fines: Vec<i64>,
    /// Number of detailed history entries.
    pub history_entries: u64,
}

/// One oversize market offer.
pub struct OversizeOffer<'a> {
    /// Unit id.
    pub id: &'a str,
    /// The `oversize_offer_data` definition the offer uses.
    pub offer_data: &'a str,
    /// Truck definition the offer requires.
    pub truck: &'a str,
    /// Game time the offer expires.
    pub expiration: u64,
}

/// One oversize route and the offers along it.
pub struct OversizeRoute<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Route definition the offers follow.
    pub route: &'a str,
    /// Offer unit ids on this route.
    pub offers: Vec<&'a str>,
}

/// The profile's bank.
pub struct Bank {
    /// Current account balance.
    pub money_account: i64,
}

/// Player skill levels stored in the economy.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Skills {
    /// ADR class bitmask.
    pub adr: u64,
    /// Long-distance skill level.
    pub long_dist: u64,
    /// Heavy-cargo skill level.
    pub heavy: u64,
    /// Fragile-cargo skill level.
    pub fragile: u64,
    /// Urgent-delivery skill level.
    pub urgent: u64,
    /// Mechanical skill level.
    pub mechanical: u64,
}

/// The profile's player.
pub struct Player<'a> {
    /// Headquarters city.
    pub hq_city: &'a str,
    /// Owned `player_vehicles` slot ids.
    pub my_vehicles: Vec<&'a str>,
    /// Owned `vehicle` unit ids, from `player.trucks`.
    pub trucks: Vec<&'a str>,
    /// Owned trailer unit ids.
    pub trailers: Vec<&'a str>,
    /// Assigned `player_vehicles` slot id, when there is one.
    pub assigned_vehicles: Option<&'a str>,
    /// The player's current `player_job` unit id, when there is one.
    pub current_job: Option<&'a str>,
}
