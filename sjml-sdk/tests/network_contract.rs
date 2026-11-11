use sjml::network::{self, ChannelRef, NetworkContext};
use sjml::{from_bytes, to_bytes, ErrorCode};

#[test]
fn prelude_exports_the_typed_network_sdk_surface() {
    use sjml::prelude::{
        network as prelude_network, ChannelRef as PreludeChannelRef, LogicalSide,
        NetworkContext as PreludeContext, NetworkDirection, NetworkSubscriptionHandle,
        Player as NetworkPlayer, Uuid,
    };

    let channel = PreludeChannelRef::local("chat").unwrap();
    assert_eq!(prelude_network::ChannelRef::local("chat").unwrap(), channel);
    let _: Option<PreludeContext> = None;
    let _: Option<NetworkSubscriptionHandle> = None;
    let _: Option<NetworkPlayer> = None;
    let _: Option<Uuid> = None;
    let _: (LogicalSide, NetworkDirection) =
        (LogicalSide::Client, NetworkDirection::ClientToServer);
}

#[test]
fn local_channel_wire_resolves_owner_on_host() {
    let channel = ChannelRef::local("chat").unwrap();
    assert_eq!(
        to_bytes(&channel).unwrap(),
        vec![1, 0, 0, 0, 0, 4, 0, 0, 0, b'c', b'h', b'a', b't']
    );
    assert_eq!(
        from_bytes::<ChannelRef>(&to_bytes(&channel).unwrap()).unwrap(),
        channel
    );
}

#[test]
fn malformed_channel_identities_fail_before_host_access() {
    for invalid in [
        "",
        "Chat",
        "0chat",
        ".chat",
        "chat..reply",
        "chat.0reply",
        "chat/reply",
        "chat.",
    ] {
        assert_eq!(
            ChannelRef::local(invalid).unwrap_err().code(),
            ErrorCode::InvalidArgument
        );
    }
    assert!(ChannelRef::local(&"a".repeat(128)).is_ok());
    assert!(ChannelRef::local(&"a".repeat(129)).is_err());
    for invalid in ["", "owner.name", "0owner", "Owner"] {
        assert!(ChannelRef::new(invalid, "module", "chat").is_err());
        assert!(ChannelRef::new("owner", invalid, "chat").is_err());
    }
    assert!(ChannelRef::new(&"a".repeat(65), "module", "chat").is_err());
}

#[test]
fn explicit_owner_round_trips_without_dotted_string_ambiguity() {
    let channel = ChannelRef::new("demo", "server", "chat.reply").unwrap();
    assert_eq!(channel.package_id(), Some("demo"));
    assert_eq!(channel.module_id(), Some("server"));
    assert_eq!(channel.name(), "chat.reply");
    assert_eq!(
        from_bytes::<ChannelRef>(&to_bytes(&channel).unwrap()).unwrap(),
        channel
    );
}

#[test]
fn channel_wire_rejects_unknown_schema_and_scope() {
    assert_eq!(
        from_bytes::<ChannelRef>(&[2, 0, 0, 0]).unwrap_err().code(),
        ErrorCode::IncompatibleAbiVersion
    );
    assert_eq!(
        from_bytes::<ChannelRef>(&[1, 0, 0, 0, 2])
            .unwrap_err()
            .code(),
        ErrorCode::InvalidArgument
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_subscribe_accepts_typed_closures_and_reports_missing_host() {
    let channel = ChannelRef::local("chat").unwrap();
    let error = network::subscribe(channel, |_: NetworkContext, _: String| Ok(())).unwrap_err();
    assert_eq!(error.code(), ErrorCode::UnknownErrorCode);
}

#[test]
fn uuid_uses_canonical_display_and_exact_binary_roundtrip() {
    use sjml::network::Uuid;
    let uuid: Uuid = "00112233-4455-6677-8899-aabbccddeeff".parse().unwrap();
    assert_eq!(uuid.to_string(), "00112233-4455-6677-8899-aabbccddeeff");
    assert_eq!(
        to_bytes(&uuid).unwrap(),
        vec![
            0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd,
            0xee, 0xff
        ]
    );
    assert_eq!(from_bytes::<Uuid>(&to_bytes(&uuid).unwrap()).unwrap(), uuid);
    for invalid in [
        "",
        "00112233445566778899aabbccddeeff",
        "00112233-4455-6677-8899-aabbccddeefg",
        "00112233_4455-6677-8899-aabbccddeeff",
    ] {
        assert!(invalid.parse::<Uuid>().is_err());
    }
}

#[test]
fn context_wire_is_typed_and_does_not_grant_reply_authority() {
    use sjml::network::{LogicalSide, NetworkDirection};
    let mut bytes = vec![1, 0, 0, 0];
    bytes.extend(to_bytes(&ChannelRef::local("chat").unwrap()).unwrap());
    bytes.extend([1, 0, 0, 0, 2, 0, 0, 0, 0]);
    let context: NetworkContext = from_bytes(&bytes).unwrap();
    assert!(context.is_server());
    assert!(!context.is_client());
    assert_eq!(context.direction, NetworkDirection::ClientToServer);
    assert_eq!(context.side, LogicalSide::Server);
    assert_eq!(context.player_uuid, None);
    #[cfg(not(target_arch = "wasm32"))]
    assert_eq!(
        context.player().unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(to_bytes(&context).unwrap(), bytes);
    assert_eq!(
        context.reply(&17_u32).unwrap_err().code(),
        ErrorCode::StaleHandle
    );
    let mut unknown = bytes.clone();
    unknown[0] = 2;
    assert_eq!(
        from_bytes::<NetworkContext>(&unknown).unwrap_err().code(),
        ErrorCode::IncompatibleAbiVersion
    );
    // A C2S delivery must be received on the logical server.
    let side_offset = bytes.len() - 5;
    bytes[side_offset] = 1;
    assert_eq!(
        from_bytes::<NetworkContext>(&bytes).unwrap_err().code(),
        ErrorCode::InvalidArgument
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn native_explicit_sends_fail_closed_and_oversize_fails_before_host_access() {
    use sjml::network::{broadcast, send_to, send_to_server, MAX_MESSAGE_BYTES};
    use sjml::{Encoder, SjmlResult, StableEncode};
    struct Oversize;
    impl StableEncode for Oversize {
        fn encode(&self, encoder: &mut Encoder) -> SjmlResult<()> {
            encoder.write_raw(&vec![0; MAX_MESSAGE_BYTES + 1])
        }
    }
    let channel = ChannelRef::local("chat").unwrap();
    let player = network::Player::from_handle(sjml::PlayerHandle::new(1, 2, 3, 4).unwrap());
    assert_eq!(
        send_to_server(channel.clone(), &1_u32).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        send_to(channel.clone(), player, &1_u32).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        send_to(
            channel.clone(),
            sjml::PlayerHandle::new(1, 2, 3, 4).unwrap(),
            &1_u32
        )
        .unwrap_err()
        .code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        broadcast(channel.clone(), &1_u32).unwrap_err().code(),
        ErrorCode::UnknownErrorCode
    );
    assert_eq!(
        send_to_server(channel.clone(), &Oversize)
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );
    assert_eq!(
        send_to(channel.clone(), player, &Oversize)
            .unwrap_err()
            .code(),
        ErrorCode::PayloadTooLarge
    );
    assert_eq!(
        broadcast(channel, &Oversize).unwrap_err().code(),
        ErrorCode::PayloadTooLarge
    );
}
