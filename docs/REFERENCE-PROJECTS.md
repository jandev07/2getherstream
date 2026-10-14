# Read-only references

Located and inspected without modifying them:

- Together 1.0 root: `C:/Users/astro/Documents/tgt/Together`
- Windows source: `windows-rust/` (Rust/Slint), plus earlier `desktop/` material
- Android main app: `android/app/` (Kotlin/Compose, existing LibVLC dependency)
- Together Pocket: `android/companion/` (keep as its own app)
- Existing shared chat: `android/chat-shared/`
- Existing distribution: `releases/Together.exe`, `Together-Android.apk`,
  `Together-Pocket.apk`
- Private settings/data: `data/` (not imported or inspected for credentials)

Reuse candidates after milestone 1: OLED/glass/pink theme concepts, fullscreen
control design, icons, chat layout, source selection and Stremio/TorBox workflow.
Review licensing and dependencies before copying assets or implementations.

Do not copy legacy playback state, engine wrappers, synchronization loops,
pairing credentials, discovery secrets or old binaries into the 2.0 architecture.
Pocket API compatibility changes wait until the main apps have passed the
two-device playback gate.
