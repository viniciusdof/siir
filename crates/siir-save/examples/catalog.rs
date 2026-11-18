//! Writes the catalog snapshot as JSON.

use std::collections::BTreeMap;
use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use siir_save::Game;
use siir_save::catalog::{
    Cargo, Catalog, Company, Engine, Part, TrailerDefinition, Transmission, TruckModel,
};

const USAGE: &str = "usage: catalog <def-dir> <locale-dir> <out.json> <ets2|ats>";

#[derive(Serialize)]
struct Snapshot<'a> {
    format: u32,
    game: &'a str,
    trucks: Vec<TruckData<'a>>,
    cargo: Vec<CargoData<'a>>,
    companies: Vec<CompanyData<'a>>,
    trailers: Vec<TrailerData<'a>>,
    models: Vec<ModelData<'a>>,
}

#[derive(Serialize)]
struct ModelData<'a> {
    token: &'a str,
    names: &'a BTreeMap<String, String>,
}

#[derive(Serialize)]
struct TruckData<'a> {
    brand: &'a str,
    model: &'a str,
    engines: Vec<EngineData<'a>>,
    transmissions: Vec<TransmissionData<'a>>,
    cabins: Vec<PartData<'a>>,
    chassis: Vec<PartData<'a>>,
    paint_jobs: Vec<PartData<'a>>,
}

#[derive(Serialize)]
struct EngineData<'a> {
    name: &'a str,
    power_hp: u32,
    torque_nm: u32,
    path: &'a str,
}

#[derive(Serialize)]
struct TransmissionData<'a> {
    name: &'a str,
    ratios: &'a [f32],
    retarder: bool,
    path: &'a str,
}

#[derive(Serialize)]
struct PartData<'a> {
    name: &'a str,
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

impl<'a> From<&'a TruckModel> for TruckData<'a> {
    fn from(truck: &'a TruckModel) -> Self {
        Self {
            brand: &truck.brand,
            model: &truck.model,
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
            name: &engine.name,
            power_hp: engine.power_hp,
            torque_nm: engine.torque_nm,
            path: &engine.path,
        }
    }
}

impl<'a> From<&'a Transmission> for TransmissionData<'a> {
    fn from(transmission: &'a Transmission) -> Self {
        Self {
            name: &transmission.name,
            ratios: &transmission.ratios,
            retarder: transmission.retarder,
            path: &transmission.path,
        }
    }
}

impl<'a> From<&'a Part> for PartData<'a> {
    fn from(part: &'a Part) -> Self {
        Self {
            name: &part.name,
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
        format: 2,
        game: name,
        trucks: catalog.trucks().iter().map(TruckData::from).collect(),
        cargo: catalog.cargo().iter().map(CargoData::from).collect(),
        companies: catalog.companies().iter().map(CompanyData::from).collect(),
        trailers: catalog.trailers().iter().map(TrailerData::from).collect(),
        models: catalog
            .vehicle_names()
            .iter()
            .map(|name| ModelData {
                token: &name.token,
                names: &name.names,
            })
            .collect(),
    };

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&output)?;
    serde_json::to_writer_pretty(&mut file, &snapshot)?;
    writeln!(&mut file)?;
    println!(
        "wrote {} trucks, {} cargoes, {} companies, {} trailers, and {} vehicle names to {}",
        snapshot.trucks.len(),
        snapshot.cargo.len(),
        snapshot.companies.len(),
        snapshot.trailers.len(),
        snapshot.models.len(),
        output.display()
    );
    Ok(())
}
