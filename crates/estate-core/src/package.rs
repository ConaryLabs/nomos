//! Immutable world packages.
//!
//! `KERNEL.md` section 5: a compiled world package is immutable evidence, laid
//! out as an inspectable directory so `ls`, `cat`, and `diff` work:
//!
//! ```text
//! build/gaol.world/
//!   manifest.json
//!   world-ir.json
//!   simulation.json
//!   navigation.json
//!   persistence.json
//!   diagnostics.json
//!   schemas.json
//!   compiler-receipts.json
//! ```
//!
//! # Why this lives in `estate-core`
//!
//! Section 10 gives `estate-core` "canonical bytes and hashing" and
//! `estate-projection` "versioned simulation/navigation/persistence/diagnostic
//! schemas". A package is not a projection schema: it is a directory of named
//! canonical byte members with a hashed manifest, which is exactly canonical
//! bytes plus hashing. It also has to be reachable from more crates than
//! `estate-projection` is. `world-ir.json` and `schemas.json` are
//! `estate-schema` artifacts, and `estate-schema` may depend only on
//! `estate-core`; putting the writer in `estate-projection` would either strand
//! the schema crate or add an edge section 10 forbids. `estate-core` is the one
//! crate every package producer and consumer can already reach.
//!
//! Consequently this module knows nothing about member *meaning*. It enforces
//! bytes, names, hashes, and immutability. What `simulation.json` must contain
//! belongs to `estate-projection`, and it stays there.
//!
//! # What is enforced
//!
//! - **Immutability (acceptance 12).** [`WorldPackage::write`] refuses to write
//!   into a path that already exists, of any kind. Nothing here can overwrite,
//!   merge into, or append to an existing package.
//! - **Atomic publication.** A complete sibling staging directory is verified
//!   before one same-filesystem rename publishes it. Failed writes remove their
//!   staging path and leave the requested destination absent.
//! - **Canonical members.** Every member is checked against the canonical byte
//!   profile before the directory is created, so a package cannot hold a member
//!   that would hash differently after a round trip.
//! - **Verified reads.** [`WorldPackage::open`] recomputes the manifest digest
//!   and every member's size and hash, revalidates canonical bytes, and refuses
//!   unknown fields, non-regular entries, or anything the manifest does not
//!   declare.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::canonical::read::parse_canonical;
use crate::canonical::{CanonicalValue, FieldName};
use crate::diagnostic::{Diagnostic, RepairClass, codes};
use crate::hash::Sha256Digest;
use crate::id::SchemaId;

/// The manifest file name.
pub const MANIFEST_FILE: &str = "manifest.json";

/// The canonical, manifest-hashed compiler/build receipt member from section 5.
pub const COMPILER_RECEIPTS_FILE: &str = "compiler-receipts.json";

static STAGING_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The name and version of the manifest schema itself.
///
/// # Panics
///
/// Panics if the built-in literal is not a valid schema id, which the crate's
/// tests rule out.
#[must_use]
pub fn manifest_schema() -> SchemaId {
    SchemaId::new("estate.package.manifest", 1).expect("the manifest schema id is a valid literal")
}

/// A package member file name, for example `world-ir.json`.
///
/// Shape: lowercase alphanumerics and inner hyphens, then `.json`.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct MemberName(String);

impl MemberName {
    /// Accepts a member file name.
    ///
    /// # Errors
    ///
    /// Returns `EK0406` when the name is not `[a-z0-9]([a-z0-9-]*[a-z0-9])?.json`,
    /// or when it collides with the manifest entry.
    pub fn new(name: &str) -> Result<Self, Diagnostic> {
        let reject = |reason: &str| {
            Err(Diagnostic::new(
                codes::PACKAGE_MEMBER_NAME_INVALID,
                format!("`{name}` is not a legal package member name: {reason}"),
            )
            .with_repair(RepairClass::UseSupportedIdentifierShape))
        };
        let Some(stem) = name.strip_suffix(".json") else {
            return reject("member files end in `.json`");
        };
        let bytes = stem.as_bytes();
        let legal_body = bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
        let legal_edges = bytes
            .first()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
            && bytes
                .last()
                .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit());
        if !legal_body || !legal_edges {
            return reject("the stem is `[a-z0-9]([a-z0-9-]*[a-z0-9])?`");
        }
        if name == MANIFEST_FILE {
            return reject("the manifest is not a member of itself");
        }
        Ok(Self(name.to_owned()))
    }

    /// The member name as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemberName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One manifest row: a member's name, byte length, and digest.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct MemberRecord {
    name: MemberName,
    size: u64,
    digest: Sha256Digest,
}

