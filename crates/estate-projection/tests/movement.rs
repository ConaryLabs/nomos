//! Projection-level ordering proof for the shared movement plan.

use estate_core::{ClaimRef, EntityId, Ident, NamespaceId, SourcePath, SourceSpan};
use estate_projection::{
    LatticeCell, MovementClaim, MovementConnectivity, MovementResolverPlan, MovementSubject,
    ProjectedActivation,
};

#[test]
fn movement_plan_bytes_ignore_subject_and_claim_insertion_order() {
    let alpha = subject("alpha", false);
    let beta = subject("beta", false);
    let forward = plan(vec![alpha, beta]);

    let alpha = subject("alpha", true);
    let beta = subject("beta", true);
    let reversed = plan(vec![beta, alpha]);

    assert_eq!(forward, reversed);
    assert_eq!(forward.to_canonical_bytes(), reversed.to_canonical_bytes());
}

fn plan(subjects: Vec<MovementSubject>) -> MovementResolverPlan {
    MovementResolverPlan::new(ident("ground"), 1, true, true, true, true, subjects).unwrap()
}

fn subject(id: &str, reverse_claims: bool) -> MovementSubject {
    let entity = EntityId::parse(id).unwrap();
    let namespace = NamespaceId::new(entity.clone(), ident("movement"));
    let blocker = MovementClaim::blocker(
        ClaimRef::new(namespace.clone(), ident("blocks_ground")),
        ProjectedActivation::Always,
        true,
        span(),
    );
    let cost = MovementClaim::traversal_cost(
        ClaimRef::new(namespace, ident("traversal_cost_ground")),
        ProjectedActivation::Always,
        3,
        span(),
    )
    .unwrap();
    let claims = if reverse_claims {
        vec![cost, blocker]
    } else {
        vec![blocker, cost]
    };
    MovementSubject::new(
        entity,
        MovementConnectivity::Region {
            min: LatticeCell::new(0, 0, 0),
            max: LatticeCell::new(0, 0, 0),
        },
        claims,
    )
    .unwrap()
}

fn ident(value: &str) -> Ident {
    Ident::new(value).unwrap()
}

fn span() -> SourceSpan {
    SourceSpan::new(
        SourcePath::new("tests/movement.projection").unwrap(),
        0,
        1,
        1,
        1,
    )
    .unwrap()
}
