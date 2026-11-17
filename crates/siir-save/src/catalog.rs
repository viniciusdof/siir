//! Reads the truck catalog from an extracted `def/` directory.

use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use siir::{Document, Index, Unit};

use crate::Game;
use crate::error::Error;
use crate::value::unquote;

/// Truck models, cargoes, companies, and trailer definitions read from `def/`.
#[derive(Debug)]
pub struct Catalog {
    trucks: Vec<TruckModel>,
    cargo: Vec<Cargo>,
    companies: Vec<Company>,
    trailers: Vec<TrailerDefinition>,
}

impl Catalog {
    /// Reads every truck under `def_dir/vehicle/truck`, every cargo under
    /// `def_dir/cargo`, every company under `def_dir/company`, and every trailer
    /// definition under `def_dir/vehicle/trailer_defs`, skipping files that do
    /// not parse. Both games use the same tree today.
    pub fn generate(def_dir: &Path, _game: Game) -> Result<Self, Error> {
        let mut trucks = Vec::new();
        for entry in directories(&def_dir.join("vehicle").join("truck"))? {
            let Some(name) = entry.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let Some((brand, model)) = name.split_once('.') else {
                continue;
            };
            if brand.is_empty() || model.is_empty() {
                continue;
            }
            let folder = entry.as_path();
            if !folder.join("data.sii").is_file() {
                continue;
            }
            trucks.push(TruckModel {
                brand: brand.to_owned(),
                model: model.to_owned(),
                engines: read_definitions(def_dir, &folder.join("engine"), engine),
                transmissions: read_definitions(
                    def_dir,
                    &folder.join("transmission"),
                    transmission,
                ),
                cabins: read_definitions(def_dir, &folder.join("cabin"), part),
                chassis: read_definitions(def_dir, &folder.join("chassis"), part),
                paint_jobs: read_definitions(def_dir, &folder.join("paint_job"), part),
            });
        }
        trucks.sort_by(|left, right| (&left.brand, &left.model).cmp(&(&right.brand, &right.model)));
        let mut cargo = read_definitions(def_dir, &def_dir.join("cargo"), cargo);
        cargo.sort_by(|left, right| left.id.cmp(&right.id));
        let mut companies = read_definitions(def_dir, &def_dir.join("company"), company);
        companies.sort_by(|left, right| left.id.cmp(&right.id));
        let mut trailers = read_definitions(
            def_dir,
            &def_dir.join("vehicle").join("trailer_defs"),
            trailer_definition,
        );
        trailers.sort_by(|left, right| left.id.cmp(&right.id));
        Ok(Self {
            trucks,
            cargo,
            companies,
            trailers,
        })
    }

    /// Every truck model, sorted by brand and model.
    pub fn trucks(&self) -> &[TruckModel] {
        &self.trucks
    }

    /// Every cargo definition, sorted by id.
    pub fn cargo(&self) -> &[Cargo] {
        &self.cargo
    }

    /// Every company definition, sorted by id.
    pub fn companies(&self) -> &[Company] {
        &self.companies
    }

    /// Every trailer definition, sorted by id.
    pub fn trailers(&self) -> &[TrailerDefinition] {
        &self.trailers
    }
}

/// One `def/vehicle/truck/<brand>.<model>` folder.
#[derive(Debug)]
pub struct TruckModel {
    /// Brand folder token, for example `renault`.
    pub brand: String,
    /// Model folder remainder, for example `etech_t`.
    pub model: String,
    /// Engines, sorted by `data_path`.
    pub engines: Vec<Engine>,
    /// Transmissions, sorted by `data_path`.
    pub transmissions: Vec<Transmission>,
    /// Cabin definitions.
    pub cabins: Vec<Part>,
    /// Chassis definitions.
    pub chassis: Vec<Part>,
    /// Paint job definitions.
    pub paint_jobs: Vec<Part>,
}

/// One engine definition.
#[derive(Debug)]
pub struct Engine {
    /// Display name from the definition.
    pub name: String,
    /// Power in horsepower, the first token of the definition's first `info[]`.
    pub power_hp: u32,
    /// Maximum torque in Newton metres.
    pub torque_nm: u32,
    /// `data_path` the save stores for this engine.
    pub path: String,
}

/// One transmission definition.
#[derive(Debug)]
pub struct Transmission {
    /// Display name from the definition.
    pub name: String,
    /// Forward gear ratios, in gear order.
    pub ratios: Vec<f32>,
    /// Whether the definition has a retarder.
    pub retarder: bool,
    /// `data_path` the save stores for this transmission.
    pub path: String,
}

/// A cabin, chassis, or paint job.
#[derive(Debug)]
pub struct Part {
    /// Display name from the definition.
    pub name: String,
    /// `data_path` the save stores for this part.
    pub path: String,
}

/// One cargo definition.
#[derive(Debug)]
pub struct Cargo {
    /// Cargo id, for example `cargo.stones`.
    pub id: String,
    /// Display name from the definition, often a translation key.
    pub name: String,
    /// Mass of one unit in kilograms.
    pub mass: f32,
    /// Volume of one unit in cubic metres.
    pub volume: f32,
    /// Reward per kilometer for one unit.
    pub reward_per_km: f32,
    /// Trailer body types that can carry the cargo.
    pub body_types: Vec<String>,
}

/// One company definition.
#[derive(Debug)]
pub struct Company {
    /// Company id, for example `company.permanent.bhv`.
    pub id: String,
    /// Display name from the definition.
    pub name: String,
}

