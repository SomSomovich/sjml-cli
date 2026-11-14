use sjml::{
    registry::{
        register_fluid_with_type_properties, register_fluid_with_visuals_and_type_properties,
        FluidDefinition, FluidTypeProperties, FluidVisualDefinition,
    },
    ErrorCode,
};

#[test]
fn public_fluid_type_builder_enforces_neo_forge_bounds() {
    let properties = FluidTypeProperties::new()
        .density(-500)
        .unwrap()
        .temperature(1_200)
        .unwrap()
        .viscosity(4_000)
        .unwrap()
        .light_level(8)
        .unwrap();

    assert!(FluidTypeProperties::new().density(-10_001).is_err());
    assert!(FluidTypeProperties::new().density(10_001).is_err());
    assert!(FluidTypeProperties::new().temperature(-1).is_err());
    assert!(FluidTypeProperties::new().temperature(10_001).is_err());
    assert!(FluidTypeProperties::new().viscosity(-1).is_err());
    assert!(FluidTypeProperties::new().viscosity(10_001).is_err());
    assert!(FluidTypeProperties::new().light_level(-1).is_err());
    assert!(FluidTypeProperties::new().light_level(16).is_err());

    assert_eq!(
        register_fluid_with_type_properties("molten_salt", FluidDefinition::default(), properties)
            .unwrap_err()
            .code(),
        ErrorCode::UnknownErrorCode,
        "native SDK builds must fail closed without wasm host imports",
    );

    let visuals = FluidVisualDefinition::new(
        "example:block/molten_salt_still",
        "example:block/molten_salt_flowing",
    )
    .unwrap();
    assert_eq!(
        register_fluid_with_visuals_and_type_properties(
            "molten_salt",
            FluidDefinition::default(),
            visuals,
            properties,
        )
        .unwrap_err()
        .code(),
        ErrorCode::UnknownErrorCode,
    );
}
