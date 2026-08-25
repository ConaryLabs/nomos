//! Every field of every document, and the source rule that keeps them integral.
//!
//! `RUNTIME.md` section 5 R1-5 asks for a schema test proving no fractional or
//! wall-clock field reaches authoritative state. This applies it twice: to the
//! emitted documents, and to this crate's own source, because a float that
//! never reaches a document today is a float that reaches one tomorrow.

mod common;

use std::collections::BTreeSet;

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{CanonicalValue, FieldName};
use nomos_play::{
    Direction, PlayCommand, play_command_schema, play_receipt_schema, play_session_schema,
    play_state_schema, presentation_state, presentation_state_schema, rendering_plan_schema,
};

fn field_names(value: &CanonicalValue) -> Vec<String> {
    match value {
        CanonicalValue::Object(fields) => {
            fields.keys().map(|key| key.as_str().to_owned()).collect()
        }
        _ => panic!("expected an object"),
    }
}

#[test]
fn the_five_identities_are_what_the_register_records() {
    assert_eq!(play_state_schema().to_string(), "nomos.play_state@1");
    assert_eq!(play_command_schema().to_string(), "nomos.play_command@1");
    assert_eq!(play_receipt_schema().to_string(), "nomos.play_receipt@1");
    assert_eq!(play_session_schema().to_string(), "nomos.play_session@1");
    assert_eq!(
        presentation_state_schema().to_string(),
        "nomos.presentation_state@1"
    );
    assert_eq!(
        rendering_plan_schema().to_string(),
        "nomos.rendering_plan@3"
    );
}

#[test]
fn a_play_state_carries_exactly_these_fields() {
    let session = common::session();
    let value = session.live().state.to_canonical();
    assert_eq!(
        field_names(&value),
        [
            "actors", "area", "counters", "kernel", "outcome", "pursuit", "schema", "tick"
        ]
    );
}

#[test]
fn the_embedded_kernel_state_is_the_persisted_envelope_verbatim() {
    // The whole point of nesting rather than encoding a blob: the sub-object's
    // canonical bytes *are* the persisted envelope, so the kernel's own strict
    // reader accepts them with no unwrapping step.
    let session = common::session();
    let area = session.live();
    let value = area.state.to_canonical();
    let CanonicalValue::Object(fields) = &value else {
        panic!("a play state is an object");
    };
    let embedded = fields
        .get(&FieldName::declared("kernel"))
        .expect("the play state embeds the kernel");
    assert_eq!(
        embedded.to_canonical_bytes(),
        area.state.kernel.to_canonical_bytes()
    );
    nomos_sim::PersistedRuntimeState::from_canonical_bytes(
        &embedded.to_canonical_bytes(),
        &area.semantics,
    )
    .expect("the kernel reads its own envelope back out of the play state");
}

#[test]
fn no_compiled_static_entity_is_copied_into_the_play_state() {
    // The persisted state's entities are the kernel's own runtime bindings. The
    // plan's entity records — kind, machine namespaces, provenance — must not
    // appear anywhere in an authoritative document.
    let session = common::session();
    let bytes = session.live().state.to_canonical_bytes();
    let text = String::from_utf8(bytes).unwrap();
    for forbidden in [
        "\"kind\":\"door\"",
        "\"kind\":\"water\"",
        "\"kind\":\"light\"",
        "machine_namespaces",
        "provenance",
        "visual_assembly",
        "material_family",
        "assembly",
    ] {
        assert!(
            !text.contains(forbidden),
            "a play state carries `{forbidden}`, which belongs to the plan"
        );
    }
}

#[test]
fn a_play_state_round_trips_through_its_own_bytes() {
    let mut session = common::session();
    common::drive(&mut session, "^^^");
    let area = session.live();
    let bytes = area.state.to_canonical_bytes();
    let decoded = nomos_play::PlayState::decode(&bytes, &area.semantics).unwrap();
    assert_eq!(decoded.to_canonical_bytes(), bytes);
    assert_eq!(&decoded, &area.state);
}

#[test]
fn a_play_state_from_another_world_is_refused_by_the_kernel() {
    // EK0813 is the second lock on the projection decoder: a state cannot be
    // paired with semantics it was not produced under.
    let session = common::session();
    let bytes = session.live().state.to_canonical_bytes();
    let other =
        nomos_projection::SimulationPlan::from_canonical_bytes(&common::semantics("north-gaol"))
            .unwrap();
    let error = nomos_play::PlayState::decode(&bytes, &other).unwrap_err();
    assert_eq!(error.code(), nomos_play::codes::KERNEL_REFUSED);
    assert!(error.message().contains("EK0813"), "{}", error.message());
}