impl MemberRecord {
    /// The member's file name.
    #[must_use]
    pub fn name(&self) -> &MemberName {
        &self.name
    }

    /// The member's byte length.
    #[must_use]
    pub fn size(&self) -> u64 {
        self.size
    }

    /// The member's SHA-256 digest.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        self.digest
    }

    fn to_canonical(&self) -> CanonicalValue {
        CanonicalValue::object_declared([
            ("name", CanonicalValue::text(self.name.as_str())),
            ("sha256", CanonicalValue::text(self.digest.to_hex())),
            ("size", CanonicalValue::Uint(self.size)),
        ])
    }
}

/// The package manifest: the schema it is written under, its member rows in
/// member-name order, and the digest that binds them together.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PackageManifest {
    schema: SchemaId,
    members: Vec<MemberRecord>,
    digest: Sha256Digest,
}

impl PackageManifest {
    /// The manifest schema and version.
    #[must_use]
    pub fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// The member rows, ordered by member name.
    #[must_use]
    pub fn members(&self) -> &[MemberRecord] {
        &self.members
    }

    /// The package digest: SHA-256 over the canonical bytes of the schema and
    /// member rows. It cannot cover itself, so the digest field is excluded
    /// from its own domain and recomputed on every read.
    #[must_use]
    pub fn digest(&self) -> Sha256Digest {
        self.digest
    }

    fn body(schema: &SchemaId, members: &[MemberRecord]) -> CanonicalValue {
        CanonicalValue::object_declared([
            (
                "members",
                CanonicalValue::Array(members.iter().map(MemberRecord::to_canonical).collect()),
            ),
            ("schema", schema.to_canonical()),
        ])
    }

    fn to_canonical(&self) -> CanonicalValue {
        let CanonicalValue::Object(mut fields) = Self::body(&self.schema, &self.members) else {
            unreachable!("the manifest body is an object");
        };
        fields.insert(
            FieldName::declared("package_digest"),
            CanonicalValue::text(self.digest.to_hex()),
        );
        CanonicalValue::Object(fields)
    }
}

/// An immutable world package on disk.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct WorldPackage {
    root: PathBuf,
    manifest: PackageManifest,
    members: BTreeMap<MemberName, Vec<u8>>,
}

impl WorldPackage {
    /// Writes a new package directory.
    ///
    /// Members are validated as canonical bytes before filesystem mutation. A
    /// complete sibling staging directory is written and verified before one
    /// same-filesystem rename publishes it. The supported atomicity boundary is
    /// a local filesystem with one publisher for the requested destination.
    ///
    /// # Errors
    ///
    /// - `EK0401` when `root` already exists. This is acceptance 12: a package
    ///   is evidence and is never written over.
    /// - `EK0302` or `EK0303` when a member's bytes are not canonical.
    /// - `EK0407` when the filesystem refuses the write.
    /// - `EK0408` when the same member name is supplied more than once.
    pub fn write(
        root: &Path,
        members: impl IntoIterator<Item = (MemberName, Vec<u8>)>,
    ) -> Result<Self, Diagnostic> {
        Self::write_internal(root, members, None)
    }

