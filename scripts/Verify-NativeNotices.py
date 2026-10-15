"""Verify retained notice bytes; optionally check staged Git and local archives.

This is an offline integrity check against PROVENANCE.json, not a license audit,
upstream authenticity check, binary/source correspondence or distribution gate.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import subprocess
import tarfile

MAX_NOTICE_BYTES = 32 * 1024 * 1024


def require(condition, message):
    if not condition:
        raise ValueError(message)


def redirected(path):
    return path.is_symlink() or bool(getattr(path.stat(), "st_file_attributes", 0) & 1024)


def notice_path(root, name):
    require(isinstance(name, str), "Invalid notice path")
    relative = PurePosixPath(name)
    require(relative.parts[:2] == ("third-party", "native-components")
            and not relative.is_absolute() and ".." not in relative.parts
            and "\\" not in name and ":" not in name, "Invalid notice path")
    path = root.joinpath(*relative.parts)
    require(all(not redirected(item) for item in [path, *path.parents] if item.exists()),
            "Redirected notice path")
    require(path.is_file() and path.stat().st_size <= MAX_NOTICE_BYTES, "Missing or oversized notice")
    return path


def archive_paths(directory, wanted):
    directory = Path(directory).absolute()
    require(directory.is_dir() and all(not redirected(item) for item in [directory, *directory.parents]),
            "Invalid archive directory")
    found = {}
    for parent, directories, files in os.walk(directory, followlinks=False):
        directories[:] = [name for name in directories if not redirected(Path(parent) / name)]
        for name in set(files) & wanted:
            path = Path(parent) / name
            require(not redirected(path), "Redirected source archive")
            if name in found:
                with found[name].open("rb") as first, path.open("rb") as second:
                    require(hashlib.file_digest(first, "sha256").digest() ==
                            hashlib.file_digest(second, "sha256").digest(),
                            "Ambiguous source archive basename: " + name)
            else:
                found[name] = path
    require(set(found) == wanted, "Missing source archives: " + ", ".join(sorted(wanted - set(found))))
    return found


def verify(root, archives=None, staged=False):
    root = Path(root).absolute()
    manifest = notice_path(root, "third-party/native-components/PROVENANCE.json")
    records = json.loads(manifest.read_text(encoding="utf-8"))
    require(isinstance(records, list) and records, "Empty or invalid notice provenance")
    expected = {}
    archive_records = {}
    url_only = 0
    for record in records:
        name, digest = record["notice_file"], record["sha256"]
        require(isinstance(digest, str) and re.fullmatch("[a-f0-9]{64}", digest), "Invalid notice digest")
        require(name not in expected or expected[name] == digest, "Conflicting retained notice digest")
        expected[name] = digest
        if "source_archive" in record:
            archive, member = record["source_archive"], record["member"]
            require(isinstance(archive, str) and archive and Path(archive).name == archive
                    and not any(c in archive for c in "\\/:"), "Invalid archive basename")
            require(isinstance(member, str) and member, "Invalid archive member")
            archive_records.setdefault(archive, []).append(record)
        else:
            require(isinstance(record.get("source_url"), str) and record["source_url"].startswith("https://"),
                    "Missing source association")
            url_only += 1
    for name, digest in expected.items():
        data = notice_path(root, name).read_bytes()
        require(hashlib.sha256(data).hexdigest() == digest, "Retained notice hash mismatch: " + name)
        if staged:
            result = subprocess.run(["git", "show", ":" + name], cwd=root, capture_output=True)
            require(result.returncode == 0 and result.stdout == data, "Staged notice differs: " + name)
    checked = 0
    if archives is not None:
        paths = archive_paths(archives, set(archive_records))
        for name, associated in archive_records.items():
            with tarfile.open(paths[name], "r:*") as source:
                members = {}
                wanted = {record["member"] for record in associated}
                for member in source:
                    if member.name in wanted:
                        require(member.name not in members, "Duplicate source archive member")
                        require(member.isfile() and member.size <= MAX_NOTICE_BYTES, "Invalid source notice member")
                        members[member.name] = hashlib.sha256(source.extractfile(member).read()).hexdigest()
                for record in associated:
                    require(members.get(record["member"]) == record["sha256"],
                            "Source notice member mismatch: " + name)
                    checked += 1
    return {"status": "retained-notice-integrity-verified", "provenance_records": len(records),
            "unique_notice_files": len(expected), "staged_notice_bytes_verified": staged,
            "archive_member_associations_verified": checked,
            "archive_associations_unchecked": sum(map(len, archive_records.values())) - checked,
            "url_only_associations_unchecked": url_only,
            "upstream_authenticity_verified": False, "binary_source_correspondence_verified": False,
            "distribution_completeness_verified": False}


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archives", type=Path, help="Directory containing locally retained source archives")
    parser.add_argument("--staged", action="store_true", help="Also require retained files to match the Git index")
    args = parser.parse_args()
    print(json.dumps(verify(Path(__file__).resolve().parent.parent, args.archives, args.staged), indent=2))
