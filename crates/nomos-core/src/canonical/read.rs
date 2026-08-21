//! The strict canonical reader.
//!
//! [`parse_canonical`] accepts bytes only if they are already exactly what
//! [`CanonicalValue::to_canonical_bytes`] would have produced. It works in two
//! steps: parse the JSON, then re-encode the parsed value and compare the
//! bytes. Anything the profile forbids — insignificant whitespace, unsorted or
//! duplicated object keys, redundant leading zeroes, a `\u` escape where a raw
//! UTF-8 character belongs, an escaped solidus, a trailing newline — changes
//! the re-encoded bytes and is refused with `EK0303`.
//!
//! Structural rejections (a floating-point literal, truncated input, invalid
//! UTF-8, trailing content) are refused earlier with `EK0302`.
//!
//! Because the reader compares against the encoder, the two can never drift
//! apart: a change to the encoder that is not also a change to the profile
//! makes every canonical fixture fail.

use std::collections::BTreeMap;

use super::{CanonicalValue, FieldName};
use crate::diagnostic::{Diagnostic, RepairClass, codes};

/// The deepest nesting the reader will accept.
const MAX_DEPTH: usize = 64;

/// Parses canonical bytes.
///
/// # Errors
///
/// Returns `EK0302` when the bytes are not well-formed JSON under the kernel's
/// integer-only number rule, and `EK0303` when they parse but are not in
/// canonical form.
pub fn parse_canonical(bytes: &[u8]) -> Result<CanonicalValue, Diagnostic> {
    let text = std::str::from_utf8(bytes)
        .map_err(|error| malformed(format!("input is not valid UTF-8: {error}")))?;
    let mut parser = Parser {
        bytes: text.as_bytes(),
        position: 0,
    };
    let value = parser.value(0)?;
    if parser.position != parser.bytes.len() {
        return Err(malformed(format!(
            "unexpected trailing bytes at offset {}",
            parser.position
        )));
    }
    let reencoded = value.to_canonical_bytes();
    if reencoded != bytes {
        return Err(Diagnostic::new(
            codes::CANONICAL_NOT_CANONICAL,
            "input parses but is not in canonical form",
        )
        .with_repair(RepairClass::EmitCanonicalBytes));
    }
    Ok(value)
}

/// Whether the bytes are exactly canonical.
#[must_use]
pub fn is_canonical(bytes: &[u8]) -> bool {
    parse_canonical(bytes).is_ok()
}

fn malformed(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(codes::CANONICAL_MALFORMED, message)
        .with_repair(RepairClass::EmitCanonicalBytes)
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn expect(&mut self, byte: u8) -> Result<(), Diagnostic> {
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(malformed(format!(
                "expected `{}` at offset {}",
                byte as char, self.position
            )))
        }
    }

    fn literal(&mut self, word: &[u8]) -> Result<(), Diagnostic> {
        if self.bytes[self.position..].starts_with(word) {
            self.position += word.len();
            Ok(())
        } else {
            Err(malformed(format!(
                "expected `{}` at offset {}",
                String::from_utf8_lossy(word),
                self.position
            )))
        }
    }

    fn value(&mut self, depth: usize) -> Result<CanonicalValue, Diagnostic> {
        if depth > MAX_DEPTH {
            return Err(malformed(format!("nesting deeper than {MAX_DEPTH} levels")));
        }
        match self.peek() {
            Some(b'n') => self.literal(b"null").map(|()| CanonicalValue::Null),
            Some(b't') => self.literal(b"true").map(|()| CanonicalValue::Bool(true)),
            Some(b'f') => self.literal(b"false").map(|()| CanonicalValue::Bool(false)),
            Some(b'"') => self.string().map(CanonicalValue::Text),
            Some(b'[') => self.array(depth),
            Some(b'{') => self.object(depth),
            Some(byte) if byte == b'-' || byte.is_ascii_digit() => self.number(),
            Some(byte) => Err(malformed(format!(
                "unexpected byte `{:#04x}` at offset {}",
                byte, self.position
            ))),
            None => Err(malformed("unexpected end of input")),
        }
    }

    fn array(&mut self, depth: usize) -> Result<CanonicalValue, Diagnostic> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(CanonicalValue::Array(items));
        }
        loop {
            items.push(self.value(depth + 1)?);
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(CanonicalValue::Array(items));
                }
                _ => {
                    return Err(malformed(format!(
                        "unterminated array at offset {}",
                        self.position
                    )));
                }
            }
        }
    }

    fn object(&mut self, depth: usize) -> Result<CanonicalValue, Diagnostic> {
        self.expect(b'{')?;
        let mut fields = BTreeMap::new();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(CanonicalValue::Object(fields));
        }
        loop {
            let key = self.string()?;
            let name = FieldName::new(&key)?;
            self.expect(b':')?;
            let value = self.value(depth + 1)?;
            fields.insert(name, value);
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(CanonicalValue::Object(fields));
                }
                _ => {
                    return Err(malformed(format!(
                        "unterminated object at offset {}",
                        self.position
                    )));
                }
            }
        }
    }

    fn number(&mut self) -> Result<CanonicalValue, Diagnostic> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
            self.position += 1;
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(malformed(format!(
                "floating-point literal at offset {start}; authoritative numbers are integers only"
            )));
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| malformed(format!("invalid number at offset {start}")))?;
        if text.is_empty() || text == "-" {
            return Err(malformed(format!("invalid number at offset {start}")));
        }
        if let Ok(signed) = text.parse::<i64>() {
            return Ok(CanonicalValue::Int(signed));
        }
        text.parse::<u64>().map(CanonicalValue::Uint).map_err(|_| {
            malformed(format!(
                "integer at offset {start} does not fit a signed or unsigned 64-bit value"
            ))
        })
    }

    fn string(&mut self) -> Result<String, Diagnostic> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| malformed("unterminated string"))?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.position += 1;
                    self.escape(&mut out)?;
                }
                control if control < 0x20 => {
                    return Err(malformed(format!(
                        "unescaped control byte `{:#04x}` at offset {}",
                        control, self.position
                    )));
                }
                _ => {
                    let rest = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| malformed("invalid UTF-8 inside string"))?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| malformed("unterminated string"))?;
                    self.position += character.len_utf8();
                    out.push(character);
                }
            }
        }
    }

    fn escape(&mut self, out: &mut String) -> Result<(), Diagnostic> {
        let byte = self
            .peek()
            .ok_or_else(|| malformed("unterminated escape"))?;
        self.position += 1;
        let character = match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{8}',
            b'f' => '\u{c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => return self.unicode_escape(out),
            other => {
                return Err(malformed(format!(
                    "unsupported escape `\\{}` at offset {}",
                    other as char,
                    self.position - 1
                )));
            }
        };
        out.push(character);
        Ok(())
    }

    fn unicode_escape(&mut self, out: &mut String) -> Result<(), Diagnostic> {
        let start = self.position;
        let digits = self
            .bytes
            .get(start..start + 4)
            .ok_or_else(|| malformed("truncated `\\u` escape"))?;
        let text = std::str::from_utf8(digits)
            .map_err(|_| malformed(format!("invalid `\\u` escape at offset {start}")))?;
        let code = u32::from_str_radix(text, 16)
            .map_err(|_| malformed(format!("invalid `\\u` escape at offset {start}")))?;
        let character = char::from_u32(code).ok_or_else(|| {
            malformed(format!(
                "`\\u{text}` at offset {start} is not a scalar value; surrogate pairs are not supported"
            ))
        })?;
        self.position = start + 4;
        out.push(character);
        Ok(())
    }
}
