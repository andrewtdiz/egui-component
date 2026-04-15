use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use clay_jsx_egui_bridge::JsxRuntimeSession;
use clay_jsx_runtime::contract::{ContractEvent, EventKind};
use serde::{Deserialize, Serialize};

const CONTRACT_PATH: &str = "reference/benchmarks/runtime-jsx-contract.json";
const LATEST_ARTIFACT_PATH: &str = "reference/benchmarks/runtime-jsx-latest.json";
const HISTORY_CSV_PATH: &str = "reference/benchmarks/runtime-jsx-history.csv";
const GENERATED_FIXTURE_DIR: &str = "target/jsx-runtime-benchmark";
const ENFORCE_A_GRADE_ENV: &str = "CLAY_JSX_BENCH_ENFORCE_A_GRADE";
const VERIFY_READINESS_SUBCOMMAND: &str = "verify-readiness-artifact";

fn main() {
    if let Err(error) = run_cli() {
        eprintln!("runtime-jsx benchmark failed: {error}");
        std::process::exit(1);
    }
}

fn run_cli() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None => run_benchmark(),
        Some(VERIFY_READINESS_SUBCOMMAND) => {
            let artifact_path = args.next().ok_or_else(|| {
                format!(
                    "usage: cargo run --example runtime-jsx-benchmark -- {VERIFY_READINESS_SUBCOMMAND} <artifact-path> <scenario>"
                )
            })?;
            let expected_scenario = args.next().ok_or_else(|| {
                format!(
                    "usage: cargo run --example runtime-jsx-benchmark -- {VERIFY_READINESS_SUBCOMMAND} <artifact-path> <scenario>"
                )
            })?;
            if let Some(unexpected) = args.next() {
                return Err(format!(
                    "unexpected argument `{unexpected}` for {VERIFY_READINESS_SUBCOMMAND}; expected exactly <artifact-path> <scenario>"
                )
                .into());
            }
            verify_readiness_artifact(Path::new(&artifact_path), &expected_scenario)
        }
        Some("-h") | Some("--help") | Some("help") => {
            println!(
                "runtime-jsx benchmark commands:\n  cargo run --example runtime-jsx-benchmark\n  cargo run --example runtime-jsx-benchmark -- {VERIFY_READINESS_SUBCOMMAND} <artifact-path> <scenario>"
            );
            Ok(())
        }
        Some(command) => Err(format!(
            "unknown command `{command}`; run with --help for supported commands"
        )
        .into()),
    }
}

fn run_benchmark() -> Result<(), Box<dyn std::error::Error>> {
    let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let contract_path = workspace_root.join(CONTRACT_PATH);
    let latest_artifact_path = workspace_root.join(LATEST_ARTIFACT_PATH);
    let history_csv_path = workspace_root.join(HISTORY_CSV_PATH);

    let contract = load_contract(&contract_path)?;
    validate_contract(&contract)?;

    let mut profiles = Vec::with_capacity(contract.profiles.len());
    for profile in &contract.profiles {
        let result = benchmark_profile(&workspace_root, &contract, profile)?;
        print_profile_summary(&result);
        profiles.push(result);
    }

    let artifact = BenchmarkArtifact {
        schema_version: contract.schema_version,
        benchmark_name: contract.benchmark_name.clone(),
        git_sha: git_sha(),
        timestamp_unix_secs: unix_timestamp_secs(),
        command: contract.command.clone(),
        warmup_iterations: contract.warmup_iterations,
        measured_iterations: contract.measured_iterations,
        drain_settle_millis: contract.drain_settle_millis,
        profiles,
    };

    write_artifact(&latest_artifact_path, &artifact)?;
    append_history_csv(&history_csv_path, &artifact)?;

    println!(
        "wrote benchmark artifacts:\n  {}\n  {}",
        latest_artifact_path.display(),
        history_csv_path.display()
    );
    println!(
        "overall A-grade status: {}",
        if artifact.all_profiles_a_grade() {
            "PASS"
        } else {
            "FAIL"
        }
    );

    if enforce_a_grade() && !artifact.all_profiles_a_grade() {
        return Err(format!(
            "A-grade thresholds failed (set {ENFORCE_A_GRADE_ENV}=0 to collect metrics without failing)"
        )
        .into());
    }

    Ok(())
}

#[derive(Debug, Clone, Deserialize)]
struct ReadinessArtifactGateCheck {
    scenario: String,
    passed: bool,
    gates: BTreeMap<String, bool>,
}

