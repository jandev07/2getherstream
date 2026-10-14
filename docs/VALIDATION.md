# Validation record — current summary updated 10 October 2026

## Current acceptance boundary

Milestone 1 has **not** passed: a physical Android phone, safe direct TorBox media
delivery and public HTTPS/WSS control testing remain required. The proof runner
accepts emulator serials only; changing its serial does not make it a phone gate.

Latest playback-candidate session run394ebb7c completed05:06:55UTC on10October,
compiled code d4e7e65. All ten exact native methods raw-verified without skips;
1800requested uninterrupted seconds completed at800msRTT/100msjitter on the
emulator synthetic fixture.17940all-eligible matches: median32ms/p95=83.5ms/
max135.323ms;17806explicit-steady matches: median32ms/p95=83.5ms/max134ms.
Both pass the unchanged150ms provisional reported-position goal. Clock/sampling
error remains; screen/audio alignment and physical-phone acceptance are unmeasured.
Strict clock analysis has zero counted violations/fallbacks, support age max30000ms,
effective step max34.5ms/raw161ms, thirty windows without omissions. All twelve
before/after inputs match; work/interval-thirty-minute-394ebb7c has20verified files.

New pairing candidate113e61c was built separately and installed only after that
session's completion/evidence preservation. Its native encrypted-storage run
06f94035 completed05:37:37UTC: three exact PairingStoreTest methods raw-verified,
including invalid address/token replacement preserving existing ciphertext and
pairing.38JVM tests pass without failures/errors/skips. Candidate archive
work/pairing-address-native-06f94035 has20verified files including both APKs.
The installed pairing candidate has no new session-duration/HTTPS/two-hour result.
The latest session receipt must not be applied to these different Android APKs.
Checksums are not authentication/startup attestation. Safe TorBox, WAN control,
physical-phone two-hour and normal UI-exit gates remain open.

The preceding session run71dca3e3 completed04:31:45UTC on10October with
source commitd4e7e65. All ten exact native methods raw-verified without skips;
600requested uninterrupted seconds completed on the emulator synthetic fixture
at800msRTT/100msjitter. The unchanged150ms provisional p95 goal passed:6078
all-eligible matches, median32ms/p95=96ms/max156.5ms. Separate explicit-steady
analysis:5938matches, median32ms/p95=96.5ms/max135ms, signed median-23ms.
These are reported-position comparisons with remaining clock/sampling error.

Strict clock analysis records5938consistent intervals, zero fallback rows and
zero counted clock-age/history/connection/playback-state violations. Maximum
support age30000ms, support count62, effective offset step35ms and raw selected
step164.5ms; authority drift max87ms/native age max105ms. Ten clock windows,
zero omissions. All twelve captured inputs matched before/after the run.
Archive work/current-interval-ten-minute-71dca3e3 has20verified checksum files;
checksums are not authentication or startup attestation. This is ten-minute
emulator evidence, not a current-build two-hour or physical-phone release gate.

The preceding1800second request b3899306 FAILED after9515steady observations.
Its new support-age assertion observed30001ms because interval selection and age
reporting used separate clock reads. The corrected controller uses one captured
calibration instant; no freshness or playback threshold was widened. Original
failed receipt/evidence remains preserved, with22verified archive files and four
separate partial diagnostic files. It must not be described as a30minute pass.

Native HTTPS run09ba67f2 completed04:18:28UTC on10October: both exact Android
methods raw-verified, plus Windows TLS-verified load/play/pause/exact seek/source
replacement. API-host alias rejection preserved the ready native source and
source receipts. Archive work/passed-api-alias-https-09ba67f2 has six verified files.
This is pinned public AndroidX fixture delivery, not TorBox or WAN control.

## Historical artifact-specific results

