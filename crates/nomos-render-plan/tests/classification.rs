//! Classification comes from typed declarations, never from a string.
//!
//! `RUNTIME.md` section 5 R1-2: "doors, water, and light are classified from
//! typed declarations: a test renames a machine and an entity identifier and
//! the classification is unchanged". Issue #139 sharpens that to *every* entity
//! id and machine namespace in the catalog, renamed consistently across the
//! facts and run bundles.

mod common;

use std::collections::BTreeSet;

use common::{Fixture, Options, scramble};
use nomos_render_plan::EntityKind;

#[test]
fn renaming_every_entity_and_machine_leaves_the_kinds_unchanged() {
    let plain = kinds_of(Fixture::new("plain"));
    let renamed = kinds_of(Fixture::with(
        "scrambled",
        Options {
            rename: scramble,
            ..Options::default()
        },
    ));

    // The rename destroys every convention `build-plan.mjs` classified by: no
    // machine namespace ends in `.access` any more, and no entity id resembles
    // a gate, a brazier, or a flooded section.
    assert!(
        renamed.iter().all(|(id, _)| !id.contains("gate")
            && !id.contains("brazier")
            && !id.contains("flooded")),
        "the rename left a recognisable entity id: {renamed:?}"
    );

    let plain_kinds: Vec<&String> = plain.iter().map(|(_, kind)| kind).collect();
    let renamed_kinds: Vec<&String> = renamed.iter().map(|(_, kind)| kind).collect();
    assert_eq!(
        plain_kinds, renamed_kinds,
        "classification moved when identifiers were renamed"
    );
    assert_eq!(plain_kinds, vec!["water", "door", "light"]);
}

/// Two closed vocabularies remain on `EntityKind`: the `kind` string the plan
/// publishes, and the socket names an effect may attach to on that kind.
///
/// The kind-to-assembly and kind-to-material tables this test used to assert
/// are not stubbed here, they are gone: issue #153 moved them to the renderer
/// catalog with `nomos.rendering_plan@3`, and this crate now names no visual
/// assembly and no material family at all. What is left is the part a kind
/// genuinely owns.
#[test]
fn the_kind_vocabularies_are_closed_and_total() {
    let kinds = [
        EntityKind::Door,
        EntityKind::Water,
        EntityKind::Light,
        EntityKind::Unknown,
    ];

    // The plan's `kind` strings, total over the enum and distinct: a consumer
    // switching on them has four arms and no default.
    let names: Vec<&str> = kinds.iter().map(|kind| kind.as_str()).collect();
    assert_eq!(names, vec!["door", "water", "light", "unknown"]);
    assert_eq!(
        names.iter().collect::<BTreeSet<_>>().len(),
        names.len(),
        "two kinds share one plan string"
    );

    // The socket vocabulary is per kind and closed. Only a door declares one,
    // which is what makes `tests/source.rs`'s "declares: none" refusal a
    // property of the kind rather than of the entity it was written for.
    assert_eq!(EntityKind::Door.sockets(), ["ward"]);
    for kind in [EntityKind::Water, EntityKind::Light, EntityKind::Unknown] {
        assert!(
            kind.sockets().is_empty(),
            "`{}` declares a socket the source reader would then accept",
            kind.as_str()
        );
    }
    for socket in EntityKind::Door.sockets() {
        assert!(
            socket
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'_'),
            "socket `{socket}` is outside the identifier grammar the source reader enforces"
        );
    }

    // `Unknown` is a kind the plan carries, not an absence: it has a string of
    // its own and no socket, so a primitive the plan has no visual kind for
    // compiles into something a consumer can see and refuse.
    assert_eq!(EntityKind::Unknown.as_str(), "unknown");
    assert!(EntityKind::Unknown.sockets().is_empty());
}

#[test]
fn a_primitive_contradicted_by_its_capabilities_is_refused() {
    let fixture = Fixture::new("contradiction");
    // Strip `blocks_ground` from the door's capability set. The primitive still
    // says door; the typed evidence says otherwise, and the compiler refuses
    // rather than emitting a door the resolver will never block.
    edit_catalog(&fixture, |text| {
        text.replace(r#""blocks_ground","boundary""#, r#""boundary""#)
    });
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0201");
    assert!(
        error.message().contains("blocks_ground"),
        "{}",
        error.message()
    );
}

#[test]
fn an_unknown_primitive_carrying_a_door_signature_is_refused() {
    let fixture = Fixture::new("unknown-door");
    // The silent `unknown` fallback at `build-plan.mjs:28` would have drawn
    // this as `visual/marker`. It is a door by every typed capability it
    // carries, so the compiler refuses instead of guessing.
    edit_catalog(&fixture, |text| {
        text.replace(
            "primitive/iron_barred_door",
            "primitive/reinforced_portcullis",
        )
    });
    let error = nomos_render_plan::compile(fixture.inputs()).unwrap_err();
    assert_eq!(error.code().as_str(), "RP0201");
    assert!(
        error.message().contains("primitive/reinforced_portcullis"),
        "{}",
        error.message()
    );
}

fn edit_catalog(fixture: &Fixture, edit: impl Fn(String) -> String) {
    let path = fixture.catalog();
    let text = std::fs::read_to_string(&path).unwrap();
    let edited = edit(text.clone());
    assert_ne!(edited, text, "the catalog edit matched nothing");
    std::fs::write(&path, edited).unwrap();
}

/// Every compiled entity's `(id, kind)`, read back out of the emitted plan.
fn kinds_of(fixture: Fixture) -> Vec<(String, String)> {
    let compiled = nomos_render_plan::compile(fixture.inputs()).unwrap();
    let plan = nomos_render_plan::json::parse(&compiled.bytes).unwrap();
    plan.get("entities")
        .and_then(nomos_render_plan::json::Json::as_array)
        .unwrap()
        .iter()
        .map(|entity| {
            (
                entity
                    .get("id")
                    .and_then(|it| it.as_text())
                    .unwrap()
                    .to_owned(),
                entity
                    .get("kind")
                    .and_then(|it| it.as_text())
                    .unwrap()
                    .to_owned(),
            )
        })
        .collect()
}
