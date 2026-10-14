# Local session journal

The Windows app enables a production JSONL journal after successfully binding
its local control listener. A second app instance cannot rotate the active
host's journal before failing to bind. The journal is local; there is no public
HTTP/WSS diagnostics or file-export endpoint.
Debug native proof hosts use an ignored `work/native-journal-proof` directory
so test startup/rotation cannot replace a normal user's retained journal.

The dedicated writer receives a bounded queue of 256 typed records. Playback
and authoritative state producers use nonblocking `try_send`; a full or closed
queue increments a drop counter instead of delaying playback. Serialization,
disk writes, rotation and export all run on the writer thread. Buffered writes
flush at least every 250 ms while the worker makes progress, including under
continuous input. A blocked disk can delay the worker, but not the producers.
An abrupt process exit may lose the buffered tail. This is diagnostic evidence,
not a transactional state store or crash-proof playback history.

Retention is eight files of at most 8 MiB each (64 MiB total), oldest-first
rotation. Startup rotates existing files and preserves the retained tail from
previous runs. This is a rolling history, not a guarantee that every record of
an arbitrarily long session survives. Every generated JSON row includes schema
version 1, a journal run UUID, increasing accepted-record sequence number, wall
time at serialization, a cumulative dropped-record count and a typed event.
Rows also include the cumulative storage-error count; the native proof rejects
fresh journals reporting dropped records or storage errors.
Control timestamps remain monotonic milliseconds since the PC server boot.

Current events:

- `control`: authenticated socket PC/phone actor, monotonic server time and a
  fixed event kind: connected, disconnected, server stopping, malformed request,
  unsupported clock protocol, frame rate limit or control heartbeat timeout.
  A termination cause event precedes the disconnected event and resulting
  authoritative snapshot; shutdown uses `server_stopping` as its closure event.
  Local in-process PC attachment is represented by session records, not socket
  events. Generic socket disconnects do not distinguish peer close, read failure
  and write failure; peer close reason text is never copied.
- `session`: authoritative generation/revision, target timeline, user play
  intent, preparation target/deadline, readiness, buffering, connections, RTT,
  native execution acknowledgements, terminal player fault and session fault.
- `native_pc`: sampled position/target, generation, pause/seek/buffering state,
  source readiness, rate and drift, sampled at most every 100 ms.
- `command`: fixed PC/phone actor, fixed action kind, client sequence (null for
  local PC calls), submitted and resulting generation, revision, optional numeric
  position and an allowlisted outcome. Accepted commands precede their resulting
  snapshot record. Rejected commands are recorded without mutating authority.
  Stale generations/epochs, duplicate sequences, host-only operations, readiness
  and execution rejection reasons have fixed codes; unknown reasons become
  `other_rejected`. Request JSON, submitted epoch strings and media metadata have
  no command-event field.

Projection into diagnostic types happens before enqueueing. Whole snapshots,
credential containers and arbitrary error strings are never serialized. Media
correlation is limited to provider and numeric torrent/file IDs. Filenames,
torrent hashes, account keys, pairing tokens/codes, media URLs, tunnel secrets,
raw mpv messages and chat text have no journal field. Session fault strings are
mapped to fixed codes, with unknown future values recorded as `unknown_fault`.
This restriction covers application-generated records; locally edited files
are not authenticated or tamper-proof.

The proof UI shows writer availability, cumulative dropped records and storage
errors. `Export session log` asks the writer to flush and combine the retained
files in order. The bounded export request can fail when the queue is full and
has a ten-second wait limit. The local Tauri command writes an explicit export
under the app-local `diagnostics/exports` directory and displays its path. Export
files are user-requested copies and are not deleted by rolling-journal retention.
Fixed error codes are returned on failure; provider/native error details do not
enter the export status. The export is a prefix through the queued export
request; later records remain in the journal for the next export.

Android native samples and clock-offset estimates are still collected by the
instrumented native proof, not by this production journal. Pre-upgrade
authentication rejection, native callback traces, URL-renewal outcomes and tunnel lifecycle events need
additional typed event integration. The existing proof logs/analyzer remain
separate so diagnostic development cannot alter an active stability run.

Validation covers bounded rotation and restart retention, ordering/export flush,
queue overflow, inaccessible storage, exclusion of untrusted metadata and faults,
and actual Host state publication. Windows metadata checking and frontend
checking validate wiring. The 200.916-second native suite (run 956cac57) passed
with actual authority/native journal records, zero drops and zero storage errors.
Its concurrent journal reader and receipt finalization also passed. The local
Tauri export UI still needs runtime verification; Host ordered export is tested.
The command-event extension is covered by local and real authenticated WebSocket
tests (accepted and rejected actions, privacy and stale-authority immutability).
It was added while run d8d2bfd2 was active and is not in that run's executable;
a fresh native proof is required after the owned soak exits.

