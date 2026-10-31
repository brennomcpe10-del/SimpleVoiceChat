# RedeScreen Voice — Horizon native adapter (0.3.0-alpha.1)

**NOT A FUNCTIONAL VOICE CHAT MOD.** Development-only native .hnmod, not a playable release.

This uses Horizon's official native hooks for Options > Mods, V and left Alt bindings, push-to-talk key state and settings. The existing Rust protocol 20/21 secret parser is included with unit tests; secrets are never persisted or logged.

Not implemented: Minecraft custom payload exchange, encrypted UDP, microphone, Opus codec, playback, proximity mixing, or live RedeScreen tests. The previous core needs aes-gcm, opus and cpal; these are not direct dependencies of the stock horizon-client, and .hnmod packages Rust mixins rather than Cargo dependency edits.

In a checkout of horizon-rust/horizon-mods, copy this folder to native/redescreen_voice and run:

```sh
(cd native/redescreen_voice && cargo test)
python3 native/tools/native.py package redescreen_voice
python3 native/tools/native.py check redescreen_voice
python3 native/tools/native.py check redescreen_voice --horizon latest
cargo horizon-mod check native/target/redescreen_voice-0.3.0-alpha.1.hnmod
```

The official launcher compiles native source into its own managed game instance. Do not install this expecting working voice. Registry submission requires full tests and an author-selected source licence.
