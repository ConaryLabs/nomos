//! A small JSON reader for the one input that is not canonical bytes.
//!
//! Every kernel document this compiler reads — the entity catalog, the
//! effective-fact documents, the run bundle, the four projections — is already
//! canonical, and [`crate::read`] hands those to `nomos-core`'s strict reader.
//! `area.json` is not: it is hand-authored, pretty-printed, camelCase, and
//! carries decimal presentation values that `CanonicalValue` has no variant
//! for. `RUNTIME.md` section 5 R1-3 replaces it with a typed, versioned source;
//! until then it needs a reader of its own.
//!
//! This one is deliberately strict for a hand-authored file: duplicate keys are
//! refused rather than resolved by last-write-wins, numbers must fit
//! [`Decimal`]'s exact profile, and nesting is bounded. There is no writer —
//! output goes through [`crate::doc`].

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use crate::decimal::Decimal;
use crate::error::{PlanError, PlanResult, codes};

/// The deepest nesting the reader accepts, matching `nomos-core`'s canonical
/// reader.
const MAX_DEPTH: usize = 64;

/// A parsed JSON value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Json {
    /// JSON `null`.
    Null,
    /// JSON `true` or `false`.
    Bool(bool),
    /// An exact decimal.
    Number(Decimal),
    /// A UTF-8 string.
    Text(String),
    /// An array in document order.
    Array(Vec<Json>),
    /// An object, held key-sorted; duplicate keys are refused on read.
    Object(BTreeMap<String, Json>),
}

impl Json {
    /// The object fields, or `None`.
    #[must_use]
    pub const fn as_object(&self) -> Option<&BTreeMap<String, Json>> {
        match self {
            Self::Object(fields) => Some(fields),
            _ => None,
        }
    }

    /// The array items, or `None`.
    #[must_use]
    pub fn as_array(&self) -> Option<&[Json]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// The string, or `None`.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The number, or `None`.
    #[must_use]
    pub const fn as_number(&self) -> Option<&Decimal> {
        match self {
            Self::Number(value) => Some(value),
            _ => None,
        }
    }

    /// The boolean, or `None`.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(*value),
            _ => None,
        }
    }

    /// One object field, or `None` when the value is not an object or the
    /// field is absent.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Json> {
        self.as_object().and_then(|fields| fields.get(name))
    }
}

/// Parses JSON bytes.
///
/// # Errors
///
/// Returns `RP0103` when the bytes are not valid UTF-8, are not well-formed
/// JSON, nest deeper than 64, repeat an object key, or carry trailing content;
/// returns `RP0205` when a number is outside [`Decimal`]'s profile.
pub fn parse(bytes: &[u8]) -> PlanResult<Json> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| malformed(format!("input is not valid UTF-8: {error}")))?;
    let mut parser = Parser {
        bytes: text.as_bytes(),
        position: 0,
    };
    parser.skip_whitespace();
    let value = parser.value(0)?;
    parser.skip_whitespace();
    if parser.position != parser.bytes.len() {
        return Err(malformed(format!(
            "unexpected trailing bytes at offset {}",
            parser.position
        )));
    }
    Ok(value)
}

