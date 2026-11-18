//! Owned rows the views render, built from a borrowed [`Save`].

use siir_save::{Save, Skills};

/// Headline values and counts of one open save.
pub struct SaveView {
    /// Save name from `info.sii`.
    pub save_name: String,
    /// Name of the profile that holds the save.
    pub profile_name: String,
    /// Bank balance, when the save has a bank.
    pub money: Option<i64>,
    /// Total experience points.
    pub experience: u64,
    /// Player skill levels.
    pub skills: Skills,
    /// Headquarters city.
    pub hq_city: String,
    /// Model folder of the player's current vehicle, when it has a data path.
    pub current_vehicle: Option<String>,
    /// Owned fleet and market sizes.
    pub counts: Counts,
}

/// Owned fleet and market sizes shown on the overview.
#[derive(Clone, Copy, Default)]
pub struct Counts {
    /// Owned `player_vehicles` slots.
    pub vehicles: usize,
    /// Owned trailer unit ids.
    pub trailers: usize,
    /// Hired AI drivers.
    pub drivers: usize,
    /// Garages with a non-zero status.
    pub garages: usize,
    /// Freight market offers.
    pub offers: usize,
}

impl SaveView {
    /// Copies the values the overview shows out of the borrowed save.
    pub fn new(save: &Save<'_>, save_name: String, profile_name: String) -> Self {
        let player = save.player.as_ref();
        Self {
            save_name,
            profile_name,
            money: save.bank.as_ref().map(|bank| bank.money_account),
            experience: save.experience,
            skills: save.skills,
            hq_city: save
                .player
                .as_ref()
                .map(|player| player.hq_city.to_owned())
                .unwrap_or_default(),
            current_vehicle: save
                .current_vehicle()
                .and_then(|vehicle| vehicle.model)
                .map(str::to_owned),
            counts: Counts {
                vehicles: player
                    .map(|player| player.my_vehicles.len())
                    .unwrap_or_default(),
                trailers: player
                    .map(|player| player.trailers.len())
                    .unwrap_or_default(),
                drivers: save.drivers.len(),
                garages: save
                    .garages
                    .iter()
                    .filter(|garage| garage.status != 0)
                    .count(),
                offers: save.offers.len(),
            },
        }
    }
}
