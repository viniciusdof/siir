//! Parsed document shared by the text and binary codecs.

use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Range;

use crate::error::Error;

/// A whole `SiiNunit` document.
pub struct Document<'a> {
    pub(crate) source: &'a str,
    pub(crate) units: Vec<Unit<'a>>,
    pub(crate) closing_brace: usize,
}

/// One `class : id { ... }` block.
pub struct Unit<'a> {
    pub(crate) class: Cow<'a, str>,
    pub(crate) id: Cow<'a, str>,
    pub(crate) attrs: Vec<Attr<'a>>,
    pub(crate) span: Range<usize>,
    pub(crate) inserted: bool,
    pub(crate) added: Vec<(String, Rendered)>,
    pub(crate) removed: bool,
}

pub(crate) struct Attr<'a> {
    pub(crate) name: &'a str,
    pub(crate) index: Index,
    pub(crate) value: &'a str,
    pub(crate) span: Range<usize>,
    pub(crate) replacement: Option<Rendered>,
    pub(crate) removed: bool,
}

/// Where an attribute sits in an implicit array.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Index {
    /// Scalar attribute, `name`.
    None,
    /// Array element without a written index, `name[]`.
    Unspecified,
    /// Array element at a fixed position, `name[i]`.
    At(u32),
}

pub(crate) struct Rendered {
    pub(crate) text: String,
    pub(crate) id: bool,
}

/// A value written into the document.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// Signed integer.
    Int(i64),
    /// Unsigned integer.
    UInt(u64),
    /// Decimal number.
    Float(f32),
    /// Boolean.
    Bool(bool),
    /// Quoted string.
    String(String),
    /// Bare token.
    Token(String),
    /// The `nil` sentinel.
    Nil,
    /// `&hex` float bits.
    Hex(u32),
    /// Parenthesized vector text, parentheses included.
    Vector(String),
}

impl Value {
    /// Signed integer.
    pub fn int64(value: i64) -> Self {
        Self::Int(value)
    }

    /// Quoted string.
    pub fn string(value: impl Into<String>) -> Self {
        Self::String(value.into())
    }

    /// Bare token.
    pub fn token(value: impl Into<String>) -> Self {
        Self::Token(value.into())
    }

    /// Parses one attribute value from its text.
    pub fn parse(text: &str) -> Result<Self, Error> {
        if let Some(inner) = text
            .strip_prefix('"')
            .and_then(|text| text.strip_suffix('"'))
        {
            if inner.contains('"') || inner.contains('\n') || inner.contains('\r') {
                return Err(Error::invalid_value("string has a quote or line break", 0));
            }
            return Ok(Self::String(inner.to_owned()));
        }
        if text == "nil" {
            return Ok(Self::Nil);
        }
        if let Some(hex) = text.strip_prefix('&') {
            let bits =
                u32::from_str_radix(hex, 16).map_err(|_| Error::invalid_value("hex value", 0))?;
            return Ok(Self::Hex(bits));
        }
        if text.starts_with('(') {
            if !is_vector(text) {
                return Err(Error::invalid_value("vector value", 0));
            }
            return Ok(Self::Vector(text.to_owned()));
        }
        if text == "true" || text == "false" {
            return Ok(Self::Bool(text == "true"));
        }
        if let Ok(value) = text.parse::<i64>() {
            return Ok(Self::Int(value));
        }
        if let Ok(value) = text.parse::<u64>() {
            return Ok(Self::UInt(value));
        }
        if let Ok(value) = text.parse::<f32>()
            && value.is_finite()
        {
            return Ok(Self::Float(value));
        }
        if is_token(text) {
            return Ok(Self::Token(text.to_owned()));
        }
        Err(Error::invalid_value("unrecognized value", 0))
    }

    fn render(&self, offset: usize) -> Result<Rendered, Error> {
        let text = match self {
            Self::Int(value) => value.to_string(),
            Self::UInt(value) => value.to_string(),
            Self::Float(value) => {
                if !value.is_finite() {
                    return Err(Error::invalid_value("float is not finite", offset));
                }
                if value.fract() == 0.0 {
                    format!("{value:.1}")
                } else {
                    value.to_string()
                }
            }
            Self::Bool(value) => value.to_string(),
            Self::String(value) => {
                if value.chars().any(|c| matches!(c, '"' | '\n' | '\r')) {
                    return Err(Error::invalid_value(
                        "string has a quote or line break",
                        offset,
                    ));
                }
                format!("\"{value}\"")
            }
            Self::Token(value) => {
                if !is_token(value) {
                    return Err(Error::invalid_value("token has invalid characters", offset));
                }
                value.clone()
            }
            Self::Vector(value) => {
                if !is_vector(value) {
                    return Err(Error::invalid_value(
                        "vector has invalid characters",
                        offset,
                    ));
                }
                value.clone()
            }
            Self::Nil => "nil".to_owned(),
            Self::Hex(bits) => format!("&{bits:08x}"),
        };
        let id = matches!(self, Self::Token(value) if is_id(value));
        Ok(Rendered { text, id })
    }
}

