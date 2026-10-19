//! The live run: environments, scripted graders, degenerate callers, trials,
//! estimators and the bootstrap interval for the pass@k minus pass^k gap.
//! Everything here follows docs/PREREGISTRATION.md; nothing is tuned to a
//! server.

use crate::client::{McpClient, Tool};
use crate::{pass_at_k, pass_power_k, Outcome};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Grader {
    /// Every sub-grader must pass.
    AllOf { graders: Vec<Grader> },
    /// The tool result, parsed as JSON (its text content when it is JSON), has `value` at `path` (JSON pointer).
    JsonPathEquals { path: String, value: serde_json::Value },
    /// The string at `path` matches `regex`.
    JsonPathRegex { path: String, regex: String },
    /// The concatenated text content contains `needle`.
    TextContains { needle: String },
    /// The concatenated text content matches `regex`.
    TextRegex { regex: String },
    /// The call must have been reported as an error by the server (isError or JSON-RPC error).
    ExpectError,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub grader: Grader,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerSpec {
    pub name: String,
    pub cmd: Vec<String>,
    #[serde(default)]
    pub version_cmd: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Setup {
    #[default]
    None,
    /// Create a throwaway git repository with one commit; `{repo}` in task args is replaced by its path.
    GitRepo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Environment {
    pub name: String,
    pub server: ServerSpec,
    #[serde(default)]
    pub setup: Setup,
    pub tasks: Vec<Task>,
}

/// What a trial saw: the parsed result (or error) and whether the call was made at all.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Observation {
    pub called: bool,
    pub is_error: bool,
    pub text: String,
    pub json: serde_json::Value,
    pub rpc_error: Option<String>,
}

fn text_of(result: &serde_json::Value) -> String {
    result["content"].as_array().map(|a| a.iter().filter_map(|c| c["text"].as_str()).collect::<Vec<_>>().join("\n")).unwrap_or_default()
}

pub fn observe(result: Result<serde_json::Value, crate::client::RpcError>) -> Observation {
    match result {
        Ok(r) => {
            let text = text_of(&r);
            let json = if !r["structuredContent"].is_null() { r["structuredContent"].clone() } else { serde_json::from_str(&text).unwrap_or(serde_json::Value::Null) };
            Observation { called: true, is_error: r["isError"].as_bool().unwrap_or(false), text, json, rpc_error: None }
        }
        Err(e) => Observation { called: true, is_error: true, text: String::new(), json: serde_json::Value::Null, rpc_error: Some(format!("{} ({})", e.message, e.code)) },
    }
}

pub fn grade(g: &Grader, o: &Observation) -> bool {
    match g {
        Grader::AllOf { graders } => graders.iter().all(|g| grade(g, o)),
        Grader::ExpectError => o.called && o.is_error,
        _ if !o.called || o.is_error => false,
        Grader::JsonPathEquals { path, value } => o.json.pointer(path) == Some(value),
        Grader::JsonPathRegex { path, regex } => o.json.pointer(path).and_then(|v| v.as_str()).map_or(false, |s| regex::Regex::new(regex).map_or(false, |re| re.is_match(s))),
        Grader::TextContains { needle } => o.text.contains(needle),
        Grader::TextRegex { regex } => regex::Regex::new(regex).map_or(false, |re| re.is_match(&o.text)),
    }
}

/// Degenerate callers from the pre-registration section 3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Caller {
    Scripted,
    NoOp,
    RandomArg,
    SchemaMismatch,
}

fn rand_string(rng: &mut rand_pcg::Pcg64Mcg, len: usize) -> String {
    use rand::Rng;
    const A: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    (0..len).map(|_| A[rng.gen_range(0..A.len())] as char).collect()
}

/// Arguments for the degenerate callers, derived from the tool's declared input schema.
pub fn degenerate_args(caller: Caller, schema: &serde_json::Value, rng: &mut rand_pcg::Pcg64Mcg) -> serde_json::Value {
    use rand::Rng;
    let mut out = serde_json::Map::new();
    let props = schema["properties"].as_object().cloned().unwrap_or_default();
    let required: Vec<String> = schema["required"].as_array().map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect()).unwrap_or_default();
    for (name, p) in props {
        let ty = p["type"].as_str().unwrap_or("string");
        let is_required = required.contains(&name);
        let v = match (caller, ty) {
            (Caller::RandomArg, "string") => {
                if let Some(e) = p["enum"].as_array().filter(|e| !e.is_empty()) { e[rng.gen_range(0..e.len())].clone() } else { serde_json::Value::String(rand_string(rng, 8)) }
            }
            (Caller::RandomArg, "integer") => serde_json::json!(rng.gen_range(-1000i64..1000)),
            (Caller::RandomArg, "number") => serde_json::json!(rng.gen_range(-1000.0f64..1000.0)),
            (Caller::RandomArg, "boolean") => serde_json::json!(rng.gen_bool(0.5)),
            (Caller::SchemaMismatch, "string") => serde_json::Value::String(if rng.gen_bool(0.5) { String::new() } else { rand_string(rng, 300) }),
            (Caller::SchemaMismatch, "integer") => serde_json::json!(-1),
            (Caller::SchemaMismatch, "number") => serde_json::json!(-1.0),
            (Caller::SchemaMismatch, "boolean") => serde_json::json!(false),
            (_, "array") => serde_json::json!([]),
            (_, "object") => serde_json::json!({}),
            _ => serde_json::Value::Null,
        };
        if is_required || rng.gen_bool(0.5) { out.insert(name, v); }
    }
    serde_json::Value::Object(out)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TaskResult {
    pub id: String,
    pub tool: String,
    pub n: usize,
    pub k: usize,
    pub c: usize,
    pub outcomes: Vec<Outcome>,
    pub refusals: usize,
    pub pass_at_1: f64,
    pub pass_at_k: f64,
    pub pass_pow_k: f64,
    pub gap: f64,
    pub gap_ci: (f64, f64),
    /// H1: gap 95% CI upper bound <= 0.10.
    pub h1_supported: bool,
    pub degenerate_pass_at_1: BTreeMap<String, f64>,
    pub flagged: bool,
    pub sample_error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunResult {
    pub tool: String,
    pub unix_time: u64,
    pub environment: String,
    pub server: ServerSpec,
    pub server_info: serde_json::Value,
    pub server_version: Option<String>,
    pub tools_advertised: Vec<String>,
    pub n: usize,
    pub k: usize,
    pub seed: u64,
    pub boot_iters: usize,
    pub tasks: Vec<TaskResult>,
    pub flagged_any: bool,
}

pub fn gap_bootstrap(outcomes: &[Outcome], k: usize, iters: usize, seed: u64) -> (f64, f64) {
    use rand::{Rng, SeedableRng};
    let n = outcomes.len();
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(seed);
    let mut gaps: Vec<f64> = (0..iters).map(|_| {
        let c = (0..n).filter(|_| outcomes[rng.gen_range(0..n)].is_pass()).count();
        pass_at_k(n, c, k) - pass_power_k(n, c, k)
    }).collect();
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let q = |p: f64| gaps[(((iters - 1) as f64) * p).round() as usize];
    (q(0.025), q(0.975))
}

pub struct RunArgs {
    pub n: usize,
    pub k: usize,
    pub seed: u64,
    pub boot_iters: usize,
    pub timeout: Duration,
    pub flag_threshold: f64,
}

fn setup_git_repo(seed: u64) -> anyhow::Result<PathBuf> {
    let dir = std::env::temp_dir().join(format!("mcpprobe-git-{}-{seed}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let git = |args: &[&str]| -> anyhow::Result<()> {
        let o = std::process::Command::new("git").args(["-c", "user.name=mcpprobe", "-c", "user.email=mcpprobe@example.invalid"]).args(args).current_dir(&dir).output()?;
        anyhow::ensure!(o.status.success(), "git {:?}: {}", args, String::from_utf8_lossy(&o.stderr));
        Ok(())
    };
    git(&["init", "-q", "-b", "main"])?;
    std::fs::write(dir.join("README.md"), "mcpprobe fixture repository\n")?;
    git(&["add", "README.md"])?;
    git(&["commit", "-q", "-m", "initial commit: mcpprobe fixture"])?;
    Ok(dir)
}

fn substitute(v: &serde_json::Value, key: &str, val: &str) -> serde_json::Value {
    match v {
        serde_json::Value::String(s) => serde_json::Value::String(s.replace(key, val)),
        serde_json::Value::Object(m) => serde_json::Value::Object(m.iter().map(|(k, x)| (k.clone(), substitute(x, key, val))).collect()),
        serde_json::Value::Array(a) => serde_json::Value::Array(a.iter().map(|x| substitute(x, key, val)).collect()),
        other => other.clone(),
    }
}

async fn one_trial(client: &mut McpClient, task: &Task, caller: Caller, schema: &serde_json::Value, rng: &mut rand_pcg::Pcg64Mcg) -> anyhow::Result<(Outcome, Observation)> {
    let obs = match caller {
        Caller::NoOp => Observation { called: false, is_error: false, text: String::new(), json: serde_json::Value::Null, rpc_error: None },
        Caller::Scripted => observe(client.tools_call(&task.tool, task.args.clone()).await?),
        c => observe(client.tools_call(&task.tool, degenerate_args(c, schema, rng)).await?),
    };
    let outcome = if obs.rpc_error.is_some() && !matches!(task.grader, Grader::ExpectError) { Outcome::Refuse } else if grade(&task.grader, &obs) { Outcome::Pass } else { Outcome::Fail };
    Ok((outcome, obs))
}

pub async fn run_environment(env: &Environment, a: &RunArgs) -> anyhow::Result<RunResult> {
    use rand::SeedableRng;
    let repo = match env.setup { Setup::GitRepo => Some(setup_git_repo(a.seed)?), Setup::None => None };
    let repo_str = repo.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
    let server_version = if env.server.version_cmd.is_empty() { None } else {
        std::process::Command::new(&env.server.version_cmd[0]).args(&env.server.version_cmd[1..]).output().ok().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    let mut client = McpClient::spawn(&env.server.cmd, repo.as_deref(), a.timeout).await?;
    client.initialize().await?;
    let tools: Vec<Tool> = client.tools_list().await?;
    let server_info = client.server_info.clone();
    let schemas: BTreeMap<String, serde_json::Value> = tools.iter().map(|t| (t.name.clone(), t.input_schema.clone())).collect();
    let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(a.seed);
    let mut results = Vec::new();
    for task in &env.tasks {
        let task = Task { id: task.id.clone(), tool: task.tool.clone(), args: substitute(&task.args, "{repo}", &repo_str), grader: task.grader.clone() };
        let schema = schemas.get(&task.tool).cloned().unwrap_or(serde_json::Value::Null);
        let mut outcomes = Vec::with_capacity(a.n);
        let mut sample_error = None;
        if !schemas.contains_key(&task.tool) {
            sample_error = Some(format!("tool `{}` not advertised by the server", task.tool));
            outcomes = vec![Outcome::Refuse; a.n];
        } else {
            for _ in 0..a.n {
                match one_trial(&mut client, &task, Caller::Scripted, &schema, &mut rng).await {
                    Ok((o, obs)) => { if sample_error.is_none() { if let Some(e) = &obs.rpc_error { sample_error = Some(e.clone()); } } outcomes.push(o); }
                    Err(e) => { sample_error = Some(e.to_string()); outcomes.push(Outcome::Refuse); }
                }
            }
        }
        let c = outcomes.iter().filter(|o| o.is_pass()).count();
        let mut degenerate = BTreeMap::new();
        for caller in [Caller::NoOp, Caller::RandomArg, Caller::SchemaMismatch] {
            let mut passes = 0usize;
            let mut made = 0usize;
            if schemas.contains_key(&task.tool) {
                for _ in 0..a.n {
                    if let Ok((o, _)) = one_trial(&mut client, &task, caller, &schema, &mut rng).await { made += 1; if o.is_pass() { passes += 1; } }
                }
            }
            degenerate.insert(serde_json::to_value(caller)?.as_str().unwrap().to_string(), if made == 0 { 0.0 } else { passes as f64 / made as f64 });
        }
        let gap = pass_at_k(a.n, c, a.k) - pass_power_k(a.n, c, a.k);
        let gap_ci = gap_bootstrap(&outcomes, a.k, a.boot_iters, a.seed ^ 0x5eed);
        let flagged = degenerate.values().any(|p| *p >= a.flag_threshold);
        results.push(TaskResult {
            id: task.id.clone(), tool: task.tool.clone(), n: a.n, k: a.k, c, refusals: outcomes.iter().filter(|o| matches!(o, Outcome::Refuse)).count(),
            outcomes, pass_at_1: c as f64 / a.n as f64, pass_at_k: pass_at_k(a.n, c, a.k), pass_pow_k: pass_power_k(a.n, c, a.k), gap, gap_ci,
            h1_supported: gap_ci.1 <= 0.10, degenerate_pass_at_1: degenerate, flagged, sample_error,
        });
    }
    client.shutdown().await;
    if let Some(r) = repo { let _ = std::fs::remove_dir_all(r); }
    Ok(RunResult {
        tool: format!("mcpprobe {}", env!("CARGO_PKG_VERSION")),
        unix_time: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs(),
        environment: env.name.clone(), server: env.server.clone(), server_info, server_version,
        tools_advertised: tools.iter().map(|t| t.name.clone()).collect(),
        n: a.n, k: a.k, seed: a.seed, boot_iters: a.boot_iters, flagged_any: results.iter().any(|r| r.flagged), tasks: results,
    })
}

pub fn load_environment(path: &Path) -> anyhow::Result<Environment> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obs(json: serde_json::Value, text: &str) -> Observation {
        Observation { called: true, is_error: false, text: text.into(), json, rpc_error: None }
    }

    #[test]
    fn graders_read_json_and_text() {
        let o = obs(serde_json::json!({"timezone": "UTC", "datetime": "2026-10-19T16:00:00+00:00"}), "irrelevant");
        assert!(grade(&Grader::JsonPathEquals { path: "/timezone".into(), value: serde_json::json!("UTC") }, &o));
        assert!(grade(&Grader::JsonPathRegex { path: "/datetime".into(), regex: r"^\d{4}-\d{2}-\d{2}T".into() }, &o));
        assert!(!grade(&Grader::JsonPathEquals { path: "/timezone".into(), value: serde_json::json!("Asia/Tokyo") }, &o));
        let t = obs(serde_json::Value::Null, "On branch main\nnothing to commit, working tree clean");
        assert!(grade(&Grader::TextContains { needle: "working tree clean".into() }, &t));
        assert!(grade(&Grader::AllOf { graders: vec![Grader::TextContains { needle: "main".into() }, Grader::TextRegex { regex: "clean$".into() }] }, &t));
    }

    #[test]
    fn noop_and_errors_never_pass_a_positive_grader() {
        let g = Grader::TextContains { needle: "".into() };
        let noop = Observation { called: false, is_error: false, text: String::new(), json: serde_json::Value::Null, rpc_error: None };
        assert!(!grade(&g, &noop));
        let err = Observation { called: true, is_error: true, text: "x".into(), json: serde_json::Value::Null, rpc_error: None };
        assert!(!grade(&g, &err));
        assert!(grade(&Grader::ExpectError, &err));
    }

    #[test]
    fn degenerate_args_cover_required_properties() {
        use rand::SeedableRng;
        let schema = serde_json::json!({"type": "object", "properties": {"timezone": {"type": "string"}, "n": {"type": "integer"}}, "required": ["timezone"]});
        let mut rng = rand_pcg::Pcg64Mcg::seed_from_u64(1);
        for caller in [Caller::RandomArg, Caller::SchemaMismatch] {
            let a = degenerate_args(caller, &schema, &mut rng);
            assert!(a.get("timezone").is_some(), "{caller:?} must keep required args");
        }
    }

    #[test]
    fn gap_bootstrap_is_zero_width_at_the_extremes() {
        let all = vec![Outcome::Pass; 8];
        assert_eq!(gap_bootstrap(&all, 4, 200, 1), (0.0, 0.0));
        let none = vec![Outcome::Fail; 8];
        assert_eq!(gap_bootstrap(&none, 4, 200, 1), (0.0, 0.0));
    }
}
