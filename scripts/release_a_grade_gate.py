#!/usr/bin/env python3

"""Automated A-grade release gate checker.

This script enforces three release criteria:
1. Zero unresolved critical/high known defects.
2. All benchmark latency profiles pass A-grade thresholds.
3. Stress and soak reliability readiness artifacts are fully passing.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any, Iterable, Mapping

ALLOWED_DEFECT_SEVERITIES = {"critical", "high", "medium", "low"}
BLOCKING_DEFECT_SEVERITIES = {"critical", "high"}
RESOLVED_DEFECT_STATUSES = {"closed", "resolved", "fixed", "mitigated", "done"}

LATENCY_METRICS = {
    "dispatch_events": ("dispatch_events_p50_ms_max", "dispatch_events_p95_ms_max"),
    "drain_pending_runtime_updates": (
        "drain_pending_runtime_updates_p50_ms_max",
        "drain_pending_runtime_updates_p95_ms_max",
    ),
    "render_tree": ("render_tree_p50_ms_max", "render_tree_p95_ms_max"),
}

LATENCY_METRIC_RESULT_FLAGS = {
    "dispatch_events": "dispatch_events_passed",
    "drain_pending_runtime_updates": "drain_pending_runtime_updates_passed",
    "render_tree": "render_tree_passed",
}


def resolve_path(repo_root: Path, raw_path: str) -> Path:
    path = Path(raw_path)
    return path if path.is_absolute() else repo_root / path


def load_json(path: Path, label: str) -> tuple[Any | None, list[str]]:
    errors: list[str] = []
    if not path.exists():
        errors.append(f"{label} file does not exist: {path}")
        return None, errors

    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as error:
        errors.append(f"failed to read {label} file {path}: {error}")
        return None, errors

    try:
        return json.loads(raw), errors
    except json.JSONDecodeError as error:
        errors.append(f"failed to parse {label} JSON file {path}: {error}")
        return None, errors


def get_number(value: Any, context: str, errors: list[str]) -> float | None:
    if isinstance(value, bool) or not isinstance(value, (int, float)):
        errors.append(f"{context} must be a number; got {value!r}")
        return None
    return float(value)


def check_known_defects(payload: Any, path: Path) -> tuple[list[str], str]:
    errors: list[str] = []

    if not isinstance(payload, Mapping):
        return [f"known defects file {path} must be a JSON object"], ""

    defects = payload.get("defects")
    if not isinstance(defects, list):
        return [f"known defects file {path} must contain a `defects` array"], ""

    blocking_defects: list[str] = []
    for index, defect in enumerate(defects):
        if not isinstance(defect, Mapping):
            errors.append(f"defects[{index}] must be an object")
            continue

        defect_id = str(defect.get("id", f"defect-{index}"))
        severity = str(defect.get("severity", "")).strip().lower()
        status = str(defect.get("status", "open")).strip().lower() or "open"

        if severity not in ALLOWED_DEFECT_SEVERITIES:
            errors.append(
                f"defect {defect_id} has invalid severity `{severity}` (expected one of {sorted(ALLOWED_DEFECT_SEVERITIES)})"
            )
            continue

        if severity in BLOCKING_DEFECT_SEVERITIES and status not in RESOLVED_DEFECT_STATUSES:
            blocking_defects.append(f"{defect_id}({severity}/{status})")

    if blocking_defects:
        errors.append(
            "blocking known defects found (critical/high unresolved): "
            + ", ".join(blocking_defects)
        )

    summary = (
        f"{len(defects)} tracked defects, {len(blocking_defects)} blocking critical/high defects"
    )
    return errors, summary


def check_latency_thresholds(payload: Any, path: Path) -> tuple[list[str], str]:
    errors: list[str] = []

    if not isinstance(payload, Mapping):
        return [f"benchmark artifact {path} must be a JSON object"], ""

    profiles = payload.get("profiles")
    if not isinstance(profiles, list) or not profiles:
        return [f"benchmark artifact {path} must contain a non-empty `profiles` array"], ""

    failing_profiles: list[str] = []

    for index, profile in enumerate(profiles):
        if not isinstance(profile, Mapping):
            errors.append(f"profiles[{index}] must be an object")
            continue

        profile_id = str(profile.get("id", f"profile-{index}"))
        profile_errors: list[str] = []

        a_grade = profile.get("a_grade")
        if not isinstance(a_grade, Mapping):
            errors.append(f"profile `{profile_id}` is missing object field `a_grade`")
            continue

        thresholds = a_grade.get("thresholds")
        if not isinstance(thresholds, Mapping):
            errors.append(f"profile `{profile_id}` is missing object field `a_grade.thresholds`")
            continue

        derived_metric_pass_results: dict[str, bool] = {}

        for metric_name, (p50_threshold_key, p95_threshold_key) in LATENCY_METRICS.items():
            stats = profile.get(metric_name)
            if not isinstance(stats, Mapping):
                profile_errors.append(
                    f"profile `{profile_id}` missing object metric `{metric_name}`"
                )
                continue

            p50 = get_number(stats.get("p50_ms"), f"{profile_id}.{metric_name}.p50_ms", profile_errors)
            p95 = get_number(stats.get("p95_ms"), f"{profile_id}.{metric_name}.p95_ms", profile_errors)
            p50_threshold = get_number(
                thresholds.get(p50_threshold_key),
                f"{profile_id}.a_grade.thresholds.{p50_threshold_key}",
                profile_errors,
            )
            p95_threshold = get_number(
                thresholds.get(p95_threshold_key),
                f"{profile_id}.a_grade.thresholds.{p95_threshold_key}",
                profile_errors,
            )

            if None in (p50, p95, p50_threshold, p95_threshold):
                continue

            metric_passed = p50 <= p50_threshold and p95 <= p95_threshold
            derived_metric_pass_results[metric_name] = metric_passed

            result_field = LATENCY_METRIC_RESULT_FLAGS[metric_name]
            declared_metric_passed = a_grade.get(result_field)
            if not isinstance(declared_metric_passed, bool):
                profile_errors.append(
                    f"profile `{profile_id}` missing boolean field `a_grade.{result_field}`"
                )
            elif declared_metric_passed != metric_passed:
                profile_errors.append(
                    f"profile `{profile_id}` has inconsistent `a_grade.{result_field}`: declared={declared_metric_passed}, derived={metric_passed}"
                )

            if not metric_passed:
                profile_errors.append(
                    f"profile `{profile_id}` metric `{metric_name}` exceeded thresholds (p50={p50:.3f}ms max={p50_threshold:.3f}ms, p95={p95:.3f}ms max={p95_threshold:.3f}ms)"
                )

        derived_profile_passed = (
            len(derived_metric_pass_results) == len(LATENCY_METRICS)
            and all(derived_metric_pass_results.values())
        )

        declared_profile_passed = a_grade.get("passed")
        if not isinstance(declared_profile_passed, bool):
            profile_errors.append(f"profile `{profile_id}` missing boolean field `a_grade.passed`")
        elif declared_profile_passed != derived_profile_passed:
            profile_errors.append(
                f"profile `{profile_id}` has inconsistent `a_grade.passed`: declared={declared_profile_passed}, derived={derived_profile_passed}"
            )

        if profile_errors:
            errors.extend(profile_errors)
            failing_profiles.append(profile_id)

    if failing_profiles:
        errors.append(
            "latency A-grade thresholds failed for profile(s): "
            + ", ".join(dict.fromkeys(failing_profiles))
        )

    summary = f"{len(profiles)} benchmark profiles evaluated"
    return errors, summary


def check_reliability_artifact(
    payload: Any, path: Path, expected_scenario: str
) -> tuple[list[str], str]:
    errors: list[str] = []

    if not isinstance(payload, Mapping):
        return [f"readiness artifact {path} must be a JSON object"], ""

    scenario = payload.get("scenario")
    if scenario != expected_scenario:
        errors.append(
            f"readiness artifact {path} scenario mismatch: expected `{expected_scenario}`, got `{scenario}`"
        )

    gates = payload.get("gates")
    if not isinstance(gates, Mapping):
        return errors + [f"readiness artifact {path} must contain object field `gates`"], ""
    if not gates:
        errors.append(f"readiness artifact {path} has no gates")

    failed_gates: list[str] = []
    for gate_name, gate_result in gates.items():
        if not isinstance(gate_result, bool):
            errors.append(
                f"readiness artifact {path} gate `{gate_name}` must be boolean, got {gate_result!r}"
            )
        elif not gate_result:
            failed_gates.append(str(gate_name))

    derived_passed = not failed_gates
    declared_passed = payload.get("passed")
    if not isinstance(declared_passed, bool):
        errors.append(f"readiness artifact {path} must contain boolean field `passed`")
    elif declared_passed != derived_passed:
        errors.append(
            f"readiness artifact {path} inconsistent `passed`: declared={declared_passed}, derived={derived_passed}"
        )

    if failed_gates:
        errors.append(
            f"readiness artifact {path} failed gate(s): {', '.join(failed_gates)}"
        )

    summary = f"scenario={expected_scenario}, gates={len(gates)}, failed={len(failed_gates)}"
    return errors, summary


def print_section_result(title: str, summary: str, errors: Iterable[str]) -> bool:
    error_list = list(errors)
    if error_list:
        print(f"[FAIL] {title}")
        for error in error_list:
            print(f"  - {error}")
        return False

    print(f"[PASS] {title}: {summary}")
    return True


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Check A-grade release gates (defects, latency, reliability)."
    )
    parser.add_argument(
        "--known-defects",
        default="docs/known-defects.json",
        help="path to known defects JSON register (default: docs/known-defects.json)",
    )
    parser.add_argument(
        "--benchmark-artifact",
        default="reference/benchmarks/runtime-jsx-latest.json",
        help="path to runtime benchmark artifact JSON (default: reference/benchmarks/runtime-jsx-latest.json)",
    )
    parser.add_argument(
        "--stress-artifact",
        default="target/jsx-runtime-readiness/stress.json",
        help="path to stress readiness artifact JSON (default: target/jsx-runtime-readiness/stress.json)",
    )
    parser.add_argument(
        "--soak-artifact",
        default="target/jsx-runtime-readiness/soak.json",
        help="path to soak readiness artifact JSON (default: target/jsx-runtime-readiness/soak.json)",
    )
    return parser.parse_args(argv)


def main(argv: list[str]) -> int:
    args = parse_args(argv)
    repo_root = Path(__file__).resolve().parents[1]

    known_defects_path = resolve_path(repo_root, args.known_defects)
    benchmark_artifact_path = resolve_path(repo_root, args.benchmark_artifact)
    stress_artifact_path = resolve_path(repo_root, args.stress_artifact)
    soak_artifact_path = resolve_path(repo_root, args.soak_artifact)

    overall_passed = True

    known_defects_payload, load_errors = load_json(known_defects_path, "known defects")
    known_defects_errors = list(load_errors)
    known_defects_summary = ""
    if known_defects_payload is not None:
        errors, known_defects_summary = check_known_defects(
            known_defects_payload, known_defects_path
        )
        known_defects_errors.extend(errors)
    overall_passed &= print_section_result(
        "Known-defects gate",
        known_defects_summary,
        known_defects_errors,
    )

    benchmark_payload, load_errors = load_json(benchmark_artifact_path, "benchmark artifact")
    benchmark_errors = list(load_errors)
    benchmark_summary = ""
    if benchmark_payload is not None:
        errors, benchmark_summary = check_latency_thresholds(
            benchmark_payload, benchmark_artifact_path
        )
        benchmark_errors.extend(errors)
    overall_passed &= print_section_result(
        "Latency gate",
        benchmark_summary,
        benchmark_errors,
    )

    stress_payload, load_errors = load_json(stress_artifact_path, "stress readiness artifact")
    stress_errors = list(load_errors)
    stress_summary = ""
    if stress_payload is not None:
        errors, stress_summary = check_reliability_artifact(
            stress_payload, stress_artifact_path, expected_scenario="stress"
        )
        stress_errors.extend(errors)
    overall_passed &= print_section_result(
        "Reliability gate (stress)",
        stress_summary,
        stress_errors,
    )

    soak_payload, load_errors = load_json(soak_artifact_path, "soak readiness artifact")
    soak_errors = list(load_errors)
    soak_summary = ""
    if soak_payload is not None:
        errors, soak_summary = check_reliability_artifact(
            soak_payload, soak_artifact_path, expected_scenario="soak"
        )
        soak_errors.extend(errors)
    overall_passed &= print_section_result(
        "Reliability gate (soak)",
        soak_summary,
        soak_errors,
    )

    print()
    if overall_passed:
        print("A-grade release gate: PASS")
        return 0

    print("A-grade release gate: FAIL")
    return 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
