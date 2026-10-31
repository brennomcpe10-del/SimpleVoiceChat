// Observe decoded Minecraft custom payloads at the network session boundary.
// The original event stream is returned unchanged.
fn redescreen_voice_poll(events: Vec<Event>) -> Vec<Event> {
    use crate::game::redescreen_voice as voice;
    for event in &events {
        if let crate::net::Event::Packet(packet) = event {
            if let horizon_protocol::packets::play::Clientbound::CustomPayload(payload) = &**packet {
                if payload.channel == "voicechat:secret" {
                    voice::on_secret(&payload.data);
                }
            }
        }
    }
    events
}
