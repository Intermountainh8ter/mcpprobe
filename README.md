# mcpprobe

**A reliability harness for Model Context Protocol servers.** Reports pass@k and
pass^k over tool-call tasks, audits every server against degenerate callers
(no-op, random-arg, schema-mismatch) before the numbers are trusted, and ships
fault-injection suites that wrap any server (frame-drop, stdio-reorder,
server-crash-after-N).

![status: initial scaffold](https://img.shields.io/badge/status-initial%20scaffold-lightgrey)
![language: Rust](https://img.shields.io/badge/language-Rust-blue)
![license: MIT](https://img.shields.io/badge/license-MIT-blue)

## Why

MCP server implementations are shipping fast and powering agent frameworks that
run in production. The public conformance tooling tests *does the server
respond*, not *does it respond consistently across k attempts*, and not *does
it refuse the obvious degenerate callers*.

Agent reliability work (trialgauntlet, tau-bench) measured pass^k on the
**agent** side. mcpprobe does the same measurement on the **server** side.

## What is in this scaffold

This repo starts as scaffold only. The initial commit ships:

- Core estimators (`pass@k`, `pass^k`) with the unbiased combinatorial
  formulas from Chen et al. (2021).
- A `report` CLI that reads an outcomes JSON and prints both numbers.
- `docs/PREREGISTRATION.md` — the statistical plan locked before any real
  server is probed.
- CI that builds on stable Rust and runs the unit tests.

Not in the initial commit (tracked, deliberately missing):

- `mcpprobe run` subcommand that actually hits an MCP server over stdio or
  HTTP+SSE and records trajectories.
- Three built-in environment adapters (filesystem, HTTP fetch, SQL query),
  each with scripted callers and deterministic graders.
- Degenerate-caller audits (`no-op`, `random-arg`, `schema-mismatch`).
- Fault-injection wrappers (`frame-drop:0.1`, `stdio-reorder`,
  `server-crash-after:N`).

## Usage (scaffold)

```sh
cargo install --path .
echo '{"n": 8, "c": 5, "k": 4}' > outcomes.json
mcpprobe report outcomes.json
# pass@4 = 0.9929
# pass^4 = 0.0714
```

## Methodology commitments

The pre-registration doc governs every result this repo ever publishes.

- No tuning on the TEST split. Trial seeds are derived from a single run seed.
- Bootstrap 95% CIs are reported on every rate.
- Degenerate baselines run on every server before a reliability number is
  trusted; a server whose no-op baseline scores above 5% is **flagged** and
  excluded from the leaderboard until the environment is fixed.
- Pre-registration is append-only: hypotheses, thresholds, and the
  degenerate-baseline gate were fixed before results and cannot be moved after.

See `docs/PREREGISTRATION.md`.

## License

MIT. See `LICENSE`.
