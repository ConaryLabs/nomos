//! Immutable packages: acceptance 12, and verified reads.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use estate_core::canonical::read::parse_canonical;
use estate_core::package::{MANIFEST_FILE, MemberName, RECEIPTS_DIR, WorldPackage};
use estate_core::{CanonicalValue, FieldName, Sha256Digest};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A fresh directory path that does not exist yet, under the target tmpdir so
/// the suite leaves nothing outside `target/`.
fn fresh_path(label: &str) -> PathBuf {
    let index = COUNTER.fetch_add(1, Ordering::Relaxed);
    // The root carries the process id so two runs never inherit each other's
    // leftovers, which would make "this path does not exist yet" a lie.
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("packages")
        .join(std::process::id().to_string());
    fs::create_dir_all(&root).unwrap();
    root.join(format!("{label}-{index}.world"))
}

fn member(name: &str) -> MemberName {
    MemberName::new(name).unwrap()
}

fn simple_members() -> Vec<(MemberName, Vec<u8>)> {
    vec![
        (
            member("world-ir.json"),
            CanonicalValue::object_declared([("tick", CanonicalValue::Uint(0))])
                .to_canonical_bytes(),
        ),
        (
            member("simulation.json"),
            CanonicalValue::object_declared([("entities", CanonicalValue::Array(vec![]))])
                .to_canonical_bytes(),
        ),
    ]
}

