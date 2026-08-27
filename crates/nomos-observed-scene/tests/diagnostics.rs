use std::collections::BTreeSet;

use nomos_core::RepairClass;
use nomos_observed_scene::{ObservedError, codes, render_rejection};

#[test]
fn the_stable_code_set_is_complete_well_formed_and_disjoint() {
    assert_eq!(codes::ALL.len(), 11);
    assert!(codes::ALL.iter().all(|code| code.is_well_formed()));
    assert_eq!(
        codes::ALL
            .iter()
            .map(|code| code.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        codes::ALL.len()
    );
    assert!(
        codes::ALL
            .iter()
            .all(|code| !code.as_str().starts_with("EK"))
    );
}

#[test]
fn rejection_envelope_and_repairs_are_canonical_sorted_and_duplicate_free() {
    let error = ObservedError::new(codes::FIELD_INVALID, "wrong shape")
        .with_repair(RepairClass::SupplyMissingMember)
        .with_repair(RepairClass::RemoveUnsupportedField)
        .with_repair(RepairClass::SupplyMissingMember);
    assert_eq!(
        render_rejection(&error),
        br#"{"diagnostics":[{"code":"OS0201","message":"wrong shape","repairs":["supply_missing_member","remove_unsupported_field"]}],"status":"rejected"}
"#
    );
}
