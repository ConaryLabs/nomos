//! SW-I persisted runtime boundary integration proof.

use nomos_compiler::compile_world;
use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName, Sha256Digest, SourcePath};
use nomos_projection::{
    CommandArgument, CommandRequirement, CommandTransition, LatticeCell, MachineDefinition,
    ProjectedEntity, RuntimeBinding, SimulationPlan,
};
use nomos_sim::{
    CausalReceipt, CausalReceiptSequence, CommandLog, CommandLogRow, CommandRequest, CommandScript,
    PersistedRuntimeState, RunResult, SimulationState, StateHashSequence, commit_transaction,
    resolve_command,
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

fn executed_evidence() -> (
    nomos_projection::SimulationPlan,
    PersistedRuntimeState,
    PersistedRuntimeState,
    CommandLog,
    CausalReceiptSequence,
    StateHashSequence,
) {
    let plan = plan(SOURCE);
    let script = CommandScript::from_bytes(
        b"schema nomos.command_script@1\nunlock north_gate with credential/gaoler_key\nopen north_gate\nunseal north_gate\nignite north_gate\nextinguish brazier_02\n",
    )
    .unwrap();
    let mut current = SimulationState::initialize(&plan).unwrap();
    let initial = PersistedRuntimeState::new(&plan, current.clone()).unwrap();
    let initial_hash = current.state_hash();
    let mut rows = Vec::new();
    let mut receipts = Vec::new();
    for (ordinal, request) in script.requests().iter().enumerate() {
        let command = resolve_command(&plan, request).unwrap();
        let input_hash = current.state_hash();
        let committed = commit_transaction(&plan, &current, &command).unwrap();
        rows.push(
            CommandLogRow::new(
                u64::try_from(ordinal).unwrap(),
                request.clone(),
                command,
                input_hash,
                committed.receipt(),
            )
            .unwrap(),
        );
        receipts.push(committed.receipt().clone());
        current = committed.into_snapshot();
    }
    let log = CommandLog::new(rows).unwrap();
    let hashes = StateHashSequence::from_command_log(initial_hash, &log).unwrap();
    let receipts = CausalReceiptSequence::new(initial.state().tick(), &log, receipts).unwrap();
    let final_state = PersistedRuntimeState::new(&plan, current).unwrap();
    (plan, initial, final_state, log, receipts, hashes)
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

    let malformed_entities = original_plan
        .entities()
        .iter()
        .map(|entity| {
            let machines = if entity.id().to_string() == "north_gate" {
                Vec::new()
            } else {
                entity.machines().to_vec()
            };
            ProjectedEntity::new(entity.id().clone(), entity.binding().clone(), machines).unwrap()
        })
        .collect();
    let malformed_plan = original_plan
        .clone()
        .with_entities(malformed_entities)
        .unwrap();
    assert_eq!(
        SimulationState::from_canonical_bytes(
            &persisted.state().to_canonical_bytes(),
            &malformed_plan,
        )
        .unwrap_err()
        .code(),
        nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID
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
        &b"schema nomos.command_script@1\nopen\tnorth_gate\n"[..],
        &b"schema nomos.command_script@1\nopen north_gate \n"[..],
        &b"schema nomos.command_script@1\nopen north_gate extra\n"[..],
        &b"schema nomos.command_script@1\nopen north_gate with\n"[..],
        &b"schema nomos.command_script@1\nopen north_gate"[..],
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

    let wrong_credential = CommandRequest::new(
        nomos_core::Ident::new("unlock").unwrap(),
        nomos_core::EntityId::parse("north_gate").unwrap(),
        Some(nomos_core::CatalogValueId::parse("credential/other_key").unwrap()),
    );
    assert_eq!(
        resolve_command(&plan, &wrong_credential)
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_ARGUMENT_MISMATCH
    );

    let unexpected_credential = CommandRequest::new(
        nomos_core::Ident::new("open").unwrap(),
        nomos_core::EntityId::parse("north_gate").unwrap(),
        Some(nomos_core::CatalogValueId::parse("credential/gaoler_key").unwrap()),
    );
    assert_eq!(
        resolve_command(&plan, &unexpected_credential)
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

    let entity = nomos_core::EntityId::parse("subject").unwrap();
    let action = nomos_core::Ident::new("toggle").unwrap();
    let off = nomos_core::Ident::new("off").unwrap();
    let on = nomos_core::Ident::new("on").unwrap();
    let namespaces = ["first", "second"].map(|name| {
        nomos_core::NamespaceId::new(entity.clone(), nomos_core::Ident::new(name).unwrap())
    });
    let machines = namespaces
        .iter()
        .map(|namespace| {
            MachineDefinition::new(
                namespace.clone(),
                vec![off.clone(), on.clone()],
                off.clone(),
                vec![CommandTransition::new(
                    action.clone(),
                    CommandRequirement::None,
                    off.clone(),
                    on.clone(),
                )],
                Vec::new(),
            )
            .unwrap()
        })
        .collect();
    let projected = ProjectedEntity::new(
        entity.clone(),
        RuntimeBinding::Cell(LatticeCell::new(0, 0, 0)),
        namespaces.to_vec(),
    )
    .unwrap();
    let ambiguous_plan = SimulationPlan::new(machines, Vec::new())
        .unwrap()
        .with_entities(vec![projected])
        .unwrap();
    let ambiguous = CommandRequest::new(action, entity, None);
    assert_eq!(
        resolve_command(&ambiguous_plan, &ambiguous)
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_COMMAND_AMBIGUOUS
    );
}

#[test]
fn causal_receipts_reconstruct_complete_typed_evidence() {
    let (_, _, _, _, receipt_sequence, _) = executed_evidence();
    let sequence_bytes = receipt_sequence.to_canonical_bytes();
    assert_eq!(
        CausalReceiptSequence::from_canonical_bytes(&sequence_bytes).unwrap(),
        receipt_sequence
    );
    let receipts = receipt_sequence.receipts();
    assert_eq!(receipts.len(), 5);
    for receipt in receipts {
        let bytes = receipt.to_canonical_bytes();
        let decoded = CausalReceipt::from_canonical_bytes(&bytes).unwrap();
        assert_eq!(&decoded, receipt);
        assert_eq!(decoded.to_canonical_bytes(), bytes);
        assert_eq!(decoded.digest(), Sha256Digest::of_bytes(&bytes));
    }
    assert_eq!(receipts[3].steps().len(), 2, "ignite carries one event");
    assert_eq!(receipts[4].projection_deltas().len(), 3);

    let mut nested_unknown =
        parse_canonical(&receipts[3].to_canonical_bytes()).expect("receipt is canonical");
    let CanonicalValue::Array(transitions) = object(&mut nested_unknown)
        .get_mut(&FieldName::declared("transitions"))
        .unwrap()
    else {
        panic!("receipt transitions must be an array")
    };
    object(&mut transitions[1]).insert(FieldName::declared("unknown"), CanonicalValue::Bool(true));
    assert_eq!(
        CausalReceipt::from_canonical_bytes(&nested_unknown.to_canonical_bytes())
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_PERSISTED_INVALID
    );
}

#[test]
fn logs_hash_sequences_and_results_round_trip_and_cross_validate() {
    let (plan, initial, final_state, log, receipts, hashes) = executed_evidence();
    let log_bytes = log.to_canonical_bytes();
    let decoded_log = CommandLog::from_canonical_bytes(&log_bytes).unwrap();
    assert_eq!(decoded_log, log);
    decoded_log
        .validate_receipts(0, receipts.receipts())
        .unwrap();
    assert_eq!(
        decoded_log
            .validate_receipts(1, receipts.receipts())
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let hash_bytes = hashes.to_canonical_bytes();
    let decoded_hashes = StateHashSequence::from_canonical_bytes(&hash_bytes).unwrap();
    assert_eq!(decoded_hashes, hashes);
    decoded_hashes.validate_command_log(&decoded_log).unwrap();
    assert_eq!(decoded_hashes.rows().len(), decoded_log.rows().len() + 1);
    assert_eq!(
        StateHashSequence::from_command_log(
            receipts.receipts().last().unwrap().state_hash(),
            &decoded_log,
        )
        .unwrap_err()
        .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let result = RunResult::completed(
        Sha256Digest::of_bytes(b"manifest.json"),
        &initial,
        &final_state,
        &decoded_log,
        &receipts,
        &decoded_hashes,
    )
    .unwrap();
    let result_bytes = result.to_canonical_bytes();
    let decoded_result = RunResult::from_canonical_bytes(&result_bytes).unwrap();
    assert_eq!(decoded_result, result);
    decoded_result
        .validate_evidence(
            &initial,
            &final_state,
            &decoded_log,
            &receipts,
            &decoded_hashes,
        )
        .unwrap();
    assert_eq!(decoded_result.committed_command_count(), 5);
    assert_eq!(decoded_result.rejection_diagnostic(), None);

    let rejected = RunResult::rejected(
        Sha256Digest::of_bytes(b"manifest.json"),
        &initial,
        &final_state,
        &decoded_log,
        &receipts,
        &decoded_hashes,
        nomos_core::diagnostic::codes::RUNTIME_SOURCE_STATE_ILLEGAL,
    )
    .unwrap();
    assert_eq!(
        RunResult::from_canonical_bytes(&rejected.to_canonical_bytes())
            .unwrap()
            .rejection_diagnostic(),
        Some(nomos_core::diagnostic::codes::RUNTIME_SOURCE_STATE_ILLEGAL)
    );

    let empty_log = CommandLog::new(Vec::new()).unwrap();
    let empty_state =
        PersistedRuntimeState::new(&plan, SimulationState::initialize(&plan).unwrap()).unwrap();
    let empty_hashes =
        StateHashSequence::from_command_log(empty_state.state_hash(), &empty_log).unwrap();
    let empty_receipts =
        CausalReceiptSequence::new(empty_state.state().tick(), &empty_log, Vec::new()).unwrap();
    assert_eq!(
        RunResult::completed(
            Sha256Digest::of_bytes(b"manifest.json"),
            &empty_state,
            &empty_state,
            &empty_log,
            &empty_receipts,
            &empty_hashes,
        )
        .unwrap_err()
        .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );
    RunResult::rejected(
        Sha256Digest::of_bytes(b"manifest.json"),
        &empty_state,
        &empty_state,
        &empty_log,
        &empty_receipts,
        &empty_hashes,
        nomos_core::diagnostic::codes::RUNTIME_SOURCE_STATE_ILLEGAL,
    )
    .unwrap();
}

#[test]
fn refreshed_receipt_digest_cannot_hide_typed_command_disagreement() {
    let (_, _, _, log, receipt_sequence, _) = executed_evidence();
    let mut receipts = receipt_sequence.receipts().to_vec();
    let last = receipts.last_mut().unwrap();
    let mut receipt_value = parse_canonical(&last.to_canonical_bytes()).unwrap();
    object(
        object(&mut receipt_value)
            .get_mut(&FieldName::declared("command"))
            .unwrap(),
    )
    .insert(
        FieldName::declared("action"),
        CanonicalValue::text("darken"),
    );
    let CanonicalValue::Array(transitions) = object(&mut receipt_value)
        .get_mut(&FieldName::declared("transitions"))
        .unwrap()
    else {
        panic!("receipt transitions must be an array")
    };
    object(
        object(&mut transitions[0])
            .get_mut(&FieldName::declared("cause"))
            .unwrap(),
    )
    .insert(
        FieldName::declared("action"),
        CanonicalValue::text("darken"),
    );
    *last = CausalReceipt::from_canonical_bytes(&receipt_value.to_canonical_bytes()).unwrap();

    let mut log_value = parse_canonical(&log.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(rows) = object(&mut log_value)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("command-log rows must be an array")
    };
    object(rows.last_mut().unwrap()).insert(
        FieldName::declared("causal_receipt_digest"),
        CanonicalValue::text(last.digest().to_hex()),
    );
    let refreshed_log = CommandLog::from_canonical_bytes(&log_value.to_canonical_bytes()).unwrap();
    assert_eq!(
        refreshed_log
            .validate_receipts(0, &receipts)
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );
}

#[test]
fn persisted_evidence_rejects_order_hash_status_and_cross_object_mutations() {
    let (_, initial, final_state, log, receipts, hashes) = executed_evidence();

    let mut reordered = parse_canonical(&log.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(rows) = object(&mut reordered)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("command-log rows must be an array")
    };
    rows.swap(0, 1);
    assert_eq!(
        CommandLog::from_canonical_bytes(&reordered.to_canonical_bytes())
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let mut changed_hash = parse_canonical(&hashes.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(rows) = object(&mut changed_hash)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("state-hash rows must be an array")
    };
    object(&mut rows[1]).insert(
        FieldName::declared("state_hash"),
        CanonicalValue::text("0".repeat(64)),
    );
    let changed_hashes =
        StateHashSequence::from_canonical_bytes(&changed_hash.to_canonical_bytes()).unwrap();
    assert_eq!(
        changed_hashes
            .validate_command_log(&log)
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let result = RunResult::completed(
        Sha256Digest::of_bytes(b"manifest.json"),
        &initial,
        &final_state,
        &log,
        &receipts,
        &hashes,
    )
    .unwrap();
    let mut changed_count = parse_canonical(&result.to_canonical_bytes()).unwrap();
    object(&mut changed_count).insert(
        FieldName::declared("committed_command_count"),
        CanonicalValue::Uint(4),
    );
    let changed_result =
        RunResult::from_canonical_bytes(&changed_count.to_canonical_bytes()).unwrap();
    assert_eq!(
        changed_result
            .validate_evidence(&initial, &final_state, &log, &receipts, &hashes)
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let mut wrong_status = parse_canonical(&result.to_canonical_bytes()).unwrap();
    object(&mut wrong_status).insert(
        FieldName::declared("status"),
        CanonicalValue::text("rejected"),
    );
    assert_eq!(
        RunResult::from_canonical_bytes(&wrong_status.to_canonical_bytes())
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let mut stale_digest_log = parse_canonical(&log.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(rows) = object(&mut stale_digest_log)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("command-log rows must be an array")
    };
    object(&mut rows[0]).insert(
        FieldName::declared("causal_receipt_digest"),
        CanonicalValue::text("0".repeat(64)),
    );
    let stale_digest_log =
        CommandLog::from_canonical_bytes(&stale_digest_log.to_canonical_bytes()).unwrap();
    assert_eq!(
        stale_digest_log
            .validate_receipts(0, receipts.receipts())
            .unwrap_err()
            .code(),
        nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
    );

    let mut log_unknown = parse_canonical(&log.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(rows) = object(&mut log_unknown)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("command-log rows must be an array")
    };
    let request = object(&mut rows[0])
        .get_mut(&FieldName::declared("request"))
        .unwrap();
    object(request).insert(FieldName::declared("unknown"), CanonicalValue::Bool(true));
    assert!(CommandLog::from_canonical_bytes(&log_unknown.to_canonical_bytes()).is_err());

    let mut hash_unknown = parse_canonical(&hashes.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(rows) = object(&mut hash_unknown)
        .get_mut(&FieldName::declared("rows"))
        .unwrap()
    else {
        panic!("state-hash rows must be an array")
    };
    object(&mut rows[0]).insert(FieldName::declared("unknown"), CanonicalValue::Bool(true));
    assert!(StateHashSequence::from_canonical_bytes(&hash_unknown.to_canonical_bytes()).is_err());

    let mut result_unknown = parse_canonical(&result.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(artifacts) = object(&mut result_unknown)
        .get_mut(&FieldName::declared("artifacts"))
        .unwrap()
    else {
        panic!("run artifacts must be an array")
    };
    object(&mut artifacts[0]).insert(FieldName::declared("unknown"), CanonicalValue::Bool(true));
    assert!(RunResult::from_canonical_bytes(&result_unknown.to_canonical_bytes()).is_err());

    let mut artifact_order = parse_canonical(&result.to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(artifacts) = object(&mut artifact_order)
        .get_mut(&FieldName::declared("artifacts"))
        .unwrap()
    else {
        panic!("run artifacts must be an array")
    };
    artifacts.reverse();
    assert!(RunResult::from_canonical_bytes(&artifact_order.to_canonical_bytes()).is_err());

    for artifact_index in 0..5 {
        let mut wrong_artifact_digest = parse_canonical(&result.to_canonical_bytes()).unwrap();
        let CanonicalValue::Array(artifacts) = object(&mut wrong_artifact_digest)
            .get_mut(&FieldName::declared("artifacts"))
            .unwrap()
        else {
            panic!("run artifacts must be an array")
        };
        object(&mut artifacts[artifact_index]).insert(
            FieldName::declared("digest"),
            CanonicalValue::text("0".repeat(64)),
        );
        let wrong_artifact_result =
            RunResult::from_canonical_bytes(&wrong_artifact_digest.to_canonical_bytes()).unwrap();
        assert_eq!(
            wrong_artifact_result
                .validate_evidence(&initial, &final_state, &log, &receipts, &hashes)
                .unwrap_err()
                .code(),
            nomos_core::diagnostic::codes::RUNTIME_EVIDENCE_INCONSISTENT
        );
    }

    let mut receipt_order = parse_canonical(&receipts.receipts()[4].to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(deltas) = object(&mut receipt_order)
        .get_mut(&FieldName::declared("projection_deltas"))
        .unwrap()
    else {
        panic!("projection deltas must be an array")
    };
    deltas.reverse();
    assert!(CausalReceipt::from_canonical_bytes(&receipt_order.to_canonical_bytes()).is_err());

    let mut overflow = parse_canonical(&receipts.receipts()[3].to_canonical_bytes()).unwrap();
    let CanonicalValue::Array(transitions) = object(&mut overflow)
        .get_mut(&FieldName::declared("transitions"))
        .unwrap()
    else {
        panic!("receipt transitions must be an array")
    };
    let payload = object(
        object(&mut transitions[1])
            .get_mut(&FieldName::declared("cause"))
            .unwrap(),
    )
    .get_mut(&FieldName::declared("payload"))
    .unwrap();
    object(payload).insert(
        FieldName::declared("amount"),
        CanonicalValue::Uint(u64::MAX),
    );
    assert!(CausalReceipt::from_canonical_bytes(&overflow.to_canonical_bytes()).is_err());
}
