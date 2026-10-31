// Native Horizon mixin runtime. No microphone or network in this milestone.
use std::cell::Cell;

pub const ID: &str = "redescreen_voice";
pub const VERSION: &str = "0.4.0-beta.1";

thread_local! {
    static PTT_HELD: Cell<bool> = const { Cell::new(false) };
}

pub fn set_ptt(pressed: bool) {
    PTT_HELD.with(|held| held.set(pressed));
}

#[allow(dead_code)]
pub fn ptt_held() -> bool {
    PTT_HELD.with(Cell::get)
}

// Protocol 20/21 secret parser carried forward from the original Rust core.
// Secrets are never logged, persisted or sent to a diagnostic endpoint.
#[allow(dead_code)]
pub mod protocol {
    #[derive(Debug, PartialEq)]
    pub enum ParseError {
        Truncated,
        InvalidBool,
        InvalidUtf8,
        InvalidLength,
        TooLong,
        TrailingBytes,
    }

    #[derive(Clone, PartialEq)]
    pub struct SecretPacket {
        pub protocol: i32,
        pub secret: [u8; 16],
        pub connection_id: [u8; 16],
        pub voice_port: i32,
        pub player_uuid: [u8; 16],
        pub codec_id: u8,
        pub mtu_size: i32,
        pub voice_chat_distance: f64,
        pub keep_alive: i32,
        pub groups_enabled: bool,
        pub voice_host: String,
        pub allow_recording: bool,
    }

