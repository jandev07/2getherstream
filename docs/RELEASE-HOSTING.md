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
for delivery and its bounded public metadata may be published.

## Current validation boundary

Source-level Worker and ordinary-file staging checks passed, as did Android JVM
redirect/cancellation checks and a test-only mapped loopback stalled-body test.
Those checks do not establish Cloudflare runtime, real GitHub CDN delivery,
PackageManager verification, installation or physical playback. The actual public
download is not ready until publication and end-to-end verification complete.
