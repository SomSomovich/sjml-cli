use sjml::{entity, EntityHandle, ErrorCode};

#[test]
fn ageable_mob_age_contract_is_bounded_before_host_access() {
    let handle = EntityHandle::new(1, 2, 3, 4).unwrap();

    assert_eq!(
        entity::set_age_ticks(handle, entity::MIN_ENTITY_AGE_TICKS - 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::set_age_ticks(handle, entity::MAX_ENTITY_AGE_TICKS + 1)
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
    assert_eq!(
        entity::set_age_ticks(handle, 0).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::set_baby(handle, true).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::age_ticks(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        entity::is_baby(handle).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(entity::AGE_WRITE_CAPABILITY, "entity.age.write");
    assert_eq!(entity::MAX_ENTITY_AGE_RESPONSE_BYTES, 8);
}
