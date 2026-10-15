# Control protocol v1

Development endpoint: `ws://127.0.0.1:47820/session`. Production through Tunnel:
`wss://<separate-together2-hostname>/session`. Authenticate the WebSocket upgrade
with an Authorization Bearer header. There are separate random PC/phone tokens
(at least 32 characters); never put credentials into URLs. One connection per
role. A second simultaneous connection receives HTTP 409. Invalid auth gets 401.
Only the PC may load a media identity. Both devices may request play/pause/seek.

Initial server message:
```json
{"type":"snapshot","server_ms":100,"state":{"protocol":1,"epoch":"boot-uuid","revision":1,"generation":0,"media":null,"desired_playing":false,"timeline":{"position_ms":0,"anchor_ms":0,"playing":false},"prepare":null,"participants":{"pc":{"connected":true,"buffering":false},"phone":{"connected":false,"buffering":false}},"fault":null}}
```

Clock request (Android currently sends every 500 ms while connected):
```json
{"type":"clock","protocol":1,"t0_ms":1234}
```
Reply echoes t0 plus t1/t2 server monotonic times and epoch. Capture t3 at receipt.
Client clock: Rust Instant on PC; elapsedRealtime on Android; no wall clocks.
Android accepts clock replies only for an outstanding exact integer t0 echo on
the current socket and epoch. Each echo is consumed once. The pending set retains
at most 16 requests for at most five seconds and resets on connection loss, open,
explicit disconnect and epoch change. Duplicates, unsolicited/fractional echoes,
expired replies and arrival times before the request cannot calibrate the clock.
Matched samples still pass the estimator's existing timestamp/RTT checks (RTT
0..1500 ms); the five-second request bound does not relax that limit. Calibration
retains the lowest-RTT of up to 64 valid samples for at most 30 seconds. At the
500 ms request cadence this covers the freshness window; the former eight-sample
capacity discarded still-fresh better calibration after about four seconds.
No clock requests are queued while the WebSocket is still connecting; failure to
enqueue a connected clock request enters the normal pause/reconnect path.

Each command has protocol, epoch, strictly increasing per-connection seq,
current generation and an action. Example:
```json
{"type":"command","protocol":1,"epoch":"boot-uuid","seq":1,"generation":0,"action":{"type":"load","media":{"provider":"torbox","torrent_id":123,"file_id":4}}}
```
Actions: `load {media}`, `play`, `pause`, `seek {position_ms}`,
`ready {position_ms}`, `buffering {active}`, `ended {position_ms,duration_ms}`,
`executed {playing,position_ms,observed_server_ms}`, `player_failed {code}`,
`player_recovered`. Failure codes are `load_failed`, `decoder_failed`,
`command_failed`, `unexpected_end`; raw player messages are never transmitted.
All time/position integers use ms.
The server rejects a stale epoch/generation and duplicate sequence. Ack includes
seq, error (null on success), server_ms and a full authoritative state.

Snapshots broadcast after accepted actions. Clients apply only newer revisions
in the same epoch; ack snapshots can be older than a previously received
broadcast, so the same revision filter applies. New epochs reset the filter.
`prepare != null` means pause, invalidate pending timers, load/seek as required,
then send readiness only after observed player readiness. `prepare == null` and
`timeline.playing == true` means schedule against the future anchor. Do not
unpause immediately on receipt. Late commit requires a new coordinated barrier.
Incremented generation invalidates queued events, URL requests and timers.

Server pings every 2 seconds and disconnects after 6 seconds without inbound
traffic. Clients answer ping, pause locally on control loss, reconnect with
bounded exponential backoff/jitter and recalibrate before reporting ready.
Snapshots never carry URLs. The device-specific source endpoint/message is
intentionally deferred until the resolver and pairing lifecycle are implemented.

Media identity now shares one Rust type across resolver and reducer: provider,
torrent_id, file_id, optional torrent_hash, filename, filesize, duration_ms,
imdb_id, season and episode. URLs/unknown fields are rejected. Hashes and supplied
metadata are validated before authoritative state changes. `fixture` reserves
0/0; `torbox` requires a nonzero torrent ID and permits file index 0. Omitted
provider is accepted only for the original 0/0 development fixture. This is an
additive v1 extension; clients that cannot resolve a provider must remain paused.

