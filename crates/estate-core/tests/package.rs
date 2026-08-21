//! Immutable packages: acceptance 12, and verified reads.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use estate_core::canonical::read::parse_canonical;
use estate_core::package::{COMPILER_RECEIPTS_FILE, MANIFEST_FILE, MemberName, WorldPackage};
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
            member(COMPILER_RECEIPTS_FILE),
            CanonicalValue::object_declared([("receipts", CanonicalValue::Array(vec![]))])
                .to_canonical_bytes(),
        ),
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
    assert!(path.join(COMPILER_RECEIPTS_FILE).is_file());
    assert!(path.join("world-ir.json").is_file());
    assert!(
        fs::read_dir(&path)
            .unwrap()
            .all(|entry| entry.unwrap().file_type().unwrap().is_file()),
        "a verified package has no unmanifested subtree"
    );

    // Members are ordered by member name in the manifest, not by write order.
    let names: Vec<String> = package
        .manifest()
        .members()
        .iter()
        .map(|record| record.name().to_string())
        .collect();
    assert_eq!(
        names,
        vec!["compiler-receipts.json", "simulation.json", "world-ir.json"]
    );

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
fn package_bytes_and_digest_ignore_input_insertion_order() {
    let forward_path = fresh_path("forward-order");
    let reverse_path = fresh_path("reverse-order");
    let forward_members = simple_members();
    let mut reverse_members = forward_members.clone();
    reverse_members.reverse();

    let forward = WorldPackage::write(&forward_path, forward_members).unwrap();
    let reverse = WorldPackage::write(&reverse_path, reverse_members).unwrap();
    assert_eq!(forward.manifest(), reverse.manifest());
    assert_eq!(
        fs::read(forward_path.join(MANIFEST_FILE)).unwrap(),
        fs::read(reverse_path.join(MANIFEST_FILE)).unwrap()
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
fn manifest_rows_must_be_in_canonical_member_name_order() {
    let path = fresh_path("unsorted-manifest");
    WorldPackage::write(&path, simple_members()).unwrap();
    rewrite_manifest(&path, |fields| {
        let CanonicalValue::Array(rows) = fields
            .get_mut(&FieldName::declared("members"))
            .expect("writer emits member rows")
        else {
            panic!("manifest members are an array")
        };
        rows.swap(0, 1);
    });

    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0405");
    assert!(rejected.message().contains("canonical member-name order"));
}

#[test]
fn manifest_schema_and_member_rows_reject_canonical_unknown_fields() {
    for (label, mutate) in [
        (
            "manifest-unknown",
            add_manifest_unknown as fn(&mut std::collections::BTreeMap<FieldName, CanonicalValue>),
        ),
        ("schema-unknown", add_schema_unknown),
        ("row-unknown", add_row_unknown),
    ] {
        let path = fresh_path(label);
        WorldPackage::write(&path, simple_members()).unwrap();
        rewrite_manifest(&path, mutate);
        let rejected = WorldPackage::open(&path).unwrap_err();
        assert_eq!(rejected.code().as_str(), "EK0405");
        assert!(rejected.message().contains("fields must be exactly"));
    }
}

#[test]
fn hash_valid_but_noncanonical_member_bytes_are_rejected() {
    let path = fresh_path("noncanonical-open");
    WorldPackage::write(&path, simple_members()).unwrap();
    let bytes = br#"{ "tick": 0 }"#.to_vec();
    fs::write(path.join("world-ir.json"), &bytes).unwrap();
    rewrite_manifest(&path, |fields| {
        let CanonicalValue::Array(rows) = fields
            .get_mut(&FieldName::declared("members"))
            .expect("writer emits member rows")
        else {
            panic!("manifest members are an array")
        };
        let CanonicalValue::Object(row) = rows
            .iter_mut()
            .find(|row| {
                matches!(
                    row,
                    CanonicalValue::Object(fields)
                        if fields.get(&FieldName::declared("name"))
                            == Some(&CanonicalValue::text("world-ir.json"))
                )
            })
            .expect("world IR row exists")
        else {
            panic!("member row is an object")
        };
        row.insert(
            FieldName::declared("sha256"),
            CanonicalValue::text(Sha256Digest::of_bytes(&bytes).to_hex()),
        );
        row.insert(
            FieldName::declared("size"),
            CanonicalValue::Uint(bytes.len() as u64),
        );
    });

    let rejected = WorldPackage::open(&path).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0410");
    assert!(rejected.message().contains("world-ir.json"));
}

#[test]
fn directories_are_never_package_members_or_unmanifested_subtrees() {
    let declared = fresh_path("declared-directory");
    WorldPackage::write(&declared, simple_members()).unwrap();
    fs::remove_file(declared.join("world-ir.json")).unwrap();
    fs::create_dir(declared.join("world-ir.json")).unwrap();
    assert_eq!(
        WorldPackage::open(&declared).unwrap_err().code().as_str(),
        "EK0409"
    );

    let undeclared = fresh_path("undeclared-directory");
    WorldPackage::write(&undeclared, simple_members()).unwrap();
    fs::create_dir(undeclared.join("receipts")).unwrap();
    assert_eq!(
        WorldPackage::open(&undeclared).unwrap_err().code().as_str(),
        "EK0409"
    );
}

#[cfg(unix)]
#[test]
fn symlinked_roots_manifests_and_members_are_rejected_without_following() {
    use std::os::unix::fs::symlink;

    let real = fresh_path("real-package");
    WorldPackage::write(&real, simple_members()).unwrap();
    let linked_root = fresh_path("linked-root");
    symlink(&real, &linked_root).unwrap();
    assert_eq!(
        WorldPackage::open(&linked_root)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0409"
    );

    let linked_manifest = fresh_path("linked-manifest");
    WorldPackage::write(&linked_manifest, simple_members()).unwrap();
    let outside_manifest = fresh_path("outside-manifest");
    fs::rename(linked_manifest.join(MANIFEST_FILE), &outside_manifest).unwrap();
    symlink(&outside_manifest, linked_manifest.join(MANIFEST_FILE)).unwrap();
    assert_eq!(
        WorldPackage::open(&linked_manifest)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0409"
    );

    let linked_member = fresh_path("linked-member");
    WorldPackage::write(&linked_member, simple_members()).unwrap();
    let outside_member = fresh_path("outside-member");
    fs::rename(linked_member.join("world-ir.json"), &outside_member).unwrap();
    symlink(&outside_member, linked_member.join("world-ir.json")).unwrap();
    assert_eq!(
        WorldPackage::open(&linked_member)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0409"
    );

    let broken_output = fresh_path("broken-output-link");
    symlink("missing-target", &broken_output).unwrap();
    let rejected = WorldPackage::write(&broken_output, simple_members()).unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0401");
    assert!(
        fs::symlink_metadata(&broken_output)
            .unwrap()
            .file_type()
            .is_symlink()
    );
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

fn rewrite_manifest(
    package: &std::path::Path,
    mutate: impl FnOnce(&mut std::collections::BTreeMap<FieldName, CanonicalValue>),
) {
    let manifest_path = package.join(MANIFEST_FILE);
    let CanonicalValue::Object(mut fields) =
        parse_canonical(&fs::read(&manifest_path).unwrap()).unwrap()
    else {
        panic!("writer emits an object manifest")
    };
    mutate(&mut fields);
    let body = CanonicalValue::object_declared([
        (
            "members",
            fields
                .get(&FieldName::declared("members"))
                .expect("manifest has members")
                .clone(),
        ),
        (
            "schema",
            fields
                .get(&FieldName::declared("schema"))
                .expect("manifest has schema")
                .clone(),
        ),
    ]);
    fields.insert(
        FieldName::declared("package_digest"),
        CanonicalValue::text(Sha256Digest::of_canonical(&body).to_hex()),
    );
    fs::write(
        manifest_path,
        CanonicalValue::Object(fields).to_canonical_bytes(),
    )
    .unwrap();
}

fn add_manifest_unknown(fields: &mut std::collections::BTreeMap<FieldName, CanonicalValue>) {
    fields.insert(FieldName::declared("extra"), CanonicalValue::Bool(true));
}

fn add_schema_unknown(fields: &mut std::collections::BTreeMap<FieldName, CanonicalValue>) {
    let CanonicalValue::Object(schema) = fields
        .get_mut(&FieldName::declared("schema"))
        .expect("manifest has schema")
    else {
        panic!("manifest schema is an object")
    };
    schema.insert(FieldName::declared("extra"), CanonicalValue::Bool(true));
}

fn add_row_unknown(fields: &mut std::collections::BTreeMap<FieldName, CanonicalValue>) {
    let CanonicalValue::Array(rows) = fields
        .get_mut(&FieldName::declared("members"))
        .expect("manifest has member rows")
    else {
        panic!("manifest members are an array")
    };
    let CanonicalValue::Object(row) = &mut rows[0] else {
        panic!("member row is an object")
    };
    row.insert(FieldName::declared("extra"), CanonicalValue::Bool(true));
}
