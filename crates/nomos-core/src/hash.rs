//! State hashing.
//!
//! `KERNEL.md` section 7 fixes the algorithm (SHA-256), the display form
//! (lowercase hexadecimal), and the hash domain (the canonical bytes of the
//! versioned authoritative runtime-state envelope only).
//!
//! [`StateHash`] is a distinct type from [`Sha256Digest`] so a package member
//! digest can never be handed to something expecting authoritative state
//! identity, or the reverse.

mod sha256;

use std::fmt;

use crate::canonical::CanonicalValue;

pub use sha256::sha256;

/// A SHA-256 digest, displayed as 64 lowercase hexadecimal characters.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    /// Hashes raw bytes.
    #[must_use]
    pub fn of_bytes(bytes: &[u8]) -> Self {
        Self(sha256(bytes))
    }

    /// Hashes the canonical bytes of a value.
    #[must_use]
    pub fn of_canonical(value: &CanonicalValue) -> Self {
        Self::of_bytes(&value.to_canonical_bytes())
    }

    /// The digest bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The lowercase hexadecimal form.
    #[must_use]
    pub fn to_hex(&self) -> String {
        let mut out = String::with_capacity(64);
        for byte in self.0 {
            out.push(nibble(byte >> 4));
            out.push(nibble(byte & 0x0f));
        }
        out
    }

    /// Parses a lowercase hexadecimal digest.
    ///
    /// Uppercase input is refused: the contract fixes one display form, and
    /// accepting both would let two spellings of one digest exist.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        let bytes = text.as_bytes();
        if bytes.len() != 64 {
            return None;
        }
        let mut digest = [0_u8; 32];
        for (index, pair) in bytes.as_chunks::<2>().0.iter().enumerate() {
            let high = value_of(pair[0])?;
            let low = value_of(pair[1])?;
            digest[index] = (high << 4) | low;
        }
        Some(Self(digest))
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

/// The identity of one authoritative runtime-state envelope.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct StateHash(Sha256Digest);

impl StateHash {
    /// Hashes a versioned authoritative runtime-state envelope.
    ///
    /// The caller is responsible for the envelope containing only section 7's
    /// included fields. This type hashes what it is given; it cannot know that
    /// a caller smuggled a timestamp into the envelope.
    #[must_use]
    pub fn of_envelope(envelope: &CanonicalValue) -> Self {
        Self(Sha256Digest::of_canonical(envelope))
    }

    /// The underlying digest.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        self.0
    }

    /// The lowercase hexadecimal form.
    #[must_use]
    pub fn to_hex(&self) -> String {
        self.0.to_hex()
    }

    /// Parses the one lowercase hexadecimal state-hash spelling.
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        Sha256Digest::from_hex(text).map(Self)
    }
}

impl fmt::Display for StateHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

fn nibble(value: u8) -> char {
    char::from(match value {
        0..=9 => b'0' + value,
        _ => b'a' + (value - 10),
    })
}

fn value_of(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}
