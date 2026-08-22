//! SW-I persisted runtime boundary integration proof.

use nomos_compiler::compile_world;
use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName, SourcePath};
use nomos_projection::CommandArgument;
use nomos_sim::{
    CommandRequest, CommandScript, PersistedRuntimeState, SimulationState, resolve_command,
};

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");

fn plan(source: &str) -> nomos_projection::SimulationPlan {
    let ir = compile_world(source, SourcePath::new("fixtures/gaol.nomos").unwrap()).unwrap();
    nomos_compiler::compile_simulation_plan(&ir).unwrap()
}

fn object(
    value: &mut CanonicalValue,
) -> &mut std::collections::BTreeMap<FieldName, CanonicalValue> {
    let CanonicalValue::Object(fields) = value else {
        panic!("expected canonical object")
    };
    fields
}

#[test]
fn state_and_persisted_envelope_round_trip_exactly() {
    let plan = plan(SOURCE);
    let state = SimulationState::initialize(&plan).unwrap();
    let state_bytes = state.to_canonical_bytes();
    let decoded = SimulationState::from_canonical_bytes(&state_bytes, &plan).unwrap();
    assert_eq!(decoded, state);
    assert_eq!(decoded.to_canonical_bytes(), state_bytes);

    let persisted = PersistedRuntimeState::new(&plan, state).unwrap();
    let persisted_bytes = persisted.to_canonical_bytes();
    let reopened = PersistedRuntimeState::from_canonical_bytes(&persisted_bytes, &plan).unwrap();
    assert_eq!(reopened, persisted);
    assert_eq!(reopened.to_canonical_bytes(), persisted_bytes);
    assert_eq!(reopened.state_hash(), reopened.state().state_hash());
}

#[test]
fn persisted_state_rejects_semantic_rebinding_and_nested_shape_mutation() {
    let original_plan = plan(SOURCE);
    let state = SimulationState::initialize(&original_plan).unwrap();
    let persisted = PersistedRuntimeState::new(&original_plan, state).unwrap();
    let bytes = persisted.to_canonical_bytes();

    let changed_source = SOURCE.replace("credential/gaoler_key", "credential/other_key");
    let changed_plan = plan(&changed_source);
    let error = PersistedRuntimeState::from_canonical_bytes(&bytes, &changed_plan).unwrap_err();
    assert_eq!(
        error.code(),
        nomos_core::diagnostic::codes::RUNTIME_SEMANTICS_MISMATCH
    );

    let mut value = parse_canonical(&bytes).unwrap();
    let state = object(&mut value)
        .get_mut(&FieldName::declared("state"))
        .unwrap();
    let CanonicalValue::Array(entities) = object(state)
        .get_mut(&FieldName::declared("entities"))
        .unwrap()
    else {
        panic!("state entities must be an array")
    };
    object(&mut entities[0]).insert(FieldName::declared("unknown"), CanonicalValue::Bool(true));
    let error =
        PersistedRuntimeState::from_canonical_bytes(&value.to_canonical_bytes(), &original_plan)
            .unwrap_err();
    assert_eq!(
        error.code(),
        nomos_core::diagnostic::codes::RUNTIME_PERSISTED_INVALID
    );
}

#[test]
fn persisted_state_rejects_hash_and_semantic_array_order_tampering() {
    let plan = plan(SOURCE);
    let state = SimulationState::initialize(&plan).unwrap();
    let persisted = PersistedRuntimeState::new(&plan, state).unwrap();
    let bytes = persisted.to_canonical_bytes();

    let mut hash_mutation = parse_canonical(&bytes).unwrap();
    object(&mut hash_mutation).insert(
        FieldName::declared("state_hash"),
        CanonicalValue::text("0".repeat(64)),
    );
    let error =
        PersistedRuntimeState::from_canonical_bytes(&hash_mutation.to_canonical_bytes(), &plan)
            .unwrap_err();
    assert_eq!(
        error.code(),
        nomos_core::diagnostic::codes::RUNTIME_STATE_HASH_MISMATCH
    );

    let mut order_mutation = parse_canonical(&bytes).unwrap();
    let state = object(&mut order_mutation)
        .get_mut(&FieldName::declared("state"))
        .unwrap();
    let CanonicalValue::Array(entities) = object(state)
        .get_mut(&FieldName::declared("entities"))
        .unwrap()
    else {
        panic!("state entities must be an array")
    };
    entities.reverse();
    assert!(
        PersistedRuntimeState::from_canonical_bytes(&order_mutation.to_canonical_bytes(), &plan)
            .is_err()
    );
}

#[test]
fn command_script_round_trips_and_resolves_owned_namespaces() {
    let bytes = b"schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nextinguish brazier_02\n";
    let script = CommandScript::from_bytes(bytes).unwrap();
    assert_eq!(script.to_bytes(), bytes);
    assert_eq!(script.requests().len(), 3);

    let plan = plan(SOURCE);
    let unlock = resolve_command(&plan, &script.requests()[0]).unwrap();
    assert_eq!(unlock.namespace().to_string(), "north_gate.access");
    assert_eq!(unlock.action().as_str(), "unlock");
    assert!(matches!(unlock.argument(), CommandArgument::Credential(_)));

    let open = resolve_command(&plan, &script.requests()[1]).unwrap();
    assert_eq!(open.namespace().to_string(), "north_gate.access");
    assert!(matches!(open.argument(), CommandArgument::None));

    let extinguish = resolve_command(&plan, &script.requests()[2]).unwrap();
    assert_eq!(extinguish.namespace().to_string(), "brazier_02.emission");
}

#[test]
fn command_language_rejects_noncanonical_text_and_requirement_mismatches() {
    for bytes in [
        &b"schema nomos.command_script@1\n"[..],
        &b"schema nomos.command_script@1\r\nopen north_gate\r\n"[..],
        &b"schema nomos.command_script@1\nopen  north_gate\n"[..],
        &b"schema nomos.command_script@1\nopen north_gate \n"[..],
        &b"schema nomos.command_script@1\n# comment\n"[..],
        &b"schema nomos.command_script@1\nopen north_gate\n\n"[..],
        &b"schema nomos.command_script@2\nopen north_gate\n"[..],
        &[0xff][..],
    ] {
        let error = CommandScript::from_bytes(bytes).unwrap_err();
        assert_eq!(
            error.code(),
            nomos_core::diagnostic::codes::RUNTIME_COMMAND_SCRIPT_INVALID
        );
    }

    let plan = plan(SOURCE);
    let missing_credential = CommandRequest::new(
        nomos_core::Ident::new("unlock").unwrap(),
        nomos_core::EntityId::parse("north_gate").unwrap(),
        None,
    );
    assert_eq!(
        resolve_command(&plan, &missing_credential)
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_ARGUMENT_MISMATCH
    );

    let unknown = CommandRequest::new(
        nomos_core::Ident::new("dance").unwrap(),
        nomos_core::EntityId::parse("north_gate").unwrap(),
        None,
    );
    assert_eq!(
        resolve_command(&plan, &unknown).unwrap_err().code(),
        nomos_core::diagnostic::codes::RUNTIME_ACTION_UNDECLARED
    );
}
