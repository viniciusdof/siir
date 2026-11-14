//! `SiiNunit` text parser.

use crate::error::Error;
use crate::model::{Attr, Document, Index, Kind, RawValue, Unit};

pub(crate) fn parse(source: &str) -> Result<Document<'_>, Error> {
    let mut parser = Parser { source, pos: 0 };
    parser.skip_trivia()?;
    let header_start = parser.pos;
    if parser.word("`SiiNunit`")? != "SiiNunit" {
        return Err(Error::parse(header_start, "`SiiNunit`"));
    }
    parser.skip_trivia()?;
    parser.expect(b'{', "`{`")?;

    let mut units = Vec::new();
    let closing_brace;
    loop {
        parser.skip_trivia()?;
        match parser.peek() {
            Some(b'}') => {
                closing_brace = parser.pos;
                parser.pos += 1;
                break;
            }
            Some(_) => units.push(parser.unit()?),
            None => return Err(parser.error("unit or `}`")),
        }
    }

    parser.skip_trivia()?;
    if parser.pos != source.len() {
        return Err(parser.error("end of file"));
    }
    Ok(Document {
        source,
        units,
        closing_brace,
    })
}

struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<u8> {
        self.source.as_bytes().get(self.pos).copied()
    }

    fn error(&self, expected: &'static str) -> Error {
        Error::parse(self.pos, expected)
    }

    fn skip_trivia(&mut self) -> Result<(), Error> {
        loop {
            while self.peek().is_some_and(|b| b.is_ascii_whitespace()) {
                self.pos += 1;
            }
            let rest = &self.source[self.pos..];
            if rest.starts_with("//") || rest.starts_with('#') {
                match rest.find('\n') {
                    Some(end) => self.pos += end + 1,
                    None => self.pos = self.source.len(),
                }
            } else if rest.starts_with("/*") {
                let Some(end) = rest.find("*/") else {
                    return Err(self.error("`*/`"));
                };
                self.pos += end + 2;
            } else {
                return Ok(());
            }
        }
    }

    fn expect(&mut self, byte: u8, expected: &'static str) -> Result<(), Error> {
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(expected))
        }
    }

    fn word(&mut self, expected: &'static str) -> Result<&'a str, Error> {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || b == b'_')
        {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.error(expected));
        }
        Ok(&self.source[start..self.pos])
    }

    fn token(&mut self) -> Result<&'a str, Error> {
        let start = self.pos;
        while self
            .peek()
            .is_some_and(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'.' | b'@' | b'-'))
        {
            self.pos += 1;
        }
        if self.pos == start {
            return Err(self.error("value"));
        }
        Ok(&self.source[start..self.pos])
    }

    fn unit(&mut self) -> Result<Unit<'a>, Error> {
        let start = self.pos;
        let class = self.word("unit class")?;
        self.skip_trivia()?;
        self.expect(b':', "`:`")?;
        self.skip_trivia()?;
        let id = self.token()?;
        self.skip_trivia()?;
        self.expect(b'{', "`{`")?;

        let mut attrs = Vec::new();
        loop {
            self.skip_trivia()?;
            match self.peek() {
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                Some(_) => attrs.push(self.attr()?),
                None => return Err(self.error("attribute or `}`")),
            }
        }

        Ok(Unit {
            class,
            id,
            attrs,
            span: start..self.pos,
            inserted: false,
            added: Vec::new(),
            removed: false,
        })
    }

    fn attr(&mut self) -> Result<Attr<'a>, Error> {
        let start = self.pos;
        let name = self.word("attribute name")?;

        let index = if self.peek() == Some(b'[') {
            self.pos += 1;
            let digits_start = self.pos;
            while self.peek().is_some_and(|b| b.is_ascii_digit()) {
                self.pos += 1;
            }
            let index = if self.pos == digits_start {
                Index::Unspecified
            } else {
                Index::At(
                    self.source[digits_start..self.pos]
                        .parse()
                        .map_err(|_| self.error("array index"))?,
                )
            };
            self.expect(b']', "`]`")?;
            index
        } else {
            Index::None
        };

        self.skip_trivia()?;
        self.expect(b':', "`:`")?;
        self.skip_trivia()?;
        let value = self.value()?;

        Ok(Attr {
            name,
            index,
            value,
            span: start..self.pos,
            replacement: None,
        })
    }

    /// No nesting: values are flat groups.
    fn group(&mut self) -> Result<(), Error> {
        self.expect(b'(', "`(`")?;
        while let Some(byte) = self.peek() {
            self.pos += 1;
            if byte == b')' {
                return Ok(());
            }
        }
        Err(self.error("`)`"))
    }

    fn value(&mut self) -> Result<RawValue<'a>, Error> {
        let start = self.pos;
        let kind = match self.peek() {
            None => return Err(self.error("value")),
            Some(b'"') => {
                self.pos += 1;
                let content_start = self.pos;
                while self.peek().is_some_and(|b| b != b'"') {
                    self.pos += 1;
                }
                if self.peek().is_none() {
                    return Err(self.error("closing quote"));
                }
                let content = &self.source[content_start..self.pos];
                self.pos += 1;
                Kind::String(content)
            }
            Some(b'&') => {
                self.pos += 1;
                let digits_start = self.pos;
                while self.peek().is_some_and(|b| b.is_ascii_hexdigit()) {
                    self.pos += 1;
                }
                let digits = &self.source[digits_start..self.pos];
                let bits =
                    u32::from_str_radix(digits, 16).map_err(|_| self.error("32-bit hex value"))?;
                Kind::Hex(bits)
            }
            Some(b'(') => {
                self.group()?;
                let after_first = self.pos;
                while self.peek().is_some_and(|b| b == b' ' || b == b'\t') {
                    self.pos += 1;
                }
                if self.peek() == Some(b'(') {
                    self.group()?;
                } else {
                    self.pos = after_first;
                }
                Kind::Vector(&self.source[start..self.pos])
            }
            Some(_) => match self.token()? {
                "nil" => Kind::Nil,
                "true" => Kind::Bool(true),
                "false" => Kind::Bool(false),
                token => number_or_token(token),
            },
        };

        Ok(RawValue {
            kind,
            raw: &self.source[start..self.pos],
        })
    }
}