    struct Reader<'a> {
        bytes: &'a [u8],
        offset: usize,
    }

    impl<'a> Reader<'a> {
        fn take(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
            let end = self.offset.checked_add(n).ok_or(ParseError::InvalidLength)?;
            let bytes = self.bytes.get(self.offset..end).ok_or(ParseError::Truncated)?;
            self.offset = end;
            Ok(bytes)
        }
        fn byte(&mut self) -> Result<u8, ParseError> { Ok(self.take(1)?[0]) }
        fn i32(&mut self) -> Result<i32, ParseError> {
            Ok(i32::from_be_bytes(self.take(4)?.try_into().unwrap()))
        }
        fn f64(&mut self) -> Result<f64, ParseError> {
            Ok(f64::from_be_bytes(self.take(8)?.try_into().unwrap()))
        }
        fn id(&mut self) -> Result<[u8; 16], ParseError> {
            Ok(self.take(16)?.try_into().unwrap())
        }
        fn bool(&mut self) -> Result<bool, ParseError> {
            match self.byte()? { 0 => Ok(false), 1 => Ok(true), _ => Err(ParseError::InvalidBool) }
        }
        fn varint(&mut self) -> Result<usize, ParseError> {
            let mut out: u32 = 0;
            for i in 0..5 {
                let b = self.byte()?;
                if i == 4 && b & 0xf0 != 0 { return Err(ParseError::InvalidLength); }
                out |= u32::from(b & 0x7f) << (7 * i);
                if b & 0x80 == 0 { return Ok(out as usize); }
            }
            Err(ParseError::InvalidLength)
        }
        fn string(&mut self, max_chars: usize) -> Result<String, ParseError> {
            let n = self.varint()?;
            if n > max_chars * 3 { return Err(ParseError::TooLong); }
            let value = std::str::from_utf8(self.take(n)?).map_err(|_| ParseError::InvalidUtf8)?;
            if value.encode_utf16().count() > max_chars { return Err(ParseError::TooLong); }
            Ok(value.to_owned())
        }
    }

    pub fn parse_version(bytes: &[u8], protocol: i32) -> Result<SecretPacket, ParseError> {
        if protocol != 20 && protocol != 21 { return Err(ParseError::InvalidLength); }
        let mut r = Reader { bytes, offset: 0 };
        let secret = r.id()?;
        let connection_id = if protocol == 21 { Some(r.id()?) } else { None };
        let voice_port = r.i32()?;
        let player_uuid = r.id()?;
        let codec_id = r.byte()?;
        let mtu_size = r.i32()?;
        let voice_chat_distance = r.f64()?;
        let keep_alive = r.i32()?;
        let groups_enabled = r.bool()?;
        let voice_host = r.string(32767)?;
        let allow_recording = r.bool()?;
        if r.offset != bytes.len() { return Err(ParseError::TrailingBytes); }
        Ok(SecretPacket {
            protocol, secret, connection_id: connection_id.unwrap_or(player_uuid),
            voice_port, player_uuid, codec_id, mtu_size, voice_chat_distance,
            keep_alive, groups_enabled, voice_host, allow_recording,
        })
    }

    pub fn encode_request_secret(version: i32) -> [u8; 4] { version.to_be_bytes() }

    #[cfg(test)]
    mod tests {
        use super::*;
        fn fixture(protocol: i32) -> Vec<u8> {
            let mut b = vec![0x11; 16];
            if protocol == 21 { b.extend([0x22; 16]); }
            b.extend(24454i32.to_be_bytes());
            b.extend([0x33; 16]);
            b.push(0);
            b.extend(1024i32.to_be_bytes());
            b.extend(48.0f64.to_be_bytes());
            b.extend(1000i32.to_be_bytes());
            b.push(1);
            b.push(9);
            b.extend(b"127.0.0.1");
            b.push(0);
            b
        }
        #[test]
        fn parses_both_protocols() {
            for version in [20, 21] {
                let p = parse_version(&fixture(version), version).unwrap();
                assert_eq!(p.voice_port, 24454);
                assert_eq!(p.voice_host, "127.0.0.1");
                assert_eq!(p.connection_id, if version == 21 { [0x22; 16] } else { [0x33; 16] });
            }
        }
        #[test]
        fn rejects_invalid_lengths_and_trailing_bytes() {
            let b = fixture(21);
            assert!(matches!(parse_version(&b[..b.len()-1], 21), Err(ParseError::Truncated)));
            let mut bad = b;
            bad.push(0);
            assert!(matches!(parse_version(&bad, 21), Err(ParseError::TrailingBytes)));
        }
        #[test]
        fn request_version_uses_big_endian_i32() {
            assert_eq!(encode_request_secret(21), [0, 0, 0, 21]);
        }
        #[test]
        fn ptt_state_resets() {
            super::super::set_ptt(true);
            assert!(super::super::ptt_held());
            super::super::set_ptt(false);
            assert!(!super::super::ptt_held());
        }
    }
}

