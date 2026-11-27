//! Game catalog loaded from snapshots or extracted `def/` and `locale/` trees.

mod generate;
mod snapshot;

use std::collections::BTreeMap;
use std::path::Path;

use crate::Game;
use crate::error::Error;

/// Vehicle models, cargoes, companies, trailer definitions, and the
/// experience levels read from `def/` and `locale/`.
#[derive(Debug)]
pub struct Catalog {
    vehicles: Vec<VehicleModel>,
    cargo: Vec<Cargo>,
    companies: Vec<Company>,
    trailers: Vec<TrailerDefinition>,
    body_types: Vec<Named>,
    brands: Vec<Named>,
    cities: Vec<Named>,
    drivers: Vec<String>,
    levels: Levels,
    currencies: Vec<Currency>,
    countries: Vec<Named>,
}

impl Catalog {
    /// Loads the committed snapshot for `game`.
    pub fn load(game: Game) -> Result<Self, Error> {
        let source = match game {
            Game::Ets2 => include_str!("../../data/ets2.json"),
            Game::Ats => include_str!("../../data/ats.json"),
        };
        snapshot::parse(source, game)
    }

    /// Generates a catalog from extracted `def/` and `locale/` directories,
    /// skipping files that do not parse.
    pub fn generate(def_dir: &Path, locale_dir: &Path, game: Game) -> Result<Self, Error> {
        generate::generate(def_dir, locale_dir, game)
    }

    /// Every truck, bus, and car model, sorted by token.
    pub fn vehicles(&self) -> &[VehicleModel] {
        &self.vehicles
    }

    /// One model by the token a save stores, `brand.model`.
    pub fn vehicle(&self, token: &str) -> Option<&VehicleModel> {
        self.vehicles.iter().find(|vehicle| vehicle.token == token)
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

    /// One trailer definition by the id a save stores.
    pub fn trailer(&self, id: &str) -> Option<&TrailerDefinition> {
        self.trailers.iter().find(|trailer| trailer.id == id)
    }

    /// Every trailer body type, sorted by token.
    pub fn body_types(&self) -> &[Named] {
        &self.body_types
    }

    /// Name of one trailer body type in `language`, else English.
    pub fn body_type_name(&self, token: &str, language: &str) -> Option<&str> {
        lookup(&self.body_types, token, language)
    }

    /// Every trailer brand, sorted by token.
    pub fn brands(&self) -> &[Named] {
        &self.brands
    }

    /// Name of one trailer brand in `language`, else English.
    pub fn brand_name(&self, token: &str, language: &str) -> Option<&str> {
        lookup(&self.brands, token, language)
    }

    /// Every city, sorted by token.
    pub fn cities(&self) -> &[Named] {
        &self.cities
    }

    /// Name of one city in `language`, else English.
    pub fn city_name(&self, token: &str, language: &str) -> Option<&str> {
        lookup(&self.cities, token, language)
    }

    /// Every AI driver name, indexed by the number in `driver.<index>`.
    pub fn drivers(&self) -> &[String] {
        &self.drivers
    }

    /// Display name of one AI driver by its unit id (`driver.<index>`).
    pub fn driver_name(&self, id: &str) -> Option<&str> {
        let index: usize = id.strip_prefix("driver.")?.parse().ok()?;
        self.drivers.get(index).map(String::as_str)
    }

    /// Display name of one model in `language`, falling back to English.
    pub fn vehicle_name(&self, token: &str, language: &str) -> Option<&str> {
        localized(&self.vehicle(token)?.names, language)
    }

    /// The player's experience levels.
    pub fn levels(&self) -> &Levels {
        &self.levels
    }

    /// The currencies the game can show money in, in the game's order; the
    /// first is the one the save stores, and `g_currency` indexes this list.
    pub fn currencies(&self) -> &[Currency] {
        &self.currencies
    }

    /// Every plate country, sorted by token.
    pub fn countries(&self) -> &[Named] {
        &self.countries
    }

    /// Display name of one plate country in `language`, falling back to
    /// English.
    pub fn country_name(&self, token: &str, language: &str) -> Option<&str> {
        lookup(&self.countries, token, language)
    }
}

/// A token (`germany`, `volvo.9700_23`, `krone`, `prague`) with its name per
/// language.
#[derive(Debug)]
pub struct Named {
    /// Token as the game stores it.
    pub token: String,
    /// Name per game language, for example `en_us` and `pt_br`.
    pub names: BTreeMap<String, String>,
}

fn lookup<'a>(entries: &'a [Named], token: &str, language: &str) -> Option<&'a str> {
    let index = entries
        .binary_search_by(|entry| entry.token.as_str().cmp(token))
        .ok()?;
    localized(&entries[index].names, language)
}

