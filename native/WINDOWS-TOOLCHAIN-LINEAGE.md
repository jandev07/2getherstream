# Windows Toolchain Cache Lineage and Source Correspondence

## Summary

The Windows Alpha candidate (`mpv-dev-x86_64-20261007-git-eb0ee10315.7z`) was compiled
in GitHub Actions workflow run `37549585368` (commit `b2856ee6deed6957380b32465f359e67f67fafd8`).
The x86_64 build job did not compile the LLVM/Clang compiler or MinGW sysroot from
source; instead, it restored pre-compiled binaries from the GitHub Actions cache key
`***-clang_root-37391591956`.

Inspection of authenticated run logs across two upstream lineage links demonstrates:
1. **Link 1 (Relay / Consumer)**: Workflow run `37391591956` (workflow `mpv clang`, created
   2026-10-06T00:00:02Z, head `05a60b3cfd04e3e3b89918f4a27f3dde2935dff2`, event
   `workflow_dispatch`). Log line 3650 restored `clang_root-37245752254` and line 4481
   saved `clang_root-37391591956`.
2. **Link 2 (Relay / Consumer)**: Workflow run `37245752254` (workflow `mpv clang`, created
   2026-10-05T00:00:03Z, head `05a60b3cfd04e3e3b89918f4a27f3dde2935dff2`, event
   `workflow_dispatch`). Log line 237 restored `clang_root-37163528702` and line 945
   saved `clang_root-37245752254`.

Both inspected `mpv clang` jobs restored an earlier cache and re-saved it. These
restore/save records identify relay runs; they do not identify the original compiler producer. Tracing halted at the bounded two-link
limit without following run `37163528702`.

### Upstream Toolchain Producer Candidate

The dedicated compiler workflow in `shinchiro/mpv-winbuild-cmake` is `.github/workflows/llvm_clang.yml`.
An inspected successful run was `31022056534` (2026-08-05, head `5bdf2f86d33589ac89649209813e2f2104e5119f`).
That run is an upstream **producer candidate**, but direct linkage to the October 2026
cache cascade was not established within the inspected two-link chain. Latest successful
toolchain workflow execution alone does not prove that its binaries reached this DLL
or remained unchanged across intervening daily cache updates.

### Recipe Version Comparison Boundary

Comparing build recipes between the candidate producer commit and the Alpha build commit:
- **Candidate producer commit `5bdf2f8` (2026-08-05)**: `toolchain/llvm/llvm.cmake` specifies
  `set(clang_version "22")` and `GIT_TAG release/22.x`.
- **Alpha build commit `b2856ee` (2026-10-07)**: `toolchain/llvm/llvm.cmake` specifies
  `set(clang_version "23")` and `GIT_TAG release/23.x`.

This is a **recipe comparison only** between two points in the repository history. Without
direct linkage to the toolchain producer run or binary compiler introspection, neither
the actual compiler version used to produce the cached binaries nor the stability of
cache contents across daily updates is established. Minor variations in compressed cache
sizes between restore and save steps (e.g. 468,048,297 B restored vs 468,049,155 B saved
in run 37549585368) cannot determine which files were added, modified, or removed.

### Metadata and Log Access

- Authenticated log retrieval using the existing Windows GitHub CLI (`gh run view --log`)
  succeeded for both October runs without credential disclosure or token extraction.
- Unauthenticated API requests returning `HTTP 403 Forbidden` reflect endpoint permissions,
  and request context; they do not establish that authorized log retrieval is unavailable.
- Exact compiler and runtime source revisions, toolchain reproduction, and distribution
  completeness remain unresolved.

---

## 1. Consumer Run Evidence (`run 37549585368`)

From `work/alpha-source-material/windows-native-build.log`:
- **Repository**: `shinchiro/mpv-winbuild-cmake`
- **Workflow**: `mpv clang` (`.github/workflows/mpv_clang.yml`)
- **Checkout Commit**: `b2856ee6deed6957380b32465f359e67f67fafd8` (2026-10-07T00:00:40Z)
- **Job**: `Building mpv (x86_64)` (Job ID: `112521997092`)

### Cache Records
- **Line 3666**: `Cache hit for restore-key: ***-clang_root-37391591956`
- **Line 3671**: `Cache Size: ~446 MB (468048297 B)`
- **Line 3674**: `Cache restored from key: ***-clang_root-37391591956`
- **Line 4515**: `Sent 468049155 of 468049155 (100.0%), 222.7 MBs/sec`
- **Line 4517**: `Cache saved with key: ***-clang_root-37549585368`

