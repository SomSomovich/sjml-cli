use sjml::{player, ErrorCode, PlayerHandle};

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn selected_hotbar_read_and_write_fail_closed_without_the_wasm_host() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::selected_hotbar_slot(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        player::set_selected_hotbar_slot(handle, 8)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn selected_hotbar_setter_rejects_nonexistent_slots_before_host_access() {
    let handle = PlayerHandle::new(1, 2, 3, 4).unwrap();
    assert_eq!(
        player::set_selected_hotbar_slot(handle, 9)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}
