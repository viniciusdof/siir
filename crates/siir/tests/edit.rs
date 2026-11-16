//! Public edit surface.

use siir::{Document, Index, Unit, Value};

const SOURCE: &str = include_str!("../testdata/economy.sii");

#[test]
fn set_changes_only_the_value() {
    let mut document = Document::parse(SOURCE).unwrap();
    document
        .get("economy")
        .unwrap()
        .set("money_account", Value::int64(1_000_000))
        .unwrap();

    let expected = include_str!("../testdata/economy_money.sii");
    assert_eq!(document.to_text(), expected);
}

#[test]
fn get_finds_by_class_and_id() {
    let mut document = Document::parse(SOURCE).unwrap();
    assert!(document.get("economy").is_some());
    assert!(document.get("_nameless.0001.0002.0003.0004").is_some());
    assert_eq!(document.units_of_class("economy").count(), 1);
    let unit = document.units_of_class("economy").next().unwrap();
    assert_eq!(unit.class(), "economy");
    assert_eq!(unit.id(), "_nameless.0001.0002.0003.0004");
}

#[test]
fn sets_every_value_form() {
    let mut document = Document::parse(SOURCE).unwrap();
    let mut unit = Unit::new("probe", "probe.one");
    unit.set("int", Value::Int(-7)).unwrap();
    unit.set("uint", Value::UInt(u64::MAX)).unwrap();
    unit.set("float", Value::Float(2.0)).unwrap();
    unit.set("bool", Value::Bool(true)).unwrap();
    unit.set("string", Value::string("a b")).unwrap();
    unit.set("token", Value::token("trailer.two")).unwrap();
    unit.set("nil", Value::Nil).unwrap();
    unit.set("hex", Value::Hex(0x3f80_0000)).unwrap();
    unit.set("vector", Value::Vector("(1, 2, 3)".to_owned()))
        .unwrap();
    document.insert(unit).unwrap();

    let text = document.to_text();
    for line in [
        "int: -7",
        "uint: 18446744073709551615",
        "float: 2.0",
        "bool: true",
        "string: \"a b\"",
        "token: trailer.two",
        "nil: nil",
        "hex: &3f800000",
        "vector: (1, 2, 3)",
    ] {
        assert!(text.contains(line), "missing {line:?} in {text:?}");
    }
}

#[test]
fn set_changes_one_array_element() {
    let source = include_str!("../testdata/gears.sii");
    let mut document = Document::parse(source).unwrap();
    document
        .get("u")
        .unwrap()
        .set("gears[0]", Value::token("3H"))
        .unwrap();

    let text = document.to_text();
    assert!(text.contains("gears[0]: 3H\n"));
    assert!(text.contains("gears[1]: 2L\n"));
    assert_eq!(document.get("u").unwrap().get("gears[0]"), Some("3H"));
}

#[test]
fn set_missing_attribute_fails() {
    let mut document = Document::parse(SOURCE).unwrap();
    let error = document
        .get("economy")
        .unwrap()
        .set("missing", Value::int64(1))
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "missing attribute \"missing\" at offset 11"
    );
}

#[test]
fn insert_adds_a_unit_before_the_closing_brace() {
    let mut document = Document::parse(SOURCE).unwrap();
    let mut trailer = Unit::new("trailer", "trailer.test");
    trailer.set("name", Value::string("box")).unwrap();
    trailer.set("mass", Value::Float(1.5)).unwrap();
    document.insert(trailer).unwrap();

    let text = document.to_text();
    assert!(text.contains("trailer : trailer.test {\n    name: \"box\"\n    mass: 1.5\n}\n"));
    assert!(text.ends_with("}\n}\n"));
    assert_eq!(Document::parse(&text).unwrap().to_text(), text);
}

#[test]
fn remove_drops_the_unit() {
    let mut document = Document::parse(SOURCE).unwrap();
    document.remove("_nameless.0001.0002.0003.0004").unwrap();

    let text = document.to_text();
    assert!(!text.contains("economy"));
    assert!(text.contains("SiiNunit"));
    assert!(document.get("economy").is_none());
    assert_eq!(document.units().count(), 0);
    assert_eq!(document.units_of_class("economy").count(), 0);
    assert!(document.remove("nope").is_err());
}

#[test]
fn dangling_lists_cited_ids_without_a_unit() {
    let source = include_str!("../testdata/dangling.sii");
    let document = Document::parse(source).unwrap();
    assert_eq!(document.dangling(), vec!["vehicle.volvo"]);
}

#[test]
fn dangling_ignores_numbers_and_strings() {
    let source = include_str!("../testdata/dangling_numbers.sii");
    let document = Document::parse(source).unwrap();
    assert!(document.dangling().is_empty());
}

#[test]
fn set_rejects_an_invalid_string() {
    let mut document = Document::parse(SOURCE).unwrap();
    let error = document
        .get("economy")
        .unwrap()
        .set("other", Value::string("a\"b"))
        .unwrap_err();
    assert_eq!(
        error.to_string(),
        "invalid value: string has a quote or line break at offset 11"
    );
}