    fn write_internal(
        root: &Path,
        members: impl IntoIterator<Item = (MemberName, Vec<u8>)>,
        fail_after_member_writes: Option<usize>,
    ) -> Result<Self, Diagnostic> {
        require_absent_destination(root)?;
        let mut unique_members = BTreeMap::new();
        for (name, bytes) in members {
            if unique_members.insert(name.clone(), bytes).is_some() {
                return Err(Diagnostic::new(
                    codes::PACKAGE_MEMBER_DUPLICATE,
                    format!("package member `{name}` is supplied more than once"),
                )
                .with_repair(RepairClass::RemoveDuplicateDeclaration));
            }
        }
        let members = unique_members;
        for (name, bytes) in &members {
            parse_canonical(bytes).map_err(|diagnostic| {
                Diagnostic::new(
                    diagnostic.code(),
                    format!("member `{name}` is not canonical: {}", diagnostic.message()),
                )
                .with_repair(RepairClass::EmitCanonicalBytes)
            })?;
        }

        let records: Vec<MemberRecord> = members
            .iter()
            .map(|(name, bytes)| MemberRecord {
                name: name.clone(),
                size: bytes.len() as u64,
                digest: Sha256Digest::of_bytes(bytes),
            })
            .collect();
        let schema = manifest_schema();
        let digest = Sha256Digest::of_canonical(&PackageManifest::body(&schema, &records));
        let manifest = PackageManifest {
            schema,
            members: records,
            digest,
        };

        let parent = package_parent(root);
        fs::create_dir_all(parent).map_err(|error| io_failure(parent, &error))?;
        let staging = create_staging_directory(root)?;
        let staged_result = (|| {
            for (written, (name, bytes)) in members.iter().enumerate() {
                if fail_after_member_writes == Some(written) {
                    return Err(io_failure(
                        &staging.join(name.as_str()),
                        &io::Error::other("injected package-write failure"),
                    ));
                }
                let path = staging.join(name.as_str());
                fs::write(&path, bytes).map_err(|error| io_failure(&path, &error))?;
            }
            let manifest_path = staging.join(MANIFEST_FILE);
            fs::write(&manifest_path, manifest.to_canonical().to_canonical_bytes())
                .map_err(|error| io_failure(&manifest_path, &error))?;

            let mut verified = Self::open(&staging)?;
            require_absent_destination(root)?;
            fs::rename(&staging, root).map_err(|error| publication_failure(root, &error))?;
            verified.root = root.to_path_buf();
            Ok(verified)
        })();

        match staged_result {
            Ok(package) => Ok(package),
            Err(diagnostic) => {
                cleanup_staging(&staging).map_err(|cleanup| {
                    Diagnostic::new(
                        codes::PACKAGE_IO,
                        format!(
                            "package write failed ({diagnostic}); staging cleanup also failed: {}",
                            cleanup.message()
                        ),
                    )
                })?;
                Err(diagnostic)
            }
        }
    }

    /// Opens a package, verifying the manifest digest and every member.
    ///
    /// # Errors
    ///
    /// - `EK0405` when the manifest is missing, not canonical, structurally
    ///   wrong, or its recomputed digest disagrees with the recorded one.
    /// - `EK0402` when a declared member is missing.
    /// - `EK0403` when a member's size or digest disagrees with the manifest.
    /// - `EK0404` when the package root holds a file the manifest does not
    ///   declare.
    /// - `EK0407` when the filesystem refuses the read.
    /// - `EK0409` when the root, manifest, or a member is not the required
    ///   directory/regular-file entry type.
    /// - `EK0410` when a hash-valid member is not canonical semantic bytes.
    pub fn open(root: &Path) -> Result<Self, Diagnostic> {
        require_package_root(root)?;
        let manifest_path = root.join(MANIFEST_FILE);
        require_manifest_file(&manifest_path)?;
        let manifest_bytes =
            fs::read(&manifest_path).map_err(|error| io_failure(&manifest_path, &error))?;
        let manifest = decode_manifest(&manifest_bytes)?;

        let declared: BTreeMap<&MemberName, &MemberRecord> = manifest
            .members
            .iter()
            .map(|record| (&record.name, record))
            .collect();

        let entries = fs::read_dir(root).map_err(|error| io_failure(root, &error))?;
        for entry in entries {
            let entry = entry.map_err(|error| io_failure(root, &error))?;
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                return Err(Diagnostic::new(
                    codes::PACKAGE_MEMBER_UNDECLARED,
                    "package entry name is not valid UTF-8",
                )
                .with_repair(RepairClass::RemoveUndeclaredMember));
            };
            let file_type = entry
                .file_type()
                .map_err(|error| io_failure(&entry.path(), &error))?;
            if !file_type.is_file() {
                return Err(entry_type_invalid(
                    &entry.path(),
                    "package entries must be regular files",
                ));
            }
            if name == MANIFEST_FILE {
                continue;
            }
            let declared_here = MemberName::new(name)
                .ok()
                .is_some_and(|member| declared.contains_key(&member));
            if !declared_here {
                return Err(Diagnostic::new(
                    codes::PACKAGE_MEMBER_UNDECLARED,
                    format!("`{name}` is present in the package but not declared by the manifest"),
                )
                .with_repair(RepairClass::RemoveUndeclaredMember));
            }
        }