The control-event extension passes an actual WebSocket test covering malformed
input without authority mutation, unsupported clock protocol, six-second client
silence, a frame flood and active server shutdown. Export contains fixed events
and excludes synthetic credentials, request text and URL/token markers, with
zero queue drops/storage errors. All 20 server tests and Windows metadata
checking pass. This extension is also source-only during run d8d2bfd2 and awaits
a fresh native build/runtime proof.

Follow-up native run 807f66ff against e9d73b9 passed five tests in 205.751
seconds. Its fresh journal has 127 command, 229 session, 2,078 native_pc and
eight control events (four connections/four disconnections), zero drops/storage
errors. Copies are retained in work/passed-short-807f66ff. This supersedes the
pending native integration boundary above for ordinary runtime recording;
server-initiated close/restart, malformed/flood/timeout events remain separately
covered by transport tests, not by that native short run. Local export UI and
physical/WAN validation remain pending.

## Tunnel lifecycle records

The observed tunnel supervisor submits transitions directly at their origin to
the Host's existing bounded, nonblocking journal. It does not rely on a watch
subscriber catching intermediate states. `TunnelSupervisor::start_observed`
keeps only a weak Host reference so monitoring cannot extend server lifetime.
Enable the Host journal before starting the observed supervisor. The unobserved
start API remains available; neither API is wired into normal app startup yet.

The new `tunnel` event contains monotonic `server_ms` and a structured status:
starting, running, backoff (attempt, delay_ms, fixed start/status failure),
stopping, stopped or failed (fixed stop failure). There is no free-text error,
hostname, binary/config/credential path, process output, tunnel ID or token.
Running still means child process alive, not public connection readiness.
Journal queue drops/storage failures remain visible through existing counters;
transition submission does not guarantee durable storage or zero loss.

A test uses the public observed API with nonexistent synthetic executable and
credential paths. Its actual Host export contains starting/backoff/stopping/
stopped in order, correct numeric retry fields, zero drops/storage errors and
none of the synthetic sensitive markers, hostname, paths, UUID or raw error.
A second test drops Host while an observed worker is running and confirms the
weak observation does not keep it alive or prevent cleanup. All37 server tests
pass (27 unit +10 transport). Windows metadata check passes with active soak
executable unchanged. Actual tunnel success/exit records and normal UI export
still need runtime verification after provisioning/integration.

## Completed-run clock analysis

`scripts/analyze_clock_proof.py` streams numeric steady-phase clock observations
from a phone proof log. It requires a passed/complete emulator receipt with a
positive completed duration matching the requested duration and a completion
timestamp. It reports selected-clock sample age, retained history size,
consecutive selected-offset jumps within a generation, authority-relative drift,
native sample age, generation count and playback/freshness violations. It does
not compare screen presentation or replace cross-device drift analysis.

The report contains only validated UUID, fixed scope/method text and numeric
aggregates. Input text is never copied; CLI errors are fixed. Output creation
refuses overwrite, including source evidence. Phone log rows have no run ID,
so the caller must retain the corresponding receipt/log pair together; passing
a receipt does not cryptographically prove their association. This tool always
marks physical acceptance false and cannot establish the physical release gate.

Four tests cover exact jumps/statistics, generation boundaries, rejection of
active/incomplete receipts and invalid/reordered numeric observations, reporting
freshness/capacity/playback violations, fixed CLI failure and overwrite refusal.
Historical ten-minute run370285e3 reproduces its archived numeric analysis:
5926steady samples, onegeneration, age30000ms, history64, offsetjump156.5ms,
authoritydrift163ms, nativeage107ms, all counted violations zero. Existing native
runner/analyzer were not modified while long run4d821313 remained active.

## Access rejection diagnostics

Rejected WebSocket authentication, rejected pairing claims and duplicate-role
connections now produce typed `access` records. Each contains only monotonic
server_ms, fixed endpoint (`session` or `pair`) and fixed kind
(`authentication_rejected`, `pairing_rejected` or `role_in_use`). An unauthenticated
request is not attributed to PC/phone and no submitted bearer, code, address,
header or raw error enters the journal. All three outcomes occur after the
existing global upgrade/pair rate bucket, so an already-throttled flood does
not generate one access record for every denied request.

