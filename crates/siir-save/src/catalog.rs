//! Reads the truck catalog from an extracted `def/` directory.

use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use siir::{Document, Unit};

use crate::Game;
use crate::error::Error;
use crate::value::unquote;

/// Truck models and their parts.
#[derive(Debug)]
pub struct Catalog {
    trucks: Vec<TruckModel>,
}

impl Catalog {
    /// Reads every truck under `def_dir/vehicle/truck`, skipping files that do
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
        Ok(Self { trucks })
    }

    /// Every truck model, sorted by brand and model.
    pub fn trucks(&self) -> &[TruckModel] {
        &self.trucks
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
