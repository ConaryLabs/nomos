//! `SimulationPlan::from_canonical_bytes` against the compiler's own output.
//!
//! The decoder is R1 read-only surface on `nomos-projection`
//! (`RUNTIME.md` section 3, issue #154). Its bound is exact re-encode
//! byte-identity; this test proves the bound holds against the bytes the
//! compiler actually writes for the Gate K base fixture, and that the decoded
//! value is the same value the compiler projected.

use nomos_compiler::compile_world_package;
use nomos_core::{Sha256Digest, SourcePath};
use nomos_projection::SimulationPlan;

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");
const PATH: &str = "fixtures/gaol.nomos";

fn compiled() -> nomos_compiler::CompiledWorld {
    compile_world_package(SOURCE, SourcePath::new(PATH).unwrap()).unwrap()
}

fn simulation_member() -> Vec<u8> {
    compiled()
        .members()
        .unwrap()
        .into_iter()
        .find(|(name, _)| name.as_str() == "simulation.json")
        .expect("the package declares a simulation member")
        .1
}

#[test]
fn the_decoder_reproduces_the_compilers_own_simulation_plan() {
    let bytes = simulation_member();
    let decoded = SimulationPlan::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(decoded.to_canonical_bytes(), bytes);
    assert_eq!(&decoded, compiled().simulation());
}

#[test]
fn the_decoded_plan_reproduces_the_runtime_semantics_digest() {
    let bytes = simulation_member();
    let decoded = SimulationPlan::from_canonical_bytes(&bytes).unwrap();
    assert_eq!(
        Sha256Digest::of_bytes(&decoded.to_canonical_bytes()),
        Sha256Digest::of_bytes(&bytes),
        "a decoded plan must hash to the digest a persisted state binds against"
    );
}

#[test]
fn a_projection_missing_a_field_is_refused() {
    let bytes = simulation_member();
    let text = String::from_utf8(bytes).unwrap();
    let broken = text.replacen(r#""causal_edges":"#, r#""causal_edgez":"#, 1);
    let error = SimulationPlan::from_canonical_bytes(broken.as_bytes()).unwrap_err();
    assert_eq!(error.code().as_str(), "EK0412");
}

#[test]
fn a_projection_naming_another_schema_is_refused() {
    let bytes = simulation_member();
    let text = String::from_utf8(bytes).unwrap();
    let broken = text.replacen(
        r#""name":"nomos.projection.simulation""#,
        r#""name":"nomos.projection.navigation""#,
        1,
    );
    let error = SimulationPlan::from_canonical_bytes(broken.as_bytes()).unwrap_err();
    assert_eq!(error.code().as_str(), "EK0412");
    assert!(error.message().contains("unsupported schema"));
}

#[test]
fn non_canonical_bytes_are_refused() {
    let bytes = simulation_member();
    let text = String::from_utf8(bytes).unwrap();
    let spaced = text.replacen(r#""causal_edges":"#, r#""causal_edges" :"#, 1);
    assert!(SimulationPlan::from_canonical_bytes(spaced.as_bytes()).is_err());
}