The real transport test exercises missing/wrong/oversized bearer, a denied
HTTP pairing request and a valid phone followed by a duplicate role. Rejections
leave authority unchanged; the authenticated phone still exchanges clock
messages. Export contains five ordered fixed records, excludes synthetic
credentials/code/address, and reports zero drops/storage errors. All38 server
tests pass (27unit/11transport), Windows metadata passes, active native executable
hash unchanged. This source extension is not in the active 732bae22 binary.

Rate-bucket denials, extractor-level malformed/oversized HTTP requests, failed
HTTP upgrade callbacks and server-stopping HTTP responses remain outside this
access event family. No new route, credential behavior or authentication policy
was introduced. Runtime public/WAN authentication remains pending.

## Explicit server shutdown boundary

`Host::request_shutdown()` is now an idempotent local lifecycle API. It signals
the existing stopping watch immediately; control actions are rejected, active
WebSockets use the existing disconnect/freeze plus best-effort 1012 restart close,
and `serve()` requests Axum graceful listener shutdown. There is no public stop
endpoint. A Host stopped before serve rejects startup and releases the supplied
listener; restarting requires a fresh Host/authority epoch.

The session ticker subscribes before shutdown checks, prioritizes stop over ticks
and checks again after acquiring the session lock. Normal serve completion joins
the ticker. Cancellation retains the existing Drop guard to abort the ticker and
notify sockets. Callers must still await the serve task with an application
shutdown deadline: graceful draining of arbitrary HTTP connections is not itself
a bounded shutdown guarantee. Windows app exit integration and coordinating the
tunnel/player shutdown remain pending.

Added unit coverage for stop-before-serve/idempotence and actual transport coverage
for a playing phone: explicit stop closes with restart semantics, preserves play
intent while freezing the timeline, releases the listener, stops subsequent
snapshot changes and releases Host ownership. Existing forced-cancellation and
fresh-epoch reconnect tests still pass. All 47 server tests pass (35 unit plus
12 transport; eight intentionally ignored fixtures/offline tests), and Windows
metadata compilation passes with the active playback executable unchanged.
This is local server lifecycle evidence, not native-player shutdown, WAN recovery
or physical-phone acceptance.

## Windows player and app exit ownership

The Windows controller now retains both its native worker and feedback task.
Explicit stop is nonblocking; asynchronous shutdown waits off the UI thread for
feedback detachment and native-thread teardown, with a five-second caller deadline.
Concurrent callers serialize and receive the same stored outcome. Cancellation
before completion is reported as incomplete on a subsequent call; deadline failure
does not imply a forcibly stopped thread. Drop requests cancellation without
joining on the UI thread. Actual libmpv loaded-fixture tests prove explicit,
concurrent/idempotent and Drop cleanup release the retained Host.

Main-window CloseRequested now preserves the video surface while ExitRequested
runs one asynchronous shutdown task: request Host stop, await player teardown,
await the server with a further three-second deadline, then allow app exit.
Server timeout aborts its serve task, retaining its cancellation guard. Fixed
failure codes alone reach stderr. Tunnel startup is still unconfigured, so this
wiring does not yet coordinate an active tunnel supervisor.

Three Windows actual-DLL tests and all 47 server tests pass; the full Windows
executable builds. Full-window-close runtime validation remains pending: automatic
approval review blocked the attempted scripted close proof before execution,
without a specific reason beyond policy. The loaded headless native tests and
transport shutdown tests do not substitute for that UI test, physical-device
acceptance or abrupt OS/process termination tests.

## Shutdown events and bounded flush acknowledgement

App exit now records typed `app_lifecycle` events: shutdown_requested,
player_stopped/player_stop_failed, server_stopped/server_stop_failed and
exit_permitted. Their only fields are monotonic server time and a fixed stage;
no paths, exception text, credentials or player metadata are accepted. The server
task retains its fixed-code Result so a successful task join containing a server
error is recorded as failure. ExitPermitted means the app allows exit after its
cleanup attempt; it is not a claim that every worker stopped successfully.

After recording the exit decision, the app awaits `Host::flush_diagnostics()`.
The writer processes a FIFO Flush command and acknowledges accepted preceding
records only after flushing its userspace buffer. It reports prior/current
storage errors rather than a false successful acknowledgement. A full/closed
queue fails immediately; writer loss and a two-second acknowledgement deadline
have fixed errors. Playback record producers remain nonblocking. This does not
recover dropped records, flush later concurrent events, join the writer, perform
fsync, or guarantee survival of abrupt termination or power loss. Health counters
still reveal drops/errors. Total normal app cleanup now has the existing player
and server caller deadlines plus up to two seconds for this flush.

