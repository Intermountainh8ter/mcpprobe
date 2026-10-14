//! mcpprobe CLI — scaffold.
//!
//! The initial release only knows how to compute pass@k / pass^k from a JSON
//! outcomes file. The `run` subcommand that actually hits an MCP server is
//! TODO; the pre-registration (`docs/PREREGISTRATION.md`) locks the metrics
//! and the degenerate-baseline gate before any real server call ships.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "mcpprobe", version, about = "MCP server reliability harness")]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Compute pass@k and pass^k from an outcomes JSON produced by `run`.
    Report {
        /// Path to a JSON file: {"n": u, "c": u, "k": u}.
        outcomes: std::path::PathBuf,
    },
    /// Print the version banner and exit.
    Version,
}

#[derive(serde::Deserialize)]
struct Outcomes {
    n: usize,
    c: usize,
    k: usize,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Cmd::Report { outcomes } => {
            let text = std::fs::read_to_string(&outcomes)?;
            let o: Outcomes = serde_json::from_str(&text)?;
            let at = mcpprobe::pass_at_k(o.n, o.c, o.k);
            let pow = mcpprobe::pass_power_k(o.n, o.c, o.k);
            println!("pass@{k} = {at:.4}", k = o.k, at = at);
            println!("pass^{k} = {pow:.4}", k = o.k, pow = pow);
        }
        Cmd::Version => {
            println!("mcpprobe {}", env!("CARGO_PKG_VERSION"));
        }
    }
    Ok(())
}
