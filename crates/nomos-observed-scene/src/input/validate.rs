//! Whole-document validation in the exact contract precedence.

use std::collections::BTreeSet;

use nomos_core::{CanonicalValue, RepairClass};

use super::{Availability, LifeState, LocalId, TerrainRole};
use crate::diagnostic::{ObservedError, ObservedResult, codes};
use crate::value::{self, Object};

pub fn document(root: &Object) -> ObservedResult<()> {
    field_phase_lexical(root)?;
    field_phase(root)?;
    bound_phase_lexical(root)?;
    bound_phase(root)?;
    identity_phase_lexical(root)?;
    identity_phase(root)?;
    order_phase_lexical(root)?;
    order_phase(root)?;
    reference_phase(root)
}

fn field_phase_lexical(root: &Object) -> ObservedResult<()> {
    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    for (index, action) in actions.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(action, &path)?;
        value::exact_fields(object, &["availability", "id", "target_actor"], &path)?;
        let availability_path = format!("{path}.availability");
        Availability::parse(
            text_field(object, "availability", &path)?,
            &availability_path,
        )?;
        text_field(object, "id", &path)?;
        text_field(object, "target_actor", &path)?;
    }

    let actors = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    for (index, actor) in actors.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(actor, &path)?;
        value::exact_fields(
            object,
            &[
                "cell",
                "controlled",
                "hostile",
                "id",
                "life_state",
                "protected",
            ],
            &path,
        )?;
        let cell_path = format!("{path}.cell");
        let cell = value::object(value::field(object, "cell", &path)?, &cell_path)?;
        value::exact_fields(cell, &["x", "y", "z"], &cell_path)?;
        integer_field(cell, "x", &cell_path)?;
        integer_field(cell, "y", &cell_path)?;
        integer_field(cell, "z", &cell_path)?;
        value::boolean(
            value::field(object, "controlled", &path)?,
            &format!("{path}.controlled"),
        )?;
        value::boolean(
            value::field(object, "hostile", &path)?,
            &format!("{path}.hostile"),
        )?;
        text_field(object, "id", &path)?;
        let life_path = format!("{path}.life_state");
        LifeState::parse(text_field(object, "life_state", &path)?, &life_path)?;
        value::boolean(
            value::field(object, "protected", &path)?,
            &format!("{path}.protected"),
        )?;
    }

    let crop = value::object(value::field(root, "crop", "$")?, "$.crop")?;
    value::exact_fields(crop, &["height", "width"], "$.crop")?;
    integer_field(crop, "height", "$.crop")?;
    integer_field(crop, "width", "$.crop")?;

    let scene = value::object(value::field(root, "scene", "$")?, "$.scene")?;
    value::exact_fields(scene, &["id"], "$.scene")?;
    text_field(scene, "id", "$.scene")?;

    let layers = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    for (index, layer) in layers.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(layer, &path)?;
        value::exact_fields(object, &["cells", "id", "role"], &path)?;
        let cells = value::array(
            value::field(object, "cells", &path)?,
            &format!("{path}.cells"),
        )?;
        for (cell_index, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.cells[{cell_index}]");
            let cell = value::object(cell, &cell_path)?;
            value::exact_fields(cell, &["x", "y"], &cell_path)?;
            integer_field(cell, "x", &cell_path)?;
            integer_field(cell, "y", &cell_path)?;
        }
        text_field(object, "id", &path)?;
        let role_path = format!("{path}.role");
        TerrainRole::parse(text_field(object, "role", &path)?, &role_path)?;
    }
    Ok(())
}