        let mut members = BTreeMap::new();
        for record in &manifest.members {
            let path = root.join(record.name.as_str());
            require_member_file(&path, &record.name)?;
            let bytes = fs::read(&path).map_err(|error| {
                if error.kind() == ErrorKind::NotFound {
                    Diagnostic::new(
                        codes::PACKAGE_MEMBER_MISSING,
                        format!(
                            "member `{}` is declared by the manifest but could not be read",
                            record.name
                        ),
                    )
                    .with_repair(RepairClass::RebuildFromSource)
                } else {
                    io_failure(&path, &error)
                }
            })?;
            if bytes.len() as u64 != record.size || Sha256Digest::of_bytes(&bytes) != record.digest
            {
                return Err(Diagnostic::new(
                    codes::PACKAGE_MEMBER_HASH_MISMATCH,
                    format!(
                        "member `{}` does not match the manifest: recorded {} bytes / {}, found {} bytes / {}",
                        record.name,
                        record.size,
                        record.digest,
                        bytes.len(),
                        Sha256Digest::of_bytes(&bytes)
                    ),
                )
                .with_repair(RepairClass::RebuildFromSource),
                );
            }
            parse_canonical(&bytes).map_err(|diagnostic| {
                Diagnostic::new(
                    codes::PACKAGE_MEMBER_NON_CANONICAL,
                    format!(
                        "member `{}` is hash-valid but not canonical: {}",
                        record.name,
                        diagnostic.message()
                    ),
                )
                .with_repair(RepairClass::RebuildFromSource)
            })?;
            members.insert(record.name.clone(), bytes);
        }

        Ok(Self {
            root: root.to_path_buf(),
            manifest,
            members,
        })
    }

    /// The package directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The verified manifest.
    #[must_use]
    pub fn manifest(&self) -> &PackageManifest {
        &self.manifest
    }

    /// The raw canonical bytes of one member.
    #[must_use]
    pub fn member_bytes(&self, name: &MemberName) -> Option<&[u8]> {
        self.members.get(name).map(Vec::as_slice)
    }

    /// One member parsed as a canonical value.
    ///
    /// # Errors
    ///
    /// Returns `EK0402` when the member is not declared by this package.
    pub fn member_value(&self, name: &MemberName) -> Result<CanonicalValue, Diagnostic> {
        let bytes = self.member_bytes(name).ok_or_else(|| {
            Diagnostic::new(
                codes::PACKAGE_MEMBER_MISSING,
                format!("this package has no member `{name}`"),
            )
            .with_repair(RepairClass::SupplyMissingMember)
        })?;
        parse_canonical(bytes)
    }
}

fn require_absent_destination(root: &Path) -> Result<(), Diagnostic> {
    match fs::symlink_metadata(root) {
        Ok(_) => Err(Diagnostic::new(
            codes::PACKAGE_OUTPUT_EXISTS,
            format!(
                "`{}` already exists; compiled packages are immutable evidence and are never written over",
                root.display()
            ),
        )
        .with_repair(RepairClass::WriteToNewOutputPath)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_failure(root, &error)),
    }
}

fn package_parent(root: &Path) -> &Path {
    root.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn create_staging_directory(root: &Path) -> Result<PathBuf, Diagnostic> {
    let Some(file_name) = root.file_name() else {
        return Err(Diagnostic::new(
            codes::PACKAGE_IO,
            format!("`{}` has no package directory name", root.display()),
        ));
    };
    let parent = package_parent(root);
    for _ in 0..64 {
        let counter = STAGING_COUNTER.fetch_add(1, Ordering::Relaxed);
        let name = format!(
            ".{}.staging-{}-{counter}",
            file_name.to_string_lossy(),
            std::process::id()
        );
        let staging = parent.join(name);
        match fs::create_dir(&staging) {
            Ok(()) => return Ok(staging),
            Err(error) if error.kind() == ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(io_failure(&staging, &error)),
        }
    }
    Err(Diagnostic::new(
        codes::PACKAGE_IO,
        format!(
            "could not allocate a fresh sibling staging directory for `{}`",
            root.display()
        ),
    ))
}

fn cleanup_staging(staging: &Path) -> Result<(), Diagnostic> {
    match fs::remove_dir_all(staging) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_failure(staging, &error)),
    }
}

