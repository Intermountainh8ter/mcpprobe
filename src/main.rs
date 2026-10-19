//! mcpprobe CLI.
//!
//! `run` probes one MCP server over stdio with the scripted tasks of an
//! environment file, runs the three degenerate callers, and writes a result
//! JSON. `report` renders `results/*.json` into the README. `estimate` is the
//! bare pass@k / pass^k calculator. docs/PREREGISTRATION.md locks n, k, the
//! gap hypothesis and the degenerate-baseline gate.

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "mcpprobe", version, about = "MCP server reliability harness")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Compute pass@k and pass^k from {"n": u, "c": u, "k": u}.
    Estimate {
        outcomes: PathBuf,
    },
    /// Probe one server with one environment file and write the result JSON.
    Run {
        #[arg(long)]
        env: PathBuf,
        #[arg(long, default_value_t = 8)]
        n: usize,
        #[arg(long, default_value_t = 4)]
        k: usize,
        #[arg(long, default_value_t = 0)]
        seed: u64,
        #[arg(long, default_value_t = 2000)]
        bootstrap: usize,
        #[arg(long, default_value_t = 30)]
        timeout_secs: u64,
        #[arg(long)]
        out: PathBuf,
    },
    /// Render results/*.json into the README between RESULTS:START/END.
    Report {
        #[arg(long, default_value = "results")]
        results: PathBuf,
        #[arg(long, default_value = "README.md")]
        readme: PathBuf,
    },
    Version,
}

#[derive(serde::Deserialize)]
struct Outcomes {
    n: usize,
    c: usize,
    k: usize,
}

fn pct(x: f64) -> String {
    format!("{:.1}%", 100.0 * x)
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Estimate { outcomes } => {
            let o: Outcomes = serde_json::from_str(&std::fs::read_to_string(&outcomes)?)?;
            println!("pass@{} = {:.4}", o.k, mcpprobe::pass_at_k(o.n, o.c, o.k));
            println!("pass^{} = {:.4}", o.k, mcpprobe::pass_power_k(o.n, o.c, o.k));
        }
        Cmd::Run { env, n, k, seed, bootstrap, timeout_secs, out } => {
            anyhow::ensure!(k <= n, "k must be <= n");
            let environment = mcpprobe::run::load_environment(&env)?;
            let args = mcpprobe::run::RunArgs { n, k, seed, boot_iters: bootstrap, timeout: std::time::Duration::from_secs(timeout_secs), flag_threshold: 0.05 };
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
            let result = rt.block_on(mcpprobe::run::run_environment(&environment, &args))?;
            if let Some(d) = out.parent() { std::fs::create_dir_all(d)?; }
            std::fs::write(&out, serde_json::to_string_pretty(&result)?)?;
            for t in &result.tasks {
                println!("{:28} c={}/{} pass@1 {} pass@{} {} pass^{} {} gap {} [{}, {}] H1 {} degenerate {:?}{}{}", t.id, t.c, t.n, pct(t.pass_at_1), k, pct(t.pass_at_k), k, pct(t.pass_pow_k), pct(t.gap), pct(t.gap_ci.0), pct(t.gap_ci.1), if t.h1_supported { "supported" } else { "NOT supported" }, t.degenerate_pass_at_1, if t.flagged { " FLAGGED" } else { "" }, t.sample_error.as_ref().map(|e| format!(" error: {e}")).unwrap_or_default());
            }
        }
        Cmd::Report { results, readme } => {
            let mut files: Vec<PathBuf> = std::fs::read_dir(&results)?.filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().map_or(false, |e| e == "json") && !p.file_name().map_or(false, |f| f.to_string_lossy().contains("MANIFEST"))).collect();
            files.sort();
            let mut t = String::from("| server (version) | task | tool | c / n | pass@1 | pass@k | pass^k | gap [95% CI] | H1 gap ≤ 10 pp | no-op / random-arg / schema-mismatch pass@1 | flagged |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
            let mut n_servers = 0;
            for f in &files {
                let r: mcpprobe::run::RunResult = match serde_json::from_str(&std::fs::read_to_string(f)?) { Ok(r) => r, Err(_) => continue };
                n_servers += 1;
                let server = format!("{} ({})", r.environment, r.server_version.clone().or_else(|| r.server_info["version"].as_str().map(String::from)).unwrap_or_else(|| "version unknown".into()));
                for task in &r.tasks {
                    let d = &task.degenerate_pass_at_1;
                    t.push_str(&format!("| {} | {} | `{}` | {} / {} | {} | {} | {} | {} [{}, {}] | {} | {} / {} / {} | {} |\n",
                        server, task.id, task.tool, task.c, task.n, pct(task.pass_at_1), pct(task.pass_at_k), pct(task.pass_pow_k), pct(task.gap), pct(task.gap_ci.0), pct(task.gap_ci.1),
                        if task.h1_supported { "supported" } else { "not supported" },
                        pct(*d.get("no_op").unwrap_or(&0.0)), pct(*d.get("random_arg").unwrap_or(&0.0)), pct(*d.get("schema_mismatch").unwrap_or(&0.0)),
                        if task.flagged { "yes" } else { "no" }));
                }
            }
            if n_servers == 0 { t.push_str("| (no runs yet) | | | | | | | | | | |\n"); }
            let block = format!("Scripted callers and graders, no LLM anywhere. n = 8 trials per task, k = 4, 2,000 bootstrap resamples for the gap interval; a task is flagged when any degenerate caller scores pass@1 ≥ 5% and is then excluded from any leaderboard. pass@k is \"at least one of k trials passes\", pass^k is \"all k pass\".\n\n{t}");
            let readme_text = std::fs::read_to_string(&readme)?;
            let (s, e) = ("<!-- RESULTS:START -->", "<!-- RESULTS:END -->");
            let (a, b) = (readme_text.find(s).ok_or_else(|| anyhow::anyhow!("README lacks {s}"))?, readme_text.find(e).ok_or_else(|| anyhow::anyhow!("README lacks {e}"))?);
            std::fs::write(&readme, format!("{}{}\n{}\n{}", &readme_text[..a], s, block, &readme_text[b..]))?;
            print!("{block}");
        }
        Cmd::Version => println!("mcpprobe {}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
