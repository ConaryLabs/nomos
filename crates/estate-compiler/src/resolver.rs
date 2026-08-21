//! Compiler-owned Gate K movement resolver preparation.

use estate_core::{Diagnostic, Ident};
use estate_schema::{
    Binding, CapabilityKind, Cell, Direction, GroundConnectivity, GroundMovementCoherence,
    IrEntity, MovementCompositionLaw, MovementResolverPlan, MovementResolverSubject,
};

pub(crate) fn construction_plan(entities: &[IrEntity]) -> Result<MovementResolverPlan, Diagnostic> {
    let mut subjects = Vec::new();
    for entity in entities {
        let claims: Vec<_> = entity
            .expansion()
            .claims()
            .iter()
            .filter(|claim| {
                matches!(
                    claim.capability(),
                    CapabilityKind::BlocksGround | CapabilityKind::TraversalCostGround
                )
            })
            .map(|claim| claim.id().clone())
            .collect();
        if claims.is_empty() {
            continue;
        }
        subjects.push(MovementResolverSubject::new(
            entity.id().clone(),
            derive_connectivity(entity.binding())?,
            claims,
        )?);
    }
    MovementResolverPlan::new(
        vec![
            MovementCompositionLaw::AnyActiveBlocker,
            MovementCompositionLaw::MaximumActiveCost,
        ],
        vec![GroundMovementCoherence::new(
            Ident::new("ground").expect("the built-in movement channel is legal"),
            1,
            true,
        )?],
        subjects,
    )
}

pub(crate) fn derive_connectivity(binding: &Binding) -> Result<GroundConnectivity, Diagnostic> {
    match binding {
        Binding::Face { cell, direction } => {
            let second = horizontal_neighbor(*cell, *direction)?;
            Ok(GroundConnectivity::FaceAdjacent {
                first: *cell,
                second,
            })
        }
        Binding::Region { min, max } => Ok(GroundConnectivity::Region {
            min: *min,
            max: *max,
        }),
        Binding::Cell(_) => Err(connectivity_invalid(
            "a cell binding does not declare a ground connection or traversable region",
        )),
    }
}

fn horizontal_neighbor(cell: Cell, direction: Direction) -> Result<Cell, Diagnostic> {
    let (x, y) = match direction {
        Direction::North => (cell.x(), cell.y().checked_sub(1).ok_or_else(grid_overflow)?),
        Direction::East => (cell.x().checked_add(1).ok_or_else(grid_overflow)?, cell.y()),
        Direction::South => (cell.x(), cell.y().checked_add(1).ok_or_else(grid_overflow)?),
        Direction::West => (cell.x().checked_sub(1).ok_or_else(grid_overflow)?, cell.y()),
        Direction::Up | Direction::Down => {
            return Err(connectivity_invalid(
                "a vertical face does not provide Gate K ground connectivity",
            ));
        }
    };
    Ok(Cell::new(x, y, cell.z()))
}

fn connectivity_invalid(message: &str) -> Diagnostic {
    Diagnostic::new(
        estate_core::diagnostic::codes::RESOLVER_CONNECTIVITY_INVALID,
        message,
    )
}

fn grid_overflow() -> Diagnostic {
    Diagnostic::new(
        estate_core::diagnostic::codes::ARITHMETIC_OVERFLOW,
        "deriving an adjacent lattice cell overflowed i32 coordinates",
    )
}
