import copy
import hashlib
import json
from pathlib import Path
import tempfile
import unittest
import android_download_stage as staging


class StagingTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="2gether-download-stage-")
        self.root = Path(self.temp.name)
        self.source = self.root / "ordinary.bin"
        self.source.write_bytes(b"abcdefghij")
        self.destination = self.root / "assets"
        self.release = {"protocol": 1, "package": "app.together.v2", "version_code": 9, "version_name": "0.1.8-private-alpha", "apk_url": staging.REPOSITORY + "/releases/download/android-v9/2gether.apk", "sha256": hashlib.sha256(self.source.read_bytes()).hexdigest(), "size_bytes": 10}

    def tearDown(self):
        self.temp.cleanup()

    def stage(self, release=None):
        return staging.stage_release(self.source, self.release if release is None else release, self.destination)

    def test_metadata_only_and_idempotent_exact_identity(self):
        self.assertEqual(self.stage(), self.release)
        self.assertEqual(json.loads((self.destination / "release.json").read_text()), self.release)
        self.assertEqual(self.stage(), self.release)
        self.assertFalse(any(self.destination.rglob("*.bin")))
        self.assertFalse(any(self.destination.rglob("*.apk")))

    def test_same_version_different_content_cannot_replace_metadata(self):
        self.stage()
        original = (self.destination / "release.json").read_bytes()
        self.source.write_bytes(b"ABCDEFGHIJ")
        changed = copy.deepcopy(self.release)
        changed["sha256"] = hashlib.sha256(self.source.read_bytes()).hexdigest()
        with self.assertRaises(ValueError): self.stage(changed)
        self.assertEqual((self.destination / "release.json").read_bytes(), original)

    def test_failed_hash_does_not_publish_manifest(self):
        self.release["sha256"] = "a" * 64
        with self.assertRaises(ValueError): self.stage()
        self.assertFalse(self.destination.exists())

    def test_higher_version_retains_old_manifest_and_rejects_downgrade(self):
        self.stage()
        higher = copy.deepcopy(self.release)
        higher.update(version_code=10, apk_url=staging.REPOSITORY + "/releases/download/android-v10/2gether.apk")
        self.stage(higher)
        self.assertEqual(json.loads((self.destination / "releases/9/release.json").read_text()), self.release)
        with self.assertRaises(ValueError): self.stage()

    def test_expiring_url_and_coerced_fields_rejected(self):
        for key, value in [("apk_url", self.release["apk_url"] + "?expires=1"), ("package", "legacy.app"), ("size_bytes", 11), ("version_code", True), ("protocol", True), ("version_name", "bad\x7f"), ("version_code", 2147483648)]:
            with self.subTest(key=key):
                invalid = copy.deepcopy(self.release)
                invalid[key] = value
                with self.assertRaises(ValueError): self.stage(invalid)
        self.assertFalse(self.destination.exists())

    def test_held_publication_lock_rejects_competing_writer_then_releases(self):
        with staging.publication_lock(self.destination):
            with self.assertRaises(OSError): self.stage()
            self.assertFalse((self.destination / "release.json").exists())
        self.stage()
        self.assertTrue((self.destination / "release.json").is_file())


if __name__ == "__main__":
    unittest.main()