/// `1H` and `RHH` are tokens; `-5` and `1.5` are numbers.
fn number_or_token(token: &str) -> Kind<'_> {
    if !token
        .bytes()
        .next()
        .is_some_and(|b| b.is_ascii_digit() || b == b'-')
    {
        return Kind::Token(token);
    }
    if let Ok(int) = token.parse::<i64>() {
        Kind::Int(int)
    } else if let Ok(uint) = token.parse::<u64>() {
        Kind::UInt(uint)
    } else if let Ok(float) = token.parse::<f32>() {
        Kind::Float(float)
    } else {
        Kind::Token(token)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    const FIXTURES: &str = "fixtures/ets2/text";

    fn attr<'a>(unit: &'a Unit<'a>, name: &str, index: Index) -> &'a Attr<'a> {
        unit.attrs
            .iter()
            .find(|attr| attr.name == name && attr.index == index)
            .unwrap_or_else(|| panic!("missing attribute {name}[{index:?}]"))
    }

    fn error(source: &str) -> Error {
        match parse(source) {
            Ok(_) => panic!("expected a parse error"),
            Err(error) => error,
        }
    }

    #[test]
    fn fixtures_round_trip_unchanged() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURES);
        let mut parsed = 0;
        for entry in fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("sii") {
                continue;
            }
            let source = fs::read_to_string(&path).unwrap();
            let document =
                parse(&source).unwrap_or_else(|error| panic!("{}: {error:?}", path.display()));
            assert_eq!(document.to_text(), source, "{}", path.display());
            assert!(!document.units.is_empty(), "{}", path.display());
            for unit in &document.units {
                assert!(!unit.attrs.is_empty(), "{}: {}", path.display(), unit.class);
            }
            if path.file_name().and_then(|name| name.to_str()) == Some("controls_osx.sii") {
                let unit = &document.units[0];
                assert_eq!(unit.class, "input_config");
                assert!(matches!(
                    attr(unit, "config_lines", Index::At(0)).value.kind,
                    Kind::String(line) if line.starts_with("device keyboard")
                ));
            }
            parsed += 1;
        }
        assert!(parsed > 0);
    }

    #[test]
    fn parses_every_value_form() {
        let source = concat!(
            "SiiNunit\n{\n",
            "economy : _nameless.0001.0002.0003.0004 {\n",
            "    money_account: 1000000\n",
            "    cargo: 18446744073709551615\n",
            "    position: (1.0, 2.0, 3.0)\n",
            "    placement: (0, 0, 0) (1; 0, 0, 0)\n",
            "    entries[]: \"a\"\n",
            "    entries[]: \"b\"\n",
            "    ratio: 1.5\n",
            "    label: \"hello\"\n",
            "    active: true\n",
            "    unused: nil\n",
            "    bits: &3f800000\n",
            "    target: trailer.foo\n",
            "    gears: 2\n",
            "    gears[0]: 1H\n",
            "    gears[1]: _nameless.0001\n",
            "}\n}\n",
        );
        let document = parse(source).unwrap();
        let unit = &document.units[0];
        assert_eq!(unit.class, "economy");
        assert_eq!(unit.id, "_nameless.0001.0002.0003.0004");
        assert_eq!(unit.span.start, source.find("economy").unwrap());
        assert!(source[unit.span.clone()].ends_with('}'));

        assert!(matches!(
            attr(unit, "money_account", Index::None).value.kind,
            Kind::Int(1_000_000)
        ));
        let money = attr(unit, "money_account", Index::None);
        assert_eq!(money.value.raw, "1000000");
        assert_eq!(&source[money.span.clone()], "money_account: 1000000");
        assert!(matches!(
            attr(unit, "cargo", Index::None).value.kind,
            Kind::UInt(u64::MAX)
        ));
        assert!(matches!(
            attr(unit, "position", Index::None).value.kind,
            Kind::Vector("(1.0, 2.0, 3.0)")
        ));
        assert!(matches!(
            attr(unit, "placement", Index::None).value.kind,
            Kind::Vector("(0, 0, 0) (1; 0, 0, 0)")
        ));
        assert!(matches!(
            attr(unit, "entries", Index::Unspecified).value.kind,
            Kind::String("a")
        ));
        assert!(
            matches!(attr(unit, "ratio", Index::None).value.kind, Kind::Float(value) if value == 1.5)
        );
        assert!(matches!(
            attr(unit, "label", Index::None).value.kind,
            Kind::String("hello")
        ));
        assert!(matches!(
            attr(unit, "active", Index::None).value.kind,
            Kind::Bool(true)
        ));
        assert!(matches!(
            attr(unit, "unused", Index::None).value.kind,
            Kind::Nil
        ));
        assert!(matches!(
            attr(unit, "bits", Index::None).value.kind,
            Kind::Hex(0x3f80_0000)
        ));
        assert!(matches!(
            attr(unit, "target", Index::None).value.kind,
            Kind::Token("trailer.foo")
        ));
        assert!(matches!(
            attr(unit, "gears", Index::None).value.kind,
            Kind::Int(2)
        ));
        assert!(matches!(
            attr(unit, "gears", Index::At(0)).value.kind,
            Kind::Token("1H")
        ));
        assert!(matches!(
            attr(unit, "gears", Index::At(1)).value.kind,
            Kind::Token("_nameless.0001")
        ));
    }

    #[test]
    fn skips_comments() {
        let source = "SiiNunit\n{\n// line\n# hash\n/* block */\nunit : id { a: 1 } // tail\n}\n";
        let document = parse(source).unwrap();
        assert!(matches!(
            document.units[0].attrs[0].value.kind,
            Kind::Int(1)
        ));
    }

    #[test]
    fn missing_header_reports_offset_zero() {
        let error = error("nope");
        assert_eq!(error.offset(), 0);
        assert_eq!(error.to_string(), "expected `SiiNunit` at offset 0");
    }

    #[test]
    fn missing_closing_quote_reports_end() {
        let source = "SiiNunit { u : i { s: \"open }";
        let error = error(source);
        assert_eq!(error.offset(), source.len());
        assert_eq!(
            error.to_string(),
            format!("expected closing quote at offset {}", source.len())
        );
    }

    #[test]
    fn trailing_garbage_is_rejected() {
        let source = "SiiNunit { u : i { } } extra";
        let error = error(source);
        let offset = source.find("extra").unwrap();
        assert_eq!(error.offset(), offset);
        assert_eq!(
            error.to_string(),
            format!("expected end of file at offset {offset}")
        );
    }
}
