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
pub(crate) const POLICE_OFFENCE_LOG: &str = "police_offence_log";

// Attributes shared by the read model and the commands.
pub(crate) const HQ_CITY: &str = "hq_city";
pub(crate) const MY_VEHICLES: &str = "my_vehicles";
pub(crate) const TRUCKS: &str = "trucks";
pub(crate) const ASSIGNED_VEHICLES: &str = "assigned_vehicles";
pub(crate) const ACCESSORIES: &str = "accessories";
pub(crate) const DATA_PATH: &str = "data_path";
pub(crate) const VEHICLES: &str = "vehicles";

// Paths.
/// Truck definitions live under `/def/vehicle/truck/<brand>.<model>/`.
pub(crate) const TRUCK_DIR: &str = "/def/vehicle/truck/";
pub(crate) const DATA_SUFFIX: &str = "/data.sii";
pub(crate) const ENGINE_DIR: &str = "/engine/";

/// A garage id is `garage.<city>`.
pub(crate) const GARAGE_PREFIX: &str = "garage.";

/// Front and side badge slots use `badge` or `*_badge` path segments.
pub(crate) const BADGE: &str = "badge";
pub(crate) const BADGE_SUFFIX: &str = "_badge";

/// `name[nth]`, the spelling of an indexed save attribute.
pub(crate) fn indexed(name: &str, nth: usize) -> String {
    format!("{name}[{nth}]")
}
