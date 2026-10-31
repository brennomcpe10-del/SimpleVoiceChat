// Runs on the original Horizon client. Requests the official Simple Voice Chat secret
// over the existing Minecraft connection, without launching another client.
fn redescreen_voice_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::redescreen_voice as voice;
    if !game.native_window() {
        return;
    }
    let ready = game.native_on(voice::ID)
        && game.remote.is_some() && game.player.is_some();
    let address = game.remote.as_ref().map(|r| r.session.address.as_str());
    if ready {
        for packet in horizon_client::net::redescreen_voice_bridge::drain() {
            voice::on_secret(&packet);
        }
    } else {
        horizon_client::net::redescreen_voice_bridge::clear();
    }
    let actions = voice::on_tick(address, ready);
    use horizon_protocol::packets::{configuration::CustomPayload, play::Serverbound};
    for action in actions {
        match action {
            voice::Action::Register => game.send(Serverbound::CustomPayload(CustomPayload {
                channel: "minecraft:register".into(),
                data: b"voicechat:secret\0voicechat:state\0voicechat:states".to_vec(),
            })),
            voice::Action::Request(protocol) => game.send(Serverbound::CustomPayload(CustomPayload {
                channel: "voicechat:request_secret".into(),
                data: protocol.to_be_bytes().to_vec(),
            })),
        }
    }
    if ready {
        if let Some(player) = game.player.as_ref() {
            let position = [player.position.x, player.position.y, player.position.z];
            let players = game.remote.as_ref()
                .map(|r| crate::game::remote::redescreen_positions::snapshot(r, position))
                .unwrap_or_default();
            let open = game.native_setting(voice::ID, "mode").as_deref() == Some("open");
            let talk = game.focused && (voice::ptt_held() || open);
            let volume = game.native_setting(voice::ID, "volume")
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(100).min(100);
            if let Some((protocol, muted, disabled)) =
                voice::ipc_tick(position, game.yaw, &players, talk, volume)
            {
                let (channel, data) = if protocol == 20 {
                    ("voicechat:update_state", vec![disabled as u8])
                } else {
                    ("voicechat:client_state", vec![muted as u8, disabled as u8])
                };
                game.send(Serverbound::CustomPayload(CustomPayload {
                    channel: channel.into(),
                    data,
                }));
            }
        }
    }
}
