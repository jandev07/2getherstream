# Android testing on your PC

The Desktop folder `2gether Android Emulator` contains `Open Android app.lnk`.
Open it when you want the phone window. Setup never opens it automatically.
It reuses the existing SidecarTest Android SDK emulator, with sound disabled.
It waits for a usable Alpha delivery receipt before installing the latest APK;
the broken Alpha 0.1.4 is not selected. Installation updates the app without
uninstalling it or clearing pairing and settings.

Use the mouse to tap, drag and scroll, and the keyboard to enter text. Open the
matching Desktop Windows Alpha. Pair the emulator with the PC address
`http://127.0.0.1:47820` and the pairing code shown by the Windows app. The
launcher forwards only this local control port through adb. No Cloudflare tunnel
or VPN is needed for this local test.

Leave the launcher's terminal open while testing. It holds the ownership lock
so autonomous development preserves the emulator, APK and local connection.
Close the emulator window when finished. If validation already owns the emulator,
the launcher asks you to wait and does not stop or restart it.

Try pairing, selecting a PC source, play/pause/seek, recovery and Send logs to PC.
Normal PC logs remain at
`C:/Users/astro/AppData/Local/app.together.v2.windows/diagnostics`.

The emulator helps verify screens and basic playback/control behaviour. Real
phone hardware decoding, battery use, mobile connectivity and uninterrupted
two-hour physical playback must still be tested separately.
