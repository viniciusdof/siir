//! BSII binary decoder.
//!
//! Field names, types, and string tables come from the structure blocks in the
//! file itself.

use std::collections::HashMap;

use crate::error::Error;

const T_END: u32 = 0;
const T_UTF8: u32 = 1;
const T_ARR_UTF8: u32 = 2;
const T_ENCODED: u32 = 3;
const T_ARR_ENCODED: u32 = 4;
const T_SINGLE: u32 = 5;
const T_ARR_SINGLE: u32 = 6;
const T_VEC2: u32 = 7;
const T_ARR_VEC2: u32 = 8;
const T_VEC3: u32 = 9;
const T_ARR_VEC3: u32 = 10;
const T_VEC3I: u32 = 17;
const T_ARR_VEC3I: u32 = 18;
const T_VEC4: u32 = 23;
const T_ARR_VEC4: u32 = 24;
const T_VEC8: u32 = 25;
const T_ARR_VEC8: u32 = 26;
const T_INT32: u32 = 37;
const T_ARR_INT32: u32 = 38;
const T_UINT32: u32 = 39;
const T_ARR_UINT32: u32 = 40;
const T_INT16: u32 = 41;
const T_ARR_INT16: u32 = 42;
const T_UINT16: u32 = 43;
const T_ARR_UINT16: u32 = 44;
const T_UINT32_2: u32 = 47;
const T_INT64: u32 = 49;
const T_ARR_INT64: u32 = 50;
const T_UINT64: u32 = 51;
const T_ARR_UINT64: u32 = 52;
const T_BOOL: u32 = 53;
const T_ARR_BOOL: u32 = 54;
const T_ORDINAL: u32 = 55;
const T_ARR_ORDINAL: u32 = 56;
const T_ID: u32 = 57;
const T_ARR_IDA: u32 = 58;
const T_ID2: u32 = 59;
const T_ARR_IDC: u32 = 60;
const T_ID3: u32 = 61;
const T_ARR_IDE: u32 = 62;
const T_VEC2I: u32 = 65;

const CHAR_TABLE: &[u8] = b"0123456789abcdefghijklmnopqrstuvwxyz_";

struct Structure {
    id: u32,
    name: String,
    fields: Vec<Field>,
    /// First ordinal table, kept for experimental type 0x38.
    ordinals: Option<HashMap<u32, String>>,
}

struct Field {
    name: String,
    kind: u32,
    ordinals: Option<HashMap<u32, String>>,
}

/// Decodes a `BSII` payload into `SiiNunit` text.
pub(crate) fn decode(data: &[u8]) -> Result<String, Error> {
    let mut cursor = Cursor { data, pos: 0 };
    cursor.take(4)?;
    let version = cursor.u32()?;
    if !(1..=3).contains(&version) {
        return Err(Error::unsupported_version(version));
    }

    let mut structures: HashMap<u32, Structure> = HashMap::new();
    let mut block_type = cursor.u32()?;
    while block_type == T_END {
        if let Some(structure) = read_structure(&mut cursor)? {
            structures.entry(structure.id).or_insert(structure);
        }
        block_type = cursor.u32()?;
    }

    let mut out = String::from("SiiNunit\n{\n");
    let mut first = true;
    loop {
        if !first {
            if cursor.remaining() == 0 {
                break;
            }
            block_type = cursor.u32()?;
        }
        first = false;

        if block_type == T_END {
            if !cursor.boolean()? {
                break;
            }
            return Err(Error::unknown_structure(0, cursor.pos));
        }

        let structure = structures
            .get(&block_type)
            .ok_or_else(|| Error::unknown_structure(block_type, cursor.pos))?;
        let id = read_id(&mut cursor)?;
        out.push_str(&structure.name);
        out.push_str(" : ");
        out.push_str(&id);
        out.push_str(" {\n");
        for field in &structure.fields {
            // shortcut: 0x38 has no documented table; reuse the structure's
            // first ordinal table until a real file shows otherwise.
            let ordinals = if field.kind == T_ARR_ORDINAL {
                structure.ordinals.as_ref()
            } else {
                field.ordinals.as_ref()
            };
            let values = decode_field(&mut cursor, field.kind, version, ordinals)?;
            write_field(&mut out, &field.name, field.kind, &values);
        }
        out.push_str("}\n\n");
    }
    out.push('}');
    Ok(out)
}

