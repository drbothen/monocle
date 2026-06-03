---
review_id: ADV-PR36-SEC-P1
pass: 1
target: implementation (security review)
scope: diff-from:develop — scripts/validate_adr_self_consistency.sh + crates/monocle-tui/src/app.rs
pr: fix/wave7-sweep-doc-nits-timing (#36)
reviewer: adversary-agent (fresh context)
date: 2026-06-03
convergence_status: PASS (1 Important finding — fix recommended; 2 Suggestions)
---

# Adversarial Security Review — Pass 1 — PR #36

**Scope:** Two focused areas per the security review brief:
1. `scripts/validate_adr_self_consistency.sh` — new file, 220 lines, PostToolUse hook.
2. `crates/monocle-tui/src/app.rs` — ScrollDown guard refactor (match guard `if !app.sessions.is_empty()`).

**Review mode:** Fresh context. No prior review passes seen. No author explanations loaded.

---

## Finding ADV-PR36-SEC-P1-F1: Hook crashes on non-JSON stdin due to set -e + pipefail (IMPORTANT)

**Location:** `scripts/validate_adr_self_consistency.sh`, line 32

**Code:**
```bash
set -euo pipefail
...
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty')
```

**Description:**

`set -euo pipefail` is active at the point where `jq` parses stdin. If the hook receives malformed JSON (empty string, partial JSON, or any non-JSON input — e.g., a different hook invocation format), `jq` exits with code 4 (parse error). With `pipefail`, the pipeline `echo "$INPUT" | jq -r ...` returns jq's exit code (4). With `set -e`, the failed command substitution `FILE_PATH=$(...)` triggers immediate script termination.

The script exits non-zero (4) **without emitting the expected blocking JSON envelope** (`{"decision":"block","reason":"..."}`). Claude Code receives an unexpected non-zero exit from the hook with no structured output — the hook has crashed, not issued a deliberate decision.

**Impact:**

- Claude Code's hook system behavior on unexpected non-zero exit without JSON is undefined by this codebase — depending on Claude Code's hook runner implementation, this may block the tool operation, produce an error in the UI, or silently fail.
- An attacker who can cause the stdin payload to be non-JSON (e.g., by corrupting the IPC message) would cause the hook to crash on every ADR write, permanently disrupting the hook.
- More practically: if Claude Code changes its hook invocation format in a future release, this script will crash silently instead of gracefully exiting 0 (pass).

**Severity:** IMPORTANT — hooks that crash unexpectedly have undefined behavior in the host system; the script should be defensive.

**Fix:**

```bash
# Replace line 32 with:
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || FILE_PATH=""
```

Or alternatively, wrap the entire jq call:

```bash
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null || true)
```

Either form prevents `set -e` from triggering on jq parse failure, allowing the subsequent `[[ -z "$FILE_PATH" ]]` guard (line 35) to exit 0 gracefully.

---

## Finding ADV-PR36-SEC-P1-F2: printf '%b' interprets backslash sequences from file content (SUGGESTION)

**Location:** `scripts/validate_adr_self_consistency.sh`, line 212

**Code:**
```bash
printf '%b' "$ERRORS" >&2
```

**Description:**

The `ERRORS` variable is assembled from file content. Specifically, `$label` in Check 1 is derived from `grep -oE '\*\*v?[0-9]+\.[0-9]+[^*]*\*\*'` applied to a line of the ADR file. If a line in the ADR file contains backslash sequences (e.g., `**v1.0\033[31m** ` — a version label with an embedded ANSI escape), `$label` will contain `\033[31m`, which `printf '%b'` will interpret as an ESC character, potentially injecting ANSI color sequences into stderr.

This is NOT command execution — no shell injection is possible. The impact is cosmetic: terminal ANSI injection in the hook's stderr output. On CI runners that capture stderr as plain text, the escape codes appear as literal characters in the log.

**Severity:** SUGGESTION — cosmetic terminal pollution; not a code-execution risk.

**Fix:** Replace `printf '%b'` with `printf '%s\n'` for the ERRORS output:

```bash
# Replace line 212:
printf '%s\n' "$ERRORS" >&2
```

Note: the `\n` separators in `$ERRORS` are literal two-character sequences `\n` (not newlines), built by `ERRORS="${ERRORS}...message...\n"`. Switching to `printf '%s\n'` would print them as `\n` literals. To preserve newline formatting, either use `echo -e` or build `$ERRORS` with actual newlines (using `$'...'` quoting). The safest non-injection alternative:

```bash
while IFS= read -r err_line; do
  echo "$err_line" >&2
done <<< "$(printf '%b' "$ERRORS")"
```

Or, since the only intended backslash sequence is `\n`, use a controlled substitution:

```bash
echo "${ERRORS//$'\\n'/$'\n'}" >&2
```

---

## Finding ADV-PR36-SEC-P1-F3: Check 3 grep pipeline fragility under set -e (SUGGESTION)

**Location:** `scripts/validate_adr_self_consistency.sh`, lines 183–187

**Code:**
```bash
if echo "$line" | grep -qE '^[0-9]+\. '; then
  current_num=$(echo "$line" | grep -oE '^[0-9]+' | head -1)
```

**Description:**

The second `grep -oE '^[0-9]+'` runs inside a command substitution `$(...)` which is subject to `set -e`. The `| head -1` pipeline after it means `pipefail` applies to the grep exit code. If `grep -oE` exits 1 (no match — theoretically impossible given the outer guard, but structurally present), `set -e` would terminate the script non-zero.

In practice this is unreachable: the outer `grep -qE '^[0-9]+\. '` guard ensures every line entering the inner branch matches `^[0-9]+`, so `grep -oE '^[0-9]+'` will always match. However, the structural fragility is present and could cause surprising behavior if the outer condition changes.

**Severity:** SUGGESTION — not exploitable in current logic; structural hardening recommended.

**Fix:**

```bash
current_num=$(echo "$line" | grep -oE '^[0-9]+' | head -1 || echo "0")
```

---

## Non-Findings (Confirmed Safe)

**Shell injection via FILE_PATH:** NOT present. `FILE_PATH` is extracted via `jq -r` (jq parses JSON and outputs the value as a string — no shell re-interpretation). `FILE_PATH` is used exclusively in:
- `[[ ! -f "$FILE_PATH" ]]` — properly quoted test
- `case "$FILE_PATH" in` — properly quoted case expression  
- `done < "$FILE_PATH"` — properly quoted redirect
- `basename "$FILE_PATH"` — properly quoted argument

No path is passed to `eval`, `exec`, or any command that would interpret its content as shell syntax. **CLEAR.**

**python3 sys.argv[1] injection from `$line`:** NOT present. The python3 invocation is `python3 -c "...code..." "$line"`. The line content from the file is passed as `argv[1]`, which the Python code accesses via `sys.argv[1]`. The Python code uses `re.findall`, `re.finditer`, and string operations — no `eval`, no `exec`, no `subprocess`. Even if `$line` contains shell metacharacters, they are inside double quotes and passed as a single argument to python3. **CLEAR.**

**jq `'.tool_input.file_path // empty'` safety:** NOT present. The `jq -r` with `//empty` safely handles missing keys (returns empty string). jq does not execute shell code. The `--rawfile` / `--arg` distinction is not relevant here (simple field access). **CLEAR.**

**`case` glob injection:** NOT present. `case "$FILE_PATH" in */.factory/specs/architecture/adr/ADR-*.md)` uses glob patterns in a `case` statement — not shell expansion. The `*` and `?` in the patterns match against the literal value of `$FILE_PATH`. An attacker who controls the file path could at most influence which `case` branch is taken; since only ADR files are allowed through, a non-ADR path falls to `*) exit 0`. **CLEAR.**

**`set -euo pipefail` correctness:** Correct for all paths EXCEPT the jq failure mode documented in F1. All other variable expansions are initialized before use (`set -u` safe). The `if $in_code_block` / `if $in_trace_section` idiom (using boolean strings as commands) works correctly with `set -e` because `if` conditions suppress the -e flag. **CLEAR (with F1 exception).**

**Exit codes:** Exit 0 (pass/non-ADR) and exit 2 (block) are correctly assigned. The blocking JSON envelope is only emitted when `$ERRORS` is non-empty — no false positives from the report section. **CLEAR.**

**grep -P fallback path:** `echo "$line" | grep -qP '...' 2>/dev/null` — `$line` is passed as stdin (not as a shell argument), so it cannot inject shell code into the grep invocation. PCRE `grep -P` failure (systems without PCRE support, e.g., macOS grep) exits 2; since this is inside an `if` condition, `set -e` does not trigger. The `2>/dev/null` suppresses the "invalid option" error message. **CLEAR.**

---

## ScrollDown Guard Refactor Analysis

**Location:** `crates/monocle-tui/src/app.rs`, ~line 2261

**Change:** `AppMode::Dashboard { focused: FocusSnapshot::Sessions }` → `AppMode::Dashboard { focused: FocusSnapshot::Sessions } if !app.sessions.is_empty()`

**Panic analysis:**

The old code had an explicit `if len > 0 { }` inner guard. The new code hoists this into a match arm guard. The arithmetic `(i + 1).min(len - 1)` and `app.sessions.get(next)` are only reached when `!app.sessions.is_empty()` — meaning `app.sessions.len() >= 1`. Therefore:

- `len - 1` where `len: usize` and `len >= 1` → **no underflow**. Safe.
- `(i + 1)` where `i` is the previously selected index from a list of length `len >= 1` → `i` is at most `len - 1`, so `i + 1` is at most `len`. No overflow risk for realistic list sizes. Safe.
- `app.sessions.get(next)` returns `Option<&Session>` — no panic, returns `None` for out-of-range. Safe.

**Fallthrough semantics:**

The `_` arm now handles three cases:
1. EventRibbon focus (original intent)
2. Non-Dashboard mode
3. Dashboard/Sessions focus with empty sessions list (new case)

Case 3 calls `scroll_ribbon_down` on an empty (or non-empty) event ribbon. This is semantically reasonable — when the session list is empty, there is nothing to cursor-navigate, and scrolling the ribbon is acceptable. **No behavioral regression.**

**Symmetry with ScrollUp:** The ScrollUp arm (`~line 2297`) was also refactored with the same `if !app.sessions.is_empty()` guard. The analysis is symmetric. **CLEAR.**

**Verdict: NO new panic paths. NO index-out-of-bounds. Fallthrough semantics are correct.**

---

## Convergence Assessment

| Finding | Severity | Status |
|---------|----------|--------|
| F1: jq crash on non-JSON stdin | IMPORTANT | OPEN — fix recommended |
| F2: printf %b ANSI injection | SUGGESTION | OPEN — cosmetic only |
| F3: Check3 grep pipeline fragility | SUGGESTION | OPEN — structural hardening |
| Shell injection (FILE_PATH) | N/A | CLEAR |
| python3 argv injection | N/A | CLEAR |
| ScrollDown panic/OOB | N/A | CLEAR |
| ScrollDown fallthrough semantics | N/A | CLEAR |

**Pass 1 verdict:** PR #36 is **SAFE TO MERGE** with one recommended improvement. The IMPORTANT finding (F1) is a defensive robustness issue, not an exploitable vulnerability in the current deployment context (Claude Code hook runner with structured JSON stdin). The two SUGGESTION findings are cosmetic/structural. None of the checked items constitute shell injection, code execution, or index-out-of-bounds vulnerabilities.

**Minimum convergence:** This is a security review with targeted scope (2 areas), not a convergence loop. Single pass sufficient for the stated security questions. If convergence loop is desired, minimum 3 clean passes required per skill protocol.
