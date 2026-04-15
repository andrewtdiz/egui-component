# Runtime JSX A-Grade Release Checklist

This checklist turns A-grade release criteria into explicit, enforceable gates.

## 1) Produce fresh gate artifacts

- [ ] Run benchmark contract and regenerate latest latency artifact:

  ```bash
  CLAY_JSX_BENCH_ENFORCE_A_GRADE=1 cargo run --example runtime-jsx-benchmark
  ```

- [ ] Run reliability stress harness:

  ```bash
  CLAY_JSX_STRESS_EVENT_BATCHES=1000 \
  CLAY_JSX_STRESS_RELOAD_CYCLES=128 \
  CLAY_JSX_STRESS_MOUNT_CYCLES=128 \
  cargo test --bin runtime-jsx-host jsx_runtime_stress_harness -- --ignored --nocapture
  ```

- [ ] Run reliability soak harness:

  ```bash
  CLAY_JSX_SOAK_SAMPLE_SECS=3 \
  CLAY_JSX_SOAK_IDLE_SECS=45 \
  CLAY_JSX_SOAK_ACTIVE_SECS=45 \
  CLAY_JSX_SOAK_RELOAD_SECS=30 \
  CLAY_JSX_SOAK_IDLE_CPU_THRESHOLD=15 \
  CLAY_JSX_SOAK_RSS_BAND_KB=131072 \
  cargo test --bin runtime-jsx-host jsx_runtime_soak_harness -- --ignored --nocapture
  ```

## 2) Update known defects register

- [ ] Confirm `docs/known-defects.json` is up to date.
- [ ] Ensure there are **zero unresolved** defects with severity `critical` or `high`.

> Severity/status policy enforced by `scripts/release_a_grade_gate.py`:
> - Blocking severities: `critical`, `high`
> - Resolved statuses: `closed`, `resolved`, `fixed`, `mitigated`, `done`

## 3) Run the automated release gate

- [ ] Execute the gate script:

  ```bash
  python3 scripts/release_a_grade_gate.py
  ```

The script fails (non-zero exit code) unless all required gates pass:

1. known defects gate (zero unresolved critical/high),
2. latency gate (`reference/benchmarks/runtime-jsx-latest.json` A-grade thresholds),
3. reliability stress gate (`target/jsx-runtime-readiness/stress.json`),
4. reliability soak gate (`target/jsx-runtime-readiness/soak.json`).

## 4) Record release evidence

- [ ] Attach gate outputs and artifacts to the release record.
- [ ] Include benchmark and readiness JSON artifacts used by the gate run.
