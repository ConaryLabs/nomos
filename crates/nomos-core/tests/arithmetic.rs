//! Section 7: integer arithmetic is checked; no authoritative arithmetic wraps.

use nomos_core::arith;

#[test]
fn overflow_is_rejected_rather_than_wrapped() {
    let rejected = arith::add_i64(i64::MAX, 1).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0201");
    assert_eq!(
        rejected
            .repairs()
            .iter()
            .map(|r| r.as_str())
            .collect::<Vec<_>>(),
        vec!["reduce_operand_magnitude"]
    );

    assert!(arith::mul_i64(i64::MIN, -1).is_err());
    assert!(arith::sub_i64(i64::MIN, 1).is_err());
    assert!(arith::add_u64(u64::MAX, 1).is_err());
    assert!(arith::mul_u32(u32::MAX, 2).is_err());

    // The wrapping answers these operations would have produced must never be
    // reachable through this module.
    assert_eq!(i64::MAX.wrapping_add(1), i64::MIN);
    assert!(arith::add_i64(i64::MAX, 1).is_err());
}

#[test]
fn unsigned_subtraction_below_zero_is_overflow_too() {
    let rejected = arith::sub_u64(0, 1).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0201");
}

#[test]
fn division_by_zero_is_a_rejection_not_a_panic() {
    assert_eq!(arith::div_i64(1, 0).unwrap_err().code().as_str(), "EK0201");
    assert_eq!(arith::div_u64(1, 0).unwrap_err().code().as_str(), "EK0201");
    assert_eq!(arith::div_i64(7, 2).unwrap(), 3);
}

#[test]
fn sums_reject_a_running_overflow() {
    assert_eq!(arith::sum_u64([1, 2, 3]).unwrap(), 6);
    assert!(arith::sum_u64([u64::MAX, 1]).is_err());
}
