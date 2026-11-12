# Usable food items

Declare food behavior for a module-owned item during `#[mod_register]` with `registry::set_item_food`. The host adds Minecraft 1.21.1's vanilla food component, so normal right-click use, eating duration, hunger restoration, and saturation follow the game’s item behavior.

```rust
use sjml::{mod_register, registry::{self, ItemDefinition, ItemFood}, SjmlResult};

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_item("trail_ration", ItemDefinition::default())?;
    let food = ItemFood::new(6, 0.6)?.fast().always_eat();
    registry::set_item_food("trail_ration", food)?;
    Ok(())
}
```

`ItemFood::new` accepts nutrition from 1 to 20 and a finite saturation modifier from 0 to 10. `fast()` opts into vanilla’s shorter eating duration; `always_eat()` permits eating while hunger is full. Both flags are off by default. The host rejects unknown flag bits, out-of-range values, duplicate food declarations, and paths that do not refer to an item declared by the same module. A block item form is also supported.

The declaration requires `content.register` and must run while the static registration callback is active. Items without a food declaration retain their existing behavior. Custom use callbacks, food effects, container returns, and custom eating logic are not part of this API.
