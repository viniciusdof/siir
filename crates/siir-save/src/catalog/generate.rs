use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use siir::{Document, Index, Unit};

use crate::Game;
use crate::error::Error;
use crate::localization;
use crate::value::unquote;

use super::{
    Cargo, Catalog, Company, Currency, Engine, LevelTitle, Levels, Named, Part, TrailerDefinition,
    Transmission, VehicleModel,
};

type Languages = Vec<(String, BTreeMap<String, String>)>;

pub(super) fn generate(def_dir: &Path, locale_dir: &Path, _game: Game) -> Result<Catalog, Error> {
    let languages = languages(locale_dir);
    let vehicles = vehicles(def_dir, &languages);
    let mut cargo = read_definitions(def_dir, &def_dir.join("cargo"), cargo, &languages);
    cargo.sort_by(|left, right| left.id.cmp(&right.id));
    let mut companies = read_definitions(def_dir, &def_dir.join("company"), company, &languages);
    companies.sort_by(|left, right| left.id.cmp(&right.id));
    let mut trailers = read_definitions(
        def_dir,
        &def_dir.join("vehicle").join("trailer_defs"),
        trailer_definition,
        &languages,
    );
    trailers.sort_by(|left, right| left.id.cmp(&right.id));
    let body_types = body_type_names(&trailers, &languages);
    let brands = brands(def_dir, &languages);
    let cities = cities(&def_dir.join("city"), &languages);
    let drivers = drivers(&locale_dir.join("en_gb").join("driver_names.sii"));
    let levels = levels(&def_dir.join("economy_data_xp.sii"), &languages);
    let currencies = currencies(&def_dir.join("economy_data.sii"));
    let countries = countries(&def_dir.join("country"), &languages);
    Ok(Catalog {
        vehicles,
        cargo,
        companies,
        trailers,
        body_types,
        brands,
        cities,
        drivers,
        levels,
        currencies,
        countries,
    })
}

fn localized_names(value: &str, languages: &Languages) -> BTreeMap<String, String> {
    let Some(key) = value
        .strip_prefix("@@")
        .and_then(|value| value.strip_suffix("@@"))
    else {
        return names_in_every_language(value, languages);
    };
    names_from_key(key, languages)
}

fn names_from_key(key: &str, languages: &Languages) -> BTreeMap<String, String> {
    let mut names: BTreeMap<String, String> = languages
        .iter()
        .filter_map(|(language, texts)| Some((language.clone(), texts.get(key)?.clone())))
        .collect();
    omit_english(&mut names);
    names
}

fn names_in_every_language(text: &str, languages: &Languages) -> BTreeMap<String, String> {
    let mut names: BTreeMap<String, String> = languages
        .iter()
        .map(|(language, _)| (language.clone(), text.to_owned()))
        .collect();
    omit_english(&mut names);
    names
}

fn omit_english(names: &mut BTreeMap<String, String>) {
    if let Some(english) = names.get("en_us").cloned() {
        names.retain(|language, text| language == "en_us" || *text != english);
    }
}

fn languages(locale_dir: &Path) -> Languages {
    localization::languages(locale_dir)
        .into_iter()
        .map(|language| {
            let texts = localization::read(locale_dir, &language);
            (language, texts)
        })
        .collect()
}

fn drivers(file: &Path) -> Vec<String> {
    let Ok(source) = fs::read_to_string(file) else {
        return Vec::new();
    };
    let source = strip_includes(source.trim_start_matches('\u{feff}')).into_owned();
    let Ok(document) = Document::parse(&source) else {
        return Vec::new();
    };
    let Some(unit) = document.units_of_class("driver_names").next() else {
        return Vec::new();
    };
    let mut entries: Vec<(u32, String)> = unit
        .attrs()
        .filter_map(|(attribute, index, value)| match (attribute, index) {
            ("name", Index::At(at)) => {
                let name = unquote(value);
                let name = name.strip_prefix('+').unwrap_or(name);
                Some((at, name.to_owned()))
            }
            _ => None,
        })
        .collect();
    entries.sort_by_key(|(at, _)| *at);
    if entries
        .iter()
        .enumerate()
        .any(|(ix, (at, _))| ix as u32 != *at)
    {
        return Vec::new();
    }
    entries.into_iter().map(|(_, name)| name).collect()
}

