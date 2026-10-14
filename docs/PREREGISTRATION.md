# mcpprobe pre-registration

**Locked before any real MCP server is probed.** This file is append-only
after the first `kind: run` result is committed. Hypotheses, thresholds, and
the family-wise correction cannot be revised after seeing results. Any
revision is a new numbered section at the bottom of this file, with the date
and the reason, and it only applies to runs committed after that section.

## 1. Scope

The population under study is **publicly listed MCP server implementations**
that advertise a stable JSON-RPC surface over stdio or HTTP+SSE, as of the
first committed result in this repo. The reliability metric is per-tool-call
pass@k and pass^k against scripted deterministic callers and scripted
deterministic graders. No LLM-in-the-loop. No LLM judge.

## 2. Primary hypothesis

For a server `s` on a task `t`, let `c(s,t,n)` be the number of passes in
`n` independent trials under a fixed callers-and-graders setup. We report:

- `pass@k(s,t) = 1 - C(n-c,k)/C(n,k)`
- `pass^k(s,t) = C(c,k)/C(n,k)`

per the unbiased combinatorial estimators (Chen et al., 2021). We use **n=8**
and **k=4** for the headline leaderboard and report pass@1 for reference.

H1 (per-server, pre-registered): for a mature MCP server implementation, the
gap `pass@k - pass^k` on tool-call tasks is **less than or equal to 10 pp
(95% bootstrap CI upper bound)**.

Rejecting H1 means the server is less consistent than its average success
rate implies: it passes often but not reliably.

## 3. Degenerate-baseline audit (gating)

Before any pass@k is reported for a server, three degenerate callers are run
against it:

- **no-op** — the caller sends no tool call and the grader is given the
  empty transcript.
- **random-arg** — the caller sends the right tool name with uniformly
  sampled argument values drawn from the declared schema's type ranges.
- **schema-mismatch** — the caller sends the right tool name with arguments
  that pass JSON type-checking but violate the declared constraints.

A server is **flagged** when any degenerate baseline scores pass@1 >= 5%
on an environment. Flagged servers are listed in the report with a warning
badge and are **excluded from the leaderboard** until the environment is
fixed. This is the environment-authored check, not the server's fault:
flagging means the task is passable by accident and the pass@k number does
not measure what we want it to.

## 4. Multiple-comparisons correction

When reporting across servers for the same environment, p-values from
paired tests are corrected with the Holm-Bonferroni method within an
environment family. Across environments we use unadjusted CIs and never
pool results.

## 5. What this repo refuses to do

- No LLM judges. All graders are scripted. A trial either passes a
  deterministic check or does not.
- No comparing against a server's own self-reported scores.
- No numbers in README, docs, or leaderboard that are not produced by a
  committed `results/*.json` file. The report CLI reads these files; no
  numbers are hand-written.
- No tuning on the TEST split. The TEST split is read once per final
  comparison.

## 6. Threats to validity

- **Environment correctness is the author's responsibility.** The scripted
  callers and graders in this repo are the specification of each task. A
  bug in a grader is reported as a known issue in `docs/THREATS_TO_VALIDITY.md`.
- **Server population is public-only.** Private or vendor-restricted MCP
  servers are not represented.
- **Pass@k is a point estimate at one `n`.** We report its 95% bootstrap CI;
  with `n < 4` the CI is wide and the metric is not reliable.
