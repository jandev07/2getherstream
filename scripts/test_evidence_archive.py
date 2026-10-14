import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
import os
from unittest.mock import patch
from verify_evidence_archive import verify


class EvidenceArchiveTests(unittest.TestCase):
    def fixture(self, root):
        (root / "nested").mkdir()
        (root / "nested" / "record.json").write_bytes(b"private synthetic record")
        return [{"file": "nested\\record.json", "Hash": hashlib.sha256(b"private synthetic record").hexdigest()}]

    def manifest(self, root, rows):
        (root / "evidence-manifest.json").write_text(json.dumps(rows), encoding="utf-8")

    def test_valid_archive_reports_counts_not_contents_or_release_acceptance(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.manifest(root, self.fixture(root))
            self.assertEqual(verify(root), {"verified_files": 1, "manifest_authenticated": False,
                                           "physical_release_gate_satisfied": False})

    def test_modified_missing_and_unlisted_files_are_rejected(self):
        for change in ("modified", "missing", "extra"):
            with self.subTest(change=change), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                self.manifest(root, self.fixture(root))
                file = root / "nested" / "record.json"
                if change == "modified": file.write_bytes(b"changed")
                elif change == "missing": file.unlink()
                else: (root / "unlisted.txt").write_bytes(b"unexpected")
                with self.assertRaises((ValueError, OSError)): verify(root)

    def test_unsafe_paths_duplicate_entries_and_bad_hashes_are_rejected(self):
        for bad in ("../outside", "C:\\private", "\\\\host\\private", "nested/../record.json", "nested/record.json:stream", "nested//record.json", "nested/record.json.", "nested/record.json "):
            with self.subTest(path=bad), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                rows = self.fixture(root)
                rows[0]["file"] = bad
                self.manifest(root, rows)
                with self.assertRaises(ValueError): verify(root)
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            rows = self.fixture(root)
            self.manifest(root, rows + rows)
            with self.assertRaises(ValueError): verify(root)
            rows[0]["Hash"] = "not-a-checksum"
            self.manifest(root, rows)
            with self.assertRaises(ValueError): verify(root)

    def test_cli_failure_does_not_echo_private_path_or_data(self):
        with tempfile.TemporaryDirectory(prefix="PRIVATE_ARCHIVE_") as folder:
            root = Path(folder)
            self.manifest(root, [{"file": "PRIVATE_DATA", "Hash": "invalid"}])
            result = subprocess.run([sys.executable, "-B", str(Path(__file__).with_name("verify_evidence_archive.py")), folder], capture_output=True, text=True)
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stdout, "")
            self.assertEqual(result.stderr, "evidence_archive_verification_failed\n")

    def test_inventory_and_manifest_bounds(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            self.manifest(root, self.fixture(root))
            with patch("verify_evidence_archive.MAX_ENTRIES", 2):
                with self.assertRaisesRegex(ValueError, "archive_inventory_too_large"):
                    verify(root)
            (root / "evidence-manifest.json").write_bytes(b" " * (4 * 1024 * 1024 + 1))
            with self.assertRaisesRegex(ValueError, "invalid_manifest"):
                verify(root)

    @unittest.skipUnless(os.name == "nt", "Windows junction semantics")
    def test_root_parent_and_nested_junctions_rejected_without_target_changes(self):
        with tempfile.TemporaryDirectory() as folder:
            base = Path(folder)
            root = base / "archive"
            root.mkdir()
            self.manifest(root, self.fixture(root))
            original = (root / "nested" / "record.json").read_bytes()
            for link, destination, selected in (
                (base / "redirect", root, base / "redirect"),
                (base / "parent", base, base / "parent" / "archive"),
                (root / "extra", root / "nested", root),
            ):
                result = subprocess.run(["cmd.exe", "/c", "mklink", "/J", str(link), str(destination)], capture_output=True)
                self.assertEqual(result.returncode, 0)
                try:
                    with self.assertRaisesRegex(ValueError, "redirected_archive_path"):
                        verify(selected)
                    self.assertEqual((root / "nested" / "record.json").read_bytes(), original)
                finally:
                    # Unlink only the verified fixture junction, never its target.
                    os.rmdir(link)
            self.assertEqual(verify(root)["verified_files"], 1)


if __name__ == "__main__":
    unittest.main()
