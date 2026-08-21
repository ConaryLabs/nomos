//! SW-E compiler proof for movement resolver preparation and projection.

use estate_compiler::{compile_navigation_plan, compile_simulation_plan, compile_source};
use estate_core::{
    ClaimRef, EntityId, Ident, NamespaceId, PrimitiveKindId, SourcePath, SourceSpan,
};
use estate_schema::{
    Binding, CapabilityKind, Cell, ClaimActivation, ClaimTemplate, ClaimValue, Direction,
    GroundConnectivity, GroundMovementCoherence, IrEntity, MachineTemplate, MovementCompositionLaw,
    MovementResolverPlan, MovementResolverSubject, PrimitiveExpansion, WorldIr, source_schema,
};

const SOURCE: &str = include_str!("../../../fixtures/gaol.estate");

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn entity(value: &str) -> EntityId {
    EntityId::parse(value).unwrap()
}

fn namespace(entity: &str, local: &str) -> NamespaceId {
    NamespaceId::new(self::entity(entity), ident(local))
}

fn fixture_ir() -> WorldIr {
    compile_source(SOURCE, SourcePath::new("fixtures/gaol.estate").unwrap()).unwrap()
}

#[test]
fn simulation_and_navigation_share_identical_movement_plan_bytes() {
    let ir = fixture_ir();
    let simulation = compile_simulation_plan(&ir).unwrap();
    let navigation = compile_navigation_plan(&ir).unwrap();
    assert_eq!(simulation.schema(), &estate_projection::simulation_schema());
    assert_eq!(navigation.schema(), &estate_projection::navigation_schema());
    assert_eq!(
        simulation.movement_resolver().to_canonical_bytes(),
        navigation.movement_resolver().to_canonical_bytes()
    );
    assert_eq!(
        simulation.movement_resolver(),
        navigation.movement_resolver()
    );
    assert_eq!(
        simulation
            .movement_resolver()
            .subjects()
            .iter()
            .map(|subject| subject.entity().to_string())
            .collect::<Vec<_>>(),
        ["flooded_section", "north_gate"]
    );
    let water = simulation
        .movement_resolver()
        .subjects()
        .iter()
        .find(|subject| subject.entity().to_string() == "flooded_section")
        .unwrap();
    assert_eq!(water.claims().len(), 1);
    assert_eq!(
        water.claims()[0].source().path().as_str(),
        "fixtures/gaol.estate"
    );
}

#[test]
fn construction_resolver_order_is_independent_of_insertion() {
    let first = resolver_subject("alpha");
    let second = resolver_subject("beta");
    let forward = empty_world().with_movement_resolver(resolver_plan(
        vec![
            MovementCompositionLaw::AnyActiveBlocker,
            MovementCompositionLaw::MaximumActiveCost,
        ],
        vec![first.clone(), second.clone()],
    ));
    let reversed = empty_world().with_movement_resolver(resolver_plan(
        vec![
            MovementCompositionLaw::MaximumActiveCost,
            MovementCompositionLaw::AnyActiveBlocker,
        ],
        vec![second, first],
    ));
    assert_eq!(forward.to_canonical_bytes(), reversed.to_canonical_bytes());
}

#[test]
fn compiler_rejects_dangling_activation_namespace_and_state() {
    let dangling_namespace = claim_world(
        ClaimActivation::StateEquals {
            namespace: namespace("subject", "missing"),
            state: ident("on"),
        },
        ClaimValue::Bool(true),
        Binding::Face {
            cell: Cell::new(0, 0, 0),
            direction: Direction::North,
        },
        valid_face_connectivity(),
    );
    assert_eq!(
        compile_navigation_plan(&dangling_namespace)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0903"
    );

    let missing_state = claim_world(
        ClaimActivation::StateEquals {
            namespace: namespace("subject", "machine"),
            state: ident("missing"),
        },
        ClaimValue::Bool(true),
        Binding::Face {
            cell: Cell::new(0, 0, 0),
            direction: Direction::North,
        },
        valid_face_connectivity(),
    );
    assert_eq!(
        compile_navigation_plan(&missing_state)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0904"
    );
}

