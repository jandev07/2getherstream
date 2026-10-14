# 2gether 2.0 — external technical review brief

Updated 15 October 2026 IST. Please review this approach critically and identify
concrete correctness/security problems, missing integration tests, or unnecessary
complexity. Separate demonstrated defects from hypotheses. Do not infer current
app usability from compilation or isolated tests.

## Goal and current state

Rebuild a couple-watching app for Windows and Android. Reliable native playback,
PC-authoritative synchronization, direct TorBox media, normal saved pairing and
recovery come before extra features or a full visual redesign. The user selects
Stremio movies/episodes/sources on Windows; Android follows the room.

Offered Alpha 0.1.7 is compiled from 5ea1824. Latest committed source is b289e4a,
with subsequent download/QR/updater/hosting work currently uncommitted. Many
reliability changes since the offered Alpha have not been tested together in a
new installed app. The user temporarily requested a new Alpha, then explicitly
deferred it and resumed sustained development. No new APK/EXE was built or
published in this session. Release packaging is on hold again.

## Architecture and ownership

- Windows: Tauri 2, Rust, Svelte, native libmpv. Android: native Kotlin/Compose,
  OkHttp/coroutines, pinned libmpvkt/libmpvkt-view 0.3.0.
- The PC owns room epoch, generation, timeline, preparation and fault state.
  HTTPS/WSS carries control only. Each native player obtains a separate direct
  TorBox HTTPS URL for the exact same canonical torrent/file; media is not proxied.
- Pairing uses one-use codes, authenticated phone control and encrypted persisted
  credentials (Windows DPAPI / Android Keystore). One Android participant per room.
- Transport generation, media/source revision, socket identity and coroutine
  cancellation fence asynchronous results. Final source publication/grant delivery
  also checks PC authority. Native calls occur outside the authority mutex; these
  checkpoints are not atomic across every subsequent native API operation.
- Native workers serialize native operations. Pending faults persist until the PC
  records the exact fault and an explicit fresh preparation/recovery allows reload.
  Android CAS/status updates preserve newly pending faults; its fault token is a
  fault code, not a unique event counter.
- Android has one in-flight control and one latest waiting action, no automatic
  retransmission. A matching successful ACK may carry the waiting action through
  exactly one same-epoch generation advance caused by its own command. Terminal
  fault/EOF feedback cannot promote an older queued Play into recovery/replay.
- Shutdown completion has independent ownership, so cancelled/timed-out callers
  do not lose eventual native joins. Temporary connection blocking work owns its
  lifecycle mutex, rather than leaving it with a cancellable awaiting caller.

## Recent work and evidence

Implemented source fixes include stale native seek/start/watchdog suppression,
first-fault retention, accurate failed-player status, bounded provider reads,
read-only Stremio cancellation, UI request admission/result ownership, pairing
setup ownership, cancellation-safe HTTP response/artifact cleanup, strict Android
snapshot number/boolean/identity validation, and bounded diagnostic upload receipts.
Most recent PC controls now clamp seek targets for sub-500ms media and ignore
obsolete control replies when displaying the latest result.

The offered Alpha passed packaged startup, normal Android code-entry/save/restore,
fixture tests and a real TorBox PC+emulator recovery sequence. Later changes have
focused JVM, component-script, real loopback server and muted Windows-native
regressions, with failed baselines retained. Several native regressions deliberately
hold workers/feed delivery or induce native position/command errors. These are
not reproduced physical-device decoder/GPU failures.

The physical-phone two-hour gate remains open. WAN/public playback-tunnel testing
was rejected by automatic approval with only 'blocked by policy' and was never
executed or retried. Native/emulator proofs are not physical/WAN acceptance.
Historical sampled steady p95 drift was 133ms in one real-provider emulator run;
this is not a general 150ms guarantee. A fixture showed 100ms reported position
steps even though native getters were fast. Observation time is not frame/PTS
freshness or presentation latency. Planning estimates remain 75% playback /
35% full beta, not measured acceptance.

