# Native emulator stability evidence — 8 October 2026

Run `d8d2bfd2-cf7d-4555-81b6-3dd1a66bd3c1` completed with receipt
`passed/complete`. It requested and completed 7,200 seconds of uninterrupted
playback within five native instrumentation tests. Total instrumentation time
was 7,340.146 seconds, from runner start 01:05:29 UTC through final receipt
03:08:32 UTC. It used source `62b8130`, the isolated Windows native libmpv host,
Android emulator-5580 with native libmpv, a local synthetic H.264/AAC fixture,
and a delayed control proxy configured for 800 ms RTT and +/-100 ms jitter.

Artifacts, receipt, matching run-ID metrics, launch record, both player JSONL
logs, instrumentation result and retained production journal were preserved in
ignored `work/passed-soak-d8d2bfd2` before starting a new build/test.

| Measurement | Result |
|---|---|
| Instrumentation | Five tests passed, none skipped |
| Uninterrupted playback | 7,200 seconds, one generation (26) |
| Steady phone observations | 71,161 |
| Largest steady phone authority-relative drift | 333 ms |
| Largest recorded steady native observation age | 128 ms |
| Steady disconnected / buffering / native-failure samples | 0 / 0 / 0 |
| Cross-device matched observations, all phases | 71,315 |
| Cross-device median / p95 / maximum | 87 / 224.5 / 416.5 ms |
| Measured clock RTT median / maximum | 862 / 1,450 ms |
| Journal validation | Fresh authority/native events, zero drops/storage errors |

The uninterrupted loop asserted unchanged generation, connection, no prepare or
authoritative execution fault, absolute authority-relative Android drift below
350 ms and native observation age at most 350 ms. Recovery tests also passed.
The first/last recorded steady rows span 7,199,971 ms; they are samples inside
the 7,200-second loop, not a separate shorter run.

The provisional **cross-device p95 goal of 150 ms was not met**. Restricting
cross-device comparison to steady rows still gives 87/224.5/416.5 ms. The worst
matched sample was in steady playback: Android authority-relative drift +272 ms,
PC drift -145 ms, Android observation age 77 ms, PC matching distance 16.5 ms,
and measured clock RTT 1,047 ms. Opposing authority-relative errors explain how
the cross-device difference can exceed either client's individual bound. This
is evidence for further correction work, not permission to relax thresholds.

Cross-device analysis matches the same generation and nearest PC native sample
within 120 ms, interpolating with observed playback rate at the Android native
observation time plus estimated clock offset. Property sampling and clock-offset
errors remain. These measurements do not establish actual screen presentation
alignment. A runner pass means its implemented assertions passed; it does not
mean every product quality requirement passed.

This is **emulator/synthetic-media evidence only**. It does not validate a
physical phone, TorBox URL safety/delivery/refresh, real CDN buffering, 4K HEVC,
Cloudflare/WAN interruption or an actual PC-process restart. The physical-phone
two-hour release gate remains open. Later source changes (command/control
journaling, stopped-server lifecycle, Android onClosing and clock reply
correlation) were developed during this run and require fresh native testing.

## Clock-history follow-up: 4d821313 (completed 07:24:57 UTC)

The current playback-core build at source commit
`6b2d8935ed87626b9edd97f0cd7ff7c5cc8a219e` completed another real two-hour
emulator/synthetic run. This includes the time-based drift filter, native
command-fault handling, correlated clock replies and 64-entry fresh clock
history. Later tunnel source changes were not in this executable and are
validated separately by unit/process/journal tests.

Receipt `4d821313-6066-4dc2-9873-587e74599ce7` is passed/complete with requested
and completed uninterrupted duration both 7,200 seconds. Five native tests
passed in 7,346.671 seconds; final receipt and metrics IDs match. Runner exited
and removed its ownership marker. Evidence is archived under ignored
`work/passed-clock-history-soak-4d821313`.

| Measurement | Result |
|---|---:|
| Configured control RTT / one-way jitter | 800 / 100 ms |
| Cross-device matches, all phases | 71,220 |
| Cross-device median / p95 / max | 72.5 / 178.5 / 280.5 ms |
| Provisional p95 target | 150 ms — not met |
| Measured clock RTT median / maximum | 744 / 1,125 ms |
| Steady phone samples / generations | 71,077 / 1 |
| Selected clock sample age / history maxima | 30,000 ms / 64 |
| Largest consecutive selected-offset jump | 173 ms |
| Largest absolute phone authority drift | 181 ms |
| Largest native observation age | 127 ms |
| Steady disconnect / buffering / seeking / prepare / non-playing violations | 0 / 0 / 0 / 0 / 0 |
| Journal validation | Fresh events, zero drops/storage errors |