#[test]
fn compiler_rejects_wrong_claim_values_and_connectivity() {
    let wrong_value = claim_world(
        ClaimActivation::Always,
        ClaimValue::Uint(1),
        Binding::Face {
            cell: Cell::new(0, 0, 0),
            direction: Direction::North,
        },
        valid_face_connectivity(),
    );
    assert_eq!(
        compile_navigation_plan(&wrong_value)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0905"
    );

    let wrong_connectivity = claim_world(
        ClaimActivation::Always,
        ClaimValue::Bool(true),
        Binding::Face {
            cell: Cell::new(0, 0, 0),
            direction: Direction::North,
        },
        GroundConnectivity::Region {
            min: Cell::new(0, 0, 0),
            max: Cell::new(0, 0, 0),
        },
    );
    assert_eq!(
        compile_navigation_plan(&wrong_connectivity)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0906"
    );

    let no_ground_connection = claim_world(
        ClaimActivation::Always,
        ClaimValue::Bool(true),
        Binding::Cell(Cell::new(0, 0, 0)),
        GroundConnectivity::Region {
            min: Cell::new(0, 0, 0),
            max: Cell::new(0, 0, 0),
        },
    );
    assert_eq!(
        compile_navigation_plan(&no_ground_connection)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0906"
    );
}

#[test]
fn compiler_rejects_resolver_subjects_without_world_entities() {
    let ghost = resolver_subject("ghost");
    let ir = empty_world().with_movement_resolver(resolver_plan(
        vec![
            MovementCompositionLaw::AnyActiveBlocker,
            MovementCompositionLaw::MaximumActiveCost,
        ],
        vec![ghost],
    ));
    assert_eq!(
        compile_navigation_plan(&ir).unwrap_err().code().as_str(),
        "EK0907"
    );
}

fn claim_world(
    activation: ClaimActivation,
    value: ClaimValue,
    binding: Binding,
    connectivity: GroundConnectivity,
) -> WorldIr {
    let subject = entity("subject");
    let claim_id = ClaimRef::new(namespace("subject", "claim"), ident("blocks_ground"));
    let claim = ClaimTemplate::new(
        claim_id.clone(),
        CapabilityKind::BlocksGround,
        activation,
        value,
    );
    let machine = MachineTemplate::new(
        namespace("subject", "machine"),
        vec![ident("off"), ident("on")],
        ident("off"),
    );
    let expansion =
        PrimitiveExpansion::new([CapabilityKind::BlocksGround], vec![machine], vec![claim])
            .unwrap();
    let entity = IrEntity::new(
        subject.clone(),
        PrimitiveKindId::parse("primitive/iron_barred_door").unwrap(),
        binding,
        None,
        expansion,
        span(),
    );
    WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity],
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
    .with_movement_resolver(resolver_plan(
        vec![
            MovementCompositionLaw::AnyActiveBlocker,
            MovementCompositionLaw::MaximumActiveCost,
        ],
        vec![MovementResolverSubject::new(subject, connectivity, vec![claim_id]).unwrap()],
    ))
}

fn empty_world() -> WorldIr {
    WorldIr::new(
        source_schema(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
}

fn resolver_subject(id: &str) -> MovementResolverSubject {
    MovementResolverSubject::new(
        entity(id),
        GroundConnectivity::Region {
            min: Cell::new(0, 0, 0),
            max: Cell::new(0, 0, 0),
        },
        Vec::new(),
    )
    .unwrap()
}

fn resolver_plan(
    laws: Vec<MovementCompositionLaw>,
    subjects: Vec<MovementResolverSubject>,
) -> MovementResolverPlan {
    MovementResolverPlan::new(
        laws,
        vec![GroundMovementCoherence::new(ident("ground"), 1, true).unwrap()],
        subjects,
    )
    .unwrap()
}

fn valid_face_connectivity() -> GroundConnectivity {
    GroundConnectivity::FaceAdjacent {
        first: Cell::new(0, 0, 0),
        second: Cell::new(0, -1, 0),
    }
}

fn span() -> SourceSpan {
    SourceSpan::new(
        SourcePath::new("tests/movement.estate").unwrap(),
        0,
        1,
        1,
        1,
    )
    .unwrap()
}