fn bound_phase_lexical(root: &Object) -> ObservedResult<()> {
    let crop = value::object(value::field(root, "crop", "$")?, "$.crop")?;
    let width = value::integer(value::field(crop, "width", "$.crop")?, "$.crop.width")?;
    let height = value::integer(value::field(crop, "height", "$.crop")?, "$.crop.height")?;

    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    require_count(actions.len(), 0, 128, "$.actions")?;

    let actors = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    require_count(actors.len(), 1, 64, "$.actors")?;
    for (index, actor) in actors.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(actor, &path)?;
        let cell_path = format!("{path}.cell");
        let cell = value::object(value::field(object, "cell", &path)?, &cell_path)?;
        let x = value::integer(
            value::field(cell, "x", &cell_path)?,
            &format!("{cell_path}.x"),
        )?;
        let y = value::integer(
            value::field(cell, "y", &cell_path)?,
            &format!("{cell_path}.y"),
        )?;
        let z = value::integer(
            value::field(cell, "z", &cell_path)?,
            &format!("{cell_path}.z"),
        )?;
        require_range(x, 0, width - 1, &format!("{cell_path}.x"))?;
        require_range(y, 0, height - 1, &format!("{cell_path}.y"))?;
        if z != 0 {
            return Err(value::bound_error(format!(
                "`{cell_path}.z` must be exactly zero"
            )));
        }
    }

    require_range(height, 1, 32, "$.crop.height")?;
    require_range(width, 1, 32, "$.crop.width")?;

    let layers = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    require_count(layers.len(), 3, 8, "$.terrain_layers")?;
    let mut assignments = 0_usize;
    let mut roles = BTreeSet::new();
    for (index, layer) in layers.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(layer, &path)?;
        let cells = value::array(
            value::field(object, "cells", &path)?,
            &format!("{path}.cells"),
        )?;
        require_count(cells.len(), 1, 1024, &format!("{path}.cells"))?;
        assignments = assignments
            .checked_add(cells.len())
            .ok_or_else(|| value::bound_error("total terrain cell assignment count overflowed"))?;
        let mut seen = BTreeSet::new();
        for (cell_index, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.cells[{cell_index}]");
            let cell = value::object(cell, &cell_path)?;
            let x = value::integer(
                value::field(cell, "x", &cell_path)?,
                &format!("{cell_path}.x"),
            )?;
            let y = value::integer(
                value::field(cell, "y", &cell_path)?,
                &format!("{cell_path}.y"),
            )?;
            require_range(x, 0, width - 1, &format!("{cell_path}.x"))?;
            require_range(y, 0, height - 1, &format!("{cell_path}.y"))?;
            if !seen.insert((x, y)) {
                return Err(value::bound_error(format!(
                    "duplicate terrain cell at `{cell_path}`"
                )));
            }
        }
        roles.insert(TerrainRole::parse(
            text_field(object, "role", &path)?,
            &format!("{path}.role"),
        )?);
    }
    require_count(assignments, 3, 4096, "total terrain cell assignments")?;
    if roles.len() != 3 {
        return Err(value::bound_error(
            "the scene must contain at least one layer of every terrain role",
        ));
    }
    Ok(())
}

fn identity_phase_lexical(root: &Object) -> ObservedResult<()> {
    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    validate_identity_collection(actions, "$.actions")?;
    for (index, action) in actions.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(action, &path)?;
        LocalId::new(text_field(object, "target_actor", &path)?)?;
    }
    validate_identity_collection(
        value::array(value::field(root, "actors", "$")?, "$.actors")?,
        "$.actors",
    )?;
    let scene = value::object(value::field(root, "scene", "$")?, "$.scene")?;
    LocalId::new(text_field(scene, "id", "$.scene")?)?;
    validate_identity_collection(
        value::array(
            value::field(root, "terrain_layers", "$")?,
            "$.terrain_layers",
        )?,
        "$.terrain_layers",
    )
}