impl<'a> Document<'a> {
    /// Parses a `SiiNunit` document.
    pub fn parse(source: &'a str) -> Result<Self, Error> {
        crate::text::parse(source)
    }

    /// Writes the document, keeping every byte that was not edited.
    pub fn to_text(&self) -> String {
        let mut out = String::with_capacity(self.source.len());
        let mut last = 0;
        for unit in &self.units {
            if unit.inserted {
                continue;
            }
            out.push_str(&self.source[last..unit.span.start]);
            if !unit.removed {
                unit.write_parsed(self.source, &mut out);
            }
            last = unit.span.end;
        }
        out.push_str(&self.source[last..self.closing_brace]);
        for unit in &self.units {
            if unit.inserted && !unit.removed {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str(&unit.to_text());
            }
        }
        out.push_str(&self.source[self.closing_brace..]);
        out
    }

    /// Finds a unit by id, then by class.
    pub fn get(&mut self, name: &str) -> Option<&mut Unit<'a>> {
        let found = self
            .units
            .iter()
            .position(|unit| !unit.removed && unit.id.as_ref() == name)
            .or_else(|| {
                self.units
                    .iter()
                    .position(|unit| !unit.removed && unit.class.as_ref() == name)
            })?;
        self.units.get_mut(found)
    }

    /// Every unit, in file order.
    pub fn units(&self) -> impl Iterator<Item = &Unit<'a>> + '_ {
        self.units.iter().filter(|unit| !unit.removed)
    }

    /// Units of one class, in file order.
    pub fn units_of_class<'b>(&'b self, class: &'b str) -> impl Iterator<Item = &'b Unit<'a>> + 'b {
        self.units()
            .filter(move |unit| unit.class.as_ref() == class)
    }

    /// Adds a unit before the closing brace.
    pub fn insert(&mut self, unit: Unit<'a>) -> Result<(), Error> {
        if !is_word(unit.class.as_ref()) {
            return Err(Error::invalid_name("unit class"));
        }
        if !is_token(unit.id.as_ref()) {
            return Err(Error::invalid_name("unit id"));
        }
        if self
            .units
            .iter()
            .any(|existing| !existing.removed && existing.id.as_ref() == unit.id.as_ref())
        {
            return Err(Error::duplicate_unit(unit.id.as_ref()));
        }
        self.units.push(unit);
        Ok(())
    }

    /// Removes a unit by id.
    pub fn remove(&mut self, id: &str) -> Result<(), Error> {
        match self
            .units
            .iter_mut()
            .find(|unit| !unit.removed && unit.id.as_ref() == id)
        {
            Some(unit) => {
                unit.removed = true;
                Ok(())
            }
            None => Err(Error::missing_unit(id)),
        }
    }

    /// Cited ids that have no unit in this document, without duplicates.
    pub fn dangling(&self) -> Vec<&str> {
        let ids: HashSet<&str> = self
            .units
            .iter()
            .filter(|unit| !unit.removed)
            .map(|unit| unit.id.as_ref())
            .collect();
        let mut seen = HashSet::new();
        let mut missing = Vec::new();
        for unit in self.units.iter().filter(|unit| !unit.removed) {
            for attr in unit.attrs.iter().filter(|attr| !attr.removed) {
                let candidate = match &attr.replacement {
                    Some(rendered) => rendered.id.then_some(rendered.text.as_str()),
                    None => (is_id(attr.value) && !is_number(attr.value)).then_some(attr.value),
                };
                if let Some(id) = candidate
                    && !ids.contains(id)
                    && seen.insert(id)
                {
                    missing.push(id);
                }
            }
            for (_, rendered) in &unit.added {
                let id = rendered.text.as_str();
                if rendered.id && !ids.contains(id) && seen.insert(id) {
                    missing.push(id);
                }
            }
        }
        missing
    }
}

impl<'a> Unit<'a> {
    /// A new unit for [`Document::insert`].
    pub fn new(class: &'a str, id: &'a str) -> Self {
        Self {
            class: Cow::Borrowed(class),
            id: Cow::Borrowed(id),
            attrs: Vec::new(),
            span: 0..0,
            inserted: true,
            added: Vec::new(),
            removed: false,
        }
    }

