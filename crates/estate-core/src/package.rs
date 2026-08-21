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
//!   receipts/
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
//! - **Canonical members.** Every member is checked against the canonical byte
//!   profile before the directory is created, so a package cannot hold a member
//!   that would hash differently after a round trip.
//! - **Verified reads.** [`WorldPackage::open`] recomputes the manifest digest
//!   and every member's size and hash, and refuses any file in the package root
//!   that the manifest does not declare.

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use crate::canonical::read::parse_canonical;
use crate::canonical::{CanonicalValue, FieldName};
use crate::diagnostic::{Diagnostic, RepairClass, codes};
use crate::hash::Sha256Digest;
use crate::id::SchemaId;

/// The manifest file name.
pub const MANIFEST_FILE: &str = "manifest.json";

/// The receipts subdirectory required by section 5.
pub const RECEIPTS_DIR: &str = "receipts";

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
    /// or when it collides with the manifest or receipts entries.
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
    /// Members are validated as canonical bytes first, then the directory is
    /// created. `receipts/` is created empty; its contents belong to the
    /// runtime slice that produces receipts.
    ///
    /// # Errors
    ///
    /// - `EK0401` when `root` already exists. This is acceptance 12: a package
    ///   is evidence and is never written over.
    /// - `EK0302` or `EK0303` when a member's bytes are not canonical.
    /// - `EK0407` when the filesystem refuses the write.
    pub fn write(
        root: &Path,
        members: impl IntoIterator<Item = (MemberName, Vec<u8>)>,
    ) -> Result<Self, Diagnostic> {
        if root.exists() {
            return Err(Diagnostic::new(
                codes::PACKAGE_OUTPUT_EXISTS,
                format!(
                    "`{}` already exists; compiled packages are immutable evidence \
                     and are never written over",
                    root.display()
                ),
            )
            .with_repair(RepairClass::WriteToNewOutputPath));
        }

        let members: BTreeMap<MemberName, Vec<u8>> = members.into_iter().collect();
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

        fs::create_dir_all(root).map_err(|error| io_failure(root, &error))?;
        fs::create_dir(root.join(RECEIPTS_DIR))
            .map_err(|error| io_failure(&root.join(RECEIPTS_DIR), &error))?;
        for (name, bytes) in &members {
            let path = root.join(name.as_str());
            fs::write(&path, bytes).map_err(|error| io_failure(&path, &error))?;
        }
        let manifest_path = root.join(MANIFEST_FILE);
        fs::write(&manifest_path, manifest.to_canonical().to_canonical_bytes())
            .map_err(|error| io_failure(&manifest_path, &error))?;

        Ok(Self {
            root: root.to_path_buf(),
            manifest,
            members,
        })
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
    pub fn open(root: &Path) -> Result<Self, Diagnostic> {
        let manifest_path = root.join(MANIFEST_FILE);
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
            let name = file_name.to_string_lossy();
            if name == MANIFEST_FILE || name == RECEIPTS_DIR {
                continue;
            }
            let declared_here = MemberName::new(&name)
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
            let bytes = fs::read(&path).map_err(|_| {
                Diagnostic::new(
                    codes::PACKAGE_MEMBER_MISSING,
                    format!(
                        "member `{}` is declared by the manifest but could not be read",
                        record.name
                    ),
                )
                .with_repair(RepairClass::RebuildFromSource)
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
    let field = |name: &'static str| fields.get(&FieldName::declared(name));

    let Some(CanonicalValue::Object(schema_fields)) = field("schema") else {
        return Err(manifest_invalid("manifest has no `schema` object"));
    };
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
    for row in rows {
        members.push(decode_member(row)?);
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