fn order_phase_lexical(root: &Object) -> ObservedResult<()> {
    require_identity_order(
        value::array(value::field(root, "actions", "$")?, "$.actions")?,
        "$.actions",
    )?;
    require_identity_order(
        value::array(value::field(root, "actors", "$")?, "$.actors")?,
        "$.actors",
    )?;
    let layers = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    require_identity_order(layers, "$.terrain_layers")?;
    for (index, layer) in layers.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(layer, &path)?;
        let cells = value::array(
            value::field(object, "cells", &path)?,
            &format!("{path}.cells"),
        )?;
        let mut prior = None;
        for (cell_index, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.cells[{cell_index}]");
            let cell = value::object(cell, &cell_path)?;
            let x = value::integer(
                value::field(cell, "x", &cell_path)?,
                &format!("{cell_path}.x"),
            )?;
            let y = value::integer(
                value::field(cell, "y", &cell_path)?,
                &format!("{cell_path}.y"),
            )?;
            if prior.is_some_and(|prior| prior >= (y, x)) {
                return Err(order_error(format!(
                    "`{path}.cells` is not strict row-major order"
                )));
            }
            prior = Some((y, x));
        }
    }
    Ok(())
}

fn field_phase(root: &Object) -> ObservedResult<()> {
    let crop = value::object(value::field(root, "crop", "$")?, "$.crop")?;
    value::exact_fields(crop, &["height", "width"], "$.crop")?;
    integer_field(crop, "height", "$.crop")?;
    integer_field(crop, "width", "$.crop")?;

    let scene = value::object(value::field(root, "scene", "$")?, "$.scene")?;
    value::exact_fields(scene, &["id"], "$.scene")?;
    text_field(scene, "id", "$.scene")?;

    let layers = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    for (index, layer) in layers.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(layer, &path)?;
        value::exact_fields(object, &["cells", "id", "role"], &path)?;
        text_field(object, "id", &path)?;
        let role_path = format!("{path}.role");
        TerrainRole::parse(text_field(object, "role", &path)?, &role_path)?;
        let cells = value::array(
            value::field(object, "cells", &path)?,
            &format!("{path}.cells"),
        )?;
        for (cell_index, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.cells[{cell_index}]");
            let cell = value::object(cell, &cell_path)?;
            value::exact_fields(cell, &["x", "y"], &cell_path)?;
            integer_field(cell, "x", &cell_path)?;
            integer_field(cell, "y", &cell_path)?;
        }
    }

    let actors = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    for (index, actor) in actors.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(actor, &path)?;
        value::exact_fields(
            object,
            &[
                "cell",
                "controlled",
                "hostile",
                "id",
                "life_state",
                "protected",
            ],
            &path,
        )?;
        text_field(object, "id", &path)?;
        value::boolean(
            value::field(object, "controlled", &path)?,
            &format!("{path}.controlled"),
        )?;
        value::boolean(
            value::field(object, "hostile", &path)?,
            &format!("{path}.hostile"),
        )?;
        value::boolean(
            value::field(object, "protected", &path)?,
            &format!("{path}.protected"),
        )?;
        let life_path = format!("{path}.life_state");
        LifeState::parse(text_field(object, "life_state", &path)?, &life_path)?;
        let cell_path = format!("{path}.cell");
        let cell = value::object(value::field(object, "cell", &path)?, &cell_path)?;
        value::exact_fields(cell, &["x", "y", "z"], &cell_path)?;
        integer_field(cell, "x", &cell_path)?;
        integer_field(cell, "y", &cell_path)?;
        integer_field(cell, "z", &cell_path)?;
    }

    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    for (index, action) in actions.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(action, &path)?;
        value::exact_fields(object, &["availability", "id", "target_actor"], &path)?;
        text_field(object, "id", &path)?;
        text_field(object, "target_actor", &path)?;
        let availability_path = format!("{path}.availability");
        Availability::parse(
            text_field(object, "availability", &path)?,
            &availability_path,
        )?;
    }
    Ok(())
}

