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
}
