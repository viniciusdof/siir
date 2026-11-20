//! Writes the catalog snapshot as JSON.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use siir_save::Game;
use siir_save::catalog::{
    Cargo, Catalog, Company, Engine, Part, TrailerDefinition, Transmission, VehicleModel,
};

const USAGE: &str = "usage: catalog <def-dir> <locale-dir> <out.json> <ets2|ats>";

#[derive(Serialize)]
struct Snapshot<'a> {
    game: &'a str,
    levels: LevelsData<'a>,
    currencies: Vec<CurrencyData<'a>>,
    trailer_body_types: Vec<NamedData<'a>>,
    trailer_brands: Vec<NamedData<'a>>,
    cities: Vec<NamedData<'a>>,
    country_names: Vec<NamedData<'a>>,
    cargo: Vec<CargoData<'a>>,
    companies: Vec<CompanyData<'a>>,
    trailers: Vec<TrailerData<'a>>,
    vehicles: Vec<VehicleData<'a>>,
}

#[derive(Serialize)]
struct NamedData<'a> {
    token: &'a str,
    names: &'a BTreeMap<String, String>,
}

#[derive(Serialize)]
struct CurrencyData<'a> {
    code: &'a str,
    ratio: f64,
    sign1: &'a str,
    sign2: &'a str,
    sign3: &'a str,
}

#[derive(Serialize)]
struct LevelsData<'a> {
    steps: &'a [u32],
    titles: Vec<LevelTitleData<'a>>,
}

#[derive(Serialize)]
struct LevelTitleData<'a> {
    from: u32,
    names: &'a BTreeMap<String, String>,
}

#[derive(Serialize)]
struct VehicleData<'a> {
    token: &'a str,
    names: &'a BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    engines: Vec<EngineData<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    transmissions: Vec<TransmissionData<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    cabins: Vec<PartData<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    chassis: Vec<PartData<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    paint_jobs: Vec<PartData<'a>>,
}

#[derive(Serialize)]
struct EngineData<'a> {
    names: &'a BTreeMap<String, String>,
    power_hp: u32,
    torque_nm: u32,
    path: &'a str,
}

#[derive(Serialize)]
struct TransmissionData<'a> {
    names: &'a BTreeMap<String, String>,
    ratios: &'a [f32],
    retarder: bool,
    path: &'a str,
}

#[derive(Serialize)]
struct PartData<'a> {
    names: &'a BTreeMap<String, String>,
    path: &'a str,
}

#[derive(Serialize)]
struct CargoData<'a> {
    id: &'a str,
    name: &'a str,
    mass: f32,
    volume: f32,
    reward_per_km: f32,
    body_types: &'a [String],
}

#[derive(Serialize)]
struct CompanyData<'a> {
    id: &'a str,
    name: &'a str,
}

#[derive(Serialize)]
struct TrailerData<'a> {
    id: &'a str,
    body_type: &'a str,
    chain_type: &'a str,
    volume: f32,
    gross_weight_limit: u64,
    chassis_mass: u64,
    axles: u64,
    length: f32,
}

impl<'a> From<&'a VehicleModel> for VehicleData<'a> {
    fn from(truck: &'a VehicleModel) -> Self {
        Self {
            token: &truck.token,
            names: &truck.names,
            engines: truck.engines.iter().map(EngineData::from).collect(),
            transmissions: truck
                .transmissions
                .iter()
                .map(TransmissionData::from)
                .collect(),
            cabins: truck.cabins.iter().map(PartData::from).collect(),
            chassis: truck.chassis.iter().map(PartData::from).collect(),
            paint_jobs: truck.paint_jobs.iter().map(PartData::from).collect(),
        }
    }
}

impl<'a> From<&'a Engine> for EngineData<'a> {
    fn from(engine: &'a Engine) -> Self {
        Self {
            names: &engine.names,
            power_hp: engine.power_hp,
            torque_nm: engine.torque_nm,
            path: &engine.path,
        }
    }
}

impl<'a> From<&'a Transmission> for TransmissionData<'a> {
    fn from(transmission: &'a Transmission) -> Self {
        Self {
            names: &transmission.names,
            ratios: &transmission.ratios,
            retarder: transmission.retarder,
            path: &transmission.path,
        }
    }
}

