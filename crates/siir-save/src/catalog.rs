//! Reads the truck catalog from an extracted `def/` directory.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use siir::{Document, Index, Unit};

use crate::Game;
use crate::error::Error;
use crate::localization;
use crate::value::unquote;

const SNAPSHOT_FORMAT: u32 = 2;

/// Truck models, cargoes, companies, trailer definitions, and vehicle names
/// read from `def/` and `locale/`.
#[derive(Debug)]
pub struct Catalog {
    trucks: Vec<TruckModel>,
    cargo: Vec<Cargo>,
    companies: Vec<Company>,
    trailers: Vec<TrailerDefinition>,
    vehicle_names: Vec<VehicleName>,
}

impl Catalog {
    /// Loads the committed snapshot for `game`.
    pub fn load(game: Game) -> Result<Self, Error> {
        let source = match game {
            Game::Ets2 => include_str!("../data/ets2.json"),
            Game::Ats => include_str!("../data/ats.json"),
        };
        parse(source, game)
    }

    /// Reads every truck under `def_dir/vehicle/truck`, every cargo under
    /// `def_dir/cargo`, every company under `def_dir/company`, and every trailer
    /// definition under `def_dir/vehicle/trailer_defs`, skipping files that do
    /// not parse. Both games use the same tree today.
    ///
    /// It also reads each model's brand and series under
    /// `def_dir/vehicle/{truck,bus,car}` and resolves them with every language
    /// folder under `locale_dir`.
    pub fn generate(def_dir: &Path, locale_dir: &Path, _game: Game) -> Result<Self, Error> {
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
        let vehicle_names = model_names(def_dir, locale_dir);
        Ok(Self {
            trucks,
            cargo,
            companies,
            trailers,
            vehicle_names,
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

    /// Every vehicle model name, sorted by token.
    pub fn vehicle_names(&self) -> &[VehicleName] {
        &self.vehicle_names
    }

    /// Display name of one model in `language`, falling back to English.
    pub fn vehicle_name(&self, token: &str, language: &str) -> Option<&str> {
        let names = &self
            .vehicle_names
            .iter()
            .find(|name| name.token == token)?
            .names;
        names
            .get(language)
            .or_else(|| names.get("en_us"))
            .map(String::as_str)
    }
}

fn parse(source: &str, game: Game) -> Result<Catalog, Error> {
    let name = match game {
        Game::Ets2 => "ets2",
        Game::Ats => "ats",
    };
    let snapshot: Snapshot =
        serde_json::from_str(source).map_err(|error| Error::Catalog(error.to_string()))?;
    if snapshot.format != SNAPSHOT_FORMAT {
        return Err(Error::Catalog(format!(
            "unsupported format {}",
            snapshot.format
        )));
    }
    if snapshot.game != name {
        return Err(Error::Catalog(format!("snapshot is for {}", snapshot.game)));
    }
    Ok(snapshot.into())
}

/// A vehicle model's display name per game language.
#[derive(Debug)]
pub struct VehicleName {
    /// Model folder token, for example `volvo.9700_23`.
    pub token: String,
    /// Name per game language, for example `en_us` and `pt_br`.
    pub names: BTreeMap<String, String>,
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

#[derive(serde::Deserialize)]
struct Snapshot {
    format: u32,
    game: String,
    trucks: Vec<TruckData>,
    cargo: Vec<CargoData>,
    companies: Vec<CompanyData>,
    trailers: Vec<TrailerData>,
    models: Vec<ModelData>,
}

#[derive(serde::Deserialize)]
struct ModelData {
    token: String,
    names: BTreeMap<String, String>,
}

#[derive(serde::Deserialize)]
struct TruckData {
    brand: String,
    model: String,
    engines: Vec<EngineData>,
    transmissions: Vec<TransmissionData>,
    cabins: Vec<PartData>,
    chassis: Vec<PartData>,
    paint_jobs: Vec<PartData>,
}

#[derive(serde::Deserialize)]
struct EngineData {
    name: String,
    power_hp: u32,
    torque_nm: u32,
    path: String,
}

#[derive(serde::Deserialize)]
struct TransmissionData {
    name: String,
    ratios: Vec<f32>,
    retarder: bool,
    path: String,
}

#[derive(serde::Deserialize)]
struct PartData {
    name: String,
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
            trucks: snapshot.trucks.into_iter().map(TruckModel::from).collect(),
            cargo: snapshot.cargo.into_iter().map(Cargo::from).collect(),
            companies: snapshot.companies.into_iter().map(Company::from).collect(),
            trailers: snapshot
                .trailers
                .into_iter()
                .map(TrailerDefinition::from)
                .collect(),
            vehicle_names: snapshot.models.into_iter().map(VehicleName::from).collect(),
        }
    }
}

impl From<ModelData> for VehicleName {
    fn from(model: ModelData) -> Self {
        Self {
            token: model.token,
            names: model.names,
        }
    }
}

impl From<TruckData> for TruckModel {
    fn from(truck: TruckData) -> Self {
        Self {
            brand: truck.brand,
            model: truck.model,
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
            name: engine.name,
            power_hp: engine.power_hp,
            torque_nm: engine.torque_nm,
            path: engine.path,
        }
    }
}

impl From<TransmissionData> for Transmission {
    fn from(transmission: TransmissionData) -> Self {
        Self {
            name: transmission.name,
            ratios: transmission.ratios,
            retarder: transmission.retarder,
            path: transmission.path,
        }
    }
}

impl From<PartData> for Part {
    fn from(part: PartData) -> Self {
        Self {
            name: part.name,
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

/// Resolves the brand and series of every vehicle model in every extracted
/// language.
fn model_names(def_dir: &Path, locale_dir: &Path) -> Vec<VehicleName> {
    let languages: Vec<(String, BTreeMap<String, String>)> = localization::languages(locale_dir)
        .into_iter()
        .map(|language| {
            let texts = localization::read(locale_dir, &language);
            (language, texts)
        })
        .collect();
    let mut names = Vec::new();
    for kind in ["truck", "bus", "car"] {
        let Ok(entries) = directories(&def_dir.join("vehicle").join(kind)) else {
            continue;
        };
        for folder in entries {
            if !folder.join("data.sii").is_file() {
                continue;
            }
            let Some(token) = folder.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !token.contains('.') {
                continue;
            }
            let Some(keys) = info_keys(&folder.join("data.sii")) else {
                continue;
            };
            let mut by_language = BTreeMap::new();
            for (language, texts) in &languages {
                let parts: Vec<&str> = keys
                    .iter()
                    .filter_map(|key| texts.get(key).map(String::as_str))
                    .collect();
                if !parts.is_empty() {
                    by_language.insert(language.clone(), parts.join(" "));
                }
            }
            if !by_language.is_empty() {
                names.push(VehicleName {
                    token: token.to_owned(),
                    names: by_language,
                });
            }
        }
    }
    names.sort_by(|left, right| left.token.cmp(&right.token));
    names.dedup_by(|left, right| left.token == right.token);
    names
}

/// The localization keys of a model's brand and series `info[]` entries.
fn info_keys(file: &Path) -> Option<Vec<String>> {
    let source = fs::read_to_string(file).ok()?;
    let source = strip_includes(&source);
    let document = Document::parse(&source).ok()?;
    let unit = document.units().next()?;
    let mut keys = Vec::new();
    for nth in 0..2 {
        let Some(value) = unit.get_nth("info[]", nth) else {
            break;
        };
        let Some(key) = unquote(value)
            .strip_prefix("@@")
            .and_then(|value| value.strip_suffix("@@"))
        else {
            continue;
        };
        keys.push(key.to_owned());
    }
    (!keys.is_empty()).then_some(keys)
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
        chassis_mass: number(unit, "chassis_mass")?.round() as u64,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot(format: u32, game: &str) -> String {
        format!(
            r#"{{"format":{format},"game":"{game}","trucks":[],"cargo":[],"companies":[],"trailers":[],"models":[]}}"#
        )
    }

    #[test]
    fn rejects_another_format() {
        let error = parse(&snapshot(9, "ets2"), Game::Ets2).unwrap_err();
        assert!(error.to_string().contains("unsupported format 9"));
    }

    #[test]
    fn rejects_another_game() {
        let error = parse(&snapshot(2, "ats"), Game::Ets2).unwrap_err();
        assert!(error.to_string().contains("snapshot is for ats"));
    }
}
