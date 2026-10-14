import unittest
import subprocess
import sys
import tempfile
from pathlib import Path
from verify_instrumentation import verify


def packet(code, name="nativePlayback", current=1, total=1):
    return ["INSTRUMENTATION_STATUS: id=AndroidJUnitRunner",
            "INSTRUMENTATION_STATUS: class=app.synthetic.NativeTest",
            f"INSTRUMENTATION_STATUS: test={name}",
            f"INSTRUMENTATION_STATUS: numtests={total}",
            f"INSTRUMENTATION_STATUS: current={current}",
            "INSTRUMENTATION_STATUS: stream=PRIVATE_SYNTHETIC_DETAIL",
            f"INSTRUMENTATION_STATUS_CODE: {code}"]


class InstrumentationStatusTests(unittest.TestCase):
    def test_same_count_cannot_substitute_a_different_selected_test(self):
        lines = packet(1) + packet(0) + ["INSTRUMENTATION_CODE: -1"]
        selected = [["app.synthetic.NativeTest", "nativePlayback"]]
        self.assertTrue(verify(lines, 1, selected)["expected_test_identities_verified"])
        with self.assertRaises(ValueError):
            verify(lines, 1, [["app.synthetic.NativeTest", "differentTest"]])
        with self.assertRaises(ValueError):
            verify(lines, 1, [["app.synthetic.OtherClass", "nativePlayback"]])

    def test_identity_manifest_requires_unique_complete_pairs(self):
        for selected in ([], [["class"]], [["", "test"]], [[1, "test"]], "invalid"):
            with self.subTest(selected=selected), self.assertRaises(ValueError):
                verify([], 1, selected)
        with self.assertRaises(ValueError):
            verify([], 2, [["class", "test"], ["class", "test"]])

    def test_cli_rejects_malformed_private_status_without_echoing_it(self):
        with tempfile.TemporaryDirectory(prefix="PRIVATE_STATUS_") as folder:
            file = Path(folder) / "PRIVATE_RESULT.txt"
            file.write_text("INSTRUMENTATION_STATUS_CODE: PRIVATE_TOKEN\n", encoding="utf-8")
            result = subprocess.run([sys.executable, "-B", str(Path(__file__).with_name("verify_instrumentation.py")),
                                     str(file), "--expected-tests", "1"], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stdout, "")
            self.assertEqual(result.stderr, "instrumentation_status_verification_failed\n")

    def test_success_requires_matching_start_and_finish_for_every_test(self):
        lines = packet(1, total=2) + packet(0, total=2)
        lines += packet(1, "seek", 2, 2) + packet(0, "seek", 2, 2)
        report = verify(lines + ["INSTRUMENTATION_CODE: -1"], 2)
        self.assertEqual(report["completed_tests"], 2)
        self.assertNotIn("PRIVATE", str(report))
        self.assertFalse(report["physical_release_gate_satisfied"])

    def test_ignored_assumption_failure_error_and_unknown_status_rejected(self):
        for code in (-1, -2, -3, -4, 2):
            with self.subTest(code=code), self.assertRaises(ValueError):
                verify(packet(1) + packet(code) + ["OK (1 tests)", "INSTRUMENTATION_CODE: -1"], 1)

    def test_missing_duplicate_mismatched_or_truncated_status_rejected(self):
        for lines in (packet(0), packet(1), packet(1) + packet(1),
                      packet(1) + packet(0, "other"), packet(1) + packet(0, current=2),
                      packet(1, total=2) + packet(0, total=2), ["OK (1 tests)"],
                      packet(1) + packet(0) + ["INSTRUMENTATION_CODE: 0"]):
            with self.subTest(lines=len(lines)), self.assertRaises(ValueError):
                verify(lines, 1)

    def test_process_failure_and_duplicate_final_result_rejected(self):
        valid = packet(1) + packet(0) + ["INSTRUMENTATION_CODE: -1"]
        for extra in ("INSTRUMENTATION_CODE: -1", "INSTRUMENTATION_FAILED: PRIVATE",
                      "INSTRUMENTATION_RESULT: shortMsg=PRIVATE"):
            with self.assertRaises(ValueError): verify(valid + [extra], 1)
        with self.assertRaises(ValueError):
            verify(["INSTRUMENTATION_RESULT: shortMsg=PRIVATE"] + valid, 1)


if __name__ == "__main__":
    unittest.main()
