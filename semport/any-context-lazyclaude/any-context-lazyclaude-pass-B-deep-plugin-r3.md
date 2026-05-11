# Pass B Deepening — `internal/plugin/` Subsystem (Round 3)

**Reference:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/plugin/`
**Round 1:** SUBSTANTIVE (14 new BC contracts).
**Round 2:** SUBSTANTIVE (8 new BC contracts, total 22). Identified late-binding projectDir race (BC-PLUGIN-022) as MEDIUM confidence pending r3 trace.
**Round 3 goal:** Validate or downgrade the r2 findings; declare NITPICK if no model-shifting findings remain.

---

## 1. Round-2 follow-ups

### 1.1 Validate BC-PLUGIN-022 (late-binding `projectDir` race)

**Trace:** `SetProjectDir` is always called on the gocui main thread from `syncPluginProject` (`app_actions.go:330`) or `syncPluginProjectOnce` (`app_actions.go:126-167`), both invoked from the main thread via key handlers `MoveCursorDown/Up` (`app_actions.go:111, 119`) and layout (`layout.go:5703, 5713`).

After `SetProjectDir(newPath)`, `syncPluginProject` immediately invokes `runPluginAsync(func(ctx)... Refresh(ctx))` (`app_actions.go:331-333`). The goroutine spawned by `runPluginAsync` (`app_actions.go:1060-1072`) eventually invokes `m.cli.<Op>(ctx, ...)` which performs `c.runner.Run(ctx, c.projectDir, ...)` (`cli.go:83, 97, 111, 125, 134, 143, 152, 161, 170, 179, 192`). The read of `c.projectDir` happens **inside the goroutine**, after the main thread has already moved on.

**Concrete race scenario:**

1. T0 (main thread): user is on project A. `c.projectDir = "/a"`.
2. T1 (main thread): user triggers Install. `runPluginAsync` launches goroutine G1; G1 has not yet reached `Run`.
3. T2 (main thread): user presses cursor-down, landing on project B. `syncPluginProject` runs, calls `SetProjectDir("/b")`. Now `c.projectDir = "/b"`.
4. T3 (main thread): same call also spawns goroutine G2 to refresh under `/b`.
5. T4 (goroutine G1): G1 finally reads `c.projectDir`, sees `"/b"`. Installs into `/b` instead of `/a`.

**However:** the GUI runs the install command via a key handler at T1, and the goroutine is launched immediately. Go's scheduler typically runs the goroutine quickly, but there is no guarantee. The main thread continues to next events. The window is real but small.

**Compounding factor:** `runPluginAsync` sets `loading=true` at T1 (`app_actions.go:1061`). But `pluginState.loading` is **not consulted** by key handlers that move the cursor — `MoveCursorDown/Up` (`app_actions.go:102-120`) does not check it. So the user can move the cursor (and thus retrigger `SetProjectDir`) while an install is in flight.

**Upgrade BC-PLUGIN-022 to HIGH confidence.** The race is structurally reachable. The mitigations would be:
- Lock `projectDir` immutability per `ExecCLI` instance (set at construction).
- Capture `projectDir` at goroutine launch (closure over the value, not the field).
- Disable cursor movement while `loading == true`.

Option 2 (capture at launch) is the cleanest minimal fix: the GUI could pass the resolved `projectDir` into `Manager.<Op>` as a parameter, or `Manager` could snapshot `c.projectDir` synchronously in `Install`/`Uninstall`/etc., returning the cwd to use rather than dereferencing the field inside the runner.

### 1.2 Verify ISO 8601 parseability

The observed timestamp format in fixtures is `2026-03-04T16:26:07.583Z` (`model_test.go:39967, 39976, 40021`). This is **`time.RFC3339Nano`-parseable** (it has fractional seconds + UTC `Z`).

The package does no `time.Parse` — confirmed by an absence of any `time.RFC` or `time.Parse` reference in `internal/plugin/*.go`. Monocle that wants relative-time display ("3 days ago") should use:

```go
t, err := time.Parse(time.RFC3339Nano, installedAt)
```

For robustness against future format drift, `time.RFC3339` (without nano) should be tried as fallback. The package's choice to keep timestamps as strings is deliberate — the only display path (`presentation/plugins.go:103-109`) wants the date-only prefix, which a `strings.IndexByte(s, 'T')` slice gives more cheaply than parsing.

### 1.3 Concurrent `SetProjectDir` vs reads

Re-examining the operations in light of r2 §2.4:

| Read site | Synchronization with `SetProjectDir` |
|---|---|
| `ListInstalled` (`cli.go:83`) | None |
| `ListAll` (`cli.go:97`) | None |
| `ListMarketplaces` (`cli.go:111`) | None |
| Mutation ops (`cli.go:125, 134, 143, 152, 161, 170, 179, 192`) | None |

All read `c.projectDir` field directly with no mutex. On 64-bit platforms with aligned string headers, the read is **not torn** (Go strings are two words, but writes/reads to aligned 16-byte values are not atomic without a sync primitive — the result is a non-torn-pointer but possibly torn-length scenario). In practice Go's race detector would flag this; the package is tested with `go test -race` via the `CLAUDE.md` standard test command, but no test exercises concurrent `SetProjectDir` + `Run`.

**This is a real, latent race.** Not just BC-PLUGIN-022 in the goroutine-late-binding sense, but also: a `Refresh` invoked from one goroutine while another goroutine calls `SetProjectDir` directly. The GUI never does this (the GUI funnels both through the main thread), but the **package API does not document this constraint**.

**BC-PLUGIN-023: `ExecCLI.SetProjectDir` is not safe to call concurrently with operations**
- **Preconditions:** Two goroutines: one calls `SetProjectDir`, the other calls any operation method.
- **Postconditions:** Undefined under Go's memory model (data race on the `projectDir` string). The package's contract implicitly requires the caller to serialize.
- **Evidence:** `cli.go:42-44, 49-51, 24-26`; absence of mutex.
- **Confidence:** HIGH

### 1.4 Could `ExecCLI` be immutable?

Yes — and this is the recommended monocle remediation:

```go
type ExecCLI struct {
    runner     Runner
    projectDir string // set once at construction
}

func NewExecCLI(projectDir string, opts ...Option) *ExecCLI { ... }
```

When the project changes, the caller would construct a new `ExecCLI` (and a new `Manager`, since `Manager` holds the `*ExecCLI`). The Manager's cache would be naturally reset because it is per-instance state.

This eliminates BC-PLUGIN-022, BC-PLUGIN-023, and the r2 §2.4 race window in one stroke. The cost is that the GUI must hold a different shape: instead of one Manager retargeted, it would have a `func(projectDir string) *Manager` factory or a `map[string]*Manager` cache keyed by project dir.

**Spec recommendation for monocle:** prefer immutability. Caching one Manager per project is a small memory cost (3 slices + 3 ints) and removes the entire class of late-binding bugs.

### 1.5 Test files re-audit

Searching for `t.Skip` and build tags in plugin tests turned up nothing — there are no skipped tests, no `_test.go` build tag exclusions. The 17 test functions cover what they cover; nothing is conditionally disabled.

This means the gaps named in r1 §9.2 are real (not skipped tests waiting on infrastructure), and would need to be authored fresh in monocle.

---

## 2. Cross-pass cross-references

### 2.1 Alignment with Pass 8 final synthesis

| Pass 8 claim | Plugin r1/r2/r3 finding | Status |
|---|---|---|
| P1-009: `runPluginAsync` uses `context.Background()` — no cancellation | Confirmed verbatim (BC-PLUGIN — implicit; r1 §7.4) | ALIGNED |
| `internal/plugin` is LOW direct relevance | Disputed: BC-PLUGIN-022/023 are real bugs, BC-PLUGIN-012 is a real product-scope question. The package itself is small, but monocle's port decision is non-trivial. | NUANCE: package small, decisions non-trivial |
| Pass 3 had no BC-PLUGIN-* contracts | Confirmed (zero) | ALIGNED |
| Drop or PORT-DIRECT depending on scope | Plugin r1 §12 says PORT-DIRECT will inherit the late-binding race and unused logger; recommend redesign | NUANCE |

### 2.2 New cross-pass items

Pass 8 §437 names P1-009 as "plumb cancellation context". Plugin r3 adds two more P1-level items that Pass 8 did not surface:

- **P1-PLUGIN-NEW-A:** `ExecCLI.projectDir` late binding (BC-PLUGIN-022). Mitigation: capture at goroutine launch or make `ExecCLI` immutable.
- **P1-PLUGIN-NEW-B:** `ExecCLI.SetProjectDir` is unsynchronized (BC-PLUGIN-023). Mitigation: same as above, or add explicit sync.

Both should be added to monocle's risk register.

---

## 3. Delta Summary

- **New BC-PLUGIN contracts:** 1 (BC-PLUGIN-023). Total now 23.
- **Confidence upgrades:** BC-PLUGIN-022 MEDIUM -> HIGH.
- **Confirmed non-issues:** multi-`@` MarketplaceName, new source types, ISO 8601 future parseability.
- **New P1-level monocle items:** P1-PLUGIN-NEW-A, P1-PLUGIN-NEW-B.
- **Recommended remediation:** `ExecCLI` immutability — eliminates the entire race class.
- **Test-audit conclusion:** no `t.Skip`, no build tags. Coverage gaps in r1 §9.2 are real.

## 4. Novelty Assessment

**Novelty: NITPICK**

Justification: This round contributes one new contract (BC-PLUGIN-023) and one confidence upgrade. The substantive remediation recommendation (`ExecCLI` immutability) is a follow-on from the r2 race finding, not a fresh discovery. The pass-8 cross-reference confirms alignment, not new gaps.

Were I to remove this round's findings, monocle would lose:
- The HIGH-confidence upgrade on BC-PLUGIN-022 (would remain MEDIUM, still actionable).
- The explicit unsynchronized-SetProjectDir contract (BC-PLUGIN-023), which is structurally the same race as BC-PLUGIN-022 just at a different code site.
- The immutability remediation recommendation.

These are refinements, not model-shifters. The plugin subsystem is now fully extracted:
- All 6 source files have been line-scanned.
- All 22 (now 23) BC contracts cover the production paths.
- All gaps named in r1 §9.2 and r2 §3 have been resolved or confirmed.
- Cross-pass alignment with Pass 8 is documented.

Another round would only refine wording or chase truly minor edge cases (e.g., does `MarketplaceUpdate` with name="" interact differently if `claude` CLI is updated to require name in future versions — purely speculative).

## 5. Convergence Declaration

**Pass B-plugin has converged.** Findings from this round are nitpicks, not gaps. The plugin subsystem deep-dive is complete at 3 rounds with 23 behavioral contracts authored, all gaps from prior rounds resolved, and clear monocle remediation recommendations.

## State Checkpoint

```yaml
pass: B-plugin
round: 3
status: complete
files_scanned: 6
prior_round_contracts: 22
new_contracts: 1
total_contracts: 23
timestamp: 2026-05-11T18:32:00Z
novelty: NITPICK
next_round_needed: false
convergence: declared
```

---

## Appendix A — Full BC-PLUGIN catalog index

| ID | Round | Confidence | Topic |
|---|---|---|---|
| BC-PLUGIN-001 | r1 | HIGH | Cache contract for Installed/Available/Markets |
| BC-PLUGIN-002 | r1 | HIGH | Refresh fallback ListAll -> ListInstalled |
| BC-PLUGIN-003 | r1 | HIGH | Marketplace listing failure non-fatal |
| BC-PLUGIN-004 | r1 | HIGH | Mutating ops trigger Refresh |
| BC-PLUGIN-005 | r1 | HIGH | ToggleEnabled direction from cache |
| BC-PLUGIN-006 | r1 | HIGH | ToggleEnabled unknown ID error |
| BC-PLUGIN-007 | r1 | MEDIUM | TERM=dumb, NO_COLOR=1 env |
| BC-PLUGIN-008 | r1 | MEDIUM | cmd.Dir = projectDir |
| BC-PLUGIN-009 | r1 | HIGH | Polymorphic Source decode |
| BC-PLUGIN-010 | r1 | HIGH | MarketplaceUpdate empty name omits arg |
| BC-PLUGIN-011 | r1 | HIGH | Defensive copy on getters |
| BC-PLUGIN-012 | r1 | HIGH | Marketplace mutations unwired from GUI |
| BC-PLUGIN-013 | r1 | HIGH | Manager logger unused |
| BC-PLUGIN-014 | r1 | MEDIUM | Cache-safe, action-unsafe concurrency |
| BC-PLUGIN-015 | r2 | HIGH | Binary resolution lives outside package |
| BC-PLUGIN-016 | r2 | HIGH | ISO 8601 timestamps never time.Parse'd |
| BC-PLUGIN-017 | r2 | HIGH | InstallPath, LastUpdated unused downstream |
| BC-PLUGIN-018 | r2 | MEDIUM | Missing claude binary recoverable per-call |
| BC-PLUGIN-019 | r2 | HIGH | All plugin ops gated to local sessions |
| BC-PLUGIN-020 | r2 | HIGH | Tab-binding: install on Marketplace, others on Plugins |
| BC-PLUGIN-021 | r2 | HIGH | Search filter scope asymmetry |
| BC-PLUGIN-022 | r2->r3 | HIGH (upgraded) | Late-binding projectDir race |
| BC-PLUGIN-023 | r3 | HIGH | SetProjectDir not concurrency-safe |

---

## Appendix B — Monocle P-level items

| ID | Priority | Item |
|---|---|---|
| P0-PLUGIN-A | P0 | Decide plugin management v1 scope |
| P1-PLUGIN-B | P1 | Plumb cancellation context (mirrors Pass-8 P1-009) |
| P1-PLUGIN-C | P1 | Action-level serialization to prevent toggle race (BC-PLUGIN-014) |
| P1-PLUGIN-D | P1 | Decide marketplace add/remove/update UX scope (BC-PLUGIN-012) |
| P1-PLUGIN-NEW-A | P1 | Eliminate late-binding projectDir race (BC-PLUGIN-022) |
| P1-PLUGIN-NEW-B | P1 | Eliminate unsynchronized SetProjectDir (BC-PLUGIN-023) |
| P2-PLUGIN-E | P2 | Fresh-read toggle direction (BC-PLUGIN-005 mitigation) |
| P2-PLUGIN-F | P2 | Wire or remove unused Manager logger (BC-PLUGIN-013) |
| P2-PLUGIN-G | P2 | Decide manifest-level reads for richer preview |
| P2-PLUGIN-H | P2 | Decide whether to display LastUpdated and InstallPath (currently dead) |

**Recommended single fix:** make `ExecCLI` immutable (constructed per project dir). This resolves P1-PLUGIN-NEW-A, P1-PLUGIN-NEW-B, and (with action-queue) P1-PLUGIN-C.
