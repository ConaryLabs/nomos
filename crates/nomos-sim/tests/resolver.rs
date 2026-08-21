//! SW-E runtime proof for projected claim activation and movement coherence.

use nomos_core::{ClaimRef, EntityId, Ident, NamespaceId, SourcePath, SourceSpan};
use nomos_projection::{
    Command, CommandArgument, CommandRequirement, CommandTransition, LatticeCell,
    MachineDefinition, MovementClaim, MovementConnectivity, MovementDisposition,
    MovementResolverPlan, MovementSubject, ProjectedActivation, SimulationPlan,
};
use nomos_sim::{SimulationState, prepare_transaction, resolve_movement};

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn entity(value: &str) -> EntityId {
    EntityId::parse(value).unwrap()
}

fn namespace(entity: &str, local: &str) -> NamespaceId {
    NamespaceId::new(self::entity(entity), ident(local))
}

fn claim(local: &str, capability: &str) -> ClaimRef {
    ClaimRef::new(namespace("subject", local), ident(capability))
}

#[test]
fn activation_truth_table_maximum_cost_and_blocker_precedence_are_projected() {
    let plan = activation_plan();
    let initial = SimulationState::initialize(&plan).unwrap();
    let initial_facts = resolve_movement(&plan, &initial).unwrap();
    let initial_disposition = initial_facts.get(&entity("subject")).unwrap();
    assert!(matches!(
        initial_disposition,
        MovementDisposition::Traversable { cost: 3, reasons }
            if reasons == &[claim("cost_high", "traversal_cost_ground")]
    ));

    let prepared = prepare_transaction(
        &plan,
        &initial,
        &Command::new(
            namespace("subject", "machine"),
            ident("turn_on"),
            CommandArgument::None,
        ),
    )
    .unwrap();
    assert_eq!(prepared.movement_before(), &initial_facts);
    assert!(matches!(
        prepared.movement_after().get(&entity("subject")).unwrap(),
        MovementDisposition::Blocked { reasons }
            if reasons == &[claim("blocker", "blocks_ground")]
    ));
}

#[test]
fn blocked_reasons_are_sorted_and_costs_are_ignored() {
    let claims = vec![
        MovementClaim::traversal_cost(
            claim("cost", "traversal_cost_ground"),
            ProjectedActivation::Always,
            99,
            span(),
        )
        .unwrap(),
        MovementClaim::blocker(
            claim("zeta", "blocks_ground"),
            ProjectedActivation::Always,
            true,
            span(),
        ),
        MovementClaim::blocker(
            claim("alpha", "blocks_ground"),
            ProjectedActivation::Always,
            true,
            span(),
        ),
    ];
    let plan = plan_with(Vec::new(), claims);
    let state = SimulationState::initialize(&plan).unwrap();
    assert!(matches!(
        resolve_movement(&plan, &state)
            .unwrap()
            .get(&entity("subject"))
            .unwrap(),
        MovementDisposition::Blocked { reasons }
            if reasons == &[
                claim("alpha", "blocks_ground"),
                claim("zeta", "blocks_ground"),
            ]
    ));
}

#[test]
fn no_active_cost_uses_the_positive_compiler_base_cost() {
    let plan = plan_with(Vec::new(), Vec::new());
    let state = SimulationState::initialize(&plan).unwrap();
    assert!(matches!(
        resolve_movement(&plan, &state)
            .unwrap()
            .get(&entity("subject"))
            .unwrap(),
        MovementDisposition::Traversable { cost: 1, reasons } if reasons.is_empty()
    ));
}

