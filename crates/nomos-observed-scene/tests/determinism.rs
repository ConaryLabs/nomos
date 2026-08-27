mod common;

use nomos_observed_scene::{ScenePlan, compile};

#[test]
fn ten_compiles_are_byte_identical_and_reopenable() {
    let source = common::scene_one();
    let expected = common::plan_one();
    for _ in 0..10 {
        let bytes = compile(&source).expect("compile").to_canonical_bytes();
        assert_eq!(bytes, expected);
        assert_eq!(
            ScenePlan::from_bytes(&bytes)
                .expect("reopen")
                .to_canonical_bytes(),
            bytes
        );
    }
}

#[test]
fn no_r2_source_type_can_hold_a_float_clock_seed_or_raw_transform() {
    let root = common::root().join("crates/nomos-observed-scene/src");
    for entry in std::fs::read_dir(root).expect("read source") {
        let entry = entry.expect("read entry");
        if entry.path().extension().and_then(|one| one.to_str()) != Some("rs") {
            continue;
        }
        let source = std::fs::read_to_string(entry.path()).expect("read Rust source");
        for forbidden in [
            "f32",
            "f64",
            "SystemTime",
            "UNIX_EPOCH",
            "random",
            "quaternion",
        ] {
            assert!(
                !source.contains(forbidden),
                "{} contains {forbidden}",
                entry.path().display()
            );
        }
    }
}
