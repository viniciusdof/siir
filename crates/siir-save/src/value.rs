//! Decoding of the value forms a save uses.

use siir::Unit;

use crate::error::Error;
use crate::names::indexed;

pub(crate) fn text<'a>(unit: &'a Unit<'_>, name: &str) -> Result<&'a str, Error> {
    unit.get(name)
        .ok_or_else(|| Error::missing(unit.class(), name))
}

pub(crate) fn string<'a>(unit: &'a Unit<'_>, name: &str) -> Result<&'a str, Error> {
    Ok(unquote(text(unit, name)?))
}

pub(crate) fn reference<'a>(unit: &'a Unit<'_>, name: &str) -> Option<&'a str> {
    match unit.get(name) {
        None | Some("null" | "nil") => None,
        Some(id) => Some(id),
    }
}

pub(crate) fn integer(unit: &Unit<'_>, name: &str) -> Result<i64, Error> {
    let value = text(unit, name)?;
    value
        .parse()
        .map_err(|_| Error::invalid(unit.class(), name, value))
}

pub(crate) fn count(unit: &Unit<'_>, name: &str) -> Result<u64, Error> {
    let value = text(unit, name)?;
    value
        .parse()
        .map_err(|_| Error::invalid(unit.class(), name, value))
}

pub(crate) fn float(unit: &Unit<'_>, name: &str) -> Result<f32, Error> {
    let value = text(unit, name)?;
    match value.strip_prefix('&') {
        Some(hex) => u32::from_str_radix(hex, 16)
            .map(f32::from_bits)
            .map_err(|_| Error::invalid(unit.class(), name, value)),
        None => value
            .parse()
            .map_err(|_| Error::invalid(unit.class(), name, value)),
    }
}

pub(crate) fn array<'a>(unit: &'a Unit<'_>, name: &str) -> Result<Vec<&'a str>, Error> {
    let count = count(unit, name)? as usize;
    let mut values = Vec::with_capacity(count);
    for nth in 0..count {
        let value = unit
            .get(&indexed(name, nth))
            .ok_or_else(|| Error::missing(unit.class(), name))?;
        values.push(value);
    }
    Ok(values)
}

pub(crate) fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
}
