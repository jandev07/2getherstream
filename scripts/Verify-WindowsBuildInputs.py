"""Verify Windows build inputs and toolchain recipe correspondence against retained archives.

Offline integrity check against native/WINDOWS-BUILD-INPUTS.json, not a complete binary
rebuild, compiler correspondence or distribution completeness certification.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile
import zipfile

MAX_MEMBER_BYTES = 64 * 1024 * 1024
BUILD_ARCHIVE_NAME = "windows-build-b2856ee.zip"
LIBPLACEBO_ARCHIVE_NAME = "libplacebo-0d043c7f6f79cd3687c023454bdacbe615e4d96f.tar.gz"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def redirected(path):
    return path.is_symlink() or bool(getattr(path.stat(), "st_file_attributes", 0) & 1024)


def safe_member_path(name):
    require(isinstance(name, str) and name, "Invalid member path")
    posix = PurePosixPath(name)
    require(not posix.is_absolute() and ".." not in posix.parts
            and "\\" not in name and ":" not in name, "Unsafe member path: " + name)
    return str(posix)


def find_archive(directory, name):
    directory = Path(directory).absolute()
    require(directory.is_dir() and all(not redirected(p) for p in [directory, *directory.parents]),
            "Invalid archive directory")
    matches = []
    for parent, directories, files in os.walk(directory, followlinks=False):
        directories[:] = [d for d in directories if not redirected(Path(parent) / d)]
        if name in files:
            path = Path(parent) / name
            require(not redirected(path), "Redirected archive: " + name)
            matches.append(path)
    require(len(matches) == 1, "Expected unique archive " + name + ", found " + str(len(matches)))
    return matches[0]


def verify(root, archives=None, staged=False):
    root = Path(root).absolute()
    inputs_file = root / "native/WINDOWS-BUILD-INPUTS.json"
    require(inputs_file.is_file() and not redirected(inputs_file), "Missing WINDOWS-BUILD-INPUTS.json")
    data = inputs_file.read_bytes()
    manifest = json.loads(data.decode("utf-8"))

    if staged:
        result = subprocess.run(["git", "show", ":native/WINDOWS-BUILD-INPUTS.json"], cwd=root, capture_output=True)
        require(result.returncode == 0 and (result.stdout == data or result.stdout == data.replace(b"\r\n", b"\n")),
                "Staged WINDOWS-BUILD-INPUTS.json differs")

    build_archive_sha = manifest.get("build_archive_sha256")
    require(isinstance(build_archive_sha, str) and re.fullmatch("[a-f0-9]{64}", build_archive_sha),
            "Invalid build_archive_sha256")

    # Collect package configuration members
    packages = manifest.get("packages", [])
    require(isinstance(packages, list) and packages, "Empty packages list")
    package_members = {}
    glad_helper = None
    libplacebo_evidence = {}
    libplacebo_archive_sha = None

    def record_member(record_dict, member_path, member_sha, source_desc):
        if member_path in record_dict:
            require(record_dict[member_path] == member_sha,
                    f"Conflicting duplicate member record for {member_path} in {source_desc}: {record_dict[member_path]} != {member_sha}")
        record_dict[member_path] = member_sha

    for p in packages:
        name = safe_member_path(p["configuration_member"])
        sha = p["configuration_sha256"]
        require(isinstance(sha, str) and re.fullmatch("[a-f0-9]{64}", sha), "Invalid configuration sha256 for " + name)
        record_member(package_members, name, sha, "package configurations")

        if p.get("project") == "glad" and "build_configuration_analysis" in p:
            helper = safe_member_path(p["build_configuration_analysis"]["build_helper_member"])
            helper_sha = p["build_configuration_analysis"]["build_helper_sha256"]
            require(isinstance(helper_sha, str) and re.fullmatch("[a-f0-9]{64}", helper_sha),
                    "Invalid helper sha256")
            require(glad_helper is None or glad_helper == (helper, helper_sha),
                    "Conflicting glad helper records")
            glad_helper = (helper, helper_sha)

        if p.get("project") == "libplacebo" and "binary_reported_source" in p:
            lp = p["binary_reported_source"]
            lp_sha = lp.get("source_archive_sha256")
            require(isinstance(lp_sha, str) and re.fullmatch("[a-f0-9]{64}", lp_sha),
                    "Invalid libplacebo archive sha256")
            require(libplacebo_archive_sha is None or libplacebo_archive_sha == lp_sha,
                    "Conflicting libplacebo archive records")
            libplacebo_archive_sha = lp_sha
            for ev in lp.get("version_source_evidence", []):
                ev_member = safe_member_path(ev["member"])
                ev_sha = ev["sha256"]
                require(isinstance(ev_sha, str) and re.fullmatch("[a-f0-9]{64}", ev_sha),
                        "Invalid version evidence sha256 for " + ev_member)
                record_member(libplacebo_evidence, ev_member, ev_sha, "version source evidence")

    # Collect toolchain members
    toolchain_inputs = manifest.get("toolchain_inputs", {})
    require(isinstance(toolchain_inputs, dict) and toolchain_inputs, "Missing toolchain_inputs")
    toolchain_members = {}

    ci_wf = toolchain_inputs.get("ci_workflow", {})
    for wf_key in ["workflow_member", "toolchain_workflow_member"]:
        wf_name = safe_member_path(ci_wf[wf_key])
        wf_sha = ci_wf[wf_key.replace("member", "sha256")]
        require(isinstance(wf_sha, str) and re.fullmatch("[a-f0-9]{64}", wf_sha), "Invalid workflow sha256")
        record_member(toolchain_members, wf_name, wf_sha, "ci workflow inputs")

    for recipe in toolchain_inputs.get("recipes", []):
        r_name = safe_member_path(recipe["configuration_member"])
        r_sha = recipe["configuration_sha256"]
        require(isinstance(r_sha, str) and re.fullmatch("[a-f0-9]{64}", r_sha), "Invalid recipe sha256 for " + r_name)
        record_member(toolchain_members, r_name, r_sha, "toolchain recipe inputs")
        for assoc in recipe.get("associated_members", []):
            a_name = safe_member_path(assoc["member"])
            a_sha = assoc["sha256"]
            require(isinstance(a_sha, str) and re.fullmatch("[a-f0-9]{64}", a_sha), "Invalid assoc sha256 for " + a_name)
            record_member(toolchain_members, a_name, a_sha, "toolchain associated inputs")

    all_build_members = {}
    for m, s in package_members.items():
        record_member(all_build_members, m, s, "package configurations")
    for m, s in toolchain_members.items():
        record_member(all_build_members, m, s, "toolchain inputs")

    archive_verified = 0
    if archives is not None:
        build_archive_path = find_archive(archives, BUILD_ARCHIVE_NAME)
        with open(build_archive_path, "rb") as bf:
            require(hashlib.file_digest(bf, "sha256").hexdigest() == build_archive_sha,
                    "Build archive sha256 mismatch")

        with zipfile.ZipFile(build_archive_path, "r") as z:
            prefix = ""
            for name in z.namelist():
                if name.endswith("/packages/amf-headers.cmake"):
                    prefix = name.split("packages/amf-headers.cmake")[0]
                    break
            require(prefix, "Cannot locate repository prefix in build archive")

            zip_infos = {}
            for info in z.infolist():
                if info.filename in zip_infos:
                    prev = zip_infos[info.filename]
                    require(prev.file_size == info.file_size and prev.CRC == info.CRC,
                            "Conflicting duplicate archive member in zip: " + info.filename)
                zip_infos[info.filename] = info

            for member_rel, expected_sha in all_build_members.items():
                full_name = prefix + member_rel
                require(full_name in zip_infos, "Missing member in build archive: " + member_rel)
                info = zip_infos[full_name]
                require(info.file_size <= MAX_MEMBER_BYTES, "Oversized archive member: " + member_rel)
                with z.open(info, "r") as mf:
                    data = mf.read(MAX_MEMBER_BYTES + 1)
                    require(len(data) <= MAX_MEMBER_BYTES, "Oversized archive member: " + member_rel)
                actual_sha = hashlib.sha256(data).hexdigest()
                require(actual_sha == expected_sha, "Member hash mismatch for " + member_rel)
                archive_verified += 1

            if glad_helper:
                h_name, h_expected = glad_helper
                full_h = prefix + h_name
                require(full_h in zip_infos, "Missing glad helper in build archive: " + h_name)
                h_info = zip_infos[full_h]
                require(h_info.file_size <= MAX_MEMBER_BYTES, "Oversized glad helper: " + h_name)
                with z.open(h_info, "r") as hf:
                    h_data = hf.read(MAX_MEMBER_BYTES + 1)
                    require(len(h_data) <= MAX_MEMBER_BYTES, "Oversized glad helper: " + h_name)
                require(hashlib.sha256(h_data).hexdigest() == h_expected, "Glad helper hash mismatch: " + h_name)
                archive_verified += 1

        lp_archive_path = find_archive(archives, LIBPLACEBO_ARCHIVE_NAME)
        require(libplacebo_archive_sha is not None, "Missing libplacebo archive sha256 in manifest")
        with open(lp_archive_path, "rb") as lpf:
            require(hashlib.file_digest(lpf, "sha256").hexdigest() == libplacebo_archive_sha,
                    "Libplacebo archive sha256 mismatch")

        with tarfile.open(lp_archive_path, "r:*") as tf:
            tar_members = {}
            for m in tf.getmembers():
                if m.name in tar_members:
                    prev_m = tar_members[m.name]
                    require(prev_m.size == m.size and prev_m.type == m.type,
                            "Conflicting duplicate archive member in tar: " + m.name)
                tar_members[m.name] = m

            for ev_name, ev_expected in libplacebo_evidence.items():
                require(ev_name in tar_members, "Missing member in libplacebo archive: " + ev_name)
                member_obj = tar_members[ev_name]
                require(member_obj.isfile() and member_obj.size <= MAX_MEMBER_BYTES,
                        "Invalid archive member: " + ev_name)
                tf_file = tf.extractfile(member_obj)
                require(tf_file is not None, "Cannot extract member: " + ev_name)
                with tf_file:
                    ev_data = tf_file.read(MAX_MEMBER_BYTES + 1)
                    require(len(ev_data) <= MAX_MEMBER_BYTES, "Oversized archive member: " + ev_name)
                actual_sha = hashlib.sha256(ev_data).hexdigest()
                require(actual_sha == ev_expected, "Version evidence hash mismatch: " + ev_name)
                archive_verified += 1

    return {
        "status": "windows-build-inputs-verified",
        "package_configurations_checked": len(package_members),
        "toolchain_members_checked": len(toolchain_members),
        "glad_helper_checked": 1 if glad_helper else 0,
        "version_evidence_checked": len(libplacebo_evidence),
        "archive_members_verified": archive_verified,
        "archive_associations_unchecked": 0 if archives is not None else (len(package_members) + len(toolchain_members) + (1 if glad_helper else 0) + len(libplacebo_evidence)),
        "binary_source_correspondence_verified": False,
        "toolchain_reproduction_verified": False,
        "distribution_completeness_verified": False
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archives", type=str, default=None, help="Path to directory containing source archives")
    parser.add_argument("--staged", action="store_true", help="Verify staged git index bytes match")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    result = verify(root, archives=args.archives, staged=args.staged)
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
