# Simple Voice Chat: native .hnmod track

This branch adds `native/redescreen_voice/` following the official Horizon native-mod guide. It does not replace the original Horizon launcher or distribute another `horizon-client.exe`.

**Current stage: 0.3.0-alpha.1, non-functional voice diagnostic.** The native settings UI and protocol 20/21 secret parsing are being migrated. The original patched-client ZIP on main is historical development work. This .hnmod is **not ready for RedeScreen**.

Missing: Minecraft plugin-message integration, encrypted UDP transport, microphone, Opus encoding/decoding, playback, and live interoperability. The previous core uses aes-gcm, opus and cpal, which are not direct dependencies of stock horizon-client; the native bundle cannot simply add Cargo dependencies.

CI packages this source using the official horizon-mods native.py and checks compilation against Horizon. Do not submit to the registry until it actually works and the author chooses a public-source licence.