fn publication_failure(root: &Path, error: &io::Error) -> Diagnostic {
    match fs::symlink_metadata(root) {
        Ok(_) => Diagnostic::new(
            codes::PACKAGE_OUTPUT_EXISTS,
            format!(
                "`{}` appeared before package publication and was left untouched",
                root.display()
            ),
        )
        .with_repair(RepairClass::WriteToNewOutputPath),
        Err(_) => io_failure(root, error),
    }
}

fn require_package_root(root: &Path) -> Result<(), Diagnostic> {
    let metadata = match fs::symlink_metadata(root) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(manifest_invalid("package root is missing"));
        }
        Err(error) => return Err(io_failure(root, &error)),
    };
    if metadata.file_type().is_dir() {
        Ok(())
    } else {
        Err(entry_type_invalid(
            root,
            "a world package root must be a directory and not a symlink",
        ))
    }
}

fn require_manifest_file(path: &Path) -> Result<(), Diagnostic> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(entry_type_invalid(
            path,
            "the package manifest must be a regular file",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            Err(manifest_invalid("package manifest is missing"))
        }
        Err(error) => Err(io_failure(path, &error)),
    }
}

fn require_member_file(path: &Path, name: &MemberName) -> Result<(), Diagnostic> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => Ok(()),
        Ok(_) => Err(entry_type_invalid(
            path,
            "manifest-declared members must be regular files",
        )),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(Diagnostic::new(
            codes::PACKAGE_MEMBER_MISSING,
            format!("member `{name}` is declared by the manifest but is missing"),
        )
        .with_repair(RepairClass::RebuildFromSource)),
        Err(error) => Err(io_failure(path, &error)),
    }
}

fn entry_type_invalid(path: &Path, reason: &str) -> Diagnostic {
    Diagnostic::new(
        codes::PACKAGE_ENTRY_TYPE_INVALID,
        format!(
            "`{}` has an invalid package entry type: {reason}",
            path.display()
        ),
    )
    .with_repair(RepairClass::RebuildFromSource)
}

fn io_failure(path: &Path, error: &std::io::Error) -> Diagnostic {
    Diagnostic::new(codes::PACKAGE_IO, format!("`{}`: {error}", path.display()))
}

/// Reads a non-negative integer, whichever variant the reader produced for it.
///
/// The canonical byte profile has one spelling for an integer, but the reader
/// hands back `Int` when the value fits `i64` and `Uint` above that, so a
/// decoder that matched only one variant would reject legal canonical bytes.
fn unsigned(value: &CanonicalValue) -> Option<u64> {
    match value {
        CanonicalValue::Int(signed) => u64::try_from(*signed).ok(),
        CanonicalValue::Uint(unsigned) => Some(*unsigned),
        _ => None,
    }
}

fn manifest_invalid(reason: impl Into<String>) -> Diagnostic {
    Diagnostic::new(codes::PACKAGE_MANIFEST_INVALID, reason)
        .with_repair(RepairClass::RebuildFromSource)
}