#[test]
fn insert_rejects_bad_names_and_duplicates() {
    let mut document = Document::parse(SOURCE).unwrap();
    assert_eq!(
        document
            .insert(Unit::new("trailer", "bad id"))
            .unwrap_err()
            .to_string(),
        "invalid name: unit id"
    );
    assert_eq!(
        document
            .insert(Unit::new("bad class", "trailer.one"))
            .unwrap_err()
            .to_string(),
        "invalid name: unit class"
    );
    assert_eq!(
        document
            .insert(Unit::new("trailer", "_nameless.0001.0002.0003.0004"))
            .unwrap_err()
            .to_string(),
        "duplicate unit \"_nameless.0001.0002.0003.0004\""
    );
    document
        .insert(Unit::new("trailer", "trailer.one"))
        .unwrap();
    assert!(
        document
            .insert(Unit::new("trailer", "trailer.one"))
            .is_err()
    );
}

#[test]
fn inserted_attribute_name_is_validated() {
    let mut unit = Unit::new("trailer", "trailer.one");
    assert_eq!(
        unit.set("bad name", Value::Int(1)).unwrap_err().to_string(),
        "invalid name: attribute name"
    );
    assert_eq!(
        unit.set("gears[x]", Value::Int(1)).unwrap_err().to_string(),
        "invalid name: attribute name"
    );
}

#[test]
fn dangling_sees_edits_and_inserted_units() {
    let mut document = Document::parse(SOURCE).unwrap();
    document
        .get("economy")
        .unwrap()
        .set("other", Value::token("vehicle.missing"))
        .unwrap();
    assert_eq!(document.dangling(), vec!["vehicle.missing"]);

    let mut trailer = Unit::new("trailer", "trailer.one");
    trailer
        .set("link", Value::token("trailer.missing"))
        .unwrap();
    document.insert(trailer).unwrap();
    assert_eq!(
        document.dangling(),
        vec!["vehicle.missing", "trailer.missing"]
    );
}

#[test]
fn set_nth_reaches_later_implicit_elements() {
    let source = include_str!("../testdata/entries.sii");
    let mut document = Document::parse(source).unwrap();
    document
        .get("u")
        .unwrap()
        .set_nth("entries[]", 1, Value::string("c"))
        .unwrap();

    let text = document.to_text();
    assert!(text.contains("entries[]: \"a\"\n"));
    assert!(text.contains("entries[]: \"c\"\n"));
    assert_eq!(
        document.get("u").unwrap().get_nth("entries[]", 1),
        Some("\"c\"")
    );
}

#[test]
fn attrs_lists_current_values() {
    let mut document = Document::parse(SOURCE).unwrap();
    let unit = document.get("economy").unwrap();
    unit.set("other", Value::Int(8)).unwrap();
    assert_eq!(
        unit.attrs().collect::<Vec<_>>(),
        vec![
            ("money_account", Index::None, "1000"),
            ("other", Index::None, "8"),
        ]
    );

    let mut probe = Unit::new("probe", "probe.one");
    probe.set("gears[0]", Value::Int(1)).unwrap();
    probe.set("entries[]", Value::string("a")).unwrap();
    assert_eq!(
        probe.attrs().collect::<Vec<_>>(),
        vec![
            ("gears", Index::At(0), "1"),
            ("entries", Index::Unspecified, "\"a\""),
        ]
    );
}

#[test]
fn remove_attr_drops_the_line() {
    let mut document = Document::parse(SOURCE).unwrap();
    let unit = document.get("economy").unwrap();
    unit.remove_attr("money_account").unwrap();
    assert_eq!(unit.get("money_account"), None);
    assert_eq!(
        unit.set("money_account", Value::Int(1))
            .unwrap_err()
            .to_string(),
        "missing attribute \"money_account\" at offset 11"
    );
    let error = unit.remove_attr("money_account").unwrap_err();
    assert_eq!(
        error.to_string(),
        "missing attribute \"money_account\" at offset 11"
    );
    assert_eq!(
        document.to_text(),
        include_str!("../testdata/economy_without_money.sii")
    );
}

#[test]
fn remove_attr_handles_arrays_and_inserted_units() {
    let source = include_str!("../testdata/gears.sii");
    let mut document = Document::parse(source).unwrap();
    document.get("u").unwrap().remove_attr("gears[0]").unwrap();
    let text = document.to_text();
    assert!(!text.contains("gears[0]"));
    assert!(text.contains("gears[1]: 2L"));

    let mut unit = Unit::new("probe", "probe.one");
    unit.set("a", Value::Int(1)).unwrap();
    unit.remove_attr("a").unwrap();
    assert_eq!(unit.get("a"), None);
    assert_eq!(unit.attrs().count(), 0);
}

#[test]
fn float_keeps_a_decimal_point() {
    let mut document = Document::parse(SOURCE).unwrap();
    document
        .get("economy")
        .unwrap()
        .set("other", Value::Float(1.0))
        .unwrap();
    assert!(document.to_text().contains("other: 1.0\n"));
}

#[test]
fn untouched_text_is_byte_identical() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/ets2/text/gearbox_layout_volvo_12.sii");
    let source = std::fs::read_to_string(path).unwrap();
    let document = Document::parse(&source).unwrap();
    assert_eq!(document.to_text(), source);
}
