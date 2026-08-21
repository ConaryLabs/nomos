//! Typed runtime initialization material decoded from `simulation.json`.

use std::collections::{BTreeMap, BTreeSet};

use nomos_core::canonical::read::parse_canonical;
use nomos_core::{
    CanonicalValue, Diagnostic, EntityId, FieldName, Ident, NamespaceId, RepairClass, SchemaId,
};

use crate::{
    LatticeCell, ProjectedDirection, ProjectedEntity, RuntimeBinding, SimulationPlan,
    simulation_schema,
};

/// One namespace machine and its validated initial state.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct InitialMachine {
    namespace: NamespaceId,
    states: Vec<Ident>,
    initial: Ident,
}

impl InitialMachine {
    fn new(
        namespace: NamespaceId,
        mut states: Vec<Ident>,
        initial: Ident,
    ) -> Result<Self, Diagnostic> {
        let mut seen = BTreeSet::new();
        for state in &states {
            if !seen.insert(state.clone()) {
                return Err(invalid("simulation machine repeats a state"));
            }
        }
        if !seen.contains(&initial) {
            return Err(invalid("simulation machine initial state is not declared"));
        }
        states.sort();
        Ok(Self {
            namespace,
            states,
            initial,
        })
    }

    /// Namespace identity.
    #[must_use]
    pub const fn namespace(&self) -> &NamespaceId {
        &self.namespace
    }

    /// Legal states in stable order.
    #[must_use]
    pub fn states(&self) -> &[Ident] {
        &self.states
    }

    /// Initial state.
    #[must_use]
    pub const fn initial(&self) -> &Ident {
        &self.initial
    }
}

/// Minimal typed material sufficient to construct `nomos.runtime_state@1`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SimulationInitialization {
    schema: SchemaId,
    entities: Vec<ProjectedEntity>,
    machines: Vec<InitialMachine>,
}

impl SimulationInitialization {
    /// Extracts initialization material from an already typed plan.
    #[must_use]
    pub fn from_plan(plan: &SimulationPlan) -> Self {
        Self {
            schema: plan.schema().clone(),
            entities: plan.entities().to_vec(),
            machines: plan
                .machines()
                .iter()
                .map(|machine| InitialMachine {
                    namespace: machine.namespace().clone(),
                    states: machine.states().to_vec(),
                    initial: machine.initial().clone(),
                })
                .collect(),
        }
    }

    /// Decodes the initialization subset from exact canonical simulation bytes.
    ///
    /// # Errors
    ///
    /// Returns `EK0412` for malformed fields, identities, bindings, duplicate
    /// rows, invalid initial states, or the wrong simulation schema.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes).map_err(|error| invalid(error.message()))?;
        let fields = object(&value, "simulation projection")?;
        exact_fields(
            fields,
            &[
                "causal_edges",
                "entities",
                "light_resolver",
                "machines",
                "movement_resolver",
                "schema",
            ],
            "simulation projection",
        )?;
        if field(fields, "schema", "simulation projection")?.to_canonical_bytes()
            != simulation_schema().to_canonical().to_canonical_bytes()
        {
            return Err(invalid("simulation projection has the wrong schema"));
        }
        let CanonicalValue::Array(entity_rows) =
            field(fields, "entities", "simulation projection")?
        else {
            return Err(invalid("simulation entities are not an array"));
        };
        let mut entities = Vec::new();
        for row in entity_rows {
            entities.push(decode_entity(row)?);
        }
        entities.sort_by(|left, right| left.id().cmp(right.id()));
        reject_duplicates(
            entities.iter().map(|entity| entity.id().to_string()),
            "entity",
        )?;

        let CanonicalValue::Array(machine_rows) =
            field(fields, "machines", "simulation projection")?
        else {
            return Err(invalid("simulation machines are not an array"));
        };
        let mut machines = Vec::new();
        for row in machine_rows {
            machines.push(decode_machine(row)?);
        }
        machines.sort_by(|left, right| left.namespace.cmp(&right.namespace));
        reject_duplicates(
            machines.iter().map(|machine| machine.namespace.to_string()),
            "machine namespace",
        )?;

        let projected = entities
            .iter()
            .flat_map(|entity| entity.machines().iter().cloned())
            .collect::<BTreeSet<_>>();
        let defined = machines
            .iter()
            .map(|machine| machine.namespace.clone())
            .collect::<BTreeSet<_>>();
        if projected != defined {
            return Err(invalid(
                "projected entity machine ownership does not match initialization machines",
            ));
        }
        Ok(Self {
            schema: simulation_schema(),
            entities,
            machines,
        })
    }

    /// Simulation schema carried by the material.
    #[must_use]
    pub const fn schema(&self) -> &SchemaId {
        &self.schema
    }

    /// Runtime entities and authoritative bindings.
    #[must_use]
    pub fn entities(&self) -> &[ProjectedEntity] {
        &self.entities
    }

    /// Machines and their initial states.
    #[must_use]
    pub fn machines(&self) -> &[InitialMachine] {
        &self.machines
    }
}