/// One display currency from `def/economy_data.sii`.
///
/// The game shows an amount as `[sign1] - [sign2] 1.234,- [sign3]`: the minus
/// sits between the first two signs, and the third follows the number.
#[derive(Debug, Clone, PartialEq)]
pub struct Currency {
    /// Code, for example `EUR` or `CZK`.
    pub code: String,
    /// Units of this currency per unit the save stores.
    pub ratio: f64,
    /// Text before the minus sign.
    pub sign1: String,
    /// Text between the minus sign and the number.
    pub sign2: String,
    /// Text after the number.
    pub sign3: String,
}

impl Currency {
    /// `native`, the amount the save stores, in this currency.
    pub fn from_native(&self, native: i64) -> i64 {
        if self.ratio == 1.0 {
            return native;
        }
        (native as f64 * self.ratio).round() as i64
    }

    /// `shown`, an amount in this currency, as the amount the save stores.
    pub fn to_native(&self, shown: i64) -> i64 {
        if self.ratio == 1.0 {
            return shown;
        }
        (shown as f64 / self.ratio).round() as i64
    }
}

/// The player's experience levels from `def/economy_data_xp.sii`.
#[derive(Debug, Default)]
pub struct Levels {
    /// Experience from each level to the next, from level 0; the game keeps
    /// using the last step past the end of the table.
    pub steps: Vec<u32>,
    /// Level titles, sorted by the first level each one names.
    pub titles: Vec<LevelTitle>,
}

/// The title the game shows from one level on.
#[derive(Debug)]
pub struct LevelTitle {
    /// First level with this title.
    pub from: u32,
    /// Title per game language, for example `en_us` and `pt_br`.
    pub names: BTreeMap<String, String>,
}

impl Levels {
    /// The level `experience` points reach; 0 without a table.
    pub fn level(&self, experience: u64) -> u32 {
        let Some(&last) = self.steps.last() else {
            return 0;
        };
        let mut start = 0u64;
        for (level, step) in self.steps.iter().enumerate() {
            let next = start + u64::from(*step);
            if experience < next {
                return level as u32;
            }
            start = next;
        }
        let past = (experience - start) / u64::from(last).max(1);
        u32::try_from(self.steps.len() as u64 + past).unwrap_or(u32::MAX)
    }

    /// The experience points at the start of `level`, or `None` without a
    /// table for any level above 0.
    pub fn experience(&self, level: u32) -> Option<u64> {
        if level == 0 {
            return Some(0);
        }
        let last = u64::from(*self.steps.last()?);
        let level = level as usize;
        let counted = level.min(self.steps.len());
        let table: u64 = self.steps[..counted].iter().copied().map(u64::from).sum();
        Some(table + (level - counted) as u64 * last)
    }

    /// Title of `level` in `language`, falling back to English.
    pub fn title(&self, level: u32, language: &str) -> Option<&str> {
        let names = &self
            .titles
            .iter()
            .rev()
            .find(|title| title.from <= level)?
            .names;
        localized(names, language)
    }
}

/// One `def/vehicle/{truck,bus,car}/<brand>.<model>` folder.
#[derive(Debug)]
pub struct VehicleModel {
    /// Model folder token, for example `renault.etech_t`.
    pub token: String,
    /// Brand and series per game language.
    pub names: BTreeMap<String, String>,
    /// Engines, sorted by `data_path`.
    pub engines: Vec<Engine>,
    /// Transmissions, sorted by `data_path`.
    pub transmissions: Vec<Transmission>,
    /// Cabin definitions.
    pub cabins: Vec<Part>,
    /// Chassis definitions.
    pub chassis: Vec<Part>,
    /// Paint job definitions, with paths only.
    pub paint_jobs: Vec<Part>,
}

/// One engine definition.
#[derive(Debug)]
pub struct Engine {
    /// Name per game language.
    pub names: BTreeMap<String, String>,
    /// Power in horsepower, the first token of the definition's first `info[]`.
    pub power_hp: u32,
    /// Maximum torque in Newton meters.
    pub torque_nm: u32,
    /// `data_path` the save stores for this engine.
    pub path: String,
}

