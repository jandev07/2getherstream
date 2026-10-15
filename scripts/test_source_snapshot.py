import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch


spec = importlib.util.spec_from_file_location("source_snapshot", Path(__file__).with_name("Publish-SourceSnapshot.py"))
publisher = importlib.util.module_from_spec(spec)
spec.loader.exec_module(publisher)


class SourceSnapshotRepositoryTests(unittest.TestCase):
    def test_inherited_git_repository_and_config_cannot_retarget_source(self):
        executable = shutil.which("git")
        if executable is None:
            self.skipTest("Git is unavailable")
        clean = {key: value for key, value in os.environ.items() if not key.upper().startswith("GIT_")}

        def git(directory, *args):
            return subprocess.run(
                [executable, "-c", "user.name=Source snapshot test", "-c", "user.email=snapshot@test.invalid",
                 "-c", "commit.gpgsign=false", *args], cwd=directory, env=clean,
                capture_output=True, check=True, timeout=15).stdout

        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            target, unrelated = root / "target", root / "unrelated"
            for directory in (target, unrelated):
                directory.mkdir()
                git(directory, "init", "-q")
                (directory / "source.txt").write_text(directory.name, encoding="utf-8")
                git(directory, "add", "source.txt")
                git(directory, "commit", "-q", "-m", directory.name)
                git(directory, "config", "snapshot.owner", directory.name)

            def state(directory):
                return (git(directory, "rev-parse", "HEAD"),
                        (directory / ".git/index").read_bytes(),
                        (directory / ".git/config").read_bytes())

            before = {directory: state(directory) for directory in (target, unrelated)}
            injected = {"GIT_DIR": str(unrelated / ".git"), "GIT_WORK_TREE": str(unrelated),
                        "GIT_INDEX_FILE": str(unrelated / ".git/index"), "GIT_CONFIG_COUNT": "1",
                        "GIT_CONFIG_KEY_0": "snapshot.owner", "GIT_CONFIG_VALUE_0": "injected"}
            try:
                with patch.object(publisher, "ROOT", target), patch.dict(os.environ, injected):
                    self.assertEqual(publisher.git("rev-parse", "HEAD"), before[target][0])
                    self.assertEqual(publisher.git("show", ":source.txt"), b"target")
                    self.assertEqual(publisher.git("config", "--get", "snapshot.owner"), b"target\n")
                    self.assertEqual({key: os.environ[key] for key in injected}, injected)
            finally:
                self.assertEqual({directory: state(directory) for directory in before}, before)

    def test_timeout_feedback_does_not_expose_child_output(self):
        timeout = subprocess.TimeoutExpired(["git", "fetch"], 120,
                                            output=b"synthetic-private-output", stderr=b"synthetic-private-error")
        with patch.object(publisher.subprocess, "run", side_effect=timeout):
            with self.assertRaisesRegex(RuntimeError, "Git source operation timed out: fetch") as error:
                publisher.git("fetch", "github", "main")
        import traceback
        rendered = "".join(traceback.format_exception(error.exception))
        self.assertNotIn("synthetic-private", rendered)


if __name__ == "__main__":
    unittest.main()