#[test]
fn missing_activation_references_fail_instead_of_becoming_false() {
    let projected_claim = MovementClaim::blocker(
        claim("blocker", "blocks_ground"),
        ProjectedActivation::Any(vec![
            ProjectedActivation::Always,
            ProjectedActivation::StateEquals {
                namespace: namespace("subject", "missing"),
                state: ident("on"),
            },
        ]),
        true,
        span(),
    );
    let plan = plan_with(Vec::new(), vec![projected_claim]);
    let state = SimulationState::initialize(&plan).unwrap();
    let before = state.to_canonical_bytes();
    let original = state.clone();
    let rejected = prepare_transaction(
        &plan,
        &state,
        &Command::new(
            namespace("subject", "missing"),
            ident("anything"),
            CommandArgument::None,
        ),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0908");
    assert_eq!(state, original);
    assert_eq!(state.to_canonical_bytes(), before);
}

#[test]
fn missing_activation_states_fail_instead_of_becoming_false() {
    let machine_namespace = namespace("subject", "machine");
    let machine = MachineDefinition::new(
        machine_namespace.clone(),
        vec![ident("off"), ident("on")],
        ident("off"),
        vec![CommandTransition::new(
            ident("turn_on"),
            CommandRequirement::None,
            ident("off"),
            ident("on"),
        )],
        Vec::new(),
    )
    .unwrap();
    let projected_claim = MovementClaim::blocker(
        claim("blocker", "blocks_ground"),
        ProjectedActivation::All(vec![
            ProjectedActivation::StateEquals {
                namespace: machine_namespace.clone(),
                state: ident("on"),
            },
            ProjectedActivation::StateEquals {
                namespace: machine_namespace.clone(),
                state: ident("missing"),
            },
        ]),
        true,
        span(),
    );
    let plan = plan_with(vec![machine], vec![projected_claim]);
    let state = SimulationState::initialize(&plan).unwrap();
    let before = state.to_canonical_bytes();
    let rejected = prepare_transaction(
        &plan,
        &state,
        &Command::new(machine_namespace, ident("turn_on"), CommandArgument::None),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0908");
    assert_eq!(state.to_canonical_bytes(), before);
}

#[test]
fn disposition_and_plan_invariants_reject_zero_or_empty_values() {
    assert_eq!(
        MovementClaim::traversal_cost(
            claim("cost", "traversal_cost_ground"),
            ProjectedActivation::Always,
            0,
            span(),
        )
        .unwrap_err()
        .code()
        .as_str(),
        "EK0909"
    );
    assert_eq!(
        MovementDisposition::blocked(Vec::new())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0909"
    );
    assert_eq!(
        MovementDisposition::traversable(0, Vec::new())
            .unwrap_err()
            .code()
            .as_str(),
        "EK0909"
    );
}

#[test]
fn invalid_projected_connectivity_fails_atomically() {
    let machine_namespace = namespace("subject", "machine");
    let machine = MachineDefinition::new(
        machine_namespace.clone(),
        vec![ident("off"), ident("on")],
        ident("off"),
        vec![CommandTransition::new(
            ident("turn_on"),
            CommandRequirement::None,
            ident("off"),
            ident("on"),
        )],
        Vec::new(),
    )
    .unwrap();
    let subject = MovementSubject::new(
        entity("subject"),
        MovementConnectivity::Region {
            min: LatticeCell::new(2, 0, 0),
            max: LatticeCell::new(1, 0, 0),
        },
        Vec::new(),
    )
    .unwrap();
    let resolver =
        MovementResolverPlan::new(ident("ground"), 1, true, true, true, true, vec![subject])
            .unwrap();
    let plan = SimulationPlan::new(vec![machine], Vec::new())
        .unwrap()
        .with_movement_resolver(resolver);
    let state = SimulationState::initialize(&plan).unwrap();
    let before = state.to_canonical_bytes();
    let rejected = prepare_transaction(
        &plan,
        &state,
        &Command::new(machine_namespace, ident("turn_on"), CommandArgument::None),
    )
    .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0906");
    assert_eq!(state.to_canonical_bytes(), before);
}

fn activation_plan() -> SimulationPlan {
    let machine_namespace = namespace("subject", "machine");
    let machine = MachineDefinition::new(
        machine_namespace.clone(),
        vec![ident("off"), ident("on")],
        ident("off"),
        vec![CommandTransition::new(
            ident("turn_on"),
            CommandRequirement::None,
            ident("off"),
            ident("on"),
        )],
        Vec::new(),
    )
    .unwrap();
    let blocker_activation = ProjectedActivation::All(vec![
        ProjectedActivation::Always,
        ProjectedActivation::Not(Box::new(ProjectedActivation::StateEquals {
            namespace: machine_namespace.clone(),
            state: ident("off"),
        })),
    ]);
    let cost_activation = ProjectedActivation::Any(vec![
        ProjectedActivation::StateEquals {
            namespace: machine_namespace.clone(),
            state: ident("off"),
        },
        ProjectedActivation::StateEquals {
            namespace: machine_namespace,
            state: ident("on"),
        },
    ]);
    plan_with(
        vec![machine],
        vec![
            MovementClaim::blocker(
                claim("blocker", "blocks_ground"),
                blocker_activation,
                true,
                span(),
            ),
            MovementClaim::traversal_cost(
                claim("cost_low", "traversal_cost_ground"),
                ProjectedActivation::Always,
                2,
                span(),
            )
            .unwrap(),
            MovementClaim::traversal_cost(
                claim("cost_high", "traversal_cost_ground"),
                cost_activation,
                3,
                span(),
            )
            .unwrap(),
        ],
    )
}

fn plan_with(machines: Vec<MachineDefinition>, claims: Vec<MovementClaim>) -> SimulationPlan {
    let subject = MovementSubject::new(
        entity("subject"),
        MovementConnectivity::Region {
            min: LatticeCell::new(0, 0, 0),
            max: LatticeCell::new(0, 0, 0),
        },
        claims,
    )
    .unwrap();
    let resolver =
        MovementResolverPlan::new(ident("ground"), 1, true, true, true, true, vec![subject])
            .unwrap();
    SimulationPlan::new(machines, Vec::new())
        .unwrap()
        .with_movement_resolver(resolver)
}

fn span() -> SourceSpan {
    SourceSpan::new(SourcePath::new("tests/resolver.plan").unwrap(), 0, 1, 1, 1).unwrap()
}