impl<'a> From<&'a Part> for PartData<'a> {
    fn from(part: &'a Part) -> Self {
        Self {
            names: &part.names,
            path: &part.path,
        }
    }
}

impl<'a> From<&'a Cargo> for CargoData<'a> {
    fn from(cargo: &'a Cargo) -> Self {
        Self {
            id: &cargo.id,
            name: &cargo.name,
            mass: cargo.mass,
            volume: cargo.volume,
            reward_per_km: cargo.reward_per_km,
            body_types: &cargo.body_types,
        }
    }
}

impl<'a> From<&'a Company> for CompanyData<'a> {
    fn from(company: &'a Company) -> Self {
        Self {
            id: &company.id,
            name: &company.name,
        }
    }
}

impl<'a> From<&'a TrailerDefinition> for TrailerData<'a> {
    fn from(trailer: &'a TrailerDefinition) -> Self {
        Self {
            id: &trailer.id,
            body_type: &trailer.body_type,
            chain_type: &trailer.chain_type,
            volume: trailer.volume,
            gross_weight_limit: trailer.gross_weight_limit,
            chassis_mass: trailer.chassis_mass,
            axles: trailer.axles,
            length: trailer.length,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let def = PathBuf::from(args.next().ok_or(USAGE)?);
    let locale = PathBuf::from(args.next().ok_or(USAGE)?);
    let output = PathBuf::from(args.next().ok_or(USAGE)?);
    let (name, game) = match args.next().ok_or(USAGE)?.as_str() {
        "ets2" => ("ets2", Game::Ets2),
        "ats" => ("ats", Game::Ats),
        _ => return Err(USAGE.into()),
    };

    let catalog = Catalog::generate(&def, &locale, game)?;
    let snapshot = Snapshot {
        game: name,
        levels: LevelsData {
            steps: &catalog.levels().steps,
            titles: catalog
                .levels()
                .titles
                .iter()
                .map(|title| LevelTitleData {
                    from: title.from,
                    names: &title.names,
                })
                .collect(),
        },
        currencies: catalog
            .currencies()
            .iter()
            .map(|currency| CurrencyData {
                code: &currency.code,
                ratio: currency.ratio,
                sign1: &currency.sign1,
                sign2: &currency.sign2,
                sign3: &currency.sign3,
            })
            .collect(),
        trailer_body_types: catalog
            .body_types()
            .iter()
            .map(|body_type| NamedData {
                token: &body_type.token,
                names: &body_type.names,
            })
            .collect(),
        trailer_brands: catalog
            .brands()
            .iter()
            .map(|brand| NamedData {
                token: &brand.token,
                names: &brand.names,
            })
            .collect(),
        cities: catalog
            .cities()
            .iter()
            .map(|city| NamedData {
                token: &city.token,
                names: &city.names,
            })
            .collect(),
        country_names: catalog
            .countries()
            .iter()
            .map(|country| NamedData {
                token: &country.token,
                names: &country.names,
            })
            .collect(),
        cargo: catalog.cargo().iter().map(CargoData::from).collect(),
        companies: catalog.companies().iter().map(CompanyData::from).collect(),
        trailers: catalog.trailers().iter().map(TrailerData::from).collect(),
        vehicles: catalog.vehicles().iter().map(VehicleData::from).collect(),
    };
    if snapshot.levels.steps.is_empty() {
        return Err(format!("no levels in {}", def.join("economy_data_xp.sii").display()).into());
    }
    if snapshot.country_names.is_empty() {
        return Err(format!("no countries in {}", def.join("country").display()).into());
    }
    if snapshot.currencies.is_empty() {
        return Err(format!(
            "no currencies in {}",
            def.join("economy_data.sii").display()
        )
        .into());
    }

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&output)?;
    serde_json::to_writer_pretty(&mut file, &snapshot)?;
    writeln!(&mut file)?;
    println!(
        "wrote {} vehicles, {} cargoes, {} companies, {} trailers, {} level steps, {} currencies, {} countries, {} brands, and {} cities to {}",
        snapshot.vehicles.len(),
        snapshot.cargo.len(),
        snapshot.companies.len(),
        snapshot.trailers.len(),
        snapshot.levels.steps.len(),
        snapshot.currencies.len(),
        snapshot.country_names.len(),
        snapshot.trailer_brands.len(),
        snapshot.cities.len(),
        output.display()
    );
    Ok(())
}