fn decode_manifest(bytes: &[u8]) -> Result<PackageManifest, Diagnostic> {
    let value = parse_canonical(bytes).map_err(|diagnostic| {
        manifest_invalid(format!("manifest is not canonical: {diagnostic}"))
    })?;
    let CanonicalValue::Object(fields) = &value else {
        return Err(manifest_invalid("manifest is not an object"));
    };
    require_exact_fields(fields, &["members", "package_digest", "schema"], "manifest")?;
    let field = |name: &'static str| fields.get(&FieldName::declared(name));

    let Some(CanonicalValue::Object(schema_fields)) = field("schema") else {
        return Err(manifest_invalid("manifest has no `schema` object"));
    };
    require_exact_fields(schema_fields, &["name", "version"], "manifest schema")?;
    let (Some(CanonicalValue::Text(schema_name)), Some(schema_version)) = (
        schema_fields.get(&FieldName::declared("name")),
        schema_fields
            .get(&FieldName::declared("version"))
            .and_then(unsigned),
    ) else {
        return Err(manifest_invalid(
            "manifest `schema` needs `name` and `version`",
        ));
    };
    let version = u32::try_from(schema_version)
        .map_err(|_| manifest_invalid("manifest schema version is out of range"))?;
    let schema = SchemaId::new(schema_name, version)
        .map_err(|diagnostic| manifest_invalid(diagnostic.message()))?;
    let expected = manifest_schema();
    if schema != expected {
        return Err(manifest_invalid(format!(
            "manifest declares schema `{schema}`; this kernel reads `{expected}`"
        )));
    }

    let Some(CanonicalValue::Array(rows)) = field("members") else {
        return Err(manifest_invalid("manifest has no `members` array"));
    };
    let mut members = Vec::with_capacity(rows.len());
    let mut names = BTreeSet::new();
    let mut previous: Option<MemberName> = None;
    for row in rows {
        let member = decode_member(row)?;
        if !names.insert(member.name.clone()) {
            return Err(manifest_invalid(format!(
                "manifest member `{}` occurs more than once",
                member.name
            )));
        }
        if let Some(prior) = &previous {
            match member.name.cmp(prior) {
                std::cmp::Ordering::Less => {
                    return Err(manifest_invalid(format!(
                        "manifest member `{}` is out of canonical member-name order",
                        member.name
                    )));
                }
                std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => {}
            }
        }
        previous = Some(member.name.clone());
        members.push(member);
    }

    let Some(CanonicalValue::Text(recorded)) = field("package_digest") else {
        return Err(manifest_invalid("manifest has no `package_digest`"));
    };
    let recorded = Sha256Digest::from_hex(recorded).ok_or_else(|| {
        manifest_invalid("`package_digest` is not a lowercase SHA-256 hex digest")
    })?;
    let recomputed = Sha256Digest::of_canonical(&PackageManifest::body(&schema, &members));
    if recorded != recomputed {
        return Err(manifest_invalid(format!(
            "`package_digest` records {recorded} but the manifest body hashes to {recomputed}"
        )));
    }

    Ok(PackageManifest {
        schema,
        members,
        digest: recorded,
    })
}

fn decode_member(row: &CanonicalValue) -> Result<MemberRecord, Diagnostic> {
    let CanonicalValue::Object(fields) = row else {
        return Err(manifest_invalid("a manifest member row is not an object"));
    };
    require_exact_fields(fields, &["name", "sha256", "size"], "manifest member row")?;
    let (Some(CanonicalValue::Text(name)), Some(CanonicalValue::Text(digest)), Some(size)) = (
        fields.get(&FieldName::declared("name")),
        fields.get(&FieldName::declared("sha256")),
        fields.get(&FieldName::declared("size")).and_then(unsigned),
    ) else {
        return Err(manifest_invalid(
            "a manifest member row needs `name`, `sha256`, and `size`",
        ));
    };
    let digest = Sha256Digest::from_hex(digest)
        .ok_or_else(|| manifest_invalid("a member `sha256` is not a lowercase hex digest"))?;
    Ok(MemberRecord {
        name: MemberName::new(name).map_err(|diagnostic| manifest_invalid(diagnostic.message()))?,
        size,
        digest,
    })
}

fn require_exact_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    context: &str,
) -> Result<(), Diagnostic> {
    let exact = fields.len() == expected.len()
        && expected
            .iter()
            .all(|name| fields.keys().any(|field| field.as_str() == *name));
    if exact {
        return Ok(());
    }
    let actual = fields
        .keys()
        .map(FieldName::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    Err(manifest_invalid(format!(
        "{context} fields must be exactly [{}]; found [{actual}]",
        expected.join(", ")
    )))
}

#[cfg(test)]
mod tests {
    use super::{MemberName, WorldPackage};
    use crate::CanonicalValue;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn an_injected_mid_write_failure_cleans_staging_and_never_publishes() {
        let parent = PathBuf::from(option_env!("CARGO_TARGET_TMPDIR").unwrap_or("target/tmp"))
            .join("package-faults")
            .join(std::process::id().to_string());
        fs::create_dir_all(&parent).unwrap();
        let root = parent.join("injected.world");
        let members = ["alpha.json", "beta.json"].map(|name| {
            (
                MemberName::new(name).unwrap(),
                CanonicalValue::object_declared([("value", CanonicalValue::Uint(1))])
                    .to_canonical_bytes(),
            )
        });

        let rejected = WorldPackage::write_internal(&root, members, Some(1)).unwrap_err();
        assert_eq!(rejected.code().as_str(), "EK0407");
        assert!(
            !root.exists(),
            "a partial destination must never be visible"
        );
        assert!(
            fs::read_dir(&parent).unwrap().all(|entry| {
                !entry
                    .unwrap()
                    .file_name()
                    .to_string_lossy()
                    .contains(".staging-")
            }),
            "a failed publication must remove its sibling staging directory"
        );
    }
}
