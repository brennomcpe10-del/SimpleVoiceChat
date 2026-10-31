//! RedeScreen voice helper. Runs alongside the ORIGINAL Horizon; no second game.
 //! Uses the existing audited v0.2.0 Opus/AES-GCM/CPAL worker.
use horizon_rede_voice::{
    audio::Options,
    secret::{parse_version, SecretPacket},
    session::Session,
};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::atomic::Ordering,
    time::{Duration, Instant},
};

const MAGIC: &[u8; 4] = b"RVH1";
const PORT: u16 = 37462;
const MAX_PLAYERS: usize = 24;

struct Active {
    address: String,
    secret: SecretPacket,
    peer: SocketAddr,
    session: Session,
    seen: Instant,
}
fn start_message(bytes: &[u8]) -> Option<(String, SecretPacket)> {
    if bytes.len() < 13 || &bytes[..4] != MAGIC || bytes[4] != 1 {
        return None;
    }
    let version = i32::from_be_bytes(bytes.get(5..9)?.try_into().ok()?);
    let address_len = u16::from_be_bytes(bytes.get(9..11)?.try_into().ok()?) as usize;
    if address_len == 0 || address_len > 255 {
        return None;
    }
    let offset = 11usize.checked_add(address_len)?;
    let address = std::str::from_utf8(bytes.get(11..offset)?).ok()?.to_owned();
    if address.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return None;
    }
    let secret_len = u16::from_be_bytes(bytes.get(offset..offset + 2)?.try_into().ok()?) as usize;
    if secret_len < 50 || secret_len > 3072 || bytes.len() != offset + 2 + secret_len {
        return None;
    }
    let secret = parse_version(bytes.get(offset + 2..)?, version).ok()?;
    if !(1..=65535).contains(&secret.voice_port)
        || !(128..=65535).contains(&secret.mtu_size)
        || !(1..=60000).contains(&secret.keep_alive)
        || !secret.voice_chat_distance.is_finite()
        || secret.voice_chat_distance < 0.0
    {
        return None;
    }
    Some((address, secret))
}
fn state_message(bytes: &[u8], active: &mut Active) -> bool {
    if bytes.len() < 36 || &bytes[..4] != MAGIC || bytes[4] != 2 {
        return false;
    }
    let count = bytes[35] as usize;
    if count > MAX_PLAYERS || bytes.len() != 36 + count * 40 {
        return false;
    }
    let flags = bytes[5];
    let volume = (bytes[6] as f32 / 100.0).clamp(0.0, 1.0);
    let mut position = [0.0f64; 3];
    for (i, value) in position.iter_mut().enumerate() {
        *value = f64::from_be_bytes(bytes[7 + i * 8..15 + i * 8].try_into().unwrap());
        if !value.is_finite() { return false; }
    }
    let yaw = f32::from_be_bytes(bytes[31..35].try_into().unwrap());
    if !yaw.is_finite() { return false; }
    let mut players = std::collections::HashMap::new();
    for i in 0..count {
        let off = 36 + i * 40;
        let uuid: [u8; 16] = bytes[off..off + 16].try_into().unwrap();
        let mut xyz = [0.0; 3];
        for (j, value) in xyz.iter_mut().enumerate() {
            *value = f64::from_be_bytes(bytes[off + 16 + j * 8..off + 24 + j * 8].try_into().unwrap());
        }
        if xyz.iter().all(|x| x.is_finite()) {
            players.insert(uuid, xyz);
        }
    }
    if let Ok(mut scene) = active.session.scene.try_lock() {
        scene.listener = position;
        scene.yaw = yaw;
        scene.players = players;
    }
    if let Ok(mut options) = active.session.options.try_lock() {
        options.volume = Some(volume);
    }
    active.session.talk.store(flags & 1 != 0, Ordering::Release);
    active.session.deaf.store(flags & 2 != 0, Ordering::Release);
    active.seen = Instant::now();
    true
}
fn code(active: &Active) -> u8 {
    if active.session.ended() { return 4; }
    match active.session.device_state() {
        Some((false, false)) => 1, // fully connected
        Some((true, false)) => 2,  // no microphone
        Some((_, true)) => 3,      // no playback device
        None => 0,                // UDP authenticating
    }
}
fn notify(socket: &UdpSocket, peer: SocketAddr, code: u8) {
    let _ = socket.send_to(&[b'R', b'V', b'H', b'1', 3, code], peer);
}
fn main() -> std::io::Result<()> {
    println!("RedeScreen Voice Helper - para o Horizon ORIGINAL 26.3");
    println!("Aguardando o mod na porta local 127.0.0.1:{PORT}.");
    println!("Inicie o Horizon, entre no servidor e pressione ALT esquerdo para falar.");
    println!("Nao feche esta janela enquanto jogar.");
    let socket = UdpSocket::bind(("127.0.0.1", PORT))?;
    socket.set_read_timeout(Some(Duration::from_millis(150)))?;
    let mut active: Option<Active> = None;
    let mut buf = [0u8; 4096];
    let mut previous_code = 255u8;
    loop {
        match socket.recv_from(&mut buf) {
            Ok((n, peer)) if peer.ip().is_loopback() && n >= 5 && &buf[..4] == MAGIC => {
                let bytes = &buf[..n];
                match bytes[4] {
                    1 => {
                        if let Some((address, secret)) = start_message(bytes) {
                            let same = active.as_ref().is_some_and(|a| a.address == address
                                && a.secret == secret && !a.session.ended());
                            if !same {
                                drop(active.take());
                                println!("Conectando voz ao servidor de Minecraft: {address}");
                                let session = Session::start(secret.clone(), address.clone(), Options::default());
                                active = Some(Active { address, secret, peer, session, seen: Instant::now() });
                                previous_code = 255;
                            } else if let Some(a) = active.as_mut() {
                                a.peer = peer;
                                a.seen = Instant::now();
                            }
                            if let Some(a) = active.as_ref() { notify(&socket, peer, code(a)); }
                        }
                    }
                    2 => {
                        if let Some(a) = active.as_mut() {
                            if a.peer == peer && state_message(bytes, a) {
                                notify(&socket, peer, code(a));
                            }
                        }
                    }
                    4 => {
                        if active.as_ref().is_some_and(|a| a.peer == peer) {
                            drop(active.take());
                            println!("Jogo desconectado; aguardando nova entrada.");
                            previous_code = 255;
                        }
                    }
                    _ => {}
                }
            }
            Err(ref e) if matches!(e.kind(), std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut) => {}
            Err(e) => eprintln!("Falha no IPC local: {e}"),
            _ => {}
        }
        if let Some(a) = active.as_ref() {
            let now = code(a);
            if now != previous_code {
                let label = match now {
                    0 => "Autenticando UDP...",
                    1 => "Voz conectada: microfone e alto-falante disponiveis",
                    2 => "Voz conectada: microfone indisponivel",
                    3 => "Voz conectada: saida de audio indisponivel",
                    _ => "Voz desconectada; aguardando reconexao",
                };
                println!("{label}");
                notify(&socket, a.peer, now);
                previous_code = now;
            }
            if a.seen.elapsed() > Duration::from_secs(5) {
                drop(active.take());
                println!("Horizon sem resposta; sessao de voz encerrada.");
                previous_code = 255;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_ipc_packets_are_rejected() {
        assert!(start_message(b"RVH1\x01").is_none());
        assert!(start_message(&[0; 40]).is_none());
    }
}
