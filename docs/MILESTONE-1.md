# Milestone 1 acceptance

Updated 2026-10-17 with scoped local native session/restart evidence. The physical release
gate has NOT passed. See ROADMAP.md for the full playback-to-beta sequence and
DEVELOPMENT.md for exact histories. Planning estimates remain75%playback/35%full
beta; these are not acceptance scores.

Offered Alpha0.1.7 is candidate20261012-202957-e253509f, compiled source5ea1824.
Later source changes are not in that package. Do not extend an older package's
native pass to the current working source. Package hold remains until feedback
or an explicit build request; this checklist does not authorize another release.

Release gate:one Windows PC and one physical Android phone playing the same
canonical TorBox file through separate direct HTTPS media links. Control carries
HTTPS/WSS messages only; never proxy media through Cloudflare. Record exact
versions, device/network/media conditions, clock uncertainty, decoder behavior,
structured diagnostics and actual two-hour receipt.

## Acceptance and evidence boundaries

| Scenario | Acceptance | Evidence available | Remaining gate |
|---|---|---|---|
| Start/play/pause | Both native players READY, scheduled start, no obsolete resume after pause | Offered0.1.7 native PC/emulator sequence; later muted PC authority/START source regressions | Revised exact candidate runtime and physical phone |
| Rapid seek | Newest request completes; obsolete queued native effects rejected | Historical native seek proof; later real PC capacity-one queue and held-authority regressions | Revised native pair and physical seek sequence |
| Buffering | Either player stalls, both hold, recovery preserves explicit pause | Offered native pair recovery; later PC watchdog/buffer feedback regressions | Actual CDN/network stall and physical recovery |
| Reconnect | Freeze on control loss, recalibrate, prepare and rejoin from PC authority | Offered native pair background/reconnect; later source transport ordering tests | Real Wi-Fi/WAN interruption on physical phone |
| PC restart | New epoch rejects old effects; normal saved pairing/rejoin works | 17October real headless PC process replacement/native Android emulator:freeze, new epoch/source preparation, paused rejoin and explicit resume | Packaged PC restart, saved credential restoration, intended network and physical phone |
| Phone lifecycle | Background/lock/unlock/surface recreation preserve correct authority | Historical emulator lifecycle/background proof | Target physical phone lifecycle, heat/battery and decoder behavior |
| EOF/replay/fault recovery | Normal EOF/replay works; pending faults survive until PC records them and explicit recovery | Offered real TorBox sequence; later real PC premature-EOF/command rejection tests with controlled inputs | Revised Android worker integration and physical fault/recovery |
| Persistent pairing | Normal code entry, encrypted save/restore and safe change-PC flow | Offered normal pairing screen/restore passed; later setup-admission JVM tests and Kotlin compilation only | Revised rendered Android clear/failure/retry flow and physical pairing |
| Direct media | Exact canonical file and separate player links, direct TorBox HTTPS | Offered native pair runf77f3066 passed; source-only range/publication run8ff11957 passed | Revised pair, intended physical-phone egress and delivery-policy evidence |
| Revised completed handoff | Pause/seek/shutdown can supersede fetched source without obsolete publication | Synthetic source/Host checks pass; real-provider rune31ca3af failed at inventory before resolution/publication | Real revised completed-handoff remains unverified; do not relabel earlier pass |
| WAN control | Reachable HTTPS/WSS address, normal pairing and recovery | Bundled temporary control-only cloudflared lifecycle; actual public test rejected by policy and unexecuted | Allowed public reachability evidence; named stable provisioning absent |
| Two-hour stability | Real two-hour physical playback with bounded drift and no correction oscillation | Earlier emulator two-hour history; offered0.1.7 short native pair only | Physical two-hour run on accepted revision remains open |

## Exact retained receipts

- Offered fixture/startup/normal pairing:
  work/private-alpha-candidates/20261012-202957-e253509f/usability-89b0cb81-70f2-42c9-abdb-c1cfb8e65d76/receipt.json,
  status passed, scope hidden Windows and Android emulator fixture.
- Offered real TorBox native pair:
  work/real-provider-pair/f77f3066-f11a-42e1-bcaa-cd037a75f9d6/receipt.json,
  status passed, physical_gate_passed=false. Loopback control/direct HTTPS from
  the same PC egress does not establish WAN or physical-phone delivery.