The prior short session run fdc4bccb completed13:26:29UTC with the Windows position
sampling correction and unchanged ab1dd4d Android APKs. All ten intended native
methods raw-verified without skips;60requested uninterrupted seconds completed
at800msRTT/100msjitter. The provisional150ms goal was **missed**:735all-playing
matches, median89.5ms/p95=269ms/max325ms;594explicit-steady matches,
median86.5ms/p95=286ms. The signed comparison reports Android behind the
interpolated PC in585of594steady observations, signed median-86.5ms. Clock-offset
error remains; this is not measured screen/audio delay. Matched Windows position
reads had p95=0ms/max1ms; slow position reads do not explain this run's large gap.
Strict clock analysis has zero counted violations, selected-offset step max164.5ms,
authority drift max263ms and native age max108ms. Timing cause remains unresolved.
Receipt explicitly records native assertions passed and timing target false.
Archive work/current-position-sampling-fdc4bccb has21verified checksum files.
No new two-hour, phone, safe TorBox, WAN or normal UI-exit acceptance is claimed.

Offline clock-transition analysis of that saved run records three within-generation
offset changes. Its largest+164.5ms step accompanies authority drift-21→-189ms
over100ms, with fresh native samples and no intervening excluded rows. The selected
clock age changes1831→8ms. This is evidence of a clock-estimate contribution to
reported drift, not the true clock error or screen delay. All original summary
fields and timing failures are preserved; no fresh playback test was launched.

The preceding short session run e3b746b1 completed13:09:49UTC on the rebuilt Windows
e60a5ba app and Android queued-source-fence working tree. All ten intended native
methods raw-verified with no skips, including the new deterministic queued-load
regression;60requested uninterrupted seconds completed at800msRTT/100msjitter.
Its provisional150ms timing goal was **missed**: all-playing p95=154ms and a
separate explicit-steady p95=156ms. Native assertion completion does not mean the
timing target passed. The strict clock report has zero counted violations, but
that does not explain away this miss. Those artifacts have no two-hour soak or
physical-device acceptance. Archive work/current-short-source-fence-e3b746b1
preserves both metrics and the exact ten-method manifest;20checksum files verified.

The prior Windows build35b1e04 and Android build78ee60f passed run
89db0028-a195-42b9-aec0-99ad9a216801 on9October, completed05:52:21UTC. All nine
selected native tests have raw matched start/success packets with no ignored or
assumption-skipped tests. The requested7200second uninterrupted segment completed
with simulated800msRTT/100msjitter on a synthetic local fixture.

Subsequent native URL-guard source changes have now rebuilt the Windows app and
Android APK. These artifacts have a fresh short HTTPS acceptance82b2a956,
completed12:57:15UTC, with both exact Android methods raw-verified and Windows
TLS-verified playback/source replacement. They have not completed a new two-hour
session soak. Preserve the older two-hour archive as evidence for its recorded
artifacts; do not apply its result automatically to the rebuilt apps.

71377matched position samples: median50.5ms, p95=135ms, maximum281.5ms; the
provisional150ms p95 target passed. Strict clock analysis found zero counted
clock-age/history/connection/buffering/seeking/preparing/playing violations across
71244steady rows. These are reported-position measurements, not frame or audio
presentation alignment. They do not establish physical-phone, TorBox, 4K, WAN,
normal UI-exit or battery performance acceptance.

Evidence is preserved in ignored work/passed-current-two-hour-89db0028, with
17verified checksum manifest files. Checksums are not authentication. Application
hashes were recorded after completion and match the preceding short-run archive;
they are not startup attestation. See [the current development handoff](DEVELOPMENT.md)
for source/build boundaries and remaining blockers. Inspect the actual latest
receipt and ownership marker before operating native test resources.

## Historical checkpoints (8 October)

The entries below preserve the investigation history. Their references to active
runner PIDs, latest builds, test counts and pending emulator durations describe
their original checkpoints; they are not current process status or acceptance.

Canonical repository: C:/Users/astro/Documents/Together2. This supersedes the
initial shell checkpoint. Milestone 1 has NOT passed.

Previous two-hour run 81aa232a FAILED after 96.6 seconds of native tests:
fresh steady native observation (69 ms age) but selected clock offset changed
262.5 ms and authority drift jumped 99 -> 361 ms. No completed two-hour duration
or metrics. Archived under work/failed-clock-jump-81aa232a. A reproduced retention
gap (eight replies evict fresh calibration after ~4 s despite a 30 s freshness
bound) is addressed with 64-sample bounded history. 25 sync-core/16 Android JVM
tests pass, app/instrumentation compile; added numeric clock-selection diagnostics.
The subsequent native ten-minute result below validates this change at that
scope; the failed two-hour attempt remains preserved evidence.

