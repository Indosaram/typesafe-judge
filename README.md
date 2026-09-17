# typesafe-judge

Fast, typed AI decision engine for coding tasks, verification gates, and agent workflows, powered by **TypeSafe System One (`Jev`)**.

Instead of waiting for slow, expensive, and uncalibrated text generation from autoregressive LLMs (System Two), `typesafe-judge` evaluates code state, choices, diffs, and hypotheses to return **calibrated probabilities**, **confidence scores**, and **rigidly typed JSON answers** in milliseconds.

## Features

- **Blazingly Fast**: Returns in ~200–500ms with zero token generation latency.
- **Native JSON First**: Outputs clean, structured JSON by default—100% machine-readable with zero ad-hoc formatting translation.
- **Git Diff Gate**: Automatic CI / pre-commit verification checking prompt fulfillment, scope cleanliness, and regression risk with structured pass/fail verdict and exit codes.
- **Automatic Key Resolution**: Seamless 4-tier fallback: `--api-key` → `TYPESAFE_API_KEY` env → `.env` file → `~/.config/typesafe/api_key`.
- **Transient Error Retry**: Automatic exponential backoff on HTTP 429 (rate limits) and 529 (overload).
- **Human Visual Mode**: Pass `--pretty` when you want colorful ANSI progress bars in an interactive terminal.

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

---

## Output Modes

`typesafe-judge` is designed first and foremost for **agents and automated workflows**:

1. **Default (Standard JSON)**: Outputs the full, typed TypeSafe JSON response. Zero translation loss, zero hallucination risk, native for LLM parsing.
2. **`--compact`**: Outputs single-line minified JSON for minimal token usage.
3. **`--quiet` / `-q`**: Outputs only the scalar winner or metric (e.g. `sqlite` or `0.990`) for shell script variable assignment.
4. **`--pretty`**: Visual ANSI colors with progress bars for human eyes in interactive terminals.

---

## Commands

### 1. `choice` — Multi-Option Architecture & Decision Selection

Picks the highest-probability option among candidates and returns the complete probability distribution.

```bash
typesafe-judge choice \
  --state "High read / low write catalog cache in Rust with 10k RPS" \
  --instructions "Which concurrency pattern best fits this scenario?" \
  --option "arc_swap:Atomic pointer swap on write with lock-free reads" \
  --option "rwlock:Standard std::sync::RwLock with reader contention" \
  --option "mutex:Standard std::sync::Mutex"
```

**JSON Output:**
```json
{
  "model": "jev-1.13.0",
  "answers": {
    "choice": {
      "type": "choice",
      "choice": "arc_swap",
      "confidence": 0.99,
      "probabilities": {
        "arc_swap": 1.0,
        "mutex": 0.0,
        "rwlock": 0.0
      }
    }
  },
  "usage": { "input_tokens": 316, "output_tokens": 34 }
}
```

### 2. `noul` — Boolean Condition & Probability Gate

Evaluates a yes/no condition and returns the calibrated probability $P(\text{yes})$.

```bash
typesafe-judge noul \
  --file src/lib.rs \
  --instructions "Does this module handle graceful shutdown properly?" \
  --threshold 0.70
```

*Note: Exits with code `0` if $P(\text{yes}) \ge \text{threshold}$, and code `1` otherwise.*

### 3. `score` — Rubric-Based Level Scoring

Rates state along descriptive, ordered levels.

```bash
typesafe-judge score \
  --file docs/migration.md \
  --instructions "Rate operational migration complexity" \
  --level "Trivial" --level "Moderate" --level "High complexity"
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

Returns a structured `gate` envelope with verdict and metrics, exiting `0` on pass and `1` on fail:
```json
{
  "gate": {
    "passed": true,
    "verdict": "PASS",
    "metrics": {
      "fulfills_prompt": 0.92,
      "is_scope_clean": 0.88,
      "regression_risk": 0.59
    },
    "elapsed_ms": 495
  },
  "model": "jev-1.13.0",
  "answers": { ... }
}
```

---

## License

MIT © Indo Yoon
