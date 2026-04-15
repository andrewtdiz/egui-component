# Runtime JSX Benchmark Contract

This directory defines the reproducible benchmark contract for **A-grade runtime performance**.

## Benchmark target

```bash
cargo run --example runtime-jsx-benchmark
```

Architecture integration gate:

```bash
cargo test -p clay-jsx-egui-bridge architecture_v1_integration_matrix_covers_load_dispatch_async_motion_reload_and_teardown -- --nocapture
```

Optional strict gate (exit non-zero when A-grade thresholds fail):

```bash
CLAY_JSX_BENCH_ENFORCE_A_GRADE=1 cargo run --example runtime-jsx-benchmark
```

Validate ship-readiness artifact gates in CI (fails non-zero on any failed gate):

```bash
cargo run --example runtime-jsx-benchmark -- verify-readiness-artifact target/jsx-runtime-readiness/stress.json stress
cargo run --example runtime-jsx-benchmark -- verify-readiness-artifact target/jsx-runtime-readiness/soak.json soak
```

Run the full A-grade release gate (defects + latency + reliability):

```bash
python3 scripts/release_a_grade_gate.py
```

See `docs/release-a-grade-checklist.md` for the full release evidence checklist.

## Contract

`runtime-jsx-contract.json` defines:

- warmup and measured sample counts,
- representative tree profiles (`label_count`),
- commit-throughput minimums,
- p50/p95 A-grade thresholds for:
  - event-dispatch latency
  - callback-drain latency

## Artifacts

Each benchmark run updates:

- `runtime-jsx-latest.json` — latest full benchmark report,
- `runtime-jsx-history.csv` — append-only p50/p95 trend rows per profile.

These artifacts provide a consistent proof surface for performance claims.
