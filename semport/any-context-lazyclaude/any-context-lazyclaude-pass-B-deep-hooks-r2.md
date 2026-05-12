# Pass B Deepening — `internal/core/config/hooks.go` Round 2

**Subsystem:** `internal/core/config/hooks.go`
**Round:** 2 (nitpick verification + edge-case contracts + convergence assessment)
**Round 1 outcome:** SUBSTANTIVE — produced canonical schema (§4), endpoint matrix (§6), restart-resilience sequence (§7), wire-byte-compatibility checklist (§12), 25 new contracts BC-HOOK-007..031, 2 P1 findings, 1 P2 finding.

**Round 2 goal:** Examine the 6 remaining gaps flagged in round 1 §14, contract-ize the load-bearing ones, confirm the trivial ones don't change the model. Per protocol minimum-2-rounds rule, this round is mandatory regardless of expected novelty.

---

## 1. Re-reading the inline JS at character level (gap 1 + gap 2 + gap 3)

Round 1 read each hook command const as a unit. Round 2 walks the JS character-by-character to extract control flow, error handling, and edge cases that round 1's structural read missed.

### Single line of the preToolUseHookCommand (hooks.go:31), expanded for readability

```js
node -e "
let d='';
process.stdin.on('data', c => d += c);
process.stdin.on('end', () => {
  try {
    const i = JSON.parse(d);
    const http = require('http');
    // resolveServerJS expansion:
    let srvPort = null, srvToken = null;
    const fs = require('fs'), path = require('path'), home = require('os').homedir();
    const lockDir = path.join(home, '.claude', 'ide');
    const locks = fs.readdirSync(lockDir).filter(f => f.endsWith('.lock'));
    let best = null;
    for (const f of locks) {
      try {
        const lk = JSON.parse(fs.readFileSync(path.join(lockDir, f), 'utf8'));
        const p = parseInt(f, 10);
        try {
          process.kill(lk.pid, 0);
          if (!best || p > best.port) best = { lock: lk, port: p };
        } catch {}
      } catch {}
    }
    if (best) { srvPort = best.port; srvToken = best.lock.authToken; }
    // end resolveServerJS
    if (!srvPort) { console.log(d); return; }
    const body = JSON.stringify({
      type: 'tool_info',
      pid: process.ppid,
      tool_name: i.tool_name || '',
      tool_input: i.tool_input || {}
    });
    const req = http.request({
      hostname: '127.0.0.1',
      port: srvPort,
      path: '/notify',
      method: 'POST',
      timeout: 300,
      headers: {
        'Content-Type': 'application/json',
        'Content-Length': Buffer.byteLength(body),
        'X-Claude-Code-Ide-Authorization': srvToken
      }
    });
    req.on('error', () => {});
    req.on('timeout', () => { req.destroy(); });
    req.write(body);
    req.end();
  } catch {}
  console.log(d);
})
"
```

This expansion is the source of truth for round 2's edge-case contracts.

---

## 2. Edge case contracts derived from the expanded JS

### BC-HOOK-032: Malformed stdin JSON does NOT prevent stdin echo for PreToolUse