fn bound_phase(root: &Object) -> ObservedResult<()> {
    let crop = value::object(value::field(root, "crop", "$")?, "$.crop")?;
    let width = value::integer(value::field(crop, "width", "$.crop")?, "$.crop.width")?;
    let height = value::integer(value::field(crop, "height", "$.crop")?, "$.crop.height")?;
    require_range(width, 1, 32, "$.crop.width")?;
    require_range(height, 1, 32, "$.crop.height")?;

    let layers = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    require_count(layers.len(), 3, 8, "$.terrain_layers")?;
    let mut assignments = 0_usize;
    let mut roles = BTreeSet::new();
    for (index, layer) in layers.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(layer, &path)?;
        let role = TerrainRole::parse(text_field(object, "role", &path)?, &format!("{path}.role"))?;
        roles.insert(role);
        let cells = value::array(
            value::field(object, "cells", &path)?,
            &format!("{path}.cells"),
        )?;
        require_count(cells.len(), 1, 1024, &format!("{path}.cells"))?;
        assignments = assignments
            .checked_add(cells.len())
            .ok_or_else(|| value::bound_error("total terrain cell assignment count overflowed"))?;
        let mut seen = BTreeSet::new();
        for (cell_index, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.cells[{cell_index}]");
            let cell = value::object(cell, &cell_path)?;
            let x = value::integer(
                value::field(cell, "x", &cell_path)?,
                &format!("{cell_path}.x"),
            )?;
            let y = value::integer(
                value::field(cell, "y", &cell_path)?,
                &format!("{cell_path}.y"),
            )?;
            require_range(x, 0, width - 1, &format!("{cell_path}.x"))?;
            require_range(y, 0, height - 1, &format!("{cell_path}.y"))?;
            if !seen.insert((x, y)) {
                return Err(value::bound_error(format!(
                    "duplicate terrain cell at `{cell_path}`"
                )));
            }
        }
    }
    require_count(assignments, 3, 4096, "total terrain cell assignments")?;
    if roles.len() != 3 {
        return Err(value::bound_error(
            "the scene must contain at least one layer of every terrain role",
        ));
    }

    let actors = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    require_count(actors.len(), 1, 64, "$.actors")?;
    for (index, actor) in actors.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(actor, &path)?;
        let cell_path = format!("{path}.cell");
        let cell = value::object(value::field(object, "cell", &path)?, &cell_path)?;
        let x = value::integer(
            value::field(cell, "x", &cell_path)?,
            &format!("{cell_path}.x"),
        )?;
        let y = value::integer(
            value::field(cell, "y", &cell_path)?,
            &format!("{cell_path}.y"),
        )?;
        let z = value::integer(
            value::field(cell, "z", &cell_path)?,
            &format!("{cell_path}.z"),
        )?;
        require_range(x, 0, width - 1, &format!("{cell_path}.x"))?;
        require_range(y, 0, height - 1, &format!("{cell_path}.y"))?;
        if z != 0 {
            return Err(value::bound_error(format!(
                "`{cell_path}.z` must be exactly zero"
            )));
        }
    }

    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    require_count(actions.len(), 0, 128, "$.actions")
}

fn identity_phase(root: &Object) -> ObservedResult<()> {
    let scene = value::object(value::field(root, "scene", "$")?, "$.scene")?;
    LocalId::new(text_field(scene, "id", "$.scene")?)?;
    validate_identity_collection(
        value::array(
            value::field(root, "terrain_layers", "$")?,
            "$.terrain_layers",
        )?,
        "$.terrain_layers",
    )?;
    validate_identity_collection(
        value::array(value::field(root, "actors", "$")?, "$.actors")?,
        "$.actors",
    )?;
    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    validate_identity_collection(actions, "$.actions")?;
    for (index, action) in actions.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(action, &path)?;
        LocalId::new(text_field(object, "target_actor", &path)?)?;
    }
    Ok(())
}

fn validate_identity_collection(rows: &[CanonicalValue], path: &str) -> ObservedResult<()> {
    let mut seen = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let row_path = format!("{path}[{index}]");
        let object = value::object(row, &row_path)?;
        let id = LocalId::new(text_field(object, "id", &row_path)?)?;
        if !seen.insert(id.clone()) {
            return Err(ObservedError::new(
                codes::IDENTITY_INVALID,
                format!("duplicate identity `{}` at `{row_path}`", id.as_str()),
            )
            .with_repair(RepairClass::RemoveDuplicateDeclaration));
        }
    }
    Ok(())
}

