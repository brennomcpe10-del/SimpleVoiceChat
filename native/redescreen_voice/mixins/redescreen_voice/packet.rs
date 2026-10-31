// Observe Simple Voice Chat plugin messages before Horizon handles normal packets.
// No server packet is blocked or rewritten.
fn redescreen_voice_packet(game: &mut crate::game::Game<'_>, packet: &horizon_protocol::packets::play::Clientbound) {
    use crate::game::redescreen_voice as voice;
    if !game.native_on(voice::ID) {
        return;
    }
    if let horizon_protocol::packets::play::Clientbound::CustomPayload(payload) = packet {
        if payload.channel == "voicechat:secret" {
            voice::on_secret(&payload.data);
        }
    }
}
