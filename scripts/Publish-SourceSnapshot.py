"""Publish a clean source tree without exporting private local Git history.

Preview by default. GitHub changes made by another contributor require review
and integration before the next publication; the publisher never force-pushes.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parent.parent
REMOTE = "https://github.com/jandev07/2getherstream.git"
RECEIPT = ROOT / "work/github-source-publication.json"
PROTECTED = {".wrangler", ".secrets", "work", "releases", "target", ".dev-cache", "node_modules"}


def git(*args, input=None):
    result = subprocess.run(["git", *args], cwd=ROOT, input=input, capture_output=True)
    if result.returncode:
        raise RuntimeError("Git source operation failed: " + args[0])
    return result.stdout


def audit(source):
    entries = git("ls-tree", "-r", "-z", source).split(b"\0")
    count = 0
    for entry in entries:
        if not entry:
            continue
        details, name = entry.split(b"\t", 1)
        mode, kind, sha = details.decode().split()
        path = Path(name.decode("utf-8"))
        if kind != "blob" or mode not in {"100644", "100755"}:
            raise RuntimeError("Unsupported source entry: " + str(path))
        if set(path.parts) & PROTECTED or path.suffix.lower() in {".apk", ".exe", ".dll", ".jks", ".keystore", ".mp4", ".pem"} or path.name in {"local.properties", "toolchains.local.json", ".env"}:
            raise RuntimeError("Protected file in source tree: " + str(path))
        data = git("cat-file", "blob", sha)
        for pattern in [rb"-----BEGIN (?:RSA |EC |OPENSSH )?PRIVATE KEY-----", rb"\b(?:gh[pousr]_[A-Za-z0-9]{30,}|github_pat_[A-Za-z0-9_]{40,})\b", rb"\bAKIA[A-Z0-9]{16}\b"]:
            matches = re.findall(pattern, data)
            if any(value != b"AKIAIOSFODNN7EXAMPLE" for value in matches):
                raise RuntimeError("Potential credential in source file: " + str(path))
        count += 1
    return count


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--publish", action="store_true")
    args = parser.parse_args()
    source = git("rev-parse", "HEAD").decode().strip()
    if git("status", "--porcelain").strip():
        raise RuntimeError("Commit and validate pending source before publication")
    if git("remote", "get-url", "github").decode().strip() != REMOTE:
        raise RuntimeError("Unexpected GitHub destination")
    count = audit(source)
    if not args.publish:
        print(json.dumps({"source": source, "files": count, "published": False, "local_history_exported": False}))
        return
    previous = json.loads(RECEIPT.read_text(encoding="utf-8"))
    git("fetch", "github", "main")
    parent = git("rev-parse", "github/main").decode().strip()
    if parent != previous["github_commit"]:
        raise RuntimeError("GitHub changed since our last publication; review contributor changes first")
    tree = git("rev-parse", source + "^{tree}").decode().strip()
    if tree == git("rev-parse", parent + "^{tree}").decode().strip():
        commit = parent
    else:
        message = git("log", "-1", "--format=%s", source).decode().strip() + "\n\nLocal source: " + source + "\n"
        commit = git("-c", "user.name=Codex", "-c", "user.email=codex@local.invalid", "commit-tree", tree, "-p", parent, input=message.encode()).decode().strip()
        git("push", "github", commit + ":refs/heads/main")
    RECEIPT.write_text(json.dumps({"local_source": source, "github_commit": commit, "files": count, "local_history_exported": False}, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"source": source, "github_commit": commit, "files": count, "published": True, "local_history_exported": False}))


if __name__ == "__main__":
    main()
