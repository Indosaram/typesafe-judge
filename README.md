# typesafe-judge

Fast, typed AI decision engine for coding tasks, verification gates, and agent workflows, powered by **TypeSafe System One (`Jev`)**.

Instead of waiting for slow, expensive, and uncalibrated text generation from autoregressive LLMs (System Two), `typesafe-judge` evaluates code state, choices, diffs, and hypotheses to return **calibrated probabilities**, **confidence scores**, and **rigidly typed answers** in milliseconds.

## Features

- **Blazingly Fast**: Returns in ~200–500ms with zero token generation latency.
- **Calibrated Probabilities**: True mathematical probabilities ($0.0 \dots 1.0$) and confidence metrics.
- **Git Diff Gate**: Automatic CI / pre-commit verification checking prompt fulfillment, scope cleanliness, and regression risk.
## Output Modes (Token Optimization)

By default, `typesafe-judge` outputs a **dense, single-line format** specifically optimized to save LLM context window tokens:

- **Default (Compact)**: 1 single line containing all key decision metrics (~10–25 tokens instead of 250+ tokens).
- **`--quiet` / `-q`**: Zero-noise output returning only the raw value (e.g. `0.990` or `arc_swap`). Ideal for shell script variables.
- **`--json`**: Structured JSON payload for programmatic consumption.
- **`--pretty`**: Visual ANSI colors with Unicode progress bars for human interactive terminals.
- **Exit Code Integration**: Noul and Diff commands exit with code `0` (pass) or `1` (fail) based on thresholds, making them one-line gates for bash pipelines.

---

## Installation

### 1. CLI Binary

From source with Cargo:

```bash
cargo install --path .
```

Or build the release binary:

```bash
cargo build --release
cp target/release/typesafe-judge ~/.cargo/bin/ # or ~/.omo/bin/
```

### 2. Agent Skill (`typesafe-ai`)

Install this skill into any coding agent (Claude Code, Cursor, OMO, Codex, Windsurf) via `skills.sh`:

```bash
# In Claude Code:
claude plugin marketplace add Indosaram/typesafe-judge
claude plugin install typesafe@typesafe-ai

# In OMO / Codex / Cursor / other agents:
npx skills add Indosaram/typesafe-judge --skill typesafe-ai -g
```

### Configuration

Export your TypeSafe API key:

```bash
export TYPESAFE_API_KEY="your_api_key_here"
```

Or pass `--api-key <KEY>` to any command.

---

## Commands

### 1. `choice` — Multi-Option Architecture & Decision Selection

Picks the highest-probability option among candidates and returns the complete probability distribution.

```bash
typesafe-judge choice \
  --state "High read / low write catalog cache in Rust with 10k RPS" \
  --instructions "Which concurrency pattern best fits this scenario?" \
  --option "arc_swap:Atomic pointer swap on write with lock-free reads" \
  --option "rwlock:Standard std::sync::RwLock" \
  --option "mutex:Standard std::sync::Mutex"
```

**Quiet mode (outputs only winner key, perfect for shell assignment):**
```bash
BEST_PAT=$(typesafe-judge choice -s "..." -o "arc_swap:..." -o "rwlock:..." -q)
echo "Selected: $BEST_PAT"
```

### 2. `noul` — Boolean Condition & Probability Gate

Evaluates a yes/no condition and returns the calibrated probability $P(\text{yes})$.

```bash
typesafe-judge noul \
  --state "$(cat src/lib.rs)" \
  --instructions "Does this module handle graceful shutdown properly?" \
  --true-desc "Explicitly intercepts SIGINT/SIGTERM and cancels child tasks" \
  --false-desc "Drops immediately or leaves background workers orphaned" \
  --threshold 0.70
```

*Note: Exits with code `0` if $P(\text{yes}) \ge \text{threshold}$, and code `1` otherwise.*

### 3. `score` — Rubric-Based Level Scoring

Rates state along descriptive, ordered levels.

```bash
typesafe-judge score \
  --state "$(git diff)" \
  --instructions "Rate the potential blast radius of these changes" \
  --level "Negligible: localized fix with zero external impact" \
  --level "Moderate: touches shared interface, requires client testing" \
  --level "Severe: breaking database schema or wire protocol change"
```

### 4. `diff` — Git Verification Gate

Pre-commit or CI gate that analyzes the current git diff against a prompt.

```bash
typesafe-judge diff --prompt "Refactor user authentication to support TOTP"
```

Queries Jev for:
1. `fulfills_prompt`: Does the diff satisfy the prompt? ($P \ge 0.65$)
2. `is_scope_clean`: Is the diff free of unrelated changes and accidental churn? ($P \ge 0.50$)
3. `regression_risk`: Score ($< 1.5$)

Exits `0` on pass, `1` on fail.

### 5. `eval` — Speculative Fan-Out Batch

Executes an arbitrary multi-question JSON payload in a single parallel API call.

```bash
typesafe-judge eval --file request.json
# or
cat request.json | typesafe-judge eval
```

---

## Agent Integration

Add `typesafe-judge` to your agent workflows (OMO, Claude Code, Codex, CI):

```bash
# Verify changes before asking for human review
typesafe-judge diff -p "Fix issue #42" || echo "Fix failed verification gate"
```

---

## License

MIT © Indo Yoon
