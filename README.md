# 2gether

A Windows and Android app for watching together. Windows owns playback authority; both devices play media directly through their native players.

## Architecture

- Windows: Tauri 2, Rust, Svelte and libmpv.
- Android: Kotlin, Compose and libmpvkt.
- HTTPS/WSS transports pairing, playback commands and room state only. Video is not proxied through the control connection.
- Stremio selection runs on Windows. TorBox media delivery uses separately resolved URLs for the same canonical file.

## Development status

This repository starts with the validated source checkpoint `b289e4a` from the Together2 checkout. Later changes are committed after focused validation. The offered private Alpha 0.1.7 was compiled from `5ea1824`; it is not equivalent to the latest source.

Short Windows/Android emulator playback evidence exists. Revised Android integration, physical-phone acceptance, a real physical two-hour run and public WAN control remain open. Unit tests, compilation and synthetic clients do not establish those gates.

Public download hosting and Windows download QR integration are under development and are not yet deployed.

## Building and contributing

See `AGENTS.md`, `docs/ROADMAP.md`, `docs/MILESTONE-1.md`, `native/README.md` and `THIRD-PARTY-NOTICES.md`. Reuse the pinned toolchains and native dependencies. Keep private credentials, signing keys, local settings, test journals, build caches and release payloads out of source control.

Playback correctness and native-device validation come before extra features or a full visual redesign. Changes should state what was verified and distinguish source tests from installed-app and physical-device results.

The user-facing name is **2gether**. Existing Together2 repository paths and internal package identifiers remain unchanged.
