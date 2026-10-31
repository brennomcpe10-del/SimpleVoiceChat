// Observe decoded Minecraft custom payloads at the network session boundary.
// The original event stream is returned unchanged.
fn redescreen_voice_poll(events: Vec<Event>) -> Vec<Event> {

    for event in &events {
        if let crate::net::Event::Packet(packet) = event {
            if let horizon_protocol::packets::play::Clientbound::CustomPayload(payload) = &**packet {
                if payload.channel == "voicechat:secret" {
                    crate::net::redescreen_voice_bridge::push(&payload.data);
                }
            }
        }
    }
    events
}
