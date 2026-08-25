//! Classification comes from typed declarations, never from a string.
//!
//! `RUNTIME.md` section 5 R1-2: "doors, water, and light are classified from
//! typed declarations: a test renames a machine and an entity identifier and
//! the classification is unchanged". Issue #139 sharpens that to *every* entity
//! id and machine namespace in the catalog, renamed consistently across the
//! facts and run bundles.

mod common;

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

#[test]
fn the_kind_to_assembly_table_is_closed_and_total() {
    for kind in [
        EntityKind::Door,
        EntityKind::Water,
        EntityKind::Light,
        EntityKind::Unknown,
    ] {
        assert!(kind.visual_assembly().starts_with("visual/"));
        assert!(!kind.material_family().is_empty());
    }
    // The four rows `build-plan.mjs:33-38,43` assigned, unchanged.
    assert_eq!(
        EntityKind::Door.visual_assembly(),
        "visual/iron_barred_door"
    );
    assert_eq!(EntityKind::Door.material_family(), "iron_oxidized");
    assert_eq!(EntityKind::Light.visual_assembly(), "visual/brazier");
    assert_eq!(EntityKind::Light.material_family(), "iron_brazier");
    assert_eq!(EntityKind::Water.visual_assembly(), "visual/shallow_water");
    assert_eq!(EntityKind::Water.material_family(), "water_cold");
    assert_eq!(EntityKind::Unknown.visual_assembly(), "visual/marker");
    assert_eq!(EntityKind::Unknown.material_family(), "stone");
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