/// One trailer definition.
#[derive(Debug)]
pub struct TrailerDefinition {
    /// Definition id, for example `trailer_def.wielton.scrapm`.
    pub id: String,
    /// Body type, for example `curtainside`.
    pub body_type: String,
    /// Chain type, for example `single` or `double`.
    pub chain_type: String,
    /// Cargo volume in cubic metres.
    pub volume: f32,
    /// Gross weight limit in kilograms.
    pub gross_weight_limit: u64,
    /// Chassis mass in kilograms.
    pub chassis_mass: u64,
    /// Number of axles.
    pub axles: u64,
    /// Trailer length in metres.
    pub length: f32,
}

fn directories(root: &Path) -> Result<Vec<PathBuf>, Error> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            found.push(entry.path());
        }
    }
    found.sort();
    Ok(found)
}

fn read_definitions<T>(
    def_dir: &Path,
    folder: &Path,
    extract: fn(&Unit<'_>, &str) -> Option<T>,
) -> Vec<T> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .is_some_and(|extension| extension == "sii" || extension == "sui")
        })
        .collect();
    files.sort();

    let mut items = Vec::new();
    for file in files {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        let source = strip_includes(&source);
        // Included cargo and company files are fragments without the
        // `SiiNunit { ... }` wrapper the game adds around a complete file.
        let source = if source.trim_start().starts_with("SiiNunit") {
            source
        } else {
            Cow::Owned(format!("SiiNunit\n{{\n{source}\n}}\n"))
        };
        let Ok(document) = Document::parse(&source) else {
            continue;
        };
        let Some(unit) = document.units().next() else {
            continue;
        };
        if let Some(item) = extract(unit, &data_path(def_dir, &file)) {
            items.push(item);
        }
    }
    items
}

// shortcut: include directives are dropped, not resolved; resolve them when a
// field we read starts living in an included file.
fn strip_includes(source: &str) -> Cow<'_, str> {
    if !source.contains("@include") {
        return Cow::Borrowed(source);
    }
    Cow::Owned(
        source
            .lines()
            .filter(|line| !line.trim_start().starts_with("@include"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn data_path(def_dir: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(def_dir).unwrap_or(file);
    format!("/def/{}", relative.to_string_lossy().replace('\\', "/"))
}

fn engine(unit: &Unit<'_>, path: &str) -> Option<Engine> {
    let name = unquote(unit.get("name")?);
    let info = unquote(unit.get_nth("info[]", 0)?);
    Some(Engine {
        name: name.to_owned(),
        power_hp: info.split_whitespace().next()?.parse().ok()?,
        torque_nm: unquote(unit.get("torque")?).parse().ok()?,
        path: path.to_owned(),
    })
}

fn transmission(unit: &Unit<'_>, path: &str) -> Option<Transmission> {
    let name = unquote(unit.get("name")?);
    let mut ratios = Vec::new();
    let mut retarder = false;
    for (attribute, _, value) in unit.attrs() {
        match attribute {
            "ratios_forward" => ratios.push(value.parse().ok()?),
            "retarder" => retarder = true,
            _ => {}
        }
    }
    if ratios.is_empty() {
        return None;
    }
    Some(Transmission {
        name: name.to_owned(),
        ratios,
        retarder,
        path: path.to_owned(),
    })
}

fn part(unit: &Unit<'_>, path: &str) -> Option<Part> {
    Some(Part {
        name: unquote(unit.get("name")?).to_owned(),
        path: path.to_owned(),
    })
}

fn cargo(unit: &Unit<'_>, _: &str) -> Option<Cargo> {
    Some(Cargo {
        id: unit.id().to_owned(),
        name: unquote(unit.get("name")?).to_owned(),
        mass: number(unit, "mass")?,
        volume: number(unit, "volume")?,
        reward_per_km: number(unit, "unit_reward_per_km")?,
        body_types: strings(unit, "body_types"),
    })
}

fn company(unit: &Unit<'_>, _: &str) -> Option<Company> {
    Some(Company {
        id: unit.id().to_owned(),
        name: unquote(unit.get("name")?).to_owned(),
    })
}

fn trailer_definition(unit: &Unit<'_>, _: &str) -> Option<TrailerDefinition> {
    Some(TrailerDefinition {
        id: unit.id().to_owned(),
        body_type: unquote(unit.get("body_type")?).to_owned(),
        chain_type: unquote(unit.get("chain_type")?).to_owned(),
        volume: number(unit, "volume")?,
        gross_weight_limit: unquote(unit.get("gross_trailer_weight_limit")?)
            .parse()
            .ok()?,
        chassis_mass: unquote(unit.get("chassis_mass")?).parse().ok()?,
        axles: unquote(unit.get("axles")?).parse().ok()?,
        length: number(unit, "length")?,
    })
}

fn number(unit: &Unit<'_>, name: &str) -> Option<f32> {
    let value = unquote(unit.get(name)?);
    match value.strip_prefix('&') {
        Some(hex) => u32::from_str_radix(hex, 16).ok().map(f32::from_bits),
        None => value.parse().ok(),
    }
}

/// Every value of an implicit `name[]` array, in file order.
fn strings(unit: &Unit<'_>, name: &str) -> Vec<String> {
    unit.attrs()
        .filter(|(attribute, index, _)| *attribute == name && *index == Index::Unspecified)
        .map(|(_, _, value)| unquote(value).to_owned())
        .collect()
}
