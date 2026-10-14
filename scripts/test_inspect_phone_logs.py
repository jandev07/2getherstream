"""Numeric source summaries must not export provider-controlled text."""
import json
import tempfile
import unittest
import importlib.util
from pathlib import Path
spec = importlib.util.spec_from_file_location("phone_log_inspector", Path(__file__).with_name("Inspect-PhoneLogs.py"))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)
inspect = module.inspect


def phone_sample(**changes):
    sample = {"elapsed_ms": 10, "utc_ms": 42, "generation": 11, "position_ms": 400,
              "duration_ms": 1000, "connected": True, "buffering": False,
              "seeking": False, "preparing": False, "source_ready": True,
              "fault": False, "playback": "paused", "drift_ms": -125, "rtt_ms": 30}
    sample.update(changes)
    return sample


class SourceSummaryTest(unittest.TestCase):
    def test_authority_faults_deduplicate_generations_per_run_and_keep_health_and_native_counts(self):
        def session(fault=None, preparing=True):
            return {"event": "session", "sample": {"server_ms": 5, "generation": 3, "revision": 9,
                "fault": fault, "preparing": preparing, "playing": False,
                "pc": {"connected": True, "buffering": False, "ready": True},
                "phone": {"connected": True, "buffering": False, "ready": False}}}
        native = {"event": "native_pc", "sample": {"server_ms": 5, "generation": 3,
            "position_ms": 40, "target_ms": 50, "paused": True, "seeking": False,
            "buffering": True, "source_ready": False, "rate": 1.0, "drift_ms": -10}}
        first_run = "00000000-0000-0000-0000-000000000001"
        second_run = "00000000-0000-0000-0000-000000000002"
        def wrap(row, run=first_run):
            return {"record": row, "run_id": run, "dropped_records": 4, "io_errors": 2}
        rows = [wrap(session()), wrap(session("execution_timeout_phone", False)),
                wrap(session("execution_timeout_phone", False)), native,
                wrap(session("execution_timeout_phone", False), second_run),
                session("execution_timeout_phone", False)]
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "session-0.jsonl").write_text("\n".join(map(json.dumps, rows)), encoding="utf-8")
            result = inspect(directory)
        self.assertEqual(result["session_samples"], 5)
        self.assertEqual(result["preparing_session_samples"], 1)
        self.assertEqual(result["pc_waiting_ready_samples"], 0)
        self.assertEqual(result["phone_waiting_ready_samples"], 1)
        self.assertEqual(result["session_fault_samples"]["execution_timeout_phone"], 4)
        self.assertEqual(result["bound_fault_generations"]["execution_timeout_phone"], 2)
        self.assertEqual(result["unbound_session_samples"], 1)
        self.assertEqual(result["maximum_journal_dropped_records"], 4)
        self.assertEqual(result["maximum_journal_io_errors"], 2)
        self.assertEqual(result["native_pc_samples"], 1)
        self.assertEqual(result["native_pc_buffering_samples"], 1)
        self.assertEqual(result["native_pc_unready_samples"], 1)
        self.assertEqual(result["maximum_native_pc_absolute_drift_ms"], 10)
        self.assertEqual(result["invalid_records"], 0)
        self.assertFalse(result["physical_release_gate_satisfied"])

    def test_malformed_authority_native_and_health_fields_never_export_private_text(self):
        session = {"event": "session", "sample": {"server_ms": 5, "generation": 3, "revision": 9,
            "fault": "PRIVATE_TOKEN https://private.invalid", "preparing": False, "playing": False,
            "pc": None, "phone": None}}
        native = {"event": "native_pc", "sample": {"server_ms": 5, "generation": 3,
            "position_ms": 40, "target_ms": 50, "paused": True, "seeking": False,
            "buffering": False, "source_ready": True, "rate": 1.0, "drift_ms": -10}}
        rows = [session, {"record": native, "run_id": "PRIVATE_TOKEN"},
                {"record": native, "dropped_records": True}]
        for rate in (True, float("nan"), float("inf"), 10**1000, 0, "PRIVATE_TOKEN"):
            rows.append({"event": "native_pc", "sample": {**native["sample"], "rate": rate}})
        rows.append({"event": "native_pc", "sample": {**native["sample"], "source_ready": "true"}})
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "session-0.jsonl").write_text("\n".join(map(json.dumps, rows)), encoding="utf-8")
            result = inspect(directory)
        self.assertEqual(result["invalid_records"], len(rows))
        self.assertEqual(result["session_samples"], 0)
        self.assertEqual(result["native_pc_samples"], 0)
        self.assertNotIn("PRIVATE_TOKEN", json.dumps(result))
        self.assertNotIn("https://", json.dumps(result))

    def test_mixed_source_and_phone_rows_and_malformed_fields(self):
        def source(stage, outcome, elapsed=350):
            return {"event": "source", "sample": {"server_ms": 5, "selection": 7,
                    "generation": 11, "refresh": True, "stage": stage,
                    "outcome": outcome, "elapsed_ms": elapsed}}
        rows = [source("current_file_and_links", "passed"),
                source("direct_ranges", "failed"), source("publication", "replaced", 600),
                source("https://private.invalid/TOKEN", "failed"),
                source("publication", "PRIVATE_TOKEN"), source("publication", "failed", True),
                {"event": "phone", "sample": phone_sample(buffering=True)}]
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "session-0.jsonl").write_text("\n".join(
                json.dumps({"record": row}) for row in rows), encoding="utf-8")
            result = inspect(directory)
        self.assertEqual(result["source_events"], 3)
        self.assertEqual(result["source_failures"]["direct_ranges"], 1)
        self.assertEqual(result["source_failures"]["publication"], 0)
        self.assertEqual(result["source_replaced"], 1)
        self.assertEqual(result["maximum_source_elapsed_ms"], 600)
        self.assertEqual(result["invalid_records"], 3)
        self.assertEqual(result["phone_samples"], 1)
        self.assertEqual(result["maximum_absolute_drift_ms"], 125)
        encoded = json.dumps(result)
        self.assertNotIn("PRIVATE_TOKEN", encoded)
        self.assertNotIn("https://", encoded)
        self.assertFalse(result["physical_release_gate_satisfied"])

    def test_rotated_upload_retries_do_not_inflate_unique_samples_and_conflicts_keep_worst_flags(self):
        original = phone_sample()
        reordered = dict(reversed(list(original.items())))
        conflicting = phone_sample(fault=True, buffering=True, connected=False, drift_ms=900)
        distinct = phone_sample(elapsed_ms=12, generation=12)
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "session-0.jsonl").write_text("\n".join(json.dumps({"record": {"event": "phone", "sample": row}})
                for row in [original, reordered, conflicting]), encoding="utf-8")
            Path(directory, "session-1.jsonl").write_text("\n".join(json.dumps({"event": "phone", "sample": row})
                for row in [original, conflicting, distinct]), encoding="utf-8")
            result = inspect(directory)
        self.assertEqual(result["raw_phone_samples"], 6)
        self.assertEqual(result["phone_samples"], 2)
        self.assertEqual(result["duplicate_phone_samples"], 2)
        self.assertEqual(result["conflicting_phone_samples"], 2)
        self.assertEqual(result["conflicting_sample_identities"], 1)
        self.assertEqual(result["fault_samples"], 1)
        self.assertEqual(result["buffering_samples"], 1)
        self.assertEqual(result["disconnected_samples"], 1)
        self.assertEqual(result["maximum_absolute_drift_ms"], 900)
        self.assertEqual(result["invalid_records"], 0)
        self.assertFalse(result["physical_release_gate_satisfied"])

    def test_invalid_identity_unknown_fields_and_private_playback_labels_are_not_exported(self):
        invalid = [phone_sample(elapsed_ms=-1), phone_sample(generation=True),
                   phone_sample(utc_ms=9223372036854775808), phone_sample(rtt_ms=True),
                   phone_sample(playback="PRIVATE_TOKEN https://private.invalid"), phone_sample(private="PRIVATE_TOKEN")]
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "session-0.jsonl").write_text("\n".join(json.dumps({"event": "phone", "sample": row})
                for row in invalid), encoding="utf-8")
            result = inspect(directory)
        self.assertEqual(result["invalid_records"], len(invalid))
        self.assertEqual(result["raw_phone_samples"], 0)
        self.assertEqual(result["phone_samples"], 0)
        self.assertNotIn("PRIVATE_TOKEN", json.dumps(result))

    def test_oversized_physical_line_cannot_turn_its_json_suffix_into_a_valid_sample(self):
        row = json.dumps({"event": "phone", "sample": phone_sample()})
        with tempfile.TemporaryDirectory() as directory:
            Path(directory, "session-0.jsonl").write_text("x" * 16385 + row + "\n" + row + "\n", encoding="utf-8")
            result = inspect(directory)
        self.assertEqual(result["invalid_records"], 1)
        self.assertEqual(result["phone_samples"], 1)
        self.assertEqual(result["raw_phone_samples"], 1)


if __name__ == "__main__":
    unittest.main()
