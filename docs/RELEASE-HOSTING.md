# Android download and update hosting

The stable Windows download link and QR target is
`https://2gether-download.vgovind28.workers.dev/`. Android reads public metadata
from `/release.json`; pairing is not required for checking updates. This address
is independent of the temporary watching connection and Windows uptime. Hosting
still depends on the GitHub/Cloudflare accounts and deployment being retained;
it is not a guarantee against account deletion or service outages.

The Worker serves a small download page and typed public metadata only. APK bytes
come from GitHub Releases at a versioned URL:
`https://github.com/jandev07/2getherstream/releases/download/android-v<code>/2gether.apk`.
The Worker does not split, assemble or proxy APKs, video or playback control.

Android manually follows at most three redirects for a canonical GitHub release
asset to GitHub's release CDN hosts, over HTTPS. Requests are rebuilt without
pairing authorization, cookies or referrer. One fifteen-minute cooperative budget
covers the redirect chain and streaming download. Metadata/control clients retain
their no-redirect policies. Downloads currently restart from zero after failure;
Range resume is not implemented or implied by CDN support.

The downloader retains exact size/hash checks, package/version validation and
comparison with the installed app's signing identity. Installation requires Android
confirmation. The stable metadata address is not an independent signing authority.

## Publication order

Public APK/metadata and Desktop delivery are held until16October2026 00:00IST
(15October18:30UTC), enforced by config/alpha-delivery.json and
scripts/AlphaDeliveryWindow.ps1. Internal source preparation/builds and audited
source pushes remain allowed. Complete native source/version correspondence and
retain the source/license materials before binary distribution, as native/README
requires; a source URL alone is not proof of completion.

1. Build and validate a source-associated candidate with a new version code and
   the existing signer. Preserve prior offered packages and failed proof history.
2. Upload the validated APK as `2gether.apk` on `android-v<code>`, retaining earlier
   release assets. Download the public asset and verify its whole size/hash against
   the candidate before exposing it in metadata.
3. Run `scripts/android_download_stage.py <candidate_id>`. It validates local APK
   bytes and writes metadata only, with a process-lifetime publication lock,
   immutable version identities and refusal of latest-version downgrade. It does
   not upload, deploy or establish that the remote asset exists.
4. Deploy `cloud/android-download/wrangler.jsonc` using the existing authorized
   account. Fetch the public page/metadata and validate the updater download path.
   This binary hosting deployment is separate from the blocked public playback
   tunnel test; it does not establish WAN playback readiness.

No secrets, source-private settings, signers, diagnostic journals or development
caches belong in source commits or public hosting assets. Only the APK intended
for delivery, its public source/license materials and bounded public metadata may
be published. Source preparation bundles stay private until their checks and the
publication window permit release.

## Current validation boundary

The download Worker has run locally under Wrangler4.151.0/workerd with the
production compatibility date2026-10-14. Page/metadata requests and typed asset
validation passed. A deliberately missing ASSETS binding reproduced HTTP500 in
the prior handler; the revised handler returns fixed HTTP503 without exception
text. Actual local GET/HEAD page+metadata checks passed, and restoring the binding
with labelled fixture metadata recovered HTTP200. These are local runtime checks,
not a public Cloudflare outage reproduction or deployment. Test fixture manifests
are not actual release assets.

Eight Worker source checks passed (two new binding-failure checks plus six
existing checks). Ordinary-file staging, Android JVM redirect/cancellation checks
and mapped loopback stalled-body tests remain separate. Actual Android archive
verification and an adb package upgrade/data-retention proof were recorded in
DEVELOPMENT; they do not establish the public updater permission/installer flow.
Real GitHub CDN delivery, public-host persistence and the normal in-app update
sequence remain unverified. The actual public download is not ready until
publication and end-to-end verification complete.


Actual production UpdateControls has now run with seeded saved metadata and a
staged signed higher-version archive on the owned emulator: grant install
permission in Android Settings, return, recreate the Activity, re-verify the
retained file, open the real system installer, then cancel. Three focused UI
cases passed with the shared test-only saved-state fixture, including invalid and
missing restored-file recovery. That permission-return-only case cancels installation. A subsequent owned run
uses the same production UI flow and confirms the real system installer. Target
replacement terminates its first instrumentation invocation; separate post-upgrade
checks verify version9, the exact intended installed APK hash, retained isolated
encrypted pairing/settings and unchanged normal pairing identity. Cleanup passes.
The upgrade itself uses the system installer, not adb install. This starts with
seeded saved metadata and a staged archive: public metadata/CDN download and the
complete normal in-app update sequence remain unverified. Earlier adb and
permission-cancel receipts remain separate historical evidence.
