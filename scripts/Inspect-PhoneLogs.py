"""Read-only phone and source-loading summary; never prints arbitrary log text."""
import argparse
import json
import os
import hashlib
import math
import uuid
from pathlib import Path

PHONE_FIELDS = ("elapsed_ms", "utc_ms", "generation", "position_ms", "duration_ms",
                "connected", "buffering", "seeking", "preparing", "source_ready",
                "fault", "playback", "drift_ms", "rtt_ms")
SESSION_FAULTS = ("prepare_timeout", "execution_timeout_pc", "execution_timeout_phone",
                  "player_failed_pc", "player_failed_phone", "unknown_fault")


def unsigned(value):
    if type(value) is not int or not 0 <= value <= 18446744073709551615:
        raise ValueError()
    return value


def validate_session(sample):
    for key in ("server_ms", "generation", "revision"):
        unsigned(sample[key])
    for key in ("preparing", "playing"):
        if type(sample[key]) is not bool:
            raise ValueError()
    if sample["fault"] is not None and sample["fault"] not in SESSION_FAULTS:
        raise ValueError()
    for key in ("pc", "phone"):
        participant = sample[key]
        if participant is not None:
            for field in ("connected", "buffering", "ready"):
                if type(participant[field]) is not bool:
                    raise ValueError()


def validate_native(sample):
    for key in ("server_ms", "generation", "position_ms", "target_ms"):
        unsigned(sample[key])
    for key in ("paused", "seeking", "buffering", "source_ready"):
        if type(sample[key]) is not bool:
            raise ValueError()
    if type(sample["drift_ms"]) is not int or not -(2**63) <= sample["drift_ms"] < 2**63:
        raise ValueError()
    if type(sample["rate"]) not in (int, float) or not 0 < sample["rate"] <= 2 or not math.isfinite(sample["rate"]):
        raise ValueError()


def validate_phone(sample):
    if set(sample) != set(PHONE_FIELDS):
        raise ValueError()
    for key in ("elapsed_ms", "utc_ms", "generation"):
        if type(sample[key]) is not int or not 0 <= sample[key] <= 9223372036854775807:
            raise ValueError()
    for key in ("position_ms", "duration_ms"):
        if type(sample[key]) is not int or not 0 <= sample[key] <= 604800000:
            raise ValueError()
    for key in ("fault", "buffering", "connected", "seeking", "preparing", "source_ready"):
        if type(sample[key]) is not bool:
            raise ValueError()
    if type(sample["drift_ms"]) is not int or abs(sample["drift_ms"]) > 604800000:
        raise ValueError()
    if sample["rtt_ms"] is not None and (type(sample["rtt_ms"]) is not int or not 0 <= sample["rtt_ms"] <= 60000):
        raise ValueError()
    if sample["playback"] not in ("starting", "loading", "playing", "paused", "ended", "failed"):
        raise ValueError()