Native retry 370285e3 on 6b2d893 passed five tests in 745.042 seconds, with 600
uninterrupted seconds and zero journal drops/errors. Drift 51/143/200.5 ms
median/p95/max, provisional p95 goal=true for this run. Selected clock age <=30 s,
history <=64; ten-minute emulator evidence only. A new 7,200-second run 4d821313
is active, PID 16588/start verified, no completed result. Protect its environment.

Control-only tunnel builder: four new tests plus cloudflared offline validation
and six rule matches pass. All 24 server tests pass; Windows metadata check
leaves active proof executable unchanged. No public tunnel/lifecycle integration
is claimed. See [tunnel boundaries](TUNNEL.md).

Ten-minute normalized-filter run adcdb076 passed five native tests in 740.856
seconds, with 600 uninterrupted seconds and zero journal drops/errors. Drift
median/p95/max 47.5/154/297 ms; provisional p95 goal=false. Archived evidence is
under work/passed-medium-adcdb076. This is not two-hour or physical validation.
New synchronous Windows command-fault handling passes three real-libmpv tests
and one Windows controller/Host integration test. Normal native regression
3eacaf06 passed five tests in 204.326 seconds, zero journal drops/errors,
median/p95/max 56/189/290 ms (provisional goal=false). No unexpected native
command fault appeared. Existing decoder/load faults retain their more specific
classification. A new 7,200-second emulator request is active on source 6836dcf,
runner PID 17748; no final result yet. Protect it and inspect its actual receipt.

Latest drift-controller source (8b3c57e): elapsed-time smoothing replaces a fixed
weight per observation, preserving thresholds/rates and rejecting duplicate or
reordered samples. 23 sync-core and 14 Android JVM tests pass. Fresh native short
run 0233c128 passed five tests in 201.004 seconds with zero journal drops/errors;
median/p95/max cross-device drift 53/159.5/210.5 ms. Provisional p95 goal remains
unmet; a single short run does not establish long-term improvement. A 600-second
follow-up is active under runner PID 18128; inspect its actual receipt/results.
The earlier two-hour pass below predates this filter change.

Completed long native emulator result: d8d2bfd2 passed/complete, 7,200 seconds
uninterrupted and five native tests passed. Cross-device median/p95/max are
87/224.5/416.5 ms; the provisional 150 ms p95 goal is NOT met. These are separate
from authority-relative assertions. See [native evidence](NATIVE-STABILITY-2026-10-08.md)
for preserved artifacts, sample freshness, worst-point explanation and exact
scope. No physical-phone/TorBox/Cloudflare acceptance is implied. A fresh short
native run 807f66ff against e9d73b9 also passed: five native tests in 205.751
seconds, a 60-second steady segment, fresh native/authority/command/control
journal with zero drops/errors. Cross-device median/p95/max 82.5/190/372.5 ms,
so provisional p95 goal=false. Actual PC restart and server-initiated Android
close callback still need specific runtime proofs. Both runs are archived.

Latest source validation: 20 session-server tests pass, including typed control
diagnostics for malformed input, protocol mismatch, heartbeat silence, rate
limiting and shutdown over actual WebSockets. Windows metadata checking passes;
the executable owned by active native run d8d2bfd2 is unchanged. This extension
is not in that running executable and awaits a fresh native proof. At the latest
check the runner was alive 76.4 minutes after launch, with no completed duration
or final metrics. Physical-phone/TorBox/two-hour acceptance remains open.

Android source follow-up: compilation and 12 JVM tests pass, including four new
clock request correlation tests. Only outstanding, one-use, unexpired integer
t0 echoes can reach calibration; unmatched replies cannot displace a good sample.
No APK was assembled or installed during the owned soak. This integration awaits
fresh native validation. Latest runner identity check matches at 106.4 minutes
after launch, still running with no completed duration or final metrics.

