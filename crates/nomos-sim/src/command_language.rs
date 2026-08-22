//! Exact persisted command language and typed projection resolution.

use std::collections::BTreeMap;

use nomos_core::id::StableId;
use nomos_core::{
    CanonicalValue, CatalogValueId, Diagnostic, EntityId, Ident, NamespaceId, RepairClass,
};
use nomos_projection::{
    Command, CommandArgument, CommandRequirement, MachineDefinition, SimulationPlan,
};

const HEADER: &str = "schema nomos.command_script@1";

/// One user-facing request before compiler-projected namespace resolution.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommandRequest {
    action: Ident,
    entity: EntityId,
    argument: Option<CatalogValueId>,
}

impl CommandRequest {
    /// Builds one typed unresolved request.
    #[must_use]
    pub const fn new(action: Ident, entity: EntityId, argument: Option<CatalogValueId>) -> Self {
        Self {
            action,
            entity,
            argument,
        }
    }

    /// Requested namespace-local action.
    #[must_use]
    pub const fn action(&self) -> &Ident {
        &self.action
    }

    /// Entity whose owned machines are searched.
    #[must_use]
    pub const fn entity(&self) -> &EntityId {
        &self.entity
    }

    /// Optional typed catalog argument.
    #[must_use]
    pub const fn argument(&self) -> Option<&CatalogValueId> {
        self.argument.as_ref()
    }

    /// Canonical semantic value used by later command-log evidence.
    #[must_use]
    pub fn to_canonical(&self) -> CanonicalValue {
        let argument = self
            .argument
            .as_ref()
            .map_or(CanonicalValue::Null, |value| {
                CanonicalValue::object_declared([
                    ("kind", CanonicalValue::text("catalog_value")),
                    ("value", value.to_canonical()),
                ])
            });
        CanonicalValue::object_declared([
            ("action", CanonicalValue::text(self.action.as_str())),
            ("argument", argument),
            ("entity", self.entity.to_canonical()),
        ])
    }

    fn to_line(&self) -> String {
        self.argument.as_ref().map_or_else(
            || format!("{} {}", self.action, self.entity),
            |argument| format!("{} {} with {argument}", self.action, self.entity),
        )
    }
}

/// A strict nonempty command script with one schema header.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CommandScript {
    requests: Vec<CommandRequest>,
}

impl CommandScript {
    /// Parses exact UTF-8 command-script bytes.
    ///
    /// # Errors
    ///
    /// Returns `EK0814` for any encoding, header, whitespace, line-ending,
    /// arity, or typed-identifier disagreement.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let text =
            std::str::from_utf8(bytes).map_err(|_| invalid("command script is not UTF-8 text"))?;
        if !text.ends_with('\n') || text.ends_with("\n\n") || text.contains('\r') {
            return Err(invalid(
                "command script must use LF lines and end in exactly one LF",
            ));
        }
        let mut lines = text[..text.len() - 1].split('\n');
        if lines.next() != Some(HEADER) {
            return Err(invalid(
                "command script has the wrong or missing schema header",
            ));
        }
        let requests = lines.map(parse_request).collect::<Result<Vec<_>, _>>()?;
        if requests.is_empty() {
            return Err(invalid("command script contains no commands"));
        }
        let script = Self { requests };
        if script.to_bytes() != bytes {
            return Err(invalid(
                "command script does not exactly re-encode from its typed meaning",
            ));
        }
        Ok(script)
    }

    /// Requests in authored execution order.
    #[must_use]
    pub fn requests(&self) -> &[CommandRequest] {
        &self.requests
    }

    /// Emits the one accepted script spelling.
    #[must_use]
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut text = String::from(HEADER);
        text.push('\n');
        for request in &self.requests {
            text.push_str(&request.to_line());
            text.push('\n');
        }
        text.into_bytes()
    }
}

