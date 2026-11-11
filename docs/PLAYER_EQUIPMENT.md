# Player equipment snapshots

`sjml::player::equipment(handle)` returns a detached snapshot of six equipment
slots for an online player: selected main hand, offhand, feet, legs, chest, and
head. An empty slot is `None`. A non-empty slot contains the canonical item ID,
stack count and maximum size, and current and maximum durability damage.

The module must request and receive `entity.read`. The operation is available
only to logical-server modules and resolves the current player for the live
login-session handle on the owning server thread. Client, static-registration,
stale-session, and wrong-thread calls are denied.

The API exposes exactly those six slots; it does not enumerate the player's
inventory, return item components or NBT, retain Minecraft objects, or provide
mutable item handles. IDs are limited to 256 bytes, stack sizes to 4096, and
durability values to one million. These bounds keep the fixed six-slot ABI
snapshot small and deterministic.
