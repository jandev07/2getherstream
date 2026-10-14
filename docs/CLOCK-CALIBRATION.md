# Fresh-window clock calibration

The Android follower translates its monotonic timestamps into the PC authority's
monotonic clock before scheduling and comparing native playback. Clock calibration
is separate from native position sampling, rendering, and the drift controller.

## Reason for the change

Saved short emulator traces exposed abrupt changes when the minimum-RTT exchange
changed. In fdc4bccb, a +164.5 ms selected-offset change coincided with an authority
drift change from -21 to -189 ms. This is evidence of a contribution to the measured
drift, not proof of the true clock offset or an explanation of all playback error.

Choosing one exchange assumes equal forward and return transit times for its point
estimate. A different exchange can have a smaller total delay and a different
directional split. The new Together estimator combines the offset constraints of
recent exchanges instead of choosing one exchange's point estimate for playback.

## Timestamp convention and assumptions

`t0` is client send, `t1` is server receive, `t2` is server send, and `t3` is client
receive. All use the respective endpoint's monotonic clock. The familiar offset
and round-trip-delay equations are documented in
[RFC 5905, section 8](https://datatracker.ietf.org/doc/html/rfc5905#section-8):

```
point offset = ((t1 - t0) + (t2 - t3)) / 2
network RTT = (t3 - t0) - (t2 - t1)
```

Assuming nonnegative transit delay and constant relative clock offset over the
fresh window, each exchange constrains the actual offset to:

```
lower = t2 - t3 = point offset - network RTT / 2
upper = t1 - t0 = point offset + network RTT / 2
```

This intersection policy is independently written Together code. It is not an
upstream Syncplay/Synkplay algorithm or a claim that RFC 5905 specifies this policy.
Existing third-party provenance is unchanged.

## Selection and fallback

Rust and Kotlin retain at most 64 accepted exchanges and include only exchanges
received no later than the current client time and no more than 30 seconds ago.
The existing accepted RTT range of 0 to 1,500 ms remains unchanged. Invalid or
non-finite exchanges cannot displace valid calibration history.

For fresh exchanges, use the greatest lower bound and least upper bound. If the
intersection is finite and nonempty, use its midpoint. Compute the midpoint as
`lower / 2 + upper / 2` rather than summing potentially large bounds first.
If the intersection is empty or non-finite, use the original minimum-RTT point
estimate and report absent bounds. Do not manufacture a narrow interval.

The effective sample retains the minimum-RTT exchange's actual RTT and receive
timestamp. Existing lead-time and sample-freshness behavior therefore continues
to use an observed exchange, rather than relabeling the intersection width as RTT.
No fresh exchange means no estimate. Epoch changes and reconnect clear history.

The raw minimum-RTT estimate remains available for diagnostics and comparisons.
Android playback conversions use the effective midpoint or explicit fallback.

## Limits

- Constant asymmetric delay can leave a biased midpoint. For example, 600 ms
  forward and 200 ms return delay gives an 800 ms-wide admissible interval and
  a midpoint 200 ms away from the true offset. It is not an asymmetry detector.
- Monotonic clocks can have different rates. A 30-second constant-offset window
  is an approximation; conflicting intervals fall back explicitly. Clock-rate
  estimation is not implemented by this change.
- New samples and expiration of constraining samples can move the midpoint.
  This policy does not promise a continuously smooth calibration.
- Native observation age, scheduling, decoder buffering, rendering, and audio
  output remain independent contributors to playback behavior.
- The interval width is conditional uncertainty, not measured clock accuracy,
  screen alignment, or playback acceptance.

## Runtime evidence

The proof log reports effective offset, raw minimum-RTT offset, optional bounds,
fresh support count, oldest support age, and an explicit consistency flag.
`analyze_clock_proof.py` verifies the reported midpoint/fallback wiring, finite
ordered bounds, support bounded to 64, and support age bounded to 30 seconds.
Historical rows without this metadata remain unknown rather than being counted
as fallback. Raw and effective offset transitions are summarized separately.

The report also retains up to 128 windows of streaming clock context. Windows
use 60-second buckets anchored at the first steady client timestamp and split
on generation changes. Each records observation count/span, effective offset
range, maximum authority drift/native age, interval width and fallback count.
Any omitted windows are counted explicitly; whole-run maxima and violation counts
still include all steady rows. No per-window alternative timing pass is computed.
These observed spans do not certify uninterrupted playback duration.

The provisional cross-device p95 target remains 150 ms. Native drift assertions
remain unchanged. Passing emulator assertions, a numeric interval check, or a
longer synthetic run cannot satisfy the physical-phone two-hour release gate.
Actual completed receipts and artifact boundaries belong in DEVELOPMENT.md.
