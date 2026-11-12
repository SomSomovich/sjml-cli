# Server difficulty

The SDK exposes the Minecraft server's single difficulty setting as `world::WorldDifficulty`:

```rust
use sjml::{world, WorldDifficulty};

let current = world::difficulty()?;
world::set_difficulty(WorldDifficulty::Hard)?;
```

Reading requires `world.read`; writing requires `world.write` only. These operations are
server-only and run on the owning server thread. Difficulty belongs to the server's shared world
data, so the API takes no dimension handle and does not change per-dimension settings.

The four values are `Peaceful`, `Easy`, `Normal`, and `Hard`, matching Minecraft's difficulty IDs
0 through 3. The setter calls `MinecraftServer.setDifficulty` with `forced = false`, so a locked
world cannot be changed to a different difficulty. Hardcore worlds remain on `Hard`; requests for
another value fail. The API does not unlock difficulty. Successful changes update mob-spawning
flags and send vanilla difficulty updates to online players.
