//! Strict decoders for the stable movement lineage.

use super::*;

impl StableWorldIr {
    /// Strictly reconstructs the complete active stable World IR from canonical bytes.
    ///
    /// # Errors
    ///
    /// Returns `EK0412` for malformed, noncanonical, or semantically invalid evidence.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes).map_err(|error| invalid(error.message()))?;
        let decoded = decode_stable_world_ir(&value)?;
        if decoded.to_canonical_bytes() != bytes {
            return Err(invalid(
                "stable World IR changes when reconstructed; persisted semantic ordering or shape is not canonical",
            ));
        }
        Ok(decoded)
    }
}

impl LegacyStableWorldIrV1 {
    /// Strictly reconstructs the one supported legacy stable World IR value.
    ///
    /// # Errors
    ///
    /// Returns `EK0412` unless the bytes are exact canonical stable-v1 evidence.
    pub fn from_canonical_bytes(bytes: &[u8]) -> Result<Self, Diagnostic> {
        let value = parse_canonical(bytes).map_err(|error| invalid(error.message()))?;
        let decoded = decode_legacy_stable_world_ir(&value)?;
        if decoded.to_canonical_bytes() != bytes {
            return Err(invalid(
                "legacy stable World IR changes when reconstructed; persisted ordering or shape is not canonical",
            ));
        }
        Ok(decoded)
    }
}

pub(super) fn decode_stable_movement_v1(
    value: &CanonicalValue,
) -> Result<StableGroundMovementV1, Diagnostic> {
    let fields = object(value, "stable movement row")?;
    exact_fields(
        fields,
        &["blocked_ground", "entity", "traversal_cost_ground"],
        "stable movement row",
    )?;
    let cost = match field(fields, "traversal_cost_ground", "stable movement row")? {
        CanonicalValue::Null => None,
        value => Some(unsigned_u32(value, "stable traversal cost")?),
    };
    rebuild(
        StableGroundMovementV1::new(
            parse_entity(
                field(fields, "entity", "stable movement row")?,
                "stable movement entity",
            )?,
            boolean(
                field(fields, "blocked_ground", "stable movement row")?,
                "blocked_ground",
            )?,
            cost,
        ),
        "stable movement row",
    )
}

pub(super) fn decode_stable_movement_v2(
    value: &CanonicalValue,
) -> Result<StableGroundMovementV2, Diagnostic> {
    let fields = object(value, "stable-v2 movement row")?;
    exact_fields(
        fields,
        &["entity", "movement_disposition_ground"],
        "stable-v2 movement row",
    )?;
    let entity = parse_entity(
        field(fields, "entity", "stable-v2 movement row")?,
        "stable movement entity",
    )?;
    let disposition_fields = object(
        field(
            fields,
            "movement_disposition_ground",
            "stable-v2 movement row",
        )?,
        "stable-v2 movement disposition",
    )?;
    let kind = text(
        field(disposition_fields, "kind", "stable-v2 movement disposition")?,
        "stable-v2 movement kind",
    )?;
    let disposition = match kind {
        "blocked" => {
            exact_fields(
                disposition_fields,
                &["kind", "reasons"],
                "blocked stable-v2 movement disposition",
            )?;
            StableMovementDispositionGround::blocked(decode_claim_reasons(field(
                disposition_fields,
                "reasons",
                "blocked stable-v2 movement disposition",
            )?)?)
        }
        "traversable" => {
            exact_fields(
                disposition_fields,
                &["cost", "kind", "reasons"],
                "traversable stable-v2 movement disposition",
            )?;
            StableMovementDispositionGround::traversable(
                unsigned_u32(
                    field(
                        disposition_fields,
                        "cost",
                        "traversable stable-v2 movement disposition",
                    )?,
                    "stable-v2 traversal cost",
                )?,
                decode_claim_reasons(field(
                    disposition_fields,
                    "reasons",
                    "traversable stable-v2 movement disposition",
                )?)?,
            )
        }
        other => Err(invalid(format!(
            "unsupported stable-v2 movement disposition `{other}`"
        ))),
    };
    Ok(StableGroundMovementV2::new(
        entity,
        rebuild(disposition, "stable-v2 movement disposition")?,
    ))
}

fn decode_claim_reasons(value: &CanonicalValue) -> Result<Vec<ClaimRef>, Diagnostic> {
    array(value, "stable-v2 movement reasons")?
        .iter()
        .map(|value| {
            ClaimRef::parse(text(value, "stable-v2 movement reason")?)
                .map_err(|error| invalid(error.message()))
        })
        .collect()
}
