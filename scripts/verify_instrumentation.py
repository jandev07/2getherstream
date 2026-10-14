"""Validate `am instrument -r -w` output, including assumption failures.

Independent parser of AndroidX InstrumentationResultPrinter status semantics:
https://github.com/android/android-test/blob/main/runner/android_junit_runner/java/androidx/test/internal/runner/listener/InstrumentationResultPrinter.java
No upstream implementation copied. Used by the isolated session proof runner.
"""
import argparse
import json
from pathlib import Path


def verify(lines, expected_tests, expected_identities=None):
    if type(expected_tests) is not int or not 1 <= expected_tests <= 100:
        raise ValueError("invalid_expected_count")
    selected = None
    if expected_identities is not None:
        if not isinstance(expected_identities, list) or len(expected_identities) != expected_tests:
            raise ValueError("invalid_expected_identities")
        selected = set()
        for identity in expected_identities:
            if (not isinstance(identity, list) or len(identity) != 2 or
                    not all(isinstance(value, str) and 0 < len(value) <= 512 for value in identity)):
                raise ValueError("invalid_expected_identity")
            selected.add(tuple(identity))
        if len(selected) != expected_tests:
            raise ValueError("duplicate_expected_identity")
    fields = {}
    active = None
    completed = set()
    final = None
    for raw in lines:
        line = raw.rstrip("\r\n")
        if len(line) > 65536:
            raise ValueError("invalid_status_line")
        if final is not None and line.startswith("INSTRUMENTATION_"):
            raise ValueError("status_after_final_result")
        if line.startswith(("INSTRUMENTATION_FAILED:", "INSTRUMENTATION_ABORTED:")) or line.startswith("INSTRUMENTATION_RESULT: shortMsg="):
            raise ValueError("instrumentation_failed")
        if line.startswith("INSTRUMENTATION_STATUS: "):
            key, separator, value = line[len("INSTRUMENTATION_STATUS: "):].partition("=")
            if not separator:
                raise ValueError("invalid_status_field")
            # Only retain protocol fields; stream/stack may contain private data.
            if key in ("class", "test", "numtests", "current", "id"):
                if key in fields:
                    raise ValueError("duplicate_status_field")
                fields[key] = value
        elif line.startswith("INSTRUMENTATION_STATUS_CODE: "):
            code = int(line[len("INSTRUMENTATION_STATUS_CODE: "):])
            if code not in (0, 1):
                # Includes -3 ignored and -4 assumption failures, even when the
                # human-readable summary says OK with the expected test count.
                raise ValueError("test_not_executed_successfully")
            if fields.get("id") != "AndroidJUnitRunner" or int(fields.get("numtests", "-1")) != expected_tests:
                raise ValueError("invalid_status_identity_or_count")
            identity = fields.get("class"), fields.get("test")
            if not all(identity) or int(fields.get("current", "-1")) != len(completed) + 1:
                raise ValueError("invalid_test_identity_or_sequence")
            if selected is not None and identity not in selected:
                raise ValueError("unexpected_test_identity")
            if code == 1:
                if active is not None or identity in completed:
                    raise ValueError("duplicate_test_start")
                active = identity
            else:
                if active != identity:
                    raise ValueError("test_finish_without_matching_start")
                completed.add(identity)
                active = None
            fields = {}
        elif line.startswith("INSTRUMENTATION_CODE: "):
            final = int(line[len("INSTRUMENTATION_CODE: "):])
    if final != -1 or fields or active is not None or len(completed) != expected_tests:
        raise ValueError("incomplete_instrumentation")
    return {"completed_tests": len(completed), "all_selected_tests_executed": True,
            "expected_test_identities_verified": selected is not None,
            "physical_release_gate_satisfied": False}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("result", type=Path)
    parser.add_argument("--expected-tests", type=int, required=True)
    parser.add_argument("--expected-identities", type=Path)
    args = parser.parse_args()
    try:
        identities = None
        if args.expected_identities is not None:
            if args.expected_identities.stat().st_size > 65536:
                raise ValueError("expected_identities_too_large")
            identities = json.loads(args.expected_identities.read_text(encoding="utf-8-sig"))
        with args.result.open(encoding="utf-8-sig") as source:
            report = verify(source, args.expected_tests, identities)
    except (ValueError, OSError, TypeError):
        parser.exit(1, "instrumentation_status_verification_failed\n")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
