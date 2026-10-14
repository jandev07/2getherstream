# Together 2.0 architecture

Status: short native PC/emulator fixture proof passes. Production sources, pairing, WAN fault testing and physical-device acceptance remain pending.

```
Svelte UI -> Windows session controller -> authoritative PC session reducer
                                                |
                                          HTTPS/WSS control
                                                |
Compose UI -> Android session controller -> local sync engine -> libmpv

PC local sync engine -> libmpv -> device-specific TorBox HTTPS URL
Phone local sync engine -> libmpv -> a separate TorBox HTTPS URL
```

`sync-core` is a deterministic reducer with injected monotonic milliseconds.
`session-server` owns it behind one mutex and emits complete snapshots. The
Windows Tauri host runs the same server in-process, bound to 127.0.0.1:47820.
The server exposes only `/health` and `/session`. There is no media proxy route.

Session authority includes boot epoch, generation, revision, media identity,
position anchor, desired play intent, preparation and per-device availability.
Server epochs change on restart. Reconnecting clients discard their old local
clock samples and commands, receive a snapshot and join a new prepare barrier.
No solo playback fallback in milestone 1: losing either participant pauses both.

Load, seek, resume, buffer changes and connection changes invalidate preparation.
Both mandatory participants must reach the target before commit. A committed
timeline has a future server monotonic anchor; clients wait until its translated
local deadline before unpausing. Pause cancels scheduled playback immediately.
Prepared operations expire after 15 seconds into a visible fault and paused state.
The user's play intent survives buffering; an mpv pause is not a play/pause intent.

Clock sync uses t0/t3 from the client and t1/t2 from the server. RTT is
`(t3-t0)-(t2-t1)`; offset (server minus client) is
`((t1-t0)+(t2-t3))/2`. Keep the lowest RTT sample in the last eight samples,
expire samples after 30 seconds, reject invalid/large RTT and reset at reconnect.
Network asymmetry limits accuracy; clients must surface uncertainty. Current
commit lead uses a 500 ms floor and measured server ping/pong RTT: 2*RTT+250 ms, capped at 5 s. This is not a measured WAN guarantee.
Both followers reject late commits by requesting a new barrier. Before WAN acceptance, validate uncertainty
and acknowledge execution. Background/suspend invalidates clock readiness.

Steady playback now uses traced Syncplay/Synkplay code adaptations. Synkplay's
filtered speed nudge uses +/-0.5%, enters at 150 ms filtered drift and exits at
30 ms. Syncplay's larger correction ladder slows a player ahead by >1.5 seconds
to 0.95x, resets below 100 ms, requests rewind above 4 seconds, and observes lag
before requesting a >5-second behind correction. Hard corrections request a
coordinated Together barrier. Clock freshness, steady playback and the current
generation are prerequisites; buffering, stale clocks and generation changes
reset rate state. See THIRD-PARTY-NOTICES.md for the upstream pins and intentional
transport/authority differences. Both native followers are integrated for the local synthetic fixture; real media resolution and WAN acceptance remain pending.

TorBox resolver and shared identity are implemented: canonical identity is
provider plus torrent/file identity with optional hash, filename, size, duration,
IMDb and episode metadata. Validate it before changing authoritative state.
Safe source delivery is still pending: the live tested CDN URL contains the
account key, and is rejected before reaching Android. Resolve separately for
each device; do not use signed URL
equality as file identity. Account tokens stay on the PC/secure storage. Device
URLs are ephemeral, delivered only to their authenticated recipient, refreshed
on expiry through a new prepare barrier and excluded from diagnostics. Confirm
TorBox account permissions and any user_ip requirement with the official API and
real devices before relying on URLs outside the PC network.

Cloudflare uses a separate named tunnel and credentials for 2.0. Do not reuse or
stop the current Together tunnel. Only map the control origin, allow only control
paths, and reject other paths. The Android client requires normal HTTPS/WSS; it
needs no cloudflared app. Tunnel creation/deployment is not done in this starter.

Diagnostics record command acceptance, generation, revision, player position/target, rate, cache state, RTT/offset and reconnect samples as JSON. Native freshness is tracked independently. Preparation/commit execution timing, uncertainty, dropped messages and bounded rotating production logs remain pending. Never include signed URLs or pairing tokens.

Legacy UI/assets/chat/source-selection are references only. No legacy source was
copied. Chat, Stremio discovery, library, tracks and Pocket adaptation remain
outside milestone 1.
