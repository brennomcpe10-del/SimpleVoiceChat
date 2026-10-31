# Horizon Simple Voice Chat — RedeScreen Edition

**Status: experimental / not ready for Horizon registry submission.**

This repository hosts the **v0.2.0 source qualification workflow** for an experimental proximity voice-chat integration targeting Horizon Minecraft 26.3. It is **not** a working or reviewed `.hnmod`. The original Work output modifies the Horizon client source and adds dependencies; the official native-mod planner reportedly rejected the prototype with `E_REVISION` and `E_CRATE_DEPENDENCY`.

## Contents and workflow

The source is distributed in **`Horizon-Simple-Voice-Chat-RedeScreen-Edition-0.2.0-fontes (1).zip`** (original Work ZIP; SHA-256 `d01e9465e8729a4a52da0cc1a7dabfae0b7e1e83e0f676f3b2b8dd94d6917c68`). It contains the standalone Rust voice core, integration patch, protocol tests, documentation and test helpers. The original ZIP includes an upstream Horizon source snapshot, which CI discards and replaces with a pinned upstream checkout.

The manually triggered **Windows voice qualification** workflow:

1. Checks the ZIP's exact SHA-256.
2. Extracts the source and checks out the pinned official Horizon commit `a4873dec445a5f619bc2200069f65a76dc170a0c`.
3. Checks and applies `integration/horizon.patch` to that source.
4. Runs the Rust core tests and the local protocol interoperability checks.
5. Compiles the patched Horizon client on Windows x64.
6. Attempts a Windows Defender scan, and only if all steps pass uploads an isolated **test candidate** with SHA-256.

The candidate is **not a registry-compatible native mod**. Do not upload this ZIP or the resulting executable to [Horizon mod submission](https://playhorizon.dev/mods/submit). The registry requires a valid checked `.hnmod` and publicly auditable source corresponding to that artifact.

## Still unverified

- Real game startup and voice UI on Windows with Horizon game assets.
- A microphone and speakers on physical hardware.
- Voice connection and bidirectional audio on the RedeScreen server.
- Compatibility with the current official launcher and native mod packaging.
- Permission and license review before publishing as an independent mod.

**Do not replace an existing Horizon installation.** Test only with an isolated copy, verify the SHA-256, and retain the original client.

## Provenance

The archive was provided by the project owner after a ChatGPT Work development session. The integration is based on upstream [Horizon](https://github.com/horizon-rust/horizon) and [Simple Voice Chat](https://github.com/henkelmax/simple-voice-chat) protocol references. See `FONTES.md`, `TESTES.md` and `WINDOWS.md` inside the source ZIP for details. Upstream code remains under its own licenses; no new project-wide license is asserted here.

## Next milestone

Investigate a genuine native-mod implementation or upstream Horizon hook support for the necessary direct Minecraft custom payloads, audio devices, AES-GCM and Opus dependencies. Only submit to the registry after `cargo horizon-mod check` succeeds on the **actual `.hnmod`** and the functional tests are complete.
