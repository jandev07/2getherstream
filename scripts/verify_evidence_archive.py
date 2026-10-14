"""Verify a saved native-proof archive without displaying its private contents.

Checksums detect accidental changes against the saved manifest. They do not
authenticate the manifest, prove which binary ran, or satisfy a playback gate.
"""
import argparse
import hashlib
import json
import re
import os
import stat
from pathlib import Path, PureWindowsPath

MAX_ENTRIES = 10000


def checked_stat(path):
    info = path.lstat()
    if stat.S_ISLNK(info.st_mode) or getattr(info, "st_file_attributes", 0) & 0x400:
        raise ValueError("redirected_archive_path")
    return info


def inventory(root):
    pending, files, count = [root], {}, 0
    while pending:
        directory = pending.pop()
        checked_stat(directory)
        with os.scandir(directory) as entries:
            for entry in entries:
                count += 1
                if count > MAX_ENTRIES:
                    raise ValueError("archive_inventory_too_large")
                path = Path(entry.path)
                info = checked_stat(path)
                if stat.S_ISDIR(info.st_mode):
                    pending.append(path)
                elif stat.S_ISREG(info.st_mode):
                    key = path.relative_to(root).as_posix().casefold()
                    if key in files:
                        raise ValueError("archive_name_collision")
                    files[key] = path
                else:
                    raise ValueError("invalid_archive_file")
    return files


def verify(root):
    # Inspect namespace components before resolution or directory traversal.
    # These checks do not claim protection from concurrent privileged swaps.
    root = Path(os.path.abspath(root))
    for parent in (root, *root.parents):
        if not stat.S_ISDIR(checked_stat(parent).st_mode):
            raise ValueError("invalid_archive_root")
    manifest = root / "evidence-manifest.json"
    info = checked_stat(manifest)
    if not stat.S_ISREG(info.st_mode) or info.st_size > 4 * 1024 * 1024:
        raise ValueError("invalid_manifest")
    rows = json.loads(manifest.read_text(encoding="utf-8-sig"))
    if not isinstance(rows, list) or not 1 <= len(rows) <= MAX_ENTRIES:
        raise ValueError("invalid_manifest")
    actual = inventory(root)
    actual.pop("evidence-manifest.json")
    expected = set()
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("invalid_entry")
        name, digest = row.get("file"), row.get("Hash")
        if not isinstance(name, str) or not name or not isinstance(digest, str) or not re.fullmatch(r"[0-9a-fA-F]{64}", digest):
            raise ValueError("invalid_entry")
        path = PureWindowsPath(name)
        if path.drive or path.root or any(not part or part in (".", "..") or ":" in part or part.endswith((" ", ".")) for part in name.replace("\\", "/").split("/")):
            raise ValueError("invalid_entry_path")
        key = "/".join(path.parts).casefold()
        if key in expected or key == "evidence-manifest.json":
            raise ValueError("duplicate_or_self_entry")
        expected.add(key)
        target = actual.get(key)
        if target is None or not stat.S_ISREG(checked_stat(target).st_mode):
            raise ValueError("invalid_entry_path")
        checksum = hashlib.sha256()
        with target.open("rb") as source:
            for chunk in iter(lambda: source.read(1024 * 1024), b""):
                checksum.update(chunk)
        if checksum.hexdigest() != digest.lower():
            raise ValueError("checksum_mismatch")
    if set(actual) != expected:
        raise ValueError("archive_inventory_mismatch")
    return {"verified_files": len(expected), "manifest_authenticated": False,
            "physical_release_gate_satisfied": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("archive", type=Path)
    args = parser.parse_args()
    try:
        report = verify(args.archive)
    except (ValueError, OSError, TypeError, RuntimeError):
        parser.exit(1, "evidence_archive_verification_failed\n")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