---

## 2. Lineage Link 1: Run `37391591956` (Intermediate Relay)

- **Repository**: `shinchiro/mpv-winbuild-cmake`
- **Workflow**: `mpv clang` (`.github/workflows/mpv_clang.yml`, ID: `70358048`)
- **Run ID**: `37391591956`
- **Triggered**: `2026-10-06T00:00:02Z`
- **Completed**: `2026-10-06T00:49:35Z`
- **Event**: `workflow_dispatch`
- **Head Branch**: `master`
- **Head SHA**: `05a60b3cfd04e3e3b89918f4a27f3dde2935dff2`
- **Log SHA256**: `31c19bf4dde5a0c17ae4ee9206347f2966260cf43c1312f9976007e5b0b72eaa` (625,999 bytes)

### x86_64 Cache Actions in Run `37391591956`
- **Line 3650**: `Cache restored from key: ***-clang_root-37245752254` (restored 468,049,040 bytes)
- **Line 4481**: `Cache saved with key: ***-clang_root-37391591956` (saved 468,048,297 bytes)

**Observed Role**: Restored previous day's toolchain cache and re-saved it at job end.
The inspected cache records do not establish the original toolchain producer.

---

## 3. Lineage Link 2: Run `37245752254` (Intermediate Relay)

- **Repository**: `shinchiro/mpv-winbuild-cmake`
- **Workflow**: `mpv clang` (`.github/workflows/mpv_clang.yml`, ID: `70358048`)
- **Run ID**: `37245752254`
- **Triggered**: `2026-10-05T00:00:03Z`
- **Completed**: `2026-10-05T00:37:22Z`
- **Event**: `workflow_dispatch`
- **Head Branch**: `master`
- **Head SHA**: `05a60b3cfd04e3e3b89918f4a27f3dde2935dff2`
- **Log SHA256**: `5e004127eabd4346824bc1c623a98a35befdd089d0781f6a9d335f2a3ab2027c` (571,510 bytes)

### x86_64 Cache Actions in Run `37245752254`
- **Line 237**: `Cache restored from key: ***-clang_root-37163528702` (restored 468,049,964 bytes)
- **Line 945**: `Cache saved with key: ***-clang_root-37245752254` (saved 468,049,040 bytes)

**Observed Role**: Restored earlier cache key ending in `37163528702` and re-saved it.
The inspected cache records do not establish the original toolchain producer.

---

## 4. Lineage Boundary and Gaps

1. **Two-Link Investigation Limit**: Investigation followed two links (`37549585368` -> `37391591956` -> `37245752254`).
   The referenced prior key `clang_root-37163528702` was not followed under this bounded scope.
2. **Producer Run Unverified**: Upstream workflow run `31022056534` (2026-08-05, `llvm_clang.yml`)
   remains an unverified producer candidate. The full chain of cache hits connecting it to
   `37163528702` has not been inspected.
3. **Floating Recipe vs Binary State**:
   - `llvm-project.git`: `release/23.x` in build commit `b2856ee`; `release/22.x` in August candidate commit `5bdf2f8`.
   - `mingw-w64.git`: `GIT_TAG` omitted in recipe, defaulting to `master` in CMake ExternalProject.
   - `rustup`: `nightly` installer ref.
   Because both inspected runs and the candidate producer used floating branch refs, and
   intermediate cache modifications cannot be ruled out, the exact source commits that
   generated the compiler binaries remain an open gap.
4. **Cache Size Variations**: Variations in compressed cache archives (e.g. 468,049,964 ->
   468,049,040 -> 468,048,297 -> 468,049,155 bytes) do not establish their cause
   or individual member differences. No cache archive was downloaded or unpacked
   for this investigation.

The candidate recipe was independently read at the corrected full commit above.
Its SHA256 is `a1168b87c9ef663c40ce08f666219e3f668efb328429e4cea8704741b8cb31db`.
The recorded recipe is available at [the pinned upstream source](https://github.com/shinchiro/mpv-winbuild-cmake/blob/5bdf2f86d33589ac89649209813e2f2104e5119f/toolchain/llvm/llvm.cmake).
This additional small source read is separate from the two inspected cache links.
