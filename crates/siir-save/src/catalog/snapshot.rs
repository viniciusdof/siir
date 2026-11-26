use std::collections::BTreeMap;

use crate::Game;
use crate::error::Error;

use super::{
    Cargo, Catalog, Company, Currency, Engine, LevelTitle, Levels, Named, Part, TrailerDefinition,
    Transmission, VehicleModel,
};

pub(super) fn parse(source: &str, game: Game) -> Result<Catalog, Error> {
    let name = match game {
        Game::Ets2 => "ets2",
        Game::Ats => "ats",
    };
    let snapshot: Snapshot =
        serde_json::from_str(source).map_err(|error| Error::Catalog(error.to_string()))?;
    if snapshot.game != name {
        return Err(Error::Catalog(format!("snapshot is for {}", snapshot.game)));
    }
    Ok(snapshot.into())
}

#[derive(serde::Deserialize)]
struct Snapshot {
    game: String,
    levels: LevelsData,
    currencies: Vec<CurrencyData>,
    trailer_body_types: Vec<NamedData>,
    trailer_brands: Vec<NamedData>,
    cities: Vec<NamedData>,
    country_names: Vec<NamedData>,
    cargo: Vec<CargoData>,
    companies: Vec<CompanyData>,
    trailers: Vec<TrailerData>,
    vehicles: Vec<VehicleData>,
}

#[derive(serde::Deserialize)]
struct NamedData {
    token: String,
    names: BTreeMap<String, String>,
}

#[derive(serde::Deserialize)]
struct CurrencyData {
    code: String,
    ratio: f64,
    sign1: String,
    sign2: String,
    sign3: String,
}

#[derive(serde::Deserialize)]
struct LevelsData {
    steps: Vec<u32>,
    titles: Vec<LevelTitleData>,
}

#[derive(serde::Deserialize)]
struct LevelTitleData {
    from: u32,
    names: BTreeMap<String, String>,
}

#[derive(serde::Deserialize)]
struct VehicleData {
    token: String,
    names: BTreeMap<String, String>,
    #[serde(default)]
    engines: Vec<EngineData>,
    #[serde(default)]
    transmissions: Vec<TransmissionData>,
    #[serde(default)]
    cabins: Vec<PartData>,
    #[serde(default)]
    chassis: Vec<PartData>,
    #[serde(default)]
    paint_jobs: Vec<PartData>,
}

#[derive(serde::Deserialize)]
struct EngineData {
    names: BTreeMap<String, String>,
    power_hp: u32,
    torque_nm: u32,
    path: String,
}

#[derive(serde::Deserialize)]
struct TransmissionData {
    names: BTreeMap<String, String>,
    ratios: Vec<f32>,
    retarder: bool,
    path: String,
}

#[derive(serde::Deserialize)]
struct PartData {
    names: BTreeMap<String, String>,
    path: String,
}

#[derive(serde::Deserialize)]
struct CargoData {
    id: String,
    name: String,
    mass: f32,
    volume: f32,
    reward_per_km: f32,
    body_types: Vec<String>,
}

#[derive(serde::Deserialize)]
struct CompanyData {
    id: String,
    name: String,
}

#[derive(serde::Deserialize)]
struct TrailerData {
    id: String,
    body_type: String,
    chain_type: String,
    volume: f32,
    gross_weight_limit: u64,
    chassis_mass: u64,
    axles: u64,
    length: f32,
}

