# Vanilla stonecutting recipes

Register one-input vanilla stonecutting recipes from `#[mod_register]` with
`content.register` requested in the module manifest and granted by the server
administrator. The declaration is staged before Minecraft freezes registries
and written as a `minecraft:stonecutting` recipe in SJML's generated server
data pack.

```rust
use sjml::prelude::*;

#[mod_register]
fn register_content() -> SjmlResult<()> {
    registry::register_stonecutting_recipe(
        "ruby_slab",
        registry::StonecuttingRecipeDefinition {
            ingredient: RecipeIngredient::tag("c:stones/ruby"),
            output_item: "ruby_slab".into(),
            output_count: 2,
        },
    )
}
```

The Rust API is:

```rust
pub struct StonecuttingRecipeDefinition {
    pub ingredient: RecipeIngredient,
    pub output_item: String,
    pub output_count: u32,
}

pub fn register_stonecutting_recipe(
    path: &str,
    definition: StonecuttingRecipeDefinition,
) -> SjmlResult<()>;
```

`RecipeIngredient::item("minecraft:stone")` selects an item; use
`RecipeIngredient::tag("c:stones")` for an item tag. A string converted with
`.into()` is an item ID, not a tag. Unqualified IDs resolve in the declaring
package and module. Namespaced IDs can address vanilla or another mod's
resources. The path is 1–128 lowercase ASCII letters, digits, `.`, `_`, or
`-`; the resulting recipe ID is `<package-id>:<module-id>/<path>`. Item IDs and
the encoded ingredient token are at most 256 ASCII bytes; the leading `#` for
a tag counts toward that limit. Output count must be 1–64. The SDK validates
these bounds before calling the host.

The generated JSON uses the vanilla shape
`{"type":"minecraft:stonecutting","ingredient":{"item":"..."},"result":{"id":"...","count":N}}`;
tag ingredients use a `tag` field. It is loaded during server data reload and
does not require a guest recipe callback or a client-side module. Vanilla
recipe IDs must be unique across shapeless, shaped, cooking, and stonecutting
families; guest callback serializer IDs are separate type declarations. The
host permits up to 256 aggregate staged content entries per module and 4,096
recipe declarations globally. Invalid ID syntax is rejected during
registration; a syntactically valid ID that does not exist is reported by
Minecraft's normal recipe reload validation.

## ABI

The host import is
`sjml_registry_v1.register_stonecutting_recipe` with exact signature
`(path_ptr: i32, path_len: i32, output_item_ptr: i32, output_item_len: i32,
output_count: i32, ingredient_token_ptr: i32, ingredient_token_len: i32) -> i32`.
The three pointer/length pairs address UTF-8 byte spans: content path, output
item ID, and ingredient token. The token is an item ID or a tag ID prefixed
with `#`; all multi-byte values follow the ABI's little-endian rules. The
output count is a scalar `i32` in the range 1–64. The function returns zero on
success or a stable nonzero ABI error.

Both `output_item` and `ingredient_token` are limited to 256 bytes at the host
boundary; malformed UTF-8, invalid resource IDs, invalid counts, unsafe paths,
duplicate IDs, closed recipe registration, and exhausted declaration quotas
are rejected. The import is bound only when `content.register` is both
requested and granted. It may be called only while `sjml_v1_mod_register` is
running; calls outside that static registration phase fail with
`PERMISSION_DENIED`. Unlike runtime world mutations, static registration is
the intended phase for this declaration.