def inspect(directory):
    directory = Path(directory)
    result = {"folder": str(directory), "available": directory.is_dir(), "files": 0,
              "phone_samples": 0, "fault_samples": 0, "buffering_samples": 0,
              "raw_phone_samples": 0, "duplicate_phone_samples": 0,
              "conflicting_phone_samples": 0, "conflicting_sample_identities": 0,
              "disconnected_samples": 0, "maximum_absolute_drift_ms": 0,
              "first_phone_utc_ms": None, "last_phone_utc_ms": None,
              "source_events": 0, "source_replaced": 0, "maximum_source_elapsed_ms": 0,
              "source_failures": {stage: 0 for stage in ("current_file_and_links", "direct_ranges", "publication")},
              "session_samples": 0, "preparing_session_samples": 0, "authoritative_playing_samples": 0,
              "pc_waiting_ready_samples": 0, "phone_waiting_ready_samples": 0,
              "session_fault_samples": {fault: 0 for fault in SESSION_FAULTS},
              "bound_fault_generations": {fault: 0 for fault in SESSION_FAULTS},
              "unbound_session_samples": 0, "native_pc_samples": 0,
              "native_pc_buffering_samples": 0, "native_pc_unready_samples": 0,
              "maximum_native_pc_absolute_drift_ms": 0,
              "maximum_journal_dropped_records": 0, "maximum_journal_io_errors": 0,
              "invalid_records": 0, "physical_release_gate_satisfied": False}
    if not result["available"]:
        return result
    seen = {}
    fault_generations = set()
    # Fixed filenames only. Do not traverse exports, credentials, junctions or links.
    for index in range(8):
        path = directory / f"session-{index}.jsonl"
        if not path.exists():
            continue
        if any(parent.is_symlink() or parent.is_junction() for parent in [path, *path.parents]) or path.stat().st_size > 8 * 1024 * 1024 + 16384:
            raise ValueError("Unsupported diagnostic path or size")
        result["files"] += 1
        with path.open(encoding="utf-8") as stream:
            while line := stream.readline(16385):
                if len(line) > 16384:
                    # Discard the complete physical line, not only its first
                    # bounded chunk. A valid-looking suffix is not another record.
                    while not line.endswith("\n"):
                        line = stream.readline(16385)
                        if not line:
                            break
                    result["invalid_records"] += 1
                    continue
                try:
                    record = json.loads(line)
                    context = None
                    if "record" in record:
                        health = {key: unsigned(record[key]) for key in ("dropped_records", "io_errors") if key in record}
                        if "run_id" in record:
                            if type(record["run_id"]) is not str:
                                raise ValueError()
                            context = str(uuid.UUID(record["run_id"]))
                        for key, value in health.items():
                            result[f"maximum_journal_{key}"] = max(result[f"maximum_journal_{key}"], value)
                    # The rotating PC journal wraps typed events with write time
                    # and diagnostic health; bare events are supported for exports.
                    record = record.get("record", record)
                    if record.get("event") == "session":
                        sample = record["sample"]
                        validate_session(sample)
                        result["session_samples"] += 1
                        result["unbound_session_samples"] += int(context is None)
                        result["preparing_session_samples"] += int(sample["preparing"])
                        result["authoritative_playing_samples"] += int(sample["playing"])
                        for actor in ("pc", "phone"):
                            participant = sample[actor]
                            result[f"{actor}_waiting_ready_samples"] += int(sample["preparing"] and
                                (participant is None or not participant["ready"]))
                        fault = sample["fault"]
                        if fault is not None:
                            result["session_fault_samples"][fault] += 1
                            identity = (context, sample["generation"], fault)
                            if context is not None and identity not in fault_generations:
                                fault_generations.add(identity)
                                result["bound_fault_generations"][fault] += 1
                        continue
                    if record.get("event") == "native_pc":
                        sample = record["sample"]
                        validate_native(sample)
                        result["native_pc_samples"] += 1
                        result["native_pc_buffering_samples"] += int(sample["buffering"])
                        result["native_pc_unready_samples"] += int(not sample["source_ready"])
                        result["maximum_native_pc_absolute_drift_ms"] = max(result["maximum_native_pc_absolute_drift_ms"], abs(sample["drift_ms"]))
                        continue
                    if record.get("event") == "source":
                        sample = record["sample"]
                        stage, outcome = sample["stage"], sample["outcome"]
                        if stage not in result["source_failures"] or outcome not in ("started", "passed", "failed", "replaced"):
                            raise ValueError()
                        for key in ("server_ms", "selection", "generation", "elapsed_ms"):
                            unsigned(sample[key])
                        if type(sample["refresh"]) is not bool:
                            raise ValueError()
                        result["source_events"] += 1
                        result["source_failures"][stage] += int(outcome == "failed")
                        result["source_replaced"] += int(outcome == "replaced")
                        result["maximum_source_elapsed_ms"] = max(result["maximum_source_elapsed_ms"], sample["elapsed_ms"])
                        continue
                    if record.get("event") != "phone":
                        continue
                    sample = record["sample"]
                    validate_phone(sample)
                    identity = (sample["elapsed_ms"], sample["utc_ms"], sample["generation"])
                    digest = hashlib.sha256(json.dumps(sample, sort_keys=True, separators=(",", ":")).encode()).digest()
                    flags = int(sample["fault"]) | (int(sample["buffering"]) << 1) | (int(not sample["connected"]) << 2)
                    result["raw_phone_samples"] += 1
                    previous = seen.get(identity)
                    if previous is None:
                        result["phone_samples"] += 1
                        previous = (digest, 0, False)
                    elif previous[0] == digest:
                        result["duplicate_phone_samples"] += 1
                    else:
                        result["conflicting_phone_samples"] += 1
                        if not previous[2]:
                            result["conflicting_sample_identities"] += 1
                    # Retain the worst flags across conflicting copies; retries
                    # cannot inflate counts or hide a fault in an earlier copy.
                    new_flags = flags & ~previous[1]
                    result["fault_samples"] += int(bool(new_flags & 1))
                    result["buffering_samples"] += int(bool(new_flags & 2))
                    result["disconnected_samples"] += int(bool(new_flags & 4))
                    seen[identity] = (previous[0], previous[1] | flags, previous[2] or previous[0] != digest)
                    result["maximum_absolute_drift_ms"] = max(result["maximum_absolute_drift_ms"], abs(sample["drift_ms"]))
                    utc = sample["utc_ms"]
                    result["first_phone_utc_ms"] = min(result["first_phone_utc_ms"] if result["first_phone_utc_ms"] is not None else utc, utc)
                    result["last_phone_utc_ms"] = max(result["last_phone_utc_ms"] if result["last_phone_utc_ms"] is not None else utc, utc)
                except (ValueError, TypeError, KeyError, AttributeError):
                    result["invalid_records"] += 1
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("directory", nargs="?", default=str(Path(os.environ.get("LOCALAPPDATA", "")) / "app.together.v2.windows" / "diagnostics"))
    print(json.dumps(inspect(parser.parse_args().directory), indent=2))