| Check | Evidence |
|---|---|
| Rust authoritative reducer / clock / drift tests | 12 passed |
| Actual loopback WebSocket transport tests | 5 passed |
| Android JVM clock/drift tests | 3 passed |
| Android native rendering and lifecycle instrumentation | Passed; decoded H.264/AAC, nonblack frame, paused load, play, stable pause, exact seek, preserved rotation, resume |
| PC + emulator native session instrumentation | Passed; prepare/READY/start, pause, seek, latest rapid seek, background, snapshot reconnect, resumed playback |
| Svelte checks | 0 errors, 0 warnings |
| Svelte frontend and native Tauri compilation | Passed |
| Repeatable proof runner | Passed: 2 real-player instrumentation tests, no skipped tests |
| Window video rendering inspection | Windows engine uses native child window and advances position; separate visible-frame inspection remains pending |
| Real TorBox independent URL streaming | Pending |
| Cloudflare/WAN jitter and buffering faults | Pending |
| Physical phone / 4K hardware decoder | Pending |
| Real two-hour stability | Pending |

Latest recorded two-player proof completed two instrumentation tests in 36.276 s.
It samples 100 ms player diagnostics and exercises a 12 s uninterrupted segment
inside the scenario, plus debug-injected loss of native health observations, room hold, explicit user pause, recovery without auto-resume and explicit play. This exercises the native-health barrier, not an actual CDN stall. Both engines are real; each decodes a separate local copy
of a synthetic 60 s fixture. The Android test captures the actual native frame.

Measured estimated inter-player position difference:

| Samples | Median | 95th percentile | Maximum |
|---:|---:|---:|---:|
| 161 | 17.5 ms | 61 ms | 95.5 ms |

Comparison uses the same generation, Android's estimated server-clock offset,
nearest PC sample within 120 ms and rate interpolation. These are sampled
position estimates, not frame/audio presentation timestamps. They do not prove
WAN accuracy, movie-length stability or physical hardware decoding performance.
A previous local pass recorded 138 samples, median 17.5 ms, p95 64 ms, max 102 ms.

Investigation caught a real foreground-recovery stall in the initial Android
SurfaceView integration. The native call blocked the UI in the published view's
surface callback. Worker-only callbacks kept UI responsive but still exposed a
surface destruction race. The current TextureView adapter retains its texture
until native detach finishes, serializes native work off Main and requires fresh
worker observations for readiness. Full native/lifecycle proof was rerun after
this fix, with no skipped tests. Both roles freeze on stale native health; late
scheduled commits request a fresh coordinated barrier instead of starting late.

Server-measured ping/pong RTT extends commit lead above 500 ms, capped at 5 s.
An 800 ms RTT reducer test verifies a 1.85 s lead. This is algorithm evidence;
actual delayed/jittered WAN/native-device transport has not been exercised yet.

Reproduce: scripts/Prepare-PlayerProof.ps1 then scripts/Test-SessionProof.ps1.
Inspect ignored work/native-proof-result.txt, native-proof-summary.json,
phone-session-proof.jsonl and native-session-proof.jsonl. The runner uses only
an emulator and loopback adb transport, temporary private debug pairing, and
stops its own PC host. Together 1.0, Pocket and their configuration stay untouched.

## 2026-10-08 resolver / TLS checkpoint

The Rust suite passes 12 reducer tests, 5 real WebSocket tests and 5 resolver
tests (including transport/API failure redaction and credential-bearing URL
rejection). Android JVM tests and main/instrumentation builds pass.

After installing the system-root PEM bundle in the native adapter, the connected
emulator proof passed both native tests again in 36.731 seconds, with no skips:
160 matched position samples, median 15.5 ms, p95 50.5 ms, maximum 81.5 ms.
These are the same short local-fixture/sampling boundaries described above.
This proves the trust-bundle initialization did not regress existing native
playback; it does not establish native HTTPS playback or TorBox streaming.

The live Rust TorBox library query succeeded and found 1,614 available video
files. The tested returned CDN URL contained the account API key, which the
resolver rejected. Live two-device TorBox playback remains blocked by that
credential boundary. No account-bearing URL was sent to Android. Details and
provider references are in TORBOX.md. Stremio credentials were stored using
Windows DPAPI only; no Stremio account login or feature migration was performed.

## Reconnect / HTTPS / delayed native transport checkpoint