Android bounds connection attempts to 10 s and missing command acknowledgements
to 8 s. Retry base delays double from 1 s to 30 s, with up to 25% positive jitter
and a 30 s cap. A validated connection must last 30 s to reset failure history.
HTTP 401/403 stops retries and asks for pairing again. Invalid control responses
freeze local playback and reconnect. Fixture preparation is cancelled/fenced by
socket, server epoch and generation; old completions cannot authorize playback.

Message/frame limits are 16 KiB. Unknown fields and invalid actions are rejected.
Upgrade/pair attempts share a bounded server bucket (burst 60, refill 1/s).
Authenticated socket frames have a burst 80/refill 40/s limit; excess closes
with 1013. Token hashes are compared in constant time. Pairing HTTP bodies are
bounded to 4 KiB. Public tunnel setup and revocation lifecycle remain pending.

Native execution reports are distinct from control-command acknowledgements.
Each participant records the generation, observed native play/pause, position,
estimated server time of observation and server receipt time. Reports must match
the current committed state and position (within 500 ms), use observations no
more than 5 s old or 250 ms in the future, and never start before the anchor.
Playing reports require at least 20 ms of position advance. Both followers wait
for native state/position observations before reporting, rather than trusting
queue acceptance. Missing reports after anchor + max(2 s, 2 * RTT + 2 s), capped
at 12 s, freeze the room with an execution_timeout_pc/phone fault and a new
generation. Explicit Play/Seek retries with a new barrier. These acknowledgements
do not establish displayed-frame or audio presentation timing.

Terminal player failures freeze the room, invalidate the generation and preserve
user play intent. They cannot be cleared by buffering transitions. READY is
rejected while a participant has player_fault. A fresh explicitly retried load
must become natively usable before player_recovered starts another barrier.
An explicit pause during recovery remains paused. Native END_FILE error events
are fenced by playlist entry ID; obsolete source failures are ignored. Premature
EOF is classified separately as unexpected_end, rather than accepting a normal
end-of-media event.

The local Windows UI can open/cancel a five-minute one-use pairing window.
POST /pair accepts only its displayed 12-hex-digit code (hyphens/case ignored),
allows eight guesses, and returns the phone credential with Cache-Control:
no-store. There is no public route to open a window. Normal Windows credentials
are persisted in a current-user DPAPI vault under application local data;
Android stores the paired server address/token with AES-GCM and Android Keystore
in noBackupFilesDir. Subsequent launches use the stored connection. Android
requires HTTPS and rejects redirects, URL credentials, query/fragment and
non-root paths during pairing. The only HTTP exception is the isolated debug
loopback proof. Forgetting locally does not revoke the PC credential; remote
revocation/rotation remains future work before production exposure.

The Android control WebSocket also disables automatic redirects. A handshake
redirect fails through the existing transport-loss/reconnect path, which keeps
the saved validated /session endpoint; it cannot silently choose another path or
origin. The separate, unauthenticated APK download redirect policy does not apply
to pairing, control, media-capability requests or manual diagnostic uploads.

Snapshots now include `ended` (default false). Native EOF can end only current,
active playback with no prepare/fault, after the scheduled anchor, from a connected
participant. Reported position must be within 1 s of reported duration and the
authoritative clock within 2.5 s of it. Known media duration must match within
1 s. Durations beyond the existing seven-day position bound are rejected.
Accepted EOF increments generation, fixes position at duration, clears play intent
and retains a paused ended state. Connection/buffering changes preserve that end
state. Play from ended prepares at zero; seeking clears ended through a barrier.

Readiness position tolerance is 100 ms on both followers and the authoritative
reducer. Exact native seeks can land on the next video frame: the 10 fps soak
fixture landed 87 ms beyond a non-frame-aligned target. The old 80 ms limit
deadlocked that valid preparation until timeout. Positions beyond 100 ms are
still rejected; this bound does not change playback drift correction thresholds,
source fencing, clock freshness, stable paused readiness or the two-party barrier.

