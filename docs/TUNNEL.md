# Control-only tunnel configuration

`ControlConfig` renders one exact public hostname and `^/(session|pair|health|media|updates/android|diagnostics/phone)$`
to `http://127.0.0.1:47820`, followed by `http_status:404`. The origin, regex and
default deny rule are fixed. No arbitrary TCP, wildcard host, private-network
or media-byte routes can be supplied. `/media` returns authenticated, generation-
bound phone playback metadata only; libmpv fetches video bytes directly from the CDN.
It cannot proxy or accept arbitrary URLs. `/pair` was missing from the old reference
example and is required for initial pairing. Application bearer authentication,
one-use pairing windows and rate limits remain in force.

The builder rejects invalid/nil UUIDs, IP/scheme/wildcard/path/port hostnames,
bad DNS labels and relative/control-character credential paths. YAML quoting
escapes Windows paths with spaces, backslashes, quotes and `#`. It does not read
credentials or tokens; the config type has no Debug/Serialize implementation.
Rendered configuration is a local setup artifact, not a diagnostic payload.
Use dedicated Together 2.0 tunnel credentials; legacy configuration is read-only.

The `tunnel-config` example accepts hostname, UUID, absolute credential path and
a new absolute `.yml`/`.yaml` output path. It renders config and refuses existing
output files. It does not create DNS, start cloudflared, read credentials or
contact Cloudflare. Domain/tunnel provisioning is still pending.