impl From<Snapshot> for Catalog {
    fn from(snapshot: Snapshot) -> Self {
        Self {
            vehicles: snapshot
                .vehicles
                .into_iter()
                .map(VehicleModel::from)
                .collect(),
            cargo: snapshot.cargo.into_iter().map(Cargo::from).collect(),
            companies: snapshot.companies.into_iter().map(Company::from).collect(),
            trailers: snapshot
                .trailers
                .into_iter()
                .map(TrailerDefinition::from)
                .collect(),
            body_types: snapshot
                .trailer_body_types
                .into_iter()
                .map(Named::from)
                .collect(),
            brands: snapshot
                .trailer_brands
                .into_iter()
                .map(Named::from)
                .collect(),
            cities: snapshot.cities.into_iter().map(Named::from).collect(),
            levels: snapshot.levels.into(),
            currencies: snapshot
                .currencies
                .into_iter()
                .map(|currency| Currency {
                    code: currency.code,
                    ratio: currency.ratio,
                    sign1: currency.sign1,
                    sign2: currency.sign2,
                    sign3: currency.sign3,
                })
                .collect(),
            countries: snapshot
                .country_names
                .into_iter()
                .map(Named::from)
                .collect(),
        }
    }
}

impl From<LevelsData> for Levels {
    fn from(levels: LevelsData) -> Self {
        Self {
            steps: levels.steps,
            titles: levels
                .titles
                .into_iter()
                .map(|title| LevelTitle {
                    from: title.from,
                    names: title.names,
                })
                .collect(),
        }
    }
}

impl From<NamedData> for Named {
    fn from(named: NamedData) -> Self {
        Self {
            token: named.token,
            names: named.names,
        }
    }
}

impl From<VehicleData> for VehicleModel {
    fn from(truck: VehicleData) -> Self {
        Self {
            token: truck.token,
            names: truck.names,
            engines: truck.engines.into_iter().map(Engine::from).collect(),
            transmissions: truck
                .transmissions
                .into_iter()
                .map(Transmission::from)
                .collect(),
            cabins: truck.cabins.into_iter().map(Part::from).collect(),
            chassis: truck.chassis.into_iter().map(Part::from).collect(),
            paint_jobs: truck.paint_jobs.into_iter().map(Part::from).collect(),
        }
    }
}

impl From<EngineData> for Engine {
    fn from(engine: EngineData) -> Self {
        Self {
            names: engine.names,
            power_hp: engine.power_hp,
            torque_nm: engine.torque_nm,
            path: engine.path,
        }
    }
}

impl From<TransmissionData> for Transmission {
    fn from(transmission: TransmissionData) -> Self {
        Self {
            names: transmission.names,
            ratios: transmission.ratios,
            retarder: transmission.retarder,
            path: transmission.path,
        }
    }
}

impl From<PartData> for Part {
    fn from(part: PartData) -> Self {
        Self {
            names: part.names,
            path: part.path,
        }
    }
}

impl From<CargoData> for Cargo {
    fn from(cargo: CargoData) -> Self {
        Self {
            id: cargo.id,
            name: cargo.name,
            mass: cargo.mass,
            volume: cargo.volume,
            reward_per_km: cargo.reward_per_km,
            body_types: cargo.body_types,
        }
    }
}

impl From<CompanyData> for Company {
    fn from(company: CompanyData) -> Self {
        Self {
            id: company.id,
            name: company.name,
        }
    }
}

impl From<TrailerData> for TrailerDefinition {
    fn from(trailer: TrailerData) -> Self {
        Self {
            id: trailer.id,
            body_type: trailer.body_type,
            chain_type: trailer.chain_type,
            volume: trailer.volume,
            gross_weight_limit: trailer.gross_weight_limit,
            chassis_mass: trailer.chassis_mass,
            axles: trailer.axles,
            length: trailer.length,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(game: &str) -> String {
        format!(
            r#"{{"game":"{game}","levels":{{"steps":[],"titles":[]}},"currencies":[],"trailer_body_types":[],"trailer_brands":[],"cities":[],"country_names":[],"cargo":[],"companies":[],"trailers":[],"vehicles":[]}}"#
        )
    }

    #[test]
    fn rejects_another_game() {
        let error = parse(&snapshot("ats"), Game::Ets2).unwrap_err();
        assert!(error.to_string().contains("snapshot is for ats"));
    }
}