24 Rust tests pass: 12 reducer, 2 media identity, 5 real WebSocket and 5 resolver.
Six Android JVM tests pass, including bounded/flapping reconnect backoff and
rejection of delayed clock samples without losing valid calibration. Native
Windows compilation and Android main/instrumentation builds pass.

Windows and emulator native HTTPS load/play/pause/exact seek pass with TLS
verification enabled, streaming the public AndroidX MP4 test asset directly:
`androidx/media` revision `8c6678b657ede1e7883fc164ef73ed483c7796c3`,
`libraries/test_data/src/test/assets/media/mp4/sample.mp4`. The Android test
passed in 4.124 s. This is a small public stream, not TorBox or 4K acceptance.

The connected synthetic fixture is now 180 s. Every Android preparation installs
its packaged copy atomically, so old installed fixtures cannot invalidate a run.
A debug-only TCP relay delays only loopback control traffic (never media), in
each direction by half the configured RTT with deterministic bounded jitter.
The real WebSocket clocks, commands, ping/pong, snapshots and reconnects all
traverse it. This is controlled local transport impairment, not Internet/WAN.

Each case passed both native tests with no skips, including a forced live socket
loss, local freeze, full snapshot reconnect and resume of preserved play intent:

| Configured RTT | Per-direction jitter | Runtime | Matched samples | Median position difference | p95 | Maximum |
|---:|---:|---:|---:|---:|---:|---:|
| 100 ms | +/-20 ms | 42.856 s | 163 | 61.5 ms | 95 ms | 125 ms |
| 300 ms | +/-50 ms | 53.248 s | 167 | 20.5 ms | 57 ms | 68 ms |
| 800 ms | +/-100 ms | 85.292 s | 184 | 25 ms | 108.5 ms | 135.5 ms |

Measured clock RTT medians were 263 ms (300 ms case) and 881 ms (800 ms case).
Jitter, packet coalescing and scheduling mean configured and measured values
are distinct. Position comparison retains the interpolation/clock/property
sampling uncertainty described above; it is not a presentation timestamp test.

The initial 800 ms run failed background reconnect. Investigation found the
Android estimator threw on timing outliers above 1.5 s, and the hardened response
handler then tore down a valid socket. Such samples now return no estimate;
valid existing samples remain until their normal expiry. The full 800 ms run
passed after the fix. Unsupported/malformed control snapshots still freeze and
reconnect. Reconnect reason logs contain event classes only, never raw errors.

Connection attempts have a 10 s bound; missing command acknowledgements trigger
recovery after 8 s. Retries double from 1 s to 30 s with bounded jitter; brief
successful handshakes do not reset history. 401/403 stops retries. Source fixture
jobs are cancelled and fenced by epoch/generation/socket, with generic errors.

Reproduce delayed transport: `scripts/Test-ControlLatency.ps1` (or call
`Test-SessionProof.ps1 -ControlRttMs 800 -JitterMs 100` for one case).
Reproduce HTTPS: `scripts/Test-HttpsProof.ps1`. Both accept emulators only.
Runner cleanup removes private proof pairing and adb reverse mappings and kills
only the processes it created. Physical-phone/movie-length gate remains open.

## EOF, replay and native source identity checkpoint

26 Rust tests pass (14 reducer, 2 media identity, 5 WebSocket, 5 resolver), six
Android JVM tests pass, and both native/debug builds pass. Native source readiness
uses mpv numeric playlist entry IDs plus usable native duration/position. Windows
no longer trusts anonymous FILE_LOADED events. Android ties readiness to the latest
load receipt and rejects effects queued before invalidation. A missing source
freezes shared availability even when paused-for-cache is false.

Windows HTTPS load/play/pause/seek and rapid HTTPS-to-local replacement passed
with native entry readiness. Android passed both HTTPS and stale queued-source
replacement tests in 7.968 s, no skips. The replacement test invalidates an older
HTTPS request, selects the local fixture, then requires the newest receipt,
correct source duration, native readiness and advancing playback.

The connected suite now holds continuous playback for 60 s and explicitly seeks
near EOF, waits for native EOF to become authoritative ended state, then verifies
Replay prepares zero and playback advances. Reducer tests separately reject stale,
premature, paused and seeking EOF reports, and preserve ended across reconnection.

