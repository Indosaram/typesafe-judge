---
name: typesafe-ai
license: MIT
description: >
  Execute high-confidence coding decisions with TypeSafe System One (Jev) via
  the typesafe-judge CLI. Use for architecture choices, debugging hypothesis
  ranking, and git diff verification gates without slow LLM text generation.
---

# TypeSafe AI (`typesafe-judge` CLI)

TypeSafe makes units of intelligence usable as fast, typed decision primitives.
Its **System One models**, including **Jev**, evaluate state and return calibrated
probabilities and typed decisions in milliseconds rather than generating text.

**PRIMARY DIRECTIVE**:
Do NOT write ad-hoc HTTP `fetch` scripts or install SDKs.
Invoke the globally installed **`typesafe-judge`** CLI binary directly via tool execution.

---

## 1. When Agents MUST Proactively Use This Skill

Agents must not guess or rationalize alone in the following 3 situations:

1. **Architecture & Design Forks**: When deciding between 2 or more distinct implementation approaches (e.g. concurrency patterns, state management, API contracts, library selection).
   → Use `typesafe-judge choice`
2. **Debugging Hypothesis Ranking**: When diagnosing crashes, race conditions, or silent failures where multiple potential root causes exist.
   → Use `typesafe-judge score` or `typesafe-judge choice`
3. **Pre-Commit / Pre-Delivery Verification**: Before committing or concluding a task.
   → Use `typesafe-judge diff --prompt "<USER_PROMPT>"`

---

## 2. Decision Primitives & CLI Invocations

### Rule on Multiline Input (CRITICAL)
When state contains code, diffs, or markdown with special characters or quotes, **NEVER** pass raw multiline text into `--state "..."`.
**ALWAYS** use `--file <PATH>` or pipe via stdin:
```bash
# Good: file input
typesafe-judge choice --file path/to/code.rs -o "opt1:desc" -o "opt2:desc"

# Good: pipe input
git diff | typesafe-judge diff --prompt "Refactor auth"
```

---

### A. `choice` — Multi-Candidate Architecture Selection
Picks the best candidate and outputs winning choice, probability distribution, and confidence.

```bash
typesafe-judge choice \
  --file src/cache.rs \
  --instructions "Which concurrency pattern best fits this high-read low-write cache?" \
  --option "arc_swap:Lock-free reads with atomic pointer swap on infrequent writes" \
  --option "rwlock:Standard RwLock with atomic read-counter contention" \
  --option "actor:Channel-based single worker actor"
```

**Quiet mode (extracts winner string for shell scripts / subagent branching):**
```bash
BEST=$(typesafe-judge choice --file ... -o "..." -o "..." -q)
```

**Confidence Routing Rules:**
- `confidence >= 0.85`: High confidence. Proceed immediately with implementation.
- `0.60 <= confidence < 0.85`: Moderate confidence. Log trade-offs and proceed with winner.
- `confidence < 0.60`: Low confidence / neck-and-neck trade-off. Stop and ask the user (`ask_user_question`).

---

### B. `noul` — Boolean Condition & Probability Gate
Evaluates a yes/no condition and returns the calibrated probability $P(\text{yes}) \in [0.0, 1.0]$.

```bash
typesafe-judge noul \
  --file src/lib.rs \
  --instructions "Does this module handle graceful shutdown properly?" \
  --true-desc "Explicitly intercepts SIGINT/SIGTERM and cleans child processes" \
  --false-desc "Drops immediately or leaves background workers orphaned" \
  --threshold 0.70
```

- **Exit Codes**: Returns `0` if $P(\text{yes}) \ge \text{threshold}$ (default: 0.50), and `1` otherwise.
- Chain directly in bash: `typesafe-judge noul ... && git push`

---

### C. `score` — Rubric-Based Level Scoring
Evaluates state against ordered, descriptive levels and computes a probability-weighted score.

```bash
typesafe-judge score \
  --file docs/migration.md \
  --instructions "Rate the operational complexity and blast radius of this migration" \
  --level "Negligible: isolated fix with zero external impact" \
  --level "Moderate: touches shared interface, requires client regression tests" \
  --level "Severe: breaking database schema or wire protocol change"
```

**Quiet mode:** Returns only the numeric score (e.g. `1.15`).

---

### D. `diff` — Pre-Commit Verification Gate
Analyzes the git diff against the user's task prompt.

```bash
# Evaluates working tree diff (including untracked files with --untracked)
typesafe-judge diff --prompt "Implement TOTP token validation" --untracked

# Evaluates specific commit range or file path
typesafe-judge diff --prompt "Fix memory leak" --revision "HEAD~1..HEAD" --path "src/core/"
```

Evaluates 3 dimensions simultaneously:
1. `fulfills_prompt`: Does the diff directly satisfy requirements? ($P \ge 0.65$)
2. `is_scope_clean`: Is the diff disciplined with zero scope creep? ($P \ge 0.50$)
3. `regression_risk`: Score ($< 1.50$)

Exits `0` on pass, `1` on failure.

---

## 3. Workflow Recipes

### Recipe 1: Debugging Hypothesis Elimination
When facing an elusive bug, state the error log and query Jev to rank hypotheses:
```bash
typesafe-judge choice \
  --file /tmp/error.log \
  --instructions "Which root cause is most probable given this stack trace and log context?" \
  --option "race_condition:Async task finishes after socket is closed" \
  --option "buffer_overflow:Incoming frame exceeds 64KiB buffer allocation" \
  --option "auth_timeout:Token expires before handshake completes"
```

### Recipe 2: Verification Gate Before Handoff
Before ending a coding task, run:
```bash
git diff | typesafe-judge diff --prompt "<what user asked>" || {
  echo "Verification failed! Check scope or missing requirements."
}
```

---

## 4. Key Configuration
`typesafe-judge` automatically resolves your API key in this order:
1. `--api-key <KEY>`
2. `TYPESAFE_API_KEY` environment variable
3. Current or parent directory `.env` file
4. `~/.config/typesafe/api_key` or `~/.typesafe/api_key`
