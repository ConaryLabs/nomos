//! Strict persisted runtime-state evidence.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{
    CanonicalValue, Diagnostic, EntityId, FieldName, Ident, NamespaceId, RepairClass, SchemaId,
    Sha256Digest, StateHash,
};
use nomos_projection::{LatticeCell, ProjectedDirection, RuntimeBinding, SimulationPlan};

use crate::{RuntimeEntityState, SimulationState};

/// One standalone runtime state bound to the complete simulation semantics
/// under which it was produced.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PersistedRuntimeState {
    schema: SchemaId,
    runtime_semantics_digest: Sha256Digest,
    state: SimulationState,
    state_hash: StateHash,
}

impl PersistedRuntimeState {
    /// Binds a validated state to the exact canonical simulation projection.
    ///
    /// # Errors
    ///
    /// Returns `EK0809` when the state does not conform to the supplied plan.
    pub fn new(plan: &SimulationPlan, state: SimulationState) -> Result<Self, Diagnostic> {
        state.validate_against(plan)?;
        Ok(Self {
            schema: crate::persisted_runtime_state_schema(),
            runtime_semantics_digest: simulation_digest(plan),
            state_hash: state.state_hash(),
            state,
        })
    }

    /// Strictly reconstructs and verifies one persisted state envelope.
    ///
    /// # Errors
    ///
    /// Returns the first canonical, schema, state-hash, plan, or runtime-
    /// semantics disagreement diagnostic.
    pub fn from_canonical_bytes(bytes: &[u8], plan: &SimulationPlan) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes)?;
        let fields = object(&value, "persisted runtime state")?;
        require_fields(
            fields,
            &["runtime_semantics_digest", "schema", "state", "state_hash"],
            "persisted runtime state",
        )?;
        let schema = schema(field(fields, "schema")?, "persisted runtime state schema")?;
        if schema != crate::persisted_runtime_state_schema() {
            return Err(invalid("persisted state names an unsupported schema"));
        }
        let runtime_semantics_digest = digest(
            field(fields, "runtime_semantics_digest")?,
            "runtime semantics digest",
        )?;
        if runtime_semantics_digest != simulation_digest(plan) {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_SEMANTICS_MISMATCH,
                "persisted state belongs to different simulation semantics",
            )
            .with_repair(RepairClass::RebuildFromSource));
        }
        let state_bytes = field(fields, "state")?.to_canonical_bytes();
        let state = decode_state(&state_bytes, plan)?;
        let state_hash = state_hash(field(fields, "state_hash")?)?;
        state.verify_hash(state_hash)?;
        let persisted = Self {
            schema,
            runtime_semantics_digest,
            state,
            state_hash,
        };
        if persisted.to_canonical_bytes() != bytes {
            return Err(invalid(
                "persisted state does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(persisted)
    }

    /// Exact simulation-projection digest binding this state file.
    #[must_use]
    pub const fn runtime_semantics_digest(&self) -> Sha256Digest {
        self.runtime_semantics_digest
    }

    /// Verified authoritative runtime snapshot.
    #[must_use]
    pub const fn state(&self) -> &SimulationState {
        &self.state
    }

    /// Verified hash of the inner runtime-state envelope only.
    #[must_use]
    pub const fn state_hash(&self) -> StateHash {
        self.state_hash
    }

    /// Exact canonical persisted bytes.
    #[must_use]
    pub fn to_canonical_bytes(&self) -> Vec<u8> {
        CanonicalValue::object_declared([
            (
                "runtime_semantics_digest",
                CanonicalValue::text(self.runtime_semantics_digest.to_hex()),
            ),
            ("schema", self.schema.to_canonical()),
            ("state", self.state.to_canonical()),
            ("state_hash", CanonicalValue::text(self.state_hash.to_hex())),
        ])
        .to_canonical_bytes()
    }
}

pub(crate) fn decode_state(
    bytes: &[u8],
    plan: &SimulationPlan,
) -> Result<SimulationState, Diagnostic> {
    let value = parse_canonical(bytes)?;
    let fields = object(&value, "runtime state")?;
    require_fields(
        fields,
        &[
            "counters",
            "entities",
            "machines",
            "scheduled_events",
            "schema",
            "tick",
        ],
        "runtime state",
    )?;
    let schema = schema(field(fields, "schema")?, "runtime-state schema")?;
    if schema != crate::runtime_state_schema() {
        return Err(invalid("runtime state names an unsupported schema"));
    }
    require_empty_array(field(fields, "counters")?, "runtime counters")?;
    require_empty_array(
        field(fields, "scheduled_events")?,
        "scheduled runtime events",
    )?;
    let tick = uint(field(fields, "tick")?, "runtime tick")?;
    let entities = decode_entities(field(fields, "entities")?)?;
    let machines = decode_machines(field(fields, "machines")?)?;
    let state = SimulationState::from_parts(schema, tick, entities, machines);
    state.validate_against(plan)?;
    if state.to_canonical_bytes() != bytes {
        return Err(invalid(
            "runtime state does not exactly re-encode from its typed meaning",
        ));
    }
    Ok(state)
}

fn decode_entities(value: &CanonicalValue) -> Result<Vec<RuntimeEntityState>, Diagnostic> {
    let rows = array(value, "runtime entities")?;
    let mut entities = Vec::with_capacity(rows.len());
    let mut seen = BTreeSet::new();
    for row in rows {
        let fields = object(row, "runtime entity")?;
        require_fields(fields, &["binding", "id"], "runtime entity")?;
        let id = EntityId::parse(text(field(fields, "id")?, "runtime entity id")?)
            .map_err(|error| invalid(error.message()))?;
        if !seen.insert(id.clone()) {
            return Err(invalid("runtime entity identity occurs more than once"));
        }
        entities.push(RuntimeEntityState::from_parts(
            id,
            binding(field(fields, "binding")?)?,
        ));
    }
    Ok(entities)
}

fn decode_machines(value: &CanonicalValue) -> Result<BTreeMap<NamespaceId, Ident>, Diagnostic> {
    let rows = array(value, "runtime machines")?;
    let mut machines = BTreeMap::new();
    for row in rows {
        let fields = object(row, "runtime machine")?;
        require_fields(fields, &["namespace", "state"], "runtime machine")?;
        let namespace = NamespaceId::parse(text(
            field(fields, "namespace")?,
            "runtime machine namespace",
        )?)
        .map_err(|error| invalid(error.message()))?;
        let state = Ident::new(text(field(fields, "state")?, "runtime machine state")?)
            .map_err(|error| invalid(error.message()))?;
        if machines.insert(namespace, state).is_some() {
            return Err(invalid("runtime machine namespace occurs more than once"));
        }
    }
    Ok(machines)
}

fn binding(value: &CanonicalValue) -> Result<RuntimeBinding, Diagnostic> {
    let fields = object(value, "runtime binding")?;
    match text(field(fields, "kind")?, "runtime binding kind")? {
        "cell" => {
            require_fields(fields, &["cell", "kind"], "cell binding")?;
            Ok(RuntimeBinding::Cell(cell(field(fields, "cell")?)?))
        }
        "face" => {
            require_fields(fields, &["cell", "direction", "kind"], "face binding")?;
            let direction = match text(field(fields, "direction")?, "face direction")? {
                "north" => ProjectedDirection::North,
                "east" => ProjectedDirection::East,
                "south" => ProjectedDirection::South,
                "west" => ProjectedDirection::West,
                "up" => ProjectedDirection::Up,
                "down" => ProjectedDirection::Down,
                _ => return Err(invalid("runtime face direction is unsupported")),
            };
            Ok(RuntimeBinding::Face {
                cell: cell(field(fields, "cell")?)?,
                direction,
            })
        }
        "region" => {
            require_fields(fields, &["kind", "max", "min"], "region binding")?;
            Ok(RuntimeBinding::Region {
                min: cell(field(fields, "min")?)?,
                max: cell(field(fields, "max")?)?,
            })
        }
        _ => Err(invalid("runtime binding kind is unsupported")),
    }
}

fn cell(value: &CanonicalValue) -> Result<LatticeCell, Diagnostic> {
    let fields = object(value, "lattice cell")?;
    require_fields(fields, &["x", "y", "z"], "lattice cell")?;
    Ok(LatticeCell::new(
        int32(field(fields, "x")?, "cell x")?,
        int32(field(fields, "y")?, "cell y")?,
        int32(field(fields, "z")?, "cell z")?,
    ))
}

fn simulation_digest(plan: &SimulationPlan) -> Sha256Digest {
    Sha256Digest::of_bytes(&plan.to_canonical_bytes())
}

pub(crate) fn schema(value: &CanonicalValue, label: &str) -> Result<SchemaId, Diagnostic> {
    let fields = object(value, label)?;
    require_fields(fields, &["name", "version"], label)?;
    let version = u32::try_from(uint(field(fields, "version")?, label)?)
        .map_err(|_| invalid(format!("{label} version exceeds u32")))?;
    SchemaId::new(text(field(fields, "name")?, label)?, version)
        .map_err(|error| invalid(error.message()))
}

pub(crate) fn digest(value: &CanonicalValue, label: &str) -> Result<Sha256Digest, Diagnostic> {
    Sha256Digest::from_hex(text(value, label)?)
        .ok_or_else(|| invalid(format!("{label} is invalid")))
}

pub(crate) fn state_hash(value: &CanonicalValue) -> Result<StateHash, Diagnostic> {
    StateHash::from_hex(text(value, "state hash")?).ok_or_else(|| invalid("state hash is invalid"))
}

fn int32(value: &CanonicalValue, label: &str) -> Result<i32, Diagnostic> {
    let value = match value {
        CanonicalValue::Int(value) => *value,
        CanonicalValue::Uint(value) => i64::try_from(*value)
            .map_err(|_| invalid(format!("{label} exceeds signed integer range")))?,
        _ => return Err(invalid(format!("{label} is not an integer"))),
    };
    i32::try_from(value).map_err(|_| invalid(format!("{label} exceeds i32")))
}

pub(crate) fn uint(value: &CanonicalValue, label: &str) -> Result<u64, Diagnostic> {
    match value {
        CanonicalValue::Uint(value) => Ok(*value),
        CanonicalValue::Int(value) => {
            u64::try_from(*value).map_err(|_| invalid(format!("{label} is negative")))
        }
        _ => Err(invalid(format!("{label} is not an unsigned integer"))),
    }
}

fn require_empty_array(value: &CanonicalValue, label: &str) -> Result<(), Diagnostic> {
    if array(value, label)?.is_empty() {
        Ok(())
    } else {
        Err(invalid(format!("{label} are unsupported in Gate K")))
    }
}

pub(crate) fn object<'a>(
    value: &'a CanonicalValue,
    label: &str,
) -> Result<&'a BTreeMap<FieldName, CanonicalValue>, Diagnostic> {
    let CanonicalValue::Object(fields) = value else {
        return Err(invalid(format!("{label} is not an object")));
    };
    Ok(fields)
}

pub(crate) fn array<'a>(
    value: &'a CanonicalValue,
    label: &str,
) -> Result<&'a [CanonicalValue], Diagnostic> {
    let CanonicalValue::Array(values) = value else {
        return Err(invalid(format!("{label} is not an array")));
    };
    Ok(values)
}

pub(crate) fn text<'a>(value: &'a CanonicalValue, label: &str) -> Result<&'a str, Diagnostic> {
    let CanonicalValue::Text(value) = value else {
        return Err(invalid(format!("{label} is not text")));
    };
    Ok(value)
}

pub(crate) fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| invalid(format!("persisted runtime value has no `{name}` field")))
}

pub(crate) fn require_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    label: &str,
) -> Result<(), Diagnostic> {
    let actual = fields.keys().map(FieldName::as_str).collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    expected.sort_unstable();
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "{label} fields are {actual:?}; expected {expected:?}"
        )))
    }
}

pub(crate) fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_PERSISTED_INVALID,
        message,
    )
    .with_repair(RepairClass::RebuildFromSource)
}
