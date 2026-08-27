//! Canonical JSON parsing with R2-specific error classification.

use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

use nomos_core::{CanonicalValue, FieldName, RepairClass};

use crate::diagnostic::{ObservedError, ObservedResult, codes};

const MAX_DEPTH: usize = 64;

/// Parses bytes and requires the exact `nomos-core` canonical profile.
pub fn parse(bytes: &[u8]) -> ObservedResult<CanonicalValue> {
    let text = std::str::from_utf8(bytes).map_err(|error| {
        ObservedError::new(
            codes::INPUT_MALFORMED,
            format!("input is not valid UTF-8: {error}"),
        )
        .with_repair(RepairClass::EmitCanonicalBytes)
    })?;
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
    if value.to_canonical_bytes() != bytes {
        return Err(ObservedError::new(
            codes::INPUT_NOT_CANONICAL,
            "input parses but is not in canonical form",
        )
        .with_repair(RepairClass::EmitCanonicalBytes));
    }
    Ok(value)
}

fn malformed(message: impl Into<String>) -> ObservedError {
    ObservedError::new(codes::INPUT_MALFORMED, message).with_repair(RepairClass::EmitCanonicalBytes)
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

    fn expect(&mut self, expected: u8) -> ObservedResult<()> {
        if self.peek() == Some(expected) {
            self.position += 1;
            return Ok(());
        }
        Err(malformed(format!(
            "expected `{}` at offset {}",
            expected as char, self.position
        )))
    }

    fn literal(&mut self, expected: &[u8]) -> ObservedResult<()> {
        if self.bytes[self.position..].starts_with(expected) {
            self.position += expected.len();
            return Ok(());
        }
        Err(malformed(format!(
            "expected `{}` at offset {}",
            String::from_utf8_lossy(expected),
            self.position
        )))
    }

    fn value(&mut self, depth: usize) -> ObservedResult<CanonicalValue> {
        if depth > MAX_DEPTH {
            return Err(malformed(format!("nesting exceeds {MAX_DEPTH} levels")));
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
                "unexpected byte {byte:#04x} at offset {}",
                self.position
            ))),
            None => Err(malformed("unexpected end of input")),
        }
    }

    fn array(&mut self, depth: usize) -> ObservedResult<CanonicalValue> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(CanonicalValue::Array(items));
        }
        loop {
            self.skip_whitespace();
            items.push(self.value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
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

    fn object(&mut self, depth: usize) -> ObservedResult<CanonicalValue> {
        self.expect(b'{')?;
        let mut fields = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(CanonicalValue::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.string()?;
            let name = FieldName::new(&key).map_err(|_| {
                ObservedError::new(
                    codes::INPUT_NOT_CANONICAL,
                    format!("object key `{key}` is outside the canonical field grammar"),
                )
                .with_repair(RepairClass::EmitCanonicalBytes)
            })?;
            self.skip_whitespace();
            self.expect(b':')?;
            self.skip_whitespace();
            let value = self.value(depth + 1)?;
            match fields.entry(name) {
                Entry::Vacant(slot) => {
                    slot.insert(value);
                }
                Entry::Occupied(slot) => {
                    return Err(ObservedError::new(
                        codes::FIELD_INVALID,
                        format!("field `{}` occurs more than once", slot.key()),
                    )
                    .with_repair(RepairClass::RemoveDuplicateDeclaration));
                }
            }
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => {
                    self.position += 1;
                    self.skip_whitespace();
                }
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

    fn number(&mut self) -> ObservedResult<CanonicalValue> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }
        while matches!(self.peek(), Some(byte) if byte.is_ascii_digit()) {
            self.position += 1;
        }
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(malformed(format!(
                "floating-point literal at offset {start}"
            )));
        }
        let text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| malformed(format!("invalid number at offset {start}")))?;
        if text.is_empty() || text == "-" {
            return Err(malformed(format!("invalid number at offset {start}")));
        }
        if let Ok(value) = text.parse::<i64>() {
            return Ok(CanonicalValue::Int(value));
        }
        text.parse::<u64>().map(CanonicalValue::Uint).map_err(|_| {
            malformed(format!(
                "integer at offset {start} is outside 64-bit bounds"
            ))
        })
    }

    fn string(&mut self) -> ObservedResult<String> {
        self.expect(b'"')?;
        let mut output = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| malformed("unterminated string"))?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(output);
                }
                b'\\' => {
                    self.position += 1;
                    self.escape(&mut output)?;
                }
                control if control < 0x20 => {
                    return Err(malformed(format!(
                        "unescaped control byte at offset {}",
                        self.position
                    )));
                }
                _ => {
                    // The whole document was validated as UTF-8 before parsing.
                    // Copy one maximal unescaped run so validation stays linear
                    // instead of rechecking the entire remaining document for
                    // every character in every string.
                    let start = self.position;
                    while matches!(self.peek(), Some(byte) if byte >= 0x20 && byte != b'"' && byte != b'\\')
                    {
                        self.position += 1;
                    }
                    let text = std::str::from_utf8(&self.bytes[start..self.position])
                        .map_err(|_| malformed("invalid UTF-8 inside string"))?;
                    output.push_str(text);
                }
            }
        }
    }

    fn escape(&mut self, output: &mut String) -> ObservedResult<()> {
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
            b'u' => return self.unicode_escape(output),
            other => {
                return Err(malformed(format!(
                    "unsupported escape `\\{}` at offset {}",
                    other as char,
                    self.position - 1
                )));
            }
        };
        output.push(character);
        Ok(())
    }

    fn unicode_escape(&mut self, output: &mut String) -> ObservedResult<()> {
        let start = self.position;
        let digits = self
            .bytes
            .get(start..start + 4)
            .ok_or_else(|| malformed("truncated Unicode escape"))?;
        let text = std::str::from_utf8(digits)
            .map_err(|_| malformed(format!("invalid Unicode escape at offset {start}")))?;
        let code = u32::from_str_radix(text, 16)
            .map_err(|_| malformed(format!("invalid Unicode escape at offset {start}")))?;
        let character = char::from_u32(code)
            .ok_or_else(|| malformed(format!("invalid Unicode scalar at offset {start}")))?;
        self.position = start + 4;
        output.push(character);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unescaped_utf8_runs_parse_and_escaped_equivalents_are_noncanonical() {
        let canonical = "{\"field\":\"café\"}".as_bytes();
        assert_eq!(parse(canonical).unwrap().to_canonical_bytes(), canonical);

        let escaped = br#"{"field":"caf\u00e9"}"#;
        assert_eq!(
            parse(escaped).unwrap_err().code(),
            codes::INPUT_NOT_CANONICAL
        );
    }
}
