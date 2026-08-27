//! Lexical field/type validation for the strict plan reader.

use nomos_core::{CanonicalValue, Sha256Digest};

use super::{ActionMarker, ActorAssembly, ActorPose, MaterialFamily, Presence, TerrainAssembly};
use crate::diagnostic::{ObservedError, ObservedResult, codes};
use crate::value::{self, Object};

pub fn fields(root: &Object) -> ObservedResult<Sha256Digest> {
    actions(root)?;
    actors(root)?;
    crop(root)?;
    scene(root)?;
    let digest = source_digest(root)?;
    terrain(root)?;
    Ok(digest)
}

fn actions(root: &Object) -> ObservedResult<()> {
    let rows = value::array(value::field(root, "actions", "$")?, "$.actions")?;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.actions[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &["availability", "id", "marker", "target_actor"],
            &path,
        )?;
        match text_field(object, "availability", &path)? {
            "enabled" | "disabled" => {}
            _ => {
                return Err(value::enum_error(
                    &format!("{path}.availability"),
                    "`enabled | disabled`",
                ));
            }
        }
        text_field(object, "id", &path)?;
        ActionMarker::parse(
            text_field(object, "marker", &path)?,
            &format!("{path}.marker"),
        )?;
        text_field(object, "target_actor", &path)?;
    }
    Ok(())
}

fn actors(root: &Object) -> ObservedResult<()> {
    let rows = value::array(value::field(root, "actors", "$")?, "$.actors")?;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.actors[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &[
                "assembly",
                "cell",
                "controlled",
                "controlled_marker",
                "hostile",
                "hostile_outline",
                "id",
                "life_state",
                "pose",
                "protected",
                "protection_ring",
            ],
            &path,
        )?;
        ActorAssembly::parse(
            text_field(object, "assembly", &path)?,
            &format!("{path}.assembly"),
        )?;
        let cell_path = format!("{path}.cell");
        let cell = value::object(value::field(object, "cell", &path)?, &cell_path)?;
        value::exact_fields(cell, &["x", "y", "z"], &cell_path)?;
        integer_field(cell, "x", &cell_path)?;
        integer_field(cell, "y", &cell_path)?;
        integer_field(cell, "z", &cell_path)?;
        bool_field(object, "controlled", &path)?;
        Presence::parse(
            text_field(object, "controlled_marker", &path)?,
            &format!("{path}.controlled_marker"),
        )?;
        bool_field(object, "hostile", &path)?;
        Presence::parse(
            text_field(object, "hostile_outline", &path)?,
            &format!("{path}.hostile_outline"),
        )?;
        text_field(object, "id", &path)?;
        match text_field(object, "life_state", &path)? {
            "living" | "dead" => {}
            _ => {
                return Err(value::enum_error(
                    &format!("{path}.life_state"),
                    "`living | dead`",
                ));
            }
        }
        ActorPose::parse(text_field(object, "pose", &path)?, &format!("{path}.pose"))?;
        bool_field(object, "protected", &path)?;
        Presence::parse(
            text_field(object, "protection_ring", &path)?,
            &format!("{path}.protection_ring"),
        )?;
    }
    Ok(())
}

fn crop(root: &Object) -> ObservedResult<()> {
    let object = value::object(value::field(root, "crop", "$")?, "$.crop")?;
    value::exact_fields(object, &["height", "width"], "$.crop")?;
    integer_field(object, "height", "$.crop")?;
    integer_field(object, "width", "$.crop")
}

fn scene(root: &Object) -> ObservedResult<()> {
    let object = value::object(value::field(root, "scene", "$")?, "$.scene")?;
    value::exact_fields(object, &["id"], "$.scene")?;
    text_field(object, "id", "$.scene").map(|_| ())
}

fn source_digest(root: &Object) -> ObservedResult<Sha256Digest> {
    let text = value::text(value::field(root, "source_sha256", "$")?, "$.source_sha256")?;
    Sha256Digest::from_hex(text).ok_or_else(|| {
        ObservedError::new(
            codes::SCHEMA_MISMATCH,
            "`$.source_sha256` must be 64 lowercase hexadecimal characters",
        )
    })
}

fn terrain(root: &Object) -> ObservedResult<()> {
    let rows = value::array(
        value::field(root, "terrain_layers", "$")?,
        "$.terrain_layers",
    )?;
    for (index, row) in rows.iter().enumerate() {
        let path = format!("$.terrain_layers[{index}]");
        let object = value::object(row, &path)?;
        value::exact_fields(
            object,
            &[
                "assembly",
                "cells",
                "id",
                "material_family",
                "role",
                "stack",
            ],
            &path,
        )?;
        TerrainAssembly::parse(
            text_field(object, "assembly", &path)?,
            &format!("{path}.assembly"),
        )?;
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
        MaterialFamily::parse(
            text_field(object, "material_family", &path)?,
            &format!("{path}.material_family"),
        )?;
        match text_field(object, "role", &path)? {
            "calm_ground" | "traversable_route" | "structure_footprint" => {}
            _ => {
                return Err(value::enum_error(
                    &format!("{path}.role"),
                    "the three declared terrain roles",
                ));
            }
        }
        integer_field(object, "stack", &path)?;
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

fn bool_field(object: &Object, name: &str, parent: &str) -> ObservedResult<()> {
    value::boolean(
        value::field(object, name, parent)?,
        &format!("{parent}.{name}"),
    )
    .map(|_| ())
}