// Session negotiation uses Horizon's own Minecraft connection. It never
// creates a second client or sends credentials to any third-party endpoint.
use std::{cell::RefCell, time::{Duration, Instant}};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Register,
    Request(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stage {
    Disconnected,
    Waiting,
    SecretReceived,
    Unavailable,
}

struct Negotiation {
    address: String,
    attempts: u8,
    protocol: i32,
    next_attempt: Option<Instant>,
    stage: Stage,
    secret: Option<protocol::SecretPacket>,
    raw_secret: Option<Vec<u8>>,
    ipc: Option<Ipc>,
    last_hello: Option<Instant>,
    last_state: Option<Instant>,
    helper_status: Option<u8>,
    last_helper_reply: Option<Instant>,
    last_reported: Option<(bool, bool)>,
}
impl Default for Negotiation {
    fn default() -> Self {
        Self {
            address: String::new(),
            attempts: 0,
            protocol: 0,
            next_attempt: None,
            stage: Stage::Disconnected,
            secret: None,
            raw_secret: None,
            ipc: None,
            last_hello: None,
            last_state: None,
            helper_status: None,
            last_helper_reply: None,
            last_reported: None,
        }
    }
}
thread_local! {
    static NEGOTIATION: RefCell<Negotiation> = RefCell::new(Negotiation::default());
}

/// A maximum of four requests (21, 20, 21, 20) at six-second intervals.
pub fn on_tick(address: Option<&str>, ready: bool) -> Vec<Action> {
    NEGOTIATION.with(|cell| {
        let mut state = cell.borrow_mut();
        let address = address.unwrap_or_default();
        if !ready || address.is_empty() {
            if let Some(ipc) = state.ipc.as_ref() { ipc.stop(); }
            *state = Negotiation::default();
            return Vec::new();
        }
        if state.address != address {
            if let Some(ipc) = state.ipc.as_ref() { ipc.stop(); }
            *state = Negotiation { address: address.to_owned(), stage: Stage::Waiting, ..Negotiation::default() };
        }
        if state.secret.is_some() || state.stage == Stage::Unavailable {
            return Vec::new();
        }
        if state.next_attempt.is_some_and(|time| Instant::now() < time) {
            return Vec::new();
        }
        if state.attempts >= 4 {
            state.stage = Stage::Unavailable;
            return Vec::new();
        }
        state.protocol = if state.attempts % 2 == 0 { 21 } else { 20 };
        state.attempts += 1;
        state.next_attempt = Some(Instant::now() + Duration::from_secs(6));
        vec![Action::Register, Action::Request(state.protocol)]
    })
}

/// Reject malformed, unsolicited and out-of-session server packets.
pub fn on_secret(bytes: &[u8]) {
    if bytes.len() > 65536 {
        return;
    }
    NEGOTIATION.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.address.is_empty() || state.attempts == 0 || state.secret.is_some() {
            return;
        }
        if let Ok(packet) = protocol::parse_version(bytes, state.protocol) {
            if packet.voice_port > 0 && packet.voice_port <= u16::MAX as i32
                && packet.mtu_size >= 128 && packet.mtu_size <= 65535
                && packet.keep_alive > 0 && packet.keep_alive <= 60000
                && packet.voice_chat_distance.is_finite()
                && packet.voice_chat_distance >= 0.0
            {
                state.secret = Some(packet);
                state.raw_secret = Some(bytes.to_vec());
                state.stage = Stage::SecretReceived;
            }
        }
    });
}

pub fn status() -> &'static str {
    NEGOTIATION.with(|cell| match cell.borrow().stage {
        Stage::Disconnected => "Aguardando entrada em um servidor",
        Stage::Waiting => "Solicitando chave de voz ao servidor",
        Stage::SecretReceived => {
            if state_is_helper_online(&cell.borrow()) { "Conectado ao auxiliar de voz" }
            else { "Chave recebida; inicie o auxiliar de voz" }
        },
        Stage::Unavailable => "Servidor nao respondeu ao protocolo de voz",
    })
}


fn state_is_helper_online(state: &Negotiation) -> bool {
    state.last_helper_reply.is_some_and(|t| t.elapsed() < Duration::from_secs(3))
}
/// Sends only to 127.0.0.1. The secret is held in memory and never logged.
pub fn ipc_tick(
    position: [f64; 3],
    yaw: f32,
    players: &[([u8; 16], [f64; 3])],
    talk: bool,
    volume: u8,
) -> Option<(i32, bool, bool)> {
    NEGOTIATION.with(|cell| {
        let mut state = cell.borrow_mut();
        if state.secret.is_none() { return None; }
        if state.ipc.is_none() {
            state.ipc = Ipc::open();
        }
        if state.ipc.is_none() { return None; }
        let now = Instant::now();
        if state.last_hello.is_none_or(|last| now.duration_since(last) >= Duration::from_secs(2)) {
            if let Some(raw) = state.raw_secret.as_ref() {
                state.ipc.as_ref().unwrap().hello(&state.address, state.protocol, raw);
            }
            state.last_hello = Some(now);
        }
        if state.last_state.is_none_or(|last| now.duration_since(last) >= Duration::from_millis(100)) {
            state.ipc.as_ref().unwrap().state(talk, false, volume, position, yaw, players);
            state.last_state = Some(now);
        }
        if let Some(code) = state.ipc.as_ref().unwrap().status() {
            state.helper_status = Some(code);
            state.last_helper_reply = Some(now);
        }
        if !state_is_helper_online(&state) { return None; }
        let desired = match state.helper_status? {
            1 => (false, false),
            2 => (true, false),
            3 | 4 => (true, true),
            _ => return None,
        };
        if state.last_reported == Some(desired) { return None; }
        state.last_reported = Some(desired);
        Some((state.protocol, desired.0, desired.1))
    })
}

