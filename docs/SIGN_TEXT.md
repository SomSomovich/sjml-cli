# Vanilla sign text

The block-entity API can read and replace the four plain-text lines on either face of an already loaded vanilla sign. Obtain the block-entity handle from a loaded position:

```rust
use sjml_sdk::block_entity::{self, SignSide, SignTextLines};
use sjml_sdk::{ErrorCode, SjmlError};

let Some(sign) = block_entity::get(world, position)? else {
    return Err(SjmlError::new(ErrorCode::InvalidArgument, "no sign at that position"));
};

let front = sign.sign_text(SignSide::Front)?;
let replacement = SignTextLines::new([
    "Welcome".to_owned(),
    "to the server".to_owned(),
    String::new(),
    "Enjoy your stay".to_owned(),
])?;
sign.set_sign_text(SignSide::Back, &replacement)?;
```

The selected side is `Front` or `Back`; each operation uses the same bounded four-line schema. Lines are UTF-8 text up to 2,048 bytes each and cannot contain control characters. Reads return the displayed component text as strings. Writes create literal components for both raw and filtered sign text, so they do not install click commands, hover actions, or arbitrary styles. The sign's dye color and glowing-text setting are preserved.

The handle must resolve to a vanilla sign block entity in an already loaded chunk. Calls run on the logical-server thread and never load chunks. Reading requires `world.read` and `block_entity.sign.read`. Writing requires `world.read`, `world.write`, and `block_entity.sign.write`; waxed signs are rejected. A module can write at most 32 sign updates per server tick.
