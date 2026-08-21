//! SW-F compiler proof for light resolution and implemented consumers.

use nomos_compiler::{
    compile_diagnostics_plan, compile_persistence_plan, compile_simulation_plan, compile_source,
    promote_world_ir,
};
use nomos_core::{ClaimRef, EntityId, Ident, NamespaceId, PrimitiveKindId, SourcePath, SourceSpan};
use nomos_projection::{
    DiagnosticsPlan, LightResolverPlan as ProjectedLightResolverPlan,
    validate_light_projection_agreement,
};
use nomos_schema::{
    Binding, CapabilityKind, Cell, ClaimActivation, ClaimTemplate, ClaimValue, IrEntity,
    LightCompositionLaw, LightResolverPlan, LightResolverSubject, MachineTemplate,
    PrimitiveExpansion, ProjectionConsumer, WorldIr, source_schema,
};

const SOURCE: &str = include_str!("../../../fixtures/gaol.nomos");

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
    compile_source(SOURCE, SourcePath::new("fixtures/gaol.nomos").unwrap()).unwrap()
}

#[test]
fn simulation_persistence_and_diagnostics_share_exact_light_semantics() {
    let ir = fixture_ir();
    let stable = promote_world_ir(&ir).unwrap();
    let simulation = compile_simulation_plan(&stable).unwrap();
    let persistence = compile_persistence_plan(&stable).unwrap();
    let diagnostics = compile_diagnostics_plan(&stable).unwrap();
    validate_light_projection_agreement(simulation.light_resolver(), &persistence, &diagnostics)
        .unwrap();
    assert_eq!(
        simulation.light_resolver().to_canonical_bytes(),
        persistence.light_resolver().to_canonical_bytes()
    );
    assert_eq!(
        persistence.light_resolver().to_canonical_bytes(),
        diagnostics.light_resolver().to_canonical_bytes()
    );
    assert_eq!(simulation.light_resolver().subjects().len(), 1);
    assert_eq!(
        simulation.light_resolver().subjects()[0]
            .entity()
            .to_string(),
        "brazier_02"
    );
    assert_eq!(persistence.entities().len(), 3);
    assert_eq!(diagnostics.entities().len(), 3);
}

#[test]
fn mismatched_persistence_or_diagnostics_light_facts_fail_closed() {
    let ir = fixture_ir();
    let stable = promote_world_ir(&ir).unwrap();
    let simulation = compile_simulation_plan(&stable).unwrap();
    let persistence = compile_persistence_plan(&stable).unwrap();
    let diagnostics = compile_diagnostics_plan(&stable).unwrap();
    let mismatched = DiagnosticsPlan::new(
        diagnostics.entities().to_vec(),
        ProjectedLightResolverPlan::empty_gate_k(),
    )
    .unwrap();
    let rejected =
        validate_light_projection_agreement(simulation.light_resolver(), &persistence, &mismatched)
            .unwrap_err();
    assert_eq!(rejected.code().as_str(), "EK0912");
}

#[test]
fn dangling_light_activations_and_conflicting_values_fail_closed() {
    let dangling_namespace = light_world(
        vec![(
            "emits_light",
            ClaimActivation::StateEquals {
                namespace: namespace("subject", "missing"),
                state: ident("lit"),
            },
            ClaimValue::Bool(true),
        )],
        true,
    );
    assert_eq!(
        promote_world_ir(&dangling_namespace)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0903"
    );

    let dangling_state = light_world(
        vec![(
            "emits_light",
            ClaimActivation::StateEquals {
                namespace: namespace("subject", "emission"),
                state: ident("missing"),
            },
            ClaimValue::Bool(true),
        )],
        true,
    );
    assert_eq!(
        promote_world_ir(&dangling_state)
            .unwrap_err()
            .code()
            .as_str(),
        "EK0904"
    );

    for value in [ClaimValue::Bool(false), ClaimValue::Uint(1)] {
        let conflicting = light_world(
            vec![
                (
                    "emits_light",
                    ClaimActivation::Always,
                    ClaimValue::Bool(true),
                ),
                ("negative_light", ClaimActivation::Always, value),
            ],
            true,
        );
        assert_eq!(
            promote_world_ir(&conflicting).unwrap_err().code().as_str(),
            "EK0910"
        );
    }
}

#[test]
fn absent_light_subject_fails_closed() {
    let world = light_world(
        vec![(
            "emits_light",
            ClaimActivation::Always,
            ClaimValue::Bool(true),
        )],
        false,
    );
    assert_eq!(
        promote_world_ir(&world).unwrap_err().code().as_str(),
        "EK0911"
    );
}

fn light_world(claims: Vec<(&str, ClaimActivation, ClaimValue)>, attach_subject: bool) -> WorldIr {
    let subject = entity("subject");
    let claim_templates = claims
        .into_iter()
        .map(|(capability, activation, value)| {
            ClaimTemplate::new(
                ClaimRef::new(namespace("subject", "emission"), ident(capability)),
                CapabilityKind::EmitsLight,
                activation,
                value,
            )
        })
        .collect::<Vec<_>>();
    let claim_refs = claim_templates
        .iter()
        .map(|claim| claim.id().clone())
        .collect::<Vec<_>>();
    let machine = MachineTemplate::new(
        namespace("subject", "emission"),
        vec![ident("extinguished"), ident("lit")],
        ident("lit"),
    );
    let expansion = PrimitiveExpansion::new(
        [CapabilityKind::EmitsLight, CapabilityKind::Machine],
        vec![machine],
        claim_templates,
    )
    .unwrap();
    let entity = IrEntity::new(
        subject.clone(),
        PrimitiveKindId::parse("primitive/extinguishable_light").unwrap(),
        Binding::Cell(Cell::new(0, 0, 0)),
        None,
        expansion,
        span(),
    );
    let subjects = if attach_subject {
        vec![LightResolverSubject::new(subject, claim_refs).unwrap()]
    } else {
        Vec::new()
    };
    WorldIr::new(
        source_schema(),
        Vec::new(),
        vec![entity],
        Vec::new(),
        Vec::new(),
    )
    .unwrap()
    .with_light_resolver(
        LightResolverPlan::new(
            LightCompositionLaw::Union,
            vec![
                ProjectionConsumer::Diagnostics,
                ProjectionConsumer::Persistence,
                ProjectionConsumer::Simulation,
            ],
            subjects,
        )
        .unwrap(),
    )
}

fn span() -> SourceSpan {
    SourceSpan::new(SourcePath::new("tests/light.nomos").unwrap(), 0, 1, 1, 1).unwrap()
}
