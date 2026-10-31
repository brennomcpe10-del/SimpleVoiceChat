// Runs on the original Horizon client. Requests the official Simple Voice Chat secret
// over the existing Minecraft connection, without launching another client.
fn redescreen_voice_tick(game: &mut crate::game::Game<'_>) {
    use crate::game::redescreen_voice as voice;
    if !game.native_window() {
        return;
    }
    let ready = game.native_on(voice::ID)
        && game.remote.as_ref().is_some_and(|r| r.joined && r.logged_in && r.reconfiguring.is_none());
    let address = game.remote.as_ref().map(|r| r.session.address.as_str());
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