fn read_structure(cursor: &mut Cursor<'_>) -> Result<Option<Structure>, Error> {
    if !cursor.boolean()? {
        return Ok(None);
    }
    let id = cursor.u32()?;
    let name = cursor.string()?;
    let mut fields = Vec::new();
    let mut first_ordinals = None;
    loop {
        let kind = cursor.u32()?;
        if kind == T_END {
            break;
        }
        let name = cursor.string()?;
        let ordinals = if kind == T_ORDINAL {
            let table = read_ordinals(cursor)?;
            if first_ordinals.is_none() {
                first_ordinals = Some(table.clone());
            }
            Some(table)
        } else {
            None
        };
        fields.push(Field {
            name,
            kind,
            ordinals,
        });
    }
    Ok(Some(Structure {
        id,
        name,
        fields,
        ordinals: first_ordinals,
    }))
}

fn read_ordinals(cursor: &mut Cursor<'_>) -> Result<HashMap<u32, String>, Error> {
    let count = cursor.u32()?;
    let mut table = HashMap::new();
    for _ in 0..count {
        let index = cursor.u32()?;
        table.insert(index, cursor.string()?);
    }
    Ok(table)
}

fn read_id(cursor: &mut Cursor<'_>) -> Result<String, Error> {
    let parts = cursor.u8()?;
    if parts == 0xFF {
        return Ok(nameless(cursor.u64()?));
    }
    if parts == 0 {
        return Ok("null".to_owned());
    }
    let mut id = String::new();
    for part in 0..parts {
        if part > 0 {
            id.push('.');
        }
        id.push_str(&encoded(cursor.u64()?));
    }
    Ok(id)
}

