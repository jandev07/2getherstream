# Autonomous development handoff

The user explicitly asked to continue development while they sleep, without
asking for a new continue message after each checkpoint. A thread heartbeat
automation, `continue-together-2-0-development`, is active every 5 minutes.
Keep working within this repository and its AGENTS.md. Do not create duplicates.
Notify only for meaningful work, failure, completion or an unavoidable decision.

## Current evidence and next constraints (9 October, 05:52 UTC)

The current Android artifacts are built from78ee60f; Windows is built35b1e04
(delayed heartbeat RTT fix). Run89db0028 passed all9raw-verified native tests and
7200steady seconds at800msRTT/100msjitter (p95=135ms; provisional150ms goal met).
Its archive is work/passed-current-two-hour-89db0028 with17verified manifest files.
The runner completed and removed its ownership marker. Inspect the marker and
latest receipt before any future native operation; do not launch repeat soaks
without a new change or unresolved concern that justifies them.

The completed7200second emulator run04311a62 belongs to the earlier Android and
Windows build3ab4e18, with terse8test output and p95=127.5ms. Preserve its archive
work/passed-2gether-two-hour-04311a62. The new89db0028 run covers later native
lifecycle/idle-polling and Windows RTT changes. None of these
synthetic loopback/adb proofs satisfies the physical-phone/TorBox/WAN release gate.

Direct safe TorBox delivery remains the primary provider blocker. requestdl's
account-bearing URL was rejected; T3 query signing did not establish valid media;
the documented redirect=true diagnostic returned307 with a rejected Location.
Neither experimental feature is enabled in either app. Do not repeat those
identical probes or weaken guards. A public frontend-bundle fetch returned403;
there is no inspected frontend contract or safe final capability to integrate.

Keep vgate0/WARP independent from the app. Read-only inventory still finds vgate0
disconnected, but no matching RustDesk/vgate/Wintun/WireGuard service or uninstall
entry established its owner. Do not infer ownership from the generic driver name,
remove the adapter or change network settings. Physical-phone testing and normal
UI-exit validation also remain pending. Continue core work where evidence supports
a useful change; avoid duplicate tests or speculative patches just to stay busy.

## Running native tests

Always inspect `work/native-proof-running.json` before builds, fixture generation
or emulator operations. It contains the owned runner PID and process start time.
Verify both, because PID reuse is possible. While the runner is alive, do not
overwrite its DLL/video/APKs, reinstall the app, start another instrumented test,
rebuild its running Windows/proxy executables or disconnect its adb reverse.
Do not kill arbitrary apps/processes. The runner cleans up only its own host,
proxy, debug pairing file, reverse mapping and temporary power request.

The long fixture is synthetic: 7,500 seconds, 640x360 at 10 fps, low bitrate with
audio. It is ignored and never included in source exports. The emulator long run
is useful native evidence, but does not satisfy the physical-phone/TorBox/4K gate.
`Test-SessionProof.ps1 -SteadySeconds 7200 -ControlRttMs 800 -JitterMs 100` requests
a two-hour uninterrupted segment plus initial lifecycle/fault checks, EOF/replay,
missed-start recovery and decoder-error recovery. A short 60-second segment is
the default. Current runs require9matched raw start/success test packets and reject
ignored/assumption-skipped tests; a terse OK(count) alone is insufficient. Clock/drift comparison
is sampled-position evidence, not presentation timing. The analyzer uses indexed
binary search so a two-hour log does not monopolize the PC with quadratic work.

Use independent source work/unit checks during an active soak; defer installs and
native rebuilds until it exits. Results are in work/native-proof-result.txt and
work/native-proof-rtt-800-jitter-100.json; stdout/stderr for an unattended run may
be in work/overnight-soak.stdout.log and work/overnight-soak.stderr.log. Inspect
the current files, process and timestamps rather than assuming a previous result
belongs to the long run. Do not report a planned or running test as passed.

## Latest implemented work

Native execution observations with generation/time/position fences, a server
missing-execution watchdog, source-fenced terminal mpv failures, explicit reload
recovery preserving pause, premature-EOF classification, bounded control traffic,
constant-time token-hash checks and persistent initial pairing are implemented.
Windows DPAPI and Android AES-GCM/Keystore stores reject tampering without silently
replacing unreadable credentials. First-use code claims expire and cannot replay.
Android restores stored credentials on launch; debug proofs use isolated tokens.

The complete 800 ms RTT / ±100 ms jitter native suite passed 5 tests in 192.804 s
with the 180-second fixture. Its 714 matching position samples had median 55.5 ms,
p95 164.5 ms and maximum 240.5 ms. This exceeds the provisional p95 150 ms target
and must not be described as release-grade stability. A compact long fixture and
stronger uninterrupted-generation checks are being validated next.

## Unresolved work

TorBox safe direct URLs remain blocked: the tested requestdl URL embeds the account
key; the documented stream alternative returned 500. Never remove the guard or
move the key into Android, proxy video or silently transcode. Investigate valid
provider-supported capabilities while continuing independent development.

Next work includes production diagnostic journaling/export, native PC restart
testing, safe media delivery/renewal, control-only Cloudflare process lifecycle,
HTTPS pairing and credential revocation, real buffering and physical-device
acceptance. Chat/discovery/Pocket migration remain deferred. Current source
credentials remain in ignored DPAPI storage; never export or print them.

The compact 7,500-second fixture also passed the five-test suite with a 60-second
uninterrupted segment: 200.58 s runtime, 723 matched position samples, median
63 ms, p95 127.5 ms and maximum 203 ms. Median measured clock RTT was 915 ms,
maximum 1301 ms. This is still a short segment, not a passed two-hour run.
The newest same-error retry guard is separately reducer-tested; the next soak
build must include it. Source guards also reject encoded/case-changed account
keys and excessive nested URL encoding. Current Rust coverage totals 35 tests.

## 2026-10-08 unattended recovery fix

The first unattended 7,200-second request failed during initial recovery checks
after 104.975 seconds, before the uninterrupted segment started. Both native
players were paused at 26,600 ms for target 26,513 ms. Their 87 ms frame-rounded
seek result exceeded the old 80 ms READY bound, so generation 24 timed out.
This is a failed run, not two-hour evidence. Earlier summary JSON is historical.

The reducer, Windows follower and Android follower now agree on a 100 ms READY
position bound, allowing one 10 fps frame without changing drift thresholds.
A reducer regression checks 87/100 ms acceptance, rejection beyond 100 ms in
both directions and preservation of the authoritative target. Native proofs now
seek to 20,013 ms, deliberately between frames. Android diagnostics report zero
ready participants on a terminal session fault instead of inferring two from
media presence. Recovery checks also require the fault to clear. The 21 core
tests passed; a fresh full native short suite is running before another long run.

The proof runner now writes `work/native-proof-last-run.json` for the native
phase with a unique run ID, running/passed/failed status, requested duration and
completed duration only after a full pass. Successful metrics include the same
ID and start/end times. A failed native run has no metrics link or completed
duration, so old success metrics cannot be mistaken for the latest result.
Build failures still appear in the launch stdout/stderr before the native receipt
is created. The script syntax check passed; runtime receipt validation is pending
the next invocation. No provider error strings or credentials enter the receipt.

The corrected short suite passed all five tests in 197.068 seconds at configured
800 ms control RTT / ±100 ms jitter, including non-frame-aligned native seek,
native health recovery, reconnect, replay, missed execution and decoder recovery.
719 matched samples: median 75.5 ms, p95 183.5 ms, maximum 241.5 ms; measured RTT
median 888 ms, maximum 1308 ms. The p95 remains above the provisional 150 ms goal.
This is a functional pass with a 60-second segment, not release stability.
All 36 Rust tests and six Android JVM tests passed; frontend and native builds
passed. Next: launch the 7,200-second native segment using these exact binaries.

The next native run launched at 2026-10-07 22:58:10 UTC with source commit
468ad6d, runner PID 16976 (verify its start time, never PID alone). Its receipt
ID is ba474c4b-5376-4662-9c1b-503ed196ce47, status running, requested segment
7,200 seconds; completed duration and metrics are null. Both APK installations
succeeded and runner identity was verified after launch. This validates the
receipt's running state only; success/failure finalization awaits completion.
Do not rebuild/install/regenerate fixtures while this owned run is alive.

## 2026-10-08 production diagnostics during active soak

Added a typed, bounded production journal in `crates/session-server/src/diagnostics.rs`.
The PC host projects authority snapshots into fixed fields; the native follower
enqueues position/state samples at 100 ms intervals. Serialization, disk I/O,
64 MiB rolling retention and local ordered export use a separate worker. Queue
overload drops diagnostics and reports a counter instead of waiting in playback.
Unknown fault text becomes a fixed code; raw snapshots, media filenames/URLs,
keys/tokens, native error strings and chat text have no record field. Windows
has local diagnostic status and Export session log controls. Production journals
use app-local storage; debug proof hosts use ignored work/native-journal-proof.
See `docs/DIAGNOSTICS.md` for schema, retention and explicit missing event coverage.

Validation: 14 server tests passed (five new diagnostics tests), covering queue
overflow, disk failure, privacy projection, ordering/rotation/restart retention,
real Host publication and existing transport/auth/reconnect behavior. Windows
Rust metadata checks and frontend checks passed. The hash of the active proof's
Windows executable was checked before/after metadata checking and is unchanged.
No APK, native executable, media fixture, proxy or adb reverse was replaced.
Native runtime/performance and the Tauri export UI remain untested until the
owned soak exits. This source change is not running in that already-started soak.

At the latest ownership check the runner was alive with matching start time,
27.9 minutes after launch; latest PC sample was playing in generation 28 with
-24 ms sampled drift, no seeking/buffering. Receipt still says running, with
no completed duration or metrics. This is a progress observation, not a test pass.
Next continuation: protect the owned test, inspect its actual eventual result,
archive its numerical proof artifacts, then rebuild and run a short native proof
with journal/export wiring before claiming runtime coverage of diagnostics.

## 2026-10-08 long-run drift failure and observation fixes

Run ba474c4b-5376-4662-9c1b-503ed196ce47 failed after 5,161.857 seconds of
instrumentation (about 86 minutes), before the requested 7,200-second segment
completed. The assertion was `Excessive measured drift` (>349 ms). Its receipt
correctly finalized failed with null completed duration/metrics. All original
numeric logs and result/receipt are preserved in ignored work/failed-soak-ba474c4b.
The final recorded phone drift was -309 ms; the old test asserted before writing
the failing row, so its exact observation timestamp/age was not captured. The PC
continued normally in that interval. Do not infer a completed two-hour pass.

Code inspection found two observation defects independently of the exact failure
cause: Android's asynchronous playback flow could overwrite a probed position
while preserving a newer native sample timestamp; and the follower compared that
position against the later control-loop time. Removed the competing playback-flow
state writer. Native position and timestamp now share one probe; atomic updates
preserve concurrent source/failure metadata. Timestamp is captured immediately
after the position property read. Drift compares the timeline at that observation
time, without extrapolating native position, and smoothing consumes each native
sample once. A pure timing helper rejects stale/future/nonfinite observations.
Two new JVM tests cover delayed control ticks and invalid observation fences.

The native test now writes diagnostics before asserting and reports the exact
failing drift; logs include native sample time/age. The analyzer uses native
sample time for new logs, retaining legacy fallback for historical logs. The
350 ms assertion and uninterrupted-generation checks are unchanged. Eight Android
JVM tests and 14 Rust server tests pass; frontend/Windows/Android builds pass.
A fresh short native suite with 800 ms RTT / ±100 ms jitter is running. This is
also the first native runtime check of production journaling. The runner now
requires fresh journal authority/native records with no storage errors/drops;
no old journal can satisfy it. Local UI export remains untested.

The first observation-fix short run failed in its stale-probe injection because
the old test expected a fresh native Paused state while native observations were
explicitly disabled. The removed competing flow had previously supplied that
state. The test now checks authoritative playing=false and desiredPlaying=false
during the outage, then still requires fresh native Paused and a cleared fault
after restoring the probe. New diagnostic fields keep authority/intent separate
from player observation. This is an observation-test correction, not relaxation
of native readiness, drift or generation checks. A second full short suite is
running with this change; do not describe the failed first short run as passed.

The corrected native suite then passed all five tests in 202.602 seconds, but
overall runner status remained failed: the added live-journal reader used Windows
ReadAllText sharing, which denied the active writer. It now explicitly opens
with ReadWrite/Delete sharing and only parses complete JSONL lines. A concurrent
open/write sharing check passed. Its native-pass/reader-failure artifacts are
preserved under work/timing-native-pass-reader-fail, including a separately
analyzed partial proof: 738 matches, median 48 ms, p95 174.5 ms, max 253 ms.
That is diagnostic analysis of a failed overall run, not a runner pass. The
next native short invocation validates the corrected concurrent reader. Receipt
phases now distinguish startup/setup, native tests, journal validation and
analysis. The HTTPS proof also checks live native ownership before any builds
or emulator installs, preventing accidental interference with an active soak.

Short run 956cac57-ec9a-4f17-9e07-20cc4a975626 fully passed in 200.916 seconds,
including five native tests, fresh live-journal validation, zero journal drops/
storage errors, analysis and receipt phase=complete/status=passed. 730 matched
samples: median 57 ms, p95 158.5 ms, maximum 206 ms; RTT median 882/max 1370 ms.
p95 is still above the provisional 150 ms goal. Artifacts are preserved in
work/timing-journal-passed-956cac57. Its 610 recorded steady-generation-26 samples
had maximum native sample age 107 ms. This is a short native/journal runtime pass,
not two-hour or physical-phone acceptance; local export UI remains untested.

The next long test also asserts native sample age <=350 ms during steady playback
and records setup/steady/recovery phases. This prevents old observations from
passing a drift-only check. The new assertion is compiled but has not yet run;
the previous short pass predates it (its recorded ages satisfy that bound).
Direct HTTPS regression passed on Windows (TLS load/play/pause/exact seek and
source replacement) and Android (two tests in 9.589 seconds) with the observation
fix. Android test APK was rebuilt with the stricter freshness assertion during
that HTTPS proof. Next: use these binaries/APKs for another two-hour run.

New 7,200-second native run launched 2026-10-08 01:05:29 UTC from source commit
62b8130. Runner PID 5480/start 01:05:29.5053360 UTC were verified together after
launch. Receipt d8d2bfd2-cf7d-4555-81b6-3dd1a66bd3c1 is running/native_tests with
null completed duration and metrics. Both APK installs succeeded. Protect this
run's binary/media/APKs/proxy/reverse mapping; independent source work only until
it exits. It includes production journal recording and the stricter native age
assertion. Do not report a planned or active two-hour run as passed.

## 2026-10-08 command diagnostics and cancellation ownership

Added typed `command` journal events at both the local PC boundary and the
authenticated WebSocket boundary. Events retain actor/action, sequence, submitted
and resulting generation, revision, numeric position and a fixed outcome code.
Accepted events precede their state snapshot; rejected events leave authority
unchanged. Unknown error strings map to other_rejected, so command metadata,
submitted epoch text, media names, raw requests and credentials cannot enter
these records. No protocol messages or public diagnostics routes were added.

While testing a live journal-enabled WebSocket server, found that aborting the
serve future could bypass ticker.abort() and leave the timer holding Host and
its journal worker. A drop guard now aborts that owned timer on both return and
cancellation. The regression closes its client, cancels the server, verifies the
last Host weak reference expires, and verifies the journal directory can be
removed. This checks ticker ownership, not active-client draining or an actual
native PC restart; those broader lifecycle proofs remain required.

All 16 server tests pass (two additional tests, cumulative Rust coverage 43).
Coverage includes accepted local load/remote pause, local stale generation,
remote host-only/duplicate sequence, no rejected-state mutation, privacy and
unknown-error sanitization. Windows metadata checking passed; the native proof
executable hash is unchanged. These new records/cancellation changes are not
running in the already-started soak; defer native rebuild/validation until it
exits. At the latest ownership check, PID 5480/start time matched and the run
was alive 18.2 minutes after launch, receipt still running with no final metrics.
Latest PC sample remained playing in generation 26, no seeking/buffering. This
is ongoing evidence only, not a passed two-hour or physical-phone gate.

## 2026-10-08 active socket shutdown and restart fencing

Extended server lifetime ownership beyond its ticker: dropping/cancelling serve
now marks the Host stopped and signals upgraded sockets. Writes (including clock,
ack, snapshot, ping/pong and rate-limit close) can be interrupted by shutdown.
The participant is disconnected/authority frozen before a best-effort 1012
server_restart close (250 ms timeout). Stopped Hosts reject local commands,
pairing and upgrades, and report unhealthy; each Host listener is single-use.
Fresh Host construction preserves supplied pairing credentials but creates a new
epoch; in-memory media selection is not persisted by this change.

Android now handles onClosing through its existing fenced pause/reconnect path
rather than waiting for final teardown/control silence. It ignores raw peer
reason text. This callback is compiled and unit checks pass, but is not installed
in the already-running native soak and needs a native restart proof afterward.

All 19 server tests pass (three new tests, cumulative Rust coverage 46), including
actual WebSocket close/freeze, release of the old Host, rebinding the same local
address with the same synthetic credentials/new epoch, old-command rejection,
pending-write cancellation, and stopped health/pairing rejection. Eight Android
JVM tests and Kotlin compilation passed; Windows metadata checking passed and
the active Windows executable hash is unchanged. No APK was assembled/reinstalled,
no proxy/player binary rebuilt and no reverse mapping/fixture changed.

Latest runner PID/start-time verification matched PID 5480 at 52.9 minutes after
launch. d8d2bfd2 receipt still running/native_tests, completed duration/metrics
null; latest PC log remained playing in generation 26 without seeking/buffering.
Do not claim the active soak tests these later source changes, native restart,
Cloudflare interruption, physical-phone stability or a passed two-hour segment.

## 2026-10-08 control connection diagnostics

Added bounded typed control journal events for authenticated WebSocket connection
and closure, server shutdown, malformed requests, unsupported clock protocol,
frame rate limits and heartbeat silence. Events carry only fixed PC/phone actor,
fixed kind and monotonic server time. Peer text, close reasons, headers and
credentials cannot enter the event type. Generic socket disconnection remains
unspecified between close/read/write failures; pre-upgrade authentication and
tunnel lifecycle events are still pending. No public diagnostics route added.

A new real WebSocket regression covers malformed input without authority
mutation, protocol mismatch, six-second silence, a frame flood and active server
shutdown. Export confirms all expected events, four socket connections, three
generic closures and one shutdown closure, no private input/credential markers,
zero drops and zero storage errors. All 20 server tests pass (cumulative Rust
coverage 47). Windows metadata check passes; active proof executable SHA-256
unchanged. No Android changes, APK installs, media/proxy rebuilds or adb changes.

Runner PID 5480 and recorded start instant still match at 76.4 minutes after
launch. Receipt d8d2bfd2 remains running/native_tests, duration and metrics null;
latest PC sample is playing generation 26 without seeking/buffering. These
diagnostic changes are not running in that earlier executable. Await completion
before rebuilding; native/emulator evidence does not satisfy the physical gate.

## 2026-10-08 Android clock reply correlation

Clock replies previously checked socket identity, epoch and estimator timing but
did not require an outstanding request. Added ClockRequests: at most 16 pending
integer monotonic t0 echoes, five-second expiry, consume once, bounded reordering
allowed, reset together with calibration on open/loss/disconnect/epoch change.
Unsolicited, duplicate, fractional, nonfinite, expired and pre-request arrivals
cannot calibrate. Existing 1,500 ms RTT and 30-second calibration freshness
limits are unchanged. Clock sends now require an opened connection; enqueue
failure takes the normal pause/reconnect path rather than silently continuing.

Android compilation and all 12 JVM tests pass (four new tests cover matching,
reordering, reset, capacity/expiry, invalid timing and protection of a good
calibration from unmatched replies that otherwise appear to have lower RTT).
Protocol documentation now records the actual 500 ms cadence and elapsedRealtime
clock. No APK assembly/install, native binary/media/proxy changes or adb commands
were performed. Clock integration still needs a fresh native short run after
the active soak exits; pure JVM tests are not a native/WAN/reconnect proof.

PID 5480/start instant matches at 106.4 minutes after launch. d8d2bfd2 remains
running/native_tests with completed duration and metrics null; PC continues in
generation 26 without seeking/buffering. This earlier run does not include the
new clock request matching. The physical-phone/TorBox release gate remains open.

## 2026-10-08 completed native two-hour emulator run

Run d8d2bfd2 passed/complete at 03:08:32 UTC, with 7,200 requested/completed
uninterrupted seconds and five instrumentation tests passed in 7,340.146 seconds.
Artifacts were copied to ignored work/passed-soak-d8d2bfd2 before any rebuilding.
Detailed results/boundaries: NATIVE-STABILITY-2026-10-08.md. One steady generation,
71,161 steady samples, maximum authority-relative Android drift 333 ms and
recorded sample age 128 ms; no steady disconnection/buffering/native failure.
Cross-device median/p95/max 87/224.5/416.5 ms. The provisional p95 goal 150 ms is
NOT met, including steady-only comparison. At the worst matched point Android
was +272 ms relative to authority and PC -145 ms; independent authority bounds
do not guarantee the same cross-device bound. No physical/TorBox gate passed.

The analyzer now emits explicit provisional p95 goal/result fields and accepts
an independent output path for archived analysis without overwriting an active
runner's summary. A standalone synthetic analysis regression checks observation
timestamps rather than receipt time, exact median/p95/max, both goal outcomes
and historical timestamp fallback. It passed; reanalysis of archived long-run
logs reproduces all 71,315 matches and original statistics, goal_met=false.

Fresh short native runner launched 03:20:51 UTC, PID 18216/source e9d73b9, with
800 ms RTT/+/-100 ms jitter and a 60-second uninterrupted segment. Windows and
Android builds/install succeeded. Receipt 807f66ff passed/complete at 03:24:43,
five native tests in 205.751 seconds, 60 requested/completed steady seconds.
737 matched samples: median 82.5/p95 190/max 372.5 ms; measured RTT median
880/max 1,194 ms. Explicit provisional p95 goal=false. The final journal has
127 command, 229 session, 2,078 native_pc and eight control records (four socket
connections/disconnections), zero drops/storage errors. Artifacts archived in
work/passed-short-807f66ff. Latest clock and typed diagnostics now have short
native runtime evidence; this does not specifically exercise server-initiated
onClosing, actual PC restart or WAN recovery, and is not another two-hour run.
The runner has exited/removed its ownership marker; future native work may
proceed after checking current state. Physical gate and p95 goal remain open.

## 2026-10-08 monotonic-time drift smoothing

Investigation found a cadence-dependent correction filter: both implementations
used a fixed 0.3 smoothing weight for each update, while the Windows worker can
update every ~10 ms and Android feeds native probe observations ~100 ms apart.
The same drift waveform therefore had different effective smoothing on each
platform. This is a code-level inconsistency, not proof it caused the observed
long-run quality miss. Both derivative controllers now use
alpha=1-(1-0.3)^(elapsed_ms/100), maintaining the original response at 100 ms.
Duplicate/reordered observations return Hold; observation timing resets with
existing generation/barrier resets and survives a seek request's filter reset
so the same observation cannot repeatedly request a barrier. Nudge thresholds,
rates, large-drift ladder and acceptance assertions remain unchanged.

23 sync-core tests and 14 Android JVM tests pass. New matching Rust/Kotlin tests
drive the same step/recovery waveform at 10 and 100 ms, compare effective rate
at common instants, require eventual nominal rate/no small-drift seek, and reject
duplicate/reordered updates including duplicate major seek requests. Existing
virtual two-hour and ladder tests still pass. License modification provenance
updated. Native timing/quality validation is pending a fresh build/run; none of
these unit/simulation results satisfy native or physical acceptance.

Fresh native short run 0233c128 (source 8b3c57e) passed/complete at 03:55:52 UTC:
five tests in 201.004 seconds, requested/completed uninterrupted segment 60 s,
fresh production journal with zero drops/storage errors. 729 matched samples:
median 53/p95 159.5/max 210.5 ms; RTT median 887/max 1,226 ms. This is lower than
the preceding short run's 82.5/190/372.5 ms, but a single run under random jitter
does not establish causality or long-run improvement. Provisional p95 goal=false.
Artifacts preserved under work/passed-drift-time-0233c128 before the next run.

A 600-second uninterrupted follow-up launched 03:56:17 UTC using the same built
source 8b3c57e, SkipBuild, 800 ms RTT/+/-100 ms jitter. Runner PID 18128/start
03:56:17.7281263 UTC; launch record work/drift-time-medium-launch.json, stdout/
stderr work/drift-time-medium.*.log. Protect binary/APKs/fixture/proxy/reverse
mapping while alive; verify PID and start instant together. The final receipt
and metrics must be inspected after it exits. This is a ten-minute emulator
test, not two-hour revalidation of the modified controller or physical acceptance.

## 2026-10-08 ten-minute result and synchronous native-command faults

Ten-minute run adcdb076 passed/complete at 04:08:48 UTC: five native tests in
740.856 seconds, 600 requested/completed uninterrupted seconds, 6,054 matched
samples. Median/p95/max drift 47.5/154/297 ms; RTT median 852/max 1,203 ms;
production journal zero drops/storage errors. Provisional p95 goal=false.
Archived to work/passed-medium-adcdb076 before changes/rebuilds. It is evidence
for the time-normalized controller at ten minutes, not a new two-hour pass or
physical acceptance; random jitter still prevents attributing improvement from
these few runs alone.

Found ignored synchronous command results in the Windows worker (generation
pause/reset, preparation pause, rate changes). Async mpv event failures already
entered typed faults, but immediate property/enqueue rejection could disappear.
The native adapter now latches native set/enqueue/load command rejection and
invalid correction rate until consumed. Later successful commands cannot clear
it. The worker consumes this latch before native readiness checks and again
before execution acknowledgement, maps it to CommandFailed without overwriting
an existing more specific load/decoder fault, and retains the established retry
barrier. Diagnostic property reads while idle do not trip the latch.

Three actual-libmpv Windows adapter tests pass: invalid property remains latched
across a successful pause, invalid rates are observable, and async enqueue
rejection is visible before callbacks. One Windows controller/Host integration
test discards an actual invalid native command result, captures the fixed fault,
reports it into authority, checks timeline freeze/generation advance, and proves
a later success does not recover the terminal fault. Windows metadata checking
passes. This covers the fault boundary with actual libmpv, not a full UI-driven
fault/retry movie session; normal native regression against this change follows.

Normal native regression 3eacaf06 passed/complete at 04:26:32 UTC on source
6836dcf: five tests in 204.326 seconds, 60 uninterrupted seconds, fresh journal
zero drops/storage errors, 729 matched samples. Median/p95/max 56/189/290 ms;
RTT median 886/max 1,193 ms, provisional p95 goal=false. No unexpected command
fault appeared in normal operations. This is normal native runtime evidence;
deliberate command rejection/authority propagation is covered by the separate
actual-libmpv/Windows controller tests above. Artifacts archived under
work/passed-command-fault-3eacaf06. Variation across short runs reinforces that
the earlier lower p95 is not proof of a sustained quality improvement.

A fresh 7,200-second uninterrupted native request launched at 04:26:55 UTC on
built source 6836dcf, using SkipBuild and 800 ms RTT/+/-100 ms jitter. Runner
PID 17748/start 04:26:55.0330269 UTC, launch record work/current-core-soak-launch.json,
stdout/stderr work/current-core-soak.*.log. Protect its native binary, DLL,
APKs, fixture, proxy and adb reverse mapping while alive. Verify both PID and
start instant; inspect receipt after completion. Earlier d8d2bfd2 two-hour pass
predates elapsed-time smoothing and synchronous fault handling. This new run
has no completed result yet and remains emulator/synthetic scope only.

## 2026-10-08 failed clock-jump run and calibration retention

81aa232a failed/native_tests at 04:28:36 UTC after 96.6 seconds of native
instrumentation; no two-hour segment completed (receipt duration/metrics null).
Exact failed steady row: generation 30, native age 69 ms, no buffering/seeking,
connected/playing/source ready; measured authority drift jumped 99 -> 361 ms as
selected offset changed -29037661 -> -29037923.5 ms (262.5 ms shift), selected
RTT 838 -> 1087 ms, while native position advanced normally 100 ms. This is
direct evidence of a calibration discontinuity, not a native sample-age failure.
Archived receipt, native/phone logs, result, launch logs and journal in
work/failed-clock-jump-81aa232a. Historical summary files are not its metrics.

Found a retention mismatch: both estimators allowed calibration to stay fresh
30 seconds but retained only eight replies, ~4 seconds at the actual 500 ms
cadence. New replies could evict a still-fresh, lower-RTT calibration during a
burst of asymmetrically delayed traffic. Expanded the bounded history to 64
samples (~32 seconds at that cadence), retaining the existing 30-second age
filter, lowest-RTT selection and 1500 ms RTT validity bound. No drift/age test
threshold changed and no failed sample was removed. Raw exchanges were not
logged in the failed run, so premature eviction is a reproduced plausible
mechanism, not proof of the exact internal selection history in that run.

25 Rust sync-core and 16 Android JVM tests pass; app and instrumentation Kotlin
compile. Matching new tests retain an 838 ms RTT sample through 20 later
1087 ms replies that disagree by 262.5 ms, still expire it at 30 seconds,
allow newer lower-RTT calibration and clear on reset. Added proof diagnostics:
selected clock sample age, bounded history size, last accepted candidate RTT
and offset. These numeric fields reset with calibration and carry no credentials
or URLs. New native validation is required before claiming the runtime fix.

Fresh ten-minute native retry launched 04:52:33 UTC from source 6b2d893.
Windows/frontend/Android builds and both APK installs succeeded. Receipt
370285e3-fa2a-4152-86ec-703c6742fb73 is running/native_tests, requested 600 s,
completed duration/metrics null. Runner PID 16768/start 04:52:33.8384305 UTC
verified together after install. Launch record work/clock-history-medium-launch.json;
logs work/clock-history-medium.*.log. Protect this run's binaries, DLL, APKs,
fixture, proxy/reverse mapping until it exits. Inspect raw clock diagnostics and
actual receipt afterward; no native, two-hour or physical success is claimed yet.

## 2026-10-08 clock-history runtime pass and control-only tunnel builder

370285e3 passed/complete at 05:05:23 UTC: five native tests in 745.042 seconds,
600 uninterrupted seconds, fresh journal zero drops/errors. 6,058 matched
samples, median/p95/max 51/143/200.5 ms; measured RTT median 751/max 1,121 ms.
Provisional 150 ms p95 goal=true for this run only. Archived to
work/passed-clock-history-370285e3. 5,926 steady rows/one generation, selected
sample age max 30,000 ms, history max 64, max selected offset change 156.5 ms,
authority drift max 163 ms and native age max 107 ms. Retention does not eliminate
all clock jumps, and this ten-minute pass is not two-hour/physical acceptance.

New 7,200-second request launched 05:21:47 UTC on built source 6b2d893. Runner
PID 16588/start 05:21:47.4059197 UTC verified together. Receipt 4d821313 is
running/native_tests, no completed duration/metrics. Launch work/clock-history-soak-launch.json;
stdout/stderr work/clock-history-soak.*.log. At latest check alive 2.5 minutes,
PC playing generation 30 with no seeking/buffering. Protect its binaries, DLL,
APK installs, fixture, proxy and reverse mapping until it exits.

Independent source work adds ControlConfig and non-deploying tunnel-config
example: exact hostname and only /session,/pair,/health to fixed loopback control
origin, default 404. No arbitrary origin/path/wildcard/TCP/media configuration.
Four new builder tests and actual copied-cloudflared offline parser/rule checks
pass. Old reference example now includes /pair. All 24 server tests pass after
fixing a pre-existing test race where legitimate heartbeat RTT advanced revision
during malformed-input rejection; playback state invariants remain checked.
Windows metadata check passes and active native executable hash is unchanged.
See TUNNEL.md for source, offline evidence and pending lifecycle/deployment scope.
No tunnel was deployed, started or connected; legacy files remain untouched.

## 2026-10-08 owned tunnel child lifecycle

Added OwnedTunnel using generated control-only config, explicit absolute binary,
random create-new owned config and direct hidden cloudflared command with
--no-autoupdate. Sanitized environment excludes inherited TUNNEL_*/proxy overrides;
stdin/out/err disconnected, no CLI token. Child handle retained; explicit stop
and Drop kill/reap only that child and remove only owned config. Process state
is not tunnel connection readiness. Fixed error codes; no credential reads by
Rust manager or raw child output. Caller supplies private runtime directory and
must keep blocking operations off UI/async executor.

Five new lifecycle tests use isolated local helper children, not cloudflared:
stop isolation/idempotence, natural exit, Drop termination checked with original
Windows process handle, failed-launch cleanup preserving unrelated files, exact
command/environment construction. 19 unit + 10 transport = 29 passed; helper
fixture intentionally ignored except explicit invocation in lifecycle tests.
Windows metadata check passes; active player executable SHA-256 unchanged.
Copied cloudflared local tunnel help confirms argument placement/options; no
real tunnel/production credential/config/account was used. See TUNNEL.md for
remaining supervisor, startup, ACL, verification, crash containment and WAN scope.

4d821313 remains running/native_tests at 32.2 minutes after launch; PID16588 and
05:21:47.4059197 UTC start-time ticks match. Requested7200s, completedduration and
metrics null. Its binary/APKs/fixture/proxy/reverse unchanged. This is still an
active emulator/synthetic run, not a completed two-hour or physical-phone gate.
Legacy Together/Pocket remain read-only; no extra features migrated.

## 2026-10-08 background tunnel supervision

Added dedicated-thread TunnelSupervisor with watch statuses, cancellable
poll/backoff, increasing retry delays (1/2/4/8/16/cap30s), uptime-based streak
reset after60s, and cleanup of previous child before retry. Stop failure disables
restart. Async shutdown joins through blocking pool with five-second caller
deadline; timeout retains cleanup ownership in worker. Drop sends cancellation
without blocking UI. Process state remains distinct from WAN connection health.
Raw errors/output, credentials, paths and hostname never enter supervisor state.
No app startup, production configuration, account/DNS or real tunnel launch.

Six added tests cover cap/overflow, backoff cancellation, retry ownership,
Drop cleanup, stop failure and actual child shutdown through supervisor. Final
25 unit + 10 transport tests pass; helper intentionally ignored except explicit
parent invocation. Windows metadata passes with playback exe SHA256 unchanged.
See TUNNEL.md for lifecycle limitations: Drop is not crash containment, async
deadline is not a guarantee of completed OS operations, awaited shutdown still
needs integration with app exit, private directory/verified executable pending.

4d821313 remains running/native_tests; PID/start ticks verified at60.7min after
launch. Direct share-safe journal read found6,297,004 bytes in session-0 and
fresh sequence38135 at0.1sec age (directory metadata had stale length/mtime
while writer open). No failure/completed-duration/metrics in current receipt.
Protect binary, DLL, APKs, fixture, proxy and adb reverse until actual completion.
No two-hour/physical pass claimed, legacy Together/Pocket unchanged.

## 2026-10-08 typed tunnel lifecycle diagnostics

Added optional start_observed API recording supervisor transitions at origin
through Host bounded journal before watch updates (watch may coalesce). Typed
tunnel event contains monotonic server_ms and status/retry fields only. No raw
error, output, hostname, UUID, paths or credentials. Weak Host reference cannot
retain server. Journal must be enabled before supervisor; queue drop/storage
counters apply, no guarantee of lossless persistence. No automatic startup or
public tunnel configuration/launch; existing start remains unobserved.

Two meaningful tests: public observed start with nonexistent synthetic exe and
credentials produces ordered starting/backoff/stopping/stopped in actual Host
export, numeric retry fields, zero drops/errors and excludes synthetic private
markers/hostname/UUID/rawerror/paths; dropping Host during observed worker run
proves weak lifetime and shutdown. All37server tests (27unit/10transport) pass;
helper ignored except explicit child invocation. Windows metadata passes and
soak exe SHA256 unchanged. Actual tunnel-success/exit and UI/WAN runtime remain
pending. See DIAGNOSTICS.md and TUNNEL.md.

4d821313 still running/native_tests at90.3min afterlaunch, PID16588/startticks
matched. Shared journal read:0.3sec old native_pc, generation30, playing,
no buffering/seeking, drift-59ms, zero drops/io errors. This single observation
is not completed stability metrics. Receipt completed duration/metrics null.
Do not disturb binaries/APKs/DLL/fixture/proxy/reverse until run completes.
Physicalphone/TorBox/WAN release gate remains unmet; legacy unchanged.

## 2026-10-08 second complete two-hour playback-core pass

4d821313 finalized passed/complete at07:24:57.2391852UTC. Five native tests
7346.671sec; requested/completed steady7200sec. MetricsID matchesreceipt;
runner exited and ownershipmarker removed. Source6b2d893 includes correlated
clock replies/time-based filter/commandfault/clock64history; later tunnel source
not in this binary. Archive work/passed-clock-history-soak-4d821313 includes
receipt, matching metrics, launch, result, phone/PC logs, stdout/err, journal.

71220crossmatches median/p95/max72.5/178.5/280.5ms; measuredRTT744/1125ms.
Provisional150ms p95goal remains false, thresholds unchanged. 71077steadyrows,
onegeneration, historymax64/selectedagemax30000ms/offsetjumpmax173ms,
authoritydriftmax181ms/nativeagemax127ms. All counted steady clockage/capacity/
disconnect/buffer/seek/prepare/notplaying violations zero; journal zero drops/io.
Steady crossdistribution identicalallphase; phone signeddriftmedian10ms,
PCmedian-59ms; worstmatch phone+132 PC-149 bothrate1.0, nativeage17ms,
matchingdistance19.5ms. +/-150ms nudge-entry band permits opposing errors;
consider a controlled tighter-band experiment next, not a weaker acceptance
threshold. No causal attribution across runs or screenalignment claim.

Added independent streaming analyze_clock_proof.py (not hooked into active
runner) with completedpass/emulatorscope/duration checks, fixed errors, numeric
aggregateonly, newoutputonly, generation-aware jumps and violationcounts.
Fourtests pass including CLI refusal/overwrite/privacy, malformednumericdata,
reorderedtime and incomplete/runreceipt. Historical370285e3 analysis reproduced
exact archivednumbers. Newclockanalysis and steadydriftanalysis saved inarchive.
Phone rows lackrunID; caller retainsreceipt/logpair, toolcannotverifyassociation.
Python caches ignored. See NATIVE-STABILITY-2026-10-08.md and DIAGNOSTICS.md.
Physicaldevice twohourgate, live safeTorBoxdelivery and WAN remain open.
Legacy untouched. Next work can rebuild only after confirming no new runner.

## 2026-10-08 tighter nudge-band experiment

Baseline4d821313 twohourpass remains archived, p95cross178.5ms>150goal.
Opposing +132/-149ms errors at rate1.0 fit existing150ms nudge-entry band.
Experiment reduces BOTH Rust/Kotlin nudge entry150->75ms and exit30->20ms;
0.5%rate, time-normalizedfilter, large-driftladder, clockestimator, schedule/
READY/barriers and acceptance thresholds unchanged. Provenance clearly marks
Together modification, not upstreamdefaults. Requiresfresh nativevalidation;
baseline twohourpass does not validate these changedconstants.

Added equivalent Rust/Kotlin behavior tests: two players initial+/-130ms converge
to<60msdifference within30sec, return1.0, <=3ratecommands each, no seeks/rate
outside0.995..1.005. +/-20msprobe noise does notstartnudge. Existing duplicate/
reorder regression uses200ms inputstep instead400ms so firstfiltered60ms remains
belownewentry; duplicate/reorder/majorseekfencing intent retained. Cadence/filter
response invariance tests pass.27sync-coretests and37servertests pass; Android
unitand instrumentation compilation pass. This is simulation/unit evidence,
not physical/stability acceptance. Launch fresh600second native proof next at
800msRTT/100msjitter for comparable distribution; operatingconditions stillvary.

Tighter-band launch finalized: commit aaf4a0c built successfully (Windows native,
frontend check/build, Android unit/build and both APKs installed). Android unit
count18/failures0. Receipt732bae22-e746-4e2a-9e77-8ec4fc30a9f9 is running/
native_tests, requested600sec, completed duration/metrics null. PID1852 and
07:52:00.3059907UTC startticks verified; launchrecordwork/nudge-band-medium-launch.json,
stdout/err work/nudge-band-medium.*.log. Existing oldsummary is historical until
this receipt produces matchingmetrics. Protect binary, DLL, APKs, media, proxy
and adb reverse while ownedrunner isalive. This experiment has no native pass
result yet, and doesnot inherit baseline twohouracceptance. Review actual
results next, including rate-command chatter, p95 and clock selectedoffsetjumps;
physicalphone gate remains open. Legacy Together/Pocket untouched.

## 2026-10-08 access rejection observability during native experiment

Continued immediately at userrequest, preserving active732bae22.
Added typed access events for missing/wrong/oversized bearer rejection, pairing
claim denial and duplicate authenticated role. No role attribution beforeauth;
only monotonic time/fixed endpoint/kind, no headers/codes/tokens/IP/rawerrors.
Events occurafterexisting ratebucket toavoidlogging everythrottledrequest.
No publicroute/authpolicychange. Extractor/rate/serverstop/upgradecallback
failures remain unrecorded bythisfamily (see DIAGNOSTICS.md).

New realtransport test exercises401cases, HTTP403pair,409duplicate, unchanged
authority afterdenials, survivingauthenticatedphone clockexchange, privatefield
exclusion andzeroqueue/storageerrors. All38servertests pass; Windowsmetadata
passes; activeplayerbinarySHA256unchanged. Thisnewsource is not in732bae22
currentrun; separatelyvalidatedtransport only. LegacyTogether/Pocket untouched.
732bae22 remainsrunning/native_tests at07:58:30UTC, requested600sec, completed
metricsnull; ownedrunnerPID1852/start07:52:00.3059907UTC previouslyverified.
Protect activeassets/binaries/APKs/fixture/proxy/reverse untilactualcompletion.

## 2026-10-08 tighter-band medium native pass and extended validation

732bae22 passed/complete at08:04:55.182205UTC: five native tests745.241sec,
requested/completed steady600sec, journal zero drops/storageerrors.6050matched
crossmedian/p95/max36.5/117/190ms, RTTmedian/max727/1243ms. Provisional150ms
p95goal true for thisrun. Archivework/passed-nudge-band-medium-732bae22 holds
matchingreceipt/metrics/logs/journal/launch.5911steadyrows/onegeneration,
clockselectedagemax29995ms/history64/jump172.5ms/authoritydrift157ms/nativeage125ms;
all countedsteadystate/clockviolationszero. Not screenalignment/physicalacceptance.

Observedphone rate transitions70/600sec, maximum24inrolling60sec,851non-nominal
samples, rates0.995..1.005. This is sampledpropertychanges, not nativecommand
count; correctionfrequency increased versusbaseline, so do notclaimchatter
absent orimperceptible. Shortp95improvement is encouraging but operatingconditions
and durationdiffer; extend7200sec with same800RTT/100jitter beforestabilityclaim.
Latestsourceincludes e704277accessdiagnostics separatelytransporttested; this
mediumbinary was aaf4a0c. Rebuildnextlongrunincludes latestsource; noothercoretweak.
Physicalphone/TorBox/WANgate remains open. Legacyunchanged.38servertestspass.

Extendedrun launched08:06:02UTC on19b7ff3 (sameplaybacktighterband plusaccess
journalextension). Builds/frontendchecks/AndroidunitandbothAPKinstalls successful.
Receiptf931b66c-0cf8-4ddf-91c2-ad587b7c29b7 running/native_tests, requested7200,
completedduration/metricsnull. RunnerPID5152/start08:06:02.7851635UTC ticksverified.
Launchwork/nudge-band-soak-launch.json; logswork/nudge-band-soak.*.log.
Protectthisrun binaries/DLL/APKs/media/proxy/adb reverse. No twohourresult yet
forchangedband. Review actualoutcome andrateclusters; oldsummaryhistorical.

## 2026-10-08 Windows job cleanup foundation

Independent sourcework while f931b66csoak ownsplayback: WindowsJob creates
unnamed/non-inheritable job withkill-on-last-handleclose, OwnedHandle lifetime
andborrowedhandle forfutureatomicjob-listcreation. No productionprocessassignment
orlaunchwiring yet. Tested explicitjobclose isolatesunrelatedchild; forcedexit
ofisolatedowner killsassignedchild andinheritedgrandchild withoutRustDrop.
Retainedprocesshandles ensurePIDreuse-safechecks; jobinheritanceflagdisabled.
RAIIfixtureguardscleanup onassertfailure. After addingguards, onefixturetest
failedbecause grandchildguard was scopedinsideifblock andkilleditbeforecheck;
fixed byretainingguardthroughfixturelifetime. Finalfullsuite40passed
(29unit+11transport,3helperfixturesignored), warning-free Windowsmetadatapass,
activeplayerSHA256unchanged.

Importantboundary: currentOwnedTunnel stdCommandspawn remainsunchanged.
Postspawnassignmentor suspended-then-assign hasstartuporphanwindow; do notclaim
productioncrashsafety. Need stableWindowsnativeatomicJOB_LISTlauncher; Rust1.99
spawn_with_attributes isstillnightly so noupgrade/nightly/bootstrap used.
DocumentationandprimaryMicrosoft/Rustsources inTUNNEL.md. Realcloudflared,
credentials/config,domain andlegacyfiles untouched. Stalefilesaftercrashremain
separatecleanupconcern.

f931b66c stillrunning/native_tests at18.8min afterlaunch. PID5152 andstart
08:06:02.7851635UTCticks match. Fresh0.1sec PCnative samplegeneration28, playing
rate1.0 drift26ms,nobuffer/seek, zerojournal drops/io. Singleobservation only;
completed7200sec/metricsstillnull. Protectbinary/DLL/APKs/fixture/proxy/reverse.
Physical/TorBox/WANreleasegate remains unmet; nochanges toacceptancethresholds.

## 2026-10-08 stable native atomic Windows tunnel launcher

Replaced the Windows OwnedTunnel library child's standard spawn with a private
stable CreateProcessW launcher. STARTUPINFOEXW includes JOB_LIST at creation and
HANDLE_LIST restricted to NUL; explicit UTF-16 environment/arguments, no inherited
parent environment, separate executable path, no shell/output and no uncontained
fallback. Owned handles close the kill-on-close job on Drop, forced owner death
and observed natural root exit. Supervisor/lifecycle tests now exercise this
path. Normal Windows app startup remains unwired; no public tunnel was opened.
Non-Windows standard launcher does not gain crash containment.

Three actual-process regressions pass: difficult CRT argument/environment round
trips including Unicode and absent inherited PATH; exact job membership and Drop;
forced isolated-owner exit removes immediate child/grandchild with retained
process-handle checks. Full suite 43 passed (32 unit/11 transport), seven ignored
(six helper fixtures and opt-in offline validation). Copied hash-matched
cloudflared separately passed offline ingress validation through NativeChild with
synthetic config/nonexistent credentials and a Unicode/spaced path. No real
credentials/account/DNS/legacy files modified. A default Windows text-encoding
edit error briefly truncated the new source and caused one compile failure;
restored as UTF-8 and reran offline/full-suite checks successfully. Final Windows
metadata check passed; active playback executable hash unchanged.

Source-level review retains attribute payload lifetimes and aligned list storage,
closes native thread handle, checks process wait/status, and cleans descendants
on root exit. See TUNNEL.md for primary references and validation scope. Stale
owned configs after abrupt death, ACLs, binary installer verification, app startup/
awaited shutdown, dedicated provisioning and public WSS/WAN health remain open.

At 09:00:32UTC f931b66c remained running/native_tests, 54.5 minutes after launch;
PID5152 and start08:06:02.7851635UTC ticks matched. Follow-up shared-stream sample
was 0.2sec old, generation28 playing/rate1, drift-43ms, not buffering/seeking,
zero journal drops/io errors. This is a single live observation, not final sync
statistics. Requested7200sec; completed duration and metrics remain null. Protect
its executable/proxy/DLL/APKs/media/adb reverse. Its built source19b7ff3 excludes
this new launcher, which has separate process/offline evidence. Physical-phone,
real safe TorBox delivery and WAN acceptance remain unmet. Legacy/Pocket untouched.

## 2026-10-08 natural tunnel-root exit and handle isolation regressions

Added two targeted native-process tests without changing production playback or
launch behavior: root exits37 leaving an immediate descendant alive until
NativeChild observes exit, closes its job and reaps descendant; repeated status/
wait preserve cached exit37. Handle isolation uses an actual exclusive file with
an inheritable duplicate: after launch, closing parent copies permits reopening
while child remains alive, proving the child did not retain that file handle.

Initial sentinel test failed under parallel execution because other ordinary
std-spawn fixtures inherited the handle from the same test runner. Moved the
sentinel check to an isolated helper subprocess; final full suite45passed
(34unit/11transport), eight ignored (seven helper fixtures plus opt-in offline
cloudflared). Windowsmetadata passes, activeexecutable SHA256 unchanged. UTF-8
explicit file reads/writes used. No new public tunnel or credential access.

At09:22:09UTC f931b66c stillrunning/native_tests,76.1min afterlaunch. PID5152/start
08:06:02.7851635UTCticks match. Latest shared-stream PCnative sample0.1sec old,
generation28 playing/rate1, drift-23ms,nobuffer/seek,zerojournaldrops/ioerrors.
Completed7200sec/metricsstillnull; do not infer finaldistribution from singlesample.
Run owns binaries/proxy/DLL/APKs/fixture/reverse unchanged. Physical-phone and
safe realTorBox/WANreleasegate remains open. LegacyTogether/Pocket untouched.

## 2026-10-08 explicit graceful Host shutdown

Implemented local-only idempotent Host::request_shutdown, existing socket stop
watch and Axum graceful listener drain. Stop-before-serve rejects startup;
subscribe-before-check handles startup races. Ticker prioritizes cancellation,
rechecks after session lock and is joined on normal serve return. Cancellation
still uses ServerLifetime Drop to signal sockets/abort ticker. Callers need an
application shutdown deadline for arbitrary HTTP drain; no unbounded-wait safety
claim. No public shutdown endpoint or normal-app exit wiring added yet.

New transport test starts a playing phone, requests stop twice, checks 1012
server_restart, frozen timeline with desired_playing retained, phone disconnected,
no later snapshot mutations, listener rebinding and released Host. New unit checks
pre-start shutdown/idempotence/listener release. First compile used direct Snapshot
equality unsupported by its type; changed to full serialized-value equality,
without changing protocol types. Final fullsuite47passed(35unit/12transport),
eightignored; Windowsmetadata passed, active executableSHA256unchanged.

At09:52:14UTC f931b66c still running/native_tests,106.2min afterlaunch. PID5152/
start08:06:02.7851635UTCticks verified; requested7200, completed duration/metrics
null. This run's built source19b7ff3 excludes new lifecycle source, which is
separately transport tested. Continue protecting binaries/proxy/DLL/APKs/media/
adb reverse until runner completes. Physicalphone/safeTorBox/publicWANreleasegate
stillopen. LegacyTogether/Pocket unchanged. Next inspect completed receipt and
cross-player/clock/rate metrics once actually available; no oldsummary reuse.

## 2026-10-08 tighter-band two-hour emulator pass

f931b66c passed/complete10:09:29UTC, source19b7ff3, five native tests7345.557sec,
requested/completed steady7200sec. RunnerPID/start no longeralive, markerremoved;
receipt/metricsIDs match. Archived receipt/metrics/native+phone logs/result/
launch/stdout+stderr/retainedrollingjournal, clockanalysis and ad-hoc numeric
rate+steadyanalysis under work/passed-nudge-band-soak-f931b66c. SHA256manifest
covers19files/116535975bytes; preserve archive unchanged before next run.

71132all-phase crossmatches, median/p95/max53.5/140/302.5ms; provisional150ms
p95goalmet. RTT750/1108msmedian/max.70996steadyrows/onegeneration, selectedclock
agemax30000/history64/jump170ms, maxphoneauthoritydrift197/nativeage221ms;
allcountedsteadyviolationszero. Steadycrossdistributionidentical; worstphone+190/
PC-113,phone.995/PC1,nativeage24/matchdistance2.5ms. Journalvalidationpassed
zero drops/storageerrors. See NATIVE-STABILITY-2026-10-08.md for scope.

798observedphone rate transitions, max22inrolling60sec,13349nonnominalsamples,
range.995..1.005. Observedpropertychanges, NOT nativecommandcounts or perceptual
proof. P95improvedversusprevious178.5butmax302.5higherthan280.5; operatingconditions
vary, no controlledcausalclaim. Keep currentthresholds, no claimchatterabsent.
Physicalphone/safeTorBox/CDN/4K/publicWAN/actualPCrestartgate remains open.
Latestatomiclauncher and explicitHostshutdown are not inthissoakbinary; separately
47servertestspassed. Next latest-source600sec native regression at800RTT/100jitter
before deciding whether another long run adds necessary evidence. Legacyuntouched.

Latest-source regression launched source05b2e3c at10:22:34UTC; runnerPID12040/
start10:22:34.8752132UTCticks verified. Frontendcheck/build, Windows nativebuild,
Androidunit/build and bothAPK installs successful. Receipt3687cb38-aaff-4653-
b9e3-3c06de6ec0a6 running/native_tests, requested600sec, completed duration/metrics
null; configured800RTT/100jitter. Launchwork/lifecycle-medium-launch.json, logs
work/lifecycle-medium.*.log. Protect its executable/proxy/DLL/APKs/media/reverse.
Priorf931b66c pass is now archivedhistorical evidence, not latest-source pass.
No playbackthresholdchanges or extra featuremigration. Current app still lacks
coordinated player/server/tunnel normal-exit wiring; gracefulHostAPI tests do not
claim the native proof exercises app-exit behavior. Review real result next.

## 2026-10-08 latest lifecycle medium pass and native worker ownership

3687cb38 passed/complete10:35:40UTC on05b2e3c. Five native tests745.57sec,600sec
uninterrupted,6049crossmatches median/p95/max42/119/184.5ms, measuredRTT747/1188ms.
P95goalmet for this short run.5913steadyrows/onegeneration,clockagemax29999/
history64/jump172ms,maxauthoritydrift186/nativeage107ms,countedviolationszero.
Archived source evidence plus clockanalysis/hashmanifest under
work/passed-lifecycle-medium-3687cb38 before new runtime work. Physicalgatefalse.

PlayerController now retains native-thread and feedback-task ownership, explicit
stop watch/atomic flag, nonblocking Drop cancellation, and asynchronous shutdown.
Native mpv pause/destruction stays on its own thread; feedback detaches localPC.
Joining native thread uses blocking pool, five-second caller deadline. Concurrent
shutdown callers serialize and reuse cached outcome; cancelled/incomplete cleanup
is not later reported successful. Timeout leaves resources owned by workers, not
forcibly killed threads. Feedback selection stays fair between snapshots/reports;
stop checked every loop. No drift/source/clock thresholds changed.

Normal Tauri main-window close now retains native surface, requests app exit,
prevents exit while an async cleanup task requests Host stop and awaits player
cleanup, then awaits server up to3sec (abort on timeout) and permits final exit.
Started/finished flags avoid duplicate cleanup. No tunnel starts automatically yet,
so this exit wiring coordinates player/server only. All3actualDLL Windows tests
pass (new loaded-fixture cleanup+concurrent/idempotent shutdown; Drop eventually
releases Host/status; existing command-failure test),47server tests pass, Windows
full executable builds warning-free. UTF8 edits; diffcheck passes.

An attempted scripted owned-window-close runtime proof was rejected before
execution by automatic approval review: blocked by policy, no specific reason
provided. No proof script/file was created or run. Do not claim full-window exit,
1012notification/process/listener release is runtime-proven by that attempt.
Actual loaded headless native cleanup tests and real transport server shutdown
remain separate passing evidence. UI exit runtime validation is pending; no
approval request needed for continued independent authorized work. Latest source
requires a fresh connected native regression; previous pass predates worker/exit
changes. Physicalphone/safeTorBox/WANgate remains open; legacy/Pocket untouched.

Worker/exit-source regression now running: source0d24a81, runnerPID14260/start
10:55:21.4974931UTCticks verified. Frontendcheck/build, Windowsnativebuild,
Androidunit/build and bothAPK installs succeeded. Receiptf0ea9a8f-9008-4a9c-
b46b-2e96d3118c84 running/native_tests requested600sec at800RTT/100jitter;
completed duration/metricsnull. Launchwork/worker-exit-medium-launch.json;
logswork/worker-exit-medium.*.log. Protect ownedruntime untilcompletion. This
proof's final cleanup force-terminates its ownedhost, so even a pass does NOT
validate the new normal-window-close callback. Headlessplayer/transportshutdown
proofs remain separate; fullUIexit pending dueblockedscript. Review matched
receipt/outcome next and do not reuse3687cb38metrics aslatestsourcepass.

## 2026-10-08 native worker regression pass and shutdown journal acknowledgement

f0ea9a8f passed/complete11:08:09UTC on0d24a81. Five native tests744.853sec,600sec
steady,6052crossmatches median/p95/max55/130.5/223.5ms,RTT762/1086ms. P95goalmet
for thisshort run.5911steadyrows/onegeneration,clockagemax29999/history64/
jump132ms,maxphoneauthoritydrift147/nativeage109ms,allcountedviolationszero.
Receipt/metricsmatch; runnerownershipmarkerremoved. Evidence preserved unchanged
withclockanalysis/hashmanifest under work/passed-worker-exit-medium-f0ea9a8f.
Forcedhostcleanup inthisproof doesnotvalidate normalwindow-close callback.

Added fixed typed app lifecycle stages and queued Flush acknowledgement. Normal
exit records requested/player/server result/exitpermitted, awaitsuserspaceflush
up to2sec then permits exit. ExitPermitted doesnot imply successfulcleanup. Server
JoinHandle now retains fixed Result; taskjoin containingserveerror becomes
ServerStopFailed ratherthan misleadingServerStopped. No rawserverIOerrorslogged.
FlushFIFO covers acceptedprefix only, noexportcopy, nofsync/powerloss/abruptkill
promise. Prior/currentstorageerrorsreturnfailure; full/closedqueue/writerloss/
withheldackfailfixed, producersremainnonblocking, droppedrecordsnotrecovered.

Realfile test reads complete ordered typedshutdownrows afterflushwithoutExport;
fullqueue/deadwriter/acceptedrequestthenwriterloss and2sec timeouttested; existing
filesystemfailure also rejectsflush. All49servertests37unit/12transport pass,
threeactualDLL Windows tests pass, fullWindowsbuild warning-free/diffcheckclean.
No newwindowclose scripting attempted; earlier policyblockedUIproof remains
pending. Latestdiagnosticschanges notin f0ea9a8f, separatelycomponenttested; no
newplaybackthresholds or redundantlongsoak launched. Next independentwork can
address tunnel deployment prerequisites or lifecycle robustness while physical
phone/TorBox/WANgate open. LegacyTogether/Pocket untouched; credentialsunused.

## 2026-10-08 shutdown races at pairing and attachment locks

Fixed lifecycle checks that previously ran only before awaited locks. Pairing
open/claim now recheck stopping under pairing lock; rejected claim returns503
without consumingcode/incrementingattempts. Local open_pairing now returnsResult
withfixed serverstoppingerror; Tauri command anddebugproofstartup handle it.
No publicpairingwireformat/authpolicychange. LocalPCattach holds rolelock until
sessionlock/stateupdate succeed and rechecksstop afterbothwaits; no partialslot
reservation onfailure. disconnect releases session before acquiringconnections,
so new orderingintroduces no lockcycle. WebSocketupgrade rechecks afterrolelock;
clientjoin rechecksundersessionlock and uses existingstop cleanup ifshutdownwon.

Two deterministic regressions poll futures pending whileholdinglocks, request
shutdown, thenrelease: pairingclaim/open preserveexistingcode andzeroattempts;
localattach separatelyblockedonconnections/session neverconnects/reservesPC or
changesrevision/generation. These tests would expose priorpre-wait-onlyguards.
Firsttestcompile needed explicit[u8;32] digesttype; fixedtestonly. Final51server
(39unit/12transport),3actualDLLWindows tests pass; Windowsnativebuildwarning-free,
diffcheckclean. Remoteupgrade lock-race branch not separatelyforced intransport;
existingrealtransportshutdown/auth/reconnect regressions stillpass. These are
post-wait checks, not a claim to preempt an alreadycommittingsynchronousoperation.
Physicalphone/safeTorBox/WANgate and priorblockedfullUIexit validation remainopen.
Latestsource needs shortconnectednative pairingregression, notanotherlongsoak.
LegacyTogether/Pocket/credentials untouched; no activeproofmarker atstart.

Pairing-stop smoke96127c4b completed11:56:38UTC on5563097: five native tests
201.787sec, requested/completedsteady60sec, receiptmetricsIDs match, ownership
markerremoved. Frontend/Windows/Androidbuilds+installs passed. Functional pass,
NOT150msqualitypass:723crossmatches median/p95/max47/155.5/243.5ms, provisional
p95goalfalse; measuredRTT777/1124ms.591steadyrows/onegeneration,clockagemax29999/
history64/jump173ms,maxauthoritydrift225/nativeage105ms,allcountedviolationszero,
journalvalidationzero drops/io. Archived evidence+clockanalysis/hashmanifest in
work/passed-pairing-stop-smoke-96127c4b beforefuturetests. Do not inherit earlier
140ms twohourqualityresult onto latestsource. Shortsmoke/operatingconditions
differ; cannotconclude regression/causality or relaxthresholds. No correction
constantschanged. Latestsource has functional native evidence; long stability,
physicalphone/safeTorBox/WAN/UIexitremain separate acceptance work.

Next useful independent work: bind phone proof rows explicitly to receipt runID
(currentrows lackID; archives keepassociation manually), and investigate bounded
credential/pairing setup robustness. Do not launch redundant long emulatorruns
solely to chase a passingp95. No nativeproofcurrentlyowns runtime atcompletion.

## 2026-10-08 binding native proof observations to receipt identity

No activeproofmarker atstart; latest96127c4b evidence alreadyarchived. Added
proof_log_schema2, runnerUUID environment toWindows/debuginstrumentationarg to
Android. PC+phone proofrows now include run_id; invalid/missingdebugUUID fails.
Productionplayback unaffected. Runner restores previousenvvalue oncleanup.
Comparison requiresExpectedRunId andchecksallPC/phone rows beforefiltering;
clockanalyzer checksallphases againstreceipt. Summariesreportlog_identity_verified.
Historicalunbound input requires explicitcompatibility, marksassociationfalse,
neverpermits mismatchedIDs; Python schema2 cannotdowngrade via legacyoption.
Olderarchives unchanged; UUIDassociation not cryptographicauthentication.

FivePython analysis tests pass (includingwrong/missing/mixedallphaseIDs andlegacy
optin), PowerShell comparison regressions pass,3actualDLL Windows tests pass.
No sync/source/clock thresholds changed. Androidinstrumentation compilation and
freshconnected shortproof stillrequired tovalidate newlogger/runnerrouting.
Physicalphone/safeTorBox/WAN/fullUIexitvalidation remainopen; nolegacychanges.

Identitysmoke3846ac42 passed/complete12:21:21UTC on4de65bf; five native tests
205.994sec, requested/completedsteady60sec, schema2 receipt andmetricsIDs match.
Windows/frontend/Androidunit+instrumentationbuilds andAPKinstalls succeeded.
723crossmatches median/p95/max40.5/107.5/158ms, RTT758/1309ms; p95goalmet inthis
short run, not newstability/causal evidence. Comparisonlog_identity_verifiedtrue.
StrictPythonclockanalysisalsoidentitytrue,591steadyrows/onegeneration,clockage
max29995/history64/jump75.5ms,maxauthoritydrift111/nativeage104ms,zero counted
violations. No correctionthresholds changed; previous155.5msmiss preserved.

Actualnewlogs pairedwitholder96127c4breceipt/expectedUUID were rejected byboth
analyzers beforeoutputcreation. This intentionalnegativecheck emitsfixed
clock_analysis_failed, not a failednativeproof. Allsourceevidence+strictclockreport
andhashmanifest archived underwork/passed-identity-smoke-3846ac42. Runner exited,
ownershipmarkerremoved; no nativeproof currentlyowns runtime. FullUIexit,
physicalphone/safeTorBox/WANgate remainsopen; legacy/Pocket untouched.


## 2026-10-08 pairing vault concurrent startup and bounded reads

Pairing vault startup now opens the existing file directly and reads at most
16 KiB plus one overflow-detection byte on that same handle. Empty and oversized
vaults fail without replacement. Other open/decrypt/format failures still fail
closed; only NotFound begins creation. This removes the separate exists/metadata
checks and bounds the bytes actually read even if the file changes size.

Creation still writes and syncs a unique temporary DPAPI file before create-only
hard-link publication. If another creator publishes first, the losing process
zeroizes its generated plaintext and loads the winning persisted vault. It never
continues with unpublished credentials or overwrites the winner. Other publication
errors remain failures; temporary-file cleanup remains best effort.

Eight concurrent temporary-vault startup calls all returned the independently
reloaded persisted identity, leaving exactly one file. Their barrier aligns call
starts, not the internal publication point, so this is concurrent integration
evidence rather than guaranteed coverage of every interleaving. Actual DPAPI
round-trip/tampering and empty/oversized preservation tests also pass. All53server
checks (41unit/12transport;8ignored), three actual-libmpv Windows tests and Windows
metadata compilation pass; diffcheck clean. No existing encrypted credentials
were accessed and legacy/Pocket stayed untouched.

No new native playback proof was launched for this storage-only change. Debug
proof startup uses ephemeral credentials and would not validate production DPAPI
startup anyway. Normal full-app pairing startup/exit, physical-phone two-hour
acceptance, safe TorBox delivery and public WAN validation remain open. Latest
native receipt remains the completed identity smoke3846ac42 on4de65bf; it must
not be attributed to these later vault changes.


## 2026-10-08 Android pairing read/write size symmetry

No active native-proof marker or host process at start; identity smoke3846ac42
remains the latest completed playback receipt. Android PairingStore now opens
AtomicFile for reading and bounds the actual stream to4096 bytes plus one overflow
byte, replacing metadata checks followed by unbounded readFully. Fragmented reads
are handled; the stream is closed on success/failure. Existing AtomicFile recovery
and instance synchronization remain in use. There is no new cross-instance/process
locking or claim of power-loss recovery validation.

Save rejects encrypted payloads over4096 bytes before startWrite. Previously a
large accepted server address could replace valid pairing with a file load would
reject. New emulator test proves the large address passes URL validation and
specifically fails the size guard, leaving ciphertext and loaded token unchanged.
Empty/4097-byte files fail without replacement. Temporary Keystore aliases/files
are unique and cleared; saved production pairing and PC credentials unused.

All20 Android JVM tests pass, including exact-bound fragmented reads and a source
that never reaches EOF stopping at4097 bytes. Debug app/instrumentation APK builds
and installs succeeded; three PairingStore instrumentation tests pass on existing
emulator5580 (final0.314sec), including real Keystore roundtrip/tampering/forget and
oversized replacement preservation. Result saved work/android-pairing-bound-test.txt.
Known Gradle/SDK warnings unchanged; diffcheck clean. No playback thresholds or
protocol changes. No new connected playback/long soak launched for storage-only
change; these results do not establish physical-phone stability, production full
app startup, safe TorBox delivery or public WAN behavior. Legacy/Pocket untouched.


## 2026-10-08 cancel obsolete local source preparation on session takeover

No native-proof marker, prior identity runner or native Windows host was active.
Found a local-proof/session handoff race: loadTest began fixture preparation before
pairing, but its late result could call loadLocal after connectSession had granted
source ownership to the authoritative session. Multiple local requests also had
independent outstanding preparations.

Added a main-thread LocalSourceLoader owning one pending job. A newer local request
cancels the previous one; session connection and controller close cancel local
preparation. Delivery checks coroutine activity after preparation returns, including
non-cancellable/blocking cleanup. This prevents stale results from loading media;
it does not forcibly interrupt underlying blocking file IO. Existing session
command invalidation still handles native work already queued before takeover.

Two controlled JVM tests hold a non-cancellable preparation until after cancellation
or supersession, then verify zero stale deliveries and exactly one current delivery.
All22 Android unit tests pass. App and instrumentation APK builds/installs succeed;
NativePlayerTest passes on emulator5580 in11.25sec, exercising packaged libmpv,
rendered video, local load, play/pause/seek, rotation/resume, decoder failure and
local-source recovery. Result work/android-local-source-test.txt. Known toolchain
warnings unchanged; diffcheck clean. These tests cover cancellation ownership and
normal native integration separately, not a real paired mid-preparation handoff.
No new connected playback receipt/long soak created; latest3846ac42 remains prior
source evidence. Physical-phone two-hour gate, safe TorBox/WAN/fullUIexit remain
open. Legacy/Pocket and saved credentials untouched; no sync thresholds changed.


## 2026-10-08 optional dedicated tunnel startup and Host lifetime integration

No active native-proof marker at start. Added bounded strict TunnelDeployment
loader for app-local-data/control-tunnel/deployment.json, fixed executable and
credential paths, per-attempt SHA-256 verification, Windows read-only sharing
handle retained across process creation, and background supervised launch. Missing
manifest disables tunnel; invalid setup yields fixed local failure while app stays
available. No actual production manifest, credential, tunnel or DNS created/read.
Native proof startup always bypasses deployment. No legacy references modified.

Windows startup now retains optional supervisor and local process-status receiver;
exit awaits tunnel cleanup before player/server and journal flush. Added fixed
TunnelStopped/TunnelStopFailed app stages. Supervisor observes weak Host shutdown/
disappearance even during30sec retry backoff (250ms polls). Server task completion
requests Host shutdown; final compilation required cloning its Arc before serve
because serve consumes the receiver. Fixed that compile error, then tests/check
passed. No worker/thread can be forcibly interrupted while inside blocking launch;
existing timeout leaves cleanup owner alive. Trusted installer/private ACLs and
production account/DNS still required; local manifest hash is integrity pin only.

All57server checks (45unit/12transport;8ignored),3actualDLLWindows tests and Windows
metadata compilation pass. New provisioning tests verify strict/Unicode/oversized
rejection, unchanged failure boundaries, hash recheck and actual held-file Windows
write/delete denial. Controlled supervisor test verifies running-child cleanup and
long-backoff cancellation on Host shutdown before explicit supervisor stop. Existing
actual-child/job ownership and real transport regressions pass. No full configured
UI startup/exit or public tunnel runtime proof; earlier blockedUIexit boundary
remains. Latest connectednative receipt3846ac42 predates this change. No sync
threshold/media code changed. Physicalphone/safeTorBox/WAN release gate remainsopen.

FullWindows native build also passed warning-free. Added deployment-to-supervisor
integration test using a synthetic image with a deliberately mismatched hash:
start enters fixed Start backoff without launching/writing runtime config, Host
shutdown stops it before explicit shutdown, and exported journal excludes private
inputs/path/hash error text. Final57server checks pass. This is a negative local
integration test, not successful configured cloudflared/app/WAN acceptance.


## 2026-10-08 explicit connector working directory and positive deployment launch

No native-proof marker at start; working tree clean. OwnedTunnel now specifies the
dedicated runtime working directory rather than inheriting the caller's directory.
Existing real native echo regression verifies the child's observed directory along
with arguments/environment; command builder regression verifies explicit cwd.
New deployment test hashes/copies the current real Rust test image into a temporary
cloudflared.exe path with a synthetic untouched credential placeholder, successfully
starts via retained verification handle, observes expected unsupported-CLI nonzero
exit, and verifies repeated stop/config cleanup. It never executes cloudflared or
contacts Cloudflare, and never reads production credentials. This extends previous
negative-only deployment evidence to successful native image creation, not WAN or
configured full-app acceptance. Runtime ACL/DLL dependency protection still pending.

All58server checks (46unit/12transport;8ignored),3actualDLLWindows tests and full
Windows native build pass, diffcheck clean. No new playback proof was necessary
for cwd/launcher component changes; latest3846ac42 remains older playback evidence.
Legacy/Pocket untouched. Physicalphone twohour/safeTorBox/publicWAN/fullUIexit gates
remainopen; no thresholds or media transport changed.


## 2026-10-08 redirect guard and stricter media capability probe

No active native marker at start; tree clean. A proposed create-only Windows
private-directory ACL setup helper/test was blocked before execution by automatic
approval review with only "blocked by policy". Neither helper script exists and
no permissions changed. Did not retry that action; provisioning remains pending.
Independent read-only deployment namespace guard now rejects existing ancestor
reparse points on manifest/exe/credential/runtime paths; runtime checked before/
after creation. Real temporary Windows junction test verifies rejection and target
preservation, removing junctions themselves before temp-tree cleanup. Not a locked
namespace or protection against concurrent privileged swaps. All59server checks
(47unit/12transport;8ignored),3actualDLLWindows tests and native build pass.

Continued independent media work: probe requests identity encoding and validates
exact0-1023 Content-Range, known total matching canonical size when present, no
compression, optional consistent1024 Content-Length, retaining bounded body/fixed
errors. Eight resolver tests pass including actual local TCP probe and malformed/
duplicate/overflow headers. Synthetic credentials only, no API/CDN/productionsecret
access. No proxy, account-key guard bypass, identity hash claim or playback change.
SafeTorBox/physicalphone/WAN/fullUIexit and permission provisioning gates remainopen.


## 2026-10-08 instrumentation suite-count regression repair

Started fresh connected60sec proof on d699acc, run d2f3aec8, with rebuild/install
and800msRTT/100msjitter. During execution found runner still required OK(5tests)
after PairingStoreTest gained a third case, making selected suite six. Let current
run finish unchanged and preserve its receipt; never relabel a failed receipt.
Added pure Assert-NativeProofResult helper requiring exactly one whole-line success
summary at declared count, rejecting failure/crash markers. Runner now declares
expected_native_tests6 in its receipt. Regression checks accept6 and reject5/7,
empty/duplicate/prefixed summaries and failure/crash output. All checks pass.
This changes test-result interpretation only, no binaries/thresholds/API/credentials.
The active test still uses the old parsed runner and will retain its actual result.


The original d2f3aec8 run finished all6 native tests successfully in209.531sec but
old runner marked receipt failed/native_tests due expected5. Receipt preserved
unchanged with raw logs/journal/artifact hashes inwork/failed-runner-count-d2f3aec8;
no drift metrics were produced by that failed wrapper. Corrected runner retry
c8408872 on e3189af used SkipBuild; SHA-256 of Windows exe and both APKs match
original d699acc-built artifacts. All6 tests pass209.025sec; receipt passed/complete,
requested/completedsteady60sec. Functional pass, qualitymiss:732matches median/
p95/max44.5/163.5/199.5ms, provisional150msgoalfalse, measuredRTT776/1158ms.
RunIDverifiedtrue, strictclockanalysis590steadyrows/onegeneration,clockagemax29998/
history64/jump126.5ms,maxauthoritydrift162/nativeage105ms,allcountedviolationszero.
No thresholds changed or redundantp95chasing runs planned. Evidence archived with
hashmanifest underwork/passed-current-lifecycle-smoke-c8408872. Ownership marker
removed; no proof owns runtime. This run bypasses provisioned tunnel startup and
uses synthetic fixture/ephemeral credentials: no publicWAN/safeTorBox/normalUIexit
or physicalphone acceptance claim. Later fullUIexit test still pending priorblock.

Public primary-source research found a new possible safe delivery route: TorBox
T3 S3-compatible read-only service at t3.nexus (official help updated2026-07-02),
AuthID as access-key ID, account APIkey as secret key, documentedV2 signature mode.
Potential file-scoped query signatures could avoid disclosing the secret key, but
presigned support/redirect chain/range behavior/canonicaltorrent-file mapping are
NOT established. Do not relax URL/account-key guard or send URLs toAndroid based
on compatibility wording alone. Next useful priority: study official S3 signing
vectors, implement/test signing independently, then PC-only sanitized read-only
capability investigation using existing encrypted credentials if justified. No
liveT3/API/account request or key access occurred in this turn. Official docs also
state15min refresh and currentlyCloudflare-hosted serving; assess whether provider
hosting meets direct-media requirements separately from our control-only tunnel.

## 2026-10-08 T3 capability investigation (not application delivery)

Started clean at7e5d2aa; no native ownership marker/running proof. Added independent
PC-only S3 V2 GET signer behind optional experimental-t3 feature (disabled in apps).
Exact encoded path/GET/expiry signed,30–900sec TTL, strict path guards, existing
account-key URL guard retained, bearer wrappers noDebug/Serialize. Seven new tests
plus existing eight pass; default eight pass separately. HMAC vector RFC2202,
complete GET independently crosschecked Python against AWS public synthetic input.
No code copied from AWS; its current renamed-bucket example signature doesn't
match that exact input. Source/protocol references and boundaries inTORBOX.md.

Read-only sanitized PC diagnostic accessed existing torbox.dpapi; never exported
key/AuthID/filenames/URLs/rawheaders/bodies. Profile/header-authlisting200; service
ignoredmax-keys50,returned1000/truncated,bodyhardbound2MiB/first50inspected. Presigned
candidate307relative then206 afteroneguardedsame-originredirect. Returned HTML,
range0–464,requestedspan/totalmismatch. Nooriginalmediacapabilityaccepted. Expired
andinvalidsignaturevariantsalso206;notauthbypassproof,error/login/cachebehavior
possible. Finaldiagnosticexit1,work/t3-capability-investigation-sanitized.log.
No safe query-signing/fileidentity/expiry contract established. Don't repeat same
live probes without a new lead. NoAndroid delivery/sourceintegration/credential
mutation/legacyedits/emulatorchanges. Both application features remain disabled.
SafeTorBox/physicalphone/WAN/normalUIexit gates remain open; native c8408872 and
its recorded qualitymiss unchanged. Continue independent core correctness work.

## 2026-10-08 bounded control writes for reconnect recovery

Independent networking audit found outgoing socket.send could block indefinitely
inside the control loop, preventing its heartbeat/recv branches from running.
That could retain a phone's reserved role and delay authority freeze/reconnect
even though idle-peer detection exists. Every normal outgoing snapshot/ack/clock/
ping/pong/close write now has an independent five-second Tokio deadline. Timeout
returns failure to the existing client cleanup path, which freezes/disconnects
before any optional bounded shutdown close and releases the role. Shutdown watch
retains biased priority and never waits for this write deadline. Typed journal
event control/write_timeout distinguishes this from ordinary disconnect.

Added two virtual-time checks: pending future survives4999ms, expires at5000ms
and is dropped; ready/error writes finish immediately, shutdown wins when both
stop and expiry are ready. Existing stop-before-write and pending-send cancellation
test updated for explicit outcome. All61server checks pass (49unit+12realWebSocket
transport,8ignoredfixture/optionalchecks); Windows custom-protocol cargo check
passes. Existing transport checks cover role release, authority freeze/reconnect
snapshot and server shutdown. No actual OS socket-backpressure injection or new
connected native/physicalphone run was performed; don't claim those from timer
tests. No native rebuild/emulator/APK change; c8408872 remains historical evidence
from earlier binaries. Experimental T3 code remains disabled in applications.

## 2026-10-08 real TCP backpressure regression

Added a real localhost WebSocket test with small real TCP buffers and a peer that
keeps sending upstream but stops reading downstream. Synthetic native READY and
Executed feedback puts authority into active playback; bounded broadcast updates
then exercise the production five-second write deadline. Observed write_timeout
in the typed journal, authority frozen/newgeneration without execution fault,
oldphone disconnected, and replacement authenticated socket receiving sameepoch/
newgeneration snapshot. This checks actual OS socket backpressure and role release,
not native playback or physicalphone networking. No timeout overrides, raw/private
data, TorBox or tunnel traffic. All62server tests pass (50unit+12transport;8ignored).

## 2026-10-08 reject unresolved source changes after a successful native load

Found both clients could retain their previous native fixture after accepting a
new unsupported identity. Windows displayed an error but didn't set native_failure,
so the old source could still appear ready. Android only checked fixture identity
inside its !loaded branch, allowing a previous load to bypass the guard. Windows
now sets terminal LoadFailed, preventing READY/seek/execution and reporting the
existing authoritative player fault. Android checks supported identity on every
nonstale snapshot before loaded state or mutation; existing onMessage loss handler
invalidates commands/pauses/disconnects on rejection. Stale revisions are ignored
before the source guard. No fake TorBox mapping or URL delivery was introduced.

Four Windows actual-DLL tests pass, including fixture->unresolvedTorBox fault and
explicit fixture-load recovery toPC READY. Android22JVM tests/builds pass; standalone
SessionIdentityTest passes on emulator(1test,0.393sec). It exercises real controller
acceptance with synthetic snapshots and both loaded flags via private reflection,
plus supported identity/stale-unsupported snapshot; no public test-injection API.
It isn't a native playback or liveTorBox check. Connected runner now explicitly
includes this class and expects7tests, preserving old six-test receipts unchanged.
Next validation is fresh connected PC/emulator proof on these app sources.

## 2026-10-08 expanded native proof found concurrent TLS bundle installation

Fresh 5f975ae5-175b-4d51-a059-73167e81b741 on8d789e5 ran seven selected tests at
800msRTT/100msjitter/60secsteady. SessionPlaybackTest completed, but NativePlayerTest
failed native initialization after SessionIdentityTest; total200.13sec,1failure.
Receipt remains failed/native_tests, completedsteady null, no fresh drift metrics.
Rawreceipt/results/PC+phone logs/journal/artifacthashes/boundary/hashmanifest archived
underwork/failed-native-init-5f975ae5 before any rebuild/overwrite. Marker removed.

Investigation identified fixed native-system-roots.tmp used by concurrent adapter
initializations. Added eight-worker barrier test performing actual Android trust
bundle installation and X.509 parsing; old implementation reproduced rename failure
in4.341sec. Now each writer uses exclusive File.createTempFile in app-private files,
then atomically renames a complete PEM. Only system roots still selected, no TLS
verification changes, no credentials or certificate contents printed. Concurrent
test verifies complete/nonempty/equal certificate counts and no leftover ownedtemps.
Afterfix, NativeTlsTest+SessionIdentityTest+NativePlayerTest pass together(3tests,
17.435sec), including native load/play/pause/seek/rotation after identity test.
Android22JVM checks/builds pass. This reproduces/fixes trust-file collision; it does
not claim all potential async player-close races are solved. Full runner now
explicitly selects NativeTlsTest and expects8, without changing failed7test receipt.
Next step: fresh full connected8test proof with current Android artifacts and
unchanged8d789e5 Windows binary. No physicalphone or safeTorBox acceptance claim.


Full retry04f494d0 onef7fb71 passed all8tests in213.065sec; receiptpassed/complete,
requested/completedsteady60sec at800msRTT/100msjitter.727crossmatches median/p95/max
49.5/152/183ms, provisional150msgoalfalse;RTT797/1336ms. Strictclock590steadyrows,
onegeneration,agemax29949/history64/jump77ms,authoritydrift105/nativeage106ms,
allcountedviolationszero. No thresholdtuning or repeatedp95chasing. Archive
work/passed-source-guard-tls-smoke-04f494d0 holds receipt/logs/journal/metrics/clock/
artifacthashes/boundary/evidencehashmanifest. Windowsbinaryhashmatchesfailedrun,
built8d789e5; Androidsourcesef7fb71, bothAPKs differ. Ownershipmarkerremoved.
Functionalpass/qualitymiss bothrecorded; previous failedreceipt preserved.

Followup ownership audit remains useful: Android close currently captures core
onMain before queued native cleanup; a handle created late during cancelled
initialization may need worker-owned cleanup. This potential ordering gap was not
deterministically injected here; the concurrent TLS fix does not prove all early-
close races solved. No credential access/account mutations/legacy edits this run.
Physicalphone/safeTorBox/publicWAN/normalUIexit gates remainopen. Current short
nativepass does not replace the earlier revision's historical two-hour run.

## 2026-10-09 user network disruption: connected testing on hold

The user reported internet/game connectivity failures and confirmed connectivity
returned after disabling a Rust network adapter in Control Panel. Read-only
inspection identifies vgate0 as Rust Wintun Tunnel Tunnel, disabled, using
wintun.sys version 0.14.0.0; wintun driver is stopped/manual. Preserve that state.
No Together or cloudflared process was running at inspection. WARP service is
running, but warp-cli explicitly reports Disconnected / Manual Disconnection.
Ethernet owns the IPv4 default route; no IPv4 split-default /1 routes were found.
WinHTTP reports direct access. Ethernet DNS is 8.8.8.8/8.8.4.4; ownership is not
established, so no DNS reset or other network mutation was attempted.

DNS and HTTPS checks to Microsoft and Steam returned HTTP 200. Those checks do
not validate game UDP/matchmaking or establish the earlier failure mechanism.
The user's adapter-disable recovery is stronger evidence of adapter involvement,
but creation/ownership and historical routes remain unverified. Searching current
scripts/crates/apps/docs found no vgate/Wintun/WARP CLI or system route/DNS setup.
No native-proof ownership marker exists. No players, tunnels or provider probes
were launched, and no services/adapters/routes/proxies/firewall settings changed.

AGENTS.md now restricts autonomous continuation to offline source work and local
unit checks until the user authorizes resuming network/tunnel testing. Do not
enable/recreate vgate0. This hold applies to scheduled continuation too. Native
and physical-device acceptance gates remain unchanged and unfulfilled.

## 2026-10-09 offline Android native ownership correction

Respected the network-impact hold: no tunnel, connected player, emulator, provider
probe or network-setting changes. Working tree was clean and no native-proof
ownership marker existed before editing. Audited LibMpvAdapter's early-close path.
close() captured core on Main before queued cleanup; a synchronous creation already
running on the calls worker could publish a handle after that null capture, while
cancellation discarded its withContext result. Cleanup now takes core on that same
worker after synchronous creation/initialization, then clears and disposes it.
The closed flag is volatile for worker visibility. The Main closed-result branch
returns without competing disposal; queued close owns cleanup. Upstream create,
initialize, detachSurface and close are synchronous, checked in the pinned source.

Offline compileDebugKotlin and testDebugUnitTest succeeded (22 tests, zero failures
or errors). These existing JVM tests do not exercise native close-during-create;
the ownership fix still needs a deterministic native lifecycle regression and
connected validation after the hold is lifted. No new playback stability claim.
Surface-destruction cancellation ownership remains a separate audit item.

## 2026-10-09 offline surface cleanup cancellation regression

Followed the remaining surface ownership path. onSurfaceTextureDestroyed returned
false (retaining Android's texture), but queued detach/release on the playback
scope. close() could cancel that pending body after the callback had cleared the
adapter's surface/texture fields, leaving neither path to release those resources.
Added PlayerCleanup with an independent supervisor on the existing serial native
dispatcher. Surface destruction queues its release there. close() disconnects the
TextureView listener, queues final native/surface cleanup and completes the cleanup
parent job, which waits for queued children instead of cancelling them. With no
owned surface, destruction returns true for Android to release the texture.

Two local regression tests exercise the actual cleanup queue: a latch blocks the
native worker, surface cleanup is queued, playback is cancelled, and final cleanup
must still release surface then native state; a completed queue rejects further
release submissions. Offline compileDebugKotlin/testDebugUnitTest passed: 24 tests,
zero failures/errors. These tests use synthetic release callbacks, not Android
Surface/mpv, so rotation/early-close/device integration remains unvalidated pending
the network hold. No tunnels, emulator, provider probes or network-setting changes;
no native-proof marker or active Together/cloudflared process was found. Existing
adb process was left untouched. Legacy apps/credentials remain untouched.

## 2026-10-09 offline teardown failure containment

Audited failure behavior in the new independent cleanup scope. A SupervisorJob
isolates siblings but does not handle uncaught launch exceptions: native teardown
failure could still reach Android's process exception handler. PlayerCleanup now
requires a failure callback and installs CoroutineExceptionHandler; the adapter
logs only the fixed cleanup_failed event, never exception details or media URLs.
Surface and final cleanup use nested finally so texture release is attempted even
if Surface.release throws. A local test injects a failing surface release and
verifies exactly one sanitized callback and successful subsequent native cleanup.

Offline Kotlin compilation and all 25 JVM tests passed, zero failures/errors.
This is synthetic teardown fault coverage, not evidence that Android native close,
rotation or reconnect integration passes. Working tree started clean, native-proof
marker absent. Network-impact hold remains: no player/tunnel/emulator/provider
launches or system-network changes, no credentials or legacy projects accessed.

## 2026-10-09 connected development resumed; product renamed 2gether

The user explicitly resumed development after personally enabling the Rust
adapter (Windows now reports vgate0 Disconnected). Ethernet still owns the default
route, no split-default /1 route observed. Current source searches show no
vgate/Wintun dependency. Adapter owner remains unestablished; no uninstall or
network-setting change attempted. Added requirement to prove the app works without
this adapter before release, and to scope requested eventual removal by ownership.
AGENTS and the existing active 30-minute automation now permit connected app/
emulator testing over loopback/adb and ordinary HTTPS/WSS, while prohibiting system
adapter/DNS/route/proxy/firewall/VPN changes. No adapter toggling should be required.

User renamed the rebuild to 2gether. Updated Windows product/window/page branding,
Android launcher/header/server-field wording and README. Kept repository path,
package IDs, protocol names and credential/pairing storage for compatibility.
Windows Svelte checks: zero errors/warnings; frontend production build succeeds.
Android offline unit tests and both debug/instrumentation APK builds succeed (25
local tests). Started existing Together2Test emulator hidden for native validation;
no new AVD or virtual network adapter created. Native results will be recorded
separately; this rename/build does not satisfy device/TorBox/WAN stability gates.

## 2026-10-09 fresh renamed-app native validation

Source/artifacts built at3ab4e18; subsequent commits changed requirements only.
Fresh23036109-fd81-4a11-a399-cbecf42064fa passed all8instrumentation tests in211.347s,
receiptpassed/complete with60requested/completedsteadyseconds,800msconfiguredRTT,
100msjitter.729matches median/p95/max38.5/93.5/157ms; provisional150msp95goalmet.
MeasuredRTTmedian/max774/1163ms. Strictclock594steadysamples,onegeneration,
selectedagemax26004ms/history64/offsetjump81.5ms/authoritydrift159/nativeage103ms;
allcountedsteadyviolationszero. RunIDsverified. No thresholds changed.

Archivework/passed-2gether-rename-cleanup-23036109 holds receipt/results/logs/journal,
metrics/strictclock/artifacthashes/boundary/evidencehashmanifest. Existing older
archive remains intact. This is current-build short emulator/synthetic proof only;
no safeTorBox/publicWAN/physicalphone/twohour/normalUIexit claim. Runner bypasses
production tunnel and force-kills its owned PC host on cleanup; marker removed.
Wintunadapter remained Disconnected throughout read-only observations; no settings
changed. Driver provider WireGuard LLC/oem41.inf identifies a generic component,
not which program created the vgate0 adapter; scoped uninstall still unestablished.

User clarified full beta scope: fresh2gether fonts/icons/interface on both platforms
and Android in-app updater before first distributed2.0Beta, no1.0APKupgrade needed.
Beta must be complete rebuild, not the proof UI. First pass physical playback gate,
then restore/rebuild features and polish. Requirements persisted inAGENTS/productdoc.

## 2026-10-09 five-minute continuation and current-build two-hour soak

User requested shorter continuation intervals; existing automation remains ACTIVE
and is now every5minutes, not a duplicate task. Prompt explicitly protects a live
native soak's artifacts/emulator/adb connection and prohibits duplicate launches.
Visual preferences persisted: round controls/icons, liquid-glass/iOS-like polish
on both apps, richer Windows effects allowed, battery/performance-sensitive native
Android implementation. Full visual work remains deferred until physical gate.

Started04311a62-6bc9-47ad-b418-660692d16144 with7200requestedsteadyseconds at800msRTT/
100msjitter using unchanged3ab4e18 app artifacts (source since then docs-only).
Receipt is running/native_tests with8expectedtests; runnerPID14036/starttime
verified against ownership marker. Do not rebuild/reinstall/touch emulator or
proof logs while it owns them. Previous shortpass is archived separately. No
completed duration or metrics yet; no two-hour pass claimed. Even a pass remains
emulator/synthetic evidence, not physical-phone/TorBox/WAN acceptance. No adapter
or other system-network settings changed. Independent source work may continue.

## 2026-10-09 independent clock-analysis evidence checks during live soak

Verified04311a62 stillrunning/native_tests and runnerPID14036/starttime match;
working tree initially clean. Did not touch its binaries, DLLs, APKs, media,
emulator, adb connection, logs or runner/Analyze-SessionProof scripts. Improved
the separate Python post-completion clock analyzer: reject monotonic timestamps
going backward across generation boundaries (previously checked only within one
generation), reject boolean requested durations, and report actual observed steady
span and largest sample gap. These describe supplied rows, not uninterrupted-play
acceptance; physical gate remains false and thresholds are unchanged.

Three new local regressions failed against old implementation (two failures and
one missing-field error), then all8Python checks pass after the fix. No application
build or network/provider probe performed. Analysis of the running receipt/logs
was not attempted; live soak continues with its original artifacts. Future
post-completion reports may use the improved analyzer with a new output path,
without overwriting previous archived evidence or claiming phone acceptance.

## 2026-10-09 independent archived-evidence checksum validation

Live04311a62 runnerPID14036/starttime still match; native receipt remains running.
No owned artifacts/emulator/adb/proof logs or running analyzer were modified.
Added a separate read-only verify_evidence_archive.py for saved archives: stream
SHA256 checks against their manifest, reject missing/modified/unlisted entries,
duplicate normalized paths, absolute/traversal/alternate-stream paths and resolved
paths outside the archive. CLI returns only counts or a fixed error, not filenames,
private paths or file contents. It explicitly does not authenticate the manifest,
prove startup binary identity or satisfy any physical release gate.

Four local tests pass covering valid inventory, changed/missing/extra files,
unsafe/duplicate paths, invalid hashes and sanitized CLI failure. Read-only check
of historicalpassed-2gether-rename-cleanup-23036109 verifies17files, without changing
its manifest/evidence. No current running logs analyzed, no Rust/Android build,
provider request, credential access, adapter/network changes or legacy edits.

## 2026-10-09 public TorBox contract research during live soak

04311a62 runnerPID14036/starttime verified alive, receipt running/native_tests;
initial tree clean. Read only public provider documentation/OpenAPI, not account
or media endpoints. Official troubleshooting describes presigned-token expiry and
one device/app per link; separate-device leases remain required. OpenAPI exposes
getstreamdata token/presigned_token parameters but does not specify a sufficient
original-file/file-scoped capability contract. No declaration that blocker is fixed.

Persisted a concrete next investigation inTORBOX.md: determine onPC whether an
account-bearing intermediate URL resolves to a safe final capability, with explicit
origin/redirect policy and existing account-key/range/identity guards retained.
This is new documentary reasoning, not permission to forward credentials to unknown
hosts or repeat identical live probes. No code copied from provider, no credential
access, authenticated request, application build, emulator/adb change or live-log
analysis. Running soak's artifacts and analysis scripts remain untouched.

## 2026-10-09 independent instrumentation acceptance audit

04311a62 runnerPID14036/starttime verified alive, receipt running/native_tests;
PC proof log metadata advances. No running resources/scripts changed or rebuilt.
Source review found PairingConnectionTest and SessionPlaybackTest intentionally use
JUnit assumptions when private proof config is absent. Current runner checks that
config exists before instrumentation and requires8tests plus fresh session/journal
evidence, so this observation does not establish a skipped test in current or prior
proofs. However, terse OK(count) output alone cannot prove zero assumption skips.
Follow up after the soak: capture structured instrumentation result/status codes,
reject ignored/assumption/skipped selected tests, and add pure-parser regression
coverage before integrating a future runner. Preserve today's result semantics
and scripts during this run; do not retroactively relabel archived passes.

## 2026-10-09 independent raw instrumentation status parser

Verified04311a62 live runnerPID14036/starttime and running receipt before changes;
PC log metadata advancing. Implemented standalone verify_instrumentation.py for a
future raw-mode runner without touching current scripts/resources. Independently
interprets AndroidX InstrumentationResultPrinter's documented status fields/codes
(primary source linked in module). Requires matched sequential start/success packets
for expected count, unique test identities and successful final instrumentation
code; rejects ignored(-3), assumption(-4), failure/error, crash, truncated/mismatched
packets and duplicate final statuses. Retains protocol fields only, not stream/stack;
CLI failures are fixed strings. No upstream implementation copied.

Five parser/CLI regressions pass; all17Python checks pass together. This is synthetic
raw-protocol coverage only: not integrated into the live runner, not verified against
an actual raw-mode emulator transcript and not a new native pass. Integrate/capture
raw output after the soak ends, preserving its current terse output and evidence.
No application build, emulator/adb/provider/credential/network-setting changes.

## 2026-10-09 completed current-build two-hour emulator proof

Run04311a62-6bc9-47ad-b418-660692d16144 completed03:04:21UTC, receipt passed,
8terse instrumentation tests OK in7343.946s; requested/completed steady7200s,
configured800msRTT/100msjitter. 71407matched samples: median47.5ms, p95=127.5ms,
max251.5ms; provisional150ms p95goal met. Measured RTTmedian752/max1174ms.
Strict run-bound clock analysis:71275steady samples, onegeneration, observed span
7199958ms/maxgap115ms; selected clockagemax30000ms/history64, offsetstep189ms,
authoritydrift195ms/nativeagemax117ms. All counted steady violation flags zero.
Observed gaps/flags are diagnostics, not visual alignment or authentication proof.

Preserved receipt/results/metrics/PC+phone logs/rotated journal/clock analysis and
post-completion application hashes inwork/passed-2gether-two-hour-04311a62.
Manifest verifier passes17files. EXE/APK/testAPK hashes match previous short run
built3ab4e18; hashes are post-completion checks, not startup attestation. No physical
phone, TorBox safe delivery, WAN, adapter independence or normal UI-exit acceptance.
Current run used terse instrumentation; raw assumption-skip audit remains absent
for this historical result. Do not retroactively upgrade its evidence boundaries.

After preserving evidence and confirming ownership marker absent, switched future
Test-SessionProof runs to raw instrumentation status packets plus standalone parser.
Reject adb failure, ignored/assumption/error packets, incomplete/mismatched counts;
store verification result in receipt. Avoid echoing complete raw transcript to console.
17pure Python tests and PowerShell syntax check pass. Native raw transcript validation
pending a new short emulator run; application binaries remain3ab4e18 unchanged.

## 2026-10-09 actual raw-instrumentation native acceptance

Short run2532b6d8-0d1b-4e96-882e-e119a24c718f passed with current application
artifacts3ab4e18 and runnerba6efb6. Raw AndroidX status parser verified8matched
start/success pairs with no ignored or assumption-skipped tests. Receipt embeds
instrumentation_verification; independent recheck of saved raw transcript passes.
This validates the new harness on actual emulator output, not just synthetic rows.
Historical two-hour04311a62 remains correctly labeled terse-output-only.

60requested/completed steady seconds at800msRTT/100msjitter;731matched samples,
median32ms/p95=122ms/max163ms, provisional150ms p95goalmet. Strict clock check:
594steady rows/span59917ms/maxgap105ms, all counted violations zero. Archived
receipt/raw result/metrics/PC+phone logs/rotated journal/clock report/boundary in
work/passed-raw-instrumentation-2532b6d8; manifest verification passes16files.
No physical-phone, TorBox safe delivery, WAN, adapter-independence or UI-exit gate
claimed. No credentials accessed or network settings changed. No duplicate run.

Next native correctness gap: deterministic close during native creation still lacks
an instrumentation regression; ordinary Activity/rotation coverage is insufficient
for that exact race. Preserve archives before any new app build or native run.

## 2026-10-09 deterministic native-creation teardown regression

No native ownership marker or active proof before edits; previous two-hour and
raw-status runs already archived. Added DEBUG-only constructor instrumentation
barriers to LibMpvAdapter, with no native handle/media/credential exposure. New
NativePlayerTest blocks the serial worker before real Mpv.create, closes twice on
Main while core is still absent, releases the barrier, and requires exactly one
real native creation and successful close plus final cleanup. Closed player must
never publish initialized=true. This targets the late-handle race directly; normal
rotation tests do not force it. Production construction uses null hooks.

Android25JVMchecks pass and both APKs build after fixing an unsupported local
lateinit-reflection expression in test cleanup. Session runner now expects9tests.
New native regression execution pending; no current-build two-hour claim for these
new Android artifacts. Windows application remains built3ab4e18. No provider or
credential access, network-setting changes or legacy modifications.

## 2026-10-09 native late-handle regression accepted

627aa081-2020-43c3-96e8-a1c8418f365c passed9actual native tests, raw-status parser
confirms no skips. The new creation barrier test executed and required one real
native create/close after double-close on Main during blocked creation. Android
artifacts builtcd637ac; Windows stays3ab4e18. 60steady seconds,800msRTT/100msjitter,
720matched samples, median40.5ms/p95=115.5ms/max214.4275ms. Strict clock594rows,
all counted violations zero. work/passed-native-late-handle-627aa081 archives
receipt/raw result/metrics/logs/journal/clock/hashes/boundary;17manifestfiles verify.
Physical-phone/TorBox/WAN/adapter-independence/normal UI-exit gates remain open.

## 2026-10-09 documented download-redirect option inspected

Added separate experimental-download-redirect feature and PC-only diagnostic for
the official requestdl redirect=true option. It sends the saved PC credential only
to the fixed API origin with automatic redirects disabled, classifies Location
through the existing URL/account-key guard, never follows Location, reads no media
body, returns no URL and creates no StreamLease. Neither application enables it.
Ten resolver tests pass, including missing/duplicate/unsafe/relative/account-bearing
Location cases; diagnostic example compiles. Default app resolver is unchanged.

One bounded live investigation on the first available existing video returned
HTTP307 with RejectedLocation. The returned location was not followed, exported,
printed or delivered. This rules out accepting that redirect as a safe capability;
it does not identify which rejection rule fired or establish a safe final URL.
No account mutation, Android delivery, network-setting or legacy changes. Do not
repeat this identical probe absent a new provider lead. No TorBox fix claimed.

## 2026-10-09 Android idle session polling budget

With no native owner active and627aa081 safely archived, inspected core behavior
around the mobile battery requirement. SessionController woke Main every10ms even
when disconnected or without media, where follow() has no playback to schedule.
Use250ms cadence while disconnected and100ms while connected without a snapshot
or media. Preserve10ms whenever a connected session has media, including prepare,
future commits, pauses and drift handling. Socket callbacks and immediate loss
handling remain event-driven; the change only avoids unnecessary idle follow ticks.
Clock polling remains500ms and controls still require current authoritative state.
No measured battery-savings claim; actual phone power measurements remain pending.

Android25JVMchecks and both APK builds pass. Native validation pending a short9-test
run for the new Android artifact; historical two-hour results do not cover it.
No new TorBox probes, credential access, network-setting or legacy changes.

## 2026-10-09 idle-polling native acceptance and further source audit

16594a45-0124-4012-b776-640580769e51 passed9raw-verified native tests on Android
78ee60f/Windows3ab4e18, including native creation teardown and session reconnect.
60steady seconds at800msRTT/100msjitter:737matched observations, median41.5ms,
p95=117.5ms/max151.74ms; provisional150ms p95goalmet. Strict clock594steady rows,
all counted violations zero. Archived inwork/passed-idle-polling-16594a45;
17manifestfiles verify. No phone power measurement or physical release acceptance.
Earlier two-hour run remains bound to its earlier Android artifacts.

Read-only audit of reducer buffering/pause/reconnect and tunnel retry lifecycle did
not establish a new defect; no speculative changes or duplicate native run made.
Public official TorBox homepage identifies its frontend asset index-a54eef60.js,
but one bounded unauthenticated fetch returnedHTTP403. No bundle saved, source
copied or authenticated browser/account session accessed. Do not infer any new
capability or workaround from this; no provider media probe or credential access.
Safe direct original-file delivery remains unresolved; preserve all current guards.

## 2026-10-09 delayed heartbeat RTT regression

Source audit found the server replaced outstanding_ping every2s, although reducer
RTT observations support5s. A real local WebSocket regression delayed its first
pong2200ms: it failed before the fix because the next heartbeat replaced the nonce.
Retain an outstanding nonce until matched or older than5000ms; only then issue a
new ping. Existing matching/sequence safeguards and six-second receive silence
limit stay intact. This restores measured RTT under high latency, supporting the
existing dynamic commit lead instead of silently losing delayed observations.

Full server suite result to be recorded after completion. Application binaries do
not yet contain this server source change; Windows rebuild and native short check
required. No provider request, private credential access, network setting or legacy
change. Existing current-build native archive16594a45 remains untouched.

Delayed-pong fix validation:50server unit checks pass (8intentional fixture/optional
checks ignored), all13transport checks pass including2200ms delayed-pong regression.
Windows native application rebuilt successfully from35b1e04. Android artifacts
remain78ee60f. A fresh800msRTT/100msjitter short native run follows to validate
integration; no new two-hour or physical-phone result is claimed.

## 2026-10-09 delayed RTT native acceptance and current-build long validation

07e21d2d-3759-4705-ab5e-e167302ea640 passed9raw-verified native tests on Windows
35b1e04/Android78ee60f. 60steady seconds at800msRTT/100msjitter;728matches,
median22.5ms/p95=78.5ms/max110.4475ms, provisional150ms p95goalmet. Strict clock
594steady rows, selected clockagemax23407ms/offsetstep38ms/authoritydrift106ms;
all counted violations zero. Archived work/passed-delayed-rtt-07e21d2d with
17verified manifestfiles. Measurements describe this run, not guaranteed improvement
or physical presentation timing. No phone/TorBox/WAN/normal UI-exit acceptance.

A new7200second run is justified by actual application changes since04311a62:
Windows heartbeat RTT lifetime, Android native teardown instrumentation and idle
session polling. It is not a duplicate continuation-triggered test. The short9-test
run passed before launching it; preserve its archives. Use original800msRTT/100ms
jitter to compare sustained current-build behavior. Running resources are owned by
its marker/receipt. Do not rebuild/reinstall/restart or claim a result until complete.

## 2026-10-09 current-build two-hour native acceptance

Run89db0028-a195-42b9-aec0-99ad9a216801 completed03:49:12–05:52:21UTC,
with7200requested/completed uninterrupted seconds at800msRTT/100msjitter.
All9selected native tests have matching raw start/success packets; no ignored or
assumption-skipped tests. Windows35b1e04 and Android78ee60f artifact hashes match
those in the preceding07e21d2d short-run archive. These are post-completion hashes,
not startup attestation. Runner cleanup completed; no active ownership marker.

71377matched sampled positions: median50.5ms, p95=135ms, max281.5ms. The provisional
150ms p95goal passed. Measured clock RTT median746ms/max1119ms. Strict clock
analysis verified run identity across71244steady rows in one generation, observed
span7199919ms/maxgap128ms. Selected clock age max30000ms/history64, consecutive
selected-offset step max228ms, authority drift max188ms/native age121ms. All counted
clock-age/history/connection/buffering/seeking/preparing/playing violations zero.
Observed spans and sampled position differences do not establish screen alignment.

Preserved work/passed-current-two-hour-89db0028, including raw instrumentation,
receipt, metrics, native/phone/journal logs, clock analysis, artifact hashes and
explicit boundaries.17manifest files verified; checksums are not authentication.
No physical-phone, TorBox safe-delivery, WAN or normal UI-exit acceptance. The
primary next blockers remain the provider capability contract and physical phone
release gate; do not weaken URL guards or migrate extra features. No credentials,
network settings or legacy projects were accessed or changed for this acceptance.

## 2026-10-09 exact native test identity verification

The raw instrumentation verifier previously required9unique successful tests but
could accept a different set with the same count. The runner now supplies the
reviewable scripts/native-proof-tests.json manifest of all nine intended class /
method pairs. Verification rejects unexpected methods/classes and malformed,
incomplete or duplicate expected identities. Reports expose only a boolean,
never raw test output or private status fields. Count-only verification remains
available for historical generic callers and reports identity verification false.

Seven parser tests pass, including same-count substitution rejection. Read-only
verification of archived89db0028 raw output against the new manifest confirms all
nine intended identities. This is additional analysis of existing evidence, not a
new native run; its saved receipt/manifest were not modified. The proof runner
passes PowerShell parsing, and the Python helper suite passes19tests. No APK/EXE
rebuild, provider request, credential access or network change was required.

## 2026-10-09 exclusive native runner ownership before builds

The running PID marker was checked before builds but created only after builds,
leaving a race where two simultaneous invocations could both pass preflight and
rebuild/install shared resources. Test-SessionProof.ps1 now acquires an exclusive
OS-held file handle before inspecting its marker or building. An outer finally
releases ownership after preflight/build errors and all native cleanup. A stale
empty lock file is harmless; ownership is the live handle, not file existence.
The existing PID/start-time check still protects older runners without the lock.

A separate-process contender is rejected while ownership is held. Reacquisition
works after normal close and synthetic build failure; the runner passes PowerShell
parsing. No native run was launched, no app artifacts rebuilt/installed, and no
completed evidence modified. This protects future automation/manual overlaps; it
does not coordinate unrelated build scripts or prove any new playback behavior.

Fixture preparation uses the same exclusive ownership handle before media/asset
writes, downloads or DLL extraction, and retains a PID/start-time fence for older
live runners. Previously Prepare-PlayerProof.ps1 had no running-proof guard and
could overwrite owned media/DLL resources. With the lock held, an actual invocation
now rejects at ownership acquisition before those operations. PowerShell parsing
passes; no fixture generation, download or extraction was executed. This covers
our preparation/session runner pair, not arbitrary external/manual file writes.

## 2026-10-09 HTTPS runner ownership and raw status boundary

Test-HttpsProof.ps1 now holds the same exclusive handle across its native Windows
proof, Android build/install and instrumentation, retaining the older PID/start
fence. Actual invocation while the lock is held rejects before these operations;
PowerShell parsing passes. No HTTPS playback or native build/install was launched.

Future HTTPS instrumentation uses raw status packets, checks adb exit immediately,
and requires the two exact NativeHttpsTest methods in https-proof-tests.json via
the shared verifier. It saves a sanitized verification report and no longer echoes
raw status to the console. Historical terse HTTPS evidence is not upgraded by this
source change; fresh runtime acceptance of this runner remains pending. This is
harness-only work and does not change current app artifacts or TorBox URL guards.

## 2026-10-09 fresh raw HTTPS acceptance

The updated HTTPS runner was exercised with existing Android78ee60f artifacts
(SkipBuild), after verifying no live native session owner. Its Windows native
proof compiled from current9cf9e53 source and passed TLS-verified HTTPS load,
play/pause/exact seek and rapid source replacement. Both intended Android
NativeHttpsTest methods passed raw start/success/identity checks; no skipped tests.
The APK/test APK were reinstalled without rebuilding; no long soak was launched.

Preserved work/passed-https-raw-20261009-0648 with raw result, sanitized verification,
explicit boundary and post-completion EXE/DLL/APK hashes; four checksum files
verified. This public pinned fixture acceptance is not TorBox capability proof,
public Cloudflare control testing or physical-phone acceptance. The89db0028
session receipt and its completed archive remain unchanged. Exclusive ownership
released after the HTTPS runner completed. No credentials or network settings used.

## 2026-10-09 HTTPS run receipts and failure boundary

HTTPS proof now writes a unique running/passed/failed receipt, with fixed stage,
timestamps and sanitized raw-verifier result. A failed Windows stage can no longer
be mistaken for an older successful Android report. Preflight ownership rejection
preserves the previous receipt. Receipt persistence occurs before releasing the
exclusive handle, including error paths. No private exception details enter it.

An injected failing cargo function produced statusfailed/windows_native with a
completed timestamp and no instrumentation verification, without executing native
playback. Then a real SkipBuild acceptance87f8bd78-e70e-475a-b06e-8a1c87ca35dd
passed both Windows native checks and both exact raw-verified Android tests,
completed08:01:45UTC. Archive work/passed-https-receipt-87f8bd78 has five verified
manifest files. EXE/DLL/APK hashes still match the preceding HTTPS archive, checked
after completion. Receipt association is not cryptographic binding of raw output.

Current two-hour session evidence89db0028 remains unchanged. This narrowly
validates the new success/failure receipt paths; it is not a new two-hour proof,
TorBox safe capability, physical-phone or Cloudflare WAN control acceptance.

## 2026-10-09 user reporting preference and precise TorBox redirect diagnosis

Persisted brief patch notes plus separate planning percentages in AGENTS.md and
PROGRESS.md: playback milestone approximately60%, complete polished beta30%.
Weights/rating assumptions are explicit and subjective; no release gate is implied
and diagnostics/docs alone do not raise estimates.

Experimental PC redirect diagnostics now classify fixed rejection causes without
retaining or printing URLs/error strings in reports.11resolver checks pass,
including opaque header/nested encoding and account-key/URL redaction. A single
newly instrumented probe resolved the prior307cause ambiguity: HTTP307,
RejectedLocation(AccountCredential). It was not followed/exported or delivered.
Existing validator unchanged; no app feature enabled, native app rebuilt, network
setting changed or legacy file modified. Native two-hour evidence remains intact.

Official public API/support research did not establish restricted file-scoped
credentials. The device-authorization/token-refresh API is not such a contract.
Safe TorBox delivery remains unresolved; don't repeat the same classified probe
or reinterpret account authorization as a media capability.

## 2026-10-09 source-selection transactions and concurrent device resolution

Added PC-side MediaSelection in source-resolver. Each selection/refresh revokes
previous undelivered leases and cancels the previous fetch future. Queued stale
fetches are rejected before polling provider work. Both validated leases must
match the selected complete MediaIdentity before either becomes available.
Opaque receipts are bound to their owning selection instance and sequence; each
role can take only its own lease once. Invalid choices revoke old delivery too.
Cancellation, resolver failure, mismatched identity, explicit shutdown and owner
drop cannot republish an obsolete/partial pair. Status/errors contain fixed enums;
the coordinator, tickets and receipts do not serialize or debug-print URLs.

Production device_streams now concurrently requests PC and phone leases, preserving
separate requests/IP bindings and existing account-key/URL guards. Either failure
cancels the other future and discards any partial result. This avoids serially
waiting two provider timeout periods. Cancellation drops client futures; it does
not claim to undo requests already received by TorBox or revoke issued CDN links.

This is resolver infrastructure, not connected Windows/Android source delivery.
Already-taken leases still require the controller's epoch/media fence immediately
before native load; the coordinator cannot revoke a URL already held by a player.
No expiry metadata is invented. Native integration, refresh/reconnect policy,
authenticated phone capability transport and safe provider capability remain open.
Synthetic race/role/failure/cancellation checks exercise the production coordinator
and concurrent collection helper without credentials or live TorBox requests.
Validation:26all-feature resolver unit tests passed,16default-feature unit tests
passed (overlapping suites, not42distinct tests), and the public API documentation
example compiled. All experimental resolver examples also compile; git whitespace
checks pass. The configured toolchain does not include rustfmt; no toolchain was
installed or changed to add it. Eight new checks cover the transaction/collection
behavior. Existing URL credential rejection checks remain passing.
No native rebuild/install/soak or WAN test was launched; existing app artifacts and
two-hour evidence remain unchanged. Playback60%/full beta30% planning estimates
remain unchanged until the delivery path is integrated and accepted.

## 2026-10-09 final handoff fence and source unload

Closed the gap between taking a resolved lease and handing it to a consumer.
MediaSelection.take now returns a SelectedLease wrapper without a URL getter,
Debug or Serialize. Its consuming deliver callback checks owner/sequence/ready
state while holding the selection lock through the synchronous handoff. Taken
but queued leases reject after source replacement, same-file refresh, invalid
choice, shutdown or owner drop, without invoking the consumer callback.

Added clear() for unloading a source: it cancels pending resolution, discards
unclaimed links and invalidates taken undelivered wrappers without permanently
closing the authority. A subsequent valid selection works. Sequence exhaustion
fails closed and discards leases rather than preserving old delivery.

Callbacks must be short, synchronous and non-reentrant. They must not return a
future or save a URL for a later unfenced operation. Native queues still need
session epoch/media checks when executing commands; copying/delivering a URL
cannot revoke it at a remote player. The final handoff check protects the local
handoff moment, not ongoing playback or an already delivered CDN capability.

Validation:30all-feature resolver tests pass (four new checks), the changed public
API documentation example compiles, and all experimental examples compile.
Existing role, identity, cancellation and credential-URL guards remain passing.
No credentials accessed, live provider requests, native app rebuild/install/soak,
phone/WAN test, network changes or legacy edits. Apps remain fixture-only; safe
TorBox capability and authenticated delivery/native integration are still open.
Planning estimates remain playback60% and full polished beta30%.

## 2026-10-09 native HTTPS input guards and explicit Windows TLS verification

Windows prepare_url now parses/bounds URLs and rejects malformed/empty authority,
userinfo (including an empty @ delimiter), fragments, control/whitespace/backslash
input and the case-normalized TorBox API host. Errors are fixed messages; URL
parser errors and NUL-bearing CString input cannot escape this entry point.
Android loadHttps uses a JVM-testable MediaUrl guard with the same tested cases,
including bounded UTF-8 bytes. URI parsing exceptions containing the original
input are discarded, with a fixed IllegalArgumentException and no private cause.
These are native boundary checks, not account-key detectors; the PC resolver's
existing key/encoding guard remains mandatory before distributing TorBox URLs.

Windows native initialization explicitly sets tls-verify=yes. The HTTPS example
now verifies that adapter default instead of supplying its own TLS override.
Android's existing system CA installation/tls-verify=yes remains intact.

Validation: five player-adapter tests pass (including two new URL checks and three
actual native command-failure checks);27Android JVM tests pass without skips.
Full Windows custom-protocol build and both Android APK builds succeed. Reused
the existing Together2Test AVD hidden; no new emulator or network adapter created.
First HTTPS attempt failed at Android installation: boot-complete was reported
before the package service became available. Its failed/android_setup receipt is
preserved in work/failed-https-media-url-*. No Android runtime pass was inferred.
After service check package reported found, a SkipBuild retry82b2a956-ed17-4b40-
a9c7-a30ec9db98ca passed Windows native load/play/pause/exact seek and rapid source
replacement, plus both intended Android methods with raw identities/no skips.
Completed12:57:15UTC. Archive work/passed-native-url-82b2a956 has six verified
manifest files: receipt, raw instrumentation, sanitized verification, boundary,
post-completion artifact hashes and source hashes. Hashes are not startup or
cryptographic run attestation. All native build/proof operations used the exclusive
owner lock; no running soak marker was present. No duplicate long test launched.

New artifacts do not inherit the prior89db0028two-hour acceptance. Physical phone,
safe TorBox delivery, public cloudflared control, normal UI-exit and a two-hour run
on these rebuilt artifacts remain unvalidated. Apps remain fixture-only at the
session-controller boundary. No account credentials accessed/exported, system
network settings changed or legacy apps modified. Planning playback60%/beta30%.

## 2026-10-09 Android queued-source guard and current short-session timing miss

Android loadSource now checks its source revision on the serialized worker before
issuing native loadfile. A newer request in the same command epoch skips the older
queued load. Existing command-epoch invalidation remains in force. Added a debug-
only worker barrier and a revision-only before-load observation to the existing
NativeLifecycleProof; no handle, path or media capability leaves this hook.
The deterministic real-native test blocks the worker, queues two sources without
invalidateCommands, releases it, and requires exactly the latest revision to reach
the native boundary and be accepted. Cleanup releases/closes its real native core.
This fences queued work, not a native command already executing before replacement.

Android27unit tests and both APK builds pass. Test-SessionProof now requires ten
exact native methods via the updated manifest. Windows e60a5ba binary remained
unchanged. Short native run e3b746b1-4c30-4bbe-b9b0-dda2307cf088, started13:06:22,
completed13:09:49UTC, raw-verified all ten methods without skips and completed the
requested60second uninterrupted segment at800msRTT/100msjitter. Runner PID13148
and its saved start time matched while live; owned resources were left alone.
This fresh run was justified by changed native sources, not a duplicate soak.

Timing result remains a miss:731matched all-playing samples, median80.5ms,
p95=154ms/max200.995ms against provisional150ms. Added an explicit SteadyOnly
diagnostic scope to Analyze-SessionProof without changing its default comparison
or the original metrics/receipt.595matched explicit-steady samples gave median
69.5ms/p95=156ms, also a miss. No filtering or rerun was used to claim a timing pass.
Strict clock report:595steady rows,59982msobserved span/max gap107ms, selected
offset step max68.5ms, authority drift148ms, native sample age102ms, zero counted
clock/history/disconnect/buffering/seeking/preparing/playing violations. These
figures do not explain away the cross-device timing miss or establish frame/audio
alignment. Cause remains unestablished; no sync thresholds were relaxed or tuned.

Future runner receipts explicitly include provisional_timing_target_met alongside
native assertion status and physical_release_gate_satisfied=false. This additive
receipt mapping passes PowerShell parsing but has not been exercised in a new
native run; existing e3b746b1receipt remains unchanged. Synthetic analysis checks
verify default versus steady scopes, excluded cross-run identity rejection and
insufficient steady sample rejection. Scope selection cannot hide stale rows.

Archive work/current-short-source-fence-e3b746b1 preserves receipt/raw statuses,
both timing reports, strict clock report, PC/phone logs, production journals, the
exact native manifest and post-completion artifact/source hashes;20manifest files
verified. Exact ten-method raw verification also passes from the saved manifest.
Checksums establish integrity against that manifest, not authenticated startup
attestation. Prior HTTPS82b2a956 evidence records its older Android APK; it is not
retroactively reassigned to this additional source-fence build. No current2h,
physical-phone, TorBox-safe-delivery, cloudflared WAN or normal UI-exit acceptance.
No system network changes, TorBox credentials or legacy files used. Estimates
remain playback60% and full polished beta30%; timing miss remains open.

## 2026-10-09 Windows position sampling and directional timing diagnostics

Offline inspection found that Windows assigned the sample timestamp before
source/duration/position reads, then recorded a rate potentially changed by a
controller correction after the position read. Windows now timestamps immediately
after position_ms, matching Android's post-read convention, and saves the rate
that applied before subsequent corrections for its proof/native journal sample.
The UI continues reporting the current applied rate. Numeric position_read_ms
records just the native position getter interval; it is not total frame/presentation
latency, timestamp uncertainty or an explanation of the earlier e3b746b1 miss.

Analyze-SessionProof reports optional matched PC read durations without inventing
latency for historical logs. Invalid present values are rejected even on excluded
PC rows. Signed median/ahead/behind/equal counts use the same time/rate-interpolated
pairs as absolute drift, with the clock-offset limitation stated explicitly.
Default scope, interpolation,150ms target and absolute acceptance remain unchanged.
Regression checks cover historical missing fields, positive/negative direction,
direction opposite to the raw position difference after rate interpolation,
selected-scope latency populations, invalid excluded values, cross-run identity
and insufficient samples. Replaying e3b746b1 still gives731matches/p95=154ms/false.

Windows custom-protocol build passes. One changed-native-code short check
fdc4bccb-c0fd-4030-90d3-7d597821343c started13:22:54UTC/completed13:26:29UTC.
It reused emulator-5580 and the unchanged ab1dd4d Android APKs; ten exact native
methods raw-verified without skips,60requested uninterrupted seconds completed,
800msRTT/100msjitter. Runner PID3256 and saved process start time matched while
live; the exclusive native owner lock covered build/test, and owned resources
were left alone until cleanup completed. No duplicate long soak was launched.

The fresh timing result is worse and remains a failure of the provisional target:
735all-playing matches, median89.5ms/p95=269ms/max325ms.594explicit-steady
matches give median86.5ms/p95=286ms;585behind/9ahead, signed median-86.5ms.
All matched PC property-read durations have p95=0ms/max1ms. These reads do not
explain the large gap in this run. Strict clock analysis has zero counted
violations, observed steady span59938ms/max gap107ms, selected-offset step
max164.5ms, authority drift max263ms, native age max108ms. Clock uncertainty,
controller response and native rendering still need investigation; no causal
claim, threshold tuning, filtering or rerun was used to turn the miss into a pass.
The additive receipt timing flag previously only parse-tested is now exercised:
status=passed describes native assertions; provisional_timing_target_met=false
and physical_release_gate_satisfied=false preserve the separate boundaries.

Archive work/current-position-sampling-fdc4bccb preserves the original receipt,
raw statuses, both original and separate signed/steady reports, strict clock report,
PC/phone logs, eight native journal files, exact manifest, boundary and post-
completion artifact/source hashes;21checksum files and all ten raw method identities
verified. Checksums are not authentication/startup attestation. The timestamp/rate
fix is validated by compilation and this short native execution, not proven to
improve sync by comparison between variable-delay runs. Physical phone, safe
TorBox delivery, public cloudflared control, normal UI exit and a two-hour run on
the rebuilt Windows artifact remain open. No secrets accessed or exported,
network settings changed, or legacy apps modified. Estimates stay60%/30%.

## 2026-10-09 Saved clock-transition context

Investigated the latest timing miss offline rather than launching another playback
run. The strict clock analyzer now preserves a bounded numeric context for the
largest signed offset step within a generation, plus transition count. Context
includes before/after authority drift, client gap, excluded non-steady row count,
playback availability, selected clock age and native sample age. Existing maxima,
violation counts, identity verification and receipt-duration requirements remain
unchanged. Generation changes are excluded from offset-transition counts; no
transition produces null context. Derived non-finite values fail with a fixed
error. Only recognized numeric fields/availability flags enter the report.

Twelve Python checks pass, including positive/negative steps, excluded phases,
playback unavailability, generation boundaries, unchanged offsets, arithmetic
overflow, input privacy and exclusive-output preservation. Saved e3b746b1 and
fdc4bccb reports were regenerated to new paths and every original report field
compared equal. Existing archives and receipts were left unchanged.

fdc4bccb has three within-generation offset changes. Its largest+164.5ms step
coincides with authority drift-21→-189ms (change-168ms) over100ms. Selected clock
age1831→8ms and native age37→34ms; both observations playing/available, zero
intervening excluded rows. Drift change plus offset step=-3.5ms is descriptive
arithmetic only, not a replacement drift metric or acceptance result. This supports
a clock-estimate contribution to measured drift, not true clock error, proof of
causation for the full p95 miss, or actual presentation alignment. e3b746b1 has
five changes, largest+68.5ms with drift-7→-78ms over101ms and selected clock
age29950→9864ms. The newer run switched to a fresh estimate before expiry; the
older run's largest transition was near the30s freshness boundary. Neither case
justifies tuning synchronization thresholds from these two traces alone.

Separate reports/boundary are preserved in work/clock-transition-context-
aa39a21d-697d-40a2-9abe-0a67a0ee6bf1;three checksum files verified. No app/native
source changes, builds, installation, duplicate tests, credentials, network settings
or legacy references touched. Current artifacts still require physical-phone,
safe TorBox, WAN, normal UI-exit and two-hour validation. Estimates remain60%/30%.

## 2026-10-10 Resumed development and finite calibration guard

User resumed development after the explicit pause. Re-enabled the same existing
continuation automation; no new automation or chat created. Checkout was clean at
54e6890 and no native ownership marker was present. Resume did not authorize any
network adapter/settings changes. No WAN test was started.

Both Rust and Android clock estimators checked finite input timestamps but could
still publish a non-finite derived offset when two same-sign finite differences
overflowed their sum. Added a finite-offset check before adding to calibration
history. Rust returns a fixed invalid-clock-offset error; Android returns null,
preserving its existing outlier-handling path. Ordinary arithmetic, min-RTT
selection, freshness window, history capacity and playback thresholds are unchanged.
This fixes malformed numeric calibration handling, not the recorded164.5ms clock
step or the provisional cross-device timing miss.

New regressions exercise both positive and negative derived overflow with finite
inputs and zero derived RTT, requiring rejection and preservation of the prior
500ms-offset/40ms-RTT sample and one-entry history. All28Rust sync-core tests and
all28Android JVM tests pass, zero failures/skips. Android production Kotlin and
unit-test compilation passed. No Windows app/APK build, installation, emulator
interaction, native run or duplicate soak performed; last native receipt remains
fdc4bccb and applies to its older artifacts, not these estimator source changes.
Toolchain deprecation/SDK-schema warnings remain non-failing and were not addressed
by unsolicited toolchain updates. Physical-phone, TorBox, WAN, normal UI-exit and
current-build two-hour gates remain open. No credentials or legacy files accessed,
no network settings changed. Planning estimates remain playback60%/full beta30%.

## 2026-10-10 Sustained clock-calibration and safe-source development

User requested 30–60 minute connected development sessions. Started this session
03:52:32UTC; updated the same existing continuation automation and AGENTS.md to
retain that preference. No new automation/chat, network settings or legacy changes.
At startup there was no active native owner or connected emulator. Reused the
existing Together2Test AVD/toolchains; no AVD recreation or adapter/VPN changes.

Implemented Together's fresh-window clock-offset interval intersection in Rust
and Kotlin. Each accepted exchange supplies conditional bounds; fresh bounds are
intersected and their midpoint used by the Android follower. Conflicting bounds
fall back explicitly to the original minimum-RTT point. The actual minimum-RTT
sample RTT/receive time remain available for existing scheduling and freshness
logic. Keep 64 bounded samples, 30-second freshness, accepted RTT range and all
playback/drift thresholds unchanged. Constant directional asymmetry remains an
unknown clock error; interval width is not measured accuracy or acceptance. This
is original Together logic, not upstream Syncplay/Synkplay reuse. See
[calibration design and limitations](CLOCK-CALIBRATION.md).

Added effective/raw clock diagnostics, optional bounds/support age/count and
explicit consistency/fallback proof metadata. Strict completed-run analysis checks
midpoint/fallback wiring, ordered finite bounds, fresh bounded support and raw
versus effective transitions. Bounded 60-second clock windows split on generation
changes; at most 128 are retained and omissions are explicit. Global metrics still
include all steady observations. Historical missing metadata remains unknown.
Every original report field is unchanged when replaying e3b746b1 and fdc4bccb.

The read-only ownership inventory compares PID and exact UTC start instant,
including Z/offset formatting, and checks the existing exclusive owner handle.
Malformed/oversized/ambiguous metadata and unavailable lock/process identity fail
closed. It creates no files, removes no marker, stops no process and never grants
permission for a duplicate native run. Regression checks pass for build ownership
before marker publication, reused PID, timezone formats, invalid inputs and
preservation. Existing runners retain their independent exclusive ownership lock.

The first new run b3899306 requested1800seconds at800msRTT/100msjitter. Runner
PID6860 and start instant matched while live; all nine recorded binary/DLL/APK/
media/compiled-clock-source inputs matched their before-run hashes after exit.
Independent source work and isolated build outputs did not replace those assets.
The run FAILED at04:15:32UTC in SessionPlaybackTest.kt:125. Nine methods succeeded;
the steady playback method failed a new support-age assertion after9515steady
rows/961785ms observed span. Completed requested duration remains null.

The final observation reports support age30001ms, selected sample age5692ms,
authority drift-50ms and native sample age70ms. Selection used a fresh timestamp,
but a subsequent clock read used for reporting age crossed the expiry boundary.
Corrected the controller to use one captured calibration instant for interval
selection and both reported ages; no bound was widened. Rust/Kotlin regressions
cover age30000 inclusion and age30001 eviction as well as independent generated
transit/server-work profiles, conflicting bounds, history reset/expiry, directional
jitter and fixed asymmetric-delay limitations. All34Rust sync-core tests and
all34Android JVM tests pass; Windows custom-protocol and both debug APK builds pass.

Failed evidence is preserved in work/failed-interval-age-b3899306:22verified
checksum files, including original failed receipt/raw status, both device logs,
eight native journals, exact ten-method manifest, before/after inputs and frozen
compiled clock sources. Checksums are not authentication/startup attestation.
Separate failed-run partial timing report has9515steady matches, median35.5ms,
p95=78.5ms/max141.5ms; signed median-17ms, native PC read p95=0ms/max1ms.
This is encouraging diagnostic data from an incomplete FAILED run, not native
acceptance,1800second completion or a causal comparison to older jitter traces.

Source-delivery review found raw syntax normalization and API DNS-root-dot aliases
could bypass PC/native URL guards. PC resolver now checks original raw authority,
whitespace/control/backslash syntax and canonical API-host aliases; redirect
classification shares its canonical host guard. Windows and Android guards reject
root-dot/encoded API aliases too. Fixed errors omit input. Ordinary CDN root dots,
encoded paths and distinct suffix domains remain accepted. Account-key guards,
TLS verification, separate device capabilities and control-only transport remain
unchanged. No TorBox account credential was accessed, sent or probed.

All32all-feature resolver tests plus its doctest pass; five player tests pass.
Isolated server validation reports50passed/eight intentionally ignored fixture
entry points, not58executed tests. No real tunnel was opened. The rebuilt native
HTTPS check passed Windows load/play/pause/exact seek/source replacement and
local API-alias rejection preserving the ready source. Both exact Android HTTPS
methods passed with no skips, including unchanged source receipts after rejection.
This uses pinned public AndroidX fixture media, not TorBox or WAN control.
The corrected build completed run71dca3e3-6e31-4d63-b424-37fe5b650dbc at
04:31:45UTC: all ten exact native methods raw-verified without skips and all600
requested uninterrupted seconds completed at800msRTT/100msjitter. The unchanged
150ms provisional p95 goal passed:6078all-eligible matches, median32ms/p95=96ms/
max156.5ms;5938explicit-steady matches, median32ms/p95=96.5ms/max135ms. Signed
steady median-23ms;1640ahead/4276behind/22equal. Clock and sampling error remain;
these are reported positions, not measured screen/audio synchronization.

Strict clock analysis includes5938steady rows across599977ms observed span,
maximum gap124ms and zero counted freshness/history/connection/playback-state
violations. Every steady row has a consistent interval; zero fallbacks, maximum
support62/age30000ms/width772ms. Maximum effective offset step35ms versus raw
minimum-RTT step164.5ms. Authority drift max87ms, native age max105ms. Ten bounded
clock windows retained without omissions. These descriptive results do not measure
true clock error, prove causation versus earlier runs, or waive other gates.

All twelve captured binary/DLL/APK/media/source inputs matched after completion.
work/current-interval-ten-minute-71dca3e3 preserves receipt, exact raw ten-method
status/manifest, default and explicit-steady metrics, strict clock context, both
device logs, eight native journals, before/after inputs and scope boundary:
20checksum files verified. Checksums are not authentication/startup attestation.
Ownership inventory after exit reports no runner marker and free lock; no duplicate
native run was launched. Native HTTPS run09ba67f2 is separately preserved in
work/passed-api-alias-https-09ba67f2 with six verified files. Partial failed-run
timing/clock context remains separately preserved in
work/failed-interval-age-partial-context-b3899306 with four verified files.

This connected session began03:52:32UTC and continued through implementation,
failure investigation, correction, rebuild, native HTTPS, ten-minute regression
and evidence review for more than40minutes. No physical-phone two-hour, safe
TorBox, WAN, current-build two-hour or normal UI-exit acceptance is claimed.
Planning estimates remain playback60%/full polished beta30%; those gates remain
open. Network settings, legacy references and encrypted credentials were untouched.

## 2026-10-10 Extended interval regression and pairing boundary

Continued the corrected clock candidate with a planned1800second regression,
not a duplicate of an active test. Exact PID15852/start04:34:10.8397684UTC and
exclusive lock matched while live. Run394ebb7c-4472-4f45-a2f2-4fe3936bb9f2
started04:34:11UTC and completed05:06:55UTC: all ten exact native methods passed
without skips and all1800uninterrupted seconds completed at800msRTT/100msjitter.
Compiled playback code remainsd4e7e65; subsequent pairing source changes were
developed independently and were not installed during this run.

17940all-eligible reported-position matches: median32ms/p95=83.5ms/max135.323ms;
17806explicit-steady matches: median32ms/p95=83.5ms/max134ms, signed median-14ms,
6686ahead/11038behind/82equal. The unchanged150ms provisional goal passed in both
scopes. Clock/sampling error remains; this is not measured screen/audio delay.
Strict analysis retains17806steady rows/1799936ms span/max117ms gap, zero counted
freshness/history/connection/playback-state violations, consistent interval metadata
on every steady row and zero fallbacks. Effective offset step max34.5ms versus
raw161ms; support max62/age30000ms/width743ms; native age max116ms/authority drift
max101ms. Thirty clock windows retained, no omissions. These are descriptive
emulator observations, not true clock error, a causal comparison or phone acceptance.

All twelve before/after binary/DLL/APK/media/clock-source inputs match. Archive
work/interval-thirty-minute-394ebb7c preserves20verified files: original receipt,
exact raw status/manifest, default/steady/clock metrics, both device logs, eight
native journals, inputs and artifact boundary. Checksums do not authenticate or
attest startup. The earlier b3899306failure remains failed and preserved separately.

Pairing review found raw URI exceptions could retain malformed pasted credentials
and invalid numeric ports could reach later HTTP handling. New PairingAddress.kt
uses fixed errors without causes, bounds input to8192characters/UTF-8 bytes,
rejects internal whitespace/control bytes, invalid/empty ports, userinfo, query,
fragment and non-root paths. HTTPS and the existing debug127.0.0.1:47820exception
remain; ordinary explicit HTTPS ports/IPv6 and trimmed input work. Header-unsafe
device credentials are rejected by Pairing construction before claim results can
be persisted/restored:32–256visible ASCII characters, fixed secret-free error.
No stored pairing is erased or replaced on invalid replacement input.

All38Android JVM tests pass without failures/errors/skips. Candidate debug app
and instrumentation APKs were built into separate work/pairing-validation-build
outputs while the soak's twelve inputs remained unchanged; installation waited
until its actual completion and evidence preservation. Native pairing run
06f94035-425d-41ba-9d07-9d0324393d9f completed05:37:37UTC on source113e61c:
all three exact PairingStoreTest methods raw-verified, including Keystore round
trip/tamper detection, explicit forget and byte-preserving rejection of invalid
addresses/tokens. Archive work/pairing-address-native-06f94035 has20verified files,
including candidate APKs, raw status, exact methods, unit XML and scope boundary.
These newly installed APKs have pairing-store acceptance only; no30minute session,
HTTPS, current-build two-hour or physical-phone result is assigned to them.
Standard app/build APK outputs still belong to the preceding playback candidate;
build the current sources or explicitly select the isolated candidate before a
future current-pairing session test. Never silently treat SkipBuild as current.

Runner entry now checks bounded metadata/exact UTC process identity before and
after acquiring its exclusive lock. Matching or unverifiable markers block entry
even when no OS lock is held. Both runner scripts use this shared guard rather
than the old two-second timestamp tolerance. Fixture checks verify preservation,
fixed errors, stale/reused PID distinction, simultaneous exclusion and ownership
release on failure. The live thirty-minute runner was not restarted or modified;
its SessionProof script was updated only after completion.

No TorBox credential access, private provider probe, WAN tunnel launch, network
configuration change or legacy reference modification occurred. Physical-phone
two-hour, safe TorBox, WAN, normal UI-exit and complete-beta gates remain open.
Planning estimates remain playback60%/full polished beta30%.

## 2026-10-10 Proof build ownership and PC credential transport boundary

Standalone Android builds now hold the native ownership lock through JVM tests,
application/instrumentation builds, source recheck and receipt publication.
Session/HTTPS SkipBuild paths reject missing or stale source/APK associations
before installation. The source inventory includes 46 current files and both
standard APK hashes; receipts are local associations, not authenticated compiler,
Windows or installed-package attestation. See PROOF-BUILDS.md.
Synthetic build-association and blocked-build-entry tests pass, including preserved
receipts and live/uncertain ownership rejection. Production missing-receipt
preflight rejected the previous unassociated APKs before host startup and preserved
the prior native receipt. The supported current Android build passed all38 JVM
methods. Native HTTPS run e92441db passed Windows and both exact Android methods;
its archive has six verified files. The subsequent current-pairing1800second
session result must be read separately; a running receipt is not acceptance.

Independent PC review found Host startup and decrypted vault validation accepted
header-unsafe identities that Android Pairing rejects. Shared PC validation now
requires32-256visible ASCII bytes, without trimming or changing stored identities.
Both PC and phone identities are checked; errors contain no supplied token.
DPAPI tests use only synthetic credentials: valid ciphertext with invalid token
contents is rejected while preserving exact ciphertext and leaving no replacement
vault. Existing concurrent creation/round-trip/tamper tests still pass.
Isolated server tests on source f960d66:52 library tests passed, eight intentionally
ignored fixture entry points; all13 socket transport tests passed. These builds
use work/header-validation-target and do not replace the running Windows binary.
No real credential was read, displayed or exported. These source changes are absent
from the concurrently running Windows playback executable compiled from d4e7e65.

The current-pairing regression subsequently completed06:20:37UTC:
553cd5b4-d673-47e8-bcc0-9a8a663062d3, all ten exact native methods executed
without skips, all1800requested uninterrupted seconds completed at800msRTT/
100msjitter. All17941eligible and17807explicit-steady position matches have
median31.5ms/p95=89.5ms/max137.5ms, steady signed median-1ms. The unchanged
150ms provisional goal passed. Strict clock analysis retains17807steady samples,
1799933ms span/max122ms gap, zero counted state/freshness/history violations,
consistent intervals in every steady row and no fallback. Effective step max43.5ms
versus raw150ms; support max62/age30000ms/width773ms; native age max107ms,
authority drift max89ms. All twelve captured inputs match before/after completion.
Archive work/current-pairing-thirty-minute-553cd5b4 has21verified checksum files,
including exact raw status/ten-method manifest, build association, both device logs,
eight journals, all/steady/clock reports, input boundaries and explicit scope.
Checksums do not authenticate the archive or attest startup. APK source113e61c and
Windows playback code d4e7e65 apply; f960d66 PC credential checks are absent from
that Windows executable. No physical-phone, safe TorBox, WAN or current two-hour
acceptance is inferred. This is the first30minute session result on the pairing
address/token candidate, rather than assigning the preceding APK's result to it.

After actual completion and archival, Android control recovery source f94c04a
now invalidates authority immediately when the local WebSocket rejects a command
send, matching clock-send failure handling. It pauses/invalidates native commands,
drops unsent intent and reconnects. Follow stops after a send can invalidate its
captured state, so an old snapshot cannot resume playback in that same tick.
A debug-only native test seam rejects one unsent pause and checks immediate control
loss, native pause and snapshot recovery with the PC's playing intent preserved.
It retries only while the existing command is awaiting acknowledgement; it never
queues the rejected pause for reconnect replay. The existing ten-method suite's
session method covers this case without replacing transport-drop/recovery cases.
Windows rebuilt on f960d66 and Android on f94c04a; all38JVM methods/builds passed.
The native outcome for this newer candidate must be read separately from553cd5b4.

The newer command-send candidate completed its separate native run
ace8e422-c303-4fc8-ae3f-ce2dab768144 at06:25:24UTC: all ten exact methods passed
without skips, including immediate local-send refusal, native pause and recovery
without replaying the unsent pause. Only60requested uninterrupted seconds completed.
All730eligible position matches: median37.5ms/p95=115.5ms/max135.5ms. Explicit steady:
594matches, median37.5ms/p95=114.5ms/max135.5ms, signed median-35.5ms. Both scopes
met the unchanged150ms provisional target. Strict clock context has594steady rows,
59962ms span/max106ms gap, zero counted freshness/state/history violations and
consistent intervals without fallback. All twelve recorded inputs match. Archive
work/command-send-native-ace8e422 has21verified files; latest APK source f94c04a,
Windows compiled code f960d66. No older30minute/two-hour result is assigned to it.

Post-exit archival analysis exposed one incomplete final PC JSON row after the
runner killed its owned host. The original successful runner metrics and exact raw
log are preserved. Analyze-SessionProof now offers explicit IgnoreUnterminatedPcTail
recovery: only malformed JSON in the final unterminated line can be omitted, with
ignored_unterminated_pc_tail=true in its report. Default parsing remains strict;
complete malformed records and valid mismatched-identity final records still reject.
Valid final records without newline are retained and checked. Synthetic regression
verifies statistics, unchanged raw bytes, complete-record and identity rejection,
valid final-record retention and truthful omission metadata. Both session-analysis
and drift-scope regression scripts pass. Source b368754 records this analysis fix;
it does not alter playback. The ace8e422 explicit-steady report uses this opt-in and
retains the incomplete original raw row in its archive.

Native HTTPS run b287f15e-d3c1-443c-956c-0bb3345d3b22 completed06:27:30UTC on the
latest f94c04a APKs. Windows verified HTTPS load/play/pause/exact seek, API-alias
rejection preserving active media and source replacement passed; both exact Android
HTTPS methods raw-verified without skips. Separate archive
work/command-send-https-b287f15e has six verified files. This uses pinned public
AndroidX fixture media, not TorBox delivery or WAN session control.

This connected implementation/validation pass began05:41UTC and continued through
06:28UTC (about47minutes). Final ownership inventory: absent runner/free lock.
No active soak was rebuilt, restarted, reinstalled or duplicated; all evidence was
saved before changing its owned resources. Network settings, read-only legacy apps
and real credentials were untouched. Physical-phone two-hour, current-candidate
extended stability, safe TorBox capabilities, WAN control and normal UI-exit gates
remain open. Planning estimates stay playback60%/complete polished beta30%.

User-requested pause and app shutdown (2026-10-10T06:47:06.0901063Z): continuation automation paused. Interrupted native run ba6f10cb-ef01-4e6c-ab15-2cdd9a0ad589 is explicitly cancelled, with its original running receipt preserved; no completed 1800-second result is claimed. Windows proof-build helpers are committed through 2245241; session-runner integration and actual Windows receipt publication remain unfinished. Windows player/test relay exited; Android proof apps and authorized emulator closed. No new implementation or tests started during shutdown. Planning estimates remain playback 60%, complete polished beta 30%; physical-phone gate remains open.

Resumed 2026-10-11: ownership absent/free and existing continuation reactivated.
Cancelled ba6f10cb partial evidence saved in work/cancelled-native-ba6f10cb with
13 verified files, no historical summary assigned to it and no phone log claimed.
Windows build association integrated in session runner; missing/stale preflight
rejected before startup and preserved the prior receipt. Actual Windows build and
frontend validation passed; 95 source inputs, 3 frontend inputs and executable/DLL
association recorded. Synthetic association and live/uncertain ownership tests pass.
New native validation remains pending; no physical release gate is inferred.

## 2026-10-11 complete-playback Alpha 1 scope and integration

User authorized private Windows/APK Alpha 1 test builds before the phone gate,
then clarified complete playback first and simple UI for testing with their
partner. This does not authorize a fixture-only release or remove the key boundary.

Baseline eb087eb1-d17d-404c-9eb8-b3e844b90a72 completed all ten exact native
methods and 1800 uninterrupted seconds at 800 ms RTT/100 ms jitter. All 12 inputs
matched before/after. Explicit steady scope: 17701 matches, median 39 ms,
p95 112.5 ms, maximum 181.5 ms; the unchanged provisional p95 goal passed.
Archive work/windows-associated-native-eb087eb1 contains 22 verified files,
including both actual build associations and raw logs. This baseline predates
the following integration. Neither it nor its clock analysis accepts TorBox,
WAN, a physical phone, visual/audio timing, or two-hour reliability.

Added private per-device source storage and an authenticated /media control-
metadata endpoint. URLs stay outside snapshots/journals. A final consuming grant
checks epoch, generation, identity and source revision before native handoff;
replacement, same-file reload, seek and shutdown revoke queued grants. Only the
paired phone may request its own metadata; PC native delivery stays in process.
Invalid media loads preserve the existing source. The exact tunnel allow-list now
includes /media metadata, while nested/media-byte routes remain denied.

Windows can list existing available TorBox videos and resolve independent PC/phone
links before committing a selection. Its native worker consumes the PC grant.
Existing encrypted development credentials are read in memory without copying
the vault into an artifact. Android requests bounded authenticated metadata with
redirects disabled, checks canonical identity and epoch/generation, and loads HTTPS
directly with native command/source fences. Failed requests retry after a bounded
delay and cannot fall back to the local fixture for a TorBox identity.
The portable Windows runtime root now follows the executable; release builds
reject a missing packaged DLL instead of substituting a checkout runtime.

Validation so far: Windows/frontend and Android builds passed; six Windows tests,
55 server library tests (eight intentional fixture entry points ignored), 13
socket tests and four targeted private-handoff tests passed. Android metadata run
709e5d9b-2ba1-4c82-9986-98b915250157 executed all four exact methods without skips,
using only synthetic loopback control responses. It is not actual TorBox or native
playback acceptance. Latest source changes require their own fresh session run.

Provider blocker remains: the previously observed download/redirect URL contains
the account key; createstream with scrobbling disabled previously returned 500.
Public OpenAPI rechecked today provides no new validated safe-capability contract.
No identical private probe was repeated, no key/URL was logged, and guards remain
enabled. Automatic lease renewal, installed PC credential setup, dedicated public
tunnel provisioning/WAN validation, corresponding native source packaging and
physical-phone acceptance still need completion before the requested usable
Alpha 1 handoff. Candidate compilation is not a claim that TorBox works end to end.
Planning estimates remain playback 60% / complete polished beta 30% until the
new connected candidate has been validated; no percentage is measured acceptance.

Fresh candidate run be634bce failed exact native verification: nine methods passed,
but SessionIdentityTest caught accepting a changed identity without a new generation.
Its failed receipt/raw logs/build associations are retained in 17 verified files at
work/failed-alpha-handoff-be634bce. No historical metrics or completed steady time
are assigned to that failed run. After ownership became absent/free, Android now
rejects same-generation identity changes and its existing regression also checks
that a legitimate new TorBox generation cannot reuse the loaded fixture or report
ready before a capability. The metadata client shares semantic identity comparison.
PC selection now verifies both bounded direct CDN ranges before publication, so a
redirect or incorrect file size cannot become a phone handoff. The resulting
Android build/JVM suite and four PC private-handoff tests pass; fresh connected
native validation is pending. These changes still do not resolve the provider URL.

Public Android v0.3.0 mpv/FFmpeg source assets and SOURCES.txt were downloaded into
work/alpha-source-material/android-v0.3.0. Hashes match the release asset digests
and published sources.sha256. Exact native BUILD-INFO/dependency sources and the
Windows corresponding-source material remain unfinished; this is preparation,
not a claim that redistribution packaging is complete. No runtime was replaced.

## 2026-10-12 private Alpha UX, updater and phone logs

Native run e3e915db passed all ten exact tests on source 279a697 with 60 seconds
steady playback at configured 800 ms RTT/100 ms jitter. The explicit steady p95
was 83.5 ms across 591 matched samples (maximum 119.5 ms). Its 22-file archive is
work/alpha-fixed-native-e3e915db, checksum verified. This remains synthetic emulator
evidence, not real TorBox, WAN or physical-phone/two-hour acceptance.

User expanded private Alpha scope: a simple Android updater, coherent overall
pair/select/watch/recover UX, and continuous local phone diagnostics with manual
Send logs to PC. Updates check private PC metadata, download HTTPS APKs directly,
validate size/hash/package/higher version/current signer set, and open Android's
installer. A real hosted higher-version installation is still unvalidated.
Phone logs sample every two seconds, rotate two 1 MiB private files, and send the
latest 128 typed samples only on demand. Server authentication and flush tests
pass; no media URLs, raw errors or credentials enter the schema. Log storage and
future-session lookup instructions are in docs/ANDROID-UPDATES-AND-LOGS.md and AGENTS.md.

Windows/frontend and Android candidate builds pass, as do 60 server library tests
(eight fixture entrypoints intentionally ignored) and the existing Android JVM
suite. Android run 9818a2ed executed all three update rejection/privacy methods
without skips. The connected native regression has been extended to upload actual
phone logs after playback recovery; fresh build/run validation is pending.
Planning estimate: playback milestone 65%, complete polished beta 30%. These are
scope estimates; the update/log additions do not resolve the safe-provider blocker.

Fresh logging-enabled native run 483d3a48-f709-45f4-8239-e1add623522f passed all ten
exact methods on app source 581a820, with 60 seconds steady at 800 ms configured
RTT/100 ms jitter. The selected session method uploaded 107 real structured phone
samples to the PC's rotating journal after recovery. Four fault samples and twelve
disconnected samples correspond to deliberate failure/reconnect scenarios; they
are diagnostic observations, not a claim that a normal session had no faults.
Explicit steady positional p95 was 107 ms across 592 matches, median 69 ms, maximum
155 ms. Clock analysis found zero selected steady-phase violations. These values
include clock/property-sampling error; the 150 ms target applies to p95, not every
sample. Current candidate has no two-hour, WAN or real-provider acceptance.

The final 21-file archive at work/ux-logs-native-483d3a48-verified is checksum
verified, includes exact instrumentation/build associations, both player traces,
eight retained journal files and the phone-summary report. Rotated journal files
also contain historical records; only session-0 belongs to this latest run.
The earlier partial archive is retained and is not the verified archive. An
archive-helper interruption exposed that the read-only inspector initially read
bare events instead of the PC journal's `record` envelope. The inspector now
handles both shapes; actual received samples are verified. An existing immutable
clock report was preserved, and a separate fresh v2 analysis was used to complete
the archive. No native test was repeated because of these archive-helper issues.

Update/privacy run ff8a2aba executed all three exact methods on the same Android
source without skips, archived at work/update-and-logs-ff8a2aba. The higher-version
installer success/pairing-preservation check still requires a prepared signed
release and hosting. Android logging has no automatic uploader; Send logs is the
only upload trigger. The paired control tunnel now allows only the two additional
exact metadata/log endpoints, with no APK/media byte proxy.

Official libmpvKt v0.3.0 tag was confirmed at b001dd97538e7d3e700df80029fd72be99ceba3f.
Its public wrapper/build-script source archive is preserved in
work/alpha-source-material with SHA-256 a34b8a65a4d6194ac0ecc033eb1bc0efbde3f6b4e401c996c882a8cc28c1c3ca.
This improves source-package preparation; native binary/build-info correspondence
and all remaining corresponding sources are not yet fully verified.

## 2026-10-12 development alongside user testing

Added explicit TorBox Refresh playback links: validate both direct ranges and
publish a new preparation generation at the current authoritative position,
preserving play/pause intent. Newer pause/seek/replacement/epoch/shutdown rejects
pending results. Ended movies require Replay. Refresh does not claim a faulted
participant has recovered; native recovery must still acknowledge it. Three new
server regressions cover both role grant revocation, rejected-operation state
preservation and participant fault retention. All 63 server library tests passed
(eight fixture entrypoints ignored); sync-core library check passed. Windows cargo
check and frontend check passed with zero frontend errors/warnings. Android main
and instrumentation Kotlin compile and JVM suite passed.

Muted future Windows proof instances and added a process-local muted Android
instrumentation runner, updating all four runner scripts. Native mute execution
is not yet verified on-device. Normal user playback audio is unchanged. Fixed
four previously mixed-encoding punctuation bytes in App.svelte so source is valid
UTF-8. No standard EXE/APK was rebuilt, no install/adb/emulator/native player was
touched while the user tests the offered builds. These changes are source-only
pending a fresh candidate/native validation; previous build receipts now describe
the offered older artifacts, not current source.

User prioritizes safe TorBox delivery and Stremio now; authorizes a conditional
private Alpha account-bearing playback-link fallback only if substantial secure
attempts cannot establish usable delivery. See AGENTS.md for exact limits. Public
OpenAPI was rechecked: createstream and getstreamdata still exist; response schema
does not establish file-scoped original-file delivery. Public documentation fetch
from Python returned403; no private probe or key access in this pass so far.
Planning estimates remain playback65% / complete-beta30%; private Alpha readiness
still requires useful movie delivery, remote setup and physical-device evidence.


## 2026-10-12 TorBox delivery and PC Stremio integration

Fresh secure-stream inspection returnedHTTP500 without scoped tokens. Following
substantial documented secure attempts and the user's conditional exception,
implemented an explicit private Alpha delivery feature. Strict default preserved;
provider CDN origins and exact token placement remain checked. A live bounded
PC proof received both1024-byte ranges for one canonical file. This is account-
bearing private delivery, not secure file scoping or physical-phone acceptance.

Implemented encrypted PC Stremio login, read-only library/configured addon access,
catalog/search and exact TorBox stream mapping. Windows UI and controller connect
catalog selection to authoritative media preparation. Later selections and newer
host commands fence pending results. Copied Torrentio/TorBox references can also
identify a movie/episode prepared by Stremio first; fresh inventory reads avoid
waiting on the normal cached list. No supplied wrapper is followed, no fileIdx is
assumed to be a TorBox file ID, and no automatic account creation/deletion occurs.
Android remains playback/sync/recovery/logging only for this PC-selection flow.

Live read-only account proof: empty saved library, fourHTTPS addons, one stream
provider,130catalog definitions,56movie titles and51stream choices. First catalog
movie had zero exact ready-account matches. Automatic cached-item preparation and
real copied-link/native handoff remain unfinished. Stremio localhost lead checked:
no11470/12470listener; no service started. See docs/STREMIO-PLAYBACK.md.

Validation:42resolver checks passed; Windows private-choice revision regression
passed; ordinary and private Windows cargo checks passed before copied-source
addition, private check and Svelte check passed again after that addition. No
standard EXE/APK rebuild, Android install, emulator/adb change or native playback.
Offered Windows EXE21,958,144bytes datedOct11 18:56:53UTC and APK224,837,969bytes
datedOct11 18:57:09UTC remain unchanged. Native ownership was free with no runner;
that check did not authorize a duplicate run. Phone logs remain at
C:/Users/astro/AppData/Local/app.together.v2.windows/diagnostics.
Planning estimates:playback68%, complete polished beta30%. Increase reflects the
new real private-delivery proof and selection integration; physical gate stays open.

## 2026-10-12 explicit cached Stremio preparation

Added provider-cached choices and an explicit Prepare in TorBox / Check preparation
flow on PC. Live read-only cache validation found 21 exact filename/size choices
among 51 Stremio descriptors for the selected catalog movie. No account writes,
wrapper follows or Android handoffs occurred in that proof.

Preparation rechecks cache and account inventory, reuses existing items, creates
only the chosen cached hash, and verifies the returned torrent hash and exact
clean file ID/name/size. Pending items do not change current playback. An uncertain
create is not automatically repeated during this PC process. Restart clears the
local attempt marker; check the account first after an uncertain result. No
account deletes, seeding changes, largest-file or file-index guesses are used.
45 resolver regressions and the Windows stale-choice check pass; private Windows
cargo check and Svelte check pass. Live creation/native acceptance remain pending.

Added an isolated candidate builder: separate frontend, Rust target and Gradle
outputs, source fingerprints and per-artifact checksums, private-policy feature
recording, matching source archive and notices. It preserves existing EXE/APK
hashes, takes the native ownership lock, and never installs/launches or publishes
standard proof receipts. Android version code becomes3/private-alpha0.1.2 with the
same application ID. Build execution and signing checks follow after this source
commit; no package success is claimed by this entry.
Planning estimates remain playback68% / full beta30% until candidate validation.

## 2026-10-12 episode selection and isolated candidate validation

PC Stremio series selection now opens verified explicit episode metadata and uses
the episode ID for configured stream requests. Returning to series titles clears
private stream choices; revision checks fence delayed metadata/navigation. Failed
library/addon refresh clears the cached client so manual reconnect can sign in
again. Unknown TorBox file scan status remains pending rather than failing the
preparation action; infected files remain rejected.

Live read-only series evidence:59 titles,99 episode identities,10 stream
descriptors for the first inspected episode and0 exact cached choices. This proves
metadata/stream plumbing, not that every episode is cached or playable. Movie
cache evidence remains21 exact choices from51 descriptors. 46resolver checks,
two Windows stale-selection/navigation checks and Svelte checks pass.

First isolated candidate built at work/private-alpha-candidates/
20261011-195725-24113414, sourcebcec487. Windows/Android compile and Android unit
checks passed. APK has versionCode3/0.1.2-private-alpha and the same signing
certificate as the offered APK (SHA256850876809cc2c4a9bf99936a2458a3c04c17c44fa8905e33132cb5d1e47b8f8a).
No installation or pairing-retention acceptance is implied. Builder now also
rejects untracked compiled sources before archiving the associated commit.

Separate PC-only native provider runb76d8778-eb3b-4022-b974-136083e32901 passed:
duration7,560,594ms;0->12,011ms progress; pause0ms drift; seek45,000ms exactly;
45,000->57,015ms resumed progress. Muted/headless with TLS verification, candidate
DLL checksum validation and real runner PID/start-time ownership. Both1024-byte
role ranges were fetched on PC; android_handofffalse/physical_gatefalse. Standard
paired receipts and offered binaries are preserved. Live Stremio creation,
Windows GUI/Android native handoff, WAN recovery and real two-hour gate remain
unverified. The proof does not attest later episode UI changes.

Planning estimates:playback70% / complete polished beta30%. Increase reflects
usable cached preparation, episode selection and isolated private test packaging;
the physical acceptance gate and final redesign/features remain open.

Final episode candidate built successfully at work/private-alpha-candidates/
20261011-201040-8a29063e, sourcef9c469534ae903cbd8cd5a4a9488d97abef6edaa.
16packaged file checksums verified; APK signature verified with matching previous
certificate, app.together.v2/versionCode3/0.1.2-private-alpha. Compiled Windows
ordinary-policy check and all8private Windows unit tests passed; session-server
unit/integration regressions passed. No apps launched/installed. Normal log-folder
inspection found one journal and zero received phone samples, so no new couple
test evidence is available. Native ownership returnedfree/absent. Old offered
EXE/APK retained their hashes. This candidate supersedes the earlier isolated
candidate for episode browsing; both folders remain available.

## 2026-10-12 current-file verification and source diagnostics

User will test tomorrow; continue backend work while preserving offered packages.
Stremio operations now capture revision and selected catalog/title under one lock.
TorBox re-reads the selected torrent before issuing either device link, requiring
the same hash, exact filename/size, unique file ID, availability and clean scan.
Ready-account and prepared-file metadata share the credential/control-text guard
before entering IPC or snapshots. This does not turn a CDN range into a content
hash or guarantee the provider cannot change after verification.

Added typed source diagnostic phases (current_file_and_links, direct_ranges,
publication), fixed outcomes and numeric selection/generation/timing context.
No URLs, names or credentials enter these events. The normal log inspector now
counts source failures/replaced selections alongside phone samples and rejects
unrecognized phase/outcome text. Journal and inspector privacy/order tests pass.

48resolver checks,8Windows checks,64session-server unit checks and13transport
checks pass;8pre-existing unit checks remain ignored. Ordinary/private Windows
compile checks pass. A fresh PC-only provider check after current-file verification
received both1024-byte ranges; no Android handoff/account writes or native playback.
Receipt:work/current-file-private-delivery-20261012.log. The ignored control
reconnect soak passed its2-second smoke (3cycles); no long-run result claimed yet.
Offered Alpha candidate f9c4695 remains unchanged and lacks these newer changes.
Planning stays70%playback/30%beta; physical gate remains open.

Follow-up selection fences now reject obsolete work before asking for links and
again before probing ranges. Final atomic publication checks remain in place.
The new Windows stale-selection/authoritative-command regression passes without
credentials or provider requests. Earlier120-second control validation and
candidate20261011-225451-5ef15e17 have incomplete logs and no live runner or build
receipt; do not count them as passes. A fresh completion attempt is separate.

Completed control-only validation:120seconds/132authenticated reconnect cycles,
pause/seek and stale-generation rejection on every cycle, clean listener/ticker
shutdown and Host release, zero journal I/O errors/dropped events. No native
players, emulator, phone, WAN or physical acceptance. Receipt:
work/control-reconnect-validation-completed-20261012.log. This bounded check is
not the20-minute default ignored soak or the physical two-hour gate.

Updated isolated candidate20261012-133128-8fdeffa9 built successfully from
d975dee05d678051422b99a4691e1b93e62b9272. All16package checksums verified;
APK app.together.v2/versionCode3/0.1.2-private-alpha has the existing matching
signature. Windows ordinary-policy check and log-inspector regression passed.
Standard artifacts preserved; no apps launched/installed or network settings
changed. Earlier offered candidate folders remain. Normal journal still has zero
received phone samples. Playback planning70%/full beta30% remains unchanged.

## 2026-10-12 Android recovery feedback and HTTP cleanup

Media preparation failures now appear on the phone playback screen. A new
authoritative generation clears the previous retry deadline and media notice;
successful media acquisition clears its notice, and an accepted command clears
the separate command-rejection notice. Native readiness/fault recovery remains
authoritative and is not implied by acquiring a link. Session close cancels and
closes its separate media HTTP client; phone-log HTTP executor also shuts down.
The media client rejects post-close requests with fixed text and no credentials.

Android versionCode4/0.1.3-private-alpha allows an update from earlier private
Alpha packages with the same signing identity. Candidate builder now records the
actual APK identity/version instead of putting a hard-coded version in README.
Instrumentation adds closed-client rejection and explicit client cleanup; these
checks are not yet executed on a device. Compilation/unit/package checks follow.

The ownership cleanup is shared by the independently created session, media,
updater and log clients. A JVM loopback regression stalls an HTTP response and
requires owner close to cancel the call, close the peer socket (not merely time
out), and terminate the executor. Final session-owner close also cancels its
WebSocket after requesting disconnect and releases endpoint/credential references.
Existing candidates remain unchanged while this final cleanup is validated.

Owned HTTP JVM regression passed in the isolated candidate build. Follow-up log
sampling now stops its timer while hidden (playback already pauses/disconnects),
records one background transition and resumes on foreground. This preserves
useful playback samples in the bounded manual-upload window. Cancellable record
and hidden/closed-owner sampler regressions were added. Final Android version
becomes5/0.1.4-private-alpha so it can update any preceding private candidate.
Instrumented device checks remain compiled-only; physical gate remains open.

Final candidate20261012-135012-d336360a built successfully from3b8b538.
41Android JVM checks passed (including stalled HTTP cleanup and both visibility
sampling regressions); instrumented APK source compiled but was not installed or
executed. All16package checksums verified. APK remainsapp.together.v2, version5/
0.1.4-private-alpha; signing certificate matches earlier private candidates.
Existing APK/EXE files preserved. No emulator/adb/app launch or network changes.
Native owner lock isfree/runnerabsent. Physical acceptance remains unverified.

User changed build cadence: source development/focused tests continue, but no new
EXE/APK packages until they test the latest version and give feedback (unless they
explicitly request a build). Alpha0.1.4 candidate20261012-135012-d336360a placed at
C:/Users/astro/Desktop/2gether Alpha 0.1.4. All16payload files copied and checksum-
verified; Windows payload flattened so2gether.exe is beside the APK, while its
native/work folders remain at the expected relative locations. Original package
instructions retained separately, START-HERE.txt added, desktop-delivery.json
records source mapping/hashes. Original candidates remain intact; no launch.

## 2026-10-12 source-only diagnostics and media boundary validation

The PC phone-log inspector now distinguishes raw uploaded copies from unique
samples, deduplicates overlapping uploads by elapsed time/UTC/generation, and
reports conflicting copies while retaining worst observed fault, buffering and
disconnection flags. It validates the complete typed schema and discards an
overlong physical line in full so its suffix cannot become a fabricated record.

Android sanitizes approved sample fields before disk storage. PhoneJournalStorage
serializes bounded journal writes, rotation and reads across recreated controller
owners with a shared disk mutex. Manual upload and the existing two-file retention
remain bounded. The source resolver also applies its 8192-byte URL limit after
canonical encoding, preventing Unicode expansion from exceeding the phone limit.

Validation completed: 49 resolver library tests, 43 Android JVM tests (including
two journal-storage regressions), and four phone-log inspector regressions passed.
Android instrumentation sources compiled only; they were not installed or run on
a device. Source-only Android receipt: work/source-validation/
20261012-141511-1b502b79/checks.json; log: checks.log in the same directory.
Resolver receipt: work/media-boundary-source-tests.log. Android source fingerprint
was stable across validation; the isolated output contained zero APK files.

These changes are not in the offered Desktop Alpha0.1.4. No new application EXE
or APK package was created, and no apps, emulator, adb or network settings were
changed. Normal PC diagnostics still contain zero received phone samples. Native
owner lock was free and runner absent; no native soak was launched. Compilation
and JVM/library checks do not satisfy the physical-phone two-hour release gate.
Planning estimates remain playback70% / full beta30%.

## 2026-10-12 long control test pacing failure and correction

Control run1e80a4d189e14a5daa018ce0f3794a98 failed after640.92seconds with HTTP429
on the next reconnect; it did not complete its configured1800seconds. The harness
slept900ms between cycles while the existing upgrade bucket refills one token per
second. That artificial sustained rate gradually exhausts its60-token burst
allowance. This is a test pacing failure, not evidence of native playback failure.
No physical acceptance is implied. Failed own-run receipt/log are retained.

The harness now waits1000ms between reconnect cycles, preserving server limits.
Two deterministic regressions demonstrate1800 one-second-spaced accepts versus
eventual burst exhaustion at900ms pacing. Session-server library suite:71 passed,
8 ignored; receipt work/control-pacing-tests.log. A replacement long run may start
only after the previous runner/owner have exited; inspect its own final receipt.
Desktop Alpha and planning estimates remain unchanged.

The user requested pausing after this iteration. Implementation and validation
are complete; preserve the offered Alpha and stop further development until the
user resumes it. Pause the existing recurring development automation as well.

## 2026-10-12 background resume and whole-batch diagnostic receipts

User resumed development while gaming: keep the screen undisturbed; automated
video playback is permitted only muted. Existing continuation automation resumed
with these restrictions and the deferred EXE/APK package preference preserved.

Phone diagnostic uploads now enter the PC journal as one bounded queue command
instead of individually queued rows plus a separate flush command. Queue pressure
rejects the entire upload before admission; unrelated playback diagnostic drops
cannot turn its own successful writes into a false failure. The writer receipt
requires all rows and userspace flush, with a two-second acknowledgement bound.
Filesystem failures/timeouts remain failures and can leave partial/repeated rows;
this is not a disk transaction or fsync guarantee. Playback producers stay
nonblocking. The phone endpoint records its retry ID only after success.

Five new regressions cover single-slot admission, unrelated producer drops,
full-queue rejection without partial admission, invalid/closed writers, ordered
flushed rows, missing acknowledgement and disk failure. Session-server library
checks:69 passed/8 ignored. Full source control/synchronization regression run:
116 passed/9 ignored across server, transport and sync suites. Receipts:
work/phone-batch-source-tests.log and work/control-source-regressions.log.
No application package was created, installed or launched. Existing Alpha0.1.4
remains offered. These checks do not satisfy physical playback acceptance.

A separate 30-minute loopback control-only reconnect soak is planned after this
commit, using the existing transport test and synthetic credentials. Its own
running marker/receipt must establish outcome before any success claim. It uses
no player, emulator, adb, provider account or system network changes. Planning
estimates remain playback70% / full beta30% until meaningful acceptance advances.

Control soak launched as run1e80a4d189e14a5daa018ce0f3794a98 from3f2c77e, with
1800seconds configured. Its runner PID18700 and exact UTC start time
2026-10-12T17:13:58.7823566Z were verified together. Status at inspection:running;
no outcome/acceptance claim. Log:work/control-soak/1e80a4d189e14a5daa018ce0f3794a98/
checks.log. Background continuation must preserve its cargo/test binaries and
inspect the final own-run receipt before reporting completion.

## 2026-10-12 independent Android log schema work during control soak

While control run1e80a4d189e14a5daa018ce0f3794a98 retains its verified owner,
Android phone-log sanitization now rejects numeric strings, floating point values
and text/numeric booleans rather than JSONObject coercing them into valid-looking
records. The nullable RTT field must be present; explicit null remains allowed.
This matches the strict PC receiver/inspector types and preserves integer bounds.
No owned control binary/log was rebuilt or modified.

Isolated Android source checks passed:45 JVM tests, zero failures/errors, including
two strict-value regressions. Instrumented JSON-schema regression compiled only;
no device execution. Receipt:work/source-validation/20261012-171853-35fc1d0d/
checks.json (sources stable; zero APK outputs; installed=false). Validation used
one Gradle worker. No apps/player/emulator/adb were opened and no packages were
created. Existing Desktop Alpha remains unchanged. The control soak is still
running, with PID/start identity matched; physical acceptance remains pending.
Planning estimates remain playback70% / full beta30%.

## 2026-10-12 Alpha startup feedback and midnight replacement

User reported Alpha0.1.4 Windows showed a directory listing, authorizing new
Windows/APK Alpha delivery around2026-10-13 00:00 IST (18:30 UTC Oct12). Root
cause: Tauri FrontendDist's URL-first untagged deserializer accepts C:/... as an
external URL, so the absolute candidate frontend override omitted embedded UI.
Candidate builder now uses a relative directory path and rejects rooted/scheme
paths. The actual packaged executable's --verify-alpha-assets checks directory
configuration, embedded index/mount, referenced JS/CSS and nonempty assets before
constructing any window, player, server or credential store.

Alpha0.1.5 candidate20261012-173601-84509a67 built from121ac63. All16payload hashes
verified; shipped EXE asset proof passed; Windows frontend-path regression passed.
45Android JVM checks passed with zero failures. Instrumented sources compiled
only. APK app.together.v2/versionCode6/0.1.5-private-alpha signature matches the
previous private Alphas (850876809cc2c4a9bf99936a2458a3c04c17c44fa8905e33132cb5d1e47b8f8a).
Candidate replacement-validation.json records these exact boundaries. Old offered
packages preserved. No interactive app/player/device/tunnel was launched.

The replacement includes the source-only phone journal ownership, strict sample
schema, whole-batch PC receipts and media URL encoding bound improvements. Actual
rendered UI and paired/native/WAN/physical playback remain unverified. Do not
claim an embedded-assets check proves those. Planning remains70% playback/30% beta.

Prepared scripts/Publish-PrivateAlphaDesktop.ps1 validates source/copy hashes,
preserves differing existing files, flattens Windows payload into Desktop Alpha
folder, checks the moved EXE invisibly, and writes guide/delivery receipt. Its
NotBeforeUtc guard returned pending without copying before18:30UTC. Existing
heartbeat must deliver this already-built candidate on/after that time, avoiding
another build. Work/alpha-replacement-last-run.json status=built links the package;
check candidate receipts and latest control-soak outcome before reporting.

## 2026-10-12 completed 30-minute control reconnect validation

Control run32b76c4c0e244c2b9f15f06eb6bd468d passed fromac30c404 with exit0.
Its exact log reports CONTROL_SOAK_PASSED seconds=1800 cycles=1773
physical_gate=false native_players=false; test duration1801.06seconds. Final
receipt completed2026-10-12T17:58:58.8937956Z. Logs/own-run receipt are preserved
under work/control-soak/32b76c4c0e244c2b9f15f06eb6bd468d, and the latest receipt
matches that run ID. The prior640.92second rate-limit failure remains separate.

The harness repeatedly connected the phone control socket, accepted pause/seek,
rejected stale-generation commands, disconnected/reconnected with fresh readiness,
and checked clean shutdown, fixed-state freeze, diagnostic counters, listener
reuse and host-owner release. No native player, APK, emulator or physical phone
was involved. This establishes only the bounded loopback control regression;
real TorBox/WAN/native paired playback and the physical two-hour gate remain open.
No duplicate soak started. Alpha0.1.5 remains prepared for midnight IST delivery.
Planning estimates remain playback70% / full beta30%.

## 2026-10-12 continued Android pairing ownership work (source only)

User reiterated keep developing while Alpha0.1.5 awaits scheduled delivery.
Saved pairing restoration is now cancellable and fenced: an explicit new claim
or forget cancels restoration and clears pending connection delivery. A late
noncancellable read/result or error cannot restore the previous PC or overwrite
feedback after the user's newer choice. Activity close permanently closes that
restore owner. PairingStore's disk/Keystore lock now spans separate owner instances
instead of using per-instance synchronization, preventing concurrent AtomicFile
read/write/clear and key creation from UI/update/log owners.

Four coroutine regressions passed for late saved connections, obsolete failures,
newer restore precedence, closed ownership, missing state and current failures.
49 Android JVM checks passed overall, zero failures. Source-only receipt:
work/source-validation/20261012-180959-ffcc5153/checks.json. Concurrent cross-owner
Keystore/AtomicFile instrumentation was added and compiled only; not run on a
phone/emulator. Its final compilation/source receipt is pairing-instrumentation-
checks.json in the same directory. Native proof expected tests now require that
case before acceptance. No application APK/EXE package was generated or installed.
These source improvements are not part of the immutable prepared Alpha0.1.5.
No screen/audio/system network changes. Planning remains playback70% / full beta30%.

## 2026-10-12 continued native timing and startup reservation (source only)

Android native readiness now requires finite, nonnegative timing within the
seven-day protocol range. NaN/infinity/negative/oversized native positions or
durations remain unavailable instead of becoming zero/saturated ready metadata.
The Rust native adapter now applies the same bounds to position and duration;
Windows controller readiness also requires the actual current position/duration
reads to succeed, so UI placeholders after read failures cannot report READY.
This keeps the authoritative buffering/readiness barrier in charge.

Windows startup now reserves the control listener before allocating credentials,
native surfaces or the player. A second instance fails before creating those
owners or touching the existing host's diagnostic journal. No user app was
launched to test that path while gaming; validation is compilation/source review.

Validation:51 Android JVM tests passed with zero failures, including two native
metadata regressions. Instrumentation compiled only; receipt work/source-validation/
20261012-182330-9b6a6303/checks.json. Two Rust native-time regressions passed without
loading a player (work/native-time-source-tests.log). Full Windows candidate-
feature cargo check passed with stable source fingerprint; receipt/log:
work/windows-native-time-source-check.json and .log. No EXE/APK package generated,
no visible window/audio/device work. Prepared Alpha0.1.5 remains immutable and
contains only source121ac63. The new pairing/timing/startup changes are future
source work, pending native/device verification and a later feedback-driven build.
Desktop-delivery preflight validated all16 payloads and exact destination mapping
without writing Desktop (candidate desktop-preflight.json). Planning70%/30% stays.

## 2026-10-13 actual Alpha startup and PC emulator preparation

Candidate20261012-184349-353ad1a9 built frombef6e5e as0.1.6-private-alpha.
All16payload hashes and51Android JVM checks passed; APK signing digest matches
prior Alphas. Hidden full Tauri app loaded the actual Svelte DOM, exercised its
IPC polling and initialized the packaged native player; stdout records
ALPHA_RENDERED_UI_AND_NATIVE_STARTUP_VERIFIED. This is actual application startup,
not only embedded-asset inspection. Eleven Android native cases are running under
ownerb20ae7f2-3a11-4b38-8cb8-420f643dbacb, PID16144/start18:44:41.5275715UTC.
Inspect its exact candidate usability directory/receipt and native owner before
any further installation, reverse change, emulator restart or native build.

Added isolated Test-AlphaUsability runner: headless/no-audio authorized SidecarTest,
private47822 Windows host, emulator47820 reverse, synthetic pairing, exact selected
native test identities, graceful owned app shutdown and separate receipts. Desktop
2gether Android Emulator/Open Android app.lnk prepared, not opened. Interactive
launcher only selects a delivered app with application_usability_verified, and
holds the native owner lock for the user's emulator session. Publisher now requires
actual startup plus every selected native case; no asset-only handoff is possible.
TorBox PC native proof can reuse the bounded isolated compiler cache after native
ownership is released. No provider proof yet for this candidate; delivery hold
remains. No physical phone or two-hour acceptance. Planning70% playback/30% beta.

## 2026-10-13 usable private Alpha0.1.6 delivered

Actual usability runb20ae7f2-3a11-4b38-8cb8-420f643dbacb passed eleven selected
Android native tests with exact identities, zero ignored/skipped cases, and actual
hidden Svelte/native Windows startup. Completed18:49:13.7089339UTC. Tests include
pairing persistence/concurrent owners, native lifecycle, synchronized controls,
seek spam/background/reconnect,60second steady phase, EOF/replay/execution failure,
decoder failure/recovery and Send logs to PC. Owned app shut down gracefully and
headless emulator closed. Synthetic local scope, physical gate false.

Steady matched594samples: median84.5ms, p95159ms, maximum217.5ms. Provisional
150ms p95 goal MISSED; retain this explicitly rather than reinterpret it as passed.
Actual real TorBox PC native run66a09169-a881-45ff-94b9-93034e60f79b passed muted,
headless play/pause/45second seek/resume with candidate DLL. Pause drift0ms.
Android real-provider handoff and WAN/physical playback remain unverified.

Candidate20261012-184349-353ad1a9/sourcebef6e5e delivered at18:50UTC to
C:/Users/astro/Desktop/2gether Alpha 0.1.6. All16copied payload hashes and moved
EXE asset proof passed. APK7/0.1.6-private-alpha signer unchanged,51JVM tests
passed, actual frontend path unit executed once/passed. Usability hold archived
as work/alpha-usability-hold-resolved-b20ae7f2.json. Older desktop packages and
user-running app preserved; no windows focused or user volume changed.

Desktop emulator launcher is ready but was not opened interactively. It selects
only usability-verified delivery and uses emulator PID/start marker as well as
held lock to preserve a live user session even if its terminal closes early.
Normal package hold resumes: no new EXE/APK until feedback or explicit request.
Planning70% playback/30% full beta unchanged; physical two-hour gate outstanding.

## 2026-10-13 strict Android media authority and timing arithmetic (source only)

MediaCapabilityClient previously used JSONObject.getInt/getLong for protocol,
generation and source_revision; those coerce numeric strings and truncate fractional
values. It now requires actual Int/Long JSON values, exact protocol/generation,
nonnegative expected generation and positive source revision. Epoch and media URL
also require actual strings. Invalid grants retain the fixed private failure and
never reach the native player. Pure unit regressions cover malformed numeric
shapes and stale authority; the actual HTTP instrumentation case was added and
compiled only, not executed with the new source.

PlaybackTiming now rejects finite clock values that would saturate Double.toLong,
and checks target addition before it can overflow into a negative correction/seek.
Paused targets remain fixed without needless clock arithmetic. Regression covers
large clock offsets, overflowing targets, exact safe boundary and negative offset.

54 Android JVM tests passed, zero failures; instrumentation compiled. Isolated
receipt/log path is recorded in work/media-grant-validation-location.txt under
work/source-validation. No APK generated/installed, no EXE build or native test,
no emulator/window/audio/system-network changes. Delivered Alpha0.1.6 unchanged;
these changes wait for feedback-driven packaging. Planning70% playback/30% beta.

## 2026-10-13 connected Stremio checks, selection fencing and maintainability

Recorded user request for periodic code-quality review in AGENTS.md: inspect
both apps after substantial changes, reduce duplication/unused code or dependencies,
optimize evidenced hot-path costs, and avoid broad refactoring during Alpha tests.
Stremio options/episode discovery now rechecks selection revision before and after
each provider/cache stage; an obsolete request stops before more network work.
Final publication fencing remains. Ten Windows unit tests passed. Read-only
connected Stremio proof passed saved-account/catalog/movie-stream/cache/series-
episode inspection:4configured addons, movie catalog56records,51stream descriptors,
24exact cached choices; series59records,99episodes,10episode streams/5cached choices.
No account writes, addon media URL follows or playback handoff. Counts-only log:
work/stremio-connected-source-check.log. Actual streamed viewing is separate.

New torbox-paired-proof standalone harness reuses the production PlayerController
with hwnd0 (null video/audio), Host and direct TorBox grants; no copied playback
engine or new app dependencies. Test-RealProviderPair uses the offered Alpha APK
and archived instrumentation without rebuilding APKs, copies hash-checked native
payloads into a new isolated run, and respects exact PID/start/lock ownership.
New tests do not update Desktop packages or imply physical/WAN acceptance.

Initial preflightb386bdaa failed before emulator start due to expected-test list
shape, corrected. First native run9e048c0c failed preparation: phone loaded real
TorBox metadata, but no PC native observations existed. Cause was HARNESS duplicate
PC attachment before PlayerController's own attachment, making its feedback task
exit. Corrected harness leaves that ownership to production controller and waits
for actual native/PC startup. This is not evidence of a shipped-app defect.
Failures remain preserved. Corrected runbc8a5396-3144-4f90-b96d-2dd35104f126 is live,
PID20236/start19:07:28.4178672UTC. Inspect exact owner/receipt, do not duplicate.

Proof runners now preserve available matching failed phone observations before
cleanup and restore process-local proof environment. The first failed run's full
phone file was not collected; its raw assertion and PC/session journal are retained.
These changes apply to subsequent runs. No native acceptance claimed yet. Routine
release-package hold remains; Alpha0.1.6 stays unchanged. Planning70%/30% unchanged.

## 2026-10-13 real Android TorBox observations and source reuse fix (source only)

Corrected runbc8a5396-3144-4f90-b96d-2dd35104f126 completedFAILED at19:11:22UTC.
Using offered Alpha0.1.6 and headless production PC controller, the phone loaded
actual TorBox media (duration1456457ms). Synchronized start, pause, exact seek,
rapid seek, reconnect/background and60second steady checks reached recovery;
EOF/replay also reached the subsequent pause. The pause-after-replay execution
acknowledgement then failed with execution_timeout_phone. This is real-provider
emulator evidence, not complete success/WAN/physical acceptance. Original failed
receipt/raw assertion and PC native/session journals remain, with review.json
recording partial observations. Full phone sample file was not collected by that
runner revision; raw assertion retains final native/diagnostic status. No repeat
of the unchanged Alpha failure is useful without a change or user feedback.

Android previously requested a fresh grant AND reloaded HTTPS media every generation,
including pause. The repeated decoder/network work is consistent with missing a
short pause execution deadline; exact causal native validation remains pending.
New source keeps requesting/authenticating the fresh current-generation grant,
then reuses the loaded file only when canonical identity, PC epoch, provider
source revision and accepted/current native revisions all match, observations are
fresh/ready and no native failure exists. Changed PC/file/grant, stale native work
or faults require reload. Explicit disconnect clears binding. Reconnect retains
only the bounded identity/revision markers and revalidates authority before use.
Grant object has private URL and no URL-bearing data-class toString; binding has
no URLs/credentials. No native acceptance of this new source is claimed.

Code-quality review also bounded total Stremio descriptors across addons to1000,
sharing the same MAX_STREAM_CHOICES with parser and cache preparation. Previously
multiple individually bounded addons could grow a much larger combined list and
then fail cache lookup. Oversized combined discovery now stops with fixed feedback.
Validation:58Android JVM tests passed, instrumentation compiled only; receipt
source-reuse-checks.json in the directory named by work/media-grant-validation-
location.txt.39resolver library tests and10Windows tests passed. No new release
APK/EXE or Desktop changes. Source fix waits for feedback-driven build and real
provider rerun. Physical two-hour/WAN gates open; planning70% playback/30% beta.

## 2026-10-13 Android connection and timing boundary review (source only)

SessionController.connect now reuses pairing address/token validation before
replacing any existing connection. Only exact /session WSS endpoints and the
already authorized debug localhost47820 endpoint are allowed. Userinfo, queries,
fragments, alternate paths/schemes/ports and malformed/oversized credentials are
rejected with fixed private errors before a socket/header is constructed.

Execution acknowledgements and EOF timing now use the same checked PlaybackTiming
helpers as drift correction. Observation time is captured once; native sample age,
finite offset, Double-to-Long saturation and timeline-addition overflow are checked.
EOF comparison uses subtraction to avoid overflowing estimate+2500. Paused targets
remain fixed. New timestamp regression checks sampling age and saturation.

The periodic playback follower now catches non-cancellation exceptions, pauses via
existing lost()/reconnect cleanup, and keeps its child loop alive. Previously a
malformed state could throw out of follow() and permanently stop that child while
clock/socket callbacks still showed connection activity. Guard is compiled/reviewed,
not claimed runtime-validated. Cancellation still propagates for normal close.

62 Android JVM checks passed with zero failures; Android instrumentation compiled
only. Receipt session-boundary-checks.json, latest immutable JUnit copies in
session-boundary-junit, and command log session-boundary-final-checks.log under the
isolated source-validation directory named in work/media-grant-validation-location.txt.
No APK/EXE release build, device installation, emulator/screen/audio/network change.
Alpha0.1.6 stays offered and unchanged; real-provider recovery fix still awaits
feedback-driven package/native validation. Planning70% playback/30% beta unchanged.

## 2026-10-13 pairing feedback and replacement source preparation

User reports normal PC/Android pairing failure, authorizes emulator use and a new
Alpha after full core checks, and requests Stremio-style PC selection UX. The
normal installation has no control-tunnel deployment; previous Windows UI showed
only a code, without the server address. This is a verified connection-setup gap,
not proof that every reported failure has the same cause. New Connection page
shows emulator address separately from a configured public HTTPS origin and
reports absent tunnel provisioning. Android has an emulator address shortcut,
fixed failure categories (reachability/code/TLS/persistence), normalized code input
and observable session mode validated before replacement. No credentials displayed.

PC now has Discover/Watch/Connection navigation, movie/series title cards, episode
and source choices and explicit connection state. Existing provider revision and
PC-authoritative control boundaries remain. Cards use text covers; untrusted
addon artwork URLs remain excluded. Hidden app validation now supports normal
DPAPI startup without synthetic credentials, remains muted, and records a one-time
nonce in ignored work. Added actual Android pairing-screen claim and normal Activity
saved reconnection instrumentation. Publisher requires this extra acceptance.

Also stops a rejected PC attachment's new native worker without detaching its
existing owner, disables raw native URL-bearing log messages explicitly, and
extends bounded diagnostics inspection with typed fault/health summaries. Source
checks:11Windows tests,7player-mpv tests,6Python diagnostic tests and Svelte check
passed. Android source JVM/instrumentation compilation passed; final candidate
checks and normal-screen native execution remain pending. Existing Alpha0.1.6
and historical failures preserved. No physical/WAN acceptance claimed;70%/30%
planning estimates unchanged. New Alpha0.1.7 delivery requires new actual checks.

## 2026-10-13 actual pairing and terminal-failure race

First candidate0.1.7/90de133,20261012-195529-e0631ad6, is NOT delivered. Native
usability run99bd7387 failed the terminal decoder-failure scenario after other
controls/reconnect/replay reached their assertions. Failed receipt and complete
matching phone observations are retained. Observation transitions show decoder_failed
at native revision5, generation37, then an unsolicited fixture reload revision6
before player_failed_phone was recorded. A buffering ACK generation race could
clear loaded and trigger reload while the failure command waited behind inflight
control. New source prevents reload until authority records the native fault and
an explicit recovery generation permits one load attempt. New-source native rerun
is pending; no resolution claim yet.

Separate normal UI runc4d408bd-90dd-40c5-b8e4-51be78fd3819 PASSED on exact candidate
90de133:actual Android address/code fields, normal DPAPI PC credentials, authenticated
clock response, encrypted save and new normal Activity saved reconnection. This
uses no injected bearer connection. Pre-existing emulator encrypted pairing is
restored afterward. It proves local normal pairing, not her phone/WAN reachability.

Missing public provisioning now has a PRIVATE Alpha temporary cloudflared control
connection path, explicit start/stop and changing-address instructions. Connector
binary2026.9.3 is copied read-only from existing authorized toolchain, pinned and
bundled with license. Child is atomically created inside its own kill-on-close
Windows job, inherits only NUL plus its own capture, never shell/token/proxy config.
Capture is bounded and supervised independently of UI polling, off playback workers;
only a strict public trycloudflare HTTPS origin enters local UI. Shutdown owns child
and capture cleanup. Control host has no media proxy route. Named deployment remains
supported. No system networking/adapter/DNS/proxy/firewall settings change.

Automatic approval review rejected the command to create/run actual public-tunnel
validation, with only 'blocked by policy'. WAN temporary-connector validation remains
UNEXECUTED; do not describe it as passed. Continue local actual app/core checks.
Cloudflare documents temporary testing URLs changing each launch:
https://developers.cloudflare.com/tunnel/get-started/quick-tunnels/.

Corrected candidate782db5e also FAILED decoder-failure acceptance in runf0730314;
it is withheld, with complete phone transitions preserved. Additional race found:
generation acceptance interpreted current pending-but-unaccepted source as lost
and reloaded it before its native decoder result. Native source loads now use
source revision fencing independently of transport command epochs. A generation
checks current source revision, preserving queued decoder work until acceptance
or failure. Grant replies also recheck newly arrived native failures before
issuing a replacement load. Earlier recovery authorization gate remains. These
changes require another actual native rerun; no success claimed from source review.

Candidate7f2a7fc/20261012-201621-9a665989 passed all11native fixture cases, including
terminal decoder recovery, runefd749d7. Normal UI initially had a harness-only
stale disabled-button click failure; the fixed instrumentation waits for fresh
enabled semantics. Normal UI run5c616583 then passed against SAME APK. All75build
inputs were snapshotted before/after; only PairingScreenTest.kt changed, APK was
not rebuilt and all18payload hashes remained. Combined component receiptd5813eb3
independently reparses the11native and1normal identities and preserves originals.
Synthetic steady594matched observations:p95110ms,median35ms,max166ms (provisional
150ms goal met in this run only). Physical gate remains open.

Real TorBox runabe0fcad also reached and passed the previous pause-after-replay
boundary but FAILED terminal decoder-failure reporting. Fresh current-generation
grant returned while the current native source was pending, and readiness-based
reuse rejection issued a replacement before the decoder result. New source waits
up to15seconds for same-authority pending native readiness/failure before deciding
reuse/reload; cancellation/generation/epoch/socket changes still fence the wait.
Actual failure is preserved for PC authority. New package/native/provider rerun
required; candidate above remains UNDELIVERED. Full matching phone log retained.

## 2026-10-13 delivered Alpha0.1.7 after sustained core repair

DELIVERED candidate20261012-202957-e253509f,compiledsource5ea1824,AndroidversionCode8,
0.1.7-private-alpha to C:/Users/astro/Desktop/2gether Alpha 0.1.7. All18package and
Desktop payload hashes and moved EXE embedded-frontend proof verified. Existing
packages preserved. Signer remains850876809cc2c4a9bf99936a2458a3c04c17c44fa8905e33132cb5d1e47b8f8a.
Normal Desktop emulator launcher selects this usability-verified delivery; no
interactive emulator/app opened by delivery. No native/control owner remains.

Actual hidden app startup, all11selected native fixture cases, and normal pairing
screen/encrypted save/normal Activity restore PASSED together in run89b0cb81-70f2-
42c9-abdb-c1cfb8e65d76. The final app includes the enabled-button harness fix, so
no composite acceptance is needed for delivery.63JVM checks passed,11Windows tests,
7player-mpv tests,32tunnel tests (9ignored fixture entrypoints; no WAN claim),6typed
log-inspection checks and Svelte zero errors/warnings. Prior failed candidates and
all failed/component receipts remain unmodified for review.

Real TorBox runf77f3066-f11a-42e1-bcaa-cd037a75f9d6 PASSED the full selected native
sequence:real media load, shared controls/seeks, background/reconnect,60second
steady interval,EOF/replay,acknowledged pause,deliberately missing start freezing,
explicit retry,terminal decoder failure held at PC authority,paused reload recovery
and authenticated phone log upload. Scope=headless nativePC+Androidemulator using
loopback control and direct TorBox HTTPS from same PC egress; NOT physical-phone
or public control/WAN acceptance. APKhashb010ffb8a389d76c9b536cb559aaa0e6c290fea7989114f3cbbef1d740cd577f.
Matching complete phone/PC journals and raw instrumentation retained under its
work/real-provider-pair directory. Steady matched589pairs:p95133ms,median60ms,
max328ms; fixture594pairs:p95150ms,median69ms,max176.5ms. Both meet provisional150ms
p95 in these runs only; interpolation/clock/property sampling uncertainty remains.

Temporary public cloudflared start/stop is bundled and explicitly user-operated.
Local process/capture/job cleanup tested; actual public-tunnel validation was
rejected by automatic approval ('blocked by policy') and remains UNEXECUTED.
START-HERE and desktop-delivery.json explicitly record public_control_tunnel_verified
false and physical_gate_passed false. No remote-phone/two-hour gate claim.

Maintainability review:no new app libraries/crates were added. Decoder loads use
source revisions; transport commands keep generation fencing. Media grant waits
remain cancellable and yield Main; native I/O/log supervision stays off playback
workers. UI uses existing typed provider/authority commands and bounded descriptor
budget. Raw native logging remains disabled; typed bounded diagnostics retained.
Public artwork URLs remain excluded; title covers are placeholders. No broad
refactor or extra chat/library migration. Normal package hold resumes until user
feedback/explicit request. Continue focused source development around user testing.
Planning estimates now75% playback/35%full beta, reflecting implemented recovery,
normal pairing UX and PC browse/watch foundation; physical/WAN gates remain open.

Final launcher guard now verifies the delivered APK hash and removes only old
private proof-pairing.json before normal user Activity launch, after owning its
newly created emulator. It retains encrypted pairing and keys. This prevents a
crashed validation's injected credential from bypassing normal pairing UI. Launcher
syntax reviewed/parsed; not opened interactively by this turn. START-HERE clarifies
one Android device at a time (close emulator before physical-phone testing).


### 2026-10-13 storage cleanup and follower retry work (source only)

Measured repository logical file sizes:93,104,870,460bytes (86.71GiB) before,
48,698,919,287bytes (45.35GiB) after; removed44,406,082,403bytes (41.36GiB).
Removed only redundant historical candidate Rust caches, historical Android
intermediates/tmp and Rust incremental caches. Kept root dependency/build data,
shared current candidate cache20261012-135012-d336360a/rust-target, SDK/AVDs,
source/Git, credentials/signers, delivered Alphas, candidate payloads/APKs/EXEs,
source archives and failed/passed validation receipts/journals. Verified587
protected files by SHA256 before/after with zero mismatches. Receipts:
work/storage-cleanup-20261012-234020.json, storage-protected-hashes.json and
storage-cleanup-verification.json. Sizes are logical file bytes, not an exact
physical-allocation claim. Rebuilding historical candidates regenerates their
removed intermediates; current dependency cache retained to limit rebuild cost.

Added scripts/Trim-DevelopmentCaches.ps1 with preview default, native/control
exclusive leases, conservative control-marker/compiler denial, strict repository
path/reparse checks and removal receipts. Preview44.4GB matched executed cleanup.
A concurrent preview while Android compilation ran was correctly refused without
mutations. Subsequent preview is empty. C: remains the active checkout; D: HDD has
space but moving source/compiler/active emulator would add random-I/O cost.
No relocation or Desktop changes were made.

Android follow() now respects mediaRetryAt before reprocessing its already-
validated current snapshot during an unloaded failed-grant retry. Previously it
repeated accept()/identity parsing and diagnostics bookkeeping every10ms while
accept() itself refused another request until3seconds. Incoming network snapshots
and native-failure/recovery handling retain their separate paths. This is a focused
hot-loop change, no new dependencies.63Android JVM checks passed, instrumentation
Kotlin compiled only, JUnit copied to source-validation/20261012-media-grant-
89aef475/storage-pass-junit and storage-pass-validation.json. No APK, EXE, emulator
or native playback test was built/launched. Offered Alpha0.1.7 remains unchanged;
source optimization is not native validated. Planning stays75%playback/35%fullbeta;
public/WAN and physical-phone two-hour gates stay open.


### 2026-10-13 cancellable pairing and occupied-room UX (source only)

Inspected working tree/latest notes/native ownership before changes:clean, no
native marker, owner lock free, no control marker. Delivered0.1.7 remains unchanged.
PairingClient previously executed a blocking OkHttp call inside Dispatchers.IO;
cancelling setup did not promptly cancel the outstanding HTTP call. It now awaits
an asynchronous owned response with cancellation tied to Call.cancel(). Decoder
runs on OkHttp worker, closes responses before delivering values/errors, and closes
late responses without decoding after owner cancellation. Pairing input/TLS/
redirect/15second/body4096byte boundaries retained. Close uses existing owned-client
utility rather than another cleanup implementation. No new dependencies.

Pairing replies now require HTTP200, an exact integer protocol1 and a string bearer
that passes existing credential validation. Fractional/coercible protocols and
non-string credentials are rejected rather than JSONObject getter coercion.
Malformed/oversize replies have fixed incompatible-response feedback, excluding
body/address/token/cause text; network failures retain reachability/TLS feedback.

Windows now disables both pairing actions while an Android device is connected
and explains closing the emulator/disconnecting before another phone. Before this,
a newly generated code was hidden immediately by normal connected-phone refresh,
making pairing appear broken. This UI guard does not replace server slot authority.
Svelte check passed0errors0warnings; actual rendered interaction not tested for
these source changes, no new Windows binary.

69Android JVM checks passed:five response-ownership tests cover successful/failed
decode cleanup, late response/failure after cancellation, and an actual loopback
OkHttp response with stalled body. Cancellation interrupted read and drained running
call within2second test bounds. Added exact-protocol rejection check. Instrumentation
Kotlin compiled only; no APK or emulator launched. Initial test incorrectly required
exception object identity (coroutine stacktrace recovery clones it); changed assertion
to preserved exception type/message, retained initial failure XML under
source-validation/20261012-media-grant-89aef475/pairing-cancellation-initial-failure.
Final69JUnit XML preserved in pairing-cancellation-junit, log
pairing-cancellation-final-checks.log and pairing-cancellation-validation.json.

Maintainability review kept scope to22line owned response helper, reused HTTP
client cleanup and existing PC connection status; no playback-loop work/dependency
added. Source fixes are not packaged/native/UI/WAN validated. Planning unchanged
75%playback/35%fullbeta; physical-phone two-hour and public control gates remain open.


### 2026-10-13 control liveness and owned HTTP response consolidation (source only)

Read latest notes/working tree/ownership first:clean, no native marker/held lock,
no control running marker. No packages, emulator or native resources changed.
Android previously updated lastSeen immediately after parsing any JSON message,
before epoch/protocol/request matching. Stale/replayed/unmatched clock frames could
therefore postpone control-silence recovery. Now snapshot/ACK liveness advances
only after state acceptance; clocks require exact integer protocol1, matching epoch,
exact nonnegative timestamp integers, ordered server stamps and a current one-use
ClockRequests echo. Unknown/server-error message types trigger fixed control loss.
Valid delayed (>estimator RTT threshold) correlated replies still count transport
liveness; they are estimator outliers, not automatically a broken connection.

Clock timestamps reject JSONObject numeric coercion and integers above2^53-1,
which cannot safely round-trip through double estimator arithmetic. Added three
JVM checks for boundaries/coercions and preserving a real outstanding exchange
when a coerced reply arrives; existing ClockRequests replay/reset/expiry checks pass.
The onMessage liveness wiring is compiled/reviewed, not native runtime verified.

MediaCapabilityClient and manual PhoneLogs upload now reuse awaitDecoded, introduced
and tested by preceding pairing work. Late cancelled responses close without decode;
normal decoding closes before delivering values/errors. Both clients explicitly
preserve CancellationException and fixed sanitized IOException categories. Grant
16KiB/body/epoch/generation/revision/canonical file/CDN/redirect checks unchanged;
log128sample/10row upload chunks/204ack/schema bounds unchanged. AndroidUpdater's
separate streaming helper retains its active-read/progress/file cleanup contract;
no broad updater rewrite. Net production code reduced; no dependencies added.

72Android JVM checks passed; instrumentation Kotlin compiled only. Final XML
copied to source-validation/20261012-media-grant-89aef475/control-liveness-junit;
control-liveness-final-checks.log and control-liveness-validation.json retained.
No APK/EXE, native or emulator test launched. Desktop Alpha0.1.7 remains compiled
source5ea1824; these source changes await a later authorized package/native pass.
Planning75%playback/35%fullbeta unchanged; no public/WAN/physical-phone gate claim.


### 2026-10-13 Windows status ordering and idle native work

Inspected clean tree/latest notes/ownership first: no native marker/owner lock or
control marker. Source-only changes; delivered0.1.7 unchanged. Windows refresh now
allows one player/session/diagnostics IPC batch at a time. Connection requests carry
an increasing local request ordinal, so an old poll cannot overwrite a later
Start/Stop/refresh response. Svelte check0errors0warnings. UI ordering is compiled/
reviewed, not actual interactive rendering validated for these source changes.

Read-only typed inspection of normal diagnostics found37796native-PC samples,
all unready, no selected-media/playing samples and zero uploaded phone samples.
This is idle evidence, not proof that any phone test passed or failed. It exposed
avoidable idle polling/journal churn. Native worker now uses100ms receive timeout
without selected media; active media/proof logs keep10ms. Real channel deliveries
wake immediately. Idle journaling uses2second spacing plus immediate native state
transitions; active media/proofs keep100ms samples. Headless players explicitly
set mute=yes in addition to null video/audio outputs; no system volume changes.

First real idle regression failed observation bound. Inspection found the feed
was sending identical snapshots every20ms, defeating the longer receive timeout.
Feeder now coalesces only already-delivered(epoch,revision,generation,sourceRevision)
identities; Full queues retry, Disconnected stops. Changed authority revisions or
media grants are forwarded. Existing native feedback/health checks remain active.
Fixed reducer RTT updates advance revision; source revision is separately included.

12Windows tests passed, including actual libmpv idle/load, persistent command faults,
source replacement and cleanup. New five-second native idle run143f7f6a-f884-4ab6-8a3f-036eb1e62d22
observed50status timestamps and3native journal rows,
then selected fixture loaded successfully; diagnostic errors/drops zero. Actual
logs/review retained in work/source-validation/idle-native-143f7f6a-f884-4ab6-8a3f-036eb1e62d22.
Initial failed log windows-idle-cadence-initial-failure.log and failed directory
idle-native-590a0848-11bd-41f8-b91d-49f7385a1b60 preserved. Final log
windows-idle-cadence-final-checks.log; source hashes/summary in
windows-idle-cadence-validation.json. Tests held native/control exclusive leases,
used null/muted players and opened no app windows/emulator/network tunnel.

Scope=headless PC idle/fixture unit validation only, not Android paired native
playback, real TorBox pair, public/WAN or physical two-hour acceptance. Earlier72
Android source checks remain historical, not rerun this pass. No new release/APK,
no Desktop changes. Maintainability kept changes in existing polling/feed owners;
no libraries/crates added. Planning remains75%playback/35%fullbeta.


### 2026-10-13 PC buffering feedback bound (source only)

Read clean working tree, AGENTS/latest notes and owner inventories before changes:
no native marker/held owner or control running marker. Reviewed grant-slot final
handoff fencing, source revisions, Android pending native results, persistent
pairing boundaries and typed journal upload. No new issue claimed in those paths;
no unchanged Android/provider/WAN tests repeated.

PC native poll runs10ms with media while the authority feeder delivers current
state on20ms ticks. Repeated buffering feedback can therefore be queued before
updated authority arrives, especially if feeder work is delayed. Native owner now
retains the last successfully queued(generation,buffering) report; identical reports
wait for authority delivery, different states/new generations remain eligible.
Failed sends do not mark a report delivered. Terminal native faults take precedence
and suppress buffering feedback, matching Android's failure-first path. Generation
changes clear this small gate. No new actor/timer/library added.

Added headless actual-libmpv regression exercising12PC buffering cycles and parsing
bounded typed command journals. Baseline run50bef0e0-2d2d-4db7-a461-719f590ae4e8 PASSED
with23buffer-command generations and0duplicates; did NOT reproduce the potential
lag-related duplicate. Baseline log buffer-feedback-baseline.log preserved. Revised
runf93942ba-a9a2-4e68-a245-1e962a9de0fd also PASSED23generations/0duplicates. Both
journals/review.json remain under work/source-validation/buffer-feedback-<runid>.
This supports normal feedback behaviour, not proof of a delayed-feeder pressure
scenario or a claim that a previously reproduced failure was fixed.

All13Windows source tests passed, including native fault retention, idle/load,
source replacement, cleanup and provider selection revision checks. Exact final
log work/source-validation/buffer-feedback-final-checks.log. Test binaries held
native/control exclusive leases; players used headless/null outputs and explicit
mute. No normal EXE/APK packaged, no user emulator/app/network tunnel disturbed.
Offered0.1.7 remains compiled source5ea1824. Validation scope=headless PC buffering
feedback/unit lifecycle; not Android pair, real provider pair, WAN/public control
or physical two-hour acceptance. Earlier72Android checks historical only. Planning
remains75%playback/35%fullbeta. Maintainability kept the guard in existing native
owner and existing typed diagnostics; no dependencies or extra features.


### 2026-10-13 recoverable phone-log batch acknowledgements (source only)

Inspected clean tree, latest AGENTS/notes and owner inventory: no native marker/
held lock or control marker. Journal lifetime io_errors was used both to refuse
phone-batch admission and to invalidate later successful batch receipts. Thus an
unrelated historical journal failure permanently blocked phone-log sending for
that app lifetime, even when the active writer could store a new batch.

Added regression with a deliberately bounded journal: an oversized unrelated
session event fails, while a smaller valid phone sample can fit. Baseline test
reproduced diagnostic_writer_unavailable. Initial baseline asserted missing output
rows first; moved the result assertion after cleanup to expose the exact rejection.
Both baseline logs preserved:phone-batch-recovery-initial-baseline.log and
phone-batch-recovery-baseline.log. No user diagnostic files or real credentials
were altered; tests used isolated temporary directories/typed synthetic data.

Admission now requires an active writer and valid bounded batch. Queue/writer/
timeout rejection remains. Worker acknowledges only that batch's record-and-flush
result; historical io_errors remain cumulative, embedded in the next successful
phone row and available to health/inspection. General Journal.flush still refuses
overall complete-history acknowledgement after prior I/O errors; this change does
not relabel lost history or offer fsync/power-loss guarantees. Current write failure
still returns diagnostic_batch_failed. Existing repeat-disk-failure test now expects
that precise current failure instead of the former permanent admission refusal;
its no-output/no-success assertions remain. Initial suite failure on that obsolete
error-category expectation preserved in phone-batch-recovery-initial-final-checks.log.

14diagnostics unit checks and2phone upload endpoint checks PASSED. Coverage includes
actual filesystem row/flush/order, historical error retention, actual batch-write
failure, queue pressure, bounded missing-ack timeout, writer disconnect, privacy,
authentication and retry deduplication. Final logs phone-batch-recovery-final-checks.log
and phone-batch-recovery-endpoint-checks.log; source hash/receipt
phone-batch-recovery-validation.json in work/source-validation. Rust tests held
native/control exclusive leases; no native playback/emulator/window/tunnel test.

Offered Alpha0.1.7 remains unchanged; fix is source/unit validated only, not a new
packaged-app/manual-button/device/WAN validation. No SDK keys, log schema fields,
Android code or dependencies changed. Planning75%playback/35%fullbeta unchanged.
Physical-phone two-hour and public-control gates remain open; public command's
prior approval rejection was not retried.


### 2026-10-13 bounded phone-log retry receipts (source only)

Read current clean tree/latest notes/AGENTS and owner inventory first:no native
marker/held lock or control marker. The upload endpoint remembered only the last
batch UUID and did not bind it to the accepted sample content. Baseline endpoint
regression A,B,retryA,changedA returned204forchangedA rather than409, acknowledging
samples it had not written. Baseline retained in phone-batch-retry-baseline.log.
The prior last-ID-only state also permitted interleaved retries to be written again.

Host now remembers at most64completed batch IDs and SHA256fingerprints of typed,
validated samples. Existing sha2 dependency reused; no raw bodies, URLs or credentials
retained. Identical cached retries receive the original receipt; conflicting reuse
returns409without writing. New IDs enter cache only after successful batch write+
userspace flush. Expired IDs/new process start may be written anew; cache is bounded
and ephemeral, not an indefinite delivery guarantee or persistence/fsync claim.
Protocol1/request fields/authentication/size/rate checks remain unchanged. Hashes
use canonical typed serialization so input object key order does not affect equality.

4phone upload tests and15diagnostics tests PASSED. Endpoint regression verifies
only two journal phone rows for A,B,retryA plus rejection ofchangedA; authentication/
privacy/retry cases retained. Additional checks cover64entry retention/eviction and
two identical HTTP attempts against an actually unwritable tiny journal:both503,
zero data rows,io_errors2 (failed IDs never become cached success). Prior historical
I/O recovery/current failure/timeout/queue/rotation tests passed. Source checks held
native/control exclusive leases, no app/emulator/player/network tunnel launched.

Exact logs phone-batch-retry-final-checks.log and phone-batch-retry-writer-checks.log;
source hashes/receipt phone-batch-retry-validation.json in work/source-validation.
Baseline failure preserved, not relabelled. Offered Alpha0.1.7 unchanged:source/
backend unit validation only, not packaged UI/manual phone button, device or WAN
validation. No Android/log-schema/dependency additions. Planning75%playback/35%full
beta unchanged; physical two-hour/public-control gates remain open. Maintainability
kept the bounded cache next to existing upload types and reused the current host
upload mutex; no actor, timer or background sender added.

### 2026-10-13 Stremio connection failure clears obsolete browsing state (source only)
Code review found that library/addon connection failure removed the client but left
old catalogs, titles and private choices. Browser had no connected flag, so the
frontend could still offer stale title actions and Refresh despite needing a new
connection. Public browser now reports connected; current connection failure
clears catalog/title/episode/choice state, while an obsolete failure cannot erase
a newer account. Connect checks revision after secret loading and before library/
addon requests, avoiding later stages of obsolete work. UI hides disconnected
browse actions and offers Connect again. Current playback/session is untouched.

Five focused Stremio backend tests passed (11 other Windows tests filtered out),
including the actual nonexistent-vault path, late-failure ordering and existing
choice fences. Svelte static check: zero errors/warnings. Baseline log reproduces
only the absent public connected field (Null vs false); live library failure and
rendered UI recovery were inferred from reviewed code, not exercised. No account
HTTP calls, native playback, emulator, APK or release EXE work. Native/control
leases protected unit validation. Exact retained baseline/final logs and source
hash receipt: work/source-validation/stremio-connection-state-validation.json.
Removed one unused mutable read-only tunnel status guard; no dependencies added.
Offered Alpha0.1.7 stays unchanged; actual provider/UI/device/WAN validation remains
open. Planning estimates remain 75% playback / 35% full beta.

### 2026-10-13 Ignored Android authority state cannot refresh liveness (source only)
Control review found accept() returned early for older same-epoch revisions, but
its caller then updated lastSeen and processed the frame's command ACK anyway.
This contradicted the accepted-state liveness boundary. accept now returns false
for that ignored state and true after current-state acceptance; the socket handler
returns before liveness, ACK completion/error notice and queued-command flush when
false. Equal revisions remain accepted (normal authority ACKs can repeat state).
Internal follower retries keep their existing current-state acceptance behaviour.

Extended the existing synthetic SessionIdentity instrumentation assertions to
check true for accepted state and false for obsolete state. Android main and test
Kotlin compiled successfully; instrumentation NOT executed, no JVM tests repeated,
no APK/new EXE, emulator or provider traffic. This is a code-review finding, not a
reproduced live socket failure or validated six-second silence/reconnect result.
Native/control leases held for compilation, ownership free afterwards. Exact log
work/source-validation/stale-snapshot-compile.log and source hashes/boundaries in
stale-snapshot-validation.json. Offered Alpha0.1.7 unchanged. Playback75%/full-beta
35% estimates unchanged; physical/WAN gates open. No new helper/library/refactor;
reuse of the existing acceptance boundary keeps liveness ordering explicit.

### 2026-10-13 Local preparation recovery and exact control envelope numbers (source only)
LocalSourceLoader previously let preparation/handoff exceptions escape its launch.
It now reports a fixed callback error, preserves CancellationException and rejects
late failure reporting after cancellation/session takeover. Player displays a fixed
sample preparation error and pauses; this does NOT invent a terminal native fault
that could prevent a later paired source load. Successful native loading clears
its existing error normally. Five focused JVM cases passed: retry after injected
preparation failure, handoff exception containment, obsolete failure cancellation
and the two previous replacement/delivery cases. Actual UI/native behaviour remains
unverified; no live crash or missing-asset APK failure was reproduced.

Related control review found JSONObject coercions still accepted protocol/revision/
generation and ACK sequence strings/fractions. Snapshot envelope now requires exact
integer protocol1 and nonnegative revision/generation before source/authority work;
ACK requires a positive exact integer sequence before acceptance/liveness. Reuses
MediaGrantNumbers.integer rather than adding another parser. Existing synthetic
SessionIdentity instrumentation now asserts invalid envelope numbers do not change
accepted state. Main and instrumentation Kotlin compiled; these instrumentation
assertions and socket ACK handling were NOT runtime executed. No APK, release EXE,
emulator, account requests, native playback or public tunnel launch in this pass.

Exact source hash receipt and immutable five-case JUnit:work/source-validation/
20261013-local-recovery/receipt.json and LocalSourceLoaderTest.xml. Logs are
local-preparation-recovery.log and control-envelope-compile.log in source-validation.
Native/control exclusive leases held for source validation; no owner afterwards.
Maintainability: one mandatory failure callback on existing loader, no new jobs,
helper types or dependencies. Delivered Alpha0.1.7 unchanged. Planning75%playback/
35%full beta unchanged; combined packaged UI/native, WAN and physical gates pending.

### 2026-10-13 Responsive Stremio preparation/watch and bounded lookup (source only)
Replaced provider-length Stremio state reservations. Cached preparation retains a
private Arc to its validated plan, releases browser state during TorBox IO, then
checks the original revision before storing a ready result. Existing provider
create-attempt serialization/no automatic create retry remain unchanged; superseding
a UI selection cannot undo an already submitted create, and this patch does not
claim it can. Pending watch resolution now also releases browser state while
fetching separate device links and probing ranges. A private non-serializable
ResolvedSelection is consumed at publication. Stremio revision is rechecked under
its lock, followed by existing TorBox selection and PC epoch/generation checks;
the short commit boundary stays fenced. URLs never enter IPC or test diagnostics.

Configured addons can number128, with20second request timeouts, so serial episode/
stream lookup previously had no usable overall time bound. Read-only episode and
source-option lookup now have a60second total deadline. Cancellation drops pending
lookup work; no cached creation/watch publication transaction uses this timeout.
This is a source latency bound, not measured real-provider performance improvement.

New concurrency checks hold synthetic provider work while browser reads and a new
selection succeed, then prove obsolete preparation cannot replace choices and
obsolete watch cannot invoke publication. Current success/pending/failure results
are covered. Full20Windows checks passed after lock changes, including headless
muted native lifecycle/fixture cases; after adding lookup deadline the focused10
Stremio checks passed (11other Windows tests filtered). Timeout check drops a real
pending future at20ms and preserves normal results. Exact logs are stremio-
preparation-lock-checks.log, stremio-selection-lock-checks.log, stremio-lookup-
deadline-checks.log; source hashes/scopes in stremio-responsive-selection-validation.json
under work/source-validation. Full run compiled existing proof examples and emitted
one unused-mut example warning; checks passed. Real TorBox revised transaction,
rendered UI and Android pair sequence were NOT exercised in this pass. No new
release EXE/APK, public tunnel or physical acceptance. Owner leases guarded checks.

Maintainability kept source transaction ownership in TorBox and browser revision
ownership in Stremio, shared deadline helper only for read-only lookups, no new
libraries/actors. Alpha0.1.7 preserved. Estimates75%playback/35%full beta unchanged.

### 2026-10-13 Cancel read-only lookups and fence late frontend results (source only)
Episode/source-option lookups now hold one watch cancellation owner; cancellation
or a replacing lookup drops pending work, with cancellation prioritized over a
simultaneously completed response. Channel identity ensures an older cleanup
cannot remove the newer lookup's cancellation owner. Cancel invalidates private
choice revision and returns retained browser titles. It does not cancel account
creation, source publication, current playback or reset pairing. Cancel lookup
appears only for the two read-only multi-provider actions. Existing60second total
lookup deadline and final revision/PC publication fences are retained.

Frontend action ordinals guard both success assignment and failure/browser refresh/
finally cleanup. A cancelled old request cannot clear a new busy state, restore
obsolete options/episodes or overwrite current errors. Failed cancellation retains
a retry affordance; this bug in the first new handler was reproduced by actual
handler test (false instead of true), then corrected. It is not an old Alpha bug.

12backend Stremio checks passed (11other Windows checks filtered), including pending
cancellation and old-cleanup/new-owner races.5 request-ordering checks execute the
actual App.svelte script transpiled by existing TypeScript, with deferred IPC and
plain state: late options, late error after a newer request, stale recovery browser,
late episodes and cancellation retry. These do NOT render Svelte/DOM or start Tauri.
Svelte static check zero errors/warnings. Initial harness microtask assumption
failure preserved separately; fixed harness waits one event-loop turn, not an app
fix. Exact logs/source hashes in work/source-validation/stremio-cancellation-
validation.json and referenced files. Source validation held native/control leases
for backend checks; no user app/emulator install, public tunnel or release package.
Actual rendered cancellation, real-provider cancellation and Android pair behaviour
remain unverified. Offered0.1.7 unchanged; planning75%/35% unchanged.

Maintainability: one bounded cancellation owner, existing revision ownership and
one component request ordinal; no app dependencies, new actors or reusable UI
framework added. scripts/Test-StremioUiRequests.mjs uses existing TypeScript only.

### 2026-10-13 Lookup cancellation owner survives normal/error/abort teardown
Quality review of the new cancellation path found normal-return-only cleanup.
Actual Tokio task.abort regression reproduced an owner left registered after the
future was dropped (baseline log retained). A private scoped LookupOwner now removes
only its own registered channel on drop. Normal success, lookup cancellation,
replacement, timeout and external abort all release ownership; a replaced owner's
drop still cannot clear the new owner's channel. This is one cleanup guard, not
an additional actor or registry.13focused Stremio checks passed (11other Windows
checks filtered). Actual async abort exercised, no real provider request, rendered
UI, APK/release EXE or Android test. Exact baseline/final logs and source hash receipt
work/source-validation/stremio-aborted-lookup-validation.json. Offered0.1.7 stays
unchanged; planning75%/35% unchanged. Ownership free after checks. This follow-up
corrects the new source cancellation path; it is not an Alpha0.1.7 failure claim.

### 2026-10-13 Catalog/search cancellation and discarded-resolution diagnostics
Catalog/search browsing now shares the existing read-only lookup deadline,
cancellation owner and scoped cleanup with episode/options lookup. The component
exposes Cancel lookup for catalog work too; late catalog titles cannot overwrite
the retained browser after cancellation. No duplicated cancellation mechanism,
new actor or dependency.6component-script deferred IPC cases passed (new late catalog
result case), Svelte static check0errors0warnings. Backend13Stremio checks passed,
11other Windows checks filtered. These remain non-rendered component logic and
synthetic provider-future checks, not real catalog HTTP/packaged UI acceptance.

A fetched TorBox transaction discarded at the Stremio revision fence now records
Publication/Replaced using the existing fixed enums, selection/generation and
elapsed-time fields, then drops its private URLs. No reason/URL-bearing message
or schema added. Coordinator regression proves discard invoked once and publish
zero times on replacement. Actual emitted journal row on that revised application
path was compiled/reviewed, NOT runtime verified. Exact logs and source hashes in
work/source-validation/stremio-catalog-cancel-validation.json. No new APK/EXE,
Android/native pair/public tunnel/device work. Offered0.1.7 unchanged, estimates
75%playback/35%full beta unchanged. Native/control leases respected; ownership free.

### 2026-10-13 Real-provider PC source transaction and discard journal proof
Added an explicitly ignored source-level proof; routine checks never read real
credentials or contact TorBox through it. Executed once under native/control leases
with existing encrypted PC vault and isolated synthetic room/diagnostic directory.
No credentials, URLs or filenames were printed or exported; existing labelled
private-Alpha link policy used only in PC memory. No Android recipient/player.

Run8ff11957-27e2-403f-a771-8a098c04ab0a PASSED at13:30:43UTC. Actual ready-provider
inventory, two real resolution transactions and separate direct1024byte range
checks for each role completed. First real fetched transaction was superseded at
the Stremio revision fence, produced one Publication/Replaced journal row, and
published no media. The next current transaction published the exact canonical
provider file through the real TorBox/Host handoff, producing one Publication/Passed
row. Journal export privacy assertion passed (no HTTPS URL/Bearer/filename text).
This closes the earlier unexecuted discard-journal/source-transaction boundary.

Exact receipt/log/typed journal retained in work/source-validation/real-source-
8ff11957-27e2-403f-a771-8a098c04ab0a; summary real-source-last-run.json. One ignored
proof explicitly executed,24other checks filtered; provider phase6.64seconds. No
provider create/delete, app installation, emulator, APK/release EXE, native decode,
phone playback, public control tunnel or two-hour gate. Direct HTTPS media range
reads from this PC do not establish WAN control or phone delivery. Offered0.1.7
unchanged. Planning75%playback/35%full beta unchanged. Ownership free after run.

### 2026-10-13 Completed source publication after PC command or shutdown
Added a focused ignored real-provider proof for pause, seek and shutdown arriving
AFTER both role ranges finish and BEFORE TorBox publication. It first installs a
real canonical source, then checks rejection leaves authoritative state and prior
source revision intact; shutdown must revoke both role capabilities. Routine tests
never contact TorBox through this proof. No public lease constructors or production
behaviour changes were added merely to make it testable.

Explicit run e31ca3af-4cae-4a01-96c1-cff4265efa76 FAILED at inventory retrieval,
before any completed provider transaction or publication. Provider phase44.92s;
fixed error only, no raw HTTP/URLs/credentials printed. Failure log and summary
retained in work/source-validation/authority-source-e31ca3af-4cae-4a01-96c1-
cff4265efa76 and authority-source-last-run.json. No retry of the unchanged earlier
successful provider proof. This is not evidence of a playback or publication bug.
The new actual TorBox command/shutdown handoff remains runtime UNVERIFIED.

Independent deterministic Host regression passed pause/seek/shutdown cases for a
fully prepared DIFFERENT replacement movie, extending prior refresh-only checks.
Old grants reject, pause/seek preserve current per-device capabilities and source
revision, shutdown revokes capabilities, and rejected publication leaves the whole
snapshot unchanged.8session-server media tests passed,79other unit/14transport
checks filtered. Synthetic capability strings only; no network/player/device.
Exact logs/source hashes and scope: work/source-validation/completed-source-
authority-validation.json. This validates existing behaviour, not a new fix.

Maintainability review: shared revision/generation/slot fences already handle this
boundary; no additional cancellation owner, new dependency or duplicated playback
loop added. Reviewed Windows single-flight status and Stremio request guards plus
Android idle cadence/local preparation ownership; no justified further change
claimed in that review. Native/control leases respected. No APK/release EXE,
GUI/emulator/provider creates/deletes or public tunnel. Offered0.1.7 unchanged;
planning75%playback/35%full beta unchanged.

### 2026-10-13 Shared TorBox read deadline and stage diagnostics
The existing25second HTTP limit applies per request. Current-file verification,
both role link requests and direct range probes can therefore accumulate roughly
75seconds; a100page inventory can wait much longer. Source now shares one60second
cooperative read deadline across these stages. Ready inventory is also bounded,
and copied-source fresh inventory consumes the same budget as subsequent source
resolution. An expired budget does not poll/start a later stage. Provider creates
and final atomic publication stay outside this read-only timeout. No automatic
create retry, extra cancellation owner, new dependency or delivery-policy change.
The deadline does not preempt synchronous work or guarantee a wall-clock UI bound
while waiting outside read stages; final authority/publication fences remain.

A timeout returns fixed retry feedback and records Failed for the active existing
source stage. Replaced remains distinct for obsolete selection/PC generation.
Dropping pending futures releases work; neither partial roles nor timed-out reads
reach publication, so existing host media/capabilities are unchanged. This addresses
an inspected cumulative-latency risk, not a reproduced live playback failure.

28Windows checks passed with2explicit real-provider proofs ignored. That suite
included existing muted hwnd0/native-PC lifecycle checks, NOT a paired native proof,
Android run or physical acceptance. Added final half-completed range timeout case;
6focused TorBox checks then passed,1ignored/24others filtered. Five new deterministic
checks cover success/error preservation, expired-stage non-polling, shared remaining
budget, pending future drop, no partial range pair, unchanged authority and actual
fixed diagnostic rows exported by the existing writer. These use synthetic futures
and shortened budgets; revised real-provider timeout/selection paths unexecuted.

Initial new helper compile error (reference pattern) retained, then corrected.
Exact compile/full/final logs, diagnostic journal and final source hash listed in
work/source-validation/source-deadline-validation.json. No unchanged provider
retry; prior e31ca3af inventory failure retained as FAILED. Current package hold
preserved: no APK/release EXE/GUI/emulator/public tunnel. Native/control leases held
for checks; ownership free afterward. Planning75%playback/35%full beta unchanged.

Maintainability review kept one small private deadline helper and one stage-outcome
helper. Existing Stremio lookup cancellation owner and Android cancellation-safe
HTTP/cadence paths remain separate owners of their own lifetimes; no broad rewrite
or new polling work. No extra app features added in this pass.

### 2026-10-14 Resume and prompt source cancellation when PC Host stops
Human resumed sustained development before sleep (approximately5hours), explicitly
reactivated the existing continuation automation, then lifted window/audio limits.
Visible interaction is now permitted when useful; no release rebuild was requested.
Existing Alpha0.1.7/package hold and credential/network boundaries remain. Automated
checks here remained hidden/muted by choice, not a requirement to ask for permission.

Host now exposes a cancellation-safe shutdown waiter using its existing watch:
subscribe-before-check covers already-stopped/setup races; no second owner/poll loop.
PC source read stages and copied-source inventory/credential/reservation waits select
that shutdown signal before polling work. Closing authority drops pending link/range
futures promptly instead of waiting for per-request/shared deadlines. Already stopped
source requests cannot reserve another selection. Creation is not wrapped in this
read-only cancellation; final Host generation/publication fences remain mandatory.
Normal source failure and shutdown use existing fixed stage Failed categories; no
raw reason/body/link/credential is journalled. No new schema or dependency.

9focused TorBox checks passed (1real-provider proof ignored/24filtered);9Host-related
shutdown unit checks passed (79filtered);1loopback shutdown transport check passed
(13filtered), including listener release/ticker stop/phone socket drain. New source
checks poll pending work before shutdown, assert dropped future ownership and unchanged
snapshot, actual stage Started/Failed journal rows with no Publication, already-stopped
non-polling/zero reserved selections, and safe cancellation followed by a new read.
Host waiter test covers two pending waiters, a dropped waiter and already-stopped
subscription. Tests use synthetic source futures; revised real provider cancellation
not run, no native pair/Android/physical phone/public tunnel. Related supervisor checks
use owned test children, not public cloudflared/WAN validation.

New tests initially lacked Future trait import (compile failure retained); corrected
then checks passed. Exact logs/journal paths/source hashes/scope receipt:
work/source-validation/source-shutdown-validation.json. This is a reviewed source
lifecycle gap, not a reproduced offered-Alpha playback failure. Existing failed
provider history preserved; Alpha0.1.7 unchanged; planning75%/35% unchanged.

### 2026-10-14 Bounded preparation preflight without cancelling transmitted creates
Cached preparation's account scan can span100pages, each with25second HTTP limits,
before creation. This source pass bounds only waiting for create ownership and the
read-only account/cache preflight with one60second deadline. An expired preflight
neither transmits nor marks a creation. Existing ready/pending account items still
reuse exact canonical identity, and no timeout resets prior attempt markers.
Once the marker is inserted and POST starts, the preflight timer does not cancel
that write or its follow-up identity read. Existing per-request HTTP limits and
uncertain-create/no-automatic-retry guard remain. No real account create/delete.

A private typed preflight-timeout category lets Windows show fixed timeout/retry
feedback instead of implying account limits. Other provider errors retain the
existing fixed preparation notice. No raw HTTP/key/link/error is surfaced. The
Windows notice path compiled; not rendered in a Tauri app in this pass.

6resolver preparation tests passed (36other checks filtered);13Stremio backend
checks passed (1real-provider proof ignored/20others filtered). New expired/ownership
checks assert no transmitted request/attempt marker. Actual loopback HTTP mock stalls
both inventory and cached-result bodies separately: timeout closes each connection,
no extra request is sent, attempt state remains empty, and later explicit reads
succeed. Mock POST response deliberately completes after the short preflight budget
and still returns Pending with attempt marker retained. Existing uncertain-create
regression confirms a malformed response cannot trigger repeat creation, while a
subsequently observed pending account item can be reused. These are synthetic
provider endpoints and shortened budgets, not real TorBox/phone/native acceptance.

Exact final/earlier logs and source hashes: work/source-validation/preparation-
preflight-validation.json. No library/schema/feature, APK/release EXE, Android,
public tunnel or physical-device test. Earlier failed real inventory run preserved.
Offered0.1.7 unchanged; planning75%playback/35%full beta unchanged. No app/playback
loop refactor; narrow read/write lifetime split retains existing create ownership.

### 2026-10-14 Android updater reuses cancellation-safe response ownership
Maintainability review found the existing updater still owned a separate OkHttp
callback while pairing/media grants/log uploads use shared awaitDecoded. Updater
now uses that response owner with its existing streaming active check, checks
coroutine ownership before and after decoding, preserves cancellation and maps
transport/response failures to fixed private-safe messages. Late cancelled responses
close before decode. A cancellation during decode cannot deliver an archive result.
Download bounds, SHA256/package/version/signer checks and Android confirmation remain;
no new updater feature, parser, library or release package was added.

Five focused JVM update HTTP checks passed: late-response non-decode/close, successful
streaming owner check/close, cancellation during decode, fixed200/503 error privacy,
transport exception privacy and late failure after cancellation. Actual updater APK
verification/install was NOT exercised. Main and instrumentation Kotlin compiled;
instrumentation NOT executed, no APK assemble/install/emulator/physical/native test.
Shared helper's prior real stalled-body check is historical, not rerun or counted as
new updater runtime validation. This pass's updater checks use synthetic Call/body.

Preserved first invocation's relative init-path failure. Initial JVM expectation
required a null cause, but coroutine stack recovery copies a SANITIZED IOException
into its cause; failure receipt retained. Corrected test asserts the entire trace
contains neither synthetic private URL nor secret. Product code was not changed to
hide that harmless coroutine cause. Final5tests/JUnit success and source hashes:
work/source-validation/update-http-validation.json, referenced immutable directories.
No claims of rendered updater, verified download, native pairing or physical gate.

Additional broader source validation after preparation changes: all42source-resolver
library checks passed including identity, URL/credential privacy, cached preparation,
role selection and actual loopback range metadata checks. Exact resolver-full.log
added to preparation-preflight-validation.json. These do not contact the real account.
Offered0.1.7 unchanged; planning75%playback/35%full beta unchanged. Leases respected.

### 2026-10-14 Update metadata deadline and exact release field types
Android metadata checks previously shared the15minute APK download call timeout.
Source now applies a12second per-call metadata timeout, using the existing owned
client/response helper; download calls retain the15minute allowance. This bounds
executing network/body waits, not dispatcher queue time or synchronous Android
package verification. No new client, polling owner, scheduler or dependency.
Actual loopback HTTP test sends valid headers and one byte of an incomplete body:
a shortened500ms per-call deadline aborts the call, fixed error feedback contains
no private detail, and the dispatcher becomes idle. Unit config check verifies only
the metadata call is shortened and the other call retains its long allowance.

Related review found UpdateRelease.parse used JSONObject numeric/string coercions.
The existing immutable model now lives in context-free UpdateRelease.kt with one
field parser used by the JSON adapter and JVM checks. It reuses MediaGrantNumbers
exact Int/Long handling; protocol/version/size strings, fractions, booleans and
missing values reject. Text fields require strings; existing URL, hash, version/
size bounds remain. No protocol/schema/signature policy change or new updater feature.
The risk was found by code review, not a reproduced offered-Alpha update failure.

7UpdateHttp +3UpdateRelease JVM checks passed (10total), main/instrumentation Kotlin
compiled. The field parser exercised valid integer limits and malformed types;
actual Android JSONObject adapter assertions compiled ONLY, instrumentation NOT
executed. No APK assembly/install/download/archive verification, native pair,
physical-phone/public tunnel or new release. Exact immutable logs/JUnit/source
hashes and limitations in work/source-validation/update-metadata-validation.json.
Earlier budget-only7check receipt retained; no failures in this pass. Alpha0.1.7
unchanged, planning75%playback/35%full beta unchanged. Ownership respected/free.

Maintainability: one shared cancellation-safe response owner, a per-call override
and one release parser. Reviewed PC source deadline/shutdown and Android request
ownership; no unrelated playback-loop optimization or dependency cleanup claimed.

### 2026-10-14 Explicit delayed authority delivery and stale-native watchdog
Added cfg(test)-only barriers/counters around authority delivery and the native
worker. They do not exist in application builds: no additional production polling,
actor, dependency or UI/debug route. Headless libmpv uses existing fixture, hwnd0,
vo/ao null and mute=yes. No new media generation or emulator/package installation.

Six buffering generations each hold the delivered snapshot back for30completed
native observations while feedback still reaches Host. The existing queued-state
gate emits exactly one buffering command per held generation (zero duplicates).
Controlled comparison replaces ONLY the buffer-report block with its historical
0066ea6 ungated form in current source/harness:175duplicates reproduced (counts
30,30,30,30,30,31), then source restored byte-for-byte. This is not a full historic
app build, and does not relabel prior12cycle baseline50bef0e0, which failed to
reproduce lag. Initial comparison compile failure from unused untyped gate variable
retained; corrected baseline removes those now-unused variables, not app behaviour.
The new test covers delayed authority delivery, NOT channel-Full queue backpressure.

A second explicit native-worker hold exposed current watchdog behaviour: while
native samples remain stale, feed emitted31buffering commands over30extra ticks,
then30more after a PC-authored DecoderFailed fault with buffering manually cleared.
That watchdog turned buffering back on despite terminal authority (fault itself
remained). Current source now checks selected/current room state, buffering and
fault before marking native unavailable once, and uses the captured generation
instead of an unfenced command. It skips idle/ended/faulted authority. Changed
state is refreshed before delivering to native. Existing media/fault semantics
remain; no recovery inferred from the watchdog.

Revised worker-hold case:1stale report,0additional fault reports, buffering=false
with DecoderFailed retained.9native player checks passed,27other Windows checks
filtered; includes delayed-delivery regression, existing lifecycle/idle/attachment/
source readiness cases. Non-test cargo check passed, including final cfg-only stop
recheck (no additional application-loop atomic read). Actual libmpv fixture/native
ownership exercised, but hold is a test barrier, not a reproduced JNI/GPU driver
hang or real decoder fault. No Android pair, provider/WAN/control-tunnel or phone
acceptance. Property-read timing uncertainty remains; no new150ms accuracy claim.

Exact failed comparisons, journals, final checks, receipts and source hash are in
work/source-validation/native-pressure-validation.json and immutable referenced
directories. Existing history retained; no APK/release EXE or offered Alpha changes.
Planning75%playback/35%full beta unchanged. Native/control leases respected/free.
Maintainability: fixed avoidable50Hz duplicate watchdog commands and made its fault
precedence consistent with native feedback; no broad rewrite or new app dependency.

### 2026-10-14 Capacity-one native queue rejects already superseded snapshots
Extended existing cfg(test)-only native hold with actual queued-generation/Full
counters and accepted native seek tracing. Real muted hwnd0 libmpv fixture test:
hold worker, enqueue1second seek, then PC Pause and2second seek while channel is
full. FiveFull attempts explicitly observed. Baseline submitted both old generation5
seek1000 and current generation7 seek2000 to libmpv; final position still reached
2000. Regression failed obsolete submissions1 instead of0, exact receipt retained.
Tracing captures successful native API submissions, NOT completion of the intermediate
asynchronous seek or an observed playback glitch. No sleep-only pressure claim.

Native dequeue now checkpoints epoch/generation/media against the existing Host
media binding before load/seek work. Superseded snapshot is dropped; Full retry still
delivers latest state. Fixed case:fiveFull attempts,0obsolete seek submissions,
1current seek submission, final paused position2000. Checkpoint is a short observation
and releases its lock before native calls; it does NOT atomically fence all native
calls against a command arriving afterward, or replace atomic source-grant delivery.
It compares references without cloning identity/epoch strings. No new actor/schema/
app dependency or public HTTP route. Test probes absent from non-test builds.

10native player checks passed/27others filtered.10final Host media checks passed,
80other unit/14transport filtered, including wrong epoch/identity, pause and shutdown
checkpoint rejection. Actual capacity-one source queue test covers both role grants
across pause/seek/reload/shutdown:obsolete grant callback never invoked; valid current
pause/seek grants retained; reload/shutdown expose no old capability. Those source
capabilities are private synthetic URLs, not a TorBox native-URL queue test or phone.
Non-test cargo check passed before allocation-only checkpoint comparison cleanup;
final Host10checks passed after cleanup. No need to relabel earlier delayed-delivery
barrier as channel-Full validation: this is a separate new exercised boundary.

Exact baseline/final logs, journals, typed receipts and final source hashes:
work/source-validation/full-queue-validation.json and immutable referenced folders.
No APK/EXE release, provider request, emulator/Android pair/WAN/physical acceptance.
Alpha0.1.7 unchanged; planning75%playback/35%full beta unchanged. Native/control
leases respected. Follow-up review should address continuous current-state authority
between deliveries separately; this patch rejects already stale queued generations
without claiming elimination of every command/property sampling race.

### 2026-10-14 Authority checkpoint after native observation, before controls
Reproduced a separate retained-state race in current source: dequeue accepted a
seek snapshot, then explicit cfg(test) barrier held worker after property reads.
PC Pause superseded it while feeder delivery was independently held. Releasing
worker submitted obsolete generation5 seek1000; thirty completed native
observations retained old generation while no new snapshot was delivered.
Baseline receipt/log preserved. This establishes an accepted obsolete seek
submission, not an observed visible glitch or intermediate seek completion.

Native control phase now checks the existing Host media binding again after
observations. A superseded state pauses if needed, resets rate if needed, clears
start/steady state and skips seek/start/drift/READY/execution feedback. Native
observations and journal timestamps continue, preserving the distinction between
waiting for authority delivery and a stalled worker. Pending decoder faults stay
owned by native until a current snapshot can report them. No lock spans native
API calls, no new actor/cancellation owner/dependency and no identity allocation.
Check is not atomic against commands arriving after it; do not claim all races
eliminated. Prior incoming queue/source capability fences remain intact.

Revised case: obsolete seek submissions0, thirty completed observations of the
retained old generation while paused, then new generation7 seek2000 accepted and
final paused position2000 verified. Eleven native player checks passed/27others
filtered; non-test cargo check passed on final production code. After adding an
explicit retained-generation assertion, final focused case passed/37others
filtered. Existing cfg(test) barrier/counters absent from application builds.
Initial test setup incorrectly expected Playing without a second participant;
that8second timeout is retained separately, not counted as a reproduced bug.
Corrected regression uses paused seek and real libmpv. Playing start/rate branch
under withheld delivery remains runtime unverified; new gate covers it in source.

Exact immutable logs, journals, before/after receipts and final source hash:
work/source-validation/current-authority-validation.json. No provider request,
Android/emulator test, WAN/control tunnel, APK/EXE release or physical gate.
Alpha0.1.7 unchanged; planning75%playback/35%full beta unchanged. Native/control
ownership respected and released. Both-app review: Android already invalidates
queued commands on accepted new generations; no copied PC checkpoint added there.
Removed duplicate command-failure consumption from the PC control phase. Added
one short binding check per native observation, no benchmark/latency improvement
claimed and no additional actor polling or dependency cleanup manufactured.

### 2026-10-14 Real Playing PC authority pressure with synthetic loopback phone
Added a separate cfg(test,windows) native regression module with authenticated
loopback WebSocket participant. It reuses the server transport test client:
Windows dev-dependencies declare already-locked tokio-tungstenite0.29/futures-util0.3.
No new resolved crate or application dependency; no test mutation API in Host.
The phone emits synthetic READY and Executed from received state (not media
observations). Each case first establishes actual muted hwnd0 libmpv Playing,
position>=50ms, and both participants' playing execution in authority, then uses
existing post-observation barrier and independently holds new snapshot delivery.

Pause, Seek2000 and Host shutdown all leave retained-generation native observations
Paused through30completed observations each. No old seek submission. Releasing
delivery after Pause permits explicit Play to resume; releasing after Seek resumes
at2000ms. Shutdown remains Paused until explicit controller teardown. Journals
export without test credentials/socket addresses; all cases io_errors0/drops0.
Normal new check passed on first run. Controlled comparison in current source
with ONLY post-observation checkpoint disabled reproduces Playing in all three
cases; it is NOT a full prior app build. Source bytes restored in finally and
revised run passed. Failed comparison and exact all-case receipts preserved.

12native player tests passed/27other Windows tests filtered; non-test cargo check
passed. This pass establishes real Playing/freeze/recovery runtime coverage for
c27b12f, not a new production bug/fix. Native rate stayed1.0: no injected non-unit
rate reset or all possible start timing race verified. Checks still release locks
before native calls and are not atomic over every later command. Synthetic phone
does not decode, so this is not Android pair/WAN/physical acceptance or a new150ms
claim. No TorBox request, emulator, APK/EXE release or offered Alpha change.

Exact logs, private-safe typed journals, first success, controlled failure and
final evidence: work/source-validation/playing-authority-validation.json and its
immutable referenced folders. Planning75%playback/35%full beta unchanged. Native/
control ownership leases held across compilation/tests and released afterward.
Both-app review retained Android accepted-generation command invalidation and
sample freshness gates; no speculative refactor/optimization or app dependency.
New socket helper is confined to tests and makes its synthetic execution explicit.

### 2026-10-14 Scheduled native START replaced before commit anchor
Extended existing cfg(test) probe with future-start observation and successful
unpause submission trace. Probe confirms native consumed a committed playing
snapshot while still paused and before its future anchor; existing barrier then
holds control and independently holds new delivery. PC Pause or Seek completes
before anchor (explicit completion timestamp assertion, not request time only).
Worker crosses actual anchor by completed observations, followed by30additional
observations while retained generation remains. No sleep-only timing claim.

Current source submits0obsolete unpauses, remains Paused in both cases and later
starts only current generation. Pause requires explicit Play; seek resumes2000ms.
Controlled current source with only post-observation checkpoint disabled submits
one obsolete unpause per case and both become Playing. Exact successful native
API calls are traced; this does not measure frame presentation or clock accuracy.
Initial comparison retained; tighter applied-before-anchor comparison also fails
with2obsolete submissions. Source bytes restored in finally. No new production
behaviour fix claimed: this verifies c27b12f for a distinct future-start boundary.

13native player checks passed/27other Windows tests filtered; non-test cargo check
passed. Native journals show0errors/drops. Test-only markers/counters absent from
application builds, no new dependencies. Existing synthetic socket phone supplies
READY/Executed without native media, so no Android/native pair, physical phone,
WAN/control tunnel/provider proof or new150msclaim. Offered Alpha0.1.7 unchanged,
no APK/EXE release. Planning75%playback/35%full beta unchanged. Native/control
leases respected/free; failure history preserved.

Exact all-case timestamps/start traces, logs, journals and source hashes:
work/source-validation/scheduled-start-validation.json and immutable referenced
folders. Maintainability review kept one existing binding check and reused native
barrier/socket helper; no additional production polling or speculative rewrite.
Android accepted-generation invalidation already protects queued controls; not
changed or retested here. PC native-fault/reload ownership merits focused follow-up:
review whether preparing a new transport generation can clear a pending native
failure before the PC records it. This is a review boundary, not a reproduced
failure; preserve source/grant revision safety and test evidence before changing it.

### 2026-10-14 Preserve pending PC native failure through transport preparation
Reproduced native-owned fault loss before authority records it. Real muted libmpv
fixture is loaded and ready; cfg(test) injects DecoderFailed at controller boundary
and existing post-observation barrier holds before feedback. PC Seek advances
preparation. Old reload condition reloads fixture once, clears native failure/error
and leaves PC player_fault absent. Original pre-fix failure retained. Expanded
seek/buffering comparison with old condition restored in current harness loses
failure in both cases (one premature reload each); source bytes restored finally.
Injection is NOT an actual decoder crash or a reproduced provider failure.

Same-source failure recovery now requires exact matching PC-recorded player_fault,
fresh prepare and no room fault. Transport generation/preparation alone cannot
reload away a pending fault. Native holds error and reports it with the new current
generation; actual media identity/source-revision replacement remains independently
supported. Existing grant final delivery fence unchanged. Once PC records the
failure, explicit Load creates authorized preparation, one native reload recovers
and PC READY is restored. No new owner, polling, dependency or schema.

Final seek and buffering cases each retain error/record DecoderFailed across60
completed native observations, zero premature reloads and exactly one explicit
recovery reload.14native player checks passed/27other Windows tests filtered;
non-test cargo check passed on final source. Existing full-queue, scheduled-start,
playing authority, watchdog, attachment, idle and lifecycle checks passed together.
No Android/emulator, revised TorBox native/provider/WAN or physical acceptance.
Alpha0.1.7 untouched; planning75%playback/35%full beta unchanged. Ownership leases
held during compilation/runtime and released; failed history retained.

Exact original/comparison failures, final logs, typed receipts/journals/source hash:
work/source-validation/pending-native-fault-validation.json. Maintainability review
confirmed Android already requires PC-recorded fault and authorized preparation;
no duplicated new Kotlin mechanism or unchanged JVM rerun. Recovery condition stays
local to native source handoff, sharing existing fault enum and authority snapshot.
Next useful evidence is the same pending-fault ordering with an actual native command
failure rather than injected DecoderFailed, using an owned test-only invalid command
and existing barriers; do not label it decoder/phone/provider acceptance.

### 2026-10-14 Actual native command rejection retains pending fault
Extended cfg(test) probe to issue an unknown command through actual Mpv.queue on
owning native thread. Native rejects it; existing production sticky-failure capture
sets CommandFailed and fixed error before held control/feedback. No fault enum is
injected in this case. Existing injected DecoderFailed regression now shares setup,
transport phase and recovery checks with command-failure regression; typed receipts
use expected_fault/authority_recorded_expected_fault to distinguish them. Earlier
receipts remain untouched with their original field names.

Seek and buffering each preserve actual CommandFailed across60completed native
observations, zero premature fixture reloads, PC records exact code, and explicit
Load recovers with one reload. Controlled current harness restoring ONLY previous
same-source reload condition loses actual fault/error in both cases, one premature
reload each and PC fault absent. Failed comparison/logs retained; source bytes
restored in finally. New case passed initially and in final regression run.

15native player checks passed/27other Windows tests filtered, including injected
DecoderFailed case after shared-helper refactor. Non-test cargo check passed. No
new production behaviour change this pass: native rejection now runtime validates
b986bae recovery gate. Invalid command is test-generated, not a user action or a
reproduced decoder/GPU fault. No Android/provider/revised TorBox/WAN/physical gate,
APK/EXE release or new dependency. Alpha0.1.7 unchanged; planning75%/35% unchanged.
Native/control ownership held for builds/runtime and released; failure history kept.

Exact native-rejection assertion, before/after receipts, typed journals, immutable
logs/source hash: work/source-validation/pending-command-fault-validation.json.
Both-app maintainability review retained Android fixed private-safe command-error
handling and recorded-fault recovery; no new Kotlin mechanism or unchanged tests.
Reused the existing native barriers and shared fault-phase helper rather than
copying another regression body. New trigger/ack fields are cfg(test) only.

### 2026-10-14 Shutdown join survives cancelled/deadline waits
App lifecycle review found controller allocated once, no in-process restart path,
and explicit PlayerStopFailed/ExitPermitted diagnostics already distinguish failure.
No unsupported app-restart bug claimed. Controller cancellation regression did
reproduce lost completion: existing native teardown barrier holds real libmpv
owner, first shutdown caller aborts, owner is released and reaches Stopped/PC
detached, but second call returns player_shutdown_incomplete. Baseline retained.

Lifecycle now retains one async completion task owning feed/native joins. Caller
awaits borrowed handle with existing5second deadline; cancellation or timeout drops
only the wait. A later caller can await the same completion. Successful/failed join
result cached only after actual completion; timeout remains a wait result, not an
irreversible cleanup verdict. Native join remains off UI on blocking worker, no
forcible native thread stop, resource transfer, new playback actor/poll or dependency.
App shutdown diagnostics remain honest if it exits after a deadline failure.

Cancelled caller case now returns Ok on second call after teardown. Actual deadline
case waits5001ms, returns player_shutdown_timeout while native remains owned, then
second call returns Ok after teardown. Controlled current harness with shutdown
method from3cb3fcc restores old behaviour: late call returns cached timeout even
though native is Stopped/PC detached. Exact failed comparison preserved and source
bytes restored.17native player checks passed/27others filtered, non-test cargo check
passed on same final source before byte-exact controlled comparison. Normal shared/
concurrent/idempotent shutdown and pending-fault/pressure tests passed together.
Teardown hold is an explicit test barrier, not an actual libmpv/GPU hang.

Immutable baseline/final logs, typed receipts/journals/source hash:
work/source-validation/shutdown-wait-validation.json. No Android/device/provider,
WAN/physical gate or APK/EXE release; Alpha0.1.7 unchanged. Native/control leases
respected/free; planning75%playback/35%full beta unchanged. Android close review
retained existing cleanup ownership after synchronous initialization; no speculative
Kotlin changes or unchanged JVM tests. Shared wait helper covers cancel/deadline
without copying setup; completion task starts only during shutdown, not playback.

### 2026-10-14 Bound queued shutdown callers by one total deadline
Controlled real native teardown hold confirms first shutdown caller owns lifecycle
mutex. Second future is explicitly polled Pending behind that owner. Cancelling
queued waiter already preserved owner and later completion correctly (no bug claimed
for that branch). Deadline branch reproduces10001ms wait: prior5s timer started
after lifecycle lock acquisition, so two waits composed to approximately10s.
Exact baseline retained; owner remained held until explicit test release.

One deadline is now established at shutdown entry and reused for lifecycle lock
acquisition and retained join completion. Queued cancellation/deadline still drops
only caller wait, not completion task or native resource owner. No forced native
termination, duplicate join task, added playback polling/dependency. Revised second
caller5000ms; both owner/waiter can time out honestly, then late call returns Ok
after actual teardown. Cached actual completion and previous cancellation recovery
remain intact. App-level stop-failure diagnostics unchanged.

18native player checks passed/27other Windows tests filtered; production cargo
check passed. Existing single-caller5s deadline, cancelled completion, shared/idempotent
shutdown, actual native command failure, transport/queue and scheduled START cases
passed together. Hold is test barrier, not reproduced libmpv/GPU driver hang. No
Android/provider/emulator/WAN/physical gate or APK/EXE release. Alpha0.1.7 unchanged;
planning75%playback/35%full beta unchanged. Native/control leases respected/free.

Exact baseline/final timestamps, logs, typed receipts/source hash:
work/source-validation/concurrent-shutdown-validation.json. Both-app review kept
Android PlayerCleanup independent of cancelled playback scope; no Kotlin fix or
unchanged JVM rerun justified. Small deadline reuse fixes composed waits without
new cancellation mechanism or broad lifecycle rewrite. Native join failure result
mapping remains distinct from timeout; do not claim actual crash validation here.

### 2026-10-14 Confirmed native join failure no longer reports live status
Added cfg(test) controlled Rust worker panic after real muted libmpv has been
safely dropped, before normal final status update. Actual std::thread join returns
Err; first and repeated shutdown already retained player_native_worker_failed
correctly (no result-cache bug claimed). Baseline status nonetheless remained
initialized=true/Paused/error=null although worker ended and PC detached; failure
receipt retained. This is test-generated Rust panic, NOT decoder/GPU/driver crash.

Lifecycle completion task now sets initialized=false, Failed and fixed private-safe
error only when blocking join confirms native std thread failure. Timed-out or
cancelled join waits do not establish release and do not take this branch. Error
result propagation/cache remain unchanged, no successful shutdown invented. Poisoned
status mutex does not trigger another panic in failure completion. No new app
actor/poll/dependency; status owner clone exists only in shutdown completion task.

19native player checks passed/27other Windows tests filtered; production cargo check
passed. Final worker failure status Failed/error fixed/initialized=false, both
shutdown results remain player_native_worker_failed and PC detached. Existing
cancel/deadline/queued ownership, pressure, scheduled start and pending-fault tests
passed together. App lifecycle source match records PlayerStopFailed for this Err;
actual rendered Tauri close/journal path not invoked here. Normal alert already
renders player.error; no frontend changes or unchanged Svelte checks needed.

Exact immutable baseline/final logs, typed receipts, journals/source hash:
work/source-validation/native-join-failure-validation.json. No Android/provider,
emulator/WAN/physical acceptance or APK/EXE release. Alpha0.1.7 unchanged; estimates
75%playback/35%full beta unchanged. Native/control leases respected/free. Android
cleanup exception handler already reports fixed failures; no speculative Kotlin fix.
Temporary connection lifecycle review found startup publication and stop serialized
under same owner mutex; no additional justified change or public tunnel attempted.

### 2026-10-14 Validate Android playback snapshot fields before effects
Review found follow loop already catches invalid state and loses control safely;
no uncaught-loop bug claimed or reproduced. Acceptance nevertheless left timeline
position/anchor and prepare values to later coercive JSON reads, after generation,
loaded state and native/loading effects could change. Added once-per-accept strict
policy reusing MediaGrantNumbers: nonnegative exact integer timeline milliseconds,
real playing/desired booleans, prepare generation matching snapshot and complete
nonnegative position/deadline. Ready array is bounded to two unique pc/phone actors.
Existing valid protocol fields remain; no wire fields/schema/library added.

Checks run after ignored-stale-revision return but before media/load/native mutations.
Accepted prepare JSONObject is reused for diagnostics; no repeated checks/parsing
added to playback ticks. Bounded ready validation avoids temporary actor/distinct
lists. Existing follow exception handling/cancellation and loss pause preserved.
This is a code-review validation gap, not a reproduced live malformed-control fault.
Long.MAX_VALUE helper tests establish representation boundaries, not native target
usability or clock accuracy. Unknown extra fields remain outside this focused guard.

Four playback-field JVM tests plus two existing integer-helper JVM tests passed
(six total, no failures). Main/instrumentation Kotlin compiled. New synthetic
JSONObject acceptance assertions verify rejection before generation/loaded changes
when eventually executed, but instrumentation NOT executed here. Existing valid
synthetic TorBox prepare updated with complete position/deadline fields. No APK,
install/emulator/socket/native/phone/WAN validation or provider requests. Offered
Alpha0.1.7 unchanged; estimates75%playback/35%full beta unchanged; owners free.

Immutable logs, exact JUnit copies, final source hashes and limits:
work/source-validation/playback-snapshot-fields-validation.json. Final compile after
ready-loop allocation cleanup succeeded; unchanged pure helper tests not rerun.
Both-app review notes Rust typed snapshot already rejects coerced booleans/integers;
no duplicate Windows guard or unchanged native suite needed. Helper remains small,
uses existing integer rules, and avoids broad DTO/controller rewrite.

### 2026-10-14 Exact Android canonical media IDs before acceptance effects
Review corrected the proposed boundary: sameMediaIdentity already compares every
field exactly (including metadata), without numeric coercion; no tuple-only reuse
bug claimed. Snapshot acceptance still used getLong, allowing string/fractional
IDs through its provider policy before loaded/generation/native effects. Replaced
that policy with shared context-free exact Int/Long guard: fixture zero/zero,
TorBox positive torrent/nonnegative file, actual known provider string. Missing
provider retains existing legacy fixture default, consistent with Rust; explicit
null no longer masquerades as fixture. This is code-review evidence, not a live
malformed-frame reproduction. Long.MAX_VALUE tests representation only.

Grant request identity and response now reuse the same guard before exact
all-field equality and URL handoff; errors remain fixed Media access unavailable.
No weakening to tuple equality, no change to source revision/generation/epoch,
cancellation, native pending-result/recovery or direct-delivery fences. Optional
metadata is still compared exactly, not claimed fully parsed by this focused guard.
Four focused JVM policy tests passed, no failures; main and instrumentation Kotlin
compiled. Synthetic actual JSONObject controller effect-fencing and adapter/equality
assertions compiled ONLY, not executed. No APK/install/emulator/socket/native pair,
provider/WAN/physical gate validation. Offered Alpha0.1.7 unchanged; estimates
75%playback/35%full beta unchanged. Exact immutable log/JUnit/source hashes:
work/source-validation/media-identity-validation.json. Native/control leases free.

Both-app review found Windows typed canonical IDs already reject string/fractional
wire values; no duplicate Rust guard or unchanged native suite justified. Android
sameMediaIdentity retains full-field comparison used by capability and source reuse;
small shared provider/ID helper adds no library/schema, retained URL/key or playback
tick work. Legacy fixture loading still uses the same default provider. Remaining
snapshot participant/execution fields are not covered by this canonical-ID guard;
review their validation before effects separately rather than claim a full parser.

### 2026-10-14 Validate participant effects and match pending recovery fault
Android acceptance previously read participant/execution data after generation,
loaded/native and reconnect effects; execution getLong could coerce types. Both
pc/phone now checked once before effects: actual connected/buffering flags, exact
nonnegative RTT and execution generation/position/observation/receipt integers,
actual execution playing, optional known player-fault code. Missing/null optional
fault/execution remain accepted. Older execution generations remain valid metadata;
only current generation counted, count reused without later JSON reads/actor list.
No per-tick participant validation, new wire field or broad DTO rewrite. This is
code-review evidence, not live malformed-frame reproduction/full snapshot parser.

Recovery previously required any nonnull recorded phone fault, unlike Windows exact
native-code match. Shared small PlayerFaultAuthority now requires known exact pending
native code plus existing fresh attempt/prepare/no-room-fault conditions. Acceptance
and follower use same policy. TorBox after network/native settle and local fixture
after IO recheck actual pending fault against latest accepted state before load
submission. Already admitted load owns its attempt; final recheck does not require
second attempt. Generation/epoch/socket/cancellation/source-revision fences retained.
This still is not atomic across native worker submission and not Android fault-race
runtime acceptance. Changed pending code cannot inherit another code's permission.

Four participant/execution plus four recovery JVM checks passed (eight distinct).
Controlled CURRENT pure helper comparison with old nonnull predicate failed three
of four recovery tests: cross-code matrix, invalid recorded fault, changed result.
Baseline log/JUnit preserved, helper restored byte-exact; final four recovery checks
passed again and main/instrumentation compiled. No whole-old-app comparison or
actual native decoder failure claim. JSONObject controller effect-fencing assertions
compiled ONLY, not executed. Final comment/EOF-only cleanup changes no behavior.
Exact logs, immutable JUnit copies, source hashes and limits:
work/source-validation/participant-authority-validation.json.

Both-app review confirmed existing Windows typed participants and exact pending
fault gate; no duplicate Rust change or unchanged native suite. Known fault codes
mirror existing wire enum, no app dependency/schema/actor or polling added. No APK,
emulator/install/native pair/provider/WAN/physical gate test or release change.
Alpha0.1.7 preserved; estimates75%playback/35%full beta unchanged; owners free.
Remaining epoch/ended/room-fault JSON reads still deserve focused acceptance review;
current guards are not claimed a complete snapshot validation contract. Do not
infer rendered usability or physical acceptance from this source compilation.

### 2026-10-14 Snapshot envelope types checked before Android effects
Code review found epoch getString coercion and ended/fault reads deferred until
after loading/generation effects. Epoch now requires actual String, without adding
UUID/length/nonempty restrictions absent from existing typed Rust protocol. After
ignored-stale-state return, ended must be actual Boolean (absent retains wire false
default; explicit null/string reject), room fault optional String (absent/null
retained, no invented fault enum). Accepted values reused in diagnostics. Guards
use existing small PlaybackSnapshotFields helper, no DTO/wire change or tick work.
This is source review, not live malformed-frame reproduction or full parser claim.

Four focused envelope JVM checks passed; main/instrumentation Kotlin compiled.
Actual JSONObject controller assertions for rejection before effects, legacy
missing-ended/fault defaults, and older malformed-terminal state ignored compiled
ONLY, not executed. Initial PowerShell command parse error occurred before any
edits/builds; corrected invocation succeeded; orchestration history retained.
Exact immutable log/JUnit, source hashes/limits:
work/source-validation/snapshot-envelope-validation.json. No APK/install/emulator,
Android socket/native/device/provider/WAN or physical acceptance. Alpha0.1.7
unchanged; estimates75%playback/35%full beta unchanged; owners free.

Both-app review confirms Windows serde already enforces these types/defaults; no
Rust churn/unchanged native suite needed. Assessed deferred Android session handoff:
production path directly needs Context/libmpv and JSONObject. A duplicate pure-JVM
flow repeating PlayerFaultAuthority would not execute it; no new JSON library,
public injection seam or broad coordinator refactor added merely to claim coverage.
Grant/fixture handoff runtime remains unverified. Adapter loadSource worker checks
source revision, but its asynchronous native commit occurs after Main's fault
authority check; do not claim that interval atomic. Existing beforeSourceLoad test
barrier may support a future focused worker ownership regression; APK hold remains.

### 2026-10-14 Android source worker retains newly pending fault
Review found queued loadSource checked source revision, then unconditionally
cleared failure at native acceptance; a new Main-classified fault while queued or
executing could be lost. Small production NativeSourceLoadFence now checks source
revision and admitted fault value before native submission, including after existing
proof barrier. Completion uses MutableStateFlow.update/CAS and clears only its
matching revision/fault. Native accepted-source receipt remains an observation even
when fault/authority changed, without inventing readiness. Source request publication
also uses update so it does not overwrite concurrent status changes.

Session passes exact fault value already checked against PC recovery authority to
local/TorBox adapter handoff; adapter must not recapture a newer fault as permission.
Existing one-argument explicit proof loads remain compatible; internal overloads
carry this value, no wire/schema or public injection mechanism. PC generation/epoch/
socket/cancellation/source revision fences retained. Fault CODE ownership, not fault
event identity: repeated same-code events are not individually tracked. Native calls
remain outside any authority lock; not a claim all native submission races solved.

Four JVM tests passed using real production policy/PlayerStatus/MutableStateFlow and
explicit coroutine pending-completion barrier; native command completion synthetic,
no JNI calls. Controlled CURRENT helper old revision-only/unconditional-clear policy
failed all four tests; baseline log/JUnit retained, bytes restored. Final four tests
passed again; main/instrumentation compiled. New native instrumentation holds existing
beforeSourceLoad barrier, calls actual controller-classified unexpectedEnd (not a
decoder crash), asserts retained fault/no native acceptance then explicit recovery.
This new native test COMPILED ONLY, NOT executed: no APK/emulator/install permitted
by package hold, no Android native ordering/usability verification claimed.

Exact immutable logs/JUnit/final source hashes/limits:
work/source-validation/native-source-fence-validation.json. No new app dependency,
actor/schema/playback polling or URL/key retention. Both-app review confirms Windows
worker keeps its fault locally and already requires exact recorded code; no duplicate
Rust abstraction or unchanged Windows native suite justified. Android load helper is
small and used at actual submission/completion, not a duplicate JVM session flow.
Alpha0.1.7 unchanged; no provider/WAN/physical test or new release. Owners free;
estimates75%playback/35%full beta unchanged. Native acceptance/fault preservation under
actual revised Android worker remains a future runtime boundary, not satisfied here.

### 2026-10-14 Actual native EOF cannot submit feedback after newer PC authority
Added cfg(test) barrier armed at first actual native eof-reached observation during
committed playing, before control/feedback checkpoint. Uses existing protected
125minute fixture, PC-authoritative near-end seek then natural final second playback;
no fixture regeneration or injected EOF/fault enum. Synthetic authenticated loopback
phone sends READY/Executed without decoding. Source PC/headless libmpv muted.

First attempt held a pre-buffering generation; normal buffering advanced4->5 and
actual EOF report completed at5 before barrier, so8s arm timed out. Test setup
failure, not application bug. Exact log/journal/source/history receipt preserved.
Arming first actual EOF rather than fixed generation corrected setup. Actual sampled
position7499900ms/duration7500000ms, then PC Pause or Seek2000 completed with new
authority delivery held. Thirty completed native observations each retain old sampled
generation, current native Paused, zero stale Ended submissions. Both later resume
Playing at2000ms after current delivery; PC ended=false/fault=null, zero journal drops
or I/O errors. This tests superseded EOF, not automatic replay through accepted END.

Controlled CURRENT source disabling ONLY post-observation checkpoint submits four
stale EOF feedback messages (two each). Host independently rejects all four with
stale_generation; PC new generation remains safe in comparison too. No newly
reproduced PC playback bug/fix claimed: validates existing checkpoint and reducer
fences, not full historical app. Exact failed comparison receipt/log/journals retained,
player.rs restored byte-exact. cfg(test) barriers/counters absent application build;
feedback success trace observes accepted queue submissions, not frame presentation.

Final20native player checks passed/27other Windows tests filtered; non-test production
cargo check passed. Exact source hashes, initial failure/corrected run, comparison,
final typed receipt/journals/check logs:
work/source-validation/eof-authority-validation.json. No provider/new APK/EXE,
Android emulator/revised worker runtime, WAN or physical phone/two-hour gate. Offered
Alpha0.1.7 unchanged; estimates75%playback/35%full beta unchanged; owners free.

Both-app maintainability review kept test probes cfg(test), reused existing native
barrier/feed/phone helper, no new app dependency/schema or playback polling. Android
new load-fault fence remains source/JVM policy validated, native instrumentation
compiled only; no unchanged Android suite rerun. No extra cleanup or provider retry.
A distinct future fault boundary is actual premature native EOF while PC timeline
still expects playback (controlled native reposition if justified, clearly separate
from decoder crash); existing exact fault/recovery fences must remain preserved.

### 2026-10-14 Premature actual EOF preserves native fault owner and reports failure
After actual muted native Playing, test-only native seek jumps the existing125minute
fixture to its end without moving PC timeline. Actual eof-reached/position7499900ms
observed while PC timeline129/133ms; no injected EOF/fault enum. This is test-induced
native position discontinuity, NOT decoder/GPU crash or reproduced normal user failure.
Synthetic authenticated phone controls never decode. Native UnexpectedEnd classification
and explicit recovery initially worked, but status misleadingly Buffering/error absent.
Initial test mistakenly expected Paused; full failure/log/receipt preserved, do not
claim unsafe playback from that assertion. Revised status prioritizes terminal native
Failed and sets fixed Native media ended unexpectedly feedback. Existing UI alert/status
source renders these fields; actual Tauri screen not rendered/verified in this pass.
Controlled CURRENT old status/error comparison fails Failed assertion; history retained.

Further actual command rejection at the real EOF observation produced CommandFailed
via capture_command_failure before feedback. Existing EOF classification overwrote it
to UnexpectedEnd: exact before-fix receipt shows PC received unexpected_end instead
of command_failed. New classification requires no already pending native failure,
retaining exact terminal owner/error. Revised command_failed and standalone unexpected_end
both PC-recorded, zero Ended submissions, thirty observations hold each, no premature
reload, then exactly one explicit Load reload and actual native Playing100ms restored.
Zero journal errors/drops. No source/grant/generation/recovery relaxation, no new app
polling/library/schema; extra discontinuity/rejection probes cfg(test) only.

22native checks passed/27other Windows tests filtered; non-test cargo check passed.
Earlier21checks passed after status fix before separate owner finding; final22supersedes
that boundary. Exact initial/status-comparison/command-owner failures, current source
hashes and final native receipts/journals/logs:
work/source-validation/premature-eof-validation.json. Classifier code change is real
fault-ordering correction under test-induced native inputs; not decoder-crash coverage.

Android review found analogous unexpectedEnd unconditional status assignment. Small
pure PlayerStatus.withUnexpectedEnd plus MutableStateFlow.update retains worker-published
terminal code/error rather than overriding from earlier follower snapshot. Three JVM
checks passed including deferred coroutine/CAS-policy ordering; controlled old unconditional
helper fails two, bytes restored, exact JUnit/logs retained in same receipt. Main and
instrumentation compiled ONLY; no Android/native race/device test executed. No duplicate
JVM session flow/public injection library. Both apps keep EOF interpretation subordinate
to pending fault ownership, without general fault-counter/DTO refactor.

No APK/EXE release/provider/emulator/WAN/physical test, Alpha0.1.7 unchanged; estimates
75%playback/35%full beta unchanged; owners free. Playback safety/recovery worked in
status comparison too; distinguish presentation fix from later native owner overwrite.
No repeated unchanged cleanup/provider proofs. Runtime revised Android worker/physical
and public-tunnel gates remain open with previously stated restrictions.

### 2026-10-14 Real queued EndFile errors remain attached to native entry
Reviewed both apps: Windows EndFile source_failure already compares playlist entry
ID, Android compares entry ID plus active/current source revision on its native
worker. No additional production fix justified. Added cfg(test) typed reason/error/
entry observation to Windows adapter, absent application builds; no raw native
messages, paths or URLs copied. Actual muted native format/load error from isolated
26byte invalid-local-format.mp4 retained in unique validation folder, never edited
protected125minute fixture. Confirmed stable native idle without draining event
queue, replaced invalid entry1 with valid fixture entry2, then polled queued error.
Actual old event reason ERROR/code-17/userdata0 classified source_failure=false and
replacement source_ready=true. Current invalid entry3 error is still true. No
EndFile error enum injected; this is format/load rejection, NOT decoder/GPU crash.
Native userdata0 on both errors is not a session protocol generation; entry identity
is the demonstrated discriminator. No controller/host/phone acceptance claim.

Controlled CURRENT source removing only entry-ID comparison wrongly attributes old
error to replacement (one false source failure) and fails assertion; actual current
code already correct. Comparison log/receipt preserved, bytes restored. Final8adapter
checks passed (native command rejection/entry ownership plus URL/timing policies),
non-test adapter cargo check passed. Final test-receipt formatting readability cleanup
followed by focused1pass/7filtered; unchanged suite not rerun. No app libraries added
for JSON receipt, uses fixed template and numeric/boolean fields. Exact logs/typed
receipts/source hash/scope:
work/source-validation/end-file-ownership-validation.json.

Both-app maintainability review left matched-entry guards in place; test observation
state cfg(test) only, test source errors isolated, small helpers proportional. No
Android change/build/test or unchanged Windows22suite rerun required for test-only
adapter observation. No new release APK/EXE, provider/emulator/WAN/physical test,
extra cleanup, public tunnel or credentials used. Alpha0.1.7 unchanged; owners free;
estimates75%playback/35%full beta unchanged. This validates a previously unexercised
native queued-error boundary without claiming a new bug/fix or rendered usability.

### 2026-10-14 Nonunit native rate resets before authority delivery
Extended existing real muted Playing authority cases through a shared helper,
avoiding a second session/phone/recovery harness. Test-only correction state admits
actual successful libmpv speed commands at0.995 and1.005 before the existing native
control barrier. This forces the local rate state, does not reproduce a drift
estimator failure. PC Pause/Seek2000/Host shutdown completes while newer snapshot
delivery remains withheld. Six cases each trace exactly one successful native
speed1.0 reset, retain old generation, become Paused/rate1.0 for thirty completed
native observations, and preserve later current-authority recovery. Zero journal
errors/drops. Reset trace records successful API return; native speed property not
read, no diagnostic allowlist/product API added. Not frame presentation, effective
speed measurement, clock accuracy or a new150ms sync result.

Controlled CURRENT source disabling only post-observation authority checkpoint
failed all three0.995 cases: no reset trace, retained Playing/rate0.995. The helper
stops after that failed rate; comparison1.005 not executed. Comparison is not whole
historical app. Failed log/receipt preserved; player source restored byte-exact.
Existing guard already correct, no production behavior fix claimed. Small speed
result binding allows cfg(test) success trace; identical non-test behavior retained.
All new probe fields/forced rate reads absent application builds, no extra playback
polling/dependency/schema. Initial focused1 passed; final23native checks passed,
27other Windows tests filtered; non-test Windows cargo check passed. Exact source
hashes/logs/native receipts/journals and comparison boundaries:
work/source-validation/nonunit-rate-validation.json.

Both-app review: Android generation change/control loss already invalidates queued
commands then submits pause/rate1.0; steady correction emits changed rates. Reviewed
only, no Android change/unchanged JVM rerun or runtime claim. Windows harness reused,
no broader abstraction needed. No provider retry, APK/EXE packaging, emulator,
Android pair, WAN or physical test. Alpha0.1.7 unchanged; estimates75%playback/
35%full beta unchanged. Nonunit reset task complete; natural estimator admission
and effective native speed/frame timing remain outside this evidence.

### 2026-10-14 Control ACK ordering and connection sequence ownership
Reviewed Android control ownership: one in-flight sequence, latest waiting user
intent only, no automatic command retransmission. Eight-second ACK deadline is
independent of six-second control silence. Socket identity fences old callbacks;
accepted stale-state return precedes liveness/ACK/flush; matching current sequence
alone clears a receipt. Generation changes invalidate native commands. No demonstrated
retry/ACK ownership bug, no Android policy extraction or production change justified.

Server socket loop serializes ACK and snapshot sends; ACK captures current session
state under authority lock before sending. Added focused actual loopback regression:
await PC Seek before submitting queued old-generation phone Pause, interleave valid
current-state Pause, inspect every intervening snapshot and assert each ACK revision
is no older than maximum accepted socket snapshot. Two connections, twelve commands
each: eleven stale-generation rejections and thirteen accepted commands; reconnect
explicitly accepts sequence1 after old connection accepted sequence12. All replies
have exact submitted sequence/same epoch; rejection retains PC seek preparation.
Phone disconnect awaited before replacement socket, then Host shutdown/server join.
Bounded reads, fixed numeric receipt, journal/export privacy checked, zero errors/drops.

Initial focused1 passed; tightened reconnect-sequence1 acceptance case focused1
passed again, fourteen unchanged transport tests filtered. No failure comparison or
new fix claim: existing guards correct for this exercised ordering. Exact receipt,
logs/source hashes: work/source-validation/ack-order-validation.json. This is SOURCE
server control behavior with synthetic client, NOT actual Android SessionController
socket execution, decoder/native pair, two-hour physical gate, WAN or arbitrary
network-fault coverage. No new app dependency/schema/polling, release APK/EXE,
provider request, emulator, public tunnel or unchanged native/JVM proof rerun.
Windows control publication reviewed unchanged; Android source-only ownership review
keeps latest-intent mechanism small. Alpha0.1.7 unchanged, owners free; estimates
75%playback/35%full beta unchanged. Do not manufacture an outbox refactor from this
review or claim source tests establish delivered-app usability.

### 2026-10-14 Source request admission shares existing busy ownership
Source-selection review found TorBox select/refresh/list buttons only disabled by
TorBox busy, while Stremio watch/preparation/import owns separate busy state.
Actual component-script deferred IPC reproduced two source submissions during a
pending Stremio watch, including enabled TorBox select path; no provider/native
publication corruption reproduced or claimed. Both handlers could later write
shared title/error/busy presentation. Source-changing handlers now reject admission
when either existing busy flag is held; corresponding TorBox buttons match that
rule. Ready-video inventory also avoids competing busy ownership during pending
source operation. Blocked copied-source handler leaves pasted input untouched.
This serializes UI admission, does not cancel any transmitted provider creation,
change backend cancellation owners, grant/publication authority or playback controls.

Existing actual component test harness extended:25pair combinations across five
source-changing handlers, blocked import preserves input/title, five failed request
then explicit retry cases, five inventory-during-source cases;36new focused checks
plus6existing metadata-order checks passed. Retry import explicitly repastes input
as normal failed import clears submitted private source. No duplicate Svelte flow
model, DOM/Tauri rendering, provider request or native player started. Svelte check
zero errors/warnings; final test-only retry-input adjustment passed all42again,
unchanged component compilation not rerun. Initial admission failures retained.
One Python default cp1252 decode failed before component edit; failed validation
still observed old admission, preserved; explicit UTF8 edit corrected it. Exact
logs/source hashes/boundaries: work/source-validation/source-admission-validation.json.

Backend review retains create-attempt safety, read-only lookup cancellation separate
from transmitted preparation, final browser revision/Host source publication fences.
No Rust change/unchanged backend/native proof rerun needed. Android has no Stremio
selection UI; no analogous source-choice admission code added there. Existing busy
flags suffice; no new coordinator/dependency/schema or playback-loop work. No APK/
EXE packages, emulator, provider, WAN or physical test. OfferedAlpha0.1.7 unchanged,
owners free, planning75%playback/35%full beta unchanged. Source handler reproduction
is not a live rendered UI/provider usability acceptance claim.

### 2026-10-14 Pairing claim/save and clear share setup admission
Persistent-pairing review: PairingRestorer already cancellation/revision fenced,
PairingStore already shares AtomicFile/Keystore disk lock. Original Activity forget
cleared sessionMode before dispatching encrypted store clear, exposing enabled new
code entry while clear still pending; separate claim/save job could overlap old
clear. This is source-review admission gap, NOT actual device encrypted-file race
reproduction. Store clear exception also escaped setup coroutine without feedback.

Small Main-serialized PairingSetup gate now owns both claim/save and clear/disconnect
through suspend waits. Existing setup scope/cancellation remains; StateFlow busy
replaces Compose-local flag, no parallel owner/thread claims. Saved connection clears
before controller disconnect; clear failure retains active connection and displays
fixed private-safe retry notice, also visible while session UI stays active. Change
connection/confirmation disabled during setup. No key/token/log/schema changes.
Gate is Activity-local, not a cross-process persistence transaction or synchronous
IO preemption; PairingStore shared lock remains unchanged. Cancellation retains
ownership until non-cancellable work completes then finally releases admission.

Three actual gate JVM coroutine checks passed: held clear rejects concurrent claim/
save, failure releases for explicit retry, cancelled owner blocks replacement until
held non-cancellable work finishes. Controlled CURRENT helper removing only admission
check fails focused overlap case; this is not whole old Activity/store comparison.
Failure/log/JUnit preserved, helper restored byte-exact; final3passed again. Main and
instrumentation Kotlin compiled ONLY before comparison, unchanged restored helper
matches compiled source; no instrumentation executed/APK assembled or installed.
Exact hashes/logs/JUnit/boundaries: work/source-validation/pairing-setup-validation.json.

Both-app review: Windows existing pairing occupant checks/reservation remain; no
new pairing protocol/cancellation mechanism or dependency needed. Android gate only
runs setup actions, no playback polling/work added; existing restore/read/HTTP tests
not repeated. No emulator, provider, native pair, WAN, physical or new release EXE/
APK test. OfferedAlpha0.1.7 unchanged; owners free; estimates75%playback/35%full beta
unchanged. Rendered disconnect feedback and actual Keystore operation remain pending
future allowed Android runtime validation; do not claim source compile is usability.

### 2026-10-14 Native observation API latency and position reporting cadence
Reviewed Windows and Android: each timestamps position after its native read.
Android reads pause/seeking/cache/EOF/duration before position and entry ownership
after; Windows reads duration before position and other flags after. Multiple
property reads are not an atomic presented-frame snapshot. No source timestamp
bug demonstrated; cached native-result age still unknown. Existing proof mode
position_read_ms would overwrite shared output, so added cfg(test,windows) isolated
adapter measurement without enabling global proof mode or new product properties.

Real existing125minute fixture, vo/ao null and explicit player mute, native position
advanced before measurement;500position/duration reads separated by10ms test sleep.
Numeric monotonic start/post-position timestamps and call durations saved to owned
CSV. Position median0.001ms/p950.003ms/max0.0712ms; duration median0.0073ms/
p950.0166ms/max0.3516ms. Actual position100to5300ms, zero backwards samples.
53distinct position values; each positive step100ms, longest observed same-value
interval94.5862ms. This is fixture/headless API reporting cadence, NOT a general
native precision guarantee; fast getter does not establish fresh internal PTS or
frame-presentation timestamp. No clock/interpolation/150ms accuracy improvement
claim, hard driver/API deadline or Android latency measurement.

Focused adapter1passed/8unchanged checks filtered; non-test adapter cargo check
passed. No production behavior fix/new dependency/diagnostic allowlist/schema or
playback polling. Test waits native readiness/actual advance, records latency without
flaky machine-speed acceptance thresholds. Exact CSV/native receipt/logs/source
hashes: work/source-validation/position-latency-validation.json. Both-app review
keeps existing position owner and matched-entry guards; no optimization warranted
by submillisecond calls in this run. Do not repeat this unchanged measurement as
acceptance or assume other providers/GPU/phones share it. No APK/EXE/provider/
emulator/controller/native pair/WAN/physical test; Alpha0.1.7 unchanged; owners
free; planning75%playback/35%full beta unchanged.

### 2026-10-14 Fresh observation timestamp and repeated PTS policy boundary
Followed native latency measurement with current source review, without repeating
that native experiment. Worker timestamp freshness proves observations, not decoder
progress. Source-ready still requires matching native entry/revision/readable duration/
position; it does not measure presented frames. Windows native watchdog detects
missing worker updates; Android unavailable includes stale sample/clock/source state.
No justified new decoder-hang detector or changed thresholds from100ms fixture.

Added three focused regression inputs to both actual Rust/Kotlin drift controllers:
100ms quantized positions with fresh10ms reads at phases0/25/50/75/99, frozen position
while timestamp/target advances, and repeated position while paused or clock expired.
Five coarse traces never request seek/slowdown and each emits at most two subtle
rate changes; this is synthetic input-policy behavior, not closed-loop native/frame
convergence. Frozen trace first authority Seek strictly after5000ms/by5200ms under
existing lag window; no new native fault classification. Paused/expired clock reset
rate1.0, never lag seek. Existing duplicate-timestamp guards and generation changes
remain. Correction Seek is an authority request, not direct native seek.

Seven Rust drift_time checks and seven JVM DriftTimingTest checks passed, three new
per language/eight existing file checks included. No production files changed,
no whole native/Android suite/instrumentation/device run. Exact logs/JUnit/source
hashes/boundaries: work/source-validation/position-policy-validation.json. No
sourceReady/controller feedback integration, actual stuck decoder or GPU driver
hang, frame accuracy/150ms/physical acceptance claim. Mirrored policy fixtures are
necessary for two independent ports, not a new app abstraction/dependency/poll.

Roadmap docs/ROADMAP.md was added at f7784fb after human request; package hold is
unchanged. Candidate source consolidation remains phase1; revised runtime and
physical gates are the highest-value next acceptance evidence when authorized.
Do not add repeated isolated proofs as a substitute. No APK/EXE/provider/emulator/
WAN/physical test, estimates75%playback/35%full beta unchanged, owners free.

### 2026-10-14 Actionable source rejection feedback preserves authority
Source publication/display review: backend already rejects changed Host authority
or replaced source/browser revision; no rejection relaxation needed. UI returned
raw fixed internal stale media resolution text. Actual component deferred IPC
baseline reproduces that raw message rather than useful retry guidance. Small
sourceFailure helper maps exact known strings only, shared by select/watch/prepare/
import/refresh. Changed playback/TorBox selection gives explicit retry action;
refresh asks Refresh playback links again; copied source asks Paste the source
again. Obsolete Stremio choice explicitly asks Refresh Stremio and choose title
again, avoiding a retry against unchanged stale choices. Stopping Host asks reopen
2gether. No automatic retry/create, new cancellation owner or serialized IPC schema.

Thirty new deferred-IPC feedback checks across five handlers:four replacement
categories plus stopping Host each preserve current title/page and release busy;
five other fixed provider errors preserve existing feedback. Together with42
existing metadata/admission checks, final72passed; Svelte zero errors/warnings.
Initial67passed before browser-choice refinement/fallback assertions; final72
supersedes. Baseline/log/history retained, exact source hashes/receipt boundaries:
work/source-validation/source-feedback-validation.json. No DOM/Tauri rendering,
actual provider stale publication or native playback failure reproduced by these
component assertions; earlier authority source receipts remain separate historical
evidence. Helper is not a general arbitrary-error/privacy sanitizer; fallback
retains existing backend messages. Known mapped responses contain no provider data.

Both-app review: Android grant preparation already uses fixed recoverable feedback
and its existing retry rules; no Android code/build/unchanged tests. PC backend
publication and typed diagnostics unchanged; no Rust/native/provider proof rerun.
No app dependencies/schema/playback work, APK/EXE packages, emulator/WAN/physical
test. Alpha0.1.7 unchanged, owners free, estimates75%playback/35%full beta unchanged.
Roadmap phase1 source consolidation continues; runtime acceptance still requires
allowed candidate build and device evidence, not another synthetic proof count.

### 2026-10-14 Updater worker owns failed or cancelled artifact cleanup
Updater source review: UUID destinations isolate repeated downloads; UI offers
only returned verified file and rechecks archive before install. No verification
bypass demonstrated. Coroutine catch deleted its file on cancellation, but can run
before HTTP callback worker creates destination. Later worker exception/cancellation
could then leave a partial file. Added small worker writeUpdateArtifact helper
around existing response-body/hash/archive work, checking ownership before/after
and deleting on failure after worker streams unwind. Coroutine deletion retained
for cancellation after completed callback/result delivery. No shared destination,
APK policy/hash/signer relaxation, new download feature/library or progress polling.
Deletion remains best-effort filesystem cleanup, not a power-loss/reboot guarantee.

Five JVM checks exercise actual helper with isolated ordinary .bin files: worker
held before creation, waiter cancelled/deletion initially false, release creates
file, cancelled worker removes it; already-cancelled no write; cancellation after
synthetic verification discards; write/verification exception cleans; current
completed bytes retained. Controlled CURRENT helper removing only worker deletion
fails one late-file assertion, exact JUnit/log preserved. This is not whole old
updater or an actual APK/PackageManager/network callback race reproduction. Helper
restored byte-exact; final5passed. Main and instrumentation Kotlin compiled ONLY
before comparison, not executed. Exact logs/JUnit/hashes/scope:
work/source-validation/update-artifact-validation.json.

Both-app review keeps existing cancellation-safe HTTP response owner and archive
re-verification; duplicate stream/file hashing serves different checks, no speculative
optimization. Windows has no analogous updater artifact path added in this pass.
No unchanged HTTP suite rerun/real release download/SDK emulator/install/APKEXE
packaging/provider/native/WAN/physical test or user-file cleanup. Alpha0.1.7 unchanged,
owners free, planning75%playback/35%full beta unchanged. Revised Android integration
needs future permitted runtime validation; filesystem tests do not establish
installer usability or physical playback acceptance.

### 2026-10-14 Milestone evidence consolidation
Reconciled outdated MILESTONE-1 current-evidence rows with exact receipts. Direct
media is no longer merely pending resolver: offered0.1.7 real native pair passed
f77f3066; separate source range/publication8ff11957 passed. Distinct completed
handoff e31ca3af still failed at inventory before publication. Packaged fixture/
normal pairing89b0cb81 passed; later Android pairing/updater receipts explicitly
instrumentation not executed. Verified these fields directly without displaying
private data or changing any payload/log/receipt. Milestone names exact paths,
compiled5ea1824 vs later source, conditional account-bearing exception=true and
physical gate=false. Separates native/emulator/source/physical/WAN boundaries,
retains historical earlier failed candidates, p95/property uncertainty and roadmap
sequence. No historical failure relabelled, no composite pass.

Documentation only, no build/test/provider/emulator/native/app launch; no source
behavior/dependency or package change. Receipt hashes/claim checks saved in
work/source-validation/milestone-consolidation-validation.json. Current source
candidate runtime and physical two-hour/WAN remain unverified; routine package
hold persists. Planning75%playback/35%full beta unchanged, owners free. Highest-value
next acceptance evidence remains allowed revised runtime candidate and physical
phone, not additional unchanged synthetic proof counts.

### 2026-10-14 Secondary native failures retain the pending fault
Human explicitly reaffirmed active source development and maximum useful use of
available PC time after two idle heartbeat runs. Package hold limits packaging;
it does not justify stopping grounded investigation. Resumed connected review,
implementation and validation across Android playback and Windows connection UI.

Android command exception catch and current-entry EndFile error callback both
unconditionally replaced pending faults, unlike existing EOF/CAS ownership and
Windows command-failure guard. They now use current StateFlow.update policies:
first failure still invalidates readiness; secondary command/decoder result retains
pending code and its fixed error. Matching explicit successful source recovery
still clears that fault through existing revision/admitted-fault completion fence.
No new fault sequence, authority schema, source/grant bypass or worker API interval
atomicity claim. Four command-policy checks initially passed; controlled CURRENT
helper restoring old unconditional command assignment failed three, retained logs/
JUnit; helper restored byte-exact. Final seven new status/recovery/deferred checks
plus seven related existing checks passed (14 JVM total). Main/instrumentation
Kotlin compiled ONLY; callbacks/JNI/device ordering not executed. Deferred failures
in JVM are synthetic completions, not Android decoder crashes.

Windows event loop likewise replaced a pending native CommandFailed with later
source_failure. Extracted its exact original policy before change. Actual muted
libmpv unknown-command rejection followed by isolated invalid 26-byte local file
produced one current-entry format event; original policy changed CommandFailed to
DecoderFailed and failed. Revised event handler admits only when no pending native
fault, preserving original message/code; first matching source failure still admits.
No decoder/GPU crash or normal user-source failure reproduced, no full controller
pending-event race claimed from this policy harness. Native format errors use
existing protocol DecoderFailed label, distinct from proof of a crashing decoder.
Two focused native checks passed and production cargo check passed; broader 24
Windows player checks passed, 27 nonplayer tests filtered, including existing
fault/recovery, authority pressure and lifecycle paths after this production change.
No added app dependencies, event polling or diagnostic allowlist. Existing event
entry fence and explicit source recovery remain intact. Small fault transforms
remain local to the native boundary; no broad coordinator/DTO refactor.

Exact failures, native receipts, logs, immutable JUnit/source hashes and boundaries:
work/source-validation/secondary-native-fault-validation.json. No Android runtime,
provider/TorBox retry, WAN/public-tunnel/physical test, release APK/EXE or offered
payload changed. Alpha0.1.7 remains compiled5ea1824; planning75%playback/35%full beta
unchanged. Source tests do not establish revised paired app usability.

### 2026-10-14 PC pairing controls share pending-operation admission
Actual component deferred IPC baseline admitted pairPhone while stopPhoneConnection
was pending; those two buttons were independently enabled. Duplicate-handler
baseline also retained. Existing pairingBusy/connectionBusy now mutually gate all
four start/stop/generate/cancel handlers and their buttons, with no new owner.
Cancel code clears display only after backend confirms; failure keeps existing code
and fixed retry notice, releases busy, permits retry. Stop clears code and invokes
cancel_pairing after confirmed stop before refreshing address; refresh failure can
no longer skip code cancellation. No tunnel implementation/backend credential or
pairing protocol change; no public connection launched.

22 new actual component-script checks pass:16 action pair/duplicate admissions,
failed cancellation retaining code/retry, stop/cancel before failed refresh, and
four operation failure/retry cases. Svelte zero errors/warnings. This is deferred
IPC over actual component script, not DOM/Tauri rendering or live cloudflared
validation. No unchanged Stremio test suite rerun. Exact baseline/final logs/hashes:
work/source-validation/pairing-control-admission-validation.json. Existing source
and pairing feedback preserved; no source publication/create/network settings,
APK/EXE/dependency/playback cadence changes. Package/privacy/public-tunnel and
physical-gate restrictions continue while substantive source development is active.

### 2026-10-14 Blocking temporary connection work retains state ownership
Continued immediately after bb28471 rather than treating patch completion as a
reason to idle. Source review found start/stop/address observation holding state
in awaiting caller while spawn_blocking owned process work. Caller cancellation
could release state before that work completed, admit another transition, and
lose publication. QuickTunnel RAII already kills its owned job on drop; this is
an operation-serialization gap, not evidence of a leaked real cloudflared process.

Small owned_transition::blocking helper transfers existing OwnedMutexGuard into
existing blocking task. Temporary slot is now Arc<Mutex> so its lease outlives
caller; start, stop, origin capture and shutdown transfer that lease through actual
work and state publication. No new task coordinator/cancellation owner/process API,
libraries, periodic wakeups or forced synchronous preemption. Empty origin observer
still avoids blocking task. Existing binary pin/control-only job/cleanup semantics,
shutdown admission checks and fixed errors retained. Origin panic retains previous
fixed stopped notice within its lease before drop, without raw panic/provider text
in product feedback. Application integration compiled only; public operation unrun.

Four helper checks passed, then five final checks after adding isolated old-pattern
comparison and explicit queued admission marker. Old caller-held guard comparison
uses actual Tokio mutex/task cancellation with mocked blocking operation:after
abort, another lock enters before release; late result unpublished. Revised actual
helper keeps lock held until blocked work releases/publishes; next operation waits
and sees result. Queued cancelled waiter never invokes work; ordinary failure and
controlled mock panic return failure/release state, no invented success. Comparison
is a positive assertion of old mock behaviour, not a failed cloudflared runtime run
or whole old Tauri app. Final production cargo check passed after origin-capture
panic feedback refinement. Exact logs/hashes/boundaries:
work/source-validation/temporary-transition-ownership-validation.json.

No actual process started, public-tunnel command retried, network settings changed,
provider/media/emulator/Android/native suite repeated, new APK/EXE or library added.
Alpha0.1.7 unchanged, 75%playback/35%full beta planning estimates unchanged. This
fix preserves blocking completion ownership; it adds no hard driver/filesystem
latency bound or physical/WAN acceptance. Development remains active as human
reaffirmed; routine package hold remains separate from source implementation.

### 2026-10-14 Connection display coalesces pending address reads
Continued connection lifecycle pass after f7ade5d. Existing latest-request display
ordinal rejected stale replies, but timer/navigation could queue independent IPC
queries while pairing_connection waited for temporary lifecycle mutex. Actual
component deferred IPC baseline25refresh requests created25calls. Now one shared
inflight query plus one latest-request flag coalesces waits; obsolete result/error
is ignored and requested follow-up reads current address after Start/Stop. Latest
error still propagates and frees owner for retry. Cleanup belongs to exact promise
so an older completion cannot clear a new read. No new timer/deadline/provider retry
or polling cadence; one bool rather than queued intents. A cancelled UI query does
not stop control process work or mutate pairing credentials.

Final33component checks:22previous pairing-admission checks plus11new address
checks (25requests coalesced, obsolete failure/current response, latest failure/
retry, completed Start with held older read, seven completion arrival timings).
Svelte zero errors/warnings. Initial26passed; next microtask probe incorrectly
assumed cross-realm promises completed before a particular host callback and failed.
Failure log retained; replaced with seven actual arrival-turn checks, not a claim
of another reproduced application bug. Exact baseline/final logs/source hashes:
work/source-validation/connection-refresh-ownership-validation.json.

No DOM/Tauri/public connection/actual slow filesystem test, no native/JVM/provider
suite repeated, no release APKEXE/dependency/timer added. Both-app review retained
Android existing callback ownership and manual action admission; no duplicate
refresh coordinator or speculative shared abstraction. Alpha0.1.7 unchanged,
75%playback/35%full beta estimates unchanged. Current revised source still requires
permitted packaged runtime and physical-phone gate. Active development remains
authorized; package hold is not a reason to substitute idle heartbeat runs for
current-code investigation.

### 2026-10-14 Android checks native property results and retains Failed status
Continued source investigation at13c0640 through current surface/lifecycle code.
Existing serial worker, generation/attached-surface checks and independent cleanup
already fence queued replacement/close work; no new surface coordinator or speculative
framework callback race fix. Pinned libmpvKt0.3.0 bytecode confirms Mpv.set returns
MpvResult<Unit>; negative native property codes become Fail rather than automatically
throwing. Six existing compiled application setter calls ended in pop:two surface-size
writes, load preflight pause/rate, control pause and control rate. Thus a rejected
write result could be ignored and bypass queue's fixed command-failure capture.

All six now explicitly set(...).getOrThrow(). Failed load preflight stops before
loadfile/source acceptance; queue retains existing fixed error and pending-fault
ownership policy. No extra wrapper, library/injected native API, new logging, source
revision/grant/authority relaxation or cancellation changes. Compiled baseline/revised
and final app bytecode retained:all six results now consumed before discard. This is
actual compiled-call evidence; no JNI property rejection/device malfunction executed.
Three JVM checks use actual pinned MpvResult with a test-only rejecting operator
setter to establish Kotlin assignment discard and explicit failure/continuation/
safe-status behavior. Not the real adapter worker or real native driver. Initial
three new plus seven related checks passed (10); immutable logs/JUnit retained.

Review also found probe classified Idle/Buffering/Paused/Playing despite existing
terminal failure, potentially overwriting failure display on next observation.
Extracted exact old classification into local policy; three of four new synthetic
checks failed, logs/JUnit preserved. Fault publications now immediately state Failed;
probe uses current CAS status failure before other observed state. Normal healthy
classification and explicit successful recovery remain unchanged; no native readiness
invented while waiting for next probe. New checks cover all failure codes vs native
states, healthy/recovered states, immediate publication, and deferred observation
with fault arriving after read. Deferred reads are synthetic, not an executed JNI
race or rendered UI result. Final21JVM passed:7new unique checks+14related existing
fault/recovery checks. Main/instrumentation Kotlin compiled ONLY, not executed.

Exact pinned-library/app bytecode hashes, logs, failure history, JUnit, source hashes
and runtime boundaries:work/source-validation/android-native-write-status-validation.json.
Both-app maintainability review:Windows already consumes writes through sticky
command failure capture and terminal Failed precedence, so no Rust/unchanged native
suite rerun. Small Android classification policy stays at native boundary; no app
library, polling interval, additional per-tick validation, DTO/schema or worker owner.
Upstream internal attach/detach/option handling unchanged:checking these six writes
is not a guarantee every native failure is observable. No APKEXE, emulator/provider/
WAN/public-tunnel/physical test or offered payload/user logs changed. Alpha0.1.7 remains
compiled5ea1824; source integration awaits authorized runtime candidate. Estimates
75%playback/35%full beta unchanged; compilation does not establish revised app usability.

### 2026-10-14 Recoverable local notice preserves concurrent native fault
Continued same connected pass after3b789e5. localPreparationFailed used direct
read/copy/write of worker-shared status and replaced error even when native fault
already pending. Source-review read/write interleaving gap, not a reproduced Android
IO/driver race. Converted to StateFlow.update with small local-notice transform:
no fault invention, preserve native observations, pending fault/error takes precedence,
existing pause request retained. This path serves existing local sample loader; no
new media selection/retry/schema/authority policy, coordinator or diagnostic fields.

Three actual helper/StateFlow JVM checks passed:recoverable notice preserves remaining
status fields, known native faults/errors remain owned, forced transform held while
another fault publishes makes actual CAS retry against latest status. The queued
thread and fault publication are synthetic; real adapter/fixture IO/JNI not executed.
Controlled CURRENT helper restoring only old unconditional error copy failed two
of three checks, immutable history/JUnit retained; source restored byte-exact and
final3passed. This comparison reproduces masking policy, not whole old adapter or
its direct read/write race. Main/instrumentation Kotlin compiled ONLY before and
after comparison, not executed. Exact logs/JUnit/hashes/boundaries:
work/source-validation/local-preparation-status-validation.json.

No unrelated JVM/native/provider suite rerun, APKEXE/emulator/WAN/physical test,
credential/user log/package change or dependency/new polling. First-fault state
transforms remain small in existing native boundary file. Windows worker status
uses its existing mutex; no analogous local recovery notice rewrite or test rerun.
Alpha0.1.7 unchanged;75%playback/35%full beta estimates unchanged. Native APK runtime
verification remains separate and requires future allowed candidate build.


2026-10-14 connected native initialization ownership pass (after06a5c66)
----------------------------------------------------------------------
Pinned libmpvkt-view0.3 actual artifact bytecode confirms applyTo(Mpv) discards
setOption result; existing OptionSink overload retains the library option generator.
Android now uses that sink with setOption.getOrThrow before existing checked log
configuration/initialize. No option changes, TLS weakening, upstream edit or new
helper/injection API. Factory bytecode confirms create precedes separate initialize.
Existing worker startup catch closes failed core; Main exposes fixed Failed status
instead of Starting. No authoritative player fault code invented.

Three JVM cases use actual pinned options/results with synthetic option writes:
unchanged sequence/values, each required TLS/pause/privacy write rejection stops
before synthetic initialization, unchecked old sink discards rejection/continues.
Actual compiled app lambda now consumes result; MpvException is caught by existing
Exception handler. This is not JNI rejection/Android startup execution. Safe phone
category projection now records uninitialized Failed as failed/fault true; existing
fault, ended, playing, paused, loading and starting remain. Initialized recovery
before next probe and recoverable local notice do not invent fault. Three projection
JVM cases passed. Controlled old fault predicate failed1of3; history/JUnit retained.
Final6JVM passed, main/instrumentation Kotlin compiled ONLY, no test APK execution.

Independent PC constructor review found CString conversion can return after create
without terminate_destroy. Private startup RAII owner now covers conversion/option/
initialize failure and unwind, transferring once to existing Mpv Drop on success.
Guard drops before loaded Library; no Send/Sync or public API change. Three mock
counter-handle ownership checks passed for key/value NUL early return, rejection/
unwind and final transfer. One actual muted vo/ao-null libmpv case rejects NUL key,
NUL value and unknown option, then opens core with pause/TLS verification retained.
Four tests passed,9unchanged filtered; non-test adapter check passed. No baseline
native leak measurement/normal-app failure claimed; normal options are trusted.

Exact artifact/bytecode/log/JUnit/failed-history/source hashes and scope:
work/source-validation/native-startup-ownership-validation.json. This pass removes
three manual PC destroy branches with one owner; Android reuses existing upstream
option generator and diagnostics categories. No dependency, schema, playback poll,
provider/emulator/native-pair/WAN/physical test, APKEXE package or user log/credential
change. OfferedAlpha0.1.7 and75%/35% estimates unchanged. Android revised native
integration still awaits a separately allowed candidate/device validation.


2026-10-14 PC startup failure caller follow-through (afterb7ea64a)
----------------------------------------------------------------
Current worker initialization rejection returned fixed private-safe error but left
state Starting player after native owner ended. Actual source-controller regression
with missing DLL in isolated root reproduced this before/after joined shutdown.
Failed baseline log/receipt retained; invalid DLL comparison was not reached after
first assertion failure. No credential/native path/error payload propagated.

Initialization rejection now sets initializedfalse/Failed alongside existing fixed
error in same status mutex. No fault enums, journal schemas or lifecycle result
changes. Existing report-channel closure releases PC attachment; shutdown success
continues to mean joined cleanup, not successful initialization. Focused test final
passed both missing and invalid isolated DLL cases, checks fixed status/error before
and after join, PC detached, zero diagnostic IO errors. Host shutdown/flush explicit.
One test passed,56unchanged filtered, final non-test Windows check passed. Exact
logs/baseline/current receipts/hash:work/source-validation/pc-startup-status-validation.json.

Existing PC UI reads state and fixed error; playback/source buttons use initialized.
Android equivalent failure status and safe category were addressed in b7ea64a;
no unchanged Android tests rerun. No rendered Tauri/Android/native decoding/provider/
physical/WAN test, release packaging, dependency/poll/schema/user-log change. Protected
DLL and fixtures untouched; isolated invalid DLL is test-owned inert bytes. Offered
Alpha0.1.7 and75%/35% unchanged; failed startup does not satisfy runtime acceptance.


2026-10-14 native availability admission follow-through (after4a33096)
------------------------------------------------------------------
Source review: five PC Tauri playback/handoff handlers relied on frontend initialized
button state and could enter backend work after a late click. No direct Tauri/provider
failure reproduced. Added one PlayerController availability check before player_intent,
Stremio Watch, TorBox select/refresh and copied-source handoff. Stop requested takes
precedence over stale initialized status; initial pending setup gets fixed wait feedback,
failed/stopped owner fixed restart feedback. No raw status detail/provider data emitted.
Initialized native media faults remain admitted for explicit recovery. No backend
metadata/cache preparation/create admission changes or extra cancellation/restart feature.

Three actual method policy checks passed for startup/stopped, initialized fault recovery
and stop request while initialized remains stale. New source-controller lifecycle check
passed two cases: real muted hwnd0libmpv idle owner successfully starts, then joins/stops
and rejects new admission; missing-runtime worker ends and rejects admission. Both PC
participants detached and shutdown joined. Receipt records exact status/outcome. Final4
passed/57unchanged filtered; production Windows check passed. Tauri command wrappers
compiled/reviewed, not invoked via GUI/IPC; no live provider count/late-click reproduction.
Existing epoch/generation/selection/source/grant atomic publication fences remain required:
admission is not a lock across later awaits. Host mutation/publication rules unchanged.

Exact immutable logs/controller receipts/source hashes/limits:
work/source-validation/player-admission-validation.json. Both-app review: Android native
queue and follow/source paths already refuse uninitialized native calls; no speculative
Android command rewrite or unchanged JVM run. Five callers reuse one availability check,
no new dependency/schema/poll/coordinator. No provider/Android/native-pair/physical/WAN
run or APKEXE package/userlog/key change. Alpha0.1.7 unchanged;75%/35% unchanged.


2026-10-14 connected PC preparation UI ownership (after5c8baf0)
-------------------------------------------------------------
Current component metadata handlers lacked admission even though source handlers
used both busy flags. Actual component deferred IPC baseline: preparation pending,
Connect invoked clears options, preparation ready response produces raw TypeError
and releases busy while metadata still pending. Both requests admitted. Exact failed
log retained. This is component ordering, not live rendered click/backend corruption:
backend revision fence already rejects replaced preparation; delayed successful reply
can precede server replacement while UI has already cleared choices.

Shared stremioAction now checks existing stremioBusy/torboxBusy before ordinal/reset/
request; catalog/search/back controls match both flags. No new owner/ordinal/helper,
metadata cancellation changes or transmitted-create cancellation.61new component
checks cover direct overlapping preparation/refresh result,30source×metadata pairs,
24metadata pairs and6inventory pairs. Existing72checks remain. Backend preparation
releases browser ownership during IO and checks exact revision on completion; source
publication/selection/PC authority fences unchanged, no backend tests rerun.

Follow-on repeated same-index preparation handler completion baseline duplicated
ready rows (2instead1). Ready row now replaces same index while retaining other
choices; existing prepared Set/cached removal stays.2new actual handler checks pass
fresh/preexisting-ready row, two sequential completions, other-index preservation.
First fresh-row baseline failed; preexisting comparison not reached until fixed.
No real duplicate creation or provider calls claimed: backend already returns its
Ready choice without creation; UI change is idempotent row display only.

Final135actual component-script checks passed (63new+72existing); Svelte0errors/
0warnings. Not DOM/Tauri/provider/native runtime. Exact history/log/hash/boundaries:
work/source-validation/preparation-ui-ownership-validation.json. Android has no PC
Stremio source-selection UI; no change/unchanged tests. No new dependency/coordinator/
schema/playback polling/APKEXE/emulator/WAN/physical/provider/native-pair/userlog/key
change. Alpha0.1.7 unchanged;75%/35% unchanged. Transmitted creation safety and final
publication remain separate from UI admission; no usability claim from source check.


2026-10-14 Android waiting control authority ownership (afteredfc8ea)
-------------------------------------------------------------------
Prior socket/ACK review correctly covered loss/disconnect queue clearing, old-socket
fences, oneinflight/latestwaiting and strict matching ACK/deadline. This pass found
a distinct source gap within a live connection: bare waiting action JSONObject
survived newer accepted generation/epoch while send stamped current authority.
That can rebase an earlier queued pause/seek to newer PC authority. No live Android
socket/native race reproduced; prior transport proof is not relabelled.

Small production QueuedControl record now owns action+epoch+generation from intent.
Flush clears the waiting slot, then returns action only for exact current authority.
Same authority/revision-only updates retain intention; newgeneration/epoch discards.
Intent checks connected/current state/socket before allocating queue, so offline stale
screen callbacks cannot enqueue a future action. Seek buttons now also require native
initialized and, in paired mode, connected/media available. Existing native/source/
fault recovery/grant/socket fences, oneinflight sequence/matching ACK8sdeadline/latest
waiting slot and manual pairing unchanged. No retransmission/extraqueueowner/schema.

Three distinct actual token JVM checks passed: exact current action identity, epoch/
generation mismatch, coroutine-held synthetic ACK then authority replacement; current
new intent admitted afterwards. Controlled CURRENT helper returning action without
ownership predicate failed2of3; immutable log/JUnit retained, helper bytes restored.
Final3passed; main/instrumentation Kotlin compiled ONLY. Compiled SessionController
bytecode confirms token construction and forAuthority call before send. This does not
execute JSONObject acceptance/flush/socket/JNI/Activity or establish native convergence.

Exact initial/failure/final/JUnit/bytecode/sourcehashes/boundaries:
work/source-validation/queued-control-authority-validation.json. Windows PC intents
are in-process authority commands; existing phone protocol rejects stale stamped
commands, so no duplicate queue class or unchanged Windows proof rerun. No new app
dependency/poll/APKEXE/provider/emulator/WAN/physical/nativepair/userlog/key change.
Alpha0.1.7 remains unchanged;75%/35% unchanged. Revised Android runtime awaits a
separately permitted candidate/device test; source compilation is not that gate.


2026-10-14 same-session queued control causal ACK refinement (after7e64475)
------------------------------------------------------------------------
Continued reducer review identified strict-generation-only queue discard would also
lose rapid Play→Pause when first successful phone command advances generation.
Refined before release/device work: sent command captures in-flight authority token;
only accepted snapshot plus matching sequence plus error-free ACK may carry waiting
action through exactly one same-epoch advance from that sent base. Current waiting
action already bound to ACK state remains intact. Further generation, changed epoch,
wrong sent base and overflow cannot carry. Rejected/unmatched/ignored stale ACK has
no carry path. Existing oneinflight/latestwaiting/ACKdeadline/socket resets preserved.

Server source verifies ACK result JSON constructed with command outcome while same
session mutex held, then mutex drops before send. Successful phone play/pause/seek
reducer advances once; external replacement before processing yields stale error,
and later newer snapshot makes older ACK ignored. No server change/schema extension.
Initial7e64475 strict-generation behavior is superseded by this refinement; its3case
proof and2of3unbound policy baseline remain honest historical receipts.

Final4distinct JVM token policy tests passed (three prior plus causal-success case):
rapid pause survives own7→8success, noadvance/current newer queued action preserved,
independent7→9/wrongepoch/wrongbase/overflow refused. Coroutine-held synthetic ACK
replacement case remains; no Android controller/socket/JNI/Activity runtime executed.
Main/instrumentation compiled ONLY, final app bytecode invokes causal carry and exact
flush check. Initial receipt assembly assertion looked in outer class; ACK call is
in coroutine inner class, corrected capture/history retained, no test/source change.
Source sequence/error branch reviewed. Exact logs/JUnit/bytecode/final hashes:
work/source-validation/queued-control-causal-validation.json.

No duplicate outbox coordinator, retransmission, dependency/schema/poll, APKEXE,
provider/Windows/nativepair/emulator/WAN/physical/userlog/key change. Alpha0.1.7
unchanged;75%/35% unchanged. Completed source task binds queued intentions while
preserving own successful command sequencing; real phone verification remains open.


2026-10-14 causal ACK server runtime contract (after8d49008)
---------------------------------------------------------
Distinct source-level loopback proof verifies server evidence behind queued-control
carry; no Kotlin queue reimplementation and no new production fix. Authenticated
synthetic phone uses existing transport helpers/dependencies, PC local authority,
random loopback port, symbolic fixture identity with no native player/decode.
Successful phone Play/Pause/Seek each returns matching seq/sameepoch/genbase+1 with
actual reducer result. Awaited PC seek supersedes each result, then old-base queued
pause rejects stalegeneration without changing epoch/gen/media/timeline/prepare/
desiredplaying/ended/fault. Fresh control based on returned state accepted gen+1.
Wrong epoch returns staleepoch with no generation advance.10ACK receipts total:
3own successes/3stalegen/3fresh successes/1staleepoch. This is protocol authority,
not Playing/native READY/physical acceptance or Android queue runtime.

Initial focused1passed; strengthened final focused1passed/15unchanged filtered.
All10phone command journal rows match corresponding ACK sequence/resultinggen/
revision/outcome. Journal0errors/0drops/export privacy check passed; no test tokens,
Bearer or loopback address in export. Graceful Host/server join, phone detached and
listener port rebound confirm cleanup. Unique initial/final receipts/journals/logs/
final hash and limits:work/source-validation/causal-ack-server-validation.json.
No failure or controlled server change/comparison needed; existing contract correct.

Review retains small Android token + one inflight metadata field, no second outbox
or polling; Windows PC owns direct authority and existing generation/grant fences.
Prior4JVM token checks and compiled controller wiring remain separate historical
proofs; not rerun/not combined into Android runtime proof. No production dependency/
schema/poll/provider/native/emulator/Android/WAN/physical/APKEXE/key/userlog change.
Alpha0.1.7 unchanged;75%/35% unchanged; physical phone and allowed candidate gates open.


2026-10-14 internal feedback ACK boundary and terminal control carry (aftercd8a3ee)
-------------------------------------------------------------------------------
Reducer review confirms successful feedback can leave generation or advance once;
terminal feedback has a distinct ownership meaning. Generic ACK carry in8d49008
could promote earlier queued Play through player_failed or ended acknowledgement,
turning older intent into new recovery/replay. Production token policy comparison
establishes this gap; not a live Android fault/EOF/automatic-recovery reproduction.
In-flight token now retains existing action type instead of Unit. Exact matched
success still required, but terminal player_failed/ended cannot carry earlier token
through their generation advance. Fresh user intent against already-recorded current
terminal authority remains admitted; same-generation repeated report does not drop
current intent. Ordinary own command/buffering/preparation progression preserved.
No new enum/protocol/DTO/owner/queue field, retry feature or per-tick policy checks.

New terminal policy case plus4related token cases final5JVM passed. Controlled CURRENT
helper removing only terminal-type predicate failed1of5 at player_failed; old ended
comparison not reached after failure; final checks cover both and ordinary feedback.
Immutable failure/JUnit retained, helper restored byte-exact. Main/instrumentation
Kotlin compiled ONLY; controller/JSON/socket/JNI/Activity terminal ordering unexecuted.

Separate new real-loopback server internal feedback test passed1/16unchanged filtered:
12ACKs(8zero/4onegeneration), READY/duplicateREADY,buffering transitions/duplicate,
CommandFailed/duplicate,recovery/duplicate,paused Executed/duplicate. Symbolic fixture
room and synthetic native reports, no native core/decode/fault injection. Exact fault,
participant buffering/fault/execution fields checked. All12phone journal rows match
wire seq/base/resultgen/revision/accepted outcome;0errors/0drops/exportprivacypassed.
Phone detached/server joined/listener rebound. Server ended not run here; EOF denial
is JVM policy only. No new server production change or violation claimed.

Exact baseline/final/log/JUnit/server receipt/journal/final sourcehashes/limits:
work/source-validation/terminal-control-carry-validation.json. Historical4JVM and
10userACK proofs remain separate, not relabelled or combined into phone verification.
Both-app review keeps one existing waiting-slot/inflight metadata record; native
fault recovery/source publication/grant fences unchanged. No library/schema/poll/
APKEXE/provider/emulator/Windowsnative/WAN/physical/userlog/key change. Alpha0.1.7
unchanged;75%/35% unchanged; revised Android device runtime gate remains open.


## 2026-10-14 — seek bounds and current control feedback (source only)

Actual Windows component/inline-button baseline submitted -499ms for a 1ms
media duration: forward seek subtracted its existing 500ms end margin without a
zero floor. Shared seek handler now clamps both buttons to [0,max(0,duration-500)],
retaining the end margin. Handler and button admission share initialized/current
room media/positive duration checks. Old stopped duration no longer enables seek.
No short native clip/Tauri unsigned argument rejection was executed or claimed.

Connected deferred-reply baseline after the bound fix: older seek rejection
replaced successful newer Pause feedback. Existing control requests now retain an
ordinal for latest-result/error display; controls remain usable while earlier
requests are outstanding. This does not reorder/serialize backend execution or
fence unrelated source/diagnostic feedback. Both baseline failures/logs retained;
first shortest-duration baseline stops before other cases; older-success baseline
not run. Final actual component script + extracted button expressions:18 checks
(7 short/normal duration,5 target clamps,3 unavailable admissions,2 reply order,
1 failure/retry), all passed. Svelte0errors/0warnings. No DOM/rendered app/native.

Android source review: current seek buttons already gate native initialized and
positive duration plus paired connected/media; fixed +/-10000ms target clamps to
[0,duration]. No Android edit/test, arbitrary Long overflow or endpoint bug claim.
PC ensure_available/Host seek <=604800000/native timing bounds unchanged. Shared
small handler avoids duplicate button arithmetic; no coordinator/library/schema/
poll/backend publication/grant change. Exact source/log hashes, initial15checks,
final18checks, baselines and boundaries:work/source-validation/seek-controls-validation.json.
No release APKEXE/provider/emulator/JNI/nativepair/WAN/physical test/userlog/key
change. Offered Alpha0.1.7 unchanged; planning75%playback/35%fullbeta unchanged.


## 2026-10-15 — reviewed source publication and Alpha 0.1.8 preparation

Human resumed background development during Kosmi and authorized the public
GitHub repository jandev07/2getherstream, ongoing validated source commits and one
significant revised Alpha for both devices. GitHub source checkpoint b289e4a was
imported as31d3515 without private local Git history. Static source assets/wrapper
are included; credentials/signers/.wrangler/work logs/payloads/caches excluded.
Synthetic AWS example was the sole key-shaped audit match. GitHub CLI normal
device authorization completed; keyring credentials remain private.

External reviews informed narrowly justified changes: explicit known-action
queued-control carry, not a broad actor/protocol rewrite or speculative fault
counter. Six queued-token JVM checks passed, including READY/Executed/unknown
refusal of hypothetical generation advance while preserving current intent.
This clarifies the existing server contract, not a reproduced bad wire ACK.
Main/instrumentation compiled only at this checkpoint.

Public updater metadata no longer requires a paired PC. Windows has one-time QR
generation and a fixed download page, with qrcodegen1.8.0 MIT notice included.
Separate APK-only redirect helper permits canonical GitHub release/CDN HTTPS
redirects (three hops, loop detection, one15minute cooperative budget), with fresh
requests and shared cancellation-safe response ownership. Metadata/control remain
no-redirect; APK size/hash/package/higher-version/installed-signer checks retained.
Seven redirect JVM +onepublic-request +threerelease tests passed. Actual isolated
loopback redirect/stalled-body test uses TEST-ONLY address mapping; not real GitHub
TLS/CDN, PackageManager install or Android device playback evidence. Initial test
constructed NUL response header rejected by OkHttp before decoder ownership;
corrected direct-policy assertion, failed log/JUnit retained.

Chunked Worker APK prototype removed before deployment; original source/tests and
prior12Node/7ordinaryfile receipts retained in work. Revised small Worker serves
only page/metadata and points to immutable GitHub Release APK. SixWorker and
sixordinaryfile staging checks passed, including OS publication lock/idempotence/
retained version/downgrade/hash rejection. No APK staged/uploaded or Worker deployed
at this checkpoint. No Range resume feature, account secret or media proxy added.
Windows productioncheck passed; Svelte0errors0warnings. Earlier QR geometry/render/
decode receipt remains separate source evidence, not phone-camera/rendered-app.
Exact logs/hash/scope:work/source-validation/github-download-validation.json.

Android versionCode9/0.1.8-private-alpha and Windows0.1.8 prepared. Candidate must
run installed native/normal pairing/recovery validation before delivery. Newly
compiled native pending-fault regression added to expected executed identities;
no old receipt relabelled. Alpha0.1.7 remains untouched, physicalphone/twohour/WAN
open, blocked public playback-tunnel test not retried. Planning75%/35% unchanged.


## 2026-10-15 — revised installed fixture/normal pairing checkpoint

Candidate20261014-195818-f829095c compiled747c2af, Androidcode9/same application
identity and Windows0.1.8.160JVM tests/43suites/0failures and embedded frontend check
passed. Exact hidden packaged startup+Android emulator fixture+normal pairing run
94d57728-32de-44e3-9f40-08713d6a73ac PASSED20:02:44UTC.12exact native identities
executed, including new pendingUnexpectedEndAfterLoadAdmissionSurvivesWorkerCommitBoundary;
normal code-entry/encrypted save/new Activity restore1case passed. New pendingfault
case uses real adapter/native worker and deliberately invoked unexpectedEnd notice,
not spontaneous EOF/decoder crash. Runtime now exercises checked Android native
option/property writes, source-load fault gate and current native adapter lifecycle.
No claim all fault interleavings, frame timing, camera QR, physical/WAN acceptance.

Immutable candidate/receipt/logs and narrower summary:
work/source-validation/alpha-018-integration-validation.json. Alpha0.1.7 unchanged;
revised candidate not delivered/published yet. Independent current-source native
TorBox host example rebuilt with association/hash recorded before distinct revised
provider-pair run. No unchanged historical provider proof relabelled or repeated.
GitHub source mirror747c2af→0d017167 tree verified equal; normal CLI authorization
succeeded. Future publisher checks remote head to preserve Antigravity/contributor
changes and never exports private original Git history.


## 2026-10-15 — visible Discover flow and requested delivery window

User permits visible windows/audio and requests the new Alpha tomorrow only:
not before2026-10-16 00:00IST /2026-10-15 18:30UTC. Internal candidate/source
validation continues. config/alpha-delivery.json plus AlphaDeliveryWindow.ps1
protect Desktop delivery; public publishing must use the same cutoff. Initial
ISO DateTime conversion double-zone bug in guard check corrected before delivery;
exact UTC cutoff verified. ValidateOnly remains available during hold.

Actual normal candidate747c2af visible review9aaa01e9 fetched20 Stremio catalog
items using existing encrypted account and a title's read-only source choices.
Source panel sat below title grid (AX present, onscreen visibility not verified).
Discover now makes Stremio primary, replaces grid with selected source panel and
provides guarded local Choose another title; copied source is collapsed. Creates,
authority and request ownership unchanged.138 actual component-script checks
(135related +3new return-admission), Svelte0errors0warnings passed. Revised layout
not yet rendered in a rebuilt EXE; old visible review is baseline only. Exact
work/source-validation/discover-flow-validation.json and immutable review records.

Real revised source-host TorBox+candidate Android run2555f660 passed20:08:41UTC:
controls/seek/background/reconnect/steady60/EOF/replay/pause/retry/terminal fault/
paused recovery/manual log upload. Private account-bearing exceptiontrue;
loopback/samePCegress/emulator, not physical/WAN or packaged Windows browse-watch
provider path. Exact real-provider-pair receipt retained; no new timing claim.

Native source provenance: official Android mpv/FFmpeg source archives downloaded,
both upstream SHA256 matches; additional two APK graphics-path.so files match
androidx.graphics:graphics-path:1.0.1 AAR. Together with earlier20mpv/JNI/FFmpeg
correspondence rows, all22 packaged native files now have artifact attribution.
This is byte correspondence, NOT complete source-distribution/legal acceptance.
Windows exact upstream build workflow/log retrieved; source bundle still in work.
work/source-validation/native-source-archive-validation.json. No public APK,
metadata/Worker deployment or Desktop replacement. Alpha0.1.7 preserved; planning
75%playback/35%beta and physical/WAN gates unchanged.


## 2026-10-15 — revised rendered source selection and native source inventory

Fresh internal candidate20261014-205824-625179ea compilede286263 after source-flow
change; build passed, candidate receipt/source/hash preserved. Actual normal
packaged Windows rendered review connected existing encrypted Stremio account,
loaded20Popular titles, displayed1ready plus cached choices for selected title.
Source panel visibly replaced grid; Choose another title restored20loaded titles.
No Watch/Prepare submission or provider creation. Exact owned review directory
is recorded by work/source-validation/discover-visible-directory.txt; ui-review
receipt ties EXE SHA to source. Window closed through normal UI, exact process
absence verified before lease release. Native/control owners now free/absent.
This is rendered UI/read-only provider evidence, not new native playback proof;
previous747c2af emulator/provider receipts retain their own scope.

Official source archives collected for all11 primary Android native projects,
plus wrapper/build scripts previously pinned. mpv/FFmpeg upstream checksums
verified; other archives have local integrity hashes only.18unmodified upstream
license/notice documents retained with exact member hashes. Lua top-level README
only references license; full original doc/readme.html included as well. Windows
build source/mpv/FFmpeg exact archives collected, refs matched published x86_64
job and DLL version. Static dependency/NDK/runtime source assembly still pending;
no completeness/legal-compliance guarantee inferred from hashes or upstreamlinks.
Public native/SOURCE-INVENTORY.json is explicitly partial. No credentials, user
logs or binary/source archives exported by source commit. No public APK/metadata/
Worker or Desktop Alpha delivery before requested16Oct00:00IST. Offered0.1.7
unchanged; physical/WAN and75%/35% planning unchanged.


## 2026-10-15 — accessible project source and license entry

Both apps now expose manual Source code and licenses entry to fixed public
GitHub notice/source page. No caller URL, pairing/media/account data or automatic
browser launch. Windows shares its existing fixed public-page opener; Android
uses ordinary ACTION_VIEW with fixed failure feedback. Existing lifecycle handles
user leaving Android; no playback poll, permission, dependency or URI guard change.
Project LICENSE now explicitAGPL-3.0-only, consistent with already-derived AGPL
modules; upstream terms preserved, not relicensed. Complete native source bundle
still required; no legal/source-completeness guarantee from new entry.

Windows production cargo check, Svelte0/0,138related component checks and Android
main/instrumentation Kotlin compile passed. No rendered click/Android intent/browser
or new native-device test for this addition; internal rendered Discover candidate
e286263 remains its own earlier source boundary. Exact source hashes/logs/receipt:
work/source-validation/source-notices-directory.txt. Initial edit cp1252 decode
failed before Svelte/Android mutation; UTF8 corrected, no test/build failure.
No new candidate/package for this addition yet; final candidate should include it
when remaining release source materials are ready. User16Oct00:00IST public/Desktop
hold maintained. GitHub source publication remains authorized. No new dependencies,
provider requests/emulator/native/WAN/physical tests or user logs/credentials changed.
Owners free after source compile; Alpha0.1.7 preserved; estimates75%/35% unchanged.


## 2026-10-15 — real Android archive verification and owned upgrade retention

Owned run1a834e76-b897-4093-9727-1e26e3bd76b1 completed21:19:05UTC at73ff192.
Internal source-associated lower8 build is validation-only; higher9 is same-source
normal version. Both isolated outputs preserve offered packages. Actual Android
PackageManager verified higher APK package/code/installed signer plus size/hash;
real FileProvider served its bytes. Wrong version/hash/size and appended artifact
rejected. Four UpdateAndLogs cases now actually executed, including JSONObject
coercion/privacy assertions; stale legacy expected3 corrected to exact4derived
count. Before5/after1validation identities all raw verified, plus1owned cleanup.

Actual adb install-r8→9 retained isolated synthetic AES-GCM/Keystore pairing and
SharedPreferences setting. Normal pairing compared privately before/after without
mutation/export; presence not recorded, so no claim actual normal pairing was
nonempty or a restored Activity connected. No HTTP/public GitHubCDNdownload,
installer permission/confirmation flow, native playback or physical/WAN acceptance.
Exact immutable APK/init/source association/logs/raw results/receipt under
work/update-archive-proof/1a834e76...; summary update-archive-integration-validation.json.

Review found new runner's cleanup-notmatch expression returned filtered text,
truthy even when target serial present. Original passed receipt retains that
incorrectly typed field; it is not silently rewritten. Real device inventory
independently confirms absence. Fixed actual AST assignment6cases: own device/
offline/error preserve ownership, absent/other serial clear; errors cannot authorize
new boot. Failed stop retains explicit unverified owner marker. This corrects the
new test runner, not an application upgrade bug or observed emulator orphan.
Exact update-archive-cleanup-boolean-validation.json. No unchanged device rerun.

Native source collection found archive download omits required gitlinks. Exact
MbedTLSframework, FreeType dlg and six libplacebo submodules now retained (8total),
API pins recorded/local archive hashes checked, no nested .gitmodules present.
Initial recursive API page returned100entries only; no absence conclusion taken;
explicit submodule-directory pages verified complete.28original notice files now
retained, including build/demo sources without claiming every one ships in APK.
Windows static dependency/NDK/runtime source assembly still pending; public
SOURCE-INVENTORY.json remains explicitly partial. No upstream product code edited.

Human asleep, Twitch drops stream must stay open/unminimized/untouched. Background
work preferred; no UI focus used this pass. Antigravity questions authorized if
needed, no message sent yet. Existing automation exact prompt replaced per human,
verified ACTIVE20min/currentchat/default notifications; no schedule/service repair
claim from config alone. Keep prompt verbatim; update DEVELOPMENT for handoffs.
GitHub source pushes authorized; no public APK/Worker/Desktop delivery before
16Oct00:00IST. Owners free/absent; no new deps, network changes, credentials/userlog
exports or provider activity; Alpha0.1.7 unchanged,75%/35% planning unchanged.


## 2026-10-15 — actual local download Worker runtime

Unchanged Worker source executed in actual local workerd with real ASSETS binding,
intended compatibilitydate2026-10-14.7HTTP checks passed: GET/HEAD metadata+landing,
APKpath404,POST405,query/cookie/auth fixture request leaves exact public manifest.
Escaped version text/CSP/no-referrer/no-store/nosniff checked.2live asset replacement
cases (coerced size, malformed JSON) return fixed503 without leaking asset data.
Synthetic metadata only; no real APK served/fetched or GitHubCDN/public Worker,
Android HTTP download/install, QR camera, provider/native/physical/WAN test.

Initial cachedWrangler4.147 runtime failed to start because its maximum supported
compatibilitydate was2026-10-08. Exact failed log retained. Isolated wrangler4.151
validation tool installed with ignore-scripts in ignoredwork, no app dependency
or production date change. Source remained unchanged and intended config tested;
not a deployed application bug or public tunnel attempt. Exact directory pointer
work/source-validation/worker-runtime-directory.txt, runtime logs/HTTP receipt/
fixtureconfighash/ownedPID+exactstart receipt retained. Owned Node/workerd process
 tree stopped after tests, both48780/48781listeners absent. CLI exit1 is scoped
termination, not a failed Worker request. No Twitch/browser UI action.

Human exact automation prompt update verified ACTIVE20min/currentchat/default
notifications. Scheduled heartbeat messages reached this chat; configuration alone
cannot explain delayed/missing visible notifications. No scheduler-service repair
or uninterrupted execution promise. AGENTS latest header now carries current
Twitch/delivery/GitHub authority to prevent obsolete Kosmi/pause text overriding it.
Public/Desktop hold and all other safety/evidence boundaries remain; source work
continues. No repeated playback/provider suites or speculative product changes.