fn decode_entity(value: &CanonicalValue) -> Result<ProjectedEntity, Diagnostic> {
    let fields = object(value, "simulation entity")?;
    exact_fields(fields, &["binding", "id", "machines"], "simulation entity")?;
    let id = EntityId::parse(text(
        field(fields, "id", "simulation entity")?,
        "entity id",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let binding = decode_binding(field(fields, "binding", "simulation entity")?)?;
    let CanonicalValue::Array(machine_values) = field(fields, "machines", "simulation entity")?
    else {
        return Err(invalid("entity machine identities are not an array"));
    };
    let machines = machine_values
        .iter()
        .map(|value| {
            NamespaceId::parse(text(value, "machine namespace")?)
                .map_err(|error| invalid(error.message()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    ProjectedEntity::new(id, binding, machines).map_err(|error| invalid(error.message()))
}

fn decode_machine(value: &CanonicalValue) -> Result<InitialMachine, Diagnostic> {
    let fields = object(value, "simulation machine")?;
    exact_fields(
        fields,
        &["commands", "handlers", "initial", "namespace", "states"],
        "simulation machine",
    )?;
    if !matches!(
        field(fields, "commands", "simulation machine")?,
        CanonicalValue::Array(_)
    ) || !matches!(
        field(fields, "handlers", "simulation machine")?,
        CanonicalValue::Array(_)
    ) {
        return Err(invalid("simulation commands and handlers must be arrays"));
    }
    let namespace = NamespaceId::parse(text(
        field(fields, "namespace", "simulation machine")?,
        "machine namespace",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let initial = Ident::new(text(
        field(fields, "initial", "simulation machine")?,
        "machine initial state",
    )?)
    .map_err(|error| invalid(error.message()))?;
    let CanonicalValue::Array(state_values) = field(fields, "states", "simulation machine")? else {
        return Err(invalid("simulation machine states are not an array"));
    };
    let states = state_values
        .iter()
        .map(|value| {
            Ident::new(text(value, "machine state")?).map_err(|error| invalid(error.message()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    InitialMachine::new(namespace, states, initial)
}

fn decode_binding(value: &CanonicalValue) -> Result<RuntimeBinding, Diagnostic> {
    let fields = object(value, "runtime binding")?;
    let kind = text(field(fields, "kind", "runtime binding")?, "binding kind")?;
    match kind {
        "cell" => {
            exact_fields(fields, &["cell", "kind"], "cell binding")?;
            Ok(RuntimeBinding::Cell(decode_cell(field(
                fields,
                "cell",
                "cell binding",
            )?)?))
        }
        "face" => {
            exact_fields(fields, &["cell", "direction", "kind"], "face binding")?;
            let direction = match text(
                field(fields, "direction", "face binding")?,
                "face direction",
            )? {
                "north" => ProjectedDirection::North,
                "east" => ProjectedDirection::East,
                "south" => ProjectedDirection::South,
                "west" => ProjectedDirection::West,
                "up" => ProjectedDirection::Up,
                "down" => ProjectedDirection::Down,
                _ => return Err(invalid("face binding has an unsupported direction")),
            };
            Ok(RuntimeBinding::Face {
                cell: decode_cell(field(fields, "cell", "face binding")?)?,
                direction,
            })
        }
        "region" => {
            exact_fields(fields, &["kind", "max", "min"], "region binding")?;
            let min = decode_cell(field(fields, "min", "region binding")?)?;
            let max = decode_cell(field(fields, "max", "region binding")?)?;
            if min.x() > max.x() || min.y() > max.y() || min.z() > max.z() {
                return Err(invalid("region binding has inverted bounds"));
            }
            Ok(RuntimeBinding::Region { min, max })
        }
        _ => Err(invalid("runtime binding has an unsupported kind")),
    }
}

fn decode_cell(value: &CanonicalValue) -> Result<LatticeCell, Diagnostic> {
    let fields = object(value, "lattice cell")?;
    exact_fields(fields, &["x", "y", "z"], "lattice cell")?;
    Ok(LatticeCell::new(
        coordinate(field(fields, "x", "lattice cell")?)?,
        coordinate(field(fields, "y", "lattice cell")?)?,
        coordinate(field(fields, "z", "lattice cell")?)?,
    ))
}

fn coordinate(value: &CanonicalValue) -> Result<i32, Diagnostic> {
    let integer = match value {
        CanonicalValue::Int(value) => *value,
        CanonicalValue::Uint(value) => {
            i64::try_from(*value).map_err(|_| invalid("cell coordinate is out of range"))?
        }
        _ => return Err(invalid("cell coordinate is not an integer")),
    };
    i32::try_from(integer).map_err(|_| invalid("cell coordinate is out of range"))
}

fn object<'a>(
    value: &'a CanonicalValue,
    context: &str,
) -> Result<&'a BTreeMap<FieldName, CanonicalValue>, Diagnostic> {
    let CanonicalValue::Object(fields) = value else {
        return Err(invalid(format!("{context} is not an object")));
    };
    Ok(fields)
}

fn field<'a>(
    fields: &'a BTreeMap<FieldName, CanonicalValue>,
    name: &'static str,
    context: &str,
) -> Result<&'a CanonicalValue, Diagnostic> {
    fields
        .get(&FieldName::declared(name))
        .ok_or_else(|| invalid(format!("{context} has no `{name}`")))
}

fn text<'a>(value: &'a CanonicalValue, context: &str) -> Result<&'a str, Diagnostic> {
    let CanonicalValue::Text(value) = value else {
        return Err(invalid(format!("{context} is not text")));
    };
    Ok(value)
}

fn exact_fields(
    fields: &BTreeMap<FieldName, CanonicalValue>,
    expected: &[&str],
    context: &str,
) -> Result<(), Diagnostic> {
    let actual = fields
        .keys()
        .map(FieldName::as_str)
        .collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if actual == expected {
        Ok(())
    } else {
        Err(invalid(format!(
            "{context} fields are {actual:?}; expected {expected:?}"
        )))
    }
}

fn reject_duplicates(
    values: impl IntoIterator<Item = String>,
    identity: &str,
) -> Result<(), Diagnostic> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(value.clone()) {
            return Err(invalid(format!("simulation repeats {identity} `{value}`")));
        }
    }
    Ok(())
}

fn invalid(reason: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::PACKAGE_MEMBER_SCHEMA_INVALID,
        reason,
    )
    .with_repair(RepairClass::RebuildFromSource)
}
