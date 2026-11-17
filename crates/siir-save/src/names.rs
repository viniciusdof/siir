//! Save schema names shared by the read model and the commands.

// Classes.
pub(crate) const ECONOMY: &str = "economy";
pub(crate) const BANK: &str = "bank";
pub(crate) const PLAYER: &str = "player";
pub(crate) const PLAYER_VEHICLES: &str = "player_vehicles";
pub(crate) const VEHICLE: &str = "vehicle";
pub(crate) const TRAILER: &str = "trailer";
pub(crate) const GARAGE: &str = "garage";
pub(crate) const DRIVER_AI: &str = "driver_ai";
pub(crate) const COMPANY: &str = "company";
pub(crate) const JOB_INFO: &str = "job_info";
pub(crate) const PLAYER_JOB: &str = "player_job";
pub(crate) const JOB_OFFER_DATA: &str = "job_offer_data";
pub(crate) const USED_TRUCK_OFFER: &str = "used_truck_offer";
pub(crate) const USED_VEHICLE_ASSORTMENT: &str = "used_vehicle_assortment";
pub(crate) const POLICE_OFFENCE_LOG: &str = "police_offence_log";

// Attributes shared by the read model and the commands.
pub(crate) const HQ_CITY: &str = "hq_city";
pub(crate) const MY_VEHICLES: &str = "my_vehicles";
pub(crate) const TRUCKS: &str = "trucks";
pub(crate) const TRAILERS: &str = "trailers";
pub(crate) const ASSIGNED_VEHICLES: &str = "assigned_vehicles";
pub(crate) const ACCESSORIES: &str = "accessories";
pub(crate) const DATA_PATH: &str = "data_path";
pub(crate) const VEHICLES: &str = "vehicles";
pub(crate) const DRIVERS: &str = "drivers";
pub(crate) const TRAILER_DEFINITION: &str = "trailer_definition";
pub(crate) const SLAVE_TRAILER: &str = "slave_trailer";
pub(crate) const CARGO_MASS: &str = "cargo_mass";
pub(crate) const LICENSE_PLATE: &str = "license_plate";
pub(crate) const TRAILER_DEF: &str = "trailer_def";
pub(crate) const COUNTRY_VALIDITY: &str = "country_validity";
pub(crate) const SOURCE_NAME: &str = "source_name";
pub(crate) const TRAILER_BODY_WEAR: &str = "trailer_body_wear";
pub(crate) const TRAILER_BODY_WEAR_UNFIXABLE: &str = "trailer_body_wear_unfixable";
pub(crate) const CHASSIS_WEAR: &str = "chassis_wear";
pub(crate) const CHASSIS_WEAR_UNFIXABLE: &str = "chassis_wear_unfixable";

// Market, job, driver, and offence attributes.
pub(crate) const EXPIRATION_TIME: &str = "expiration_time";
pub(crate) const EXPIRATION_GAME_TIME: &str = "expiration_game_time";
pub(crate) const NEXT_GENERATION_GAME_TIME: &str = "next_generation_game_time";
pub(crate) const URGENCY: &str = "urgency";
pub(crate) const UNITS_COUNT: &str = "units_count";
pub(crate) const FILL_RATIO: &str = "fill_ratio";
pub(crate) const HOMETOWN: &str = "hometown";
pub(crate) const DRIVER_JOB: &str = "driver_job";
pub(crate) const ASSIGNED_TRUCK: &str = "assigned_truck";
pub(crate) const TRAINING_POLICY: &str = "training_policy";
pub(crate) const OFFENCE_TOTAL_COUNTS: &str = "offence_total_counts";
pub(crate) const OFFENCE_TOTAL_FINES: &str = "offence_total_fines";
pub(crate) const DETAILED_HISTORY_ENTRIES: &str = "detailed_history_entries";

// Economy and profile attributes.
pub(crate) const COMPANIES: &str = "companies";
pub(crate) const EXPERIENCE_POINTS: &str = "experience_points";
pub(crate) const ADR: &str = "adr";
pub(crate) const LONG_DIST: &str = "long_dist";
pub(crate) const HEAVY: &str = "heavy";
pub(crate) const FRAGILE: &str = "fragile";
pub(crate) const URGENT: &str = "urgent";
pub(crate) const MECHANICAL: &str = "mechanical";
pub(crate) const VISITED_CITIES: &str = "visited_cities";
pub(crate) const VISITED_CITIES_COUNT: &str = "visited_cities_count";
pub(crate) const UNLOCKED_DEALERS: &str = "unlocked_dealers";
pub(crate) const STATUS: &str = "status";
pub(crate) const STORED_VEHICLE_PLACEMENT: &str = "stored_vehicle_placement";
pub(crate) const STORED_TRAILER_PLACEMENTS: &str = "stored_trailer_placements";
pub(crate) const STORED_TRAILER_ATTACHED: &str = "stored_trailer_attached";
pub(crate) const USER_PROFILE: &str = "user_profile";
pub(crate) const PROFILE_NAME: &str = "profile_name";

// Paths.
/// Truck definitions live under `/def/vehicle/truck/<brand>.<model>/`.
pub(crate) const TRUCK_DIR: &str = "/def/vehicle/truck/";
pub(crate) const DATA_SUFFIX: &str = "/data.sii";
pub(crate) const ENGINE_DIR: &str = "/engine/";
pub(crate) const TRANSMISSION_DIR: &str = "/transmission/";
pub(crate) const CABIN_DIR: &str = "/cabin/";
pub(crate) const CHASSIS_DIR: &str = "/chassis/";
pub(crate) const PAINT_JOB_DIR: &str = "/paint_job/";
pub(crate) const ACCESSORY_DIR: &str = "/accessory/";

/// A garage id is `garage.<city>`.
pub(crate) const GARAGE_PREFIX: &str = "garage.";

/// Front and side badge slots use `badge` or `*_badge` path segments.
pub(crate) const BADGE: &str = "badge";
pub(crate) const BADGE_SUFFIX: &str = "_badge";

// Profile files and directories.
pub(crate) const PROFILE_FILE: &str = "profile.sii";
pub(crate) const INFO_FILE: &str = "info.sii";
pub(crate) const GAME_FILE: &str = "game.sii";
pub(crate) const SAVES_DIR: &str = "saves";
pub(crate) const ALBUM_DIR: &str = "album";
pub(crate) const SETTINGS_FILES: [&str; 3] = ["config.cfg", "config_local.cfg", "controls.sii"];

/// `name[nth]`, the spelling of an indexed save attribute.
pub(crate) fn indexed(name: &str, nth: usize) -> String {
    format!("{name}[{nth}]")
}
