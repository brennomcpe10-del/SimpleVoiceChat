// Native Horizon mixin runtime. No microphone or network in this milestone.
use std::cell::Cell;

pub const ID: &str = "redescreen_voice";
pub const VERSION: &str = "0.3.0-alpha.1";

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