fn levels(file: &Path, languages: &Languages) -> Levels {
    let Ok(source) = fs::read_to_string(file) else {
        return Levels::default();
    };
    let source = strip_includes(source.trim_start_matches('\u{feff}')).into_owned();
    let Ok(document) = Document::parse(&source) else {
        return Levels::default();
    };
    let Some(unit) = document
        .units()
        .find(|unit| unit.id() == "economy.data.xp_levels")
    else {
        return Levels::default();
    };
    let mut steps: Vec<(u32, u32)> = unit
        .attrs()
        .filter_map(|(attribute, index, value)| match (attribute, index) {
            ("level_xp", Index::At(at)) => Some((at, value.parse().ok()?)),
            _ => None,
        })
        .collect();
    steps.sort_by_key(|(at, _)| *at);
    if steps
        .iter()
        .enumerate()
        .any(|(ix, (at, _))| ix as u32 != *at)
    {
        return Levels::default();
    }
    let mut titles: Vec<LevelTitle> = strings(unit, "level_names")
        .iter()
        .filter_map(|entry| {
            let (from, name) = entry.split_once('|')?;
            let key = name.strip_prefix("@@")?.strip_suffix("@@")?;
            Some(LevelTitle {
                from: from.trim().parse().ok()?,
                names: names_from_key(key, languages),
            })
        })
        .collect();
    titles.sort_by_key(|title| title.from);
    Levels {
        steps: steps.into_iter().map(|(_, step)| step).collect(),
        titles,
    }
}

fn currencies(file: &Path) -> Vec<Currency> {
    let Ok(source) = fs::read_to_string(file) else {
        return Vec::new();
    };
    let source = strip_includes(source.trim_start_matches('\u{feff}')).into_owned();
    let Ok(document) = Document::parse(&source) else {
        return Vec::new();
    };
    let Some(unit) = document
        .units()
        .find(|unit| unit.get("currency_code[]").is_some())
    else {
        return Vec::new();
    };
    let codes = strings(unit, "currency_code");
    let ratios = strings(unit, "currency_ratio");
    if ratios.len() != codes.len() {
        return Vec::new();
    }
    let sign1 = strings(unit, "currency_sign1");
    let sign2 = strings(unit, "currency_sign2");
    let sign3 = strings(unit, "currency_sign3");
    let sign = |values: &[String], ix: usize| values.get(ix).cloned().unwrap_or_default();
    let mut currencies = Vec::with_capacity(codes.len());
    for ix in 0..codes.len() {
        let Ok(ratio) = ratios[ix].parse::<f64>() else {
            return Vec::new();
        };
        currencies.push(Currency {
            code: codes[ix].clone(),
            ratio,
            sign1: sign(&sign1, ix),
            sign2: sign(&sign2, ix),
            sign3: sign(&sign3, ix),
        });
    }
    currencies
}

fn countries(folder: &Path, languages: &Languages) -> Vec<Named> {
    let mut found: Vec<Named> = named_units(folder, "country.data.", "name_localized", "name")
        .into_iter()
        .map(|(token, key, fallback)| Named {
            token,
            names: names_with_fallback(&key, &fallback, languages),
        })
        .collect();
    found.sort_by(|left, right| left.token.cmp(&right.token));
    found.dedup_by(|left, right| left.token == right.token);
    found
}

fn cities(folder: &Path, languages: &Languages) -> Vec<Named> {
    let mut found: Vec<Named> = named_units(folder, "city.", "city_name_localized", "city_name")
        .into_iter()
        .map(|(token, key, fallback)| Named {
            token,
            names: names_with_fallback(&key, &fallback, languages),
        })
        .collect();
    found.sort_by(|left, right| left.token.cmp(&right.token));
    found.dedup_by(|left, right| left.token == right.token);
    found
}