The format follows Cloudflare's documented ordered hostname/path matching and
required catch-all rule in its
[configuration guide](https://developers.cloudflare.com/cloudflare-one/networks/connectors/cloudflare-tunnel/do-more-with-tunnels/local-management/configuration-file/).
That page also documents offline ingress validation and rule matching.

Actual cloudflared offline validation passed using a copied existing executable
(SHA-256 matched the read-only reference), fake identity/hostname, nonexistent
fake credential path and new config under ignored `work/tunnel-validation`.
No legacy config/secret was read or changed and no tunnel process was started.
The generated credential path containing spaces and `#` parsed successfully.

| Offline rule check | Destination |
|---|---|
| Correct hostname `/session`, `/pair`, `/health`, `/media` | Local control origin |
| Correct hostname `/media/movie.mp4`, `/session/other` | 404 |
| Different hostname `/session` | 404 |

Four builder tests and all 24 server tests pass. A pre-existing malformed-input
test race was corrected: heartbeat RTT publication may advance revision while
input is rejected. It now compares all state except revision/participant RTT,
retaining generation, timeline, media, preparation and fault invariants.

This module is not wired into app startup. Automatic owned-process lifecycle,
retry, settings, dedicated credentials, DNS, public WSS/pairing, WAN interruption
recovery and physical-phone validation remain pending. Offline ingress validation
does not establish end-to-end authentication or playback reliability.

## Owned process lifecycle (not yet wired to startup)

`OwnedTunnel::start` accepts an explicitly selected absolute executable, an
existing private runtime directory, and the reviewed `ControlConfig`. It writes
a random new config with create-new semantics, then directly launches
`cloudflared tunnel --no-autoupdate --config <owned-config> run`. The copied
cloudflared executable's local CLI help confirmed these tunnel options.
No shell or token argument is used. Child environment is cleared except
SystemRoot/WINDIR/TEMP/TMP, preventing inherited TUNNEL_* or proxy overrides.
Standard streams are disconnected and Windows uses CREATE_NO_WINDOW. The
executable must eventually be installed/verified by the application; existence
checks here do not establish binary provenance.

The retained child handle owns only that process. Explicit stop and Drop kill
and reap it; stopping is idempotent. They never enumerate processes, kill by
name, touch a legacy tunnel, or remove any config except the newly owned one.
Failed launch removes its new config. Status reports process running/exit/stopped,
not Cloudflare connection readiness. Errors are fixed codes with no raw process
output or paths. Start/stop perform blocking file/process operations and must
run off the UI/async executor thread. The runtime directory must be private to
the current user; this layer does not establish directory ACLs.

Five lifecycle tests use isolated local test-process children, never a real
tunnel: independent-child isolation, natural exit, Drop termination using a
retained Windows process handle (safe against PID reuse), failed-start cleanup,
and argument/environment construction. The child fixture is excluded from the
normal suite and invoked explicitly by its parent tests. All 29 server tests
pass (19 unit + 10 transport); Windows compile check passes without changing the
active soak executable.

Automatic startup/shutdown integration, restart/backoff, bounded asynchronous
shutdown, abrupt-parent-crash containment, connection health, tunnel diagnostics,
installer verification, directory ACL provisioning, dedicated account/config,
DNS and actual public WSS still remain. No real cloudflared tunnel was launched
by this work. Standard Drop cleanup is not a guarantee after a process crash or
forced termination, and no descendant-process ownership is claimed.

## Background supervisor and retry

`TunnelSupervisor` now owns the process lifecycle on a dedicated worker thread.
Async shutdown requests cancellation and joins that worker through Tokio's
blocking pool; dropping the supervisor only sends cancellation, so it does not
wait on the UI/executor. The async caller has a five-second shutdown deadline.
A timeout leaves cleanup owned by the worker; it does not forcibly terminate a
thread or promise that an OS operation has finished. Drop without awaited
shutdown cannot guarantee cleanup before the application exits.

Failed starts and exited/status-failed children use delays of 1, 2, 4, 8, 16,
then at most 30 seconds. Sixty seconds of process uptime resets the failure
streak on the next exit; this is process uptime, not network health. Cancellation
interrupts both the 250 ms status polling interval and retry delays. The previous
child is stopped/reaped before another start, and stop failure disables retries
rather than risk a second owned child. Config/start/stop still use the existing
restricted configuration and child-handle primitive.

Watch status contains only Starting, Running, Backoff, Stopping, Stopped or a
fixed failure enum. It never contains raw errors, paths, hostname, credentials
or child output. Running means process alive; public reachability/authentication
remain unproven. There is no automatic account, DNS, config or app-startup wiring.

Six added tests cover delay saturation, cancellation during backoff, retry
ordering, Drop cancellation, stop failure, and async-supervisor shutdown of an
actual isolated local child. The real-child test checks the original Windows
process handle and config cleanup. All 35 server tests pass (25 unit + 10
transport, plus one helper fixture intentionally ignored); Windows compile
check passes with active playback executable hash unchanged.

Remaining work includes actual startup/shutdown integration, abrupt-crash
containment, installer verification, private-directory ACLs, production tunnel
provisioning, connection health, structured journal integration and WAN tests.
No real cloudflared tunnel was started, and process tests are not playback or
physical-phone acceptance.

## Lifecycle journal integration

The optional `start_observed` API submits typed transition events to the Host
journal on the supervisor worker, before publishing watch updates. This avoids
losing rapid transitions solely because watch subscribers see only the latest
status. A weak Host reference avoids retaining the server. Enable diagnostics
before using it; bounded queue drop/error counters still apply. No secret/path/
hostname/tunnel identity/process text is serialized. Running is not WAN ready.
The normal unobserved start API is unchanged.

Observed public-API start-failure/export and weak-Host lifetime tests pass;
all37 server tests pass, Windows metadata passes, active playback exe hash is
unchanged. Tests do not launch cloudflared or use real credentials. This is
source/local-journal integration, not normal-app or public-tunnel validation.

## Windows process-group cleanup foundation

`WindowsJob` creates an unnamed, non-inheritable Windows job with only
JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE enabled. Its owned handle is closed on Drop;
a borrowed handle is exposed for future PROC_THREAD_ATTRIBUTE_JOB_LIST process
creation. No existing process is assigned by the production API, no process is
discovered by name/PID, and no tunnel is launched by this module.

Two isolated Windows process tests prove: closing a job terminates an assigned
child while an unrelated child remains running; forcibly terminating the job
owner (without Rust cleanup) terminates its assigned child and the descendant
created after assignment. The checks retain native process handles before
termination to avoid PID reuse. Job handle inheritance is explicitly checked
as disabled. Test children have cleanup guards on assertion failure. Two helper
fixtures are intentionally ignored by the normal suite and explicitly invoked
by parent tests, alongside the pre-existing process fixture.

This is a tested **ownership building block, not production crash containment**.
The current OwnedTunnel launcher still uses ordinary Command::spawn. Assigning
a job after that call leaves a startup/descendant gap; spawning suspended then
assigning still leaves an orphan window if the parent dies before assignment.
Production integration must use atomic creation with a job-list attribute (and
fail closed if it cannot be established). Rust 1.99 CommandExt's
spawn_with_attributes is nightly-only, so no unstable feature, nightly toolchain
or RUSTC_BOOTSTRAP workaround was introduced. A stable native launcher remains
required. Crashed-owner config-file cleanup also remains a separate concern.

References: [Microsoft job objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects),
[atomic job-list creation guidance](https://devblogs.microsoft.com/oldnewthing/20230209-00/?p=107812/),
[Rust Windows command extensions](https://doc.rust-lang.org/std/os/windows/process/trait.CommandExt.html).
All40 server tests pass (29unit/11transport; three fixtures intentionally ignored).
Windows metadata check passes with active playback executable hash unchanged.
No real cloudflared process, production account/config/credential or legacy file
was used. These tests do not establish public reachability or playback stability.

## Stable Windows atomic tunnel launch

The Windows `OwnedTunnel` library path now uses `NativeChild` and stable
CreateProcessW with STARTUPINFOEXW. PROC_THREAD_ATTRIBUTE_JOB_LIST attaches the
new process to its owned kill-on-close job during creation. There is no
post-spawn assignment interval and no fallback to an uncontained process.
Closing the job after root exit also terminates any surviving descendants.
This supersedes the foundation-only production limitation above; it does not
add automatic tunnel startup to the Windows app.

The executable path is supplied separately from the quoted UTF-16 command line.
Only explicit Command environment entries are included; the production builder
allows SystemRoot, WINDIR, TEMP and TMP and clears inherited settings. A handle
list restricts inheritance to one NUL handle used for all standard streams.
The job handle itself is not inheritable. Native handles and attribute storage
have owned lifetimes; errors remain fixed codes without process output, paths
or credentials. This intentionally narrow launcher is private, not a general
replacement implementing every std::process::Command setting. Non-Windows
builds retain the previous standard child launcher without crash containment.

Three added normal tests exercise actual CRT argument/environment round trips
(including empty arguments, quotes, backslashes and Unicode), immediate exact
job membership plus Drop termination, and forced owner termination killing an
immediately spawned descendant tree. Retained process handles avoid PID reuse.
The full server suite passes 43 tests (32 unit and 11 transport); six helper
fixtures and one opt-in offline test are excluded from the normal run.

The opt-in test also passed separately with a hash-matched copy of the legacy
cloudflared executable: `tunnel --no-autoupdate --config ... ingress validate`.
Its synthetic config used a nonexistent credential file and a path containing
spaces, punctuation and Unicode. The native launcher returned successful exit;
output was discarded. This did not open a tunnel, contact a Together server,
read production credentials, alter DNS or modify legacy files. Windows app
metadata compilation passed with the active playback executable hash unchanged.
The active two-hour playback run uses earlier source and does not test this
launcher; the launcher evidence is the separate process/offline checks above.

Remaining work: normal-app startup and awaited shutdown wiring, installer binary
verification, private-directory ACL provisioning, production account/config/DNS,
WAN health and authenticated public WSS validation. A killed app can still leave
its owned config file behind; process containment does not clean files. None of
these checks satisfy the physical-phone playback release gate.

References: [Microsoft CreateProcessW](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-createprocessw),
[process creation attributes](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-updateprocthreadattribute),
[atomic job-list creation guidance](https://devblogs.microsoft.com/oldnewthing/20230209-00/?p=107812/).

## Additional process ownership regressions

Two more actual-process checks pass. A root fixture exits with code 37 while its
immediate descendant stays alive; observing the root's exit closes its owned job,
terminates that descendant and preserves the same cached nonzero exit status on
subsequent try_wait/wait calls. The other check creates a deliberately inheritable,
exclusively opened sentinel file handle, launches a living child, closes the
parent's handles and successfully reopens the file before the child exits. This
verifies the launcher's handle list excludes that unrelated inheritable handle.

The sentinel check runs inside an isolated helper process. Its first in-process
version failed because concurrent standard-spawn fixtures inherited the sentinel
from the shared test runner, independently of the native launcher under test.
Isolation removes that interference without changing production launch behavior.
The full suite now passes 45 tests (34 unit/11 transport); eight are ignored
(seven explicitly invoked helper fixtures and one opt-in cloudflared offline
check). Windows metadata compilation passes and the active player executable
hash is unchanged. These tests extend process ownership evidence, not public
Cloudflare or physical playback acceptance.


## Optional normal-app startup wiring

Normal Windows startup now checks its app-local-data `control-tunnel` directory
for `deployment.json` after binding the local control listener and enabling the
journal. Missing configuration leaves local development available. Invalid or
unreadable provisioning disables the tunnel with a fixed error; no raw manifest
or credentials enter status/diagnostics. Debug native-session proofs always skip
this configuration, even if it is installed.

The dedicated directory layout is fixed:

- deployment.json: version1, hostname, tunnel_id, executable_sha256 only.
- cloudflared.exe: dedicated executable, never found through PATH.
- credentials.json: dedicated Together2 named-tunnel credential file.
- runtime/: only generated create-new ingress configurations.

The JSON manifest accepts at most16KiB, rejects unknown fields, requires a64-digit
ASCII hex SHA-256 and uses the existing strict hostname/UUID validation. No path,
origin, route or token override is accepted. A future trusted installer/provisioner
must install the verified release digest and protect this directory and credentials
with appropriate ACLs. **A hash supplied by a local manifest is an integrity pin,
not independent proof of publisher authenticity.** No such production manifest or
credentials were installed by this development change; DNS remains unprovisioned.

Before every launch/retry, the background worker hashes the dedicated executable
(max128MiB) and rejects mismatches/empty/missing images before writing runtime
config. On Windows it retains a read handle with FILE_SHARE_READ through process
creation; ordinary writes/deletion of that file fail while held. This follows
[Rust Windows share-mode semantics](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode).
The guard does not harden an attacker-controlled directory namespace, establish
installer ACLs or defend against a privileged local attacker. The existing atomic
job creation, environment/handle allowlists and control-only routes remain in use.

The app retains the supervisor and awaits its existing five-second shutdown before
player/server cleanup, recording typed tunnel_stopped/tunnel_stop_failed stages.
Observed supervisors also stop when Host stops/disappears, checking at most250ms
apart during long backoff; normal server task completion requests Host shutdown.
An already executing blocking launch/hash is not forcibly interrupted. A timeout
leaves worker-owned cleanup pending. The local Tauri tunnel_status command reports
unconfigured/setup_failed or configured process status only; Running is not proof
of WAN readiness. No public endpoint or sensitive setup details are added.

Validation:57server checks (45unit/12transport;8ignored), three actual-libmpv
Windows tests and Windows compilation pass. Added tests cover missing/invalid/
oversized/unknown-field/non-ASCII manifests, hash match/mismatch, missing credentials,
no runtime artifacts on failed validation, and actual Windows write/delete denial
while the verified file is held. A controlled supervisor test proves Host shutdown
stops a running fake child and interrupts30sec backoff before explicit supervisor
shutdown. Existing real-child/job/transport tests remain passing. Normal configured
app startup/close and public WSS are not exercised by these component tests.
Installer ACLs/trusted binary provisioning, DNS/account setup, WAN health, stale
crash-config cleanup and physical-phone acceptance remain pending.


## Connector working directory and successful local image launch

OwnedTunnel now supplies its dedicated runtime directory as the child's working
directory, instead of inheriting the app launcher's directory. The native launcher
already supports explicit UTF-16 current-directory storage through CreateProcessW;
its cross-process echo regression now verifies the observed working directory as
well as arguments/environment. This removes inherited-directory dependence; it
does not provision or validate runtime directory ACLs or all DLL dependencies.

A Windows deployment integration test copies the current Rust test harness to a
new temporary cloudflared.exe path, pins its actual hash, installs only a synthetic
credential placeholder, and successfully creates that real native image through
the verified deployment path. The harness rejects the fixed cloudflared arguments
and exits nonzero; the test observes exit and verifies idempotent stop removes the
owned ingress config while preserving the placeholder. It proves verified-file
sharing and native image creation work together. It does not execute cloudflared,
read actual credentials or make a public tunnel/WAN connection.

All58server tests (46unit/12transport;8ignored), three Windows actual-libmpv tests
and the full Windows native build pass. Provisioned normal-app/WAN success and
physical-phone acceptance remain pending.


## Reject existing redirected deployment paths

Deployment loading and each launch now inspect the manifest/executable/credential/
runtime path ancestors with symlink_metadata, rejecting Windows reparse points
(and symlinks on other platforms). Runtime is checked before and after creation.
This is read-only path inspection; it does not modify permissions or read credential
contents. Existing junctions cannot redirect startup/config writes to another tree.
This is not a locked namespace or protection from concurrent privileged path swaps.
A real Windows test creates new temporary directory junctions, verifies redirected
manifest/runtime rejection and unchanged target sentinel, then removes the links
without recursing through them. All59server tests (47unit/12transport;8ignored),
three Windows native-player tests and full Windows build pass.

A proposed create-only private-directory setup helper and temporary permission
test were rejected by automatic approval review before execution with only
"blocked by policy". Neither script was written or run; no ACL was changed and
that action was not retried through another route. Private-directory permission
provisioning remains pending. The read-only redirect guard is independent work.