#[test]
fn a_written_package_is_inspectable_with_ordinary_file_tools() {
    let path = fresh_path("inspectable");
    let package = WorldPackage::write(&path, simple_members()).unwrap();

    assert!(path.join(MANIFEST_FILE).is_file());
    assert!(path.join(RECEIPTS_DIR).is_dir());
    assert!(path.join("world-ir.json").is_file());

    // Members are ordered by member name in the manifest, not by write order.
    let names: Vec<String> = package
        .manifest()
        .members()
        .iter()
        .map(|record| record.name().to_string())
        .collect();
    assert_eq!(names, vec!["simulation.json", "world-ir.json"]);

    // The manifest itself is canonical, so `sha256sum` on a member file
    // reproduces the recorded digest byte for byte.
    let bytes = fs::read(path.join("world-ir.json")).unwrap();
    assert_eq!(bytes, br#"{"tick":0}"#.to_vec());

    let reopened = WorldPackage::open(&path).unwrap();
    assert_eq!(reopened.manifest(), package.manifest());
    // Compared as canonical bytes: the reader spells an integer that fits
    // `i64` as `Int`, and the byte profile — not the Rust variant — is the
    // contract.
    assert_eq!(
        reopened
            .member_value(&member("world-ir.json"))
            .unwrap()
            .to_canonical_bytes(),
        CanonicalValue::object_declared([("tick", CanonicalValue::Uint(0))]).to_canonical_bytes()
    );
}

#[test]
fn writing_over_an_existing_package_is_refused() {
    // Acceptance 12: compile, command, run, replay, and migrate write new
    // outputs. A package is evidence, and evidence is not edited.
    let path = fresh_path("immutable");
    WorldPackage::write(&path, simple_members()).unwrap();

    let rejected = WorldPackage::write(&path, simple_members()).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0401");
    assert_eq!(
        rejected
            .repairs()
            .iter()
            .map(|r| r.as_str())
            .collect::<Vec<_>>(),
        vec!["write_to_new_output_path"]
    );

    // Even an empty directory in the way is refused; there is no "merge into".
    let occupied = fresh_path("occupied");
    fs::create_dir_all(&occupied).unwrap();
    assert_eq!(
        WorldPackage::write(&occupied, simple_members())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0401"
    );

    // A plain file in the way is refused too, and is left untouched.
    let file_path = fresh_path("occupied-file");
    fs::write(&file_path, b"not a package").unwrap();
    assert!(WorldPackage::write(&file_path, simple_members()).is_err());
    assert_eq!(fs::read(&file_path).unwrap(), b"not a package".to_vec());
}

#[test]
fn a_tampered_member_is_caught_on_read() {
    let path = fresh_path("tampered");
    WorldPackage::write(&path, simple_members()).unwrap();

    // Same length, different bytes: only the hash catches this.
    fs::write(path.join("world-ir.json"), br#"{"tick":9}"#).unwrap();
    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0403");
    assert!(rejected.message().contains("world-ir.json"));
}

#[test]
fn a_missing_member_is_caught_on_read() {
    let path = fresh_path("missing");
    WorldPackage::write(&path, simple_members()).unwrap();
    fs::remove_file(path.join("simulation.json")).unwrap();

    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0402");
}

#[test]
fn a_smuggled_extra_file_is_caught_on_read() {
    let path = fresh_path("smuggled");
    WorldPackage::write(&path, simple_members()).unwrap();
    fs::write(path.join("extra.json"), br#"{"a":1}"#).unwrap();

    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0404");
    assert!(rejected.message().contains("extra.json"));
}

#[test]
fn a_tampered_manifest_is_caught_on_read() {
    let path = fresh_path("manifest");
    WorldPackage::write(&path, simple_members()).unwrap();

    let manifest = fs::read_to_string(path.join(MANIFEST_FILE)).unwrap();
    // Re-point a member at a different size; the package digest no longer
    // matches the body it claims to cover.
    let edited = manifest.replacen(r#""size":10"#, r#""size":11"#, 1);
    assert_ne!(edited, manifest);
    fs::write(path.join(MANIFEST_FILE), &edited).unwrap();

    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0405");
    assert!(rejected.message().contains("package_digest"));
}

#[test]
fn a_non_canonical_member_never_reaches_disk() {
    let path = fresh_path("non-canonical");
    let rejected = WorldPackage::write(
        &path,
        vec![(member("world-ir.json"), br#"{ "tick": 0 }"#.to_vec())],
    )
    .unwrap_err();
    // Insignificant whitespace is refused structurally by the reader (EK0302)
    // rather than by the re-encode comparison (EK0303); both mean the same
    // thing here, which is that the bytes are not canonical.
    assert_eq!(rejected.code().as_str(), "EK0302");
    assert!(
        !path.exists(),
        "a refused write must not leave a partial package behind"
    );
}

#[test]
fn duplicate_input_members_fail_before_the_package_exists() {
    let path = fresh_path("duplicate-input");
    let bytes =
        CanonicalValue::object_declared([("tick", CanonicalValue::Uint(0))]).to_canonical_bytes();
    let rejected = WorldPackage::write(
        &path,
        vec![
            (member("world-ir.json"), bytes.clone()),
            (member("world-ir.json"), bytes),
        ],
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0408");
    assert!(!path.exists());
}

#[test]
fn duplicate_manifest_rows_fail_even_with_a_recomputed_digest() {
    let path = fresh_path("duplicate-manifest-row");
    WorldPackage::write(&path, simple_members()).unwrap();

    let manifest_path = path.join(MANIFEST_FILE);
    let CanonicalValue::Object(mut fields) =
        parse_canonical(&fs::read(&manifest_path).unwrap()).unwrap()
    else {
        panic!("the writer emits an object manifest")
    };
    let schema = fields
        .get(&FieldName::declared("schema"))
        .expect("the writer emits a schema")
        .clone();
    let CanonicalValue::Array(rows) = fields
        .get_mut(&FieldName::declared("members"))
        .expect("the writer emits members")
    else {
        panic!("manifest members are an array")
    };
    rows.push(rows[0].clone());
    let body = CanonicalValue::object_declared([
        ("members", CanonicalValue::Array(rows.clone())),
        ("schema", schema),
    ]);
    fields.insert(
        FieldName::declared("package_digest"),
        CanonicalValue::text(Sha256Digest::of_canonical(&body).to_hex()),
    );
    fs::write(
        &manifest_path,
        CanonicalValue::Object(fields).to_canonical_bytes(),
    )
    .unwrap();

    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0405");
    assert!(rejected.message().contains("occurs more than once"));
}

#[test]
fn member_names_are_constrained() {
    assert!(MemberName::new("world-ir.json").is_ok());
    assert!(MemberName::new("schemas.json").is_ok());
    for illegal in [
        "manifest.json",
        "World-IR.json",
        "world_ir.json",
        "-world.json",
        "world-.json",
        "world-ir.txt",
        "receipts",
        "../escape.json",
    ] {
        assert!(
            MemberName::new(illegal).is_err(),
            "`{illegal}` must be refused"
        );
    }
}
