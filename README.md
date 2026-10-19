# mcpprobe

**A reliability harness for Model Context Protocol servers.** Reports pass@k and
pass^k over tool-call tasks, audits every server against degenerate callers
(no-op, random-arg, schema-mismatch) before the numbers are trusted, and ships
fault-injection suites that wrap any server (frame-drop, stdio-reorder,
server-crash-after-N).

![status: first live runs](https://img.shields.io/badge/status-first%20live%20runs-blue)
![language: Rust](https://img.shields.io/badge/language-Rust-blue)
![license: MIT](https://img.shields.io/badge/license-MIT-blue)

## Results

<!-- RESULTS:START -->
Scripted callers and graders, no LLM anywhere. n = 8 trials per task, k = 4, 2,000 bootstrap resamples for the gap interval; a task is flagged when any degenerate caller scores pass@1 ≥ 5% and is then excluded from any leaderboard. pass@k is "at least one of k trials passes", pass^k is "all k pass".

| server (version) | task | tool | c / n | pass@1 | pass@k | pass^k | gap [95% CI] | H1 gap ≤ 10 pp | no-op / random-arg / schema-mismatch pass@1 | flagged |
|---|---|---|---|---|---|---|---|---|---|---|
| mcp-server-git (Name: mcp-server-git
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools to read, search, and manipulate Git repositories programmatically via LLMs
Home-page: 
Author: Anthropic, PBC.
Author-email: 
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: click, gitpython, mcp, pydantic
Required-by:) | status-clean-tree | `git_status` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 50.0% | yes |
| mcp-server-git (Name: mcp-server-git
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools to read, search, and manipulate Git repositories programmatically via LLMs
Home-page: 
Author: Anthropic, PBC.
Author-email: 
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: click, gitpython, mcp, pydantic
Required-by:) | log-shows-fixture-commit | `git_log` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 25.0% | yes |
| mcp-server-git (Name: mcp-server-git
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools to read, search, and manipulate Git repositories programmatically via LLMs
Home-page: 
Author: Anthropic, PBC.
Author-email: 
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: click, gitpython, mcp, pydantic
Required-by:) | branch-list-has-main | `git_branch` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 0.0% | no |
| mcp-server-git (Name: mcp-server-git
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools to read, search, and manipulate Git repositories programmatically via LLMs
Home-page: 
Author: Anthropic, PBC.
Author-email: 
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: click, gitpython, mcp, pydantic
Required-by:) | show-head-commit | `git_show` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 0.0% | no |
| mcp-server-git (Name: mcp-server-git
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools to read, search, and manipulate Git repositories programmatically via LLMs
Home-page: 
Author: Anthropic, PBC.
Author-email: 
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: click, gitpython, mcp, pydantic
Required-by:) | missing-repo-is-an-error | `git_status` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 100.0% / 50.0% | yes |
| mcp-server-time (Name: mcp-server-time
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools for time queries and timezone conversions for LLMs
Home-page: 
Author: 
Author-email: Mariusz 'maledorak' Korzekwa <mariusz@korzekwa.dev>
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: mcp, pydantic, tzdata, tzlocal
Required-by:) | current-time-utc | `get_current_time` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 0.0% | no |
| mcp-server-time (Name: mcp-server-time
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools for time queries and timezone conversions for LLMs
Home-page: 
Author: 
Author-email: Mariusz 'maledorak' Korzekwa <mariusz@korzekwa.dev>
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: mcp, pydantic, tzdata, tzlocal
Required-by:) | current-time-tokyo | `get_current_time` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 0.0% | no |
| mcp-server-time (Name: mcp-server-time
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools for time queries and timezone conversions for LLMs
Home-page: 
Author: 
Author-email: Mariusz 'maledorak' Korzekwa <mariusz@korzekwa.dev>
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: mcp, pydantic, tzdata, tzlocal
Required-by:) | convert-utc-to-tokyo | `convert_time` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 0.0% | no |
| mcp-server-time (Name: mcp-server-time
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools for time queries and timezone conversions for LLMs
Home-page: 
Author: 
Author-email: Mariusz 'maledorak' Korzekwa <mariusz@korzekwa.dev>
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: mcp, pydantic, tzdata, tzlocal
Required-by:) | convert-utc-to-kolkata | `convert_time` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 0.0% / 0.0% | no |
| mcp-server-time (Name: mcp-server-time
Version: 2026.10.19
Summary: A Model Context Protocol server providing tools for time queries and timezone conversions for LLMs
Home-page: 
Author: 
Author-email: Mariusz 'maledorak' Korzekwa <mariusz@korzekwa.dev>
License: MIT
Location: /home/ammo/bench/.venv-mcp/lib/python3.12/site-packages
Requires: mcp, pydantic, tzdata, tzlocal
Required-by:) | invalid-timezone-is-an-error | `get_current_time` | 8 / 8 | 100.0% | 100.0% | 100.0% | 0.0% [0.0%, 0.0%] | supported | 0.0% / 100.0% / 100.0% | yes |

<!-- RESULTS:END -->

Rendered by `mcpprobe report` from `results/*.json`. Environments (server command, scripted tasks, graders) are committed under `envs/` and were pre-registered in [docs/PREREGISTRATION.md](docs/PREREGISTRATION.md) section 7 before the first run. A flagged task is one a degenerate caller can pass: for the `expect_error` tasks that is by construction (any wrong argument is also an error); for the two git tasks it is a finding about the environment, the server accepts an empty `repo_path` and answers for its working directory, so the grader is passable without the right argument and the task is excluded from any leaderboard until the grader pins the repository identity.

## Running

```
mcpprobe run --env envs/mcp-server-time.json --out results/mcp-server-time.json
mcpprobe run --env envs/mcp-server-git.json  --out results/mcp-server-git.json
mcpprobe report
```

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
