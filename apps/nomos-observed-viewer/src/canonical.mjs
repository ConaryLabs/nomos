const FIELD = /^[a-z][a-z0-9_]*$/;
const MIN_I64 = -(1n << 63n);
const MAX_U64 = (1n << 64n) - 1n;
const decoder = new TextDecoder("utf-8", { fatal: true });
const encoder = new TextEncoder();

export class CanonicalFailure {
  constructor(kind, message, path = "$") {
    this.kind = kind;
    this.message = message;
    this.path = path;
    Object.freeze(this);
  }
}

const fail = (kind, message, path = "$") => {
  throw new CanonicalFailure(kind, message, path);
};

class Parser {
  constructor(text) {
    this.text = text;
    this.at = 0;
  }

  peek() {
    return this.text[this.at];
  }

  whitespace() {
    while ([" ", "\t", "\n", "\r"].includes(this.peek())) this.at += 1;
  }

  value(path, depth = 0) {
    if (depth > 64) fail("json", "nesting exceeds 64 levels", path);
    const first = this.peek();
    if (first === "{") return this.object(path, depth + 1);
    if (first === "[") return this.array(path, depth + 1);
    if (first === '"') return this.string(path);
    if (first === "t" && this.take("true")) return true;
    if (first === "f" && this.take("false")) return false;
    if (first === "n" && this.take("null")) return null;
    if (first === "-" || (first >= "0" && first <= "9")) return this.integer(path);
    fail("json", `unexpected token at character ${this.at}`, path);
  }

  take(text) {
    if (!this.text.startsWith(text, this.at)) return false;
    this.at += text.length;
    return true;
  }

  expect(character, path) {
    if (this.peek() !== character) {
      fail("json", `expected ${JSON.stringify(character)} at character ${this.at}`, path);
    }
    this.at += 1;
  }

  object(path, depth) {
    this.expect("{", path);
    const value = Object.create(null);
    const seen = new Set();
    this.whitespace();
    if (this.peek() === "}") {
      this.at += 1;
      return value;
    }
    for (;;) {
      this.whitespace();
      if (this.peek() !== '"') fail("json", "object key is not a string", path);
      const key = this.string(path);
      const child = `${path}.${key}`;
      if (!FIELD.test(key)) fail("canonical", `invalid canonical field name ${key}`, child);
      if (seen.has(key)) fail("duplicate", `field ${key} occurs more than once`, child);
      seen.add(key);
      this.whitespace();
      this.expect(":", child);
      this.whitespace();
      value[key] = this.value(child, depth);
      this.whitespace();
      if (this.peek() === "}") {
        this.at += 1;
        return value;
      }
      this.expect(",", path);
      this.whitespace();
    }
  }

  array(path, depth) {
    this.expect("[", path);
    const value = [];
    this.whitespace();
    if (this.peek() === "]") {
      this.at += 1;
      return value;
    }
    for (;;) {
      this.whitespace();
      value.push(this.value(`${path}[${value.length}]`, depth));
      this.whitespace();
      if (this.peek() === "]") {
        this.at += 1;
        return value;
      }
      this.expect(",", path);
      this.whitespace();
    }
  }

  string(path) {
    const start = this.at;
    this.expect('"', path);
    let escaped = false;
    for (;;) {
      const character = this.peek();
      if (character === undefined) fail("json", "unterminated string", path);
      this.at += 1;
      if (escaped) {
        escaped = false;
        if (character === "u") {
          const digits = this.text.slice(this.at, this.at + 4);
          if (!/^[0-9a-fA-F]{4}$/.test(digits)) fail("json", "invalid unicode escape", path);
          this.at += 4;
        } else if (!'"\\/bfnrt'.includes(character)) {
          fail("json", "invalid string escape", path);
        }
      } else if (character === "\\") {
        escaped = true;
      } else if (character === '"') {
        try {
          return JSON.parse(this.text.slice(start, this.at));
        } catch {
          fail("json", "invalid JSON string", path);
        }
      } else if (character.charCodeAt(0) < 0x20) {
        fail("json", "unescaped control character", path);
      }
    }
  }

  integer(path) {
    const rest = this.text.slice(this.at);
    const match = /^-?(?:0|[1-9][0-9]*)/.exec(rest);
    if (!match) fail("json", "invalid number", path);
    this.at += match[0].length;
    if (/[.eE]/.test(this.peek() ?? "")) fail("canonical", "numbers must be integers", path);
    try {
      const value = BigInt(match[0]);
      if (value < MIN_I64 || value > MAX_U64) fail("json", "integer is outside 64-bit bounds", path);
      return value;
    } catch {
      fail("json", "invalid integer", path);
    }
  }
}

const encodeString = (value) => {
  let output = '"';
  for (const character of value) {
    const code = character.codePointAt(0);
    if (character === '"') output += '\\"';
    else if (character === "\\") output += "\\\\";
    else if (character === "\b") output += "\\b";
    else if (character === "\f") output += "\\f";
    else if (character === "\n") output += "\\n";
    else if (character === "\r") output += "\\r";
    else if (character === "\t") output += "\\t";
    else if (code < 0x20) output += `\\u00${code.toString(16).padStart(2, "0")}`;
    else output += character;
  }
  return `${output}"`;
};

export const encodeCanonical = (value) => {
  if (value === null) return "null";
  if (value === true) return "true";
  if (value === false) return "false";
  if (typeof value === "bigint") return value.toString(10);
  if (typeof value === "number" && Number.isSafeInteger(value)) return String(value);
  if (typeof value === "string") return encodeString(value);
  if (Array.isArray(value)) return `[${value.map(encodeCanonical).join(",")}]`;
  if (value && typeof value === "object") {
    const keys = Object.keys(value).sort();
    if (keys.some((key) => !FIELD.test(key))) fail("canonical", "invalid object key");
    return `{${keys.map((key) => `${encodeString(key)}:${encodeCanonical(value[key])}`).join(",")}}`;
  }
  fail("canonical", "value is outside the canonical profile");
};

export const parseCanonical = (bytes) => {
  let text;
  try {
    text = typeof bytes === "string" ? bytes : decoder.decode(bytes);
  } catch {
    fail("utf8", "artifact is not UTF-8");
  }
  const parser = new Parser(text);
  parser.whitespace();
  const value = parser.value("$");
  parser.whitespace();
  if (parser.at !== text.length) fail("json", `trailing bytes at character ${parser.at}`);
  const canonical = encodeCanonical(value);
  const actual = typeof bytes === "string" ? encoder.encode(bytes) : new Uint8Array(bytes);
  const expected = encoder.encode(canonical);
  if (actual.length !== expected.length || actual.some((byte, index) => byte !== expected[index])) {
    fail("canonical", "artifact bytes are not canonical");
  }
  return value;
};

export const canonicalBytes = (value) => encoder.encode(encodeCanonical(value));

export const deepFreeze = (value) => {
  if (value && typeof value === "object" && !Object.isFrozen(value)) {
    for (const child of Object.values(value)) deepFreeze(child);
    Object.freeze(value);
  }
  return value;
};
