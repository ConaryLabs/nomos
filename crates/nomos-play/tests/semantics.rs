//! The projection decoder, against the compiler's own output.
//!
//! `SimulationPlan::from_canonical_bytes` is R1 read-only surface on
//! `nomos-projection` (`RUNTIME.md` section 3). Its first bound — exact
//! re-encode byte identity — is checked inside the kernel by
//! `crates/nomos-compiler/tests/projection_decode.rs` against
//! `fixtures/gaol.nomos`. This is the other half: every committed area of
//! the R1 corpus, decoded by the crate that actually consumes them, compared
//! with the value the compiler projected.
//!
//! The dev-dependency edge to `nomos-compiler` exists for exactly this. It is
//! not in the built library, so the browser build reaches no compiler and no
//! Canonical World IR.

mod common;

use nomos_core::{Sha256Digest, SourcePath};
use nomos_projection::SimulationPlan;

fn compiled(area: &str) -> nomos_compiler::CompiledWorld {
    let bytes = common::area(area);
    nomos_compiler::compile_world_package(
        &bytes.source,
        SourcePath::new(&bytes.source_path).unwrap(),
    )
    .expect("the committed area compiles")
}

#[test]
fn the_decoder_reproduces_the_compilers_own_plan_for_every_area() {
    for area in common::area_ids() {
        let bytes = common::semantics(&area);
        let decoded = SimulationPlan::from_canonical_bytes(&bytes)
            .unwrap_or_else(|error| panic!("{area}: {error:?}"));
        assert_eq!(decoded.to_canonical_bytes(), bytes, "{area} re-encodes");
        assert_eq!(
            &decoded,
            compiled(&area).simulation(),
            "{area} is the same plan"
        );
    }
}

#[test]
fn every_committed_plan_publishes_the_digest_of_the_projection_it_was_compiled_against() {
    // The first of the decoder's two locks, checked against the committed
    // artifacts rather than against a constructed pair.
    for area in common::area_ids() {
        let plan = nomos_play::AreaPlan::decode(&common::plan(&area)).unwrap();
        assert_eq!(
            plan.semantics_digest,
            Sha256Digest::of_bytes(&common::semantics(&area)),
            "{area}"
        );
    }
}

#[test]
fn the_projection_digest_is_the_runtime_semantics_digest_the_kernel_binds() {
    // The second lock, and why the first one is enough: the kernel derives
    // `runtime_semantics_digest` as the SHA-256 of the plan's own canonical
    // bytes, so a projection that hashes to what the rendering plan published
    // is the projection every persisted state of that world was bound to.
    for area in common::area_ids() {
        let bytes = common::semantics(&area);
        let decoded = SimulationPlan::from_canonical_bytes(&bytes).unwrap();
        let state = nomos_sim::SimulationState::initialize(&decoded).unwrap();
        let persisted = nomos_sim::PersistedRuntimeState::new(&decoded, state).unwrap();
        assert_eq!(
            persisted.runtime_semantics_digest(),
            Sha256Digest::of_bytes(&bytes),
            "{area}"
        );
    }
}

#[test]
fn a_projection_compiled_from_a_different_path_is_a_different_projection() {
    // The source path is inside every claim's span, so it is inside the
    // projection's bytes and inside the digest. This is the same fact
    // `RUNTIME.md` section 5 R1-1 records for the effective-facts projection,
    // seen from the other side.
    let area = common::behavior_area();
    let bytes = common::area(&area);
    let elsewhere = nomos_compiler::compile_world_package(
        &bytes.source,
        SourcePath::new("somewhere/else/world.nomos").unwrap(),
    )
    .unwrap();
    let moved = elsewhere
        .members()
        .unwrap()
        .into_iter()
        .find(|(name, _)| name.as_str() == "simulation.json")
        .unwrap()
        .1;
    assert_ne!(moved, common::semantics(&area));

    let error = nomos_play::PlaySession::start(&common::plan(&area), &moved).unwrap_err();
    assert_eq!(error.code(), nomos_play::codes::SEMANTICS_DIGEST);
}

#[test]
fn a_projection_this_runtime_cannot_reconstruct_is_refused() {
    let bytes = common::semantics(&common::behavior_area());
    let text = String::from_utf8(bytes).unwrap();
    let broken = text.replacen(r#""phase":"causal""#, r#""phase":"eventual""#, 1);
    let error = SimulationPlan::from_canonical_bytes(broken.as_bytes()).unwrap_err();
    assert_eq!(error.code().as_str(), "EK0412");
    assert!(error.message().contains("phase"), "{}", error.message());
}

#[test]
fn the_decoded_plan_carries_every_machine_the_compiler_projected() {
    for area in common::area_ids() {
        let decoded = SimulationPlan::from_canonical_bytes(&common::semantics(&area)).unwrap();
        let expected = compiled(&area);
        assert_eq!(
            decoded.machines().len(),
            expected.simulation().machines().len(),
            "{area}"
        );
        assert_eq!(
            decoded.causal_edges().len(),
            expected.simulation().causal_edges().len(),
            "{area}"
        );
        assert_eq!(
            decoded.movement_resolver().subjects().len(),
            expected.simulation().movement_resolver().subjects().len(),
            "{area}"
        );
        assert_eq!(
            decoded.light_resolver().subjects().len(),
            expected.simulation().light_resolver().subjects().len(),
            "{area}"
        );
    }
}