The steady-only cross-device distribution is also 72.5/178.5/280.5 ms. Signed
phone authority drift min/median/p95/max is -167/10/90/181 ms; PC drift is
-162/-59/11/71 ms. At the worst matched instant phone was +132 ms and PC -149 ms,
both at rate 1.0, with phone native age 17 ms and matching distance 19.5 ms.
The correction controller starts nudging only beyond filtered +/-150 ms, so
opposing deviations inside that band are a concrete remaining quality concern.
This observation does not prove a single cause or justify changing thresholds
without a new experiment. No thresholds were relaxed or changed in this run.

The distributions are lower than the earlier d8d2bfd2 run, but latency, jitter
and operating conditions differ. This is not a controlled causal comparison.
The 64-entry history still permits selected-offset changes as fresh samples
expire; it does not eliminate clock estimation error. Numeric clock analysis
is now reproducible with `scripts/analyze_clock_proof.py`; it requires a
completed pass receipt and refuses output overwrite. The receipt/log pair must
remain together because phone rows do not independently identify their run.

This is a stability assertion pass, **not completion of the 150 ms p95 quality
goal or physical-phone release gate**. Physical Android/TorBox/WAN, CDN buffering,
URL renewal, 4K HEVC and actual PC-process restart still require validation.

## Tighter correction band: f931b66c (completed 10:09:29 UTC)

Source `19b7ff3b5f205c14face28e84f9adf3eb567791c` completed five native tests
in 7,345.557 seconds, including the requested 7,200-second uninterrupted loop.
Receipt and metrics IDs match `f931b66c-0cf8-4ddf-91c2-ad587b7c29b7`; the runner
exited and removed its ownership marker. Evidence is preserved in ignored
`work/passed-nudge-band-soak-f931b66c`, with 19 hashed source/report files in
`evidence-manifest.json`. This includes the rolling journal's retained files,
not an unbounded journal history. No acceptance thresholds were relaxed.

| Measurement | Result |
|---|---:|
| Configured control RTT / one-way jitter | 800 / 100 ms |
| Cross-device matches, all phases | 71,132 |
| Cross-device median / p95 / maximum | 53.5 / 140 / 302.5 ms |
| Provisional p95 target | 150 ms — met in this run |
| Measured clock RTT median / maximum | 750 / 1,108 ms |
| Steady phone samples / generations | 70,996 / 1 |
| Selected clock sample age / history maxima | 30,000 ms / 64 |
| Largest consecutive selected-offset change | 170 ms |
| Largest absolute phone authority drift | 197 ms |
| Largest native observation age | 221 ms |
| Steady disconnect / buffering / seeking / prepare / non-playing violations | 0 / 0 / 0 / 0 / 0 |
| Observed phone rate transitions / maximum in 60 seconds | 798 / 22 |
| Non-nominal rate samples / rate range | 13,349 / 0.995–1.005 |

Steady-only comparison also gives 53.5/140/302.5 ms across 70,996 matched samples.
At the worst match the phone was +190 ms relative to authority, PC -113 ms,
phone rate 0.995, PC rate 1.0, native observation age 24 ms and matching distance
2.5 ms. Thus the distribution met the provisional p95 target while occasional
larger opposing deviations remained. This does not establish screen/audio
presentation alignment or a hard 150 ms bound.

Rates above are observed phone property transitions, not native command counts;
sampling can miss commands. The 75 ms entry / 20 ms exit correction band improved
the measured p95 versus the previous two-hour 178.5 ms result, but operating
conditions varied and maximum drift was higher. This is not a controlled causal
comparison. Correction frequency increased; do not claim chatter is absent or
perceptually harmless. Retain current settings pending further validation.

This is the first two-hour emulator/synthetic run of this tighter band. It meets
its implemented stability assertions and provisional p95 goal, **not the
physical-phone release gate**. Safe TorBox URL delivery, real CDN buffering/URL
renewal, 4K HEVC, public Cloudflare/WAN interruption and actual PC process restart
remain unproven. Later atomic tunnel launcher and explicit graceful Host shutdown
source were developed during this run and have separate local tests; this
executable does not validate those changes. Run the latest source next.

## Later lifecycle and pairing regressions

These short connected runs use the same synthetic media and configured 800 ms
RTT / 100 ms jitter. Each passed all five implemented functional tests and its
requested steady interval. They are not additional two-hour stability results.

