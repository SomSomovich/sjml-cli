#![allow(dead_code)]

use sjml::prelude::*;

/// Registers a grate with two separated collision and outline bars.
#[mod_register]
fn register_content() -> SjmlResult<()> {
    let shape = BlockShapeBoxes::new([
        BlockAabb::new([0.0, 0.0, 0.0], [0.2, 1.0, 1.0])?,
        BlockAabb::new([0.8, 0.0, 0.0], [1.0, 1.0, 1.0])?,
    ])?;
    registry::register_block_with_shape_boxes("ruby_grate", BlockDefinition::default(), &shape)
}

fn main() {}