## Current download/update work — proposed, not deployed

The user requested a permanent Android download link and QR in Windows while
preserving the built-in updater. Their separate Velvet project already uses a
persistent Cloudflare Worker for small static APK assets; it is read-only reference.

A separate planned 2gether-download Worker will serve only public APK/download
metadata, never playback/control/media or credentials. No new tunnel is required.
The APK exceeds Cloudflare's 25MiB single-asset limit; R2 is not enabled and would
require billing setup. Proposed design: versioned <=20MiB binary parts in Workers
static assets, streamed sequentially with backpressure as one immutable HTTPS APK
response using FixedLengthStream. Mutable release.json points to a version-specific
APK URL; a permanent landing page is the Windows QR target. Range requests are
currently rejected explicitly. No host has been deployed, no real hosted APK
transfer verified, and a persistent URL does not guarantee service uptime.

Windows QR generation uses qrcodegen 1.8.0, a new small no-transitive-dependency
library; generated SVG contains only numeric geometry, a white background and
four-module quiet zone. QR decode/render validation is pending. Android's new
public metadata check is intended to work independently of PC availability;
the existing authenticated paired-PC check remains available. All existing
metadata bounds, HTTPS/no-redirect/no-auth APK download, package/version/hash and
installed signer checks remain. User confirms installation; pairing is retained.
The public metadata trust transition and release publication workflow need review.

## Questions worth reviewing

1. Is ACK-based one-generation intent carry sufficiently causal under concurrent
   PC commands, READY/buffering/fault/EOF feedback and snapshot-before-ACK delivery?
2. Can same-code native faults or the interval between authority checkpoint and
   native call still clear/revive stale work? Which realistic integration tests
   best address the remaining gap without broad abstractions or extra polling?
3. Are repeated/quantized native position samples handled appropriately, and what
   frame/presentation measurements should complement position-based drift checks?
4. Is the proposed chunk-streamed Worker APK hosting robust under aborts, corrupt
   or missing parts, cold requests, free-plan resource limits and cache changes?
   Would another non-expiring hosting option be safer/simpler without billing setup?
5. Does public metadata plus strict APK signer/package/higher-version/hash checking
   provide the desired update trust model? How should rollback, manifest/APK race,
   immutable version publication and failed download/install recovery be tested?
6. Have many isolated source fixes created too much integration risk? What should
   be frozen and checked in the next exact PC/Android candidate before more work?

Full local evidence lives in docs/DEVELOPMENT.md and work/source-validation;
docs/ROADMAP.md and docs/MILESTONE-1.md separate source, emulator, physical and
release gates. Credentials, signers and provider URLs must never be shared.

## Review follow-up — 15 October 2026

This document above records the brief supplied to external reviewers; its hosting
proposal is historical. Claude, DeepSeek and Antigravity reviews were checked
against source rather than treated as instructions. The existing serialized
native owners and server-atomic ACK result do not justify a broad actor/protocol
rewrite. Matching fault codes are not unique event IDs, but the cited same-code
load/event path did not establish a new failure. Android runtime ordering remains
a validation boundary; readiness is not proof of frame presentation.

The queued-intent carry now explicitly lists commands capable of advancing
generation, excluding READY/Executed/unknown actions from hypothetical +1 carry.
This clarifies the existing server contract; no bad production ACK was reproduced.

Hosting is now a small Cloudflare landing page/public metadata endpoint pointing
to versioned GitHub Release APK assets. Chunk assembly was removed before any
public deployment. The Android downloader uses a bounded GitHub-only redirect
chain without credentials; control and metadata clients still reject redirects.
Range resume is not implemented. See RELEASE-HOSTING.md for exact publication and
validation boundaries. The user has authorized one significant revised Alpha,
which must be source-associated and checked as an installed candidate before
delivery. Physical-phone, two-hour and public playback gates remain open.