| Run / source | Steady seconds | Median / p95 / max cross difference | Provisional p95 goal |
|---|---:|---:|---|
| 3687cb38 / 05b2e3c | 600 | 42 / 119 / 184.5 ms | Met |
| f0ea9a8f / 0d24a81 | 600 | 55 / 130.5 / 223.5 ms | Met |
| 96127c4b / 5563097 | 60 | 47 / 155.5 / 243.5 ms | Not met |

96127c4b completed at 11:56:38 UTC, five native tests in 201.787 seconds. Its
591 steady phone rows stayed in one generation; selected clock age/history
maxima were 29,999 ms / 64, largest consecutive selected-offset change 173 ms,
maximum absolute authority drift 225 ms and native age 105 ms. Counted steady
clock/state violations were zero; production-journal validation passed. Receipt
and metrics IDs match, the runner marker was removed, and evidence is archived
with a hash manifest in `work/passed-pairing-stop-smoke-96127c4b`.

The latest smoke is a functional pass and a provisional p95 miss, both retained.
Different durations and operating conditions do not establish a regression or
causal comparison. The earlier two-hour 140 ms result cannot be attributed to
this later source. No correction or acceptance thresholds were changed. Native
proof cleanup force-terminates its owned app, so it does not validate normal
window-close behavior. Physical-phone, safe TorBox, real CDN/URL renewal, 4K HEVC
and public WAN release requirements remain open.

## First proof with bound log identity: 3846ac42

Source `4de65bf` completed five native tests in 205.994 seconds at 12:21:21 UTC,
including a 60-second steady interval at configured 800 ms RTT / 100 ms jitter.
Schema-2 receipt `3846ac42-ff1a-487c-85de-db013859239b`, every PC/phone observation
and metrics share the same UUID. Both cross-device and clock summaries report
`log_identity_verified: true`. Pairing these actual logs with the prior run's
receipt/expected UUID was rejected by both analyzers without creating reports.

Cross-device matches numbered 723, with median/p95/max 40.5/107.5/158 ms and RTT
median/max 758/1,309 ms. The 591 steady phone rows stayed in one generation, with
zero counted clock/state violations. Maximum selected clock age was 29,995 ms,
history 64, consecutive offset change 75.5 ms, absolute authority drift 111 ms
and native observation age 104 ms. Evidence and a hash manifest are archived in
`work/passed-identity-smoke-3846ac42`; the runner removed its ownership marker.

This validates the new proof identity plumbing and functional integration. It
is a short emulator/synthetic run, not a new stability result or evidence that
synchronization improved because of the logging change. No correction constants
changed; the preceding 155.5 ms p95 miss remains recorded. The physical-phone,
safe real TorBox, WAN and full-window-close validation boundaries remain open.


## Source guards and concurrent TLS installation: 04f494d0

The expanded seven-test run5f975ae5 on8d789e5 failed NativePlayerTest initialization
after the new SessionIdentityTest, although SessionPlaybackTest completed. Its
failed/native_tests receipt remains unchanged, completed durationnull, no fresh
metrics produced. Evidence is archived inwork/failed-native-init-5f975ae5.

A separate eight-worker Android trust-bundle installation test reproduced a
shared temporary-file rename failure. Each installation now uses an exclusive
temporary file before complete PEM publication. System-root selection and TLS
verification are unchanged. Concurrent trust installation, identity acceptance
and actual native playback tests then passed together(3tests,17.435sec).

Fresh eight-test run04f494d0-5c87-4c51-bf26-504189c5fc24 completed at15:10:43UTC,
213.065sec instrumentation time, requested/completedsteady60sec at configured
800msRTT/100msjitter. Runner/Android sourceef7fb71; Windows executable built8d789e5
and hash-identical to failed run. Both Android APK hashes changed as expected.
All8tests passed.727matches yielded median/p95/max49.5/152/183ms; provisional
150ms p95 goal NOT met. MeasuredclockRTT median/max797/1336ms. Receipt/log/metrics
IDs agree. Strictclock analysis:590steadyrows/onegeneration, selectedagemax29949ms,
history64,offsetjump77ms,authoritydrift105ms,nativeage106ms; counted violationszero.
Archivework/passed-source-guard-tls-smoke-04f494d0 includes hashes and boundaries.

This is a short functional emulator/synthetic pass with a retained quality miss.
No thresholds were changed. It does not establish causal timing improvement,
current-revision two-hour stability, physical-phone readiness, original TorBox
media delivery/renewal, public WAN/tunnel behavior or normal full-window exit.
