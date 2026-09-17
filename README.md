# typesafe-judge

Fast, typed AI decision engine for coding tasks, verification gates, and agent workflows, powered by **TypeSafe System One (`Jev`)**.

Instead of waiting for slow, expensive, and uncalibrated text generation from autoregressive LLMs (System Two), `typesafe-judge` evaluates code state, choices, diffs, and hypotheses and outputs the **exact, authentic raw JSON response** from TypeSafe System One in milliseconds.

## Features

- **Blazingly Fast**: Returns in ~200–500ms with zero token generation latency.
- **100% Verbatim Raw Output**: Outputs the exact wire HTTP JSON response directly from `api.typesafe.ai`. Zero re-serialization, zero dropped fields, zero synthetic wrapper noise.
- **Git Diff Gate**: Automatic CI / pre-commit verification checking prompt fulfillment, scope cleanliness, and regression risk with exit codes (`0` on pass, `1` on fail).
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

1. **Default (Verbatim Raw JSON)**: Outputs the exact, unmutated wire JSON response body from TypeSafe API. Upstream sends single-line compact JSON, providing minimal token footprint with complete data fidelity.
2. **`-q` / `--quiet`**: Outputs only the scalar value (e.g. `arc_swap` or `0.980`) for shell script variable assignment.
3. **`--pretty`**: Visual ANSI progress bars for human eyes in interactive terminals.

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

**Output (Exact wire JSON):**
```json
{"model":"jev-1.13.0","answers":{"choice":{"type":"choice","choice":"arc_swap","confidence":0.99,"probabilities":{"arc_swap":1.0,"mutex":0.0,"rwlock":0.0}}},"usage":{"input_tokens":316,"output_tokens":34}}
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

Queries Jev for `fulfills_prompt`, `is_scope_clean`, and `regression_risk`.
Outputs the verbatim wire JSON to stdout and exits with code `0` on pass, `1` on fail.

---

## License

MIT © Indo Yoon
