//! Writes the catalog snapshot as JSON.

use std::error::Error;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use siir_save::Game;
use siir_save::catalog::{Catalog, Engine, Part, Transmission, TruckModel};

const USAGE: &str = "usage: catalog <def-dir> <out.json> <ets2|ats>";

#[derive(Serialize)]
struct Snapshot<'a> {
    format: u32,
    game: &'a str,
    trucks: Vec<TruckData<'a>>,
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

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let def = PathBuf::from(args.next().ok_or(USAGE)?);
    let output = PathBuf::from(args.next().ok_or(USAGE)?);
    let (name, game) = match args.next().ok_or(USAGE)?.as_str() {
        "ets2" => ("ets2", Game::Ets2),
        "ats" => ("ats", Game::Ats),
        _ => return Err(USAGE.into()),
    };

    let catalog = Catalog::generate(&def, game)?;
    let snapshot = Snapshot {
        format: 1,
        game: name,
        trucks: catalog.trucks().iter().map(TruckData::from).collect(),
    };

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = File::create(&output)?;
    serde_json::to_writer_pretty(&mut file, &snapshot)?;
    writeln!(&mut file)?;
    println!(
        "wrote {} trucks to {}",
        snapshot.trucks.len(),
        output.display()
    );
    Ok(())
}
