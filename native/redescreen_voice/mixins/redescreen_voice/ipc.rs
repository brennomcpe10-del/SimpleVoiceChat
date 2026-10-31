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
