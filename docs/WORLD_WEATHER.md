# World weather API

The Rust SDK exposes `world::weather` and `world::set_weather` for server-side
modules. Weather is dimension-wide vanilla state; this API does not create
custom weather effects or load dimensions/chunks.

Both operations require a valid module-owned world handle from
`world::get`/`world::loaded`. Reading requires the module to request and receive
`world.read`. Changing weather additionally requires `world.write`. The host
binds the imports during static registration so compiled modules have stable
signatures, but calls from a registration callback return `PERMISSION_DENIED`.
Client modules cannot read or change server weather.

`sjml_world_v1.weather` accepts four `i32` arguments:

| Argument | Meaning |
| --- | --- |
| `handle_ptr` | Pointer to the 24-byte ABI v1 world handle. |
| `handle_len` | Must be 24. |
| `output_ptr` | Pointer to the output buffer. |
| `output_capacity` | Capacity in bytes; at least 18 bytes. |

It returns an `i32` ABI status. On success the host writes an 18-byte,
little-endian schema-v1 record: `u32 version` (`1`), `u8 raining`,
`u8 thundering`, then `u32 clear_ticks`, `u32 rain_ticks`, and
`u32 thunder_ticks`. Boolean fields are encoded as `0` or `1`. If the buffer is
too short, the call returns `PAYLOAD_TOO_LARGE` without writing a partial
record.

`sjml_world_v1.set_weather` takes four `i32` arguments:

| Argument | Meaning |
| --- | --- |
| `handle_ptr` | Pointer to the 24-byte ABI v1 world handle. |
| `handle_len` | Must be 24. |
| `condition` | `0` clear, `1` rain, or `2` thunder. |
| `duration_ticks` | Inclusive range `1..=1,728,000` (up to 24 in-game hours). |

Thunder uses vanilla rain and thunder timers together; clear weather uses the
vanilla clear-weather timer. Invalid conditions, zero/negative durations, and
durations above the limit return `INVALID_ARGUMENT`. A stale, foreign, unloaded,
or client-side world handle is rejected by the normal world-handle checks.
