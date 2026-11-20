//! Accessory export and import.

use siir::{Document, Index, Unit, Value};

use crate::error::Error;
use crate::names;
use crate::value::{array, unquote};

const PARTS: [&str; 6] = [
    "data",
    "engine",
    "transmission",
    "cabin",
    "chassis",
    "paint",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    Data,
    Engine,
    Transmission,
    Cabin,
    Chassis,
    Paint,
}

impl Part {
    pub(crate) fn index(self) -> usize {
        match self {
            Self::Data => 0,
            Self::Engine => 1,
            Self::Transmission => 2,
            Self::Cabin => 3,
            Self::Chassis => 4,
            Self::Paint => 5,
        }
    }

    pub(crate) fn name(self) -> &'static str {
        PARTS[self.index()]
    }
}

/// Exports one vehicle's accessory units as a `SiiNunit` text block.
pub fn export_accessories(document: &Document<'_>, id: &str) -> Result<String, Error> {
    let accessories = array(vehicle(document, id)?, names::ACCESSORIES)?;
    let mut out = String::from("SiiNunit\n{\n");
    for accessory in accessories {
        let unit = document
            .units()
            .find(|unit| unit.id() == accessory)
            .ok_or_else(|| Error::MissingUnit {
                id: accessory.to_owned(),
            })?;
        out.push_str(&unit.to_text());
    }
    out.push_str("}\n");
    Ok(out)
}

/// Replaces one vehicle's accessory list with the units in `block`.
pub fn import_accessories(document: &mut Document<'_>, id: &str, block: &str) -> Result<(), Error> {
    let old: Vec<String> = array(vehicle(document, id)?, names::ACCESSORIES)?
        .into_iter()
        .map(str::to_owned)
        .collect();
    let target = id.to_owned();
    let parsed = Document::parse(block)?;
    let mut next = 1;
    let mut units = Vec::new();
    let mut parts = [false; PARTS.len()];
    for unit in parsed.units() {
        if !is_accessory_class(unit.class()) {
            return Err(Error::InvalidAccessoryClass {
                class: unit.class().to_owned(),
            });
        }
        let new_id = fresh_id(document, &mut next);
        let mut new_unit = Unit::new_owned(unit.class().to_owned(), new_id);
        for (name, index, value) in unit.attrs() {
            new_unit.insert_attr(&indexed_name(name, index), Value::parse(value)?)?;
        }
        if let Some(path) = new_unit.get(names::DATA_PATH)
            && let Some(part) = classify(unquote(path))
        {
            parts[part.index()] = true;
        }
        units.push(new_unit);
    }
    for (nth, found) in parts.iter().enumerate() {
        if !found {
            return Err(Error::MissingAccessoryPart { part: PARTS[nth] });
        }
    }

    let ids: Vec<String> = units.iter().map(|unit| unit.id().to_owned()).collect();
    for unit in units {
        document.insert(unit)?;
    }

    let unit = document
        .get(&target)
        .filter(|unit| unit.class() == names::VEHICLE && unit.id() == target.as_str())
        .ok_or_else(|| Error::MissingUnit { id: target.clone() })?;
    write_accessories(unit, &ids, old.len())?;
    remove_orphans(document, &target, &old);
    Ok(())
}

fn remove_orphans(document: &mut Document<'_>, target: &str, old: &[String]) {
    for id in old {
        let referenced = document.units_of_class(names::VEHICLE).any(|unit| {
            if unit.id() == target {
                return false;
            }
            let Some(count) = unit
                .get(names::ACCESSORIES)
                .and_then(|value| value.parse::<usize>().ok())
            else {
                return false;
            };
            (0..count)
                .any(|nth| unit.get(&names::indexed(names::ACCESSORIES, nth)) == Some(id.as_str()))
        });
        if !referenced {
            let _ = document.remove(id);
        }
    }
}

pub(crate) fn classify(path: &str) -> Option<Part> {
    if path.ends_with(names::DATA_SUFFIX) {
        Some(Part::Data)
    } else if path.contains(names::ENGINE_DIR) {
        Some(Part::Engine)
    } else if path.contains(names::TRANSMISSION_DIR) {
        Some(Part::Transmission)
    } else if path.contains(names::CABIN_DIR) {
        Some(Part::Cabin)
    } else if path.contains(names::CHASSIS_DIR) {
        Some(Part::Chassis)
    } else if path.contains(names::PAINT_JOB_DIR) {
        Some(Part::Paint)
    } else {
        None
    }
}

pub(crate) fn write_accessories(
    unit: &mut Unit<'_>,
    kept: &[String],
    total: usize,
) -> Result<(), Error> {
    unit.set(names::ACCESSORIES, Value::UInt(kept.len() as u64))?;
    for (nth, accessory) in kept.iter().enumerate() {
        let name = names::indexed(names::ACCESSORIES, nth);
        let value = Value::token(accessory.as_str());
        if unit.get(&name).is_some() {
            unit.set(&name, value)?;
        } else {
            unit.insert_attr(&name, value)?;
        }
    }
    for nth in kept.len()..total {
        unit.remove_attr(&names::indexed(names::ACCESSORIES, nth))?;
    }
    Ok(())
}

fn vehicle<'a, 'd>(document: &'a Document<'d>, id: &str) -> Result<&'a Unit<'d>, Error> {
    document
        .units_of_class(names::VEHICLE)
        .find(|unit| unit.id() == id)
        .ok_or_else(|| Error::MissingUnit { id: id.to_owned() })
}