    /// A new unit that owns its class and id, for [`Document::insert`].
    pub fn new_owned(class: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            class: Cow::Owned(class.into()),
            id: Cow::Owned(id.into()),
            attrs: Vec::new(),
            span: 0..0,
            inserted: true,
            added: Vec::new(),
            removed: false,
        }
    }

    /// Unit class, for example `economy`.
    pub fn class(&self) -> &str {
        self.class.as_ref()
    }

    /// Unit id, named or `_nameless`.
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }

    /// Current text of an attribute, edited or original. Accepts `name[i]`.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.get_nth(name, 0)
    }

    /// Current text of the `nth` element of an implicit array (`name[]`).
    pub fn get_nth(&self, name: &str, nth: usize) -> Option<&str> {
        let (base, index) = split_name(name);
        let parsed = self
            .attrs
            .iter()
            .filter(|attr| !attr.removed && attr.name == base && attr.index == index)
            .map(|attr| {
                attr.replacement
                    .as_ref()
                    .map_or(attr.value, |rendered| rendered.text.as_str())
            });
        let added = self
            .added
            .iter()
            .filter(|(added, _)| {
                let (added_base, added_index) = split_name(added);
                added_base == base && added_index == index
            })
            .map(|(_, rendered)| rendered.text.as_str());
        parsed.chain(added).nth(nth)
    }

    /// Attributes as `(name, index, value)`, with edits applied.
    pub fn attrs(&self) -> impl Iterator<Item = (&str, Index, &str)> + '_ {
        let parsed = self.attrs.iter().filter(|attr| !attr.removed).map(|attr| {
            (
                attr.name,
                attr.index,
                attr.replacement
                    .as_ref()
                    .map_or(attr.value, |rendered| rendered.text.as_str()),
            )
        });
        let added = self.added.iter().map(|(name, rendered)| {
            let (base, index) = split_name(name);
            (base, index, rendered.text.as_str())
        });
        parsed.chain(added)
    }

    /// Writes this unit with its current attributes.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str(self.class());
        out.push_str(" : ");
        out.push_str(self.id());
        out.push_str(" {\n");
        for (name, index, value) in self.attrs() {
            out.push_str("    ");
            out.push_str(&attr_name(name, index));
            out.push_str(": ");
            out.push_str(value);
            out.push('\n');
        }
        out.push_str("}\n");
        out
    }

    /// Removes an attribute, edited or original. Accepts `name[i]`.
    pub fn remove_attr(&mut self, name: &str) -> Result<(), Error> {
        if !is_attr_name(name) {
            return Err(Error::invalid_name("attribute name"));
        }
        let (base, index) = split_name(name);
        if let Some(position) = self
            .attrs
            .iter()
            .position(|attr| !attr.removed && attr.name == base && attr.index == index)
        {
            self.attrs[position].removed = true;
            return Ok(());
        }
        if let Some(position) = self.added.iter().position(|(added, _)| {
            let (added_base, added_index) = split_name(added);
            added_base == base && added_index == index
        }) {
            self.added.remove(position);
            return Ok(());
        }
        let offset = if self.inserted { 0 } else { self.span.start };
        Err(Error::missing_attribute(name, offset))
    }

    /// Adds an attribute, failing when the same name and index already exist.
    pub fn insert_attr(&mut self, name: &str, value: Value) -> Result<(), Error> {
        if !is_attr_name(name) {
            return Err(Error::invalid_name("attribute name"));
        }
        let offset = if self.inserted { 0 } else { self.span.start };
        let (base, index) = split_name(name);
        let exists = self
            .attrs
            .iter()
            .any(|attr| !attr.removed && attr.name == base && attr.index == index)
            || self.added.iter().any(|(added, _)| {
                let (added_base, added_index) = split_name(added);
                added_base == base && added_index == index
            });
        if exists {
            return Err(Error::duplicate_attribute(name, offset));
        }
        let rendered = value.render(offset)?;
        self.added.push((name.to_owned(), rendered));
        Ok(())
    }

    /// Sets `name`, `name[i]`, or `name[]`; adds it on an inserted unit.
    pub fn set(&mut self, name: &str, value: Value) -> Result<(), Error> {
        self.set_nth(name, 0, value)
    }

    /// Sets the `nth` element of an implicit array (`name[]`).
    pub fn set_nth(&mut self, name: &str, nth: usize, value: Value) -> Result<(), Error> {
        if !is_attr_name(name) {
            return Err(Error::invalid_name("attribute name"));
        }
        let offset = if self.inserted { 0 } else { self.span.start };
        let rendered = value.render(offset)?;
        if self.inserted {
            let mut seen = 0;
            for (added, current) in &mut self.added {
                if added == name {
                    if seen == nth {
                        *current = rendered;
                        return Ok(());
                    }
                    seen += 1;
                }
            }
            if seen == nth {
                self.added.push((name.to_owned(), rendered));
                return Ok(());
            }
            return Err(Error::missing_attribute(name, offset));
        }
        let (base, index) = split_name(name);
        if let Some(attr) = self
            .attrs
            .iter_mut()
            .filter(|attr| !attr.removed && attr.name == base && attr.index == index)
            .nth(nth)
        {
            attr.replacement = Some(rendered);
            return Ok(());
        }
        if let Some((_, current)) = self
            .added
            .iter_mut()
            .filter(|(added, _)| {
                let (added_base, added_index) = split_name(added);
                added_base == base && added_index == index
            })
            .nth(nth)
        {
            *current = rendered;
            return Ok(());
        }
        Err(Error::missing_attribute(name, offset))
    }

    fn write_parsed(&self, source: &str, out: &mut String) {
        let mut last = self.span.start;
        for attr in &self.attrs {
            if attr.removed {
                let (start, end) = removal(source, attr.span.clone());
                let start = start.max(last);
                out.push_str(&source[last..start]);
                last = end;
                continue;
            }
            let Some(rendered) = &attr.replacement else {
                continue;
            };
            let value_start = attr.span.end - attr.value.len();
            out.push_str(&source[last..value_start]);
            out.push_str(&rendered.text);
            last = attr.span.end;
        }
        if self.added.is_empty() {
            out.push_str(&source[last..self.span.end]);
            return;
        }
        let brace = self.span.end - 1;
        out.push_str(&source[last..brace]);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        for (name, rendered) in &self.added {
            out.push(' ');
            out.push_str(name);
            out.push_str(": ");
            out.push_str(&rendered.text);
            out.push('\n');
        }
        out.push('}');
    }
}