#[test]
fn every_command_kind_round_trips_and_refuses_a_foreign_field() {
    let commands = [
        PlayCommand::Move {
            direction: Direction::North,
        },
        PlayCommand::Interact {
            entity: nomos_core::id::EntityId::parse("north_gate").unwrap(),
            action: nomos_core::Ident::new("ignite").unwrap(),
        },
        PlayCommand::Cross {
            gate: nomos_core::id::EntityId::parse("north_gate").unwrap(),
        },
    ];
    for command in commands {
        let bytes = command.to_canonical_bytes();
        assert_eq!(PlayCommand::decode(&bytes).unwrap(), command);
    }

    // A move carrying an interact's fields is a shape refusal, not a rule one.
    let mixed = br#"{"direction":"north","entity":"north_gate","kind":"move","schema":"nomos.play_command@1"}"#;
    let error = PlayCommand::decode(mixed).unwrap_err();
    assert_eq!(error.code(), nomos_play::codes::COMMAND_SHAPE);
    assert!(!error.is_rule_refusal());
}

#[test]
fn a_command_naming_another_schema_is_refused() {
    let bytes = br#"{"direction":"north","kind":"move","schema":"nomos.play_command@2"}"#;
    let error = PlayCommand::decode(bytes).unwrap_err();
    assert_eq!(error.code(), nomos_play::codes::SCHEMA_MISMATCH);
}

#[test]
fn a_receipt_carries_exactly_these_fields() {
    let mut session = common::session();
    common::drive(&mut session, "^");
    let value = session.receipts()[0].to_canonical();
    assert_eq!(
        field_names(&value),
        [
            "accepted",
            "actor_deltas",
            "area",
            "counters_after",
            "input",
            "kernel_state_hash_after",
            "kernel_state_hash_before",
            "ordinal",
            "outcome_after",
            "outcome_before",
            "play_state_hash_after",
            "previous_receipt_hash",
            "refusal",
            "schema",
            "tick_after",
            "tick_before",
        ]
    );
}

#[test]
fn a_session_carries_exactly_these_fields() {
    let session = common::session();
    assert_eq!(
        field_names(&session.to_canonical()),
        [
            "areas",
            "areas_cleared",
            "log",
            "outcome",
            "position",
            "receipt_chain_head",
            "receipts",
            "route",
            "schema",
        ]
    );
}

#[test]
fn a_presentation_state_carries_exactly_these_fields() {
    let session = common::session();
    let value = presentation_state(session.live()).unwrap();
    assert_eq!(
        field_names(&value),
        [
            "actors",
            "area",
            "counters",
            "effective_light",
            "interactions",
            "kernel_state_hash",
            "machine_states",
            "movement",
            "outcome",
            "pursuit",
            "schema",
            "tick",
        ]
    );
}

#[test]
fn the_presentation_state_spells_movement_the_way_the_plan_does() {
    // Load-bearing: the viewer's scenario accessors read a presentation state
    // unchanged because the two documents spell these three collections
    // identically, including the `null` cost on a blocked subject.
    let session = common::session_at("north-gaol");
    let value = presentation_state(session.live()).unwrap();
    let CanonicalValue::Object(fields) = &value else {
        panic!("a presentation state is an object");
    };
    let movement = fields.get(&FieldName::declared("movement")).unwrap();
    let CanonicalValue::Array(rows) = movement else {
        panic!("movement is an array");
    };
    assert!(!rows.is_empty());
    for row in rows {
        assert_eq!(
            field_names(row),
            ["cost", "disposition", "entity", "reasons"]
        );
    }
    let text = String::from_utf8(value.to_canonical_bytes()).unwrap();
    assert!(
        text.contains(r#""cost":null,"disposition":"blocked""#),
        "a blocked subject spells its cost `null`"
    );

    let light = fields.get(&FieldName::declared("effective_light")).unwrap();
    let CanonicalValue::Array(rows) = light else {
        panic!("effective_light is an array");
    };
    for row in rows {
        assert_eq!(field_names(row), ["emitting", "entity"]);
    }
}

#[test]
fn no_emitted_document_carries_a_decimal_or_a_duration() {
    let session = common::play_route();
    let mut documents = vec![
        session.to_canonical_bytes(),
        presentation_state(session.live())
            .unwrap()
            .to_canonical_bytes(),
    ];
    documents.extend(session.receipts().iter().map(|r| r.to_canonical_bytes()));

    for bytes in documents {
        let text = String::from_utf8(bytes).unwrap();
        // Strip every string literal first: a hex digest is not a number, and a
        // claim identity may contain a dot.
        let mut stripped = String::with_capacity(text.len());
        let mut inside = false;
        for character in text.chars() {
            if character == '"' {
                inside = !inside;
                continue;
            }
            if !inside {
                stripped.push(character);
            }
        }
        assert!(
            !stripped.contains('.'),
            "an emitted document carries a decimal point outside a string"
        );
        for word in ["millis", "seconds", "elapsed", "frame", "timestamp", "now"] {
            assert!(!text.contains(word), "an emitted document carries `{word}`");
        }
    }
}

#[test]
fn this_crate_holds_no_float_no_clock_and_no_randomness() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/src");
    let mut files = Vec::new();
    collect(std::path::Path::new(root), &mut files);
    assert!(files.len() >= 12, "the source tree is where it is expected");
    for path in files {
        // Comments are stripped first: this module's own doc-comment says the
        // reducer "draws no random number", and prose stating the rule must not
        // fail the rule. The crate uses line comments only.
        let text: String = std::fs::read_to_string(&path)
            .unwrap()
            .lines()
            .map(|line| match line.find("//") {
                Some(at) => &line[..at],
                None => line,
            })
            .collect::<Vec<_>>()
            .join("\n");
        for forbidden in [
            "f32",
            "f64",
            "SystemTime",
            "Instant",
            "rand::",
            "random",
            "HashMap",
            "HashSet",
            "elapsed",
            "now()",
        ] {
            assert!(
                !text.contains(forbidden),
                "{} mentions `{forbidden}` outside a comment",
                path.display()
            );
        }
    }
}