impl Engine {
    /// Name in `language`, else English.
    pub fn name(&self, language: &str) -> Option<&str> {
        localized(&self.names, language)
    }
}

/// One transmission definition.
#[derive(Debug)]
pub struct Transmission {
    /// Name per game language.
    pub names: BTreeMap<String, String>,
    /// Forward gear ratios, in gear order.
    pub ratios: Vec<f32>,
    /// Whether the definition has a retarder.
    pub retarder: bool,
    /// `data_path` the save stores for this transmission.
    pub path: String,
}

impl Transmission {
    /// Name in `language`, else English.
    pub fn name(&self, language: &str) -> Option<&str> {
        localized(&self.names, language)
    }
}

/// A cabin, chassis, or paint job.
#[derive(Debug)]
pub struct Part {
    /// Name per game language.
    pub names: BTreeMap<String, String>,
    /// `data_path` the save stores for this part.
    pub path: String,
}

impl Part {
    /// Name in `language`, else English.
    pub fn name(&self, language: &str) -> Option<&str> {
        localized(&self.names, language)
    }
}

fn localized<'a>(names: &'a BTreeMap<String, String>, language: &str) -> Option<&'a str> {
    names
        .get(language)
        .or_else(|| names.get("en_us"))
        .map(String::as_str)
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
    /// Volume of one unit in cubic meters.
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
    /// Cargo volume in cubic meters.
    pub volume: f32,
    /// Gross weight limit in kilograms.
    pub gross_weight_limit: u64,
    /// Chassis mass in kilograms.
    pub chassis_mass: u64,
    /// Number of axles.
    pub axles: u64,
    /// Trailer length in meters.
    pub length: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn levels() -> Levels {
        let title = |from, name: &str| LevelTitle {
            from,
            names: BTreeMap::from([("en_us".to_owned(), name.to_owned())]),
        };
        Levels {
            steps: vec![200, 500, 700],
            titles: vec![title(0, "Newbie"), title(2, "Enthusiast")],
        }
    }

    #[test]
    fn levels_follow_the_table_then_repeat_the_last_step() {
        let levels = levels();
        assert_eq!(levels.level(0), 0);
        assert_eq!(levels.level(199), 0);
        assert_eq!(levels.level(200), 1);
        assert_eq!(levels.level(1399), 2);
        assert_eq!(levels.level(1400), 3);
        assert_eq!(levels.level(2099), 3);
        assert_eq!(levels.level(2100), 4);
        assert_eq!(levels.experience(0), Some(0));
        assert_eq!(levels.experience(1), Some(200));
        assert_eq!(levels.experience(3), Some(1400));
        assert_eq!(levels.experience(4), Some(2100));
        for level in 0..20 {
            assert_eq!(levels.level(levels.experience(level).unwrap()), level);
        }
    }

    #[test]
    fn titles_pick_the_last_one_reached() {
        let levels = levels();
        assert_eq!(levels.title(0, "en_us"), Some("Newbie"));
        assert_eq!(levels.title(1, "pt_br"), Some("Newbie"));
        assert_eq!(levels.title(2, "en_us"), Some("Enthusiast"));
        assert_eq!(levels.title(90, "en_us"), Some("Enthusiast"));
    }

    #[test]
    fn currencies_convert_both_ways() {
        let currency = |ratio| Currency {
            code: "XYZ".to_owned(),
            ratio,
            sign1: String::new(),
            sign2: String::new(),
            sign3: String::new(),
        };
        let euro = currency(1.0);
        assert_eq!(euro.from_native(i64::MAX), i64::MAX);
        assert_eq!(euro.to_native(-7), -7);
        let koruna = currency(24.4);
        assert_eq!(koruna.from_native(1000), 24_400);
        assert_eq!(koruna.from_native(-1000), -24_400);
        assert_eq!(koruna.to_native(24_400), 1000);
        assert_eq!(koruna.to_native(1000), 41);
    }

    #[test]
    fn no_table_means_level_zero_only() {
        let levels = Levels::default();
        assert_eq!(levels.level(5000), 0);
        assert_eq!(levels.experience(0), Some(0));
        assert_eq!(levels.experience(1), None);
        assert_eq!(levels.title(0, "en_us"), None);
    }
}