/// Resolves one request to exactly one compiler-projected external namespace.
///
/// # Errors
///
/// Returns a stable runtime diagnostic when the entity/action is absent, the
/// action spans multiple owned namespaces, requirements disagree, or the
/// supplied argument does not exactly satisfy the compiled requirement.
pub fn resolve_command(
    plan: &SimulationPlan,
    request: &CommandRequest,
) -> Result<Command, Diagnostic> {
    let entity = plan
        .entities()
        .iter()
        .find(|entity| entity.id() == request.entity())
        .ok_or_else(|| {
            Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_TARGET_MISSING,
                format!("command entity `{}` does not exist", request.entity()),
            )
        })?;
    let machines = plan
        .machines()
        .iter()
        .filter(|machine| entity.machines().contains(machine.namespace()))
        .collect::<Vec<_>>();
    let mut candidates = BTreeMap::<NamespaceId, CommandRequirement>::new();
    for machine in machines {
        if let Some(requirement) = action_requirement(machine, request.action())? {
            candidates.insert(machine.namespace().clone(), requirement);
        }
    }
    if candidates.len() != 1 {
        return if candidates.len() > 1 {
            Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_COMMAND_AMBIGUOUS,
                format!(
                    "command `{}` is exposed by multiple machines owned by `{}`",
                    request.action(),
                    request.entity()
                ),
            ))
        } else {
            Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_ACTION_UNDECLARED,
                format!(
                    "entity `{}` exposes no external command `{}`",
                    request.entity(),
                    request.action()
                ),
            ))
        };
    }
    let (namespace, requirement) = candidates
        .into_iter()
        .next()
        .expect("the candidate count was exactly one");
    let argument = match (&requirement, request.argument()) {
        (CommandRequirement::None, None) => CommandArgument::None,
        (CommandRequirement::Credential(required), Some(supplied)) if required == supplied => {
            CommandArgument::Credential(supplied.clone())
        }
        _ => {
            return Err(Diagnostic::new(
                nomos_core::diagnostic::codes::RUNTIME_ARGUMENT_MISMATCH,
                "command argument does not match the compiled input requirement",
            ));
        }
    };
    Ok(Command::new(namespace, request.action().clone(), argument))
}

fn action_requirement(
    machine: &MachineDefinition,
    action: &Ident,
) -> Result<Option<CommandRequirement>, Diagnostic> {
    let mut requirements = machine
        .commands()
        .iter()
        .filter(|transition| transition.action() == action)
        .map(|transition| transition.requirement().clone());
    let Some(first) = requirements.next() else {
        return Ok(None);
    };
    if requirements.any(|requirement| requirement != first) {
        return Err(Diagnostic::new(
            nomos_core::diagnostic::codes::RUNTIME_STATE_INVALID,
            format!(
                "machine `{}` gives action `{action}` inconsistent input requirements",
                machine.namespace()
            ),
        ));
    }
    Ok(Some(first))
}

fn parse_request(line: &str) -> Result<CommandRequest, Diagnostic> {
    if line.is_empty()
        || line.starts_with(' ')
        || line.ends_with(' ')
        || line.contains("  ")
        || line.contains('\t')
    {
        return Err(invalid("command line uses unsupported whitespace"));
    }
    let tokens = line.split(' ').collect::<Vec<_>>();
    let (action, entity, argument) = match tokens.as_slice() {
        [action, entity] => (*action, *entity, None),
        [action, entity, "with", argument] => (*action, *entity, Some(*argument)),
        _ => {
            return Err(invalid(
                "command line has unsupported arity or argument syntax",
            ));
        }
    };
    let action = Ident::new(action).map_err(|error| invalid(error.message()))?;
    let entity = EntityId::parse(entity).map_err(|error| invalid(error.message()))?;
    let argument = argument
        .map(CatalogValueId::parse)
        .transpose()
        .map_err(|error| invalid(error.message()))?;
    Ok(CommandRequest::new(action, entity, argument))
}

fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        nomos_core::diagnostic::codes::RUNTIME_COMMAND_SCRIPT_INVALID,
        message,
    )
    .with_repair(RepairClass::FixSourceSyntax)
}