fn order_phase(root: &Object) -> ObservedResult<()> {
    let layers = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    require_identity_order(layers, "$.terrain_layers")?;
    for (index, layer) in layers.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(layer, &path)?;
        let cells = value::array(
            value::field(object, "cells", &path)?,
            &format!("{path}.cells"),
        )?;
        let mut prior = None;
        for (cell_index, cell) in cells.iter().enumerate() {
            let cell_path = format!("{path}.cells[{cell_index}]");
            let cell = value::object(cell, &cell_path)?;
            let x = value::integer(
                value::field(cell, "x", &cell_path)?,
                &format!("{cell_path}.x"),
            )?;
            let y = value::integer(
                value::field(cell, "y", &cell_path)?,
                &format!("{cell_path}.y"),
            )?;
            if prior.is_some_and(|prior| prior >= (y, x)) {
                return Err(order_error(format!(
                    "`{path}.cells` is not strict row-major order"
                )));
            }
            prior = Some((y, x));
        }
    }
    require_identity_order(
        value::array(value::field(root, "actors", "$")?, "$.actors")?,
        "$.actors",
    )?;
    require_identity_order(
        value::array(value::field(root, "actions", "$")?, "$.actions")?,
        "$.actions",
    )
}

fn require_identity_order(rows: &[CanonicalValue], path: &str) -> ObservedResult<()> {
    let mut prior: Option<&str> = None;
    for (index, row) in rows.iter().enumerate() {
        let row_path = format!("{path}[{index}]");
        let object = value::object(row, &row_path)?;
        let id = text_field(object, "id", &row_path)?;
        if prior.is_some_and(|prior| prior >= id) {
            return Err(order_error(format!(
                "`{path}` is not strictly ordered by identity"
            )));
        }
        prior = Some(id);
    }
    Ok(())
}

fn reference_phase(root: &Object) -> ObservedResult<()> {
    let actors = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    let actor_ids: BTreeSet<&str> = actors
        .iter()
        .enumerate()
        .map(|(index, actor)| {
            let path = format!("$.actors[{index}]");
            let actor = value::object(actor, &path)?;
            text_field(actor, "id", &path)
        })
        .collect::<ObservedResult<_>>()?;
    let actions = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    for (index, action) in actions.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let action = value::object(action, &path)?;
        let target = text_field(action, "target_actor", &path)?;
        if !actor_ids.contains(target) {
            return Err(ObservedError::new(
                codes::TARGET_DANGLING,
                format!("`{path}.target_actor` names no actor"),
            )
            .with_repair(RepairClass::DeclareReferencedEntity));
        }
    }
    Ok(())
}

fn text_field<'a>(object: &'a Object, name: &str, parent: &str) -> ObservedResult<&'a str> {
    value::text(
        value::field(object, name, parent)?,
        &format!("{parent}.{name}"),
    )
}

fn integer_field(object: &Object, name: &str, parent: &str) -> ObservedResult<()> {
    let path = format!("{parent}.{name}");
    match value::field(object, name, parent)? {
        CanonicalValue::Int(_) | CanonicalValue::Uint(_) => Ok(()),
        _ => Err(value::field_error(format!("`{path}` must be an integer"))),
    }
}

fn require_range(value_: i64, minimum: i64, maximum: i64, path: &str) -> ObservedResult<()> {
    if (minimum..=maximum).contains(&value_) {
        Ok(())
    } else {
        Err(value::bound_error(format!(
            "`{path}` must be in {minimum}..={maximum}"
        )))
    }
}

fn require_count(value_: usize, minimum: usize, maximum: usize, path: &str) -> ObservedResult<()> {
    if (minimum..=maximum).contains(&value_) {
        Ok(())
    } else {
        Err(value::bound_error(format!(
            "`{path}` must contain {minimum}..={maximum} rows"
        )))
    }
}

fn order_error(message: impl Into<String>) -> ObservedError {
    ObservedError::new(codes::INPUT_NOT_CANONICAL, message)
        .with_repair(RepairClass::EmitCanonicalBytes)
}
