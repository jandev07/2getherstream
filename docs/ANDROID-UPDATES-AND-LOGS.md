# Private Alpha updates and phone diagnostics

Android keeps structured playback diagnostics while the app is visible, sampling
every two seconds and writing off the main/player threads. Backgrounding pauses
playback/disconnects control, records that transition, then cancels periodic log
sampling. Returning resumes it. Idle background samples therefore do not overwrite
the recent useful playback history or keep a repeating log timer alive. Two private files
rotate at 1 MiB each. No automatic network upload runs during playback.
**Send logs to PC** sends the latest 128 samples (about four minutes) to the
currently paired Windows app. On partial failure the phone keeps its local logs;
retrying may repeat previously accepted records. PC acknowledgement means its
userspace journal flush completed, not a power-loss durability guarantee.

## Location to inspect in later development sessions

The normal Windows log folder on this user's PC is:
`C:\Users\astro\AppData\Local\app.together.v2.windows\diagnostics`

Received Android records have `event: "phone"` in `session-0.jsonl` and the rotated
`session-1.jsonl` through `session-7.jsonl`. PC and phone samples share the journal,
so investigate them together. The Windows **Export session log** button creates
a combined export in this folder's `exports` subfolder. Each file is bounded at
8 MiB, with eight retained. Proof runs use
`C:\Users\astro\Documents\Together2\work\native-journal-proof` instead; never
confuse proof logs with the user's normal session logs.

`scripts/Inspect-PhoneLogs.py` defaults to the normal PC folder above and returns
only bounded numeric/boolean summaries. Pass an archived proof journal directory
explicitly when checking a regression. UTC times come from the phone's wall clock;
neither that clock nor sample counts establish clock alignment, unique delivery
or a playback acceptance gate. Manual upload retries can duplicate samples.

The inspector reports raw uploads separately from unique `phone_samples`, using
phone elapsed time, UTC time and generation as the sample identity. Identical
copies of the first observed sample count as duplicates; copies with different
validated values count as conflicts. Repeated conflicting copies remain conflicts
against that first observation. Fault/buffering/disconnect counts retain the worst
flags per identity, and maximum drift includes conflicting copies. This prevents
retries inflating sample counts or conflicting copies hiding a problem. These
counts do not establish a continuous run or physical acceptance. Oversized log
lines are discarded whole so their suffixes cannot become extra records.

PC source loading now records fixed phases `current_file_and_links`,
`direct_ranges` and `publication`, with started/passed/failed/replaced outcomes.
Selection and generation numbers link phases; elapsed_ms is cumulative from the
beginning of that link resolution, not a per-phase duration. The inspector counts
source failures and replaced selections alongside received phone samples. These
events contain no media names, URLs, credentials or arbitrary provider errors.

Phone-local files are in its private `noBackupFilesDir/diagnostics`, named
`phone-current.jsonl` and `phone-previous.jsonl`. Logs include numeric playback
positions, generation, clock RTT, drift and fixed playback/connection flags.
No raw exception messages, media URLs, filenames, TorBox keys or pairing tokens
enter the diagnostic schema. The authenticated phone-only upload accepts at most
20 typed samples per request, with a 16 KiB request bound. The UI sends ten at a
time. Upload failures do not change playback authority or stop the player.

The PC admits each validated upload as one bounded diagnostic queue item. A full
queue rejects the whole item without queuing a subset of its samples. The writer
acknowledges only after writing all rows and flushing userspace buffers; unrelated
playback diagnostic drops do not invalidate that batch receipt. Writer failure or
a two-second acknowledgement timeout returns unavailable. This is not a disk
transaction or fsync guarantee: filesystem failure or a lost receipt can leave
partial or repeated rows, which the inspector handles conservatively.

## Android updates

The phone checks authenticated release metadata from its paired PC. The PC reads
`updates/android-release.json` beneath its local application-data folder. Missing
or invalid configuration returns unavailable; it must not falsely mean up to date.
The metadata shape is:

```json
{
  "protocol": 1,
  "package": "app.together.v2",
  "version_code": 3,
  "version_name": "Alpha 1",
  "apk_url": "https://releases.example.org/2gether.apk",
  "sha256": "REPLACE_WITH_THE_ACTUAL_64_HEX_DIGIT_APK_HASH",
  "size_bytes": 224000000
}
```

This is a schema illustration, not a configured or published release. Publish
only after a real higher-version APK, stable signing identity and hosting have
been prepared and authorized. Both apps reject non-HTTPS APK addresses, userinfo,
query strings and fragments. APK downloads follow no redirects, carry no paired
control bearer, and never pass APK bytes through the control tunnel. The APK must
match the declared size/hash, package, higher version and installed signer set.
Signing-certificate rotation is deliberately unsupported for this first flow.
Android 26–27 use the older signature API; newer devices use SigningInfo.

Check -> Download -> Install is explicit. Android may ask the user to allow
installations from 2gether, then confirm the install. Pairing/settings remain in
private app storage for normal future 2gether updates. Together 1.0 compatibility
is not required, and no legacy package or signing key is touched.

The update flow is not accepted end to end until a real signed higher-version
APK has been downloaded/installed on a phone and persistence checked. Rejection
tests and compilation alone do not prove successful updates. Playback's physical
phone/two-hour gate and actual safe TorBox delivery remain separate requirements.
