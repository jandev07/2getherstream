# 2gether 2.0 roadmap

Updated 2026-10-14. Source baseline:07630f8. Offered package:Alpha0.1.7,
compiled source5ea1824. Later source fixes are not in that offered package.
Planning estimates:75% playback milestone,35% complete polished beta. These are
judgment estimates, not acceptance scores or release dates.

## Priority and critical path

Reliable couple playback comes first. The critical path is:

Current Alpha feedback → focused source stabilization → explicitly authorized
candidate build → revised native PC/Android validation → reachable HTTPS control
and physical-phone acceptance → two-hour gate → complete beta features/design →
release candidate validation.

Source development may continue under the package hold. This roadmap does not
itself authorize a new APK/EXE, deployment, public-tunnel test or network changes.
Do not keep adding isolated synthetic proofs as a substitute for the next runtime
gate. Treat existing passing evidence as historical when the tested revision differs.

## 1. Consolidate the playback candidate

Status:in progress; source foundation substantially implemented.

- Use feedback from the delivered Alpha to prioritize reproducible user failures.
- Consolidate native fault ownership, authority fences, shutdown completion,
  PC source admission and Android pairing setup improvements into a reviewable
  source candidate.
- Resolve demonstrated blockers in normal connect/select/watch/recover use.
- Review both apps for duplicated ownership logic, unused dependencies and work
  in playback loops; optimize only when evidence supports a change.
- Keep credentials private, direct TorBox media delivery separate from control,
  and conditional private-Alpha account-bearing delivery explicitly labelled.

Exit:bounded source scope, no known unresolved critical source regression,
appropriate checks passing, exact evidence and unverified paths recorded. Small
source tests and compilation do not establish app usability.

## 2. Validate one revised candidate end to end

Status:explicitly authorized 15 October. Revised Alpha0.1.8 candidate
20261014-195818-f829095c is built from747c2af; hidden packaged startup,12native
emulator fixture tests and normal pairing/save/restore passed. Direct TorBox,
public download/update and physical/WAN acceptance still require their own checks.
Alpha0.1.7 remains offered and preserved until revised delivery is ready.

After build authorization:

- Preserve all previous payloads and verify source/build/signing/payload hashes.
- Run the actual packaged Windows startup and rendered source-selection flow.
- Exercise normal Android code entry, encrypted save, restored connection,
  change-PC/disconnect and failure feedback.
- Verify real TorBox PC and Android native playback:play/pause/seek, buffering,
  background/reconnect, EOF/replay, fault retention and explicit recovery.
- Include revised pairing and native-worker integration; compiled instrumentation
  assertions alone are not a pass.

Exit:one exact candidate passes the required runtime sequence with immutable
receipts and failures preserved. No stitched success from different candidates.
This remains emulator acceptance, not physical-phone or WAN acceptance.

## 3. Establish the physical connection and playback gate

Status:open. Public control tunnel verification=false; physical gate=false.

- Confirm a genuinely reachable HTTPS/WSS control address and normal phone pairing
  over the intended network. Remind that this is Together cloudflared, not a VPN
  adapter. Temporary addresses change after restart.
- Respect the prior public-tunnel automatic policy rejection; do not retry that
  blocked action through another tool. Reachability evidence must come from an
  allowed test path once the restriction is resolved or the user supplies evidence.
- Test one Windows PC and one physical Android phone using separate direct TorBox
  links for the same canonical file. No control-tunnel media proxy.
- Exercise shared controls, buffering recovery, Wi-Fi interruption/reconnect,
  PC restart/new epoch, phone lock/unlock/background/surface recreation and
  source refresh. Only one Android participant per room; close emulator first.
- Assess actual decoder compatibility, heat, battery and hardware decoding on the
  target phone. HEVC/4K support requires specific device evidence.
- Complete a real two-hour playback session on the accepted revision with paired
  PC/phone diagnostics and a clear pass/fail receipt.

Exit:the physical two-hour gate and required recovery scenarios pass. Record
network/device/media conditions and versions. Provisional targets from
MILESTONE-1.md:p95 absolute drift≤150ms,p99≤300ms,no sustained>500ms for>5s outside
barriers. Assess presentation latency and clock/property uncertainty; sampled
position estimates and cheap native getters are insufficient to prove them.

## 4. Complete the beta feature and design scope

Status:deferred until the physical playback gate passes. Authorized PC Stremio
browse/source-selection foundation already exists; full polish remains.

- Inventory Together1.0 and Pocket as read-only references; produce a concrete
  must-have parity checklist and explicitly identify deferred items.
- Implement the agreed rebuilt chat, library/discovery and any required Pocket
  integration after the gate. Preserve playback ownership and direct delivery.
- Complete PC Stremio usability, real covers, navigation and error/recovery flows.
- Establish stable connection provisioning and clear credential rotation/revocation
  behavior; named stable tunnel provisioning is currently absent.
- Finish persistent pairing and updater delivery as user-facing workflows, including
  signed update metadata/archive checks and Android installation confirmation.
- Create the final2gether visual identity:typography, icons, round/capsule controls,
  liquid-glass treatment and smooth transitions; retain native Compose and minimize
  Android battery/performance cost.

Exit:the agreed full-beta parity checklist is complete, both apps are coherent,
normal flows are rendered and tested, and playback/reconnect regression checks
remain passing. No beta label for a minimal player proof.

## 5. Release candidate and beta delivery

Status:not started.

- Freeze an exact candidate; validate installation/update, persistent pairing,
  source selection, playback/recovery, control lifecycle and diagnostic export.
- Recheck the physical gate after material playback changes; preserve older failures.
- Verify shipped assets, signers and payload hashes; inspect privacy of packages/logs.
- Demonstrate operation without Wintun/vgate0/WARP dependency. Do not change the
  user's adapters,DNS,routes,proxy,firewall or VPN services during development.
- Deliver the complete rebuilt beta with a plain-language guide, known limitations,
  versioned validation receipt and recovery/update instructions.

Exit:actual installed Windows and physical Android experience passes the release
checklist, complete-beta scope is fulfilled and the user can use the apps normally.

## Immediate next checkpoint

The highest-value next checkpoint is current Alpha feedback and an explicitly
permitted revised runtime candidate. Independent source work remains authorized,
but should target demonstrated failures or precise unverified boundaries. Avoid
repeating unchanged cleanup/tests or inflating completion percentages.

Record progress as:implemented → source-tested → native/emulator-verified →
physical-phone-verified → release-ready. Never merge those statuses into one
"done" percentage. Provide outcome, evidence, remaining blocker and next action
for each meaningful checkpoint. Dates depend on feedback, build authorization,
network-policy resolution and physical-phone availability; no reliable calendar
ETA is established yet.