New real-file tests read complete ordered shutdown records after Flush without
using Export; queue saturation, writer disappearance and withheld acknowledgement
are exercised. Filesystem startup failure also rejects flush. All49server tests
(37unit/12transport), three actual-DLL Windows tests and full Windows build pass.
Full-window-close runtime validation remains pending from the earlier policy block;
tests establish journal/worker/server components, not the complete UI exit path.

## Shutdown while pairing or attachment waits

Stopping is checked again after awaited pairing, connection-role and session locks.
A pairing claim that was waiting when shutdown occurred returns 503 without
consuming the code or spending an attempt; opening a pairing window returns a
fixed local error. The local pairing-open API now returns Result, handled by both
Tauri UI command and isolated proof startup. Public request/response schemas are
unchanged. Local PC attachment reserves its role only after the state update can
succeed. WebSocket reservation and session join also recheck after lock waits.

Deterministic tests hold locks, poll the operation pending, request shutdown and
release locks. They verify unchanged pairing digest/attempts and absent PC
reservation/state mutation for both local attachment lock waits. All51server and
three actual-DLL Windows tests pass; Windows builds. Existing transport shutdown,
authentication and reconnect tests still pass, but the remote-upgrade wait race
has not been independently forced in a transport test. These guards cover waits;
they do not preempt synchronous work already committing when stop is requested.

## Proof log run identity

New native proof receipts declare `proof_log_schema: 2`. The runner passes its
random run UUID to Windows through `TOGETHER_PROOF_RUN_ID` and Android test
instrumentation through `proofRunId`. Both native observation logs include that
UUID on every row. Debug proof logging rejects a missing/invalid UUID; normal
production playback does not depend on this test-only environment value.

The cross-device analyzer requires an explicit expected UUID and checks every
PC/phone row before filtering or matching. The clock analyzer checks every phone
row against the completed receipt, including setup/recovery rows. New summaries
include `log_identity_verified`; matching IDs establish run association, not
cryptographic authenticity or physical presentation alignment.

Historical archived logs remain unchanged. To analyze unbound historical rows,
use `-AllowLegacyUnbound` with `-ExpectedRunId` for the PowerShell comparison or
`--allow-legacy-unbound` for the Python clock tool. Such reports set
`log_identity_verified: false`; mismatched IDs are still rejected. The Python
clock tool rejects missing IDs in schema-2 receipts even with legacy compatibility.
The normal proof runner never enables legacy compatibility. Run identity is a
non-secret UUID; no account/pairing credential is passed to these analyzers.


## Pairing vault startup boundaries

The Windows current-user DPAPI pairing vault is read through one opened handle,
with a 16 KiB limit enforced on the actual read. Empty, oversized, unreadable,
undecryptable or invalid vaults fail without automatic replacement. Only a missing
file initiates creation. Create-only publication preserves an existing identity;
a concurrent losing creator discards its generated plaintext and reads the
winning persisted vault. Other publication failures remain errors.

Temporary-vault tests exercise real DPAPI, concurrent first launches, retained
identity, tampering rejection and empty/oversized rejection without replacement.
Concurrent launch barriers do not force a specific internal publication ordering.
These checks do not access the saved production vault. Native proof runs use
ephemeral credentials, so their playback results do not establish production
vault startup, normal app close, recovery from filesystem failure, or power-loss
durability. No pairing/account secret is added to diagnostics.


Android pairing storage also enforces its4096-byte limit on the opened stream,
reading at most one overflow byte. Encrypted saves exceeding the same limit are
rejected before starting an AtomicFile write, preserving an existing pairing.
The implementation uses [AtomicFile.openRead](https://developer.android.com/reference/android/util/AtomicFile#openRead())
and closes the stream on all paths. Existing synchronization is per store instance;
this change does not introduce cross-instance/process file locking. JVM bounds
checks and emulator tests with temporary real Keystore entries validate size
rejection, retained credentials, roundtrip, tampering and explicit forget. They do
not exercise full-app startup or physical-phone/power-loss acceptance.


## Control write deadline

A stalled outgoing WebSocket write emits typed control/write_timeout, followed
by the usual connection cleanup/disconnect records. It carries no socket error,
peer-supplied text, address, URL or credential. The five-second write deadline is
independent of heartbeat inactivity and shutdown cancellation. Real local TCP
backpressure, role release and reconnect snapshot are now covered by an automated
test; this does not simulate physical-phone/WAN/tunnel conditions.
