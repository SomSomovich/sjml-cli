# Entity custom-name visibility

Server modules can read or change whether Minecraft displays a loaded
non-player entity's custom name:

```rust
use sjml::{entity, EntityHandle, SjmlResult};

fn show_name(entity_handle: EntityHandle) -> SjmlResult<()> {
    if !entity::is_custom_name_visible(entity_handle)? {
        entity::set_custom_name_visible(entity_handle, true)?;
    }
    Ok(())
}
```

Reading requires `entity.read`. Changing the flag requires both `entity.read`
and `entity.custom_name.write`. The operations require a live handle to a loaded
non-player entity and run synchronously on the owning server thread. The setter
changes only Minecraft's custom-name visibility flag; it leaves the name and
other entity data unchanged and returns an error if the host does not observe
the requested value afterward.