fn named_units(
    folder: &Path,
    prefix: &str,
    localized: &str,
    plain: &str,
) -> Vec<(String, String, String)> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();
    files.sort();
    let mut found = Vec::new();
    for file in files {
        let Ok(source) = fs::read_to_string(&file) else {
            continue;
        };
        let source = strip_includes(source.trim_start_matches('\u{feff}')).into_owned();
        let source = if source.trim_start().starts_with("SiiNunit") {
            source
        } else {
            format!("SiiNunit\n{{\n{source}\n}}\n")
        };
        let Ok(document) = Document::parse(&source) else {
            continue;
        };
        for unit in document.units() {
            let Some(token) = unit.id().strip_prefix(prefix) else {
                continue;
            };
            let Some(key) = unit
                .get(localized)
                .map(unquote)
                .and_then(|value| {
                    value
                        .strip_prefix("@@")
                        .and_then(|key| key.strip_suffix("@@"))
                })
                .map(str::to_owned)
            else {
                continue;
            };
            let Some(plain) = unit.get(plain).map(unquote) else {
                continue;
            };
            found.push((token.to_owned(), key, plain.to_owned()));
        }
    }
    found
}

fn names_with_fallback(
    key: &str,
    fallback: &str,
    languages: &Languages,
) -> BTreeMap<String, String> {
    let mut names: BTreeMap<String, String> = languages
        .iter()
        .map(|(language, texts)| {
            let name = texts
                .get(key)
                .cloned()
                .unwrap_or_else(|| fallback.to_owned());
            (language.clone(), name)
        })
        .collect();
    omit_english(&mut names);
    names
}

fn vehicles(def_dir: &Path, languages: &Languages) -> Vec<VehicleModel> {
    let mut vehicles = Vec::new();
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
            if !token
                .split_once('.')
                .is_some_and(|(brand, model)| !brand.is_empty() && !model.is_empty())
            {
                continue;
            }
            let mut names = BTreeMap::new();
            let keys = info_keys(&folder.join("data.sii")).unwrap_or_default();
            for (language, texts) in languages {
                let parts: Vec<&str> = keys
                    .iter()
                    .filter_map(|key| texts.get(key).map(String::as_str))
                    .collect();
                if !parts.is_empty() {
                    names.insert(language.clone(), parts.join(" "));
                }
            }
            omit_english(&mut names);
            let read = |dir| folder.join(dir);
            vehicles.push(VehicleModel {
                token: token.to_owned(),
                names,
                engines: read_definitions(def_dir, &read("engine"), engine, languages),
                transmissions: read_definitions(
                    def_dir,
                    &read("transmission"),
                    transmission,
                    languages,
                ),
                cabins: read_definitions(def_dir, &read("cabin"), part, languages),
                chassis: read_definitions(def_dir, &read("chassis"), part, languages),
                paint_jobs: read_definitions(def_dir, &read("paint_job"), paint_job, languages),
            });
        }
    }
    vehicles.sort_by(|left, right| left.token.cmp(&right.token));
    vehicles.dedup_by(|left, right| left.token == right.token);
    vehicles
}

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
    extract: fn(&Unit<'_>, &str, &Languages) -> Option<T>,
    languages: &Languages,
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
        if let Some(item) = extract(unit, &data_path(def_dir, &file), languages) {
            items.push(item);
        }
    }
    items
}

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

fn engine(unit: &Unit<'_>, path: &str, languages: &Languages) -> Option<Engine> {
    let name = unquote(unit.get("name")?);
    let info = unquote(unit.get_nth("info[]", 0)?);
    Some(Engine {
        names: localized_names(name, languages),
        power_hp: info.split_whitespace().next()?.parse().ok()?,
        torque_nm: unquote(unit.get("torque")?).parse().ok()?,
        path: path.to_owned(),
    })
}

fn transmission(unit: &Unit<'_>, path: &str, languages: &Languages) -> Option<Transmission> {
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
        names: localized_names(name, languages),
        ratios,
        retarder,
        path: path.to_owned(),
    })
}

fn part(unit: &Unit<'_>, path: &str, languages: &Languages) -> Option<Part> {
    Some(Part {
        names: localized_names(unquote(unit.get("name")?), languages),
        path: path.to_owned(),
    })
}