#[cfg(test)]
mod negotiation_tests {
    use super::*;
    #[test]
    fn requests_both_protocols_and_resets_on_disconnect() {
        assert!(on_tick(None, false).is_empty());
        assert_eq!(on_tick(Some("localhost:25565"), true),
            vec![Action::Register, Action::Request(21)]);
        assert!(on_tick(Some("localhost:25565"), true).is_empty());
        assert!(on_tick(None, false).is_empty());
        assert_eq!(status(), "Aguardando entrada em um servidor");
    }
    #[test]
    fn unsolicited_secret_does_not_create_session() {
        on_tick(None, false);
        on_secret(&[0; 100]);
        assert_eq!(status(), "Aguardando entrada em um servidor");
    }
}

// Self-contained IPC support, also compiled by the native crate's standalone tests.
// Nonblocking loopback-only link to the tiny Windows audio companion.
// The game itself is NEVER replaced or restarted.
use std::net::UdpSocket;

pub struct Ipc {
    socket: UdpSocket,
}
impl Ipc {
    pub fn open() -> Option<Self> {
        let socket = UdpSocket::bind("127.0.0.1:0").ok()?;
        socket.connect("127.0.0.1:37462").ok()?;
        socket.set_nonblocking(true).ok()?;
        Some(Self { socket })
    }
    pub fn hello(&self, address: &str, protocol: i32, secret: &[u8]) {
        if address.len() > 255 || secret.len() > 3072 || address.is_empty() {
            return;
        }
        let mut bytes = Vec::with_capacity(13 + address.len() + secret.len());
        bytes.extend_from_slice(b"RVH1");
        bytes.push(1);
        bytes.extend_from_slice(&protocol.to_be_bytes());
        bytes.extend_from_slice(&(address.len() as u16).to_be_bytes());
        bytes.extend_from_slice(address.as_bytes());
        bytes.extend_from_slice(&(secret.len() as u16).to_be_bytes());
        bytes.extend_from_slice(secret);
        let _ = self.socket.send(&bytes);
    }
    pub fn state(
        &self,
        talk: bool,
        deaf: bool,
        volume: u8,
        position: [f64; 3],
        yaw: f32,
        players: &[([u8; 16], [f64; 3])],
    ) {
        if !position.iter().all(|x| x.is_finite()) || !yaw.is_finite() {
            return;
        }
        let mut bytes = Vec::with_capacity(36 + players.len().min(24) * 40);
        bytes.extend_from_slice(b"RVH1");
        bytes.push(2);
        bytes.push((talk as u8) | ((deaf as u8) << 1));
        bytes.push(volume.min(100));
        for value in position {
            bytes.extend_from_slice(&value.to_be_bytes());
        }
        bytes.extend_from_slice(&yaw.to_be_bytes());
        bytes.push(players.len().min(24) as u8);
        for (uuid, xyz) in players.iter().take(24) {
            bytes.extend_from_slice(uuid);
            for value in xyz {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
        let _ = self.socket.send(&bytes);
    }
    pub fn status(&self) -> Option<u8> {
        let mut buffer = [0u8; 64];
        let mut latest = None;
        loop {
            match self.socket.recv(&mut buffer) {
                Ok(6) if &buffer[..5] == b"RVH1\x03" && buffer[5] <= 4 => latest = Some(buffer[5]),
                Ok(_) => {},
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                Err(_) => break,
            }
        }
        latest
    }
    pub fn stop(&self) {
        let _ = self.socket.send(b"RVH1\x04");
    }
}