Android native position and its timestamp come from one serialized property
probe. Asynchronous playback-flow updates cannot replace that position while
retaining a different observation timestamp. Drift compares native position to
the authoritative timeline at the probe's client-monotonic time plus estimated
server offset; it never compares an old position to a later control tick or
extrapolates native position to hide a stalled player. Observations older than
1,000 ms or from the future cannot authorize correction. The nudge filter consumes
each native observation once. During native-probe loss, authoritative frozen
state and desired pause are observable separately from stale native player state;
fresh native paused/readiness observations are required after recovery.

Native readiness additionally requires the selected libmpv playlist entry ID and
usable position/duration properties. Android load receipts increase monotonically;
only the newest receipt can become ready. Anonymous FILE_LOADED callbacks no
longer authorize Windows readiness. Numeric entry IDs stay at the native boundary;
source URLs remain excluded from snapshots and diagnostics. Losing source readiness
during playback participates in the availability barrier, separately observable
from native cache buffering.

## Server lifetime and restart

Each Host owns one listener lifetime. Normal listener return or cancellation
marks that Host stopped, aborts its timer, and signals its upgraded WebSockets.
Every socket write is cancellation-aware, so a peer which stops reading cannot
hold authority indefinitely during shutdown. Closing a participant first marks
it disconnected/freezes the room, then attempts a 1012 `server_restart` close with
a 250 ms bound. An unresponsive peer may receive EOF instead of the close frame.
Stopped Hosts reject local playback commands and new pairing/upgrades; health
returns 503. Their listener cannot be restarted or concurrently served again.

A restart constructs a fresh Host with a new authority epoch, using the same
persisted pairing credentials. It sends the current snapshot, not missed command
history. Old-epoch commands are rejected. Current in-memory session state is not
persisted across a PC process restart: a fresh Host starts with no selected media.
Android treats the peer's onClosing callback as loss of control authority and
starts its pause/reconnect path without waiting for final socket teardown or the
silence watchdog. Peer-supplied close reason text is not logged.

Transport tests cover active-client close/freeze, resource release, rebind to the
same local address, authentication with the same credentials, new-epoch snapshot,
old-command rejection, pending-write cancellation and stopped pairing/health.
Actual native PC-process restart, Android close callback runtime and WAN/tunnel
interruption still require native/end-to-end validation.

## Native integration checkpoint
Participants include server-measured rtt_ms. Matching WebSocket ping/pong timestamps update RTT; client wall clocks do not set it. Commit lead is max(500 ms, 2 * largest participant RTT + 250 ms), capped at 5000 ms. This needs WAN fault testing. The trusted Windows controller is in-process and reserves its PC role; UI intentions enter the host reducer. Android serializes intention acknowledgements, generation-fences queued player effects, and requires fresh monotonic clock/native observations before READY. Both native workers freeze on stale health. The synthetic fixture uses torrent_id=0/file_id=0 only. Shared validated identity and PC-only resolver are implemented; safe production source delivery remains pending.


## Bounded writes and unresolved sources

Every ordinary outgoing control frame has a five-second write deadline in
addition to shutdown cancellation. This covers snapshots, acknowledgements,
clock replies and WebSocket ping/pong/close frames. A write timeout ends that
connection through the normal authority-freeze/role-release path. Shutdown
retains priority; the separate best-effort restart close remains bounded to250ms.
A real TCP/WebSocket backpressure test keeps upstream traffic alive while not
reading downstream, verifies the write-timeout event and frozen timeline, then
authenticates a replacement phone and verifies a fresh same-epoch snapshot.
Synthetic readiness/execution feedback drives this test, not native players.

Until safe source delivery is integrated, clients support only the canonical
fixture provider/torrent0/file0. Unsupported media changes must never reuse an
already-loaded native file as READY for the new identity. Windows reports a
terminal LoadFailed. Android validates every nonstale snapshot before consulting
its loaded flag; rejection enters its existing pause/disconnect path. Explicitly
loading a supported identity can recover through a new prepare barrier. This is
fail-closed behavior, not TorBox source-resolution support.
