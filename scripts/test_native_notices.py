import importlib.util
import io
import json
from pathlib import Path
import hashlib
import subprocess
import tarfile
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("native_notices", Path(__file__).with_name("Verify-NativeNotices.py"))
notices = importlib.util.module_from_spec(spec)
spec.loader.exec_module(notices)


class NativeNoticeTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.directory = self.root / "third-party/native-components"
        self.directory.mkdir(parents=True)
        self.blob = b"Original notice\r\nKeep exact bytes.\r\n"
        (self.directory / "LICENSE").write_bytes(self.blob)
        self.record = {"notice_file": "third-party/native-components/LICENSE",
                       "sha256": hashlib.sha256(self.blob).hexdigest(),
                       "source_archive": "source.tar.gz", "member": "source/LICENSE"}
        self.archives = self.root / "archives"
        self.archives.mkdir()
        self.write_manifest([self.record])
        self.write_archive([("source/LICENSE", self.blob)])

    def write_manifest(self, records):
        (self.directory / "PROVENANCE.json").write_text(json.dumps(records), encoding="utf-8")

    def write_archive(self, entries):
        with tarfile.open(self.archives / "source.tar.gz", "w:gz") as archive:
            for name, blob in entries:
                member = tarfile.TarInfo(name)
                member.size = len(blob)
                archive.addfile(member, io.BytesIO(blob))

    def test_shared_notice_preserves_distinct_archive_and_url_evidence(self):
        remote = dict(self.record)
        del remote["source_archive"]
        remote["source_url"] = "https://example.invalid/original/LICENSE"
        self.write_manifest([self.record, remote])
        result = notices.verify(self.root, self.archives)
        self.assertEqual((result["provenance_records"], result["unique_notice_files"]), (2, 1))
        self.assertEqual(result["archive_member_associations_verified"], 1)
        self.assertEqual(result["url_only_associations_unchecked"], 1)
        self.assertFalse(result["distribution_completeness_verified"])
        self.assertEqual(notices.verify(self.root)["archive_associations_unchecked"], 1)

    def test_changed_retained_line_endings_are_rejected(self):
        (self.directory / "LICENSE").write_bytes(self.blob.replace(b"\r\n", b"\n"))
        with self.assertRaisesRegex(ValueError, "Retained notice hash mismatch"):
            notices.verify(self.root, self.archives)

    def test_wrong_archive_content_or_member_is_rejected(self):
        for entries in [[("source/LICENSE", b"Changed notice")], [("other/LICENSE", self.blob)]]:
            with self.subTest(entries=entries):
                self.write_archive(entries)
                with self.assertRaisesRegex(ValueError, "Source notice member mismatch"):
                    notices.verify(self.root, self.archives)

    def test_duplicate_archive_member_is_rejected(self):
        self.write_archive([("source/LICENSE", self.blob)] * 2)
        with self.assertRaisesRegex(ValueError, "Duplicate source archive member"):
            notices.verify(self.root, self.archives)

    def test_missing_or_ambiguous_archive_is_rejected(self):
        (self.archives / "source.tar.gz").rename(self.archives / "absent.tar.gz")
        with self.assertRaisesRegex(ValueError, "Missing source archives"):
            notices.verify(self.root, self.archives)
        self.write_archive([("source/LICENSE", self.blob)])
        other = self.archives / "other"
        other.mkdir()
        (other / "source.tar.gz").write_bytes((self.archives / "source.tar.gz").read_bytes())
        self.assertEqual(notices.verify(self.root, self.archives)["archive_member_associations_verified"], 1)
        (other / "source.tar.gz").write_bytes(b"Conflicting archive with the same basename")
        with self.assertRaisesRegex(ValueError, "Ambiguous source archive basename"):
            notices.verify(self.root, self.archives)

    def test_notice_path_cannot_escape_retained_tree(self):
        for name in ["../LICENSE", "third-party/native-components/../../LICENSE",
                     "third-party/native-components/C:secret", "third-party\\native-components\\LICENSE"]:
            with self.subTest(name=name):
                self.write_manifest([dict(self.record, notice_file=name)])
                with self.assertRaisesRegex(ValueError, "Invalid notice path"):
                    notices.verify(self.root)

    def test_conflicting_digests_for_shared_file_are_rejected(self):
        self.write_manifest([self.record, dict(self.record, sha256="0" * 64)])
        with self.assertRaisesRegex(ValueError, "Conflicting retained notice digest"):
            notices.verify(self.root)

    def test_staged_check_requires_exact_index_bytes(self):
        subprocess.run(["git", "init", "-q"], cwd=self.root, check=True, capture_output=True)
        with self.assertRaisesRegex(ValueError, "Staged notice differs"):
            notices.verify(self.root, staged=True)
        # Disable line conversion in this isolated repository only.
        subprocess.run(["git", "-c", "core.autocrlf=false", "add", "third-party/native-components/LICENSE"],
                       cwd=self.root, check=True, capture_output=True)
        self.assertTrue(notices.verify(self.root, staged=True)["staged_notice_bytes_verified"])
        changed = self.blob + b"A new retained line\r\n"
        (self.directory / "LICENSE").write_bytes(changed)
        self.write_manifest([dict(self.record, sha256=hashlib.sha256(changed).hexdigest())])
        with self.assertRaisesRegex(ValueError, "Staged notice differs"):
            notices.verify(self.root, staged=True)


if __name__ == "__main__":
    unittest.main()
