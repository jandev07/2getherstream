# Stremio movie selection — private Alpha boundary

The PC reads its existing DPAPI Stremio credentials. Email/password intake signs
in only at the official fixed HTTPS login endpoint; the resulting auth key stays
in PC memory. Library and configured addon requests are read-only. Account keys
and configured addon transports never enter the public browser models or packages.

Windows source now offers catalog browsing/search, movie selection and Watch for
an exact ready TorBox file. Stream resource types and ID prefixes are respected.
Series titles open an episode list from a configured metadata addon. The PC uses
explicit episode IDs, season and episode numbers, checks the returned series ID
and type, and discards URLs/private fields. Back restores the original series
list and invalidates old stream choices. No array index is treated as an episode ID.
The user selects on PC; Android does not need a Stremio browser.

An alternative PC-only masked input accepts a copied Torrentio/TorBox source link
after the user starts a movie or episode in Stremio. It clears immediately upon
submission and is not persisted or logged. The parser extracts only the exact
torrent hash and filename, discards the supplied account key and file index, and
does not follow the wrapper. The PC performs a fresh TorBox inventory read so an
item just prepared by Stremio can be recognized. It then issues its own separate
device leases. This is not support for arbitrary CDN or localhost playback links.

Canonical matching requires a unique available TorBox file with hash/filename or
explicit provider torrent/file IDs. Duplicate basenames and hash-only descriptors
fail closed. TorBox sequences are reserved before copied-source inventory work;
later user selections supersede pending results. Both ranges are validated before
publication against the original host epoch/generation. A separate Prepare action
can add an explicitly selected provider-cached torrent. It never deletes items
or changes seeding preferences.

The parser is independently implemented from the maintainer's public
[TorBox route](https://github.com/TheBeastLT/torrentio-scraper/blob/master/addon/moch/torbox.js)
and [wrapper construction](https://github.com/TheBeastLT/torrentio-scraper/blob/master/addon/moch/moch.js).
PC account requests follow Stremio's
[official request definitions](https://github.com/Stremio/stremio-core/blob/master/src/types/api/request.rs).

## Live evidence and remaining work

2026-10-12 read-only encrypted-intake proof returned four configured HTTPS addons,
one stream provider, 130 catalog definitions, 56 movie titles and 51 stream choices
for the first catalog title. The saved library was empty. Zero choices for that
title matched a ready file currently in this TorBox account. Count-only receipt:
work/stremio-readonly-selection-proof-20261012.txt.

The copied-source flow has parser/compile validation, not a live user-picked
Stremio movie/episode handoff. Explicit provider-cached preparation is now implemented:
list/checkcached verifies a unique exact filename and size; Prepare rechecks the
cache and reuses an existing account item when present. A single form-encoded
create request may add the selected 40-character torrent hash. Returned account
hash, file ID, filename, size and clean scan status are checked before Watch is
enabled. Queued/not-ready items and unknown scan status remain pending without
changing current playback; infected files are rejected.
An uncertain creation retains an in-process attempt marker so it cannot be
automatically repeated; restarting the PC app clears this local marker, so check
the TorBox account before trying again after an uncertain outcome. No account
items are deleted or stopped to make room. The
[official torrent API](https://github.com/TorBox-App/torbox-sdk-js/blob/main/documentation/services/TorrentsService.md)
documents fresh inventory reads and cache/create requests.

Fresh read-only cache validation found 21 exact provider-cached video choices among
the 51 stream descriptors, with no account writes. Receipt:
work/stremio-cached-selection-proof-20261012.txt. Three localhost HTTP/parser
regressions validate reuse, exact clean-file matching, form request construction,
pending states and uncertain-create retry suppression. Live account creation and
paired native playback remain unverified.

The [official Stremio server documentation](https://github.com/Stremio/server-docker/blob/main/README.md)
lists HTTP11470/HTTPS12470. Neither port was listening in this pass's read-only PC
check. No Stremio service was started or modified. PC localhost is not the remote
phone's localhost. A local media wrapper does not establish a file-scoped TorBox
credential; relaying movie bytes would change the direct-CDN design and must not
use the control-only Cloudflare tunnel.

Fresh read-only series proof returned 59 series titles, 99 explicit episode IDs
for the inspected series, and 10 descriptors for its first episode. That episode
had zero exact cached choices; metadata/stream support does not guarantee every
episode is cached. No account writes or handoffs occurred. Receipt:
work/stremio-series-proof-20261012.txt.

46 resolver checks, two Windows selection regressions and Svelte checks pass.
The isolated private candidate builder has run successfully and checked that
offered EXE/APK hashes are unchanged. A separate muted/headless PC-only TorBox
native proof passed real decode progression, pause (0ms drift), seek to45s and
resumed progression. Both separate-role1024-byte ranges were fetched on PC;
Android did not receive either link. Receipt:
work/pc-provider-native/b76d8778-eb3b-4022-b974-136083e32901/receipt.json.
This is not a live Stremio create/watch flow, GUI pairing, physical-phone, WAN or
two-hour acceptance. No Android installation or emulator/adb changes occurred.