- Later PC source transactions and bounded direct ranges:
  work/source-validation/real-source-last-run.json,
  run8ff11957-27e2-403f-a771-8a098c04ab0a passed, source/Host only, no native players.
- Distinct real completed-handoff attempt:
  work/source-validation/authority-source-last-run.json,
  rune31ca3af-4cae-4a01-96c1-cff4265efa76 failed at inventory before handoff.
- Later Android pairing setup and updater artifact integration:
  work/source-validation/pairing-setup-validation.json and
  work/source-validation/update-artifact-validation.json, focused JVM checks
  passed; main/instrumentation compiled only, instrumentation not executed.
- Current-source local fixture sessions and owned PC host process replacement:
  work/source-validation/current-session-integration-validation.json and
  work/source-validation/host-restart-directory.txt. The first long fixture run
  failed after its steady phase; its corrected short full sequence passed.
  Restart finalb70f1181 passes one exact selected native/socket method with two
  distinct PC processes/epochs and an Android Activity that stays open.
- Actual current debug Tauri/native surface and Android emulator fixture:
  work/source-validation/rendered-native-pair-directory.txt. Run376af517 passes
  the full selected session method with600steady seconds and normal window-close
  cleanup. Visible native picture checkpoints with minimized intervals;5931
  position matches,p95119ms. No continuous rendered-frame/audio, packaged release,
  provider, pairing-storage, physical-phone or WAN acceptance is implied.
  These use synthetic credentials; they do not establish saved pairing or a
  packaged Windows restart. See DEVELOPMENT.md for exact artifact boundaries.

These are separate evidence scopes, not one composite native receipt. Prior failed
0.1.6/0.1.7 candidates and controlled-comparison failures remain retained; successful
later runs do not relabel them. New tests are not required just to refresh this table.

## Delivery privacy

Normal secure delivery keeps the TorBox account key on the PC. Historical milestone
text described rejection of an account-bearing URL. The later native real-provider
receipt explicitly records account_bearing_private_exception=true under the user's
conditional private-Alpha authorization. That proves the labelled exception path,
not universally file-scoped media credentials or general-release security.

Never bundle/display/log/export keys, signed links, pairing tokens or tunnel
credentials. Retain the exception label and scope; continue preferring file-scoped
credentials. Do not infer direct media delivery is control-tunnel reachability.

## Timing targets and limitations

Provisional steady targets:p95 absolute drift≤150ms,p99≤300ms; no sustained>500ms
for>5s outside declared barriers. Validate presentation latency, clock and property
uncertainty; sampled estimates alone are insufficient.

Offered real TorBox matched samples:p95133ms,median60ms,max328ms; fixture samples:
p95150ms,median69ms,max176.5ms. These are estimates for their recorded runs, not
physical or universal sync guarantees. Earlier delayed native emulator estimates
at100/300/800ms configured RTT remain historical; see VALIDATION.md.

Later isolated native adapter measurement found fast getter calls but100ms reported
position steps on the fixture. Fresh post-read timestamps do not prove equally fresh
internal PTS or displayed frames. Source-level repeated-PTS drift-policy regressions
are not decoder-hang detection or closed-loop physical timing validation.

## Next acceptance work

1. Consolidate source against actual Alpha feedback; preserve current release hold.
2. When authorized, build one exact revised candidate and validate packaged PC UX,
   normal Android pairing, revised worker recovery and real direct-media native pair.
3. Establish physical-phone HTTPS control via an allowed path. Prior public-tunnel
   policy rejection remains binding; no tool workaround or adapter/network changes.
4. Test physical controls, real network/media stalls, restart and phone lifecycle,
   then complete two-hour playback with diagnostics and device/media compatibility.
5. Only after this gate, complete agreed full-beta features/design/parity. Authorized
   PC Stremio browse/source-selection foundation is already in scope before the gate.

Compilation, unit tests, synthetic control clients and emulator tests cannot satisfy
physical acceptance. Do not repeat unchanged cleanup/provider proofs or add isolated
tests merely to increase counts. No reliable calendar release ETA is established.
