//! A small permissive JSON reader for `cargo metadata` output.
//!
//! This is deliberately *not* the kernel's canonical reader. `cargo metadata`
//! is ordinary JSON with whitespace and unsorted keys, and it is tool input
//! rather than authoritative state, so the strict reader would rightly refuse
//! it. Keeping this parser here rather than borrowing `nomos-core` also keeps
//! the boundary checker outside the dependency graph it checks.

use std::collections::BTreeMap;

/// A parsed JSON value.
#[derive(Clone, Debug)]
pub enum Value {
    Null,
    /// `true` or `false`. Parsed for completeness; no boundary rule reads one.
    #[expect(
        dead_code,
        reason = "the reader is complete; the checker reads only strings and arrays"
    )]
    Bool(bool),
    /// Numbers are kept as their source text; the checker never does arithmetic
    /// on them.
    #[expect(
        dead_code,
        reason = "the reader is complete; the checker reads only strings and arrays"
    )]
    Number(String),
    Text(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
}

impl Value {
    /// The value of an object field.
    pub fn get(&self, key: &str) -> Option<&Value> {
        match self {
            Value::Object(fields) => fields.get(key),
            _ => None,
        }
    }

    /// This value as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Text(text) => Some(text),
            _ => None,
        }
    }

    /// This value as an array.
    pub fn as_array(&self) -> Option<&[Value]> {
        match self {
            Value::Array(items) => Some(items),
            _ => None,
        }
    }

    /// An object field read as a string.
    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    /// An object field read as an array.
    pub fn field_array(&self, key: &str) -> &[Value] {
        self.get(key).and_then(Value::as_array).unwrap_or(&[])
    }
}

/// Parses JSON text.
pub fn parse(text: &str) -> Result<Value, String> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        position: 0,
    };
    parser.skip_whitespace();
    let value = parser.value()?;
    parser.skip_whitespace();
    if parser.position != parser.bytes.len() {
        return Err(format!("trailing bytes at offset {}", parser.position));
    }
    Ok(value)
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

    fn expect(&mut self, byte: u8) -> Result<(), String> {
        self.skip_whitespace();
        if self.peek() == Some(byte) {
            self.position += 1;
            Ok(())
        } else {
            Err(format!(
                "expected `{}` at offset {}",
                byte as char, self.position
            ))
        }
    }

    fn value(&mut self) -> Result<Value, String> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'n') => self.word(b"null", Value::Null),
            Some(b't') => self.word(b"true", Value::Bool(true)),
            Some(b'f') => self.word(b"false", Value::Bool(false)),
            Some(b'"') => self.string().map(Value::Text),
            Some(b'[') => self.array(),
            Some(b'{') => self.object(),
            Some(byte) if byte == b'-' || byte.is_ascii_digit() => Ok(self.number()),
            Some(byte) => Err(format!(
                "unexpected byte `{byte:#04x}` at offset {}",
                self.position
            )),
            None => Err("unexpected end of input".to_owned()),
        }
    }

    fn word(&mut self, word: &[u8], value: Value) -> Result<Value, String> {
        if self.bytes[self.position..].starts_with(word) {
            self.position += word.len();
            Ok(value)
        } else {
            Err(format!("unexpected literal at offset {}", self.position))
        }
    }

    fn number(&mut self) -> Value {
        let start = self.position;
        while matches!(
            self.peek(),
            Some(byte) if byte.is_ascii_digit()
                || matches!(byte, b'-' | b'+' | b'.' | b'e' | b'E')
        ) {
            self.position += 1;
        }
        Value::Number(String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned())
    }

    fn array(&mut self) -> Result<Value, String> {
        self.expect(b'[')?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(Value::Array(items));
        }
        loop {
            items.push(self.value()?);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b']') => {
                    self.position += 1;
                    return Ok(Value::Array(items));
                }
                _ => return Err(format!("unterminated array at offset {}", self.position)),
            }
        }
    }

    fn object(&mut self) -> Result<Value, String> {
        self.expect(b'{')?;
        let mut fields = BTreeMap::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(Value::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.string()?;
            self.expect(b':')?;
            let value = self.value()?;
            fields.insert(key, value);
            self.skip_whitespace();
            match self.peek() {
                Some(b',') => self.position += 1,
                Some(b'}') => {
                    self.position += 1;
                    return Ok(Value::Object(fields));
                }
                _ => return Err(format!("unterminated object at offset {}", self.position)),
            }
        }
    }

    fn string(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let mut out = String::new();
        loop {
            let byte = self
                .peek()
                .ok_or_else(|| "unterminated string".to_owned())?;
            match byte {
                b'"' => {
                    self.position += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.position += 1;
                    let escape = self
                        .peek()
                        .ok_or_else(|| "unterminated escape".to_owned())?;
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
                        b'u' => {
                            let digits = self
                                .bytes
                                .get(self.position..self.position + 4)
                                .ok_or_else(|| "truncated escape".to_owned())?;
                            let text = String::from_utf8_lossy(digits).into_owned();
                            let code = u32::from_str_radix(&text, 16)
                                .map_err(|_| format!("bad escape `\\u{text}`"))?;
                            out.push(char::from_u32(code).unwrap_or('\u{fffd}'));
                            self.position += 4;
                        }
                        other => return Err(format!("unsupported escape `\\{}`", other as char)),
                    }
                }
                _ => {
                    let rest = std::str::from_utf8(&self.bytes[self.position..])
                        .map_err(|_| "invalid UTF-8 in string".to_owned())?;
                    let character = rest
                        .chars()
                        .next()
                        .ok_or_else(|| "unterminated string".to_owned())?;
                    self.position += character.len_utf8();
                    out.push(character);
                }
            }
        }
    }
}
