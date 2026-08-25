//! Presentation numbers, as exact decimals rather than floats.
//!
//! `area.json` still carries raw decimal presentation values — `wallHeight`
//! `4.5`, a masonry mass `height` of `2.6`, a `presentationAnchor` component of
//! `3.8`. `docs/review/executable-gaol-ownership-audit.md` section 4 lists all
//! twenty-six of them and `RUNTIME.md` section 5 R1-3 is the slice that removes
//! them; R1-2 must carry them through unchanged in the meantime.
//!
//! It carries them without ever holding a float. A presentation number is kept
//! as the verbatim lexeme the source wrote plus an exact scaled integer, so
//! comparison ([`Decimal::units`]) is integer comparison and emission is a byte
//! copy. No `f32` or `f64` appears anywhere in this crate; `grep -rn 'f64\|f32'
//! crates/nomos-render-plan/src` is empty, and a test asserts it.
//!
//! The accepted profile is narrow on purpose, because every value outside it is
//! a value R1-3 would have to migrate anyway: an optional `-`, digits with no
//! redundant leading zero, an optional fraction of one to [`MAX_SCALE`] digits,
//! and no exponent.

use crate::error::{PlanError, PlanResult, codes};

/// The fixed number of fraction digits every decimal is scaled to.
pub const MAX_SCALE: u32 = 6;

const UNIT: i128 = 1_000_000;

/// An exact decimal presentation number.
///
/// [`Decimal::lexeme`] is what the source wrote and what the plan emits;
/// [`Decimal::units`] is that value multiplied by `10^MAX_SCALE`, for
/// comparison.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Decimal {
    lexeme: String,
    units: i128,
}

impl Decimal {
    /// Accepts one JSON number lexeme.
    ///
    /// # Errors
    ///
    /// Returns `RP0205` when the lexeme carries an exponent, a redundant
    /// leading zero, a leading `+`, an empty fraction, more than
    /// [`MAX_SCALE`] fraction digits, or does not fit the scaled integer.
    pub fn parse(lexeme: &str) -> PlanResult<Self> {
        let reject = |reason: &str| {
            PlanError::new(
                codes::NUMBER_UNSUPPORTED,
                format!("presentation number `{lexeme}` {reason}"),
            )
        };
        let (negative, digits) = match lexeme.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, lexeme),
        };
        if digits.is_empty() {
            return Err(reject("is empty"));
        }
        let (whole, fraction) = match digits.split_once('.') {
            Some((whole, fraction)) => (whole, Some(fraction)),
            None => (digits, None),
        };
        if whole.is_empty() || !whole.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(reject("does not begin with base-10 digits"));
        }
        if whole.len() > 1 && whole.starts_with('0') {
            return Err(reject("carries a redundant leading zero"));
        }
        if let Some(fraction) = fraction {
            if fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(reject("has a fraction that is not base-10 digits"));
            }
            if fraction.len() as u32 > MAX_SCALE {
                return Err(reject(&format!(
                    "carries more than {MAX_SCALE} fraction digits"
                )));
            }
        }
        let whole_units = whole
            .parse::<i128>()
            .ok()
            .and_then(|value| value.checked_mul(UNIT))
            .ok_or_else(|| reject("does not fit an exact scaled integer"))?;
        let fraction_units = match fraction {
            None => 0,
            Some(fraction) => {
                let padded = format!("{fraction:0<width$}", width = MAX_SCALE as usize);
                padded
                    .parse::<i128>()
                    .map_err(|_| reject("has an unrepresentable fraction"))?
            }
        };
        let magnitude = whole_units
            .checked_add(fraction_units)
            .ok_or_else(|| reject("does not fit an exact scaled integer"))?;
        if negative && magnitude == 0 {
            return Err(reject("spells negative zero"));
        }
        Ok(Self {
            lexeme: lexeme.to_owned(),
            units: if negative { -magnitude } else { magnitude },
        })
    }

    /// Builds an exact decimal from an integer.
    #[must_use]
    pub fn from_i64(value: i64) -> Self {
        Self {
            lexeme: value.to_string(),
            units: i128::from(value) * UNIT,
        }
    }

    /// The verbatim source lexeme, which is what the plan emits.
    #[must_use]
    pub fn lexeme(&self) -> &str {
        &self.lexeme
    }

    /// The value scaled by `10^MAX_SCALE`.
    #[must_use]
    pub const fn units(&self) -> i128 {
        self.units
    }

    /// Whether the value is a whole number.
    #[must_use]
    pub const fn is_integer(&self) -> bool {
        self.units % UNIT == 0
    }

    /// The value as an `i64` when it is a whole number in range.
    #[must_use]
    pub fn as_i64(&self) -> Option<i64> {
        if !self.is_integer() {
            return None;
        }
        i64::try_from(self.units / UNIT).ok()
    }

    /// Whether the value is strictly greater than an integer bound.
    #[must_use]
    pub fn greater_than(&self, bound: i64) -> bool {
        self.units > i128::from(bound) * UNIT
    }

    /// Whether the value is less than or equal to an integer bound.
    #[must_use]
    pub fn at_most(&self, bound: i64) -> bool {
        self.units <= i128::from(bound) * UNIT
    }
}
