"""Summarize numeric clock diagnostics from a completed emulator proof.

Every row must match the receipt run ID. Historical unbound logs require an
explicit compatibility option and never claim verified association.
"""
import argparse
import json
import math
import uuid
from pathlib import Path


class ClockWindows:
    """Bounded streaming context, never an alternate playback acceptance scope."""
    def __init__(self):
        self.origin = None
        self.current = None
        self.rows = []
        self.omitted = 0

    def add(self, values, interval):
        stamp = values["client_ms"]
        if self.origin is None:
            self.origin = stamp
        elapsed = stamp - self.origin
        if not math.isfinite(elapsed) or elapsed < 0:
            raise ValueError("finite_window_time_required")
        index = int(elapsed // 60000)
        key = index, values["generation"]
        if self.current is None or self.current[0] != key:
            self.finish()
            self.current = key, dict(window_index=index, generation=values["generation"],
                samples=0, first_client_ms=stamp, last_client_ms=stamp,
                minimum_used_offset_ms=values["offset_ms"], maximum_used_offset_ms=values["offset_ms"],
                maximum_authority_drift_ms=0, maximum_native_age_ms=0,
                interval_observed_samples=0, interval_fallback_samples=0,
                maximum_interval_width_ms=None)
        row = self.current[1]
        row["samples"] += 1
        row["last_client_ms"] = stamp
        row["minimum_used_offset_ms"] = min(row["minimum_used_offset_ms"], values["offset_ms"])
        row["maximum_used_offset_ms"] = max(row["maximum_used_offset_ms"], values["offset_ms"])
        row["maximum_authority_drift_ms"] = max(row["maximum_authority_drift_ms"], abs(values["drift_ms"]))
        row["maximum_native_age_ms"] = max(row["maximum_native_age_ms"], values["sample_age_ms"])
        if interval is not None:
            row["interval_observed_samples"] += 1
            row["interval_fallback_samples"] += not interval["consistent"]
            width = interval["width_ms"]
            if width is not None:
                row["maximum_interval_width_ms"] = width if row["maximum_interval_width_ms"] is None else max(width, row["maximum_interval_width_ms"])

    def finish(self):
        if self.current is None:
            return
        row = self.current[1]
        row["observed_span_ms"] = row["last_client_ms"] - row["first_client_ms"]
        row["used_offset_span_ms"] = row["maximum_used_offset_ms"] - row["minimum_used_offset_ms"]
        if not all(math.isfinite(row[key]) for key in ("observed_span_ms", "used_offset_span_ms")):
            raise ValueError("finite_window_span_required")
        # A two-hour proof produces about 120 minute windows. Bound storage even
        # for an unexpectedly long log or repeated generation changes.
        if len(self.rows) < 128:
            self.rows.append(row)
        else:
            self.omitted += 1
        self.current = None


def interval_observation(row, used_offset):
    keys = ("clock_interval_consistent", "clock_lower_offset_ms", "clock_upper_offset_ms",
            "clock_support_samples", "clock_support_age_ms", "raw_selected_offset_ms")
    if not any(key in row for key in keys):
        return None  # Historical logs do not establish interval behavior.
    consistent = row.get("clock_interval_consistent")
    if not isinstance(consistent, bool):
        raise ValueError("interval_consistency_flag_required")
    count = row.get("clock_support_samples")
    if isinstance(count, bool) or not isinstance(count, int) or not 1 <= count <= 64:
        raise ValueError("bounded_interval_support_required")
    numbers = {}
    for key in ("clock_support_age_ms", "raw_selected_offset_ms"):
        value = row.get(key)
        if isinstance(value, bool) or not isinstance(value, (int, float)) or not math.isfinite(value):
            raise ValueError("numeric_interval_diagnostics_required")
        numbers[key] = value
    if not 0 <= numbers["clock_support_age_ms"] <= 30000:
        raise ValueError("fresh_interval_support_required")
    lower, upper = row.get("clock_lower_offset_ms"), row.get("clock_upper_offset_ms")
    width = None
    if consistent:
        if any(isinstance(v, bool) or not isinstance(v, (int, float)) or not math.isfinite(v) for v in (lower, upper)) or lower > upper:
            raise ValueError("ordered_finite_interval_required")
        width = upper - lower
        if not math.isfinite(width) or not math.isclose(used_offset, lower / 2 + upper / 2, rel_tol=0, abs_tol=0.000001):
            raise ValueError("interval_midpoint_required")
    elif lower is not None or upper is not None or used_offset != numbers["raw_selected_offset_ms"]:
        raise ValueError("explicit_lowest_rtt_fallback_required")
    return dict(consistent=consistent, width_ms=width, support_samples=count,
                support_age_ms=numbers["clock_support_age_ms"], raw_offset_ms=numbers["raw_selected_offset_ms"])


def analyze(receipt, lines, allow_legacy_unbound=False):
    if not isinstance(receipt, dict):
        raise ValueError("receipt_object_required")
    if receipt.get("status") != "passed" or receipt.get("phase") != "complete":
        raise ValueError("completed_pass_receipt_required")
    if receipt.get("validation_scope") != "emulator_synthetic_fixture":
        raise ValueError("emulator_scope_required")
    completed = receipt.get("completed_uninterrupted_seconds")
    requested = receipt.get("requested_uninterrupted_seconds")
    if isinstance(requested, bool) or not isinstance(requested, int) or isinstance(completed, bool) or not isinstance(completed, int) or completed <= 0 or completed != requested or not receipt.get("completed_utc"):
        raise ValueError("completed_duration_required")
    if not isinstance(receipt.get("run_id"), str):
        raise ValueError("run_identity_required")
    run_id = str(uuid.UUID(receipt["run_id"]))
    identity_verified = True
    count = 0
    generations = set()
    maxima = dict(selected_age_ms=0, history_size=0, selected_offset_change_ms=0,
                  authority_drift_ms=0, native_age_ms=0)
    previous = None
    previous_values = None
    previous_available = False
    intervening_nonsteady_rows = 0
    offset_transition_count = 0
    largest_offset_step_context = None
    interval_summary = dict(observed_samples=0, consistent_samples=0, fallback_samples=0,
                            maximum_width_ms=None, maximum_support_samples=None,
                            maximum_support_age_ms=None, raw_offset_transition_count=0,
                            maximum_raw_offset_step_ms=None)
    previous_interval = None
    windows = ClockWindows()
    first_client_ms = None
    max_steady_gap_ms = 0
    violations = dict(clock_age=0, history_size=0, disconnected=0, buffering=0,
                      seeking=0, preparing=0, not_playing=0)
    for line in lines:
        row = json.loads(line)
        if not isinstance(row, dict):
            raise ValueError("observation_object_required")
        identity = row.get("run_id")
        if identity is None and allow_legacy_unbound and receipt.get("proof_log_schema", 1) < 2:
            identity_verified = False
        elif not isinstance(identity, str) or identity != run_id:
            raise ValueError("observation_run_identity_mismatch")
        if row.get("phase") != "steady":
            if previous is not None:
                intervening_nonsteady_rows += 1
            continue
        count += 1
        values = {}
        for key in ("generation", "client_ms", "offset_ms", "clock_sample_age_ms",
                    "clock_history_size", "drift_ms", "sample_age_ms"):
            value = row.get(key)
            if isinstance(value, bool) or not isinstance(value, (float, int)) or not math.isfinite(value):
                raise ValueError("numeric_clock_diagnostics_required")
            values[key] = value
        generation = values["generation"]
        interval = interval_observation(row, values["offset_ms"])
        windows.add(values, interval)
        if interval is not None:
            interval_summary["observed_samples"] += 1
            interval_summary["consistent_samples" if interval["consistent"] else "fallback_samples"] += 1
            for summary_key, observation_key in (("maximum_width_ms", "width_ms"),
                                                  ("maximum_support_samples", "support_samples"),
                                                  ("maximum_support_age_ms", "support_age_ms")):
                value = interval[observation_key]
                if value is not None:
                    old = interval_summary[summary_key]
                    interval_summary[summary_key] = value if old is None else max(old, value)
            raw_step = 0
            if previous_interval is not None and previous_interval[0] == generation:
                raw_step = abs(interval["raw_offset_ms"] - previous_interval[1])
                if not math.isfinite(raw_step):
                    raise ValueError("finite_raw_offset_step_required")
                interval_summary["raw_offset_transition_count"] += raw_step != 0
            old = interval_summary["maximum_raw_offset_step_ms"]
            interval_summary["maximum_raw_offset_step_ms"] = raw_step if old is None else max(old, raw_step)
            previous_interval = generation, interval["raw_offset_ms"]
        else:
            previous_interval = None
        generations.add(generation)
        maxima["selected_age_ms"] = max(maxima["selected_age_ms"], values["clock_sample_age_ms"])
        maxima["history_size"] = max(maxima["history_size"], values["clock_history_size"])
        maxima["authority_drift_ms"] = max(maxima["authority_drift_ms"], abs(values["drift_ms"]))
        maxima["native_age_ms"] = max(maxima["native_age_ms"], values["sample_age_ms"])
        violations["clock_age"] += not 0 <= values["clock_sample_age_ms"] <= 30000
        violations["history_size"] += not 1 <= values["clock_history_size"] <= 64
        violations["disconnected"] += row.get("connected") is not True
        for key in ("buffering", "seeking", "preparing"):
            violations[key] += row.get(key) is not False
        violations["not_playing"] += row.get("state") != "Playing"
        available = (row.get("connected") is True and row.get("state") == "Playing"
                     and all(row.get(key) is False for key in ("buffering", "seeking", "preparing")))
        if first_client_ms is None:
            first_client_ms = values["client_ms"]
        if previous:
            if values["client_ms"] < previous[1]:
                raise ValueError("steady_time_reordered")
            max_steady_gap_ms = max(max_steady_gap_ms, values["client_ms"] - previous[1])
            if previous[0] == generation:
                step = values["offset_ms"] - previous[2]
                drift_change = values["drift_ms"] - previous_values["drift_ms"]
                drift_change_plus_step = drift_change + step
                if not all(math.isfinite(value) for value in (step, drift_change, drift_change_plus_step)):
                    raise ValueError("finite_transition_diagnostics_required")
                if step != 0:
                    offset_transition_count += 1
                if abs(step) > maxima["selected_offset_change_ms"]:
                    largest_offset_step_context = dict(
                        generation=generation, client_before_ms=previous[1],
                        client_after_ms=values["client_ms"],
                        client_gap_ms=values["client_ms"] - previous[1],
                        intervening_nonsteady_rows=intervening_nonsteady_rows,
                        both_observations_playing_and_available=previous_available and available,
                        signed_offset_step_ms=step,
                        authority_drift_before_ms=previous_values["drift_ms"],
                        authority_drift_after_ms=values["drift_ms"],
                        authority_drift_change_ms=drift_change,
                        drift_change_plus_offset_step_ms=drift_change_plus_step,
                        selected_age_before_ms=previous_values["clock_sample_age_ms"],
                        selected_age_after_ms=values["clock_sample_age_ms"],
                        native_age_before_ms=previous_values["sample_age_ms"],
                        native_age_after_ms=values["sample_age_ms"])
                maxima["selected_offset_change_ms"] = max(maxima["selected_offset_change_ms"], abs(step))
        previous = generation, values["client_ms"], values["offset_ms"]
        previous_values = values
        previous_available = available
        intervening_nonsteady_rows = 0
    if count == 0:
        raise ValueError("steady_samples_required")
    windows.finish()
    return dict(run_id=run_id, validation_scope="emulator_synthetic_fixture",
                log_identity_verified=identity_verified, steady_samples=count, steady_generation_count=len(generations),
                steady_observed_span_ms=previous[1] - first_client_ms, max_steady_gap_ms=max_steady_gap_ms,
                maxima=maxima, violation_counts=violations,
                offset_transition_count=offset_transition_count,
                largest_offset_step_context=largest_offset_step_context,
                interval_diagnostics=interval_summary,
                steady_clock_windows=windows.rows, omitted_clock_windows=windows.omitted,
                window_method="60-second client-time buckets anchored at first steady observation; generation changes split windows. At most 128 windows retained, omissions explicit. Streaming extrema and observed spans are diagnostic context, not continuous playback duration, cross-device p95, clock accuracy or acceptance. Global metrics still include every steady row.",
                interval_method="Optional fresh-window metadata; midpoint/fallback wiring checked against reported used offset. Interval width assumes constant relative clock offset and nonnegative transit delays; it is not measured clock error or playback acceptance. Historical missing metadata remains unknown.",
                transition_method="Adjacent included steady observations in the same generation; excluded-row count and client gap remain explicit. Drift change plus signed offset step is descriptive arithmetic, not a corrected acceptance metric or proof of clock error, causation or screen/audio alignment.",
                physical_release_gate_satisfied=False,
                method="Consecutive selected-offset changes within a generation; numeric steady samples only. Observed span/gaps describe available steady rows, not verified uninterrupted playback. Run IDs establish association, not cryptographic authenticity or screen alignment.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--receipt", required=True, type=Path)
    parser.add_argument("--phone-log", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    parser.add_argument("--allow-legacy-unbound", action="store_true", help="Explicitly permit old rows without run IDs; association remains unverified")
    args = parser.parse_args()
    try:
        receipt = json.loads(args.receipt.read_text(encoding="utf-8-sig"))
        with args.phone_log.open(encoding="utf-8-sig") as lines:
            result = analyze(receipt, lines, args.allow_legacy_unbound)
        # New output only; never overwrite source evidence or a previous report.
        with args.output.open("x", encoding="utf-8") as output:
            json.dump(result, output, indent=2)
            output.write("\n")
    except (ValueError, OSError, KeyError, TypeError, OverflowError):
        # Do not echo paths, input rows or parsing exception text.
        parser.exit(1, "clock_analysis_failed\n")
    print("Completed emulator clock analysis written; physical release gate remains open")


if __name__ == "__main__":
    main()