fn malformed(message: impl Into<String>) -> PlanError {
    PlanError::new(codes::INPUT_MALFORMED, message)
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    fn expect(&mut self, byte: u8) -> PlanResult<()> {
        if self.peek() == Some(byte) {
            self.position += 1;
            return Ok(());
        }
        Err(malformed(format!(
            "expected `{}` at offset {}",
            byte as char, self.position
        )))
    }

    fn literal(&mut self, text: &str) -> PlanResult<()> {
        if self.bytes[self.position..].starts_with(text.as_bytes()) {
            self.position += text.len();
            return Ok(());
        }
        Err(malformed(format!(
            "expected `{text}` at offset {}",
            self.position
        )))
    }

    fn value(&mut self, depth: usize) -> PlanResult<Json> {
        if depth > MAX_DEPTH {
            return Err(malformed(format!("input nests deeper than {MAX_DEPTH}")));
        }
        match self.peek() {
            Some(b'n') => {
                self.literal("null")?;
                Ok(Json::Null)
            }
            Some(b't') => {
                self.literal("true")?;
                Ok(Json::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(Json::Bool(false))
            }
            Some(b'"') => Ok(Json::Text(self.string()?)),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(byte) if byte == b'-' || byte.is_ascii_digit() => self.number(),
            Some(byte) => Err(malformed(format!(
                "unexpected byte `{}` at offset {}",
                byte as char, self.position
            ))),
            None => Err(malformed("input ends before a value")),
        }
    }

    fn array(&mut self, depth: usize) -> PlanResult<Json> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Json::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Json::Array(items));
                }
                _ => {
                    return Err(malformed(format!(
                        "expected `,` or `]` at offset {}",
                        self.position
                    )));
                }
            }
        }
    }

    fn object(&mut self, depth: usize) -> PlanResult<Json> {
        self.expect(b'{')?;
        let mut fields = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Json::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let name = self.string()?;
            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.value(depth + 1)?;
            match fields.entry(name) {
                Entry::Vacant(slot) => {
                    slot.insert(value);
                }
                Entry::Occupied(slot) => {
                    return Err(malformed(format!(
                        "object field `{}` occurs more than once",
                        slot.key()
                    )));
                }
            }
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Json::Object(fields));
                }
                _ => {
                    return Err(malformed(format!(
                        "expected `,` or `}}` at offset {}",
                        self.position
                    )));
                }
            }
        }
    }

    fn number(&mut self) -> PlanResult<Json> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
            self.position += 1;
        }
        if self.peek() == Some(b'.') {
            self.position += 1;
            while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
                self.position += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            return Err(PlanError::new(
                codes::NUMBER_UNSUPPORTED,
                format!("number at offset {start} carries an exponent"),
            ));
        }
        let lexeme = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| malformed(format!("number at offset {start} is not UTF-8")))?;
        Ok(Json::Number(Decimal::parse(lexeme)?))
    }

    fn string(&mut self) -> PlanResult<String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| malformed("input ends inside a string"))?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.position += 1;
                    let escape = self
                        .peek()
                        .ok_or_else(|| malformed("input ends inside an escape"))?;
                    self.position += 1;
                    match escape {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => out.push(self.unicode_escape()?),
                        other => {
                            return Err(malformed(format!(
                                "unsupported string escape `\\{}`",
                                other as char
                            )));
                        }
                    }
                }
                control if control < 0x20 => {
                    return Err(malformed(format!(
                        "unescaped control byte {control:#04x} inside a string"
                    )));
                }
                _ => {
                    let rest = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| malformed("string is not valid UTF-8"))?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| malformed("input ends inside a string"))?;
                    self.position += character.len_utf8();
                    out.push(character);
                }
            }
        }
    }

    fn unicode_escape(&mut self) -> PlanResult<char> {
        let high = self.hex4()?;
        let code = if (0xd800..0xdc00).contains(&high) {
            self.literal("\\u")?;
            let low = self.hex4()?;
            if !(0xdc00..0xe000).contains(&low) {
                return Err(malformed("unpaired UTF-16 high surrogate"));
            }
            0x10000 + ((high - 0xd800) << 10) + (low - 0xdc00)
        } else {
            high
        };
        char::from_u32(code)
            .ok_or_else(|| malformed(format!("escape {code:#x} is not a character")))
    }

    fn hex4(&mut self) -> PlanResult<u32> {
        let end = self.position + 4;
        let digits = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| malformed("input ends inside a `\\u` escape"))?;
        let mut value = 0_u32;
        for digit in digits {
            let nibble = match digit {
                b'0'..=b'9' => u32::from(digit - b'0'),
                b'a'..=b'f' => u32::from(digit - b'a') + 10,
                b'A'..=b'F' => u32::from(digit - b'A') + 10,
                _ => return Err(malformed("`\\u` escape is not four hex digits")),
            };
            value = value * 16 + nibble;
        }
        self.position = end;
        Ok(value)
    }
}
