# `sjml-macros`

Procedural macros used by [`sjml-sdk`](https://crates.io/crates/sjml-sdk).
Guest projects should normally depend on `sjml-sdk`, which re-exports these
macros, rather than adding this crate directly.

The SDK re-exports lifecycle attributes `#[mod_register]`, `#[mod_init]`, and
`#[mod_shutdown]`; callback attributes `#[event]`, `#[event_async]`,
`#[command]`, `#[service]`, `#[recipe]`, and `#[compute]`; and the
`StableEncode` and `StableDecode` derives. These generate the ABI entrypoints
and stable payload codecs used by the host.

See the [SDK guide](https://github.com/SomSomovich/sjml-cli/blob/master/docs/SDK_GUIDE.md)
for supported signatures, examples, schema annotations, and callback
constraints. The low-level calling convention is documented in the
[ABI v1 specification](https://github.com/SomSomovich/sjml-cli/blob/master/docs/ABI.md).
