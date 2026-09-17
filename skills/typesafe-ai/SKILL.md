---
name: typesafe-ai
license: MIT
description: >
  Execute high-confidence coding decisions with TypeSafe System One (Jev) via
  the typesafe-judge CLI. Use for architecture choices, debugging hypothesis
  ranking, and git diff verification gates returning verbatim raw JSON.
---

# TypeSafe AI (`typesafe-judge` CLI)

TypeSafe makes units of intelligence usable as fast, typed decision primitives.
Its **System One models**, including **Jev**, evaluate state and return calibrated
probabilities and typed decisions in milliseconds rather than generating text.

**PRIMARY DIRECTIVE**:
Do NOT write ad-hoc HTTP `fetch` scripts or install SDKs.
Invoke the globally installed **`typesafe-judge`** CLI binary directly via tool execution.
All commands output the **verbatim raw JSON response directly from the TypeSafe API** by default.

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
Picks the best candidate and outputs verbatim TypeSafe wire JSON:

```bash
typesafe-judge choice \
  --file src/cache.rs \
  --instructions "Which concurrency pattern best fits this high-read low-write cache?" \
  --option "arc_swap:Lock-free reads with atomic pointer swap on infrequent writes" \
  --option "rwlock:Standard RwLock with atomic read-counter contention" \
  --option "actor:Channel-based single worker actor"
```

**Verbatim Wire Output:**
```json
{"model":"jev-1.13.0","answers":{"choice":{"type":"choice","choice":"arc_swap","confidence":0.99,"probabilities":{"arc_swap":1.0,"rwlock":0.0,"actor":0.0}}},"usage":{"input_tokens":316,"output_tokens":34}}
```

**Confidence Routing Rules:**
- `confidence >= 0.85`: High confidence. Proceed immediately with implementation.
- `0.60 <= confidence < 0.85`: Moderate confidence. Proceed with winner.
- `confidence < 0.60`: Low confidence / neck-and-neck trade-off. Stop and ask the user (`ask_user_question`).

---

### B. `noul` — Boolean Condition & Probability Gate
Evaluates a yes/no condition and returns the calibrated probability $P(\text{yes}) \in [0.0, 1.0]$.

```bash
typesafe-judge noul \
  --file src/lib.rs \
  --instructions "Does this module handle graceful shutdown properly?" \
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

---

### D. `diff` — Pre-Commit Verification Gate
Analyzes the git diff against the user's task prompt.

```bash
# Evaluates working tree diff (including untracked files with --untracked)
typesafe-judge diff --prompt "Implement TOTP token validation" --untracked

# Evaluates specific commit range or file path
typesafe-judge diff --prompt "Fix memory leak" --revision "HEAD~1..HEAD" --path "src/core/"
```

Outputs verbatim raw JSON to stdout. Exits with status `0` on pass, `1` on failure (failure diagnostics output to stderr).

---

## 3. Output Modes
- **Default**: 100% Verbatim raw wire JSON directly from the TypeSafe server. Compact, zero translation loss, zero hallucination.
- **`-q` / `--quiet`**: Returns only the scalar value (e.g. `arc_swap` or `0.990`) for shell script variable assignment.
- **`--pretty`**: Visual ANSI progress bars for human eyes in interactive terminals.

---

## 4. Key Configuration
`typesafe-judge` automatically resolves your API key in this order:
1. `--api-key <KEY>`
2. `TYPESAFE_API_KEY` environment variable
3. Current or parent directory `.env` file
4. `~/.config/typesafe/api_key` or `~/.typesafe/api_key`