**Preconditions:** Claude Code pipes malformed JSON to a PreToolUse hook (theoretically impossible but defensible behavior).
**Postconditions:** `JSON.parse(d)` throws → caught by outer `try{}catch{}` (hooks.go:31). The outer `console.log(d)` at end-of-block runs unconditionally (it's AFTER the catch, not inside the try). **Stdin is echoed even on parse failure.** Claude Code's tool call proceeds.
**Evidence:** hooks.go:31 — the `}catch{}console.log(d);` sequence. The `console.log(d)` is outside the try/catch.
**Confidence:** HIGH
**Implication:** PreToolUse is **doubly fail-open**: (a) no server found → echo stdin; (b) malformed input → echo stdin. Both paths converge on "let Claude Code proceed". This is a deliberate-looking pattern.

### BC-HOOK-033: Malformed stdin JSON silently drops the hook for the OTHER 4 hooks (Notification, Stop, SessionStart, UserPromptSubmit)

**Preconditions:** Same as above for non-PreToolUse hooks.
**Postconditions:** Outer `try{...}catch{}` swallows. No `console.log(d)` at end-of-block for these four (verified: hooks.go:35, 38, 41, 44 end with `}catch{}})"` — note no trailing `console.log(d)`). Hook drops; no error visible to Claude Code or to lazyclaude.
**Evidence:** hooks.go:31 (has trailing `console.log(d)`) vs hooks.go:35, 38, 41, 44 (no trailing).
**Confidence:** HIGH
**Implication:** Asymmetric fail-mode is intentional: PreToolUse must not block Claude Code; the others are observability and may safely drop.

### BC-HOOK-034: `parseInt(f, 10)` filename parsing gracefully handles non-numeric lock files via NaN comparison

**Preconditions:** A file like `vscode-abc.lock` exists in `~/.claude/ide/`.
**Postconditions:** `parseInt("vscode-abc.lock", 10)` returns `NaN`. Then:
  - `process.kill(lk.pid, 0)` runs first — if PID is alive, this succeeds.
  - Then `if(!best || p > best.port)` — `p` is `NaN`; `NaN > <anything>` is `false`; so this branch is taken ONLY when `best` is null.
  - If best is null AND p is NaN: `best = {lock: lk, port: NaN}`.
  - On next iteration: `p > best.port` where best.port is NaN → false. So a numeric port can never displace a NaN-port `best`.
  - At end, `srvPort = best.port = NaN`. The subsequent `if(!srvPort)` evaluates `NaN` as falsy → fallback path runs.
**Evidence:** hooks.go:18-19 (filename parse + comparison); ECMAScript spec for NaN comparisons.
**Confidence:** HIGH (verifiable by reasoning about NaN semantics)
**P3 (minor) finding:** If the FIRST lock file enumerated is non-numerically-named AND has an alive PID, all subsequent numeric locks are skipped because `p > NaN === false`. Net: lazyclaude could silently drop hooks if `~/.claude/ide/` contains a non-numeric `.lock` with an alive PID, AND that file is enumerated before any numeric lock.
**Practical risk:** Very low. `~/.claude/ide/` is populated by IDE integrations following the lock-file convention (`<port>.lock`). Non-numeric names are non-standard. But a defensive port should add `if (isNaN(p)) continue;` before the kill check.
**Disposition:** Optional fix for monocle port. Not a P1/P2 because in practice no IDE writes a non-numeric `.lock`.

### BC-HOOK-035: Lock-file READ errors and JSON parse errors are silently skipped

**Preconditions:** A `.lock` file exists but is unreadable (e.g. permission denied for a setuid'd lock from another user) OR is malformed JSON.
**Postconditions:** Inner `try{...const lk = JSON.parse(...)...}catch{}` (hooks.go:16-20) swallows. The loop continues to the next file. No alive-server-search degradation.
**Evidence:** hooks.go:16 (outer try) + hooks.go:20 (catch).
**Confidence:** HIGH

### BC-HOOK-036: `Buffer.byteLength(body)` returns UTF-8 byte length, NOT character count

**Postconditions:** Content-Length header uses UTF-8 byte length. If `tool_input` contains non-ASCII characters (e.g. emoji, CJK), the byte length exceeds the character count. HTTP servers parsing Content-Length get the correct byte count.
**Evidence:** hooks.go:31 etc. `'Content-Length':Buffer.byteLength(body)`. Node.js `Buffer.byteLength(str)` defaults to UTF-8 encoding.
**Confidence:** HIGH
**Implication:** Rust port using `body.len()` on a `String` (byte length) is correct. Using `body.chars().count()` would be wrong.

### BC-HOOK-037: `req.write(body)` then `req.end()` sends body and closes write-side immediately

**Postconditions:** Body is sent in a single write. `req.end()` signals no more data. The hook process then immediately exits (the `.on('end', ...)` callback returns). The TCP connection is half-closed (we won't read response). Response is discarded.
**Evidence:** hooks.go:31 etc. `req.write(body);req.end();` followed by callback return.
**Confidence:** HIGH
**Implication:** Server can return any status code; the hook doesn't read it. Server-side response encoding is purely diagnostic for logs / future protocol use.

### BC-HOOK-038: Two-server-same-port race is structurally impossible (lock-after-bind ordering)

**Preconditions:** Theoretical: two lazyclaude processes try to bind the same port and write a lock file.
**Postconditions:** `net.Listen("tcp", ":<port>")` is atomic at the kernel level — one succeeds, the other gets EADDRINUSE. The losing server returns error from `Start` without ever calling `lock.Write`. So `<port>.lock` always corresponds to the bound server (or a stale entry from a dead one — which `CleanStale` and PID-liveness handle).
**Evidence:** `internal/server/server.go:123-143` — `net.Listen` first, then `lock.Write`.
**Confidence:** HIGH (Go stdlib + POSIX TCP semantics)
**Implication:** The "highest port wins" tie-break never has to handle ties. The implementation is correct even if filename uniqueness gave you two same-port locks, but that's structurally impossible.

---

## 3. WriteHooksSettingsFile atomicity (gap 5 from round 1)

### BC-HOOK-039: `os.WriteFile` for hooks-settings.json is NOT atomic; a torn read by a concurrent Claude Code launch is theoretically possible

**Preconditions:** Two lazyclaude sessions are launched in parallel; both call `WriteHooksSettingsFile(<sameRuntimeDir>)`. Each calls `os.WriteFile(path, data, 0o600)`.
**Postconditions:** `os.WriteFile` on POSIX is implemented as `open(O_WRONLY|O_CREAT|O_TRUNC)` then `write(...)` then `close()`. A concurrent reader (Claude Code, just before it parses settings) sees one of:
  - The complete pre-write content (if the open hasn't started).
  - An empty file (after O_TRUNC, before any write).
  - A partial file (after some writes, before all written).
  - The complete new content (after the final write).
**Practical risk for hooks-settings.json:**
  - Content is **deterministic across writes** — `buildHooksMap()` returns the same map every time. Two parallel writes produce byte-identical content (modulo Go's map ordering randomness — see BC-HOOK-040 below).
  - Even a torn read of identical-content writes ALMOST always yields valid JSON: the JSON encoder writes top-to-bottom, so partial writes give truncated-prefix content that fails to parse. Claude Code's `--settings` loader presumably errors and falls back to no-settings.
  - Risk window: ~1 ms (small file write).
**Mitigation pattern (NOT used by lazyclaude):** Atomic write via `write to <path>.tmp then rename`. POSIX rename is atomic.
**Evidence:** hooks.go:71 (`os.WriteFile` direct).
**Confidence:** HIGH (Go stdlib + POSIX semantics)
**P3 finding:** Not load-bearing because (a) the content is deterministic, (b) parallel session launches are rare in practice, (c) the worst case is one Claude Code session launching without hooks — silent degradation, not corruption. A monocle port may prefer the temp-rename atomic-write pattern for robustness. Optional improvement.

### BC-HOOK-040: Go map iteration randomness causes hooks-settings.json byte-content NON-DETERMINISM between runs

**Postconditions:** `buildHooksMap` returns a `map[string]any` (hooks.go:92-99). When this is JSON-encoded, Go's `encoding/json` sorts map keys alphabetically for objects (verified Go stdlib behavior — `encoding/json/encode.go` `mapEncoder` does `sort.Strings(sv)` for `interface` value maps). So the top-level `hooks` object's keys are emitted in alphabetical order:
```
Notification → PreToolUse → SessionStart → Stop → UserPromptSubmit
```
The same applies to the inner `{"matcher": "*", "hooks": [...]}` objects: alphabetical `hooks, matcher` order.

**Actual byte order (verified by reasoning about Go's json encoder):** Top-level keys sorted alphabetically.

**Evidence:** hooks.go:78-99 (Go maps), Go stdlib `encoding/json` sorts map keys alphabetically when serializing `map[string]any`.
**Confidence:** MEDIUM (relying on Go stdlib documented behavior; not asserted by hooks_test.go).
**Implication for byte-compatibility:** A Rust port using `serde_json` with `BTreeMap` (sorted) OR with explicit struct field order (PascalCase via `#[serde(rename = ...)]`) can match byte order. If the Rust port uses `HashMap`, output order is undefined → may not match.
**Disposition:** Document. For monocle port, use a fixed-order struct (sketched in round 1 §4). This guarantees byte-stable output without relying on encoder details.

---

## 4. The hooks_test.go path-literal assertion gap (gap 6)

### BC-HOOK-041: The hooks_test.go does NOT assert the file path is `<tmpDir>/hooks-settings.json`

**Postconditions:** hooks_test.go:17-19 only asserts:
```go
path, err := config.WriteHooksSettingsFile(tmp)
require.NoError(t, err)
assert.NotEmpty(t, path)
```
The path could be `<tmp>/foo.json` and this test would still pass. The literal filename `hooks-settings.json` (load-bearing for byte-compatibility) is NOT asserted.
**Evidence:** hooks_test.go:14-43 (full test).
**Confidence:** HIGH
**P2 finding:** The test does not verify the canonical filename. A refactor that changed the filename would not break this test. A monocle port that picks a different filename would silently diverge — until end-to-end testing with Claude Code reveals the mismatch.
**Disposition:** A monocle port should ADD an explicit assertion `assert path.ends_with("hooks-settings.json")` to its test. Or, better, derive the filename from a public constant exposed by the config module.

---

## 5. Cross-validation: round 1's contracts against fresh re-read

Round 2 re-walked all 100 LOC of hooks.go. Findings:

- **All 31 BC-HOOK contracts from round 1 verified.** No retractions.
- **All file:line citations resolve correctly.**
- **The 5-hook → endpoint matrix (round 1 §6) is exact.** Reading the inline JS character-by-character confirmed every field, timeout, header.
- **The restart-resilience sequence (round 1 §7) is exact.**
- **The hooks-settings.json schema (round 1 §4) is exact** — adding only the Go-map-iteration-randomness clarification (BC-HOOK-040).

---

## 6. New contracts in round 2

| BC ID | Topic | Confidence | Disposition |
|---|---|---|---|
| BC-HOOK-032 | PreToolUse stdin echo on parse failure | HIGH | Doubly fail-open |
| BC-HOOK-033 | Other 4 hooks silently drop on parse failure | HIGH | Intentional asymmetry |
| BC-HOOK-034 | `parseInt` NaN handling for non-numeric `.lock` files | HIGH | P3 minor; defensive port can add `isNaN` skip |
| BC-HOOK-035 | Lock-file read/parse errors silently skipped | HIGH | Defensive design |
| BC-HOOK-036 | `Buffer.byteLength` is UTF-8 bytes, not chars | HIGH | Port note for Rust |
| BC-HOOK-037 | `req.write` + `req.end` is fire-and-forget; response discarded | HIGH | Port semantics |
| BC-HOOK-038 | Two-server-same-port race structurally impossible | HIGH | Confirms tie-break correctness |
| BC-HOOK-039 | WriteHooksSettingsFile not atomic; torn-read theoretically possible | HIGH | P3; optional temp-rename pattern in monocle |
| BC-HOOK-040 | Go map iteration randomness → byte-stability via alphabetical key sort | MEDIUM | Use fixed-order struct in monocle |
| BC-HOOK-041 | hooks_test.go does NOT assert the canonical filename | HIGH | P2; monocle test should add assertion |

10 new contracts, all edge-case refinements. None change the architectural model. None invalidate the canonical deliverables from round 1.

---

## 7. Delta Summary

- **New contracts added:** BC-HOOK-032..041 (10 contracts) — all edge-case refinements.
- **Round 1 contracts confirmed:** BC-HOOK-001..031 (31 contracts). No retractions.
- **New findings:**
  - **P2** (BC-HOOK-041): hooks_test.go lacks canonical-filename assertion.
  - **P3** (BC-HOOK-034): NaN-port edge case in lock-file enumeration.
  - **P3** (BC-HOOK-039): Non-atomic write of hooks-settings.json.
- **Canonical deliverables (round 1):** All confirmed exact. No changes.

### Remaining gaps after round 2

None. The 6 gaps flagged in round 1 §14 are all addressed:

1. Highest-port-wins tie-break → BC-HOOK-038 (structurally impossible).
2. Malformed-stdin parsing → BC-HOOK-032, BC-HOOK-033.
3. `parseInt` edge cases → BC-HOOK-034.
4. Lock-file size limit → covered implicitly by BC-HOOK-035 (errors swallowed; no size check; not a realistic risk for owner-writable files).
5. WriteHooksSettingsFile atomicity → BC-HOOK-039.
6. hooks_test.go missing literal filename assertion → BC-HOOK-041.

The model is complete. No further deepening surface visible.

---

## Novelty Assessment

**Novelty: NITPICK.**

Justification: Removing this round's findings would NOT change how monocle is spec'd. Every new contract is an edge-case refinement:

- BC-HOOK-032..037 document specific JS behavior in error / edge paths. None alter the architectural model (the model is "fire-and-forget, fail-open for PreToolUse, fail-closed for others, restart-resilient via lock-file rescan"). These refinements are useful for a porter writing exhaustive tests, but they don't change the spec.

- BC-HOOK-038 confirms structural correctness of the tie-break logic. Doesn't add new behavior; rules out a hypothetical.

- BC-HOOK-039 (non-atomic write) is an optional improvement, not a correctness gap. The deterministic content makes torn reads non-corrupting in practice.

- BC-HOOK-040 (map iteration) is a porting note, not a behavioral discovery. The monocle Rust port using a fixed-order struct is byte-stable; this just documents why.

- BC-HOOK-041 (test gap) is a test-suite recommendation, not a behavioral finding.

**Test of substance:** Would removing round 2 change the spec? No — the spec already has the canonical schema (§4), the matrix (§6), the sequence (§7), and the checklist (§12) from round 1. Round 2 adds defensive details a careful porter benefits from but doesn't change what they build.

## Convergence Declaration

**Pass B-deep-hooks has converged.** Round 1 was SUBSTANTIVE (produced canonical deliverables). Round 2 is NITPICK (edge-case verification with no architectural impact). Per the convergence protocol, two rounds with novelty trajectory SUBSTANTIVE → NITPICK satisfies the minimum-rounds rule and indicates honest convergence. Stop here.

**Summary of subsystem coverage:**
- 41 BC-HOOK contracts total (6 from Pass 3 + 25 from round 1 + 10 from round 2).
- 3 P1/P2 findings: untested JS content, `LAZYCLAUDE_IDE_DIR` env-var asymmetry, hooks_test.go filename assertion gap.
- 1 P2 finding: hook JS does not filter by `lock.app` (cross-IDE collision risk).
- 2 P3 findings: NaN-port edge case, non-atomic write.
- Canonical deliverables ready for monocle Rust port: schema, matrix, sequence, checklist.

## State Checkpoint

```yaml
pass: B
subsystem: internal/core/config/hooks
round: 2
status: complete
files_scanned:
  - /Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/core/config/hooks.go (full re-read, character-level for inline JS)
prior_pass_files_consulted:
  - any-context-lazyclaude-pass-B-deep-hooks-r1.md (round 1 baseline)
contracts_added: 10 (BC-HOOK-032..041)
contracts_confirmed: 31 (BC-HOOK-001..031 from Pass 3 + round 1)
contracts_total_after_r2: 41
p1_findings_round: 0 (carried forward 2 from r1)
p2_findings_round: 1 (BC-HOOK-041 test-suite gap; carried forward 1 from r1)
p3_findings_round: 2 (BC-HOOK-034 NaN; BC-HOOK-039 non-atomic)
canonical_deliverables: unchanged from r1 (all confirmed exact)
timestamp: 2026-05-11T19:50:00Z
novelty: NITPICK
convergence: CONVERGED — 2 rounds, SUBSTANTIVE → NITPICK trajectory
next_round_needed: false
```
