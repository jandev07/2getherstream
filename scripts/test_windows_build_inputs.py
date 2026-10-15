import importlib.util
import copy
import io
import json
import os
from pathlib import Path
import hashlib
import shutil
import stat
import subprocess
import tarfile
import tempfile
import unittest
import unittest.mock
import zipfile

spec = importlib.util.spec_from_file_location("windows_build_inputs", Path(__file__).with_name("Verify-WindowsBuildInputs.py"))
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


class WindowsBuildInputsTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)

        # Setup minimal synthetic native directory and archives
        self.native_dir = self.root / "native"
        self.native_dir.mkdir(parents=True)
        self.archives_dir = self.root / "archives"
        self.archives_dir.mkdir(parents=True)

        self.pkg_content = b"# minimal package cmake\n"
        self.tc_content = b"# minimal toolchain cmake\n"
        self.wf_content = b"# minimal workflow yaml\n"
        self.helper_content = b"# minimal helper cmake\n"
        self.version_py = b"import subprocess\nprint('v1.0.0')\n"

        self.pkg_sha = hashlib.sha256(self.pkg_content).hexdigest()
        self.tc_sha = hashlib.sha256(self.tc_content).hexdigest()
        self.wf_sha = hashlib.sha256(self.wf_content).hexdigest()
        self.helper_sha = hashlib.sha256(self.helper_content).hexdigest()
        self.version_py_sha = hashlib.sha256(self.version_py).hexdigest()

        self.prefix = "mpv-winbuild-cmake-test/"

        # Build synthetic zip
        self.zip_path = self.archives_dir / "windows-build-b2856ee.zip"
        self.write_zip({
            self.prefix + "packages/amf-headers.cmake": self.pkg_content,
            self.prefix + "packages/glad.cmake": self.pkg_content,
            self.prefix + "packages/libplacebo.cmake": self.pkg_content,
            self.prefix + "cmake/custom_steps.cmake": self.helper_content,
            self.prefix + ".github/workflows/mpv_clang.yml": self.wf_content,
            self.prefix + ".github/workflows/llvm_clang.yml": self.wf_content,
            self.prefix + "toolchain/llvm/llvm.cmake": self.tc_content,
        })
        self.zip_sha = hashlib.sha256(self.zip_path.read_bytes()).hexdigest()

        # Build synthetic tar.gz
        self.tar_path = self.archives_dir / "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f.tar.gz"
        self.write_tar({
            "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f/src/version.py": self.version_py,
        })
        self.tar_sha = hashlib.sha256(self.tar_path.read_bytes()).hexdigest()

        self.manifest = {
            "status": "build-configurations-not-complete-binary-source-graph",
            "build_repository_commit": "b2856ee6deed6957380b32465f359e67f67fafd8",
            "build_archive_sha256": self.zip_sha,
            "packages": [
                {
                    "project": "amf-headers",
                    "configuration_member": "packages/amf-headers.cmake",
                    "configuration_sha256": self.pkg_sha,
                },
                {
                    "project": "glad",
                    "configuration_member": "packages/glad.cmake",
                    "configuration_sha256": self.pkg_sha,
                    "build_configuration_analysis": {
                        "build_helper_member": "cmake/custom_steps.cmake",
                        "build_helper_sha256": self.helper_sha,
                    }
                },
                {
                    "project": "libplacebo",
                    "configuration_member": "packages/libplacebo.cmake",
                    "configuration_sha256": self.pkg_sha,
                    "binary_reported_source": {
                        "source_archive_sha256": self.tar_sha,
                        "version_source_evidence": [
                            {
                                "member": "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f/src/version.py",
                                "sha256": self.version_py_sha,
                            }
                        ]
                    }
                }
            ],
            "toolchain_inputs": {
                "ci_workflow": {
                    "workflow_member": ".github/workflows/mpv_clang.yml",
                    "workflow_sha256": self.wf_sha,
                    "toolchain_workflow_member": ".github/workflows/llvm_clang.yml",
                    "toolchain_workflow_sha256": self.wf_sha,
                },
                "recipes": [
                    {
                        "name": "llvm",
                        "configuration_member": "toolchain/llvm/llvm.cmake",
                        "configuration_sha256": self.tc_sha,
                    }
                ]
            }
        }
        self.write_manifest(self.manifest)

    def write_manifest(self, data):
        (self.native_dir / "WINDOWS-BUILD-INPUTS.json").write_text(json.dumps(data, indent=2), encoding="utf-8")

    def write_zip(self, entries):
        with zipfile.ZipFile(self.zip_path, "w") as z:
            for name, content in entries.items():
                z.writestr(name, content)

    def write_tar(self, entries):
        with tarfile.open(self.tar_path, "w:gz") as t:
            for name, content in entries.items():
                ti = tarfile.TarInfo(name)
                ti.size = len(content)
                t.addfile(ti, io.BytesIO(content))

    def test_positive_offline_verification(self):
        result = verifier.verify(self.root, archives=self.archives_dir)
        self.assertEqual(result["status"], "windows-build-inputs-verified")
        self.assertEqual(result["package_configurations_checked"], 3)
        self.assertEqual(result["toolchain_members_checked"], 3)
        self.assertEqual(result["glad_helper_checked"], 1)
        self.assertEqual(result["version_evidence_checked"], 1)
        self.assertEqual(result["archive_members_verified"], 8)
        self.assertEqual(result["archive_associations_unchecked"], 0)
        self.assertFalse(result["binary_source_correspondence_verified"])

    def test_changed_toolchain_hash_is_rejected(self):
        bad_manifest = dict(self.manifest)
        bad_manifest["toolchain_inputs"]["recipes"][0]["configuration_sha256"] = "0" * 64
        self.write_manifest(bad_manifest)
        with self.assertRaisesRegex(ValueError, "Member hash mismatch"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_missing_toolchain_member_in_archive_is_rejected(self):
        # Rewrite zip omitting toolchain member
        self.write_zip({
            self.prefix + "packages/amf-headers.cmake": self.pkg_content,
            self.prefix + "packages/glad.cmake": self.pkg_content,
            self.prefix + "packages/libplacebo.cmake": self.pkg_content,
            self.prefix + "cmake/custom_steps.cmake": self.helper_content,
            self.prefix + ".github/workflows/mpv_clang.yml": self.wf_content,
            self.prefix + ".github/workflows/llvm_clang.yml": self.wf_content,
        })
        new_manifest = dict(self.manifest)
        new_manifest["build_archive_sha256"] = hashlib.sha256(self.zip_path.read_bytes()).hexdigest()
        self.write_manifest(new_manifest)
        with self.assertRaisesRegex(ValueError, "Missing member in build archive"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_changed_version_evidence_hash_is_rejected(self):
        bad_manifest = dict(self.manifest)
        bad_manifest["packages"][2]["binary_reported_source"]["version_source_evidence"][0]["sha256"] = "1" * 64
        self.write_manifest(bad_manifest)
        with self.assertRaisesRegex(ValueError, "Version evidence hash mismatch"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_missing_version_evidence_member_in_archive_is_rejected(self):
        # Rewrite tar without version.py
        self.write_tar({
            "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f/src/other.c": b"other"
        })
        new_manifest = dict(self.manifest)
        new_manifest["packages"][2]["binary_reported_source"]["source_archive_sha256"] = hashlib.sha256(self.tar_path.read_bytes()).hexdigest()
        self.write_manifest(new_manifest)
        with self.assertRaisesRegex(ValueError, "Missing member in libplacebo archive"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_libplacebo_archive_hash_mismatch_with_intact_version_evidence(self):
        """Negative regression: changing libplacebo archive while preserving its checked version.py
        member must fail whole-file digest verification.
        """
        # Rewrite tar with identical version.py content, plus an extra unverified file
        self.write_tar({
            "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f/src/version.py": self.version_py,
            "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f/src/unverified.txt": b"different content"
        })
        # The manifest still expects the original source_archive_sha256, but version.py is intact
        with self.assertRaisesRegex(ValueError, "Libplacebo archive sha256 mismatch"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_conflicting_duplicate_member_record_rejected(self):
        bad_manifest = dict(self.manifest)
        bad_manifest["toolchain_inputs"] = dict(bad_manifest["toolchain_inputs"])
        bad_manifest["toolchain_inputs"]["recipes"] = list(bad_manifest["toolchain_inputs"]["recipes"])
        bad_manifest["toolchain_inputs"]["recipes"].append({
            "configuration_member": "packages/amf-headers.cmake",
            "configuration_sha256": "0" * 64,
            "associated_members": []
        })
        self.write_manifest(bad_manifest)
        with self.assertRaisesRegex(ValueError, "Conflicting duplicate member record"):
            verifier.verify(self.root)

    def test_declared_oversized_zip_member_rejected(self):
        with unittest.mock.patch.object(verifier, "MAX_MEMBER_BYTES", 5):
            with self.assertRaisesRegex(ValueError, "Oversized archive member"):
                verifier.verify(self.root, archives=self.archives_dir)

    def test_conflicting_glad_helpers_cannot_be_overwritten(self):
        duplicate = copy.deepcopy(self.manifest["packages"][1])
        duplicate["build_configuration_analysis"]["build_helper_sha256"] = "0" * 64
        self.manifest["packages"].insert(0, duplicate)
        self.write_manifest(self.manifest)
        with self.assertRaisesRegex(ValueError, "Conflicting glad helper"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_conflicting_source_archive_hashes_cannot_be_overwritten(self):
        duplicate = copy.deepcopy(self.manifest["packages"][2])
        duplicate["binary_reported_source"]["source_archive_sha256"] = "0" * 64
        self.manifest["packages"].insert(0, duplicate)
        self.write_manifest(self.manifest)
        with self.assertRaisesRegex(ValueError, "Conflicting libplacebo archive"):
            verifier.verify(self.root, archives=self.archives_dir)

    def test_unsafe_member_path_traversal_is_rejected(self):
        for bad in ["../secret.cmake", "packages/../../secret.cmake", "C:escape.cmake", "toolchain\\bad.cmake"]:
            with self.subTest(bad=bad):
                bad_manifest = dict(self.manifest)
                bad_manifest["toolchain_inputs"]["recipes"][0]["configuration_member"] = bad
                self.write_manifest(bad_manifest)
                with self.assertRaisesRegex(ValueError, "Unsafe member path"):
                    verifier.verify(self.root)

    def test_invalid_digest_format_is_rejected(self):
        bad_manifest = dict(self.manifest)
        bad_manifest["toolchain_inputs"]["ci_workflow"]["workflow_sha256"] = "not-a-valid-hex"
        self.write_manifest(bad_manifest)
        with self.assertRaisesRegex(ValueError, "Invalid workflow sha256"):
            verifier.verify(self.root)

    def test_synthetic_submodule_symlink_dirty_mechanism(self):
        """Demonstrates that replacing a submodule gitlink with an external directory symlink
        causes git describe --dirty to report -dirty.
        Label: synthetic mechanism evidence, not actual CI-tree reconstruction.
        """
        git_path = shutil.which("git")
        if not git_path:
            self.skipTest("git executable not found")

        repo_root = Path(__file__).resolve().parent.parent
        git_timeout = 30

        # Sanitized environment: strip any inherited GIT_* environment variables
        clean_env = {k: v for k, v in os.environ.items() if not k.upper().startswith("GIT_")}
        clean_env["GIT_OPTIONAL_LOCKS"] = "0"

        def resolve_git_path(p_str):
            p_str = p_str.strip()
            if os.name == "nt" and p_str.startswith("/mnt/") and len(p_str) > 6 and p_str[6] == "/":
                drive = p_str[5].upper()
                return Path(f"{drive}:/{p_str[7:]}")
            p = Path(p_str)
            if not p.is_absolute():
                p = repo_root / p
            return p

        # Resolve git paths dynamically through git
        config_out = subprocess.check_output(
            [git_path, "rev-parse", "--git-path", "config"],
            cwd=repo_root, env=clean_env, timeout=git_timeout, text=True
        )
        index_out = subprocess.check_output(
            [git_path, "rev-parse", "--git-path", "index"],
            cwd=repo_root, env=clean_env, timeout=git_timeout, text=True
        )
        config_path = resolve_git_path(config_out)
        index_path = resolve_git_path(index_out)

        # 1. Record original repository state using sanitized environment and resolved git paths
        def get_repo_state():
            head = subprocess.check_output(
                [git_path, "rev-parse", "HEAD"],
                cwd=repo_root, env=clean_env, timeout=git_timeout, text=True
            ).strip()
            status = subprocess.check_output(
                [git_path, "status", "--porcelain"],
                cwd=repo_root, env=clean_env, timeout=git_timeout, text=True
            )
            config_bytes = config_path.read_bytes() if config_path.is_file() else b""
            index_bytes = index_path.read_bytes() if index_path.is_file() else b""
            return head, status, config_bytes, index_bytes

        orig_head, orig_status, orig_config, orig_index = get_repo_state()

        def run_git(args, cwd):
            cmd = [
                git_path,
                "-c", "user.name=Synthetic Test",
                "-c", "user.email=synthetic@test.invalid",
                "-c", "commit.gpgsign=false",
                "-c", "tag.gpgsign=false",
                "-c", "advice.addEmbeddedRepo=false",
                *args
            ]
            return subprocess.run(
                cmd, cwd=cwd, env=clean_env, capture_output=True, text=True, check=True, timeout=git_timeout
            )

        try:
            with tempfile.TemporaryDirectory() as temp_dir_str:
                temp_dir = Path(temp_dir_str)
                sub_dir = temp_dir / "sub"
                sub_dir.mkdir()
                main_dir = temp_dir / "main"
                main_dir.mkdir()
                ext_dir = temp_dir / "ext"
                ext_dir.mkdir()

                # Initialize synthetic submodule repository
                run_git(["init", "-q"], cwd=sub_dir)
                (sub_dir / "sub.txt").write_text("sub\n", encoding="utf-8")
                run_git(["add", "sub.txt"], cwd=sub_dir)
                run_git(["commit", "-m", "sub init", "-q"], cwd=sub_dir)

                # Initialize synthetic main repository
                run_git(["init", "-q"], cwd=main_dir)
                (main_dir / "file.txt").write_text("dummy\n", encoding="utf-8")
                run_git(["add", "file.txt"], cwd=main_dir)
                run_git(["commit", "-m", "init", "-q"], cwd=main_dir)

                # Add submodule as gitlink
                submod_path = main_dir / "3rdparty" / "submod"
                submod_path.parent.mkdir(parents=True, exist_ok=True)
                run_git(["clone", "-q", "../sub", "3rdparty/submod"], cwd=main_dir)
                run_git(["add", "3rdparty/submod"], cwd=main_dir)
                run_git(["commit", "-m", "add submodule", "-q"], cwd=main_dir)
                run_git(["tag", "-a", "v1.0.0", "-m", "Synthetic version"], cwd=main_dir)

                # Assert clean state
                clean_desc = run_git(["describe", "--dirty"], cwd=main_dir).stdout.strip()
                self.assertEqual(clean_desc, "v1.0.0")

                # Replace submodule with directory symlink
                (ext_dir / "ext.txt").write_text("external content\n", encoding="utf-8")
                def handle_remove_readonly(func, path, exc):
                    os.chmod(path, stat.S_IWRITE)
                    func(path)
                shutil.rmtree(submod_path, onexc=handle_remove_readonly)
                try:
                    os.symlink(str(ext_dir), str(submod_path), target_is_directory=True)
                except OSError as e:
                    # Record unsupported symlink capability honestly, not as a pass
                    self.skipTest(f"Directory symlink capability unsupported or unprivileged on host: {e}")

                # Assert dirty state and git submodule warning
                dirty_desc = run_git(["describe", "--dirty"], cwd=main_dir).stdout.strip()
                self.assertEqual(dirty_desc, "v1.0.0-dirty")

                status_res = subprocess.run(
                    [git_path, "-c", "user.name=Synthetic Test", "-c", "user.email=synthetic@test.invalid", "status", "--short"],
                    cwd=main_dir, env=clean_env, capture_output=True, text=True, timeout=git_timeout
                )
                status_combined = status_res.stdout + status_res.stderr
                self.assertIn("expected submodule path '3rdparty/submod' not to be a symbolic link", status_combined)
        finally:
            # Prove original repository HEAD, status, config, and index remain strictly unchanged
            current_head, current_status, current_config, current_index = get_repo_state()
            self.assertEqual(current_head, orig_head, "Original repository HEAD was modified by test!")
            self.assertEqual(current_status, orig_status, "Original repository working tree status was modified by test!")
            self.assertEqual(current_config, orig_config, "Original repository config was modified by test!")
            self.assertEqual(current_index, orig_index, "Original repository index was modified by test!")


if __name__ == "__main__":
    unittest.main()
