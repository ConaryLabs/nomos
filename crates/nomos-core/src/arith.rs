//! Checked arithmetic for authoritative state.
//!
//! `KERNEL.md` section 7: *"Integer arithmetic is checked. Overflow rejects the
//! transaction with a stable error; no authoritative arithmetic wraps
//! implicitly."*
//!
//! These helpers return a [`Diagnostic`] rather than a bare `Option` so the
//! rejection carries `EK0201` and a repair class all the way out to the
//! command result. Rust's own `checked_*` methods are the mechanism; the point
//! of this module is that the *rejection* is a kernel diagnostic, so an
//! overflow cannot be silently swallowed by a `?` on an `Option`.
//!
//! ```
//! use nomos_core::arith;
//! assert_eq!(arith::add_i64(2, 3).unwrap(), 5);
//! let rejected = arith::add_i64(i64::MAX, 1).unwrap_err();
//! assert_eq!(rejected.code().as_str(), "EK0201");
//! ```

use crate::diagnostic::{Diagnostic, RepairClass, codes};

fn overflow(
    operation: &str,
    left: impl std::fmt::Display,
    right: impl std::fmt::Display,
) -> Diagnostic {
    Diagnostic::new(
        codes::ARITHMETIC_OVERFLOW,
        format!("checked {operation} of {left} and {right} does not fit its integer type"),
    )
    .with_repair(RepairClass::ReduceOperandMagnitude)
}

macro_rules! checked_ops {
    ($($add:ident, $sub:ident, $mul:ident, $div:ident, $ty:ty;)*) => {
        $(
            #[doc = concat!("Adds two `", stringify!($ty), "` values.")]
            ///
            /// # Errors
            ///
            /// Returns `EK0201` when the result does not fit.
            pub fn $add(left: $ty, right: $ty) -> Result<$ty, Diagnostic> {
                left.checked_add(right).ok_or_else(|| overflow("addition", left, right))
            }

            #[doc = concat!("Subtracts two `", stringify!($ty), "` values.")]
            ///
            /// # Errors
            ///
            /// Returns `EK0201` when the result does not fit. For unsigned
            /// types that includes any result below zero.
            pub fn $sub(left: $ty, right: $ty) -> Result<$ty, Diagnostic> {
                left.checked_sub(right).ok_or_else(|| overflow("subtraction", left, right))
            }

            #[doc = concat!("Multiplies two `", stringify!($ty), "` values.")]
            ///
            /// # Errors
            ///
            /// Returns `EK0201` when the result does not fit.
            pub fn $mul(left: $ty, right: $ty) -> Result<$ty, Diagnostic> {
                left.checked_mul(right).ok_or_else(|| overflow("multiplication", left, right))
            }

            #[doc = concat!("Divides two `", stringify!($ty), "` values, truncating toward zero.")]
            ///
            /// # Errors
            ///
            /// Returns `EK0201` on division by zero and on the signed
            /// `MIN / -1` overflow.
            pub fn $div(left: $ty, right: $ty) -> Result<$ty, Diagnostic> {
                left.checked_div(right).ok_or_else(|| overflow("division", left, right))
            }
        )*
    };
}

checked_ops! {
    add_i64, sub_i64, mul_i64, div_i64, i64;
    add_u64, sub_u64, mul_u64, div_u64, u64;
    add_u32, sub_u32, mul_u32, div_u32, u32;
}

/// Sums an iterator of `u64` values, rejecting overflow.
///
/// Traversal-cost composition (`TraversalCost<mode> = maximum_applicable_cost`)
/// does not sum, but claim counting and package sizes do, and they must not
/// wrap either.
///
/// # Errors
///
/// Returns `EK0201` when the running total overflows.
pub fn sum_u64(values: impl IntoIterator<Item = u64>) -> Result<u64, Diagnostic> {
    let mut total: u64 = 0;
    for value in values {
        total = add_u64(total, value)?;
    }
    Ok(total)
}