fn collect(directory: &std::path::Path, into: &mut Vec<std::path::PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect(&path, into);
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            into.push(path);
        }
    }
}

#[test]
fn the_direction_table_is_the_one_the_renderer_declares() {
    // `apps/nomos-viewer/src/catalog.mjs` declares the same four deltas for the
    // renderer. Two tables, one meaning; this pins the Rust half so a change on
    // either side is visible.
    assert_eq!(Direction::North.delta(), (0, -1));
    assert_eq!(Direction::South.delta(), (0, 1));
    assert_eq!(Direction::West.delta(), (-1, 0));
    assert_eq!(Direction::East.delta(), (1, 0));

    let catalog = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../apps/nomos-viewer/src/catalog.mjs"
    ))
    .unwrap();
    for (name, dx, dy) in [
        ("north", 0, -1),
        ("south", 0, 1),
        ("west", -1, 0),
        ("east", 1, 0),
    ] {
        assert!(
            catalog.contains(&format!("{name}: Object.freeze({{ dx: {dx}, dy: {dy} }})")),
            "the viewer catalog declares {name} as ({dx}, {dy})"
        );
    }
}

#[test]
fn a_plan_without_a_player_is_refused() {
    let bytes = common::plan("north-gaol");
    let text = String::from_utf8(bytes).unwrap();
    let broken = text.replacen(r#""role":"player""#, r#""role":"pursuer""#, 1);
    let error = nomos_play::AreaPlan::decode(broken.as_bytes()).unwrap_err();
    assert_eq!(error.code(), nomos_play::codes::ACTORS_INVALID);
}

#[test]
fn a_plan_at_the_retired_version_is_refused() {
    let bytes = common::plan("north-gaol");
    let text = String::from_utf8(bytes).unwrap();
    let broken = text.replacen("nomos.rendering_plan@3", "nomos.rendering_plan@2", 1);
    let error = nomos_play::AreaPlan::decode(broken.as_bytes()).unwrap_err();
    assert_eq!(error.code(), nomos_play::codes::SCHEMA_MISMATCH);
}

#[test]
fn the_two_ticks_are_different_numbers() {
    // `play_state.tick` counts committed batches, which is inputs. The kernel's
    // tick counts committed kernel transactions. A refused input moves the
    // first and not the second, and neither is the other.
    let mut session = common::session_at("north-gaol");
    common::drive(&mut session, "^^^^>>");
    common::drive(&mut session, "*");
    // Walk into the wall to the north of the gate cell: refused, tick advances.
    session
        .step(&common::step(Direction::East))
        .expect("the input is well formed");
    let area = session.live();
    assert_eq!(area.state.tick, 8);
    assert_eq!(area.state.kernel.state().tick(), 1);
    assert_eq!(
        session.receipts().last().unwrap().tick_after,
        session.receipts().last().unwrap().tick_before + 1
    );
}

#[test]
fn every_emitted_document_reparses_to_the_same_bytes() {
    let session = common::play_route();
    for bytes in [
        session.to_canonical_bytes(),
        presentation_state(session.live())
            .unwrap()
            .to_canonical_bytes(),
        session.receipts()[0].to_canonical_bytes(),
    ] {
        let reparsed = parse_canonical(&bytes).expect("the document is canonical");
        assert_eq!(reparsed.to_canonical_bytes(), bytes);
    }
}

#[test]
fn the_actor_identities_are_free() {
    // The ownership audit's items 7 and 21: `player` and `gaoler` were magic
    // identities. Rename both and nothing changes but the names.
    let text = String::from_utf8(common::plan("north-gaol")).unwrap();
    let renamed = text
        .replace(r#""id":"player""#, r#""id":"runner""#)
        .replace(r#""id":"gaoler""#, r#""id":"warden""#);
    let plan = nomos_play::AreaPlan::decode(renamed.as_bytes()).unwrap();
    let roles: BTreeSet<&str> = plan.actors.iter().map(|a| a.role.as_str()).collect();
    assert_eq!(roles, BTreeSet::from(["player", "pursuer"]));
    let ids: BTreeSet<String> = plan.actors.iter().map(|a| a.id.to_string()).collect();
    assert_eq!(
        ids,
        BTreeSet::from(["runner".to_owned(), "warden".to_owned()])
    );
}