fn paint_job(unit: &Unit<'_>, path: &str, _: &Languages) -> Option<Part> {
    unit.get("name")?;
    Some(Part {
        names: BTreeMap::new(),
        path: path.to_owned(),
    })
}

fn cargo(unit: &Unit<'_>, _: &str, _: &Languages) -> Option<Cargo> {
    Some(Cargo {
        id: unit.id().to_owned(),
        name: unquote(unit.get("name")?).to_owned(),
        mass: number(unit, "mass")?,
        volume: number(unit, "volume")?,
        reward_per_km: number(unit, "unit_reward_per_km")?,
        body_types: strings(unit, "body_types"),
    })
}

fn company(unit: &Unit<'_>, _: &str, _: &Languages) -> Option<Company> {
    Some(Company {
        id: unit.id().to_owned(),
        name: unquote(unit.get("name")?).to_owned(),
    })
}

fn trailer_definition(unit: &Unit<'_>, _: &str, _: &Languages) -> Option<TrailerDefinition> {
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

fn body_type_names(trailers: &[TrailerDefinition], languages: &Languages) -> Vec<Named> {
    let mut tokens: Vec<&str> = trailers
        .iter()
        .map(|trailer| trailer.body_type.as_str())
        .collect();
    tokens.sort_unstable();
    tokens.dedup();
    tokens
        .into_iter()
        .map(|token| Named {
            token: token.to_owned(),
            names: names_from_key(token, languages),
        })
        .collect()
}

fn brands(def_dir: &Path, languages: &Languages) -> Vec<Named> {
    let Ok(entries) = directories(&def_dir.join("vehicle").join("trailer_owned")) else {
        return Vec::new();
    };
    let mut brands: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for folder in entries {
        let Some(name) = folder.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some((brand, _)) = name.split_once('.') else {
            continue;
        };
        let Some(key) = brand_key(&folder.join("data.sii")) else {
            continue;
        };
        brands
            .entry(brand.to_owned())
            .or_insert_with(|| names_from_key(&key, languages));
    }
    brands
        .into_iter()
        .map(|(token, names)| Named { token, names })
        .collect()
}

fn brand_key(file: &Path) -> Option<String> {
    let source = fs::read_to_string(file).ok()?;
    let source = strip_includes(&source);
    let document = Document::parse(&source).ok()?;
    let unit = document.units().next()?;
    unquote(unit.get_nth("info[]", 0)?)
        .strip_prefix("@@")?
        .strip_suffix("@@")
        .map(str::to_owned)
}

fn number(unit: &Unit<'_>, name: &str) -> Option<f32> {
    let value = unquote(unit.get(name)?);
    match value.strip_prefix('&') {
        Some(hex) => u32::from_str_radix(hex, 16).ok().map(f32::from_bits),
        None => value.parse().ok(),
    }
}

fn strings(unit: &Unit<'_>, name: &str) -> Vec<String> {
    unit.attrs()
        .filter(|(attribute, index, _)| *attribute == name && *index == Index::Unspecified)
        .map(|(_, _, value)| unquote(value).to_owned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_the_currency_table_when_a_sign_array_is_missing() {
        let path =
            std::env::temp_dir().join(format!("siir-catalog-currency-{}.sii", std::process::id()));
        std::fs::write(
            &path,
            "SiiNunit\n{\neconomy_data : economy.data.storage\n{\n\
             currency_code[]: \"EUR\"\n\
             currency_ratio[]: 1\n\
             currency_sign2[]: \"€\"\n\
             currency_sign3[]: \"\"\n\
             currency_code[]: \"CZK\"\n\
             currency_ratio[]: 24.4\n\
             currency_sign2[]: \"\"\n\
             currency_sign3[]: \" Kč\"\n}\n}\n",
        )
        .unwrap();
        let currencies = currencies(&path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(currencies.len(), 2, "a missing sign array keeps the table");
        assert_eq!(currencies[0].sign1, "");
        assert_eq!(currencies[1].code, "CZK");
        assert_eq!(currencies[1].sign3, " Kč");
    }
}