fn fresh_id(document: &Document<'_>, next: &mut usize) -> String {
    loop {
        let id = format!("_nameless.0000.0000.{:04x}", *next);
        *next += 1;
        if !document.units().any(|unit| unit.id() == id) {
            return id;
        }
    }
}

fn is_accessory_class(class: &str) -> bool {
    class.starts_with("vehicle_") && class.ends_with("_accessory")
}

fn indexed_name(name: &str, index: Index) -> String {
    match index {
        Index::None => name.to_owned(),
        Index::Unspecified => format!("{name}[]"),
        Index::At(at) => format!("{name}[{at}]"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = include_str!("../testdata/vehicle.sii");

    #[test]
    fn exports_and_imports_a_full_set() {
        let mut document = Document::parse(SOURCE).unwrap();
        let block = export_accessories(&document, "vehicle.one").unwrap();
        assert!(block.contains("vehicle_accessory : accessory.data"));
        assert!(block.contains("vehicle_accessory : accessory.mirror"));

        import_accessories(&mut document, "vehicle.two", &block).unwrap();
        let target = document
            .units()
            .find(|unit| unit.id() == "vehicle.two")
            .unwrap();
        assert_eq!(target.get(names::ACCESSORIES), Some("9"));
        let first = target.get("accessories[0]").unwrap();
        let first = document.units().find(|unit| unit.id() == first).unwrap();
        assert_eq!(
            first.get(names::DATA_PATH),
            Some("\"/def/vehicle/truck/test.model/data.sii\"")
        );

        let block = export_accessories(&document, "vehicle.two").unwrap();
        assert!(block.contains("vehicle_accessory : _nameless.0000."));
    }

    #[test]
    fn exports_edited_values() {
        let mut document = Document::parse(SOURCE).unwrap();
        document
            .get("accessory.data")
            .unwrap()
            .set(
                names::DATA_PATH,
                Value::string("/def/vehicle/truck/test.model/data2.sii"),
            )
            .unwrap();
        let block = export_accessories(&document, "vehicle.one").unwrap();
        assert!(block.contains("data2.sii"));
        assert!(!block.contains("/def/vehicle/truck/test.model/data.sii"));
    }

    #[test]
    fn a_smaller_set_replaces_the_list() {
        let mut document = Document::parse(SOURCE).unwrap();
        let block = export_accessories(&document, "vehicle.one").unwrap();
        import_accessories(&mut document, "vehicle.two", &block).unwrap();
        let old: Vec<String> = array(
            document
                .units()
                .find(|unit| unit.id() == "vehicle.two")
                .unwrap(),
            names::ACCESSORIES,
        )
        .unwrap()
        .into_iter()
        .map(str::to_owned)
        .collect();

        let smaller = include_str!("../testdata/accessory_set.sii");
        import_accessories(&mut document, "vehicle.two", smaller).unwrap();
        let target = document
            .units()
            .find(|unit| unit.id() == "vehicle.two")
            .unwrap();
        assert_eq!(target.get(names::ACCESSORIES), Some("6"));
        assert_eq!(target.get("accessories[6]"), None);
        for id in old {
            assert!(
                document.units().all(|unit| unit.id() != id),
                "{id} was not removed"
            );
        }
    }

    #[test]
    fn rejects_incomplete_sets() {
        let mut document = Document::parse(SOURCE).unwrap();
        let error =
            import_accessories(&mut document, "vehicle.one", "SiiNunit\n{\n}\n").unwrap_err();
        assert_eq!(error.to_string(), "accessory set is missing the data");
        assert_eq!(
            document.get("vehicle.one").unwrap().get(names::ACCESSORIES),
            Some("9")
        );

        let partial = "SiiNunit\n{\nvehicle_accessory : one\n{\n    data_path: \"/def/vehicle/truck/test.model/data.sii\"\n}\n}\n";
        let error = import_accessories(&mut document, "vehicle.one", partial).unwrap_err();
        assert_eq!(error.to_string(), "accessory set is missing the engine");
    }

    #[test]
    fn rejects_a_non_accessory_class() {
        let mut document = Document::parse(SOURCE).unwrap();
        let block = "SiiNunit\n{\neconomy : economy.one\n{\n}\n}\n";
        let error = import_accessories(&mut document, "vehicle.two", block).unwrap_err();
        assert_eq!(error.to_string(), "invalid accessory class \"economy\"");
        let target = document
            .units()
            .find(|unit| unit.id() == "vehicle.two")
            .unwrap();
        assert_eq!(target.get(names::ACCESSORIES), Some("0"));
    }

    #[test]
    fn classifies_each_part_path() {
        for (path, part) in [
            ("/def/vehicle/truck/x/data.sii", Part::Data),
            ("/def/vehicle/truck/x/engine/e.sii", Part::Engine),
            (
                "/def/vehicle/truck/x/transmission/t.sii",
                Part::Transmission,
            ),
            ("/def/vehicle/truck/x/cabin/c.sii", Part::Cabin),
            ("/def/vehicle/truck/x/chassis/c.sii", Part::Chassis),
            ("/def/vehicle/truck/x/paint_job/p.sii", Part::Paint),
        ] {
            assert_eq!(classify(path), Some(part), "{path}");
        }
        assert_eq!(classify("/def/vehicle/truck/x/accessory/a.sii"), None);
    }
}
