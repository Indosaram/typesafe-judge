---
name: typesafe-ai
license: MIT
description: >
  Build and judge AI-powered decisions with TypeSafe System One (Jev) via the
  fast typesafe-judge CLI. Returns calibrated probabilities, confidence scores,
  and typed judgments for architecture paths, hypothesis ranking, and git diff gates.
  Use when an agent needs fast, typed decisions instead of slow LLM text generation.
---

# Build with TypeSafe (via `typesafe-judge` CLI)

TypeSafe makes units of AI intelligence usable as fast, typed decision primitives.
Its **System One models**, including **Jev**, evaluate state and return calibrated
probabilities and typed decisions in milliseconds rather than generating text.

**EXECUTION DIRECTIVE**:
Do NOT write ad-hoc HTTP `fetch` scripts or install heavyweight SDKs.
Execute the globally installed **`typesafe-judge`** CLI binary directly via tool execution.

---

## 1. Fast Decision Primitives via CLI

### `choice` — Multi-Candidate Selection & Architecture Gates
Picks the best option among candidates and outputs the winning choice, calibrated probability distribution, and confidence.

```bash
typesafe-judge choice \
  --state "High read / low write catalog cache in Rust with 10k RPS" \
  --instructions "Which concurrency pattern best fits this scenario?" \
  --option "arc_swap:Atomic pointer swap on write with lock-free reads" \
  --option "rwlock:Standard std::sync::RwLock with reader contention" \
  --option "mutex:Standard std::sync::Mutex"
```

**Quiet mode (returns only the winning key for shell variables / branching):**
```bash
BEST=$(typesafe-judge choice -s "..." -o "a:..." -o "b:..." -q)
```

**Decision Rule for Agents:**
- `confidence >= 0.85`: High confidence. Proceed immediately with implementation.
- `confidence < 0.60`: Low confidence / close trade-off. Ask the user for steering (`ask_user_question`).

---

### `noul` — Boolean Condition & Calibrated Probability Gate
Evaluates a yes/no condition and returns the calibrated probability $P(\text{yes}) \in [0.0, 1.0]$.

```bash
typesafe-judge noul \
  --state "$(cat src/lib.rs)" \
  --instructions "Does this module handle graceful shutdown properly?" \
  --true-desc "Explicitly intercepts SIGINT/SIGTERM and cancels child tasks" \
  --false-desc "Drops immediately or leaves background workers orphaned" \
  --threshold 0.70
```

**Exit Code Integration:**
- Exits with status `0` if $P(\text{yes}) \ge \text{threshold}$ (default: 0.50).
- Exits with status `1` if $P(\text{yes}) < \text{threshold}$.
- Ideal for one-line assertion chains in terminal/bash workflows.

---

### `score` — Rubric-Based Level Scoring
Evaluates state against ordered, descriptive levels and computes a probability-weighted score.

```bash
typesafe-judge score \
  --state "$(git diff)" \
  --instructions "Rate the potential blast radius of these changes" \
  --level "Negligible: localized fix with zero external impact" \
  --level "Moderate: touches shared interface, warrants careful review" \
  --level "Severe: breaking database schema or wire protocol change"
```

**Quiet mode:** Returns only the numeric score (e.g. `1.15`).

---

### `diff` — Pre-Commit & Verification Gate
Automatically evaluates the current git diff against the user's task prompt.

```bash
# Evaluates working tree diff
typesafe-judge diff --prompt "Implement TOTP token validation"

# Or evaluate staged diff / piped diff
git diff --staged | typesafe-judge diff --prompt "..."
```

Evaluates 3 dimensions simultaneously:
1. `fulfills_prompt`: Does the diff directly satisfy requirements? ($P \ge 0.65$)
2. `is_scope_clean`: Is the diff disciplined with zero scope creep? ($P \ge 0.50$)
3. `regression_risk`: Score ($< 1.5$)

Exits with code `0` on pass, `1` on failure. Run this before finishing coding turns!

---

### `eval` — Speculative Fan-Out Batch
Sends dozens of questions in a single parallel API request.

```bash
typesafe-judge eval --file request.json
# or
cat request.json | typesafe-judge eval
```

---

## 2. Configuration & Key Management

The CLI automatically reads the API key from:
1. `TYPESAFE_API_KEY` environment variable (pre-configured in `~/.zshenv`, `~/.zshrc`, `launchctl`, and `.env`)
2. Fallback key file: `~/.config/typesafe/api_key`
3. Optional CLI override: `--api-key <KEY>`
