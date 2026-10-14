"""Stage validated public metadata only; APK bytes stay on GitHub Releases."""
from pathlib import Path
from contextlib import contextmanager
import argparse
import hashlib
import json
import os
import re
import unicodedata

ORIGIN = "https://2gether-download.vgovind28.workers.dev"
REPOSITORY = "https://github.com/jandev07/2getherstream"
MAX_BYTES = 536870912


def require(condition, message="Invalid APK release input"):
    if not condition:
        raise ValueError(message)


def plain_path(path):
    for item in [path, *path.parents]:
        if item.exists():
            attributes = getattr(item.stat(), "st_file_attributes", 0)
            require(not item.is_symlink() and not attributes & 1024, "Refuse redirected release path")


@contextmanager
def publication_lock(destination):
    # OS lock, released on close/process exit; no stale sentinel and no network.
    destination.mkdir(parents=True, exist_ok=True)
    path = destination / ".publication.lock"
    plain_path(path)
    with path.open("a+b") as owner:
        if owner.tell() == 0:
            owner.write(b"0")
            owner.flush()
        owner.seek(0)
        if os.name == "nt":
            import msvcrt
            msvcrt.locking(owner.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl
            fcntl.flock(owner, fcntl.LOCK_EX | fcntl.LOCK_NB)
        try:
            yield
        finally:
            owner.seek(0)
            if os.name == "nt":
                msvcrt.locking(owner.fileno(), msvcrt.LK_UNLCK, 1)
            else:
                fcntl.flock(owner, fcntl.LOCK_UN)


def atomic_text(path, text):
    plain_path(path)
    temporary = path.with_name(path.name + ".pending")
    plain_path(temporary)
    temporary.write_text(text, encoding="utf-8")
    os.replace(temporary, path)


def stage_release(apk, release, destination):
    """Validate exact local bytes, preserve version identity, switch one manifest.

    Does not upload an APK, verify a remote asset, or deploy the Worker. The final
    publisher must verify the public asset before deploying this metadata.
    """
    apk, destination = Path(apk), Path(destination)
    require(set(release) == {"protocol", "package", "version_code", "version_name", "apk_url", "sha256", "size_bytes"})
    code = release["version_code"]
    require(type(code) is int and 1 <= code <= 2147483647)
    require(type(release["protocol"]) is int and release["protocol"] == 1 and release["package"] == "app.together.v2")
    require(release["apk_url"] == f"{REPOSITORY}/releases/download/android-v{code}/2gether.apk")
    name = release["version_name"]
    require(isinstance(name, str) and 1 <= len(name) <= 64 and not any(unicodedata.category(c) == "Cc" for c in name))
    require(type(release["size_bytes"]) is int and 0 < release["size_bytes"] <= MAX_BYTES)
    require(isinstance(release["sha256"], str) and re.fullmatch("[a-f0-9]{64}", release["sha256"]))
    plain_path(apk)
    require(apk.is_file() and apk.stat().st_size == release["size_bytes"])
    digest = hashlib.sha256()
    total = 0
    with apk.open("rb") as source:
        while data := source.read(1024 * 1024):
            total += len(data)
            require(total <= release["size_bytes"])
            digest.update(data)
    require(total == release["size_bytes"] and digest.hexdigest() == release["sha256"], "APK content changed or hash mismatched")
    plain_path(destination)
    with publication_lock(destination):
        latest_path = destination / "release.json"
        plain_path(latest_path)
        if latest_path.exists():
            latest = json.loads(latest_path.read_text(encoding="utf-8"))
            require(latest["version_code"] <= code, "Refuse latest-release downgrade")
            if latest["version_code"] == code:
                require(latest == release, "Do not replace a published version identity")
        version = destination / "releases" / str(code)
        plain_path(version)
        version.mkdir(parents=True, exist_ok=True)
        manifest = version / "release.json"
        plain_path(manifest)
        encoded = json.dumps(release, indent=2) + "\n"
        if manifest.exists():
            require(json.loads(manifest.read_text(encoding="utf-8")) == release, "Existing immutable release differs")
        else:
            atomic_text(manifest, encoded)
        atomic_text(latest_path, encoded)
    return release


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("candidate_id")
    args = parser.parse_args()
    require(re.fullmatch(r"\d{8}-\d{6}-[a-f0-9]{8}", args.candidate_id))
    root = Path(__file__).resolve().parent.parent
    candidate = root / "work/private-alpha-candidates" / args.candidate_id
    build = json.loads((candidate / "candidate-build.json").read_text(encoding="utf-8-sig"))
    require(build["candidate_id"] == args.candidate_id)
    payload = next(item for item in build["files"] if item["file"] == "2gether-PrivateAlpha.apk")
    identity = build["android_identity"]
    release = {"protocol": 1, "package": identity["application_id"], "version_code": identity["version_code"], "version_name": identity["version_name"], "apk_url": f"{REPOSITORY}/releases/download/android-v{identity['version_code']}/2gether.apk", "sha256": payload["sha256"], "size_bytes": payload["bytes"]}
    stage_release(candidate / "package/2gether-PrivateAlpha.apk", release, root / "work/android-download-assets")
    print(json.dumps({"candidate_id": args.candidate_id, "size_bytes": release["size_bytes"], "sha256": release["sha256"], "deployed": False, "remote_asset_verified": False}))


if __name__ == "__main__":
    main()
