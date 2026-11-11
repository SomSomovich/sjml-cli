//! Types used by guest-defined crafting recipe serializers.

use crate::{ErrorCode, SjmlError, SjmlResult, StableDecode, StableEncode};

/// Version of the stable crafting callback input and output records.
pub const RECIPE_CALLBACK_SCHEMA_VERSION: u32 = 1;

/// One occupied crafting-grid slot passed to a guest recipe callback.
#[derive(Clone, Debug, Eq, PartialEq, StableDecode, StableEncode)]
#[sjml(version = 1)]
pub struct CraftingSlot {
    /// Registry ID of the stack's item.
    pub item_id: String,
    /// Number of items in this slot.
    pub count: u32,
}

/// Snapshot of a 1–3 by 1–3 crafting grid and the serializer's opaque JSON payload.
#[derive(Clone, Debug, Eq, PartialEq, StableDecode, StableEncode)]
#[sjml(version = 1)]
pub struct RecipeMatchInput {
    /// The recipe JSON's `payload` string, or an empty string when omitted.
    pub payload: String,
    /// Grid width.
    pub width: u32,
    /// Grid height.
    pub height: u32,
    /// Slots in row-major order. Empty slots are `None`.
    pub slots: Vec<Option<CraftingSlot>>,
}

impl RecipeMatchInput {
    /// Creates a callback input using the current schema version.
    pub fn new(
        payload: impl Into<String>,
        width: u32,
        height: u32,
        slots: Vec<Option<CraftingSlot>>,
    ) -> Self {
        Self {
            payload: payload.into(),
            width,
            height,
            slots,
        }
    }
}

/// Result returned by a guest recipe callback.
#[derive(Clone, Copy, Debug, Eq, PartialEq, StableDecode, StableEncode)]
#[sjml(version = 1)]
pub struct RecipeEvaluation {
    /// Whether the current crafting grid matches this recipe.
    pub matches: bool,
    /// Output stack count. Must be zero for a non-match, and 1–64 for a match.
    pub output_count: u32,
}

impl RecipeEvaluation {
    /// Constructs a matching result with a bounded output count.
    pub fn matched(output_count: u32) -> SjmlResult<Self> {
        if !(1..=64).contains(&output_count) {
            return Err(SjmlError::new(
                ErrorCode::InvalidArgument,
                "recipe output count must be between 1 and 64",
            ));
        }
        Ok(Self {
            matches: true,
            output_count,
        })
    }

    /// Constructs a non-matching result.
    pub const fn no_match() -> Self {
        Self {
            matches: false,
            output_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CraftingSlot, RecipeEvaluation, RecipeMatchInput, RECIPE_CALLBACK_SCHEMA_VERSION};
    use crate::{from_bytes, to_bytes};

    #[test]
    fn recipe_callback_input_round_trips_the_versioned_stable_layout() {
        let input = RecipeMatchInput::new(
            "minimum_heat=4",
            2,
            1,
            vec![
                Some(CraftingSlot {
                    item_id: "minecraft:iron_ingot".into(),
                    count: 3,
                }),
                None,
            ],
        );
        let encoded = to_bytes(&input).unwrap();

        assert_eq!(input, from_bytes(&encoded).unwrap());
        assert_eq!(
            RECIPE_CALLBACK_SCHEMA_VERSION,
            u32::from_le_bytes(encoded[..4].try_into().unwrap())
        );
    }

    #[test]
    fn recipe_evaluation_helpers_enforce_the_output_count_contract() {
        assert_eq!(RecipeEvaluation::matched(64).unwrap().output_count, 64);
        assert_eq!(RecipeEvaluation::no_match().output_count, 0);
        assert!(RecipeEvaluation::matched(0).is_err());
        assert!(RecipeEvaluation::matched(65).is_err());
    }
}
