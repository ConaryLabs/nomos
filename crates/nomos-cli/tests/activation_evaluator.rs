//! Acceptance proof for the unified projected-activation evaluator (issue #136).
//!
//! `nomos-compiler` and `nomos-sim` each carried a private evaluator of
//! `ProjectedActivation`. `RUNTIME.md` section 1 adoption criterion 2 — no
//! shadow resolver survives — applies inside the kernel too, so the evaluator
//! now lives once in `nomos-projection` and both callers pass it their own
//! state lookup. The design record is `docs/review/activation-evaluator.md`.
//!
//! The two crates cannot see each other (`KERNEL.md` section 10 permits
//! `nomos-compiler` -> core/schema/projection and `nomos-sim` ->
//! core/projection, and no edge between them), so this test lives in the one
//! crate that depends on both. It is the equivalence nothing proved before:
//! the compile-time initial movement recorded in stable World IR must equal
//! what `resolve_movement` resolves at the initial simulation state, subject by
//! subject, for the fixture and every committed gaol area.

mod common;

use nomos_core::SourcePath;
use nomos_projection::MovementDisposition;
use nomos_sim::{SimulationState, resolve_movement};

#[test]
fn compile_time_initial_movement_equals_the_resolver_at_the_initial_state() {
    let mut subjects_compared = 0_usize;
    for (path, source) in common::worlds() {
        let stable =
            nomos_compiler::compile_world(&source, SourcePath::new(&path).unwrap()).unwrap();
        let plan = nomos_compiler::compile_simulation_plan(&stable).unwrap();
        let initial = SimulationState::initialize(&plan).unwrap();
        let resolved = resolve_movement(&plan, &initial).unwrap();

        assert_eq!(
            stable.movement_v2().len(),
            resolved.facts().len(),
            "{path}: compile-time and runtime subject counts differ"
        );
        for row in stable.movement_v2() {
            let entity = row.entity();
            let compiled = row.movement_disposition_ground();
            let runtime = resolved
                .get(entity)
                .unwrap_or_else(|| panic!("{path}: `{entity}` is absent from resolved facts"));
            match runtime {
                MovementDisposition::Blocked { reasons } => {
                    assert!(
                        compiled.is_blocked(),
                        "{path}: `{entity}` is blocked at runtime but not at compile time"
                    );
                    assert_eq!(
                        compiled.cost(),
                        None,
                        "{path}: `{entity}` blocked disposition carries a cost"
                    );
                    assert_eq!(
                        compiled.reasons(),
                        reasons.as_slice(),
                        "{path}: `{entity}` blocking reasons differ"
                    );
                }
                MovementDisposition::Traversable { cost, reasons } => {
                    assert!(
                        !compiled.is_blocked(),
                        "{path}: `{entity}` is traversable at runtime but blocked at compile time"
                    );
                    assert_eq!(
                        compiled.cost(),
                        Some(*cost),
                        "{path}: `{entity}` effective cost differs"
                    );
                    assert_eq!(
                        compiled.reasons(),
                        reasons.as_slice(),
                        "{path}: `{entity}` cost reasons differ"
                    );
                }
            }
            subjects_compared += 1;
        }
    }
    assert!(
        subjects_compared > 0,
        "the equivalence must compare at least one movement subject"
    );
}

#[test]
fn every_world_carries_a_state_dependent_activation_for_the_evaluator_to_decide() {
    // Without this the equivalence above could pass on worlds whose every claim
    // is `always`, which would exercise no state lookup at all.
    for (path, source) in common::worlds() {
        let stable =
            nomos_compiler::compile_world(&source, SourcePath::new(&path).unwrap()).unwrap();
        let plan = nomos_compiler::compile_simulation_plan(&stable).unwrap();
        let mut state_predicates = 0_usize;
        for subject in plan.movement_resolver().subjects() {
            for claim in subject.claims() {
                claim
                    .activation()
                    .visit_state_equals(&mut |_, _| state_predicates += 1);
            }
        }
        assert!(
            state_predicates > 0,
            "{path}: no movement claim depends on machine state"
        );
    }
}