| Configured RTT/jitter | Native test runtime | Matched positions | Median | p95 | Maximum |
|---|---:|---:|---:|---:|---:|
| 300 ms / +/-50 ms | 110.009 s | 659 | 17.5 ms | 48.5 ms | 98.5 ms |
| 800 ms / +/-100 ms | 152.802 s | 682 | 31 ms | 130.5 ms | 194.5 ms |

The 300 ms run validated EOF/replay before entry-ID readiness was added. The final
800 ms run includes all new source guards and availability handling. Both passed
two native tests without skips. Final median clock RTT was 835 ms; maximum accepted
sample 1498 ms. These are estimated positions, not presentation timestamps, and
at most one minute of uninterrupted playback, not the required two-hour gate.

The alternative TorBox createstream investigation returned HTTP 500 for a tested
available MP4 with scrobbling disabled. It established no usable safe capability.
The account-bearing CDN URL remains rejected before Android delivery.

## Native execution, terminal errors and persistent pairing

The PC now records current-generation native play/pause observations and freezes
the room if a committed state is not executed in its RTT-scaled deadline. The
connected test suppresses Android's native start, verifies the timeout, and
explicitly retries. Source-fenced native END_FILE errors are tested with invalid
local media, on Windows and Android. The connected decoder-failure case holds
the room until an explicit retry and preserves a pause through reload recovery.
A discovered readiness/reload race was fixed by distinguishing accepted load
receipts from source readiness. Same-code failed retries have reducer coverage.

Persistent pairing has Windows current-user DPAPI and Android AES-GCM/Keystore
round-trip/tamper tests, initial one-use HTTP claim/replay rejection and an actual
authenticated socket connection with a restored Android credential. HTTPS,
credential-URL and address validation are enforced; live HTTPS pairing through a
named tunnel is not yet tested. Socket/upgrade limits and fixed-size constant-time
token hash comparisons are implemented. Public tunnel/revocation are pending.

| Fixture / control impairment | Runtime | Native tests | Samples | Median | p95 | Maximum |
|---|---:|---:|---:|---:|---:|---:|
| 180 s / 800 ms RTT, +/-100 ms jitter | 192.804 s | 5 passed | 714 | 55.5 ms | 164.5 ms | 240.5 ms |
| 7,500 s compact / 800 ms RTT, +/-100 ms jitter | 200.58 s | 5 passed | 723 | 63 ms | 127.5 ms | 203 ms |

Both used only a 60-second uninterrupted segment, with no skips. The first run
exceeded the provisional p95 150 ms target. Longer synthetic media does not
itself constitute a longer stability test. The runner now supports an explicitly
requested 7,200-second uninterrupted segment, and checks that the generation does
not change during it. Presentation, physical device, real TorBox and 4K gates
remain open. Rust coverage is 35 tests; existing six Android JVM tests pass.
The bounded indexed analyzer reproduces the former comparison on the short logs
without quadratic work. API-key URL guards also reject encoded/case variants.
# 2026-10-08 readiness recovery regression

Connection-lifetime follow-up: all 19 server tests pass. New transport coverage
stops an active WebSocket, checks authority freeze and Host release, rebinds the
same local address, authenticates with unchanged test credentials and rejects an
old-epoch command. Additional checks cover pending write cancellation and stopped
health/pairing admission. Eight Android JVM tests/Kotlin compile and Windows
metadata checks pass. Android onClosing now enters its existing reconnect path;
native runtime/restart and WAN tests remain pending the active soak's exit.

Command-journal follow-up: all 16 server tests pass, including local/real WebSocket
accepted and rejected command projection, secret exclusion and cancellation of
the server's owned ticker. Windows metadata checking passed without changing the
running soak executable. Cumulative Rust test coverage is 43. This source change
still needs native runtime coverage after the active d8d2bfd2 soak exits.

Latest observation-fix short proof: all five native tests plus live production
journal validation passed in 200.916 seconds (956cac57). No journal drops or
storage errors. 730 matched samples: median 57 ms, p95 158.5 ms, max 206 ms;
RTT median 882 ms, max 1370 ms. Maximum sample age in 610 steady-generation
observations was 107 ms. A subsequent direct HTTPS/source-replacement regression
passed on native Windows and Android (two tests, 9.589 seconds). The next soak
adds a <=350 ms native observation-age assertion; compiled, execution pending.
This remains short emulator evidence; two-hour and physical-device gates are open.

