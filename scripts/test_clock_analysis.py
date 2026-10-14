import json
import unittest
import subprocess
import sys
import tempfile
from pathlib import Path
from analyze_clock_proof import analyze

RECEIPT = dict(run_id="09fdab9b-262a-481f-b5f9-3d8bb705ce58", status="passed", phase="complete", validation_scope="emulator_synthetic_fixture", requested_uninterrupted_seconds=600, completed_uninterrupted_seconds=600, completed_utc="2026-10-08T05:05:23Z")

def row(**changes):
    result = dict(run_id=RECEIPT["run_id"], phase="steady", generation=30, client_ms=1000, offset_ms=500,
                  clock_sample_age_ms=30000, clock_history_size=64, drift_ms=-59,
                  sample_age_ms=100, connected=True, buffering=False, seeking=False,
                  preparing=False, state="Playing")
    result.update(changes)
    return json.dumps(result)

class ClockAnalysisTests(unittest.TestCase):
    def test_clock_windows_use_time_boundaries_and_split_generation_changes(self):
        report=analyze(RECEIPT,[row(client_ms=1000,drift_ms=-10),
            row(client_ms=60999,offset_ms=510,drift_ms=20),
            row(client_ms=61000,offset_ms=700,drift_ms=-30),
            row(client_ms=61100,generation=31,offset_ms=9000,drift_ms=40)])
        windows=report["steady_clock_windows"]
        self.assertEqual(len(windows),3)
        self.assertEqual([w["window_index"] for w in windows],[0,1,1])
        self.assertEqual(windows[0]["samples"],2)
        self.assertEqual(windows[0]["observed_span_ms"],59999)
        self.assertEqual(windows[0]["used_offset_span_ms"],10)
        self.assertEqual(windows[0]["maximum_authority_drift_ms"],20)
        self.assertEqual(windows[1]["used_offset_span_ms"],0)
        self.assertEqual(windows[2]["generation"],31)
        self.assertEqual(windows[0]["interval_observed_samples"],0)
        self.assertEqual(report["omitted_clock_windows"],0)

    def test_windows_keep_interval_fallback_explicit_and_bound_storage(self):
        fields=dict(clock_interval_consistent=False,clock_support_samples=1,
                    clock_support_age_ms=0,raw_selected_offset_ms=500)
        rows=[row(client_ms=1000+i*60000,drift_ms=i,secret="PRIVATE_INPUT",**fields) for i in range(140)]
        report=analyze(RECEIPT,rows)
        self.assertEqual(len(report["steady_clock_windows"]),128)
        self.assertEqual(report["omitted_clock_windows"],12)
        self.assertEqual(report["steady_samples"],140)
        self.assertEqual(report["maxima"]["authority_drift_ms"],139)
        self.assertTrue(all(w["interval_fallback_samples"]==1 for w in report["steady_clock_windows"]))
        self.assertTrue(all(w["maximum_interval_width_ms"] is None for w in report["steady_clock_windows"]))
        self.assertNotIn("PRIVATE",json.dumps(report))

    def test_consistent_interval_width_is_visible_in_its_window(self):
        fields=dict(clock_interval_consistent=True,clock_lower_offset_ms=480,
                    clock_upper_offset_ms=520,clock_support_samples=2,
                    clock_support_age_ms=100,raw_selected_offset_ms=600)
        report=analyze(RECEIPT,[row(**fields)])
        window=report["steady_clock_windows"][0]
        self.assertEqual(window["interval_observed_samples"],1)
        self.assertEqual(window["interval_fallback_samples"],0)
        self.assertEqual(window["maximum_interval_width_ms"],40)

    def test_interval_metadata_tracks_raw_steps_without_changing_used_steps(self):
        fields=dict(clock_interval_consistent=True,clock_lower_offset_ms=480,
                    clock_upper_offset_ms=520,clock_support_samples=2,
                    clock_support_age_ms=100,raw_selected_offset_ms=600)
        report=analyze(RECEIPT,[row(**fields),row(client_ms=1100,**dict(fields,raw_selected_offset_ms=550))])
        data=report["interval_diagnostics"]
        self.assertEqual(data["observed_samples"],2)
        self.assertEqual(data["consistent_samples"],2)
        self.assertEqual(data["fallback_samples"],0)
        self.assertEqual(data["maximum_width_ms"],40)
        self.assertEqual(data["maximum_raw_offset_step_ms"],50)
        self.assertEqual(data["raw_offset_transition_count"],1)
        self.assertEqual(report["maxima"]["selected_offset_change_ms"],0)

    def test_explicit_fallback_and_legacy_missing_metadata_are_distinct(self):
        fields=dict(clock_interval_consistent=False,clock_support_samples=2,
                    clock_support_age_ms=100,raw_selected_offset_ms=500)
        report=analyze(RECEIPT,[row(**fields)])
        self.assertEqual(report["interval_diagnostics"]["fallback_samples"],1)
        self.assertIsNone(report["interval_diagnostics"]["maximum_width_ms"])
        legacy=analyze(RECEIPT,[row()])["interval_diagnostics"]
        self.assertEqual(legacy["observed_samples"],0)
        self.assertEqual(legacy["fallback_samples"],0)
        self.assertIsNone(legacy["maximum_raw_offset_step_ms"])

    def test_invalid_interval_metadata_cannot_claim_a_valid_midpoint(self):
        fields=dict(clock_interval_consistent=True,clock_lower_offset_ms=480,
                    clock_upper_offset_ms=520,clock_support_samples=2,
                    clock_support_age_ms=100,raw_selected_offset_ms=600)
        invalids=[dict(clock_interval_consistent="yes"),dict(clock_lower_offset_ms=521),
                  dict(clock_upper_offset_ms=float("inf")),dict(clock_support_samples=True),
                  dict(clock_support_samples=65),dict(clock_support_age_ms=30001),
                  dict(raw_selected_offset_ms="PRIVATE_INPUT"),dict(offset_ms=501),
                  dict(clock_interval_consistent=False)]
        for invalid in invalids:
            with self.assertRaises(ValueError):
                analyze(RECEIPT,[row(**dict(fields,**invalid))])

    def test_interval_raw_steps_do_not_cross_generation_or_missing_metadata(self):
        fields=dict(clock_interval_consistent=False,clock_support_samples=1,
                    clock_support_age_ms=0,raw_selected_offset_ms=500)
        report=analyze(RECEIPT,[row(**fields),row(client_ms=1100),
                    row(client_ms=1200,offset_ms=9000,**dict(fields,raw_selected_offset_ms=9000)),
                    row(client_ms=1300,generation=31,offset_ms=2000,**dict(fields,raw_selected_offset_ms=2000))])
        self.assertEqual(report["interval_diagnostics"]["raw_offset_transition_count"],0)
        self.assertEqual(report["interval_diagnostics"]["maximum_raw_offset_step_ms"],0)

    def test_offset_step_context_preserves_signed_drift_and_original_maxima(self):
        report = analyze(RECEIPT, [row(offset_ms=500, drift_ms=-21, clock_sample_age_ms=1831),
                                  row(client_ms=1100, offset_ms=664.5, drift_ms=-189, clock_sample_age_ms=8),
                                  row(client_ms=1200, offset_ms=526, drift_ms=-52)])
        self.assertEqual(report["offset_transition_count"], 2)
        context = report["largest_offset_step_context"]
        self.assertEqual(context["signed_offset_step_ms"], 164.5)
        self.assertEqual(context["authority_drift_change_ms"], -168)
        self.assertEqual(context["drift_change_plus_offset_step_ms"], -3.5)
        self.assertEqual(context["client_gap_ms"], 100)
        self.assertEqual(context["intervening_nonsteady_rows"], 0)
        self.assertEqual(context["selected_age_before_ms"], 1831)
        self.assertEqual(context["selected_age_after_ms"], 8)
        self.assertTrue(context["both_observations_playing_and_available"])
        self.assertEqual(report["maxima"]["selected_offset_change_ms"], 164.5)
        self.assertEqual(report["maxima"]["authority_drift_ms"], 189)
        self.assertEqual(sum(report["violation_counts"].values()), 0)

    def test_negative_step_and_excluded_rows_are_not_hidden(self):
        report = analyze(RECEIPT, [row(offset_ms=700, drift_ms=-100),
                                  row(phase="recovery", secret="PRIVATE_URL"),
                                  row(client_ms=4000, offset_ms=500, drift_ms=100, buffering=True)])
        context = report["largest_offset_step_context"]
        self.assertEqual(context["signed_offset_step_ms"], -200)
        self.assertEqual(context["drift_change_plus_offset_step_ms"], 0)
        self.assertEqual(context["client_gap_ms"], 3000)
        self.assertEqual(context["intervening_nonsteady_rows"], 1)
        self.assertFalse(context["both_observations_playing_and_available"])
        self.assertNotIn("PRIVATE", json.dumps(report))

    def test_generation_boundary_and_unchanged_offsets_have_no_context(self):
        for rows in ([row()], [row(), row(client_ms=1100)],
                     [row(), row(client_ms=1100, generation=31, offset_ms=9000)]):
            report = analyze(RECEIPT, rows)
            self.assertEqual(report["offset_transition_count"], 0)
            self.assertIsNone(report["largest_offset_step_context"])

    def test_derived_overflow_is_rejected_without_nonfinite_report(self):
        for before, after in ((dict(offset_ms=-1e308), dict(offset_ms=1e308)),
                              (dict(drift_ms=-1e308), dict(drift_ms=1e308))):
            with self.assertRaises(ValueError):
                analyze(RECEIPT, [row(**before), row(client_ms=1100, **after)])

    def test_monotonic_time_is_required_across_generation_boundaries(self):
        with self.assertRaises(ValueError):
            analyze(RECEIPT, [row(client_ms=1100), row(generation=31, client_ms=1000)])

    def test_observed_span_and_largest_gap_do_not_claim_receipt_duration(self):
        report = analyze(RECEIPT, [row(client_ms=1000), row(client_ms=1100),
                                  row(generation=31, client_ms=3100)])
        self.assertEqual(report["steady_observed_span_ms"], 2100)
        self.assertEqual(report["max_steady_gap_ms"], 2000)
        self.assertEqual(report["steady_samples"], 3)
        self.assertFalse(report["physical_release_gate_satisfied"])
        single = analyze(RECEIPT, [row()])
        self.assertEqual(single["steady_observed_span_ms"], 0)
        self.assertEqual(single["max_steady_gap_ms"], 0)

    def test_requested_duration_must_be_an_integer_not_boolean(self):
        with self.assertRaises(ValueError):
            analyze(dict(RECEIPT, requested_uninterrupted_seconds=True,
                         completed_uninterrupted_seconds=1), [row()])

    def test_exact_numeric_summary_excludes_text_and_generation_boundary_jumps(self):
        report = analyze(RECEIPT, [row(secret="PRIVATE_KEY_URL"), row(client_ms=1100, offset_ms=600), row(generation=31, client_ms=1200, offset_ms=9000)])
        self.assertEqual(report["maxima"]["selected_offset_change_ms"], 100)
        self.assertEqual(report["maxima"]["authority_drift_ms"], 59)
        self.assertEqual(report["steady_generation_count"], 2)
        self.assertEqual(sum(report["violation_counts"].values()), 0)
        self.assertNotIn("PRIVATE", json.dumps(report))
        self.assertFalse(report["physical_release_gate_satisfied"])

    def test_rejects_running_receipt_missing_or_invalid_numeric_data(self):
        with self.assertRaises(ValueError):
            analyze(dict(RECEIPT, status="running"), [row()])
        with self.assertRaises(ValueError):
            analyze(dict(RECEIPT, completed_uninterrupted_seconds=None), [row()])
        for malformed in ("[]", "null", "123"):
            with self.assertRaises(ValueError):
                analyze(RECEIPT, [malformed])
        for bad in (None, True, float("nan"), float("inf"), "PRIVATE"):
            with self.assertRaises(ValueError):
                analyze(RECEIPT, [row(offset_ms=bad)])
        with self.assertRaises(ValueError):
            analyze(RECEIPT, [row(client_ms=1100), row(client_ms=1000)])

    def test_cli_refuses_running_receipt_and_existing_output_without_input_leak(self):
        with tempfile.TemporaryDirectory(prefix="together-clock-analysis-") as directory:
            directory = Path(directory)
            receipt = directory / "receipt.json"
            phone = directory / "phone.jsonl"
            output = directory / "summary.json"
            phone.write_text(row(secret="PRIVATE_INPUT") + "\n", encoding="utf-8")
            command = [sys.executable, "-B", str(Path(__file__).with_name("analyze_clock_proof.py")), "--receipt", str(receipt), "--phone-log", str(phone), "--output", str(output)]
            receipt.write_text(json.dumps(dict(RECEIPT, status="running")), encoding="utf-8")
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stderr, "clock_analysis_failed\n")
            self.assertFalse(output.exists())
            receipt.write_text(json.dumps(RECEIPT), encoding="utf-8")
            output.write_text("EXISTING_EVIDENCE", encoding="utf-8")
            result = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stderr, "clock_analysis_failed\n")
            self.assertEqual(output.read_text(encoding="utf-8"), "EXISTING_EVIDENCE")

    def test_run_identity_covers_all_phases_and_legacy_is_explicit(self):
        for phase in ("setup", "steady", "recovery"):
            for bad in (None, "different-run", "19fdab9b-262a-481f-b5f9-3d8bb705ce58", True):
                with self.assertRaises(ValueError):
                    analyze(RECEIPT, [row(), row(phase=phase, run_id=bad)])
        self.assertTrue(analyze(RECEIPT, [row()])["log_identity_verified"])
        old=json.loads(row());old.pop("run_id")
        with self.assertRaises(ValueError): analyze(RECEIPT, [json.dumps(old)])
        self.assertFalse(analyze(RECEIPT, [json.dumps(old)], True)["log_identity_verified"])
        with self.assertRaises(ValueError): analyze(dict(RECEIPT, proof_log_schema=2), [json.dumps(old)], True)
        with self.assertRaises(ValueError): analyze(RECEIPT, [row(run_id="other")], True)

    def test_reports_freshness_capacity_and_playback_violations(self):
        report = analyze(RECEIPT, [row(clock_sample_age_ms=30001, clock_history_size=65, connected=False, buffering=True, seeking=True, preparing=True, state="Paused")])
        self.assertEqual(report["violation_counts"], dict(clock_age=1, history_size=1, disconnected=1, buffering=1, seeking=1, preparing=1, not_playing=1))
        with self.assertRaises(ValueError):
            analyze(RECEIPT, [row(phase="recovery")])

if __name__ == "__main__":
    unittest.main()