fn encoded(mut value: u64) -> String {
    value &= !(1u64 << 63);
    let mut out = Vec::new();
    while value != 0 {
        let index = (value % 38) as i64 - 1;
        value /= 38;
        if (0..CHAR_TABLE.len() as i64).contains(&index) {
            out.push(CHAR_TABLE[index as usize]);
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

fn nameless(address: u64) -> String {
    let data = address.to_le_bytes();
    let mut value = String::new();
    let mut current = String::new();
    for (i, byte) in data.iter().enumerate() {
        if i % 2 == 0 && i > 0 {
            if i >= data.len() - 2 {
                current = current.trim_start_matches('0').to_owned();
            }
            if !current.is_empty() {
                value = format!("{current}.{value}");
            }
            current.clear();
        }
        current = format!("{byte:02x}{current}");
        if i == data.len() - 1 {
            current = current.trim_start_matches('0').to_owned();
            if !current.is_empty() {
                value = format!("{current}.{value}");
            }
            current.clear();
        }
    }
    format!("_nameless.{}", value.trim_end_matches('.'))
}

fn decode_field(
    cursor: &mut Cursor<'_>,
    kind: u32,
    version: u32,
    ordinals: Option<&HashMap<u32, String>>,
) -> Result<Vec<String>, Error> {
    let values = match kind {
        T_UTF8 => vec![utf8_scalar(cursor.string()?)],
        T_ARR_UTF8 => array(cursor, |c| Ok(utf8_array_elem(c.string()?)))?,
        T_ENCODED => vec![ordinal_value(encoded(cursor.u64()?))],
        T_ORDINAL => {
            let index = cursor.u32()?;
            let table = ordinal_table(ordinals, cursor.pos)?;
            let value = table.get(&index).cloned().ok_or_else(|| {
                Error::invalid_value("ordinal value is not in the table", cursor.pos)
            })?;
            vec![ordinal_value(value)]
        }
        T_ARR_ENCODED => array(cursor, |c| Ok(ordinal_value(encoded(c.u64()?))))?,
        T_ARR_ORDINAL => {
            let table = ordinal_table(ordinals, cursor.pos)?;
            let count = cursor.u32()?;
            let mut values = Vec::new();
            for _ in 0..count {
                let index = cursor.u32()?;
                let value = table.get(&index).cloned().ok_or_else(|| {
                    Error::invalid_value("ordinal value is not in the table", cursor.pos)
                })?;
                values.push(ordinal_value(value));
            }
            values
        }
        T_SINGLE => vec![single(cursor.f32()?)],
        T_ARR_SINGLE => array(cursor, |c| Ok(single(c.f32()?)))?,
        T_VEC2 => vec![vec2(cursor)?],
        T_ARR_VEC2 => array(cursor, vec2)?,
        T_VEC3 => vec![vec3(cursor)?],
        T_ARR_VEC3 => array(cursor, vec3)?,
        T_VEC3I => vec![vec3i(cursor)?],
        T_ARR_VEC3I => array(cursor, vec3i)?,
        T_VEC4 => vec![vec4(cursor)?],
        T_ARR_VEC4 => array(cursor, vec4)?,
        T_VEC8 => vec![vec8(cursor, version)?],
        T_ARR_VEC8 => array(cursor, |c| vec8(c, version))?,
        T_INT32 => vec![cursor.i32()?.to_string()],
        T_ARR_INT32 => array(cursor, |c| Ok(c.i32()?.to_string()))?,
        T_UINT32 | T_UINT32_2 => vec![u32_text(cursor.u32()?)],
        T_ARR_UINT32 => array(cursor, |c| Ok(u32_text(c.u32()?)))?,
        T_INT16 => vec![i16_text(cursor.i16()?)],
        T_ARR_INT16 => array(cursor, |c| Ok(c.i16()?.to_string()))?,
        T_UINT16 => vec![u16_text(cursor.u16()?)],
        T_ARR_UINT16 => array(cursor, |c| Ok(c.u16()?.to_string()))?,
        T_INT64 => vec![cursor.i64()?.to_string()],
        T_ARR_INT64 => array(cursor, |c| Ok(c.i64()?.to_string()))?,
        T_UINT64 => vec![u64_text(cursor.u64()?)],
        T_ARR_UINT64 => array(cursor, |c| Ok(c.u64()?.to_string()))?,
        T_BOOL => vec![cursor.boolean()?.to_string()],
        T_ARR_BOOL => array(cursor, |c| Ok(c.boolean()?.to_string()))?,
        T_ID | T_ID2 | T_ID3 => vec![read_id(cursor)?],
        T_ARR_IDA | T_ARR_IDC | T_ARR_IDE => array(cursor, read_id)?,
        T_VEC2I => vec![vec2i(cursor)?],
        _ => return Err(Error::unknown_type(kind, cursor.pos)),
    };
    Ok(values)
}

fn array(
    cursor: &mut Cursor<'_>,
    decode: impl Fn(&mut Cursor<'_>) -> Result<String, Error>,
) -> Result<Vec<String>, Error> {
    let count = cursor.u32()?;
    let mut values = Vec::new();
    for _ in 0..count {
        values.push(decode(cursor)?);
    }
    Ok(values)
}

fn write_field(out: &mut String, name: &str, kind: u32, values: &[String]) {
    if is_array(kind) {
        out.push(' ');
        out.push_str(name);
        out.push_str(": ");
        out.push_str(&values.len().to_string());
        out.push('\n');
        for (index, value) in values.iter().enumerate() {
            out.push(' ');
            out.push_str(name);
            out.push('[');
            out.push_str(&index.to_string());
            out.push_str("]: ");
            out.push_str(value);
            out.push('\n');
        }
    } else if let Some(value) = values.first() {
        out.push(' ');
        out.push_str(name);
        out.push_str(": ");
        out.push_str(value);
        out.push('\n');
    }
}

fn is_array(kind: u32) -> bool {
    matches!(
        kind,
        T_ARR_UTF8
            | T_ARR_ENCODED
            | T_ARR_ORDINAL
            | T_ARR_SINGLE
            | T_ARR_VEC2
            | T_ARR_VEC3
            | T_ARR_VEC3I
            | T_ARR_VEC4
            | T_ARR_VEC8
            | T_ARR_INT32
            | T_ARR_UINT32
            | T_ARR_INT16
            | T_ARR_UINT16
            | T_ARR_INT64
            | T_ARR_UINT64
            | T_ARR_BOOL
            | T_ARR_IDA
            | T_ARR_IDC
            | T_ARR_IDE
    )
}

fn single(value: f32) -> String {
    if (value as i32) as f32 != value || value >= 1e7 {
        format!("&{:08x}", value.to_bits())
    } else {
        (value as i32).to_string()
    }
}

fn vec2(cursor: &mut Cursor<'_>) -> Result<String, Error> {
    Ok(format!(
        "({}, {})",
        single(cursor.f32()?),
        single(cursor.f32()?)
    ))
}

fn vec3(cursor: &mut Cursor<'_>) -> Result<String, Error> {
    Ok(format!(
        "({}, {}, {})",
        single(cursor.f32()?),
        single(cursor.f32()?),
        single(cursor.f32()?)
    ))
}

fn vec4(cursor: &mut Cursor<'_>) -> Result<String, Error> {
    Ok(format!(
        "({}; {}, {}, {})",
        single(cursor.f32()?),
        single(cursor.f32()?),
        single(cursor.f32()?),
        single(cursor.f32()?)
    ))
}

fn vec2i(cursor: &mut Cursor<'_>) -> Result<String, Error> {
    Ok(format!("({}, {})", cursor.i32()?, cursor.i32()?))
}

fn vec3i(cursor: &mut Cursor<'_>) -> Result<String, Error> {
    Ok(format!(
        "({}, {}, {})",
        cursor.i32()?,
        cursor.i32()?,
        cursor.i32()?
    ))
}

fn vec8(cursor: &mut Cursor<'_>, version: u32) -> Result<String, Error> {
    if version == 1 {
        return Ok(format!(
            "({}, {}, {}) ({}; {}, {}, {})",
            single(cursor.f32()?),
            single(cursor.f32()?),
            single(cursor.f32()?),
            single(cursor.f32()?),
            single(cursor.f32()?),
            single(cursor.f32()?),
            single(cursor.f32()?)
        ));
    }
    let mut values = [0.0; 8];
    for value in &mut values {
        *value = cursor.f32()?;
    }
    let bias = values[3] as i64;
    let shift = ((bias & 0xFFF) - 2048) << 9;
    let shift_c = (((bias >> 12) & 0xFFF) - 2048) << 9;
    let a = values[0] + shift as f32;
    let c = values[2] + shift_c as f32;
    Ok(format!(
        "({}, {}, {}) ({}; {}, {}, {})",
        single(a),
        single(values[1]),
        single(c),
        single(values[4]),
        single(values[5]),
        single(values[6]),
        single(values[7])
    ))
}

fn u32_text(value: u32) -> String {
    if value == u32::MAX {
        "nil".to_owned()
    } else {
        value.to_string()
    }
}

fn u64_text(value: u64) -> String {
    if value == u64::MAX {
        "nil".to_owned()
    } else {
        value.to_string()
    }
}

fn u16_text(value: u16) -> String {
    if value == u16::MAX {
        "nil".to_owned()
    } else {
        value.to_string()
    }
}

fn i16_text(value: i16) -> String {
    if value == i16::MAX {
        "nil".to_owned()
    } else {
        value.to_string()
    }
}

fn utf8_scalar(value: String) -> String {
    if value.parse::<i64>().is_ok() {
        return value;
    }
    if value.is_empty() {
        return "\"\"".to_owned();
    }
    if limited(&value) {
        value
    } else {
        format!("\"{value}\"")
    }
}

fn utf8_array_elem(value: String) -> String {
    if value.parse::<u64>().is_ok() {
        return value;
    }
    if value.is_empty() {
        return "\"\"".to_owned();
    }
    if limited(&value) {
        value
    } else {
        format!("\"{value}\"")
    }
}

fn limited(value: &str) -> bool {
    value.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn ordinal_table(
    ordinals: Option<&HashMap<u32, String>>,
    offset: usize,
) -> Result<&HashMap<u32, String>, Error> {
    ordinals.ok_or_else(|| Error::missing_ordinals(offset))
}

fn ordinal_value(value: String) -> String {
    if limited(&value) && !value.is_empty() {
        value
    } else {
        format!("\"{value}\"")
    }
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Cursor<'a> {
    fn remaining(&self) -> usize {
        self.data.len() - self.pos
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], Error> {
        if self.remaining() < len {
            return Err(Error::truncated(len, self.remaining(), self.pos));
        }
        let bytes = &self.data[self.pos..self.pos + len];
        self.pos += len;
        Ok(bytes)
    }

    fn u8(&mut self) -> Result<u8, Error> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, Error> {
        let bytes = self.take(2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    fn u32(&mut self) -> Result<u32, Error> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn u64(&mut self) -> Result<u64, Error> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn i16(&mut self) -> Result<i16, Error> {
        Ok(self.u16()? as i16)
    }

    fn i32(&mut self) -> Result<i32, Error> {
        Ok(self.u32()? as i32)
    }

    fn i64(&mut self) -> Result<i64, Error> {
        Ok(self.u64()? as i64)
    }

    fn f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_bits(self.u32()?))
    }

    fn boolean(&mut self) -> Result<bool, Error> {
        Ok(self.u8()? != 0)
    }

    fn string(&mut self) -> Result<String, Error> {
        let len = self.u32()? as usize;
        let start = self.pos;
        let bytes = self.take(len)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::invalid_encoding(start))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u32(data: &mut Vec<u8>, value: u32) {
        data.extend_from_slice(&value.to_le_bytes());
    }

    fn push_str(data: &mut Vec<u8>, value: &str) {
        push_u32(data, value.len() as u32);
        data.extend_from_slice(value.as_bytes());
    }

    #[test]
    fn encoded_string_ignores_bit_63() {
        assert_eq!(encoded(1 << 63), "");
        assert_eq!(encoded((1 << 63) | 1), "0");
    }

    #[test]
    fn unsupported_version_fails() {
        let error = decode(b"BSII\x04\x00\x00\x00").unwrap_err();
        assert_eq!(error.to_string(), "unsupported BSII version 4");
    }

    #[test]
    fn unknown_structure_fails_at_its_offset() {
        let error = decode(b"BSII\x03\x00\x00\x00\x01\x00\x00\x00").unwrap_err();
        assert_eq!(error.to_string(), "unknown BSII structure 1 at offset 12");
    }

    #[test]
    fn unknown_type_fails_at_its_offset() {
        let mut data = Vec::new();
        data.extend_from_slice(b"BSII");
        data.extend_from_slice(&3u32.to_le_bytes());
        data.extend_from_slice(&0u32.to_le_bytes());
        data.push(1);
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(b"u");
        data.extend_from_slice(&999u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.extend_from_slice(b"x");
        data.extend_from_slice(&0u32.to_le_bytes());
        data.extend_from_slice(&1u32.to_le_bytes());
        data.push(0);

        let error = decode(&data).unwrap_err();
        assert_eq!(
            error.to_string(),
            format!("unknown BSII data type 999 at offset {}", data.len())
        );
    }

    #[test]
    fn ordinal_array_resolves_and_quotes() {
        let mut cursor = Cursor {
            data: &[2, 0, 0, 0, 1, 0, 0, 0, 9, 0, 0, 0],
            pos: 0,
        };
        let mut ordinals = HashMap::new();
        ordinals.insert(1, "spot".to_owned());
        ordinals.insert(9, "not set".to_owned());
        let values = decode_field(&mut cursor, T_ARR_ORDINAL, 3, Some(&ordinals)).unwrap();
        assert_eq!(values, vec!["spot", "\"not set\""]);
    }

    #[test]
    fn ordinal_array_rejects_a_missing_index() {
        let mut cursor = Cursor {
            data: &[2, 0, 0, 0, 1, 0, 0, 0, 9, 0, 0, 0],
            pos: 0,
        };
        let mut ordinals = HashMap::new();
        ordinals.insert(1, "spot".to_owned());
        let error = decode_field(&mut cursor, T_ARR_ORDINAL, 3, Some(&ordinals)).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("ordinal value is not in the table")
        );
    }

    #[test]
    fn ordinal_array_without_table_fails() {
        let mut cursor = Cursor {
            data: &[2, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0],
            pos: 0,
        };
        let error = decode_field(&mut cursor, T_ARR_ORDINAL, 3, None).unwrap_err();
        assert_eq!(error.offset(), 0);
        assert_eq!(error.to_string(), "missing ordinal table at offset 0");
    }

    #[test]
    fn ordinal_array_serializes_indexed() {
        let mut data = Vec::new();
        data.extend_from_slice(b"BSII");
        push_u32(&mut data, 3);
        push_u32(&mut data, 0);
        data.push(1);
        push_u32(&mut data, 1);
        push_str(&mut data, "u");
        push_u32(&mut data, T_ORDINAL);
        push_str(&mut data, "kind");
        push_u32(&mut data, 2);
        push_u32(&mut data, 1);
        push_str(&mut data, "spot");
        push_u32(&mut data, 2);
        push_str(&mut data, "none");
        push_u32(&mut data, T_ARR_ORDINAL);
        push_str(&mut data, "kinds");
        push_u32(&mut data, 0);
        push_u32(&mut data, 1);
        data.push(0);
        push_u32(&mut data, 1);
        push_u32(&mut data, 2);
        push_u32(&mut data, 1);
        push_u32(&mut data, 2);
        push_u32(&mut data, 0);
        data.push(0);

        let text = decode(&data).unwrap();
        assert!(text.contains(" kind: spot\n kinds: 2\n kinds[0]: spot\n kinds[1]: none\n"));
    }

    #[test]
    fn ordinal_tables_are_per_field() {
        let mut data = Vec::new();
        data.extend_from_slice(b"BSII");
        push_u32(&mut data, 3);
        push_u32(&mut data, 0);
        data.push(1);
        push_u32(&mut data, 1);
        push_str(&mut data, "u");
        push_u32(&mut data, T_ORDINAL);
        push_str(&mut data, "first");
        push_u32(&mut data, 2);
        push_u32(&mut data, 0);
        push_str(&mut data, "a");
        push_u32(&mut data, 1);
        push_str(&mut data, "b");
        push_u32(&mut data, T_ORDINAL);
        push_str(&mut data, "second");
        push_u32(&mut data, 2);
        push_u32(&mut data, 0);
        push_str(&mut data, "x");
        push_u32(&mut data, 1);
        push_str(&mut data, "y");
        push_u32(&mut data, 0);
        push_u32(&mut data, 1);
        data.push(0);
        push_u32(&mut data, 1);
        push_u32(&mut data, 1);
        push_u32(&mut data, 0);
        data.push(0);

        let text = decode(&data).unwrap();
        assert!(text.contains(" first: b\n second: y\n"), "{text}");
    }

    #[test]
    fn int_pair_is_written_as_a_vector() {
        let mut data = Vec::new();
        data.extend_from_slice(b"BSII");
        push_u32(&mut data, 3);
        push_u32(&mut data, 0);
        data.push(1);
        push_u32(&mut data, 1);
        push_str(&mut data, "u");
        push_u32(&mut data, T_VEC2I);
        push_str(&mut data, "size");
        push_u32(&mut data, 0);
        push_u32(&mut data, 1);
        data.push(0);
        data.extend_from_slice(&1024i32.to_le_bytes());
        data.extend_from_slice(&1024i32.to_le_bytes());
        push_u32(&mut data, 0);
        data.push(0);

        let text = decode(&data).unwrap();
        assert!(text.contains(" size: (1024, 1024)\n"), "{text}");
    }

    #[test]
    fn invalid_utf8_reports_the_data_offset() {
        let mut cursor = Cursor {
            data: b"\x01\x00\x00\x00\xff",
            pos: 0,
        };
        let error = cursor.string().unwrap_err();
        assert_eq!(error.offset(), 4);
    }

    #[test]
    fn truncated_structure_fails() {
        let error = decode(b"BSII\x03\x00\x00\x00\x00").unwrap_err();
        assert_eq!(error.offset(), 8);
        assert!(error.to_string().starts_with("truncated:"));
    }
}
