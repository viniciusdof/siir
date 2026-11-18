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
    /// Every bus stop.
    pub bus_stops: Vec<BusStop<'a>>,
    /// Every bus line offer.
    pub bus_offers: Vec<BusLineOffer<'a>>,
    /// The player's current bus job, when there is one.
    pub bus_job: Option<BusJob<'a>>,
    /// The player's current car job, when there is one.
    pub car_job: Option<CarJob<'a>>,
    /// Every car market offer.
    pub car_offers: Vec<CarOffer<'a>>,
    /// The bus job log, when the save has one.
    pub bus_job_log: Option<JobLog>,
    /// The car job log, when the save has one.
    pub car_job_log: Option<JobLog>,
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
    /// Model folder under `/def/vehicle/{truck,bus,car}/`, for example
    /// `renault.etech_t` or `volvo.9700_23`.
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

/// One bus stop.
pub struct BusStop<'a> {
    /// Unit id, `bus_stop.city.<city>`.
    pub id: &'a str,
    /// City the stop sits in.
    pub city: &'a str,
    /// Whether the player has discovered the stop.
    pub discovered: bool,
    /// Line offer unit ids at this stop.
    pub offers: Vec<&'a str>,
}

/// One bus line offer.
pub struct BusLineOffer<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Line type the game stores, for example `regular`.
    pub kind: &'a str,
    /// Source bus stop unit id.
    pub source: &'a str,
    /// Target city names, unquoted.
    pub targets: Vec<&'a str>,
    /// Distance to each target in kilometers.
    pub distances: Vec<u64>,
    /// Planned time to each target in game minutes.
    pub times: Vec<u64>,
    /// Departure time of day in game minutes.
    pub departure_time_of_day: u64,
    /// Bus model token the offer requires.
    pub bus: &'a str,
    /// Game time the offer expires.
    pub expiration_time: u64,
}

/// The player's current bus job.
pub struct BusJob<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Job type the game stores, for example `regular`.
    pub kind: &'a str,
    /// Bus company, absent for a public line.
    pub company_bus: Option<&'a str>,
    /// Stages in order.
    pub stages: Vec<BusJobStage<'a>>,
    /// Index of the current stage.
    pub stage_index: u64,
    /// Number of passengers aboard.
    pub passengers: u64,
}

/// One stage of a bus job.
pub struct BusJobStage<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Source bus stop unit id.
    pub source: &'a str,
    /// Target bus stop unit id.
    pub target: &'a str,
    /// Planned road distance in kilometers.
    pub planned_distance: u64,
    /// Planned departure time, absent before the stage starts.
    pub planned_departure_time: Option<u64>,
    /// Planned arrival time, absent before it is known.
    pub planned_arrival_time: Option<u64>,
    /// Planned stage time in game minutes.
    pub planned_stage_time: u64,
    /// Planned ferry time in game minutes.
    pub planned_ferry_time: u64,
    /// Passengers expected to board.
    pub planned_entering_passengers: u64,
    /// Passengers expected to leave.
    pub planned_leaving_passengers: u64,
    /// Actual departure time, absent before the stage starts.
    pub departure_time: Option<u64>,
    /// Actual arrival time, absent before the stage ends.
    pub arrival_time: Option<u64>,
    /// Distance travelled so far in kilometers.
    pub travelled_distance: u64,
    /// Satisfaction the game stores, `-1` while pending.
    pub satisfaction: f32,
}

/// One car market offer.
pub struct CarOffer<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Source company token.
    pub source: &'a str,
    /// Target company token.
    pub target: &'a str,
    /// Game time the offer expires, absent for an empty slot.
    pub expiration_time: Option<u64>,
    /// Urgency class, absent for an empty slot.
    pub urgency: Option<u64>,
    /// Job type the game stores, for example `delivery`.
    pub job_type: &'a str,
    /// Shortest road distance in kilometers.
    pub shortest_distance_km: u64,
    /// Car job definition, absent for an empty slot.
    pub job: Option<&'a str>,
    /// Cargo definition, absent for an empty slot.
    pub cargo: Option<&'a str>,
    /// Car model token the offer requires.
    pub company_car: &'a str,
    /// Number of cars offered.
    pub units_count: u64,
}

/// The player's current car job.
pub struct CarJob<'a> {
    /// Unit id.
    pub id: &'a str,
    /// Job type the game stores.
    pub job_type: &'a str,
    /// Source company unit id.
    pub source_company: &'a str,
    /// Target company unit id.
    pub target_company: &'a str,
    /// Planned road distance in kilometers.
    pub planned_distance_km: u64,
    /// Urgency class the game stores.
    pub urgency: u64,
    /// Earliest delivery time.
    pub time_lower_limit: u64,
    /// Latest delivery time.
    pub time_upper_limit: u64,
    /// Cargo definition.
    pub cargo: Option<&'a str>,
    /// Number of cars hauled.
    pub units_count: u64,
    /// Distance travelled so far in kilometers.
    pub travelled_distance: u64,
}

/// A bus or car job log.
pub struct JobLog {
    /// Log structure version.
    pub version: u64,
    /// Number of entries kept.
    pub entries: u64,
    /// Cached jobs the game keeps for the car log.
    pub cached_jobs: Option<u64>,
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
    /// Owned bus unit ids.
    pub buses: Vec<&'a str>,
    /// Owned car unit ids.
    pub cars: Vec<&'a str>,
    /// Assigned `player_vehicles` slot id, when there is one.
    pub assigned_vehicles: Option<&'a str>,
    /// The player's current `player_job` unit id, when there is one.
    pub current_job: Option<&'a str>,
    /// The player's current `player_bus_job` unit id, when there is one.
    pub current_bus_job: Option<&'a str>,
    /// The selected bus line offer unit id, when there is one.
    pub selected_bus_offer: Option<&'a str>,
    /// The player's current `player_car_job` unit id, when there is one.
    pub current_car_job: Option<&'a str>,
}
