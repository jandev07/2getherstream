# TorBox integration checkpoint

## 9 October 2026 public documentation follow-up

Official [download troubleshooting](https://support.torbox.app/en/articles/15315517-why-are-my-download-links-not-working)
describes temporary download links, a three-hour lifetime, presigned-token errors,
and using a separate link for each app/device. This supports independent device
leases and renewal planning, but does not prove the tested account-bearing URL
can safely be delivered or that every CDN returns a distinct link.

The current [OpenAPI](https://api.torbox.app/openapi.json) exposes createstream
with server-side authentication and getstreamdata with token/presigned_token query
parameters. Descriptions do not establish the exact returned capability contract,
original-file transport, or scope of those tokens. The previous createstream HTTP
500 remains the only live result; no new authenticated probe was made here.

Next investigation should distinguish an intermediate account-bearing download
URL from any final file-scoped redirect capability, on the PC only. This is a
hypothesis, not a validated bypass: do not send the intermediate URL to Android,
automatically forward credentials to unknown hosts, remove query fields by guess,
or relax validate_stream_url. Before a future bounded probe, define provider/CDN
origin policy and safe redirect handling; accept only a final HTTPS URL passing
the existing key guard, correct ranges/size and original-file identity checks.
Inspect the documented stream response shape only with bounded, redacted PC
diagnostics and scrobbling off; do not adopt transcoding as an implicit fallback.
No safe capability, expiry contract or provider compatibility is claimed.

The account key is stored only in ignored `.secrets/torbox.dpapi`, encrypted for
the current Windows user with DPAPI. It is entered through a hidden prompt.
The Rust resolver decrypts it in memory and clears the decrypted DPAPI buffer.
Never copy this file to another machine, source archive or APK.

The PC-only source-resolver crate lists existing available video files, paginates
the library, skips absent file lists and unknown/infected scan results, and uses
provider/torrent ID/file ID/hash/name/size as source metadata. API errors and CDN
errors are redacted. Responses are bounded to 8 MiB and requests to 25 seconds.
Redirects are disabled. No torrent is created, modified or deleted.

## Live discovery on 2026-10-08

The supplied credential authenticated successfully. Rust listed 1,614 available
video files. The tested `requestdl` response returned an HTTPS CDN URL whose
`token` query parameter contained the **account API key itself**. An independent
shape-only check confirmed this; no URL/key was printed or saved.

The resolver rejects this response before creating a deliverable StreamLease.
Therefore the two-device live CDN proof has **not passed**. No account-bearing
URL was transmitted to the emulator. The existing native session fixture remains
the only connected playback proof.

## Required resolution

The requested requirements are both: original media streamed directly to Android,
and account API key remaining solely on the PC. The tested TorBox direct URL
cannot currently satisfy both. Need a provider-supported file-scoped download
capability that does not expose the account key. Do not remove the guard, place
the token in an Android header, invent an undocumented signed URL, proxy video
through Cloudflare, or silently switch to transcoding.

TorBox's separate stream API exists, but its direct-original quality and credential
properties have not been established. It is not a validated substitute.

The two device request method intentionally makes separate requestdl calls for
the same identity, with optional per-device public IPs. Equal URL strings may be
a legitimate provider response; different strings are not assumed or fabricated.
For now the guard stops at the first unsafe response.

## Android HTTPS preparation

The Android native adapter now exports Android system CA certificates to an app
private PEM bundle for libmpv, keeps `tls-verify=yes`, and supports generation-
fenced HTTPS loads. User-installed roots are excluded. This is player-boundary
preparation only; TorBox source delivery and refresh are not yet connected to
the authoritative session controller. Native HTTPS load/play/pause/exact-seek
tests now pass on Windows and the emulator with a pinned public AndroidX fixture.

A follow-up read-only CDN request with the token removed returned HTTP 400.
Simply stripping the account token is not a working solution. TorBox's official
web-streaming documentation describes both raw browser-compatible playback and
transcoding/chunked processing; that is insufficient evidence of a file-scoped,
original-file download capability. The stream API is still under investigation.

The identity type is now shared with sync-core and carried by authoritative
snapshots, including optional duration/IMDb/episode metadata. Live delivery is
still blocked before a TorBox room is prepared.

A live alternative-stream investigation used an existing available MP4 and the
documented createstream endpoint with scrobbling disabled. It returned HTTP 500.
No usable file-scoped/original-file capability was established. This is one
provider error, not proof that a safe capability cannot exist. No stream URL or
account key was delivered to Android; no proxy/transcoding fallback was adopted.
Reference: [TorBox web streaming](https://support.torbox.app/en/articles/12662996-torbox-web-streaming).

## Reproduce

Use the local toolchain script, then run `cargo test -p together-source-resolver`.
The explicit live diagnostic is `cargo run -p together-source-resolver --example
torbox-proof`. It lists counts only and requests a link for the first available
video. It currently fails closed with the credential-in-media-URL error.

References: [official OpenAPI](https://api.torbox.app/openapi.json),
[official main API documentation](https://api-docs.torbox.app/).


## Range-probe validation

The PC-only 1024-byte probe now requests identity encoding and validates one
Content-Range reporting exactly bytes0-1023 with a known total. If canonical file
size is available, the total must match it. Compressed, missing/duplicate/malformed
range headers and inconsistent optional Content-Length are rejected before reading
the body. Existing 1024-byte body bound and fixed sanitized errors remain. This
uses [HTTP Content-Range semantics](https://www.rfc-editor.org/rfc/rfc9110.html#section-14.4)
with a deliberately stricter capability contract requiring a known total; it does
not compute a content hash or prove that equal-sized files are identical.

Eight resolver tests pass. New header cases cover wrong offsets/lengths/totals,
overflow, unknown total, duplicates and compression. A local TCP fixture exercises
the actual probe request/response path and confirms the Range/Accept-Encoding
request plus matching-size acceptance and mismatched-size rejection. Only this
private test constructs an HTTP lease; production lease validation still requires
HTTPS. No live account/API/CDN request or saved credential access was performed.
The unsafe account-key URL blocker remains unchanged; this does not establish safe
TorBox playback or source refresh.


## Unverified alternative: T3 signed capabilities

[TorBox's official T3 documentation](https://support.torbox.app/en/articles/15531689-torbox-t3)
(updatedJuly2,2026) describes a read-only S3-compatible service at t3.nexus, using
TorBox AuthID as access-key ID, APIkey as secret key and V2 signature mode. This
suggests investigating file-scoped presigned capabilities generated on the PC;
compatibility wording alone does not establish query signing support. Object-to-
torrent/file mapping, redirects (including any account-key exposure), original-file
ranges and URL renewal still require verification. Keep current guard unchanged.
The page also states15min refresh and currentlyCloudflare-hosted serving. No live
account/T3 request, implementation or delivery was performed for this research.

## 2026-10-08 experimental T3 investigation

An independently written, feature-gated PC signer now implements a narrow S3 V2
presigned GET for one decoded bucket/object at the fixed HTTPS T3 origin. It signs
the exact encoded path with HMAC-SHA1/Base64, bounds lifetime to30–900 whole seconds,
rejects ambiguous paths and retains the existing repeatedly decoded account-key
URL guard. Experimental bearer types have no Debug/Serialize implementations.
Neither application enables `experimental-t3`; this code creates no StreamLease.
It is protocol research, not copied Syncplay/Synkplay/AWS implementation code.

References: [AWS S3 V2 query authentication and canonical resource rules](https://docs.aws.amazon.com/AmazonS3/latest/developerguide/RESTAuthentication.html#RESTAuthenticationQueryStringAuth),
[RFC2202 public HMAC-SHA1 test case1](https://www.rfc-editor.org/rfc/rfc2202#section-3).
The complete GET signature test uses AWS's public synthetic credential/path input
and an independently calculated Python stdlib HMAC result. The current AWS page's
renamed bucket and printed example signature disagree; do not treat its displayed
signature as a passing vector for the renamed path. No AWS source was copied.

Fifteen resolver checks pass with the feature enabled, including seven experimental
checks for HMAC, exact encoding, signature scope, expiry bounds, account-key
rejection and guarded relative/same-origin redirects. The default eight resolver
checks also pass. These are offline checks, not proof of provider support.

Explicit live diagnostic uses existing DPAPI credentials on the PC only:
`cargo run -p together-source-resolver --features experimental-t3 --example t3-proof`.
It performs read-only profile/listing/range requests, emits fixed labels/counts/
status codes only and never prints or exports account IDs, filenames, URLs,
headers or response bodies. No Android delivery or account mutation occurs.
Metadata is bounded to2MiB, the inspected listing subset to50, error-JSON reads
to1024bytes. Automatic redirects are disabled; at most three explicit redirects
may pass HTTPS/account-key validation and must remain on the same T3 origin.
External redirects are unverified and rejected. The diagnostic doesn't establish
canonical torrent/file identity; a T3 object listing is insufficient for that.

Observed profile and header-authenticated fixed torbox-bucket listing HTTP200.
The listing returned1000entries/truncated despite max-keys50; only first50 were
inspected. One video candidate's presigned request returned307 with a relative
Location, followed by206 after one guarded same-origin redirect. That response was
HTML, declared bytes0–464, and did not match the requested0–1023 or listing size.
It was rejected without accepting original media. Expired and deliberately invalid
signature requests also ended with206 after guarded redirects. This does NOT prove
an authentication bypass: the responses may be error/login/cache behavior and are
not validated media. Provider acceptance of query signing, authorization scope,
expiry rejection and original-file delivery remain unproven.

The final diagnostic exits1 deliberately; sanitized evidence is kept at
`work/t3-capability-investigation-sanitized.log`. No safe T3 capability was
established, no movie was proxied, and account-key guards were not relaxed.
Do not wire T3 into playback or repeat identical live probes without a new lead.
Direct safe TorBox delivery and the physical-phone two-hour gate remain open.

## 2026-10-09 fixed API redirect inspection

The documented [requestdl redirect option](https://api-docs.torbox.app/) can be
inspected separately with the experimental-download-redirect feature and
`cargo run -p together-source-resolver --features experimental-download-redirect --example download-redirect-proof`.
Credentials go only to the fixed API origin; automatic redirects remain disabled.
The diagnostic emits status plus a fixed Location classification, never returns
or follows Location, reads no media body and creates no deliverable lease. A
credential-free classification alone would not prove provider origin or file scope.
Existing key/HTTPS/API-origin guards remain unchanged. Neither app enables this.

Ten resolver checks and example compilation pass. One live PC-only request for an
existing available video returned HTTP307/RejectedLocation. No returned URL was
followed, exposed or sent to Android. This report does not distinguish which guard
rejected it, and no safe final capability was established. Do not repeat the same
probe without a new lead; direct original-file delivery remains blocked.

## 9 October typed redirect rejection follow-up

The experimental PC diagnostic now emits only fixed rejection categories instead
of an ambiguous RejectedLocation: invalid header/URL, API origin, account
credential, unsafe URL or nested encoding. The production URL validator and
redirect policy are unchanged. Synthetic rejection/redaction checks pass;
11resolver checks pass and the diagnostic compiles. Neither app enables it.

One newly instrumented read-only probe was justified by the remaining ambiguity
about which guard rejected the earlier307. It returned307 with
RejectedLocation(AccountCredential). The final redirect itself contains the
account credential according to the existing case/percent-decoding guard. No URL
was followed, printed, exported or delivered to Android. This establishes the
rejection cause for that request, not that all provider paths are impossible.
Do not repeat it without a different provider lead.

The official Main API documentation's token refresh/device authorization entries
do not establish a file-scoped download credential contract. Public support search
also did not identify such a contract. Do not rotate the account key, treat device
authorization as file scoping, or copy a website/account token into Android by
inference. Provider clarification or a validated alternative remains necessary.

## Source-selection infrastructure

The PC resolver now provides selection transactions with supersession/cancellation
and atomic publication of two matching per-device leases. Production link requests
run concurrently, cancelling their sibling on failure; all existing URL guards
remain in force. Same-file refreshes also invalidate previous delivery receipts.
Only validated StreamLease objects enter this path; there is no public constructor
accepting arbitrary URLs or a test-provider option in the apps.

This infrastructure does not establish a safe TorBox capability or enable either
app's media resolver. Phone delivery/authentication, native epoch/media fencing and
lease refresh/reconnect/expiry handling remain required. Links already taken by a
caller cannot be revoked locally; final native-load checks remain the controller's
responsibility. Tests use synthetic capabilities and deterministic async barriers.

The subsequent handoff fence changes take() to return SelectedLease. URL access
requires its consuming deliver() callback to pass a current-owner/selection check;
replacement cannot interleave with that short synchronous callback. clear() also
invalidates pending/queued delivery while allowing a later source selection.
Do not retain a copied URL or return a future for later unfenced use. The native
controller still checks epoch/media when a queued load actually executes, and
already delivered capabilities cannot be revoked by this local check.

## 10 October URL boundary and public contract review

The PC resolver and both native guards now reject DNS-root-dot and encoded API
host aliases. Original raw URL syntax is also checked before publishing a PC
lease; parser normalization cannot discard an empty userinfo delimiter, raw
whitespace/control bytes or backslashes and thereby authorize delivery. Redirect
classification shares the canonical API-host check. Synthetic tests preserve
ordinary CDN root dots, encoded spaces and distinct suffix domains. Native HTTPS
checks require rejected aliases to preserve the currently loaded source and
receipt. No actual account credential or API media request was used for this work.

Rechecked public documentation on10October. The official
[JavaScript SDK download-link documentation](https://github.com/TorBox-App/torbox-sdk-js/blob/main/documentation/services/TorrentsService.md#requestdownloadlink)
still describes the account API key as the request token and a key-bearing API
permalink as its redirect mechanism. It describes temporary CDN links but does
not, in the reviewed section, define a separately issued file-scoped credential.
Its duration wording mentions both three hours and one hour, so no new precise
expiry guarantee was inferred. This review provides no new lead justifying a
repeat of the rejected private probes. The prior AccountCredential rejection
remains unresolved; these stronger URL guards do not establish safe TorBox
delivery, connect the app resolver or satisfy the physical-phone gate.

## 12 October private Alpha delivery exception

The fresh PC-only createstream inspection returned HTTP500, no file/presigned
 tokens and zero candidate URLs. This supplements earlier account-bearing
requestdl/redirect findings, failed token removal and unverified T3 signing; it
 does not prove every possible provider alternative is impossible. No returned
stream URL was followed or delivered.

Under the user's explicit conditional authorization, source now provides an
opt-in private-alpha-account-links feature. TorBox::new remains strict even when
compiled with all features. The separate private constructor first prefers
credential-free links, then accepts the exact known account key in one token
parameter only on provider-published TorBox CDN suffixes over HTTPS443. API origins,
userinfo, fragments, duplicate tokens, root-dot aliases and keys elsewhere remain
rejected. The UI states that the paired phone receives account-bearing links.
Keys are not embedded in packages or exported to logs; this is not file scoping.

CDN ownership was checked against official support:
https://support.torbox.app/en/articles/12667085-torbox-is-being-blocked-by-my-anti-virus-software
https://support.torbox.app/en/articles/13181686-accessing-torbox-with-nextdns
No DNS, VPN, antivirus or system network changes were made.

One real private-policy proof issued separate links for the same canonical file
and received exact HTTP206/Content-Range/identity-encoding responses: 1024bytes for
PC and 1024bytes for the phone candidate, both fetched on PC. No Android handoff,
actual movie decode, WAN playback or physical two-hour acceptance occurred.
See STREMIO-PLAYBACK.md for catalog/copy-source integration and remaining limits.