The subsequent requested two-hour run failed after 5,161.857 seconds of total
instrumentation with excessive measured Android drift. The exact failing native
sample was missing from the old log (assert preceded record). The completed
duration remains null; this is failed long-run evidence. Follow-up code removes
competing Android position writers and compares timelines at native observation
time, with explicit sample-age logging and record-before-assert. Eight Android
JVM tests and 14 server tests pass. A native short retry is in progress; the first
retry uncovered a stale-probe test assumption, now corrected to use authoritative
freeze/intent during an outage and fresh native pause only after recovery.

Production journal follow-up: five additional diagnostic tests passed, bringing
server coverage to 14 tests and cumulative Rust coverage to 41. Windows metadata
checking and frontend checking pass. The existing native soak executable was
verified unchanged. The new journal/export code is not part of that running
soak; native runtime, UI export and timing overhead checks are pending its exit.
Detailed journal validation and boundaries: [DIAGNOSTICS.md](DIAGNOSTICS.md).

An unattended two-hour request failed after 104.975 seconds during setup:
native players landed at 26,600 ms for target 26,513 ms, outside the old 80 ms
readiness limit. No two-hour segment completed. A shared 100 ms readiness bound
now accepts a frame-rounded exact seek while preserving stale-generation,
source, stable-pause, buffering and clock checks. Drift thresholds are unchanged.

With the compact 7,500-second/10 fps synthetic fixture, the corrected five-test
native suite passed in 197.068 seconds at configured 800 ms RTT / ±100 ms jitter.
The suite now deliberately seeks between video frames and requires recovery to
clear the authoritative fault. Its 60-second uninterrupted segment produced
719 matched samples: median 75.5 ms, p95 183.5 ms, maximum 241.5 ms. Clock RTT
median was 888 ms, maximum 1308 ms. p95 exceeds the provisional 150 ms goal.
These are sampled native positions, not measured screen presentation times.
All 36 Rust tests, six Android JVM tests, frontend checks and native builds pass.
The physical-phone/TorBox/two-hour acceptance gate remains open.

## 2026-10-10 Current pairing and local command-send recovery

The current pairing-address/token APKs (113e61c) completed native run553cd5b4:
all ten exact methods, no skips,1800uninterrupted seconds at800msRTT/100msjitter;
17807steady matches, median31.5ms/p95=89.5ms/max137.5ms. Archive
work/current-pairing-thirty-minute-553cd5b4 has21verified files and all twelve
captured binary/DLL/APK/media/source inputs match. Android build association is
embedded in the receipt; this does not attest the Windows binary or installed app.

After preserving that run, newer Android f94c04a pauses and reconnects immediately
when the local socket refuses a command and stops following its captured old state.
Its separate ace8e422 native run passed all ten methods with only60steady seconds;
594steady matches, median37.5ms/p95=114.5ms/max135.5ms. Its unsent pause was not
replayed; PC play intent survived reconnect. Both current APKs also passed the two
exact HTTPS methods in b287f15e, with Windows native HTTPS regression passing.
The prior30minute result does not apply to this newer APK automatically.

PC f960d66 rejects header-unsafe tokens both at Host startup and decrypted-vault
loading, preserving invalid ciphertext. Isolated server tests:52library passed,
eight intentionally ignored fixture entry points,13transport passed. Android38JVM
methods pass without failures/errors/skips. New build receipts reject stale APK/
source associations; standalone Android builds share exclusive native ownership.

ace8e422 raw PC evidence has one incomplete post-exit final JSON row. Its explicit
steady report opts into malformed unterminated-tail recovery and reports that
omission. The original raw bytes and runner metrics remain archived; complete bad
records and valid wrong-run tail records still reject. Synthetic analysis tests pass.
Source b368754 records this diagnostic-only change. Checksum inventories are not
authentication or startup attestation. Physical-phone two-hour, safe TorBox delivery,
WAN control, current-candidate extended stability and normal UI-exit remain open.