fn verify_readiness_artifact(
    artifact_path: &Path,
    expected_scenario: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let bytes = fs::read(artifact_path).map_err(|error| {
        format!(
            "failed to read readiness artifact {}: {error}",
            artifact_path.display()
        )
    })?;
    let artifact: ReadinessArtifactGateCheck = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse readiness artifact {}: {error}",
            artifact_path.display()
        )
    })?;

    if artifact.scenario != expected_scenario {
        return Err(format!(
            "readiness artifact {} scenario mismatch: expected `{expected_scenario}`, got `{}`",
            artifact_path.display(),
            artifact.scenario
        )
        .into());
    }

    if artifact.gates.is_empty() {
        return Err(format!(
            "readiness artifact {} contains no gates",
            artifact_path.display()
        )
        .into());
    }

    let failed_gates = artifact
        .gates
        .iter()
        .filter_map(|(name, passed)| (!*passed).then_some(name.as_str()))
        .collect::<Vec<_>>();
    let derived_passed = failed_gates.is_empty();

    if artifact.passed != derived_passed {
        return Err(format!(
            "readiness artifact {} has inconsistent pass summary: passed={} but derived_passed={} from gates",
            artifact_path.display(),
            artifact.passed,
            derived_passed
        )
        .into());
    }

    if !artifact.passed {
        return Err(format!(
            "readiness artifact {} failed gates: {}",
            artifact_path.display(),
            failed_gates.join(", ")
        )
        .into());
    }

    println!(
        "readiness artifact check passed: {} (scenario={}, gates={})",
        artifact_path.display(),
        artifact.scenario,
        artifact.gates.len(),
    );
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkContract {
    schema_version: u32,
    benchmark_name: String,
    command: String,
    warmup_iterations: usize,
    measured_iterations: usize,
    drain_settle_millis: u64,
    profiles: Vec<BenchmarkProfileContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BenchmarkProfileContract {
    id: String,
    label_count: usize,
    a_grade: AGradeThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AGradeThresholds {
    commit_throughput_commits_per_sec_min: f64,
    event_dispatch_latency_p50_ms_max: f64,
    event_dispatch_latency_p95_ms_max: f64,
    callback_drain_latency_p50_ms_max: f64,
    callback_drain_latency_p95_ms_max: f64,
}

#[derive(Debug, Clone, Serialize)]
struct BenchmarkArtifact {
    schema_version: u32,
    benchmark_name: String,
    git_sha: String,
    timestamp_unix_secs: u64,
    command: String,
    warmup_iterations: usize,
    measured_iterations: usize,
    drain_settle_millis: u64,
    profiles: Vec<BenchmarkProfileArtifact>,
}

impl BenchmarkArtifact {
    fn all_profiles_a_grade(&self) -> bool {
        self.profiles.iter().all(|profile| profile.a_grade.passed)
    }
}

#[derive(Debug, Clone, Serialize)]
struct BenchmarkProfileArtifact {
    id: String,
    label_count: usize,
    commit_throughput: CommitThroughputStats,
    event_dispatch_latency: LatencyStats,
    callback_drain_latency: LatencyStats,
    a_grade: ProfileGradeResult,
}

#[derive(Debug, Clone, Serialize)]
struct ProfileGradeResult {
    passed: bool,
    commit_throughput_passed: bool,
    event_dispatch_latency_passed: bool,
    callback_drain_latency_passed: bool,
    thresholds: AGradeThresholds,
}

#[derive(Debug, Clone, Serialize)]
struct CommitThroughputStats {
    measured_commits: u64,
    total_elapsed_ms: f64,
    commits_per_sec: f64,
}

#[derive(Debug, Clone, Serialize)]
struct LatencyStats {
    samples: usize,
    min_ms: f64,
    mean_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    max_ms: f64,
}

fn benchmark_profile(
    workspace_root: &Path,
    contract: &BenchmarkContract,
    profile: &BenchmarkProfileContract,
) -> Result<BenchmarkProfileArtifact, Box<dyn std::error::Error>> {
    let fixture_dir = workspace_root.join(GENERATED_FIXTURE_DIR);
    fs::create_dir_all(&fixture_dir)?;

    let dispatch_fixture = fixture_dir.join(format!("dispatch-{}.tsx", profile.id));
    let drain_fixture = fixture_dir.join(format!("drain-{}.tsx", profile.id));
    write_dispatch_fixture(&dispatch_fixture, profile.label_count)?;
    write_drain_fixture(&drain_fixture, profile.label_count)?;

    let (mut dispatch_session, rendered) = JsxRuntimeSession::load(&dispatch_fixture)?;
    let _initial_tree = rendered.tree.ok_or_else(|| {
        format!(
            "{} did not return an initial tree",
            dispatch_fixture.display()
        )
    })?;
    let dispatch_benchmark = benchmark_dispatch_events(
        &mut dispatch_session,
        contract.warmup_iterations,
        contract.measured_iterations,
    )?;
    let _ = dispatch_session.teardown()?;

    let (mut drain_session, _rendered) = JsxRuntimeSession::load(&drain_fixture)?;
    let callback_drain_samples = benchmark_drain_pending_runtime_updates(
        &mut drain_session,
        contract.warmup_iterations,
        contract.measured_iterations,
        Duration::from_millis(contract.drain_settle_millis),
    )?;
    let _ = drain_session.teardown()?;

    let event_dispatch_latency = compute_latency_stats(&dispatch_benchmark.samples)?;
    let callback_drain_latency = compute_latency_stats(&callback_drain_samples)?;

    let a_grade = evaluate_profile_grade(
        &profile.a_grade,
        &dispatch_benchmark.commit_throughput,
        &event_dispatch_latency,
        &callback_drain_latency,
    );

    Ok(BenchmarkProfileArtifact {
        id: profile.id.clone(),
        label_count: profile.label_count,
        commit_throughput: dispatch_benchmark.commit_throughput,
        event_dispatch_latency,
        callback_drain_latency,
        a_grade,
    })
}

struct DispatchBenchmarkResult {
    samples: Vec<Duration>,
    commit_throughput: CommitThroughputStats,
}

fn benchmark_dispatch_events(
    session: &mut JsxRuntimeSession,
    warmup_iterations: usize,
    measured_iterations: usize,
) -> Result<DispatchBenchmarkResult, Box<dyn std::error::Error>> {
    let mut samples = Vec::with_capacity(measured_iterations);
    let total_iterations = warmup_iterations + measured_iterations;
    let mut measured_total_elapsed = Duration::ZERO;
    let mut measured_metrics_baseline = None;

    for iteration in 0..total_iterations {
        if iteration == warmup_iterations {
            measured_metrics_baseline = Some(session.debug_metrics());
        }

        let started = Instant::now();
        let rendered =
            session.dispatch_events(&[ContractEvent::new("dispatch-next", EventKind::Clicked)])?;
        let elapsed = started.elapsed();

        let _current_tree = rendered.tree.ok_or_else(|| {
            format!(
                "dispatch iteration {iteration} returned no contract tree; fixture must produce a visible update"
            )
        })?;

        if iteration >= warmup_iterations {
            samples.push(elapsed);
            measured_total_elapsed += elapsed;
        }
    }

    let Some(measured_metrics_baseline) = measured_metrics_baseline else {
        return Err("dispatch benchmark did not establish a measured metrics baseline".into());
    };
    let measured_metrics_final = session.debug_metrics();
    let measured_commits = measured_metrics_final
        .mutation_batch_count
        .saturating_sub(measured_metrics_baseline.mutation_batch_count);
    if measured_commits == 0 {
        return Err("dispatch benchmark produced no measured commit batches".into());
    }
    if measured_total_elapsed.is_zero() {
        return Err("dispatch benchmark produced zero measured elapsed time".into());
    }

    let commit_throughput = CommitThroughputStats {
        measured_commits,
        total_elapsed_ms: round_ms(measured_total_elapsed.as_secs_f64() * 1_000.0),
        commits_per_sec: round_ms(measured_commits as f64 / measured_total_elapsed.as_secs_f64()),
    };

    Ok(DispatchBenchmarkResult {
        samples,
        commit_throughput,
    })
}

fn benchmark_drain_pending_runtime_updates(
    session: &mut JsxRuntimeSession,
    warmup_iterations: usize,
    measured_iterations: usize,
    settle_duration: Duration,
) -> Result<Vec<Duration>, Box<dyn std::error::Error>> {
    let mut samples = Vec::with_capacity(measured_iterations);
    let total_iterations = warmup_iterations + measured_iterations;

    for iteration in 0..total_iterations {
        let _ =
            session.dispatch_events(&[ContractEvent::new("schedule-drain", EventKind::Clicked)])?;
        std::thread::sleep(settle_duration);
        let elapsed = drain_until_non_empty_update(session, iteration)?;
        if iteration >= warmup_iterations {
            samples.push(elapsed);
        }
    }

    Ok(samples)
}

fn drain_until_non_empty_update(
    session: &mut JsxRuntimeSession,
    iteration: usize,
) -> Result<Duration, Box<dyn std::error::Error>> {
    let deadline = Instant::now() + Duration::from_millis(250);
    loop {
        let started = Instant::now();
        let update = session.drain_pending_runtime_updates()?;
        let elapsed = started.elapsed();
        if update.is_some() {
            return Ok(elapsed);
        }

        if Instant::now() >= deadline {
            return Err(format!(
                "drain iteration {iteration} timed out waiting for an async runtime update"
            )
            .into());
        }

        std::thread::sleep(Duration::from_millis(1));
    }
}

fn compute_latency_stats(samples: &[Duration]) -> Result<LatencyStats, Box<dyn std::error::Error>> {
    if samples.is_empty() {
        return Err("latency sample set cannot be empty".into());
    }

    let mut values_ms = samples
        .iter()
        .map(|sample| sample.as_secs_f64() * 1_000.0)
        .collect::<Vec<_>>();
    values_ms.sort_by(|left, right| left.total_cmp(right));

    let sum = values_ms.iter().sum::<f64>();
    let count = values_ms.len();
    let mean = sum / count as f64;
    let p50 = percentile_nearest_rank(&values_ms, 0.50);
    let p95 = percentile_nearest_rank(&values_ms, 0.95);

    Ok(LatencyStats {
        samples: count,
        min_ms: round_ms(values_ms[0]),
        mean_ms: round_ms(mean),
        p50_ms: round_ms(p50),
        p95_ms: round_ms(p95),
        max_ms: round_ms(values_ms[count - 1]),
    })
}

fn percentile_nearest_rank(sorted_values: &[f64], percentile: f64) -> f64 {
    debug_assert!(!sorted_values.is_empty());
    let rank = (percentile * sorted_values.len() as f64).ceil() as usize;
    let index = rank.saturating_sub(1).min(sorted_values.len() - 1);
    sorted_values[index]
}

fn round_ms(value: f64) -> f64 {
    (value * 1_000.0).round() / 1_000.0
}

fn evaluate_profile_grade(
    thresholds: &AGradeThresholds,
    commit_throughput: &CommitThroughputStats,
    event_dispatch_latency: &LatencyStats,
    callback_drain_latency: &LatencyStats,
) -> ProfileGradeResult {
    let commit_throughput_passed =
        commit_throughput.commits_per_sec >= thresholds.commit_throughput_commits_per_sec_min;
    let event_dispatch_latency_passed = event_dispatch_latency.p50_ms
        <= thresholds.event_dispatch_latency_p50_ms_max
        && event_dispatch_latency.p95_ms <= thresholds.event_dispatch_latency_p95_ms_max;
    let callback_drain_latency_passed = callback_drain_latency.p50_ms
        <= thresholds.callback_drain_latency_p50_ms_max
        && callback_drain_latency.p95_ms <= thresholds.callback_drain_latency_p95_ms_max;

    ProfileGradeResult {
        passed: commit_throughput_passed
            && event_dispatch_latency_passed
            && callback_drain_latency_passed,
        commit_throughput_passed,
        event_dispatch_latency_passed,
        callback_drain_latency_passed,
        thresholds: thresholds.clone(),
    }
}

fn print_profile_summary(profile: &BenchmarkProfileArtifact) {
    println!(
        "[{id}] labels={labels}: commit throughput={commit_throughput:.3} commits/s ({commit_count} commits) | dispatch p50={dispatch_p50:.3}ms p95={dispatch_p95:.3}ms | callback drain p50={drain_p50:.3}ms p95={drain_p95:.3}ms | A-grade={grade}",
        id = profile.id,
        labels = profile.label_count,
        commit_throughput = profile.commit_throughput.commits_per_sec,
        commit_count = profile.commit_throughput.measured_commits,
        dispatch_p50 = profile.event_dispatch_latency.p50_ms,
        dispatch_p95 = profile.event_dispatch_latency.p95_ms,
        drain_p50 = profile.callback_drain_latency.p50_ms,
        drain_p95 = profile.callback_drain_latency.p95_ms,
        grade = if profile.a_grade.passed { "PASS" } else { "FAIL" },
    );
}

fn load_contract(path: &Path) -> Result<BenchmarkContract, Box<dyn std::error::Error>> {
    let bytes = fs::read(path)?;
    let contract: BenchmarkContract = serde_json::from_slice(&bytes)?;
    Ok(contract)
}

fn validate_contract(contract: &BenchmarkContract) -> Result<(), Box<dyn std::error::Error>> {
    if contract.warmup_iterations == 0 {
        return Err("benchmark contract warmup_iterations must be > 0".into());
    }
    if contract.measured_iterations == 0 {
        return Err("benchmark contract measured_iterations must be > 0".into());
    }
    if contract.profiles.is_empty() {
        return Err("benchmark contract must include at least one profile".into());
    }
    for profile in &contract.profiles {
        if profile.id.trim().is_empty() {
            return Err("benchmark profile id cannot be empty".into());
        }
        if profile.label_count == 0 {
            return Err(
                format!("benchmark profile {} must have label_count > 0", profile.id).into(),
            );
        }
    }
    Ok(())
}

fn write_artifact(
    path: &Path,
    artifact: &BenchmarkArtifact,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(artifact)?;
    fs::write(path, json)?;
    Ok(())
}

fn append_history_csv(
    path: &Path,
    artifact: &BenchmarkArtifact,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    if file.metadata()?.len() == 0 {
        file.write_all(
            b"timestamp_unix_secs,git_sha,profile,label_count,commit_throughput_commits_per_sec,event_dispatch_latency_p50_ms,event_dispatch_latency_p95_ms,callback_drain_latency_p50_ms,callback_drain_latency_p95_ms,a_grade_passed\n",
        )?;
    }

    for profile in &artifact.profiles {
        writeln!(
            file,
            "{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{}",
            artifact.timestamp_unix_secs,
            artifact.git_sha,
            profile.id,
            profile.label_count,
            profile.commit_throughput.commits_per_sec,
            profile.event_dispatch_latency.p50_ms,
            profile.event_dispatch_latency.p95_ms,
            profile.callback_drain_latency.p50_ms,
            profile.callback_drain_latency.p95_ms,
            profile.a_grade.passed,
        )?;
    }

    Ok(())
}

fn write_dispatch_fixture(
    path: &Path,
    label_count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let source = format!(
        r#"
import {{ render, useState }} from "egui";

const labels = Array.from({{ length: {label_count} }}, (_, index) => `bench-${{String(index)}}`);

function App() {{
  const [revision, setRevision] = useState(0);
  const ordered = revision % 2 === 0 ? labels : [...labels].reverse();

  return (
    <div id="root" data-slot="column" className="flex flex-col items-stretch gap-1">
      <button
        id="dispatch-next"
        label="dispatch-next"
        onClick={{() => setRevision((value) => value + 1)}}
      />
      <label id="dispatch-status" text={{`revision:${{String(revision)}}`}} />
      <div id="dispatch-list" data-slot="column" className="flex flex-col items-stretch gap-1">
        {{ordered.map((label, index) => (
          <label
            key={{label}}
            id={{`dispatch-${{label}}`}}
            text={{`${{label}}:${{String(revision)}}:${{String(index)}}`}}
          />
        ))}}
      </div>
    </div>
  );
}}

render(<App />);
"#,
    );
    fs::write(path, source)?;
    Ok(())
}

fn write_drain_fixture(path: &Path, label_count: usize) -> Result<(), Box<dyn std::error::Error>> {
    let source = format!(
        r#"
import {{ render, useState }} from "egui";

const labels = Array.from({{ length: {label_count} }}, (_, index) => `bench-${{String(index)}}`);

function App() {{
  const [tick, setTick] = useState(0);

  return (
    <div id="root" data-slot="column" className="flex flex-col items-stretch gap-1">
      <button
        id="schedule-drain"
        label="schedule-drain"
        onClick={{() => setTimeout(() => setTick((value) => value + 1), 1)}}
      />
      <label id="drain-status" text={{`tick:${{String(tick)}}`}} />
      <div id="drain-list" data-slot="column" className="flex flex-col items-stretch gap-1">
        {{labels.map((label, index) => (
          <label
            key={{label}}
            id={{`drain-${{label}}`}}
            text={{`${{label}}:${{String(index)}}`}}
          />
        ))}}
      </div>
    </div>
  );
}}

render(<App />);
"#,
    );
    fs::write(path, source)?;
    Ok(())
}

fn git_sha() -> String {
    let output = match Command::new("git").args(["rev-parse", "HEAD"]).output() {
        Ok(output) => output,
        Err(_) => return "unknown".to_owned(),
    };
    if !output.status.success() {
        return "unknown".to_owned();
    }
    String::from_utf8(output.stdout)
        .map(|sha| sha.trim().to_owned())
        .unwrap_or_else(|_| "unknown".to_owned())
}

fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_secs()
}

fn enforce_a_grade() -> bool {
    std::env::var(ENFORCE_A_GRADE_ENV)
        .ok()
        .as_deref()
        .map(|value| matches!(value, "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}