fn removal(source: &str, span: Range<usize>) -> (usize, usize) {
    let line_start = source[..span.start]
        .rfind('\n')
        .map_or(0, |newline| newline + 1);
    if source[line_start..span.start]
        .bytes()
        .all(|byte| byte == b' ' || byte == b'\t')
    {
        let end = source[span.end..]
            .find('\n')
            .map_or(span.end, |newline| span.end + newline + 1);
        (line_start, end)
    } else {
        (span.start, span.end)
    }
}

fn attr_name(name: &str, index: Index) -> String {
    match index {
        Index::None => name.to_owned(),
        Index::Unspecified => format!("{name}[]"),
        Index::At(at) => format!("{name}[{at}]"),
    }
}

fn split_name(name: &str) -> (&str, Index) {
    let Some(open) = name.rfind('[') else {
        return (name, Index::None);
    };
    if !name.ends_with(']') {
        return (name, Index::None);
    }
    let inside = &name[open + 1..name.len() - 1];
    let index = if inside.is_empty() {
        Index::Unspecified
    } else if let Ok(at) = inside.parse() {
        Index::At(at)
    } else {
        return (name, Index::None);
    };
    (&name[..open], index)
}

fn is_word(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

fn is_attr_name(name: &str) -> bool {
    let (base, index) = split_name(name);
    is_word(base) && (index != Index::None || !name.contains('['))
}

fn is_token(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'@' | b'-'))
}

fn is_vector(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
        return false;
    }
    let mut groups = 0;
    let mut depth = 0;
    for &byte in bytes {
        match byte {
            b'(' => {
                if depth != 0 {
                    return false;
                }
                groups += 1;
                if groups > 2 {
                    return false;
                }
                depth = 1;
            }
            b')' => {
                if depth == 0 {
                    return false;
                }
                depth = 0;
            }
            b' ' | b'\t' | b'.' | b',' | b';' | b'+' | b'-' | b'&' => {}
            _ if byte.is_ascii_alphanumeric() => {}
            _ => return false,
        }
    }
    depth == 0 && groups > 0
}

fn is_number(text: &str) -> bool {
    text.parse::<i64>().is_ok() || text.parse::<u64>().is_ok() || text.parse::<f32>().is_ok()
}

fn is_id(token: &str) -> bool {
    if token.starts_with("_nameless.") {
        return true;
    }
    let mut parts = 0;
    for part in token.split('.') {
        if part.is_empty()
            || part.len() > 12
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        {
            return false;
        }
        parts += 1;
    }
    parts >= 2
}
