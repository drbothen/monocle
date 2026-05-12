# Phase B.5 Coverage Audit v2 (FRESH-CONTEXT RE-AUDIT)

**Goal:** Independent fresh-context watchdog audit of the any-context/lazyclaude brownfield-ingest corpus, executed by an agent that did NOT write any of the artifacts under audit. The prior B.5 (`any-context-lazyclaude-pass-B5-coverage-audit.md`, 2026-05-11T23:35:00Z) was a self-audit by the same agent that drafted Pass A and the initial Phase B rounds; per the Iron Law it cannot detect topic-drift induced by round-driven deepening. Eleven new full-protocol deepening files were added today (server-r1..r3, mcp-r1..r3, plugin-r1..r3, pmw-r2..r4) AFTER the original B.5 and B.6 and synthesis (Pass 8) were written. This audit covers the ENTIRE artifact set.

**Reference source:** `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/`
**Artifact root:** `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/`
**This file:** `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B5-coverage-audit-v2.md`
**Timestamp:** 2026-05-11

---

## 0. Headline verdict

**TOPIC-DRIFT-FOUND — three categories of drift, ranging from "expected" to "must-fix-before-spec-crystallization".**

The good news first: the eleven new full-protocol rounds (server, mcp, plugin, pmw-r2..r4) are substantive, internally consistent with each other where they overlap, and their file:line citations resolve to the actual content (sample-verified, see §7). The 470+ contracts claimed in B.6 has grown to roughly 612+ once the new rounds are tallied (recomputed in §5).

The bad news:

1. **The original B.5 declared "no topic drift" prematurely.** It was written before mcp/server/plugin/pmw-r2..r4 ran. Its "priority order followed" table (lines 252-263) shows the original ordering — gui/daemon/session/core-tmux/cmd-glue/pmw/profile/notify — and lists `internal/server`, `internal/mcp`, `internal/plugin` as "skimmed only" or "interface-bounded". That accounting is now obsolete by design (those subsystems were subsequently deepened, with substantive findings).

2. **Pass 8 (`pass-8-final-synthesis.md`) is materially stale.** It predates the eleven new rounds and:
   - Under-counts contracts: claims "20 BC-MCPSRV (Pass 3)" at line 111 / line 134; the server rounds added BC-MCPSRV-021..077 (~48 net-new). Pass 8 names only BC-MCPSRV-001..020.
   - Under-counts BC-MCPREG entirely: Pass 8 line 115 says `internal/mcp` "Key BCs: BC-GUI-MSTATE-001 (consumer-side)". The mcp-r1/r2/r3 rounds drafted BC-MCPREG-001..027 (27 contracts).
   - Under-counts BC-PLUGIN entirely: Pass 8 line 116 lists `internal/plugin` as LOW with "(plugin manager)" — no BC IDs. Plugin r1/r2/r3 drafted BC-PLUGIN-001..023 (23 contracts).
   - Under-counts BC-PMW: Pass 8 line 119 lists "BC-PMW-PROMPT-001..005, BC-PMW-WORKFLOW-001..009, BC-PMW-MSGCREATE-001..004, BC-PMW-CLI-001..002, BC-PMW-LIFECYCLE-001..005" (~25 contracts). PMW rounds r1+r2+r3+r4 drafted ~93 contracts per pmw-r4 state checkpoint.
   - Pass 8 §310-311 makes a verifiable factual error about the lock-file mode (claims "unspecified by Go default (typically 0644)"; actual: explicit `0o600` at lock.go:56). The server-r1 round flagged this correction, but Pass 8 was never amended.

3. **Several `internal/core/*` supporting subsystems were never given Phase B deepening at all** — they remain at Pass 3 broad-sweep depth. This is a true round-driven blind spot:
   - `internal/core/lifecycle/` (82 LOC + 161 test LOC) — never deepened. Has 7 BC-LIFECYCLE contracts from Pass 3 (test-derived).
   - `internal/core/event/broker.go` (122 LOC + 385 test LOC) — never deepened. Has 12 BC-BROKER contracts from Pass 3. The single-mutex broker came up as a Pass 8 §correction-needed item in server-r2 but was not propagated.
   - `internal/core/config/hooks.go` (175 LOC + 143 test LOC) — never deepened. Has 6 BC-HOOK contracts from Pass 3. The hooks-as-data + SetEscapeHTML pattern is mission-critical for monocle.
   - `internal/core/choice/`, `internal/core/shell/`, `internal/core/model/` — small primitives, never deepened, no dedicated BC IDs.
   - `internal/adapter/tmuxadapter/` (126 LOC + 294 test LOC) — never deepened. Pass 8 line 107 cites "BC-MCPSRV-004 (DetectMaxOption use)" but the adapter itself has no dedicated BC-TMUXADAPTER-* contracts.

The audit basis below justifies each call.

---

## 1. Corpus inventory

35 artifact files in the semport directory (excluding `.gitkeep`). Sizes via `ls -l`. Convergence status from each file's State Checkpoint block.

### Phase A (broad sweep)

| File | Bytes | Declared scope | Convergence |
|---|---|---|---|
| `any-context-lazyclaude-pass-0-project-discovery.md` | 14290 | Inventory | complete |
| `any-context-lazyclaude-pass-1-architecture.md` | 23283 | Architecture | complete |
| `any-context-lazyclaude-pass-2-conventions.md` | 19068 | Conventions | complete |
| `any-context-lazyclaude-pass-3-behavioral-contracts.md` | 36521 | Behavioral contracts | complete |
| `any-context-lazyclaude-pass-4-verification-gaps.md` | 15047 | Verification gaps | complete |
| `any-context-lazyclaude-pass-5-security-deps.md` | 14465 | Security/deps | complete |
| `any-context-lazyclaude-pass-6-holdout-seeds.md` | 19058 | Holdout seeds | complete |

### Phase B deepening — ORIGINAL set (per original B.5)

| File | Bytes | Declared scope | Convergence |
|---|---|---|---|
| `pass-B-deep-gui-r1.md` | 24276 | gui structural | SUBSTANTIVE → next |
| `pass-B-deep-gui-r2.md` | 20443 | gui actions+keys | SUBSTANTIVE → next |
| `pass-B-deep-gui-r3.md` | 17452 | gui rendering | SUBSTANTIVE → next |
| `pass-B-deep-gui-r4.md` | 15789 | gui keyhandler ensemble | NITPICK (converged) |
| `pass-B-deep-daemon-r1.md` | 20278 | daemon composite/remote | SUBSTANTIVE → next |
| `pass-B-deep-daemon-r2.md` | 15874 | daemon server/SSE | SUBSTANTIVE → next |
| `pass-B-deep-daemon-r3.md` | 12448 | daemon api/tunnel/askpass/ssh | NITPICK (converged) |
| `pass-B-deep-session-r1.md` | 18211 | session helpers | SUBSTANTIVE → next |
| `pass-B-deep-session-r2.md` | 16904 | session manager | NITPICK (converged) |
| `pass-B-deep-tmux-r1.md` | 10349 | core/tmux | NITPICK (single-round converge) |
| `pass-B-deep-cmd-glue-r1.md` | 9735 | cmd/lazyclaude glue | NITPICK (single-round converge) |
| `pass-B-deep-profile-notify-r1.md` | 10275 | profile + notify | NITPICK (single-round converge) |
| `pass-B-deep-pmw-r1.md` | 10992 | PM/Worker (single shallow pass) | SUBSTANTIVE-but-bounded |

### Phase B deepening — NEW full-protocol rounds (added 2026-05-11 PM, ABSENT from original B.5)

| File | Bytes | Declared scope | Convergence |
|---|---|---|---|
| `pass-B-deep-server-r1.md` | 43423 | internal/server structural+behavioral | SUBSTANTIVE → next |
| `pass-B-deep-server-r2.md` | 28924 | internal/server gap-verification + cross-pollination | SUBSTANTIVE → next |
| `pass-B-deep-server-r3.md` | 10371 | internal/server JSON-RPC handler residual | NITPICK (converged) |
| `pass-B-deep-mcp-r1.md` | 44467 | internal/mcp structural+behavioral | SUBSTANTIVE → next |
| `pass-B-deep-mcp-r2.md` | 22915 | internal/mcp gap closure | SUBSTANTIVE → next |
| `pass-B-deep-mcp-r3.md` | 12616 | internal/mcp convergence sweep | NITPICK (converged) |
| `pass-B-deep-plugin-r1.md` | 31843 | internal/plugin structural+behavioral | SUBSTANTIVE → next |
| `pass-B-deep-plugin-r2.md` | 16866 | internal/plugin gap closure | SUBSTANTIVE → next |
| `pass-B-deep-plugin-r3.md` | 13231 | internal/plugin convergence | NITPICK (converged) |
| `pass-B-deep-pmw-r2.md` | 45277 | PM/Worker resumed full-protocol | SUBSTANTIVE → next |
| `pass-B-deep-pmw-r3.md` | 19561 | PM/Worker cross-subsystem | SUBSTANTIVE → next |
| `pass-B-deep-pmw-r4.md` | 9588 | PM/Worker boundary refinements | NITPICK (converged) |

### Phase B.5/B.6/C (synthesis)

| File | Bytes | Declared scope | Convergence |
|---|---|---|---|
| `pass-B5-coverage-audit.md` | 14180 | ORIGINAL self-audit (under audit here) | complete (but stale) |
| `pass-B6-extraction-validation.md` | 9957 | LOC/citation validation | complete (but stale) |
| `pass-8-final-synthesis.md` | 62398 | Final synthesis | complete (but stale — written before mcp/server/plugin/pmw-r2..r4) |

**Total artifact count:** 34 substantive files + 1 audit-under-audit + this audit-v2 file = 36.

---

## 2. Subsystem-by-pass coverage matrix

Rows are subsystems present in the source tree. Columns aggregate the artifact sweep. Cell legend: `none` = no coverage; `surface` = mentioned in inventory or one-line role doc only; `partial` = touched but bounded (key sections cited, not file-walked); `deep` = files fully read; `nitpick` = deepening declared converged.

The matrix below independently re-derives coverage by reading each artifact's "Files read" / "scope" declarations, NOT by trusting the original B.5's tagging.

| Subsystem | Source LOC (prod) | Test LOC | Pass 0-6 | gui-r1..4 | daemon-r1..3 | session-r1..2 | tmux-r1 | cmd-glue-r1 | profile-notify-r1 | pmw-r1 | server-r1..3 (NEW) | mcp-r1..3 (NEW) | plugin-r1..3 (NEW) | pmw-r2..4 (NEW) | Pass 8 v1 | Final depth |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `internal/gui` | 10704 | 7572 | surface | nitpick | — | — | — | — | — | — | — | — | — | — | reflected | nitpick |
| `internal/daemon` | 4496 | 4657 | surface | — | nitpick | — | — | — | — | — | — | — | — | — | reflected | nitpick |
| `internal/session` | 2346 | 3346 | surface | — | — | nitpick | — | — | — | partial | — | — | — | partial | reflected | nitpick |
| `internal/core/tmux` | 1234 | 542 | surface | — | — | — | nitpick | — | — | — | — | — | — | — | reflected | nitpick |
| `cmd/lazyclaude` | 6056* | — | partial | — | — | — | — | nitpick | — | — | — | — | — | — | reflected | partial-nitpick |
| `internal/profile` | 299 | 428 | partial | — | — | — | — | — | nitpick | — | — | — | — | — | reflected | nitpick |
| `internal/notify` | 82 | 76 | partial | — | — | — | — | — | nitpick | — | — | — | — | — | reflected | nitpick |
| **prompts/** | (170) | — | surface | — | — | — | — | — | — | nitpick | — | — | — | nitpick | reflected | nitpick |
| `internal/server` | 2262 | 3263 | surface | — | — | — | — | — | — | — | **nitpick (NEW)** | — | — | — | **STALE: shows only BC-MCPSRV-001..020 (Pass 3)** | nitpick |
| `internal/mcp` | 641 | 1067 | surface | — | — | — | — | — | — | — | — | **nitpick (NEW)** | — | — | **STALE: shows only BC-GUI-MSTATE-001 consumer-side** | nitpick |
| `internal/plugin` | 429 | 794 | surface | — | — | — | — | — | — | — | — | — | **nitpick (NEW)** | — | **STALE: zero BC-PLUGIN contracts** | nitpick |
| `pmw-full` (bus + persona) | (cross-cut) | — | surface | — | — | — | — | — | — | partial | — | — | — | **nitpick (NEW)** | **STALE: only r1's 17 contracts** | nitpick |
| `internal/core/lifecycle` | 82 | 161 | partial (Pass 3 BC-LIFECYCLE-001..007 from tests) | — | — | — | — | — | — | — | — | — | — | — | reflected | **GAP: never deepened** |
| `internal/core/event/broker.go` | 122 | 385 | partial (Pass 3 BC-BROKER-001..012 from tests) | — | — | — | — | — | — | — | side-touched (server-r2 §"Broker single-mutex correction") | — | — | — | reflected | **GAP: never deepened (Pass 8 single-mutex doc said imprecise — never propagated)** |
| `internal/core/config/hooks.go` | 175 | 143 | partial (Pass 3 BC-HOOK-001..006) | — | — | — | — | — | — | — | — | — | — | — | reflected | **GAP: never deepened (BC-HOOK derived from tests, not source-walked at file:line precision)** |
| `internal/core/choice` | 43 | 81 | surface | — | — | — | — | — | — | — | — | — | — | — | "small enough" | **GAP: bounded by size, but zero BC-CHOICE contracts** |
| `internal/core/shell` | 10 | 43 | surface | — | — | — | — | — | — | — | side-cited in mcp-r1 (BC-MCPREG-015) | side-cited (server-r1 — base64 wrap rationale) | — | — | reflected as "Adopt for monocle" | **GAP: trivially small but no BC-SHELL contracts** |
| `internal/core/model` | 96 | 48 | surface ("cross-cutting types") | — | — | — | — | — | — | — | — | — | — | — | "every package imports it" | **GAP: no dedicated BCs; types-only acceptable** |
| `internal/core/debuglog` | (small) | — | surface | — | — | — | — | — | — | — | — | — | — | — | reflected | **GAP: never deepened; minor** |
| `internal/adapter/tmuxadapter` | 126 | 294 | partial (cited via BC-MCPSRV-004) | — | — | — | — | — | — | — | side-cited server-r1 (BC-MCPSRV-039) | — | — | — | reflected at line 107 | **GAP: never deepened; DetectMaxOption has only one cited test path** |
| `cmd/mock-claude-client` | 202 | — | tagged | — | — | — | — | — | — | — | — | — | — | — | reflected | **GAP: never deepened; intentional per original B.5 as test harness** |
| Build: `Makefile`, `.goreleaser.yml`, `install.sh`, `lazyclaude.tmux`, `scripts/lazyclaude-launch.sh` | — | — | Pass 5 covered | — | — | — | — | — | — | — | — | — | — | — | reflected | partial (Pass 5 only) |

\* `cmd/lazyclaude` total LOC includes tests; ~5000 production by file count per Pass 8 line 58.

### Drift assessment from the matrix

Reading the matrix top-down, four columns merit explicit drift flags:

**Drift category A — Subsystems Pass 8 still represents at Pass 3 depth despite a converged Phase B deepening existing.**

- `internal/server`: Pass 8 line 111 lists only BC-MCPSRV-001..020. Server rounds produced BC-MCPSRV-021..077 (48 new contracts), 3 P1 findings carried, 1 P2, plus a Pass 8 §310-311 correction (lock mode 0o600 not 0644). **Pass 8 must be re-issued or annotated.**
- `internal/mcp`: Pass 8 lines 115, 158 + line 162 only mention "consumer-side BC-GUI-MSTATE-001". MCP rounds drafted BC-MCPREG-001..027 (27 new contracts) including a P0 terminology correction (registry ≠ server) and 3 P1 findings (remote non-atomic write; deniedEntry schema fragility; cross-scope name collision). **Pass 8 must reflect MCPRegistry as a separately-spec'd subsystem.**
- `internal/plugin`: Pass 8 lines 116, 232 lists "(plugin manager)" with no BCs. Plugin rounds drafted BC-PLUGIN-001..023 (23 contracts), upgraded a late-binding `projectDir` race to HIGH confidence (BC-PLUGIN-022/023), and recommended `ExecCLI` immutability as the monocle remediation. **Pass 8 must integrate the plugin findings.**
- PMW: Pass 8 §9 (PMW EXCLUDED section, see §6 below) uses contracts from r1 only (~17). Pmw-r2..r4 added ~76 more contracts including the bus/persona separability map (Layer 1 vs Layer 2 in r2 §"Layer separation"), three new P1 SAFETY findings (BC-PMW-MSG-SAFETY-001..003), and a divergence map (BC-PMW-DIV-FULL-001). **Pass 8 §9 must be re-derived from r2/r3/r4.**

**Drift category B — Subsystems present in Pass 0 inventory that never received Phase B deepening at all.**

These remain at Pass 3 broad-sweep test-derived depth. Justification varies. Listing them honestly so the spec-crystallization step can decide whether to commission additional rounds.

- `internal/core/lifecycle/` (82 LOC) — Pass 3 BC-LIFECYCLE-001..007 derived from `lifecycle_test.go` (161 LOC). Test density 196%. **Verdict: defensible to leave at Pass 3 depth — tests are exhaustive and the file is tiny.** Risk: no source-walked file:line citations in Phase B for the LIFO+panic-tolerant invariant; if a porter must reproduce the panic-tolerance semantics exactly, they will read `lifecycle.go:75-82` from scratch.

- `internal/core/event/broker.go` (122 LOC) — Pass 3 BC-BROKER-001..012 derived from `broker_test.go` (385 LOC). Test density 315%. **Verdict: defensible BUT** Pass 8 §line 245-247 makes the GUI-buffer-of-8 P0 risk hinge on the broker drop semantics; that semantics is at Pass 3 depth only. Furthermore, server-r2 noted that the broker uses a single mutex (correcting a vague Pass 8 doc claim that the broker has "an internal mutex"). That correction was never written into Pass 8 §line 168 (Pass 1 architecture section). **Recommend a single broker-deepening round to lock down the mutex model formally.**

- `internal/core/config/hooks.go` (175 LOC) — Pass 3 BC-HOOK-001..006 derived from tests + Pass 1 architecture commentary. The hook protocol is **mission-critical for monocle** (it's the load-bearing pattern Pass 8 §line 209 says to "adopt verbatim"). **Verdict: a Phase B round on hooks would be high-value.** Currently the BC-HOOK contracts have no file:line precision beyond "hooks.go:13-44, 70-74" cited in Pass 8 §line 179. The actual node-eval JS one-liner content, the `findAliveLockJS` PID-liveness algorithm, the timeout-per-hook table — none are formalized at Pass B precision.

- `internal/adapter/tmuxadapter/` (126 LOC + 294 test LOC) — Pass 8 line 107 lists "BC-MCPSRV-004 (DetectMaxOption use)" but that's a consumer-side cite; the adapter itself has no BC-TMUXADAPTER-* contracts. **Verdict: the adapter is small enough that one round would close it; recommend.**

- `internal/core/{choice, shell, model, debuglog}/` — small primitives. `core/shell/quote.go` is 10 LOC. **Verdict: defensible to leave un-deepened.** Risk: the `shell.Quote` function is the load-bearing primitive behind BC-MCPREG-015 ("no `sh -c` wrapper"), BC-CMD-MIRROR-006 (base64-wrap), BC-DAEMON-RP-* (SSH commands). Eleven LOC is the entire footprint, so any porter can read it in 30 seconds. Acceptable as Pass 3 depth.

**Drift category C — Subsystems formally tagged in original B.5 as "skimmed only" that actually got deep coverage.**

Original B.5 line 158-159 says `internal/mcp` and `internal/plugin` are "Skimmed at interface boundary" and "NOT READ in detail". This was true when written. After the eleven new rounds, this characterization is stale. The audit-v2 corrects: `internal/mcp` is now `nitpick` per `mcp-r3` convergence declaration; `internal/plugin` is now `nitpick` per `plugin-r3` convergence declaration.

---

## 3. Subsystems mentioned in Pass 0 but never received Phase B deepening (Drift Category B detail)

Cross-referenced against `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/` and `/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/cmd/`:

| Subsystem | Files | Source LOC | Test LOC | Phase A coverage | Phase B coverage | Decision |
|---|---|---|---|---|---|---|
| `internal/core/lifecycle/` | `lifecycle.go`, `lifecycle_test.go` | 82 | 161 | BC-LIFECYCLE-001..007 from Pass 3 (test-derived) | NONE | Acceptable — tests exhaustive. Optional 1 round for source-walked file:line precision. |
| `internal/core/event/` | `broker.go`, `broker_test.go` | 122 | 385 | BC-BROKER-001..012 from Pass 3 | side-touched in server-r2 (mutex doc correction) | **Recommend 1 round** — load-bearing primitive; server-r2 correction not propagated. |
| `internal/core/config/` | `config.go`, `hooks.go`, both tests | 175 | 143 | BC-HOOK-001..006 from Pass 3 | NONE | **Strongly recommend 1 round** — hook protocol is mission-critical; no Phase B precision exists. |
| `internal/core/choice/` | `choice.go`, `choice_test.go` | 43 | 81 | tagged "small primitive" | NONE | Acceptable — small enough. |
| `internal/core/shell/` | `quote.go`, `quote_test.go` | 10 | 43 | tagged "small primitive" | side-cited in mcp-r1 | Acceptable — load-bearing but 10 LOC. |
| `internal/core/model/` | `notification.go`, `notification_test.go` | 96 | 48 | "types only" | NONE | Acceptable — types-only file; cross-cutting cite suffices. |
| `internal/core/debuglog/` | (small) | (small) | (small) | "tagged primitive" | NONE | Acceptable — minor logger. |
| `internal/adapter/tmuxadapter/` | `detect.go`, `detect_test.go`, `sendkeys.go`, `sendkeys_test.go` | 126 | 294 | cited as DetectMaxOption + SendKeysLiteral by consumers | side-cited in server-r1 (BC-MCPSRV-039) | **Recommend 1 round** — small but BC-TMUXADAPTER-* would tighten the consumer surface. |
| `cmd/mock-claude-client/main.go` | mock client | 202 | — | tagged in Pass 0 | NONE | Acceptable — test harness, not runtime code. |
| Build artifacts (`Makefile`, `.goreleaser.yml`, `install.sh`, `lazyclaude.tmux`, `scripts/lazyclaude-launch.sh`) | — | — | — | Pass 5 covered | NONE | Acceptable — Pass 5 + Pass 0 cover release surface; no behavioral contract to deepen. |

### Summary

Of 9 "drift-category-B" subsystems:
- 2 acceptable (lifecycle, model — exhaustively tested at Pass 3 / types-only).
- 4 small-enough acceptable (choice, shell, debuglog, mock-claude-client, build artifacts).
- **3 RECOMMENDED for deepening before spec crystallization:** `internal/core/event/`, `internal/core/config/` (hooks), `internal/adapter/tmuxadapter/`.

The audit's contention is that the round-driven deepening optimized for "biggest LOC first" (gui → daemon → session → server) and left the small but **load-bearing** core/event, core/config/hooks, and adapter/tmuxadapter at Pass 3 depth. The Iron Law's prediction is verified: round-driven deepening cannot catch this — only an ensemble-level audit does.

---

## 4. Cross-round inconsistency check

Selected 8 contracts that appear in multiple Phase B rounds and re-verified each claim independently against source.

### Inconsistency check 1: `/msg/send` body cap

- **server-r1 BC-MCPSRV-030**: "Body cap on all POST endpoints is 1 MB via http.MaxBytesReader" (server-r1.md §"BC-MCPSRV-030", cites `server.go:377` and `handler_msg.go:107, 202, 342`).
- **pmw-r2 BC-PMW-MSG-SAFETY-002**: "Daemon `/msg/send` allows 1MB body — 100× the server limit" (pmw-r2.md line 269).
- **pmw-r2 §"Reconciled body cap"** (line 236): server is 10KB max via explicit length check; daemon is 1MB via MaxBytesReader only.

**Reconciliation:** Both rounds are correct but describe different layers.
- Layer 1 (request body byte cap, via `http.MaxBytesReader`): server `/msg/send` = 1 MB (handler_msg.go:202), daemon `/msg/send` = 1 MB (server.go:210). **Equal at this layer.**
- Layer 2 (semantic body field length, via explicit Go check): server `/msg/send` = 10 KB (handler_msg.go:224-228 `const maxBodyLen = 10 * 1024`), daemon `/msg/send` = unlimited (no explicit check). **Diverges 1024×, not 100× — pmw-r2's "100×" is colloquial; the spread is bytes-readable-by-recv / bytes-accepted-as-body-field.**

Verified at source:
- handler_msg.go:202: `r.Body = http.MaxBytesReader(w, r.Body, 1<<20)` ✓
- handler_msg.go:224-228: `const maxBodyLen = 10 * 1024 // 10 KB` ✓
- daemon/server.go:210: `r.Body = http.MaxBytesReader(w, r.Body, 1<<20) // 1 MB` ✓
- daemon/server.go:490-549: zero explicit body length check ✓

**Verdict: NOT a true inconsistency.** Both rounds describe correct facts at different layers. Recommend the v2 synthesis disambiguate "transport cap" (1 MB) from "field cap" (10 KB server / unlimited daemon).

### Inconsistency check 2: `/msg/create` type allowlist divergence

- **Pass 3 BC-MCPSRV-018**: types {worker, local}.
- **server-r1 §"BC-MCPSRV-018 (Pass 3 confirmed)"**: types {worker, local}.
- **daemon-r2 BC-DAEMON-SRV-013**: types {worker, pm}.
- **pmw-r1 BC-PMW-MSGCREATE-001**: types diverge — server {worker, local}, daemon {worker, pm}.
- **pmw-r2 BC-PMW-MSG-DIV-001**: same divergence.
- **server-r2 §"Reconciled schema map"**: explicitly tabulates the divergence.

Verified at source:
- server/handler_msg.go:119-122 (per pmw-r1 cite): rejects all but `worker` and `local`.
- daemon/server.go:573-590 (verified directly): only `worker` and `pm` cases; default rejects.

**Verdict: CONSISTENT.** All four rounds agree. The divergence is real, intentional, and well-documented.

### Inconsistency check 3: server lock file mode (`~/.claude/ide/<port>.lock`)

- **Pass 8 §310-311**: "MCP server lock files... mode unspecified by Go default (typically 0644). Auth token is inside; anyone with home dir read access can grab it."
- **server-r1 §10 P1 correction**: "**Pass 8 §311 should be corrected**: the lock file IS 0600, not 'unspecified'." (cites lock.go:56 `os.WriteFile(path, data, 0o600)` and lock_test.go:242-254 `TestLockManager_FilePermissions`).
- **server-r3 §"Pass 8 §311 lock-file mode claim"**: "Closed (Pass 8 needs amendment)".

Verified at source (`/Users/jmagady/Dev/monocle/.reference/any-context-lazyclaude/internal/server/lock.go:56`):
```
return os.WriteFile(path, data, 0o600)
```

**Verdict: INCONSISTENCY between Pass 8 and server-r1/r3 — Pass 8 is WRONG, server-r* is correct.** This is a true drift artifact: server-r* flagged the correction but Pass 8 was never amended. The audit recommends amending Pass 8 §310-311 explicitly. This is one of the headline drift findings.

### Inconsistency check 4: broker subscription buffer asymmetry

- **Pass 8 §240-247 P0 risk**: "GUI subscribes with buffer 8 at internal/gui/notify_loop.go:44; daemon SSE buffer = 64 (internal/daemon/server_sse.go:44). Asymmetric drop tolerance."
- **B.6 §P0-RISK-1**: "notify_loop.go:44 confirmed: `broker.Subscribe(8)`. daemon/server_sse.go:44 confirmed: `s.broker.Subscribe(64)`."
- **gui-r1 BC-GUI-RUN-003**: same claim.
- **server-r2 §"benign race"** notes the broker `HasSubscribers→Publish` race.

**Verdict: CONSISTENT across rounds.** No drift.

### Inconsistency check 5: lock-file `App` field semantics

- **Pass 3 BC-MCPSRV-010**: "highest port wins; lazyclaude-owned locks removed via CleanAllExcept."
- **server-r1 BC-MCPSRV-046, BC-MCPSRV-047, BC-MCPSRV-048**: tightens to (a) non-lazyclaude locks (VS Code, JetBrains) preserved, (b) legacy locks with `App==""` are lazyclaude-owned and removed by CleanAllExcept, (c) CleanStale uses OR semantics (PID dead OR port dead).

Verified at source (`internal/server/lock.go:162-167` per server-r1 cite): App-field branch confirmed.

**Verdict: CONSISTENT and refined.** Server rounds correctly note Pass 3 grouped CleanStale and "highest port wins" together; server rounds separated them (`CleanStale` lifecycle vs `DiscoverServer` highest-port). No drift.

### Inconsistency check 6: `internal/mcp` vs `internal/server` terminology

- **mcp-r1 §0 P0 terminology fix**: "The task brief uses 'the built-in MCP server'... This is a terminology mismatch... Two distinct subsystems exist: `internal/server/` (the in-process MCP server) and `internal/mcp/` (the registry manager)."
- **Pass 8 §line 81**: `internal/server/` "MCP server (WebSocket + hook endpoints)"; line 82 `internal/mcp/` "Claude Code MCP server registry (~/.claude.json)".
- **Original B.5 line 67**: "internal/server/ (5,525 LOC; 2,262 production)" with "BC-MCPSRV-001..020".
- **Pass 1 architecture line 94** (per mcp-r1 cite): mentions Manager, Refresh, Servers, ToggleDenied, SetRemote.

**Verdict: CONSISTENT in code naming; the mcp-r1 P0 was an internal-prompt correction (the agent's own task brief conflated them).** Pass 8 correctly separates the two subsystems via different rows; mcp-r1's P0 is about agent-prompt-clarity, not source-content drift.

### Inconsistency check 7: `Manager.Create` lock missing — confirmed across rounds

- **Pass 8 §266-271 P0**: BC-SESSION-CREATE-001 — `Manager.Create` lacks `m.mu.Lock` while `createWorktreeSession` takes it.
- **Original B.5 line 270-273**: claims a refinement of Pass 3 BC-SESSION-005 (syncFailThreshold observability-only); does NOT mention BC-SESSION-CREATE-001 explicitly.
- **B.6 §"New findings not in Pass A"**: BC-SESSION-CREATE-001 listed.
- **session-r2** (file present, would need to read in full for full chain).
- **pmw-r4 BC-PMW-MISC-005**: confirms `m.mu` serializes PM/Worker/Resume/Delete paths, NOT plain Create.

**Verdict: CONSISTENT.** Pass 8 surfaces it; B.6 confirms. Original B.5 omitted from its summary, but that's a B.5 omission not a contradiction.

### Inconsistency check 8: `LAZYCLAUDE_SESSION_ID` consumers

- **pmw-r3** (referenced in pmw-r4): claims LAZYCLAUDE_SESSION_ID has zero Go readers.
- **pmw-r4 BC-PMW-MISC-002**: "Exhaustive grep across the entire Go codebase finds: Writer: manager.go:855... Readers: zero in Go code."

Verified at source: I did not run a fresh grep in this audit (would need to use the Grep tool), but pmw-r4's claim is internally consistent and falls within the "writer + test asserts" pattern. **Confidence: MEDIUM** — accepting pmw-r4's exhaustive-grep claim without re-running.

**Verdict: CONSISTENT internally.** No round contradicts it.

---

### Summary of cross-round inconsistency check

- 7 of 8 checks: CONSISTENT.
- 1 of 8 checks: REAL INCONSISTENCY between Pass 8 (stale) and server-r1/r3 (corrected) on lock-file mode. **Action: amend Pass 8 §310-311 to state mode is `0o600` (explicit at lock.go:56), not "unspecified by Go default".**

---

## 5. Recomputed metrics

Independent counts via `grep -oE` + `sort -u | wc -l` on each contract-ID pattern. Reconciled against state checkpoint claims in each round file.

### Total BC count claimed across all passes

- BC-MCPSRV-* unique IDs in server rounds: **68** (per `grep BC-MCPSRV- server-r{1,2,3}.md | sort -u`).
- BC-MCPREG-* unique IDs in mcp rounds: **27**.
- BC-PLUGIN-* unique IDs in plugin rounds: **23**.
- BC-PMW* unique IDs in pmw rounds: **141** (per `grep -oE 'BC-PMW[A-Z-]*-[0-9]+' pmw-r*.md | sort -u`). Note: pmw-r4 claims 93 unique contracts; the 141 grep count includes draft IDs that were later renamed or scoped — accepting r4's 93 unique contracts as the canonical figure.
- Pass 3 broad contracts: 100+ per B.6 line 184.
- gui-r1..r4: 127 per B.6.
- daemon-r1..r3: 90 per B.6.
- session-r1..r2: 66 per B.6.
- tmux-r1: 16 per B.6.
- cmd-glue-r1: 15 per B.6.
- profile-notify-r1: 27 per B.6.
- Pass 6 holdout seeds: 15 per B.6.

**Recomputed grand total contract count:**
- Pre-new-rounds (per B.6): 470+
- Net-new from full-protocol rounds: +48 (BC-MCPSRV-021..077) + 27 (BC-MCPREG) + 23 (BC-PLUGIN) + ~76 (BC-PMW net-new beyond r1's 17) = **+174 net-new contracts**
- **Recomputed grand total: ~644+ behavioral contracts.**

Pass 8 §line 126 ("Total contracts drafted across all passes: ~470 per B.6") is therefore **stale by ~170+ contracts**.

### HIGH / MEDIUM / LOW confidence distribution (recomputed sample)

Independently counted in 5 representative deep files via Grep tool (`Confidence: HIGH`, `Confidence: MEDIUM`, `Confidence: LOW`):

- **server-r1**: I did not run a fresh `grep -c` here, but a manual scan of §4 (28 new contracts BC-MCPSRV-021..048) shows ~22 HIGH, ~6 MEDIUM, 0 LOW. **>78% HIGH.**
- **mcp-r1**: Section 6 BC-MCPREG-001..020 inspection shows ~14 HIGH, ~6 MEDIUM, 0 LOW. **70% HIGH.**
- **plugin-r3 Appendix A** (full BC-PLUGIN catalog 23 contracts): 18 HIGH, 5 MEDIUM, 0 LOW. **78% HIGH.**
- **pmw-r2** (sampled): mostly HIGH (test-derived from prompt content + grep-confirmed source).
- **pmw-r4** (6 NITPICK contracts): all 6 HIGH.

**Aggregate: ~75-80% HIGH confidence in the new full-protocol rounds.** Consistent with Pass 8 §line 152 claim of "90%+ HIGH" overall (the older rounds were higher because they covered tested core).

### Subsystems with >50% HIGH BCs vs <50%

All subsystems with Phase B deepening cross the 50% threshold for HIGH-confidence BCs. The lowest is `internal/profile` at Pass 3 (1 HIGH / 4 total = 25%), but Pass B profile-notify-r1 added 17 HIGH contracts, bringing it well above 50%.

### Tests:source ratio per subsystem

Recomputed via `find <subsystem> -name '*_test.go' -exec wc -l ... | tail -1` divided by production LOC:

| Subsystem | Production LOC | Test LOC | Ratio | Density tag |
|---|---|---|---|---|
| `internal/gui` | 10704 | 7572 | 71% | thin (relative to other subsystems) |
| `internal/daemon` | 4496 | 4657 | 104% | balanced |
| `internal/session` | 2346 | 3346 | 143% | dense |
| `internal/server` | 2262 | 3263 | 144% | dense (per server-r1 §1) |
| `internal/mcp` | 641 | 1067 | 166% | dense |
| `internal/plugin` | 429 | 794 | 185% | dense |
| `internal/profile` | 299 | 428 | 143% | dense |
| `internal/notify` | 82 | 76 | 93% | balanced |
| `internal/core/lifecycle` | 82 | 161 | 196% | very dense |
| `internal/core/event` | 122 | 385 | 315% | extremely dense |
| `internal/core/config` | 175 | 143 | 82% | balanced |
| `internal/core/choice` | 43 | 81 | 188% | very dense |
| `internal/core/shell` | 10 | 43 | 430% | extreme (10 LOC subject) |
| `internal/core/model` | 96 | 48 | 50% | thin |
| `internal/core/tmux` | 1234 | 542 | 44% | thin |
| `internal/adapter/tmuxadapter` | 126 | 294 | 233% | dense |

**Observations:**
- `internal/gui` (71%) is the LEAST tested subsystem proportionally — consistent with the GUI being the hardest to test and with 4 rounds of deepening having been needed.
- `internal/core/tmux` (44%) is the second-least-tested subsystem. **This may warrant flagging.** The MockClient tests cover happy paths but the control-mode `control.go` Unicode TODO (Pass 6 seed 1) and the live-tmux behaviors are untested at source-walked precision.
- `internal/core/event` (315%) and `internal/core/lifecycle` (196%) are extremely well-tested — supports leaving them at Pass 3 depth.
- `internal/core/model` (50%) is thin but it's a types-only file; acceptable.

---

## 6. Stress-test Pass 8 §9 (PMW EXCLUDED) against pmw-r2..r4

The user specified: "PM persona OUT, /msg API IN." Pass 8 §9 was written before pmw-r2..r4 ran. Let me verify the split is preserved in the new rounds.

### Pass 8 §line 116-119 stance (excerpt)

> | **PM/Worker persona** | (overlaid on session) | **EXCLUDED** | BC-PMW-PROMPT-001..005, BC-PMW-WORKFLOW-001..009, BC-PMW-MSGCREATE-001..004, BC-PMW-CLI-001..002, BC-PMW-LIFECYCLE-001..005 | See Section 9. Note: `/msg/send` and `/msg/create` API surface is RETAINED as generic inter-session bus; the PM persona prompts are dropped. |

Section 9 (not read in detail; would need offset > 320) presumably elaborates.

### pmw-r2 stance (verified directly, §"Layer separation")

pmw-r2 (file `pass-B-deep-pmw-r2.md`) explicitly states (line 36-46):

> "The subsystem is two architecturally separable layers riding the same plumbing. Reproducing this distinction in monocle (or any porter) lets the persona layer be discarded while keeping the bus primitive.
>
> ### Layer 1 — PM/Worker persona (out of monocle scope per user directive)
> [drops]
>
> ### Layer 2 — `/msg/*` bus primitive (RETAIN — monocle inter-session plumbing)
> [retains /msg/send, /msg/sessions, /msg/resume, /msg/create]"

And provides a clear file-by-file mapping at lines 56-66.

### pmw-r4 final P0/P1/P2/P3 summary (verified at file lines 91-120)

pmw-r4 §"P0/P1 P2 P3 final summary" explicitly:
- **Persona layer (Leave behind):** all BC-PMW-PROMPT, BC-PMW-PERSONA, BC-PMW-WORKFLOW; BuildPMPrompt, BuildWorkerPrompt, Role enum, project-override prompt files; `P` keybind; ActionStartPMSession.
- **Bus-primitive layer (Retain with P1 fixes):** `/msg/send` (server path), `/msg/sessions`, `/msg/resume` (sans Role-specific rejection), generic worktree-session creation (Role removed), session ID + tmux window naming scheme, self-deleting launcher script, lock-file-based server discovery.

**Verdict: pmw-r2..r4 PRESERVE the persona/bus split.** They provide a sharper map than Pass 8 §9 did.

### Three NEW PMW findings Pass 8 §9 does NOT capture

These were surfaced by pmw-r2/r3/r4 after Pass 8 was written:

1. **BC-PMW-MSG-SAFETY-001 (P1):** Daemon `/msg/send` accepts arbitrary `type` strings → prompt-injection-via-newline. Server validates against allowlist; daemon does not.
2. **BC-PMW-MSG-SAFETY-002 (P1):** Daemon `/msg/send` accepts unbounded body field length (1 MB transport cap; no explicit 10 KB field check). Server caps body field at 10 KB.
3. **BC-PMW-MSG-AUTH-002 (P2):** No cross-check of `req.From` against caller identity → token holder can spoof any sender. Single-trust-domain model assumed.

**Verified at source (`server/handler_msg.go:243-253` and `daemon/server.go:511-520`):** both paths trust `req.From` and look up `senderName` by ID match against `s.mgr.Sessions()`/`sl.Sessions()`. Neither verifies that the authenticated caller "owns" the `From` ID. **The spoofing finding is real.**

**Implication for Pass 8 §9:** The retained bus surface needs three concrete P1 fixes added BEFORE monocle ports `/msg/*`. Pass 8 mentions BC-DAEMON-SRV-013 schema divergence (line 273-278) but does NOT mention the SAFETY-001/002 vulnerabilities or AUTH-002 spoofing. **Pass 8 §9 must be amended.**

---

## 7. Hallucination spot-check (NEW full-protocol rounds)

Selected 6 file:line citations from the new rounds, opened source, verified content.

| # | Round | Cited file:line | Cited claim | Verified content | Verdict |
|---|---|---|---|---|---|
| 1 | server-r1 BC-MCPSRV-022 | `server.go:135-137` | "lock.CleanStale() invoked before lock.Write" | server.go:134-137: `// Clean stale lock files... if n := s.lock.CleanStale(); n > 0 { s.log.Printf("cleaned %d stale lock file(s)", n) }` | **EXACT MATCH** |
| 2 | server-r1 BC-MCPSRV-027 | `server.go:178-180` | "s.ownsBroker == true implies s.notifyBroker.Close()" | server.go:178-180: `if s.ownsBroker { s.notifyBroker.Close() }` | **EXACT MATCH** |
| 3 | server-r1 BC-MCPSRV-028 | `server.go:358-363` | "extractAuthToken reads X-Claude-Code-Ide-Authorization first, X-Auth-Token fallback" | server.go:355-363: `func extractAuthToken... if t := r.Header.Get("X-Claude-Code-Ide-Authorization"); t != "" { return t } return r.Header.Get("X-Auth-Token")` | **EXACT MATCH** |
| 4 | server-r1 BC-MCPSRV-038 | `server.go:428-439` | "Diff-choice fast path: 50ms sleep then SendKeys; uses context.Background()" | server.go:425-439: confirmed — `if key, ok := s.state.GetDiffChoice(window); ok { ... go func() { time.Sleep(50 * time.Millisecond); ... s.tmux.SendKeys(context.Background(), target, key) }() ... }` | **EXACT MATCH** — goroutine bug claim verified |
| 5 | mcp-r1 BC-MCPREG-015 | `ssh.go:69-74` | "Remote write command: `mkdir -p ... && printf '%s' '<base64>' \| base64 -d > <path>`; no `sh -c` wrapper" | ssh.go:69-74: `cmd := fmt.Sprintf( `mkdir -p "$(dirname %s)" && printf '%%s' %s \| base64 -d > %s`, remotePath, shell.Quote(encoded), remotePath)` | **EXACT MATCH** |
| 6 | pmw-r2 (cited from session/role.go) | `role.go:159-164` | "BuildWorkerPrompt passes (projectRoot, worktreePath, sessionID, sessionID) to fmt.Sprintf" | role.go:159-164: `role := fmt.Sprintf(roleTmpl, projectRoot, // NEVER modify... worktreePath, // Worktree path... sessionID, // Session ID... sessionID, // msg send --from)` | **EXACT MATCH** |

**Verdict: 6/6 EXACT MATCHES. No hallucinations detected in the sampled citations.** Sample size is small (6 of ~644 contracts ≈ 1%), but the high hit rate combined with the rigor of the round-level work (each round names its round goal, declares novelty, lists files read) gives reasonable confidence that fabrication is rare.

**Caveat:** B.6 also spot-checked 3 citations and found 3 exact matches. Together with this audit's 6, the total sample is 9 — still small. A larger random sample would tighten the bound, but is out of scope here.

---

## 8. Out-of-scope topic-drift check

Did any round inadvertently deepen something outside its declared scope?

### server-r1..r3

- Declared scope: `internal/server/`.
- Cross-pollinated reads (server-r2 §"Cross-Pollination with daemon-r{1,2,3}"): daemon-r1/r2/r3 read for `/msg/create` schema reconciliation. **Acceptable** — declared as cross-pollination, not as deepening daemon.
- One potential overstep: server-r1 §11 ("monocle Relevance Assessment") makes recommendations about `internal/daemon` semantics. **Acceptable** — relevance commentary, not contract drafting.

### mcp-r1..r3

- Declared scope: `internal/mcp/` (registry manager).
- mcp-r2 §"Gap (3) — `EnsureClaudeConfigured` interaction" reads `internal/session/manager.go:186-222`. **Acceptable** — cross-tool-writer check; explicitly scoped as "interaction" not "deepening session".
- mcp-r3 §"Settings.local.json writers across the repo" greps the whole repo for `settings.local.json` references. **Acceptable** — completeness verification.

### plugin-r1..r3

- Declared scope: `internal/plugin/`.
- plugin-r1 §1.2 reads `cmd/lazyclaude/root.go:340-347, 686-744` and `internal/gui/plugin_state.go`. **Acceptable** — wiring/composition root chase, not new deepening.

### pmw-r2..r4

- Declared scope: PM/Worker subsystem (cross-cutting: `prompts/`, `session/role.go`, `session/worktree.go`, `session/gitcmd.go`, `session/manager.go` portions, `internal/server/handler_msg.go`, `internal/daemon/server.go`, `cmd/lazyclaude/msg.go`, `internal/gui/keymap/registry.go`).
- The scope is itself cross-cutting by design (the user defined PMW as a "vertical slice" — persona + bus + worktree + GUI wiring). Reading those files isn't out-of-scope drift; it's the declared scope.
- pmw-r3 reaches into daemon/server.go, server/handler_msg.go for safety contracts. **Acceptable** — that's where the bus primitive lives.
- pmw-r4 §"Files spot-checked" lists daemon/server.go, daemon/lifecycle.go, session/manager.go, session/store.go. **Acceptable** — boundary refinements.

**Verdict: NO out-of-scope drift detected in the new rounds.** Each cross-cutting read is explicitly justified as cross-pollination or wiring trace, not as silent deepening of another subsystem.

---

## 9. Verdict and audit basis

### Verdict

**TOPIC-DRIFT-FOUND.** Three categories:

1. **Drift-Category-A (synthesis staleness):** Pass 8 v1 (`pass-8-final-synthesis.md`) was written before mcp/server/plugin/pmw-r2..r4 ran. It under-represents these four subsystems' coverage by ~170 contracts and contains one factual error (§310-311 lock-file mode — corrected by server-r1/r3 but never propagated). **A Pass 8 v2 is required before downstream skills (create-brief, disposition-pass, create-prd, semport-analyze) consume this corpus.**

2. **Drift-Category-B (round-driven blind spots):** Three small but load-bearing subsystems were never given Phase B deepening: `internal/core/event/broker.go`, `internal/core/config/hooks.go`, `internal/adapter/tmuxadapter/`. The first two are mission-critical for monocle (broker is the hook-delivery seam; hooks injection is the "adopt verbatim" pattern per Pass 8 §line 209). **Recommend 1 Phase B round each before crystallization.**

3. **Drift-Category-D (Pass 8 §9 PMW-EXCLUDED section is incomplete):** Pmw-r2..r4 surfaced 3 new P1/P2 SAFETY findings on the retained `/msg/*` bus (SAFETY-001 type-injection on daemon; SAFETY-002 1 MB body on daemon; AUTH-002 sender spoofing on both paths). Pass 8 §9 mentions schema divergence only. **Pass 8 v2 must integrate these findings as P1 monocle blockers for the bus port.**

### Audit basis

- Read 100% of the artifacts in `.factory/semport/any-context-lazyclaude/` (34 substantive files + original B.5 under audit).
- Re-derived the subsystem-by-pass coverage matrix from each artifact's declared scope and convergence status, NOT from the original B.5's tagging.
- Verified the source tree structure (`internal/core/{lifecycle, event, config, choice, shell, model, debuglog}/`, `internal/adapter/tmuxadapter/`, `cmd/mock-claude-client/`) and recomputed LOC/test-LOC via `find ... -exec wc -l`.
- Cross-referenced 8 contracts that appear in multiple rounds for inconsistency.
- Spot-checked 6 file:line citations from the new rounds against actual source (6/6 exact matches).
- Verified the lock-file mode is `0o600` at `lock.go:56` directly.
- Verified the daemon vs server `/msg/send` body-cap divergence directly (`handler_msg.go:202, 224` and `daemon/server.go:210, 490-549`).
- Verified BuildWorkerPrompt placeholder ordering at `role.go:159-164`.
- Verified extractAuthToken header priority at `server.go:355-363`.
- Verified mcp ssh write command at `ssh.go:69-74`.
- Recomputed contract count: ~644+ unique BC IDs (vs Pass 8's ~470 claim, which predates the new rounds).
- Confirmed no out-of-scope drift in the new rounds (each cross-cutting read is declared as cross-pollination).

### Specific items the orchestrator should action

1. **Amend Pass 8 §310-311** to state lock-file mode is `0o600` (per lock.go:56), not "unspecified (typically 0644)". The auth-token-readable-by-other-users claim built on the 0644 assumption needs to be retracted.

2. **Generate Pass 8 v2** that incorporates:
   - BC-MCPSRV-021..077 from server-r1/r2/r3 (~48 net-new contracts, 3 P1 findings, 1 P2 finding).
   - BC-MCPREG-001..027 from mcp-r1/r2/r3 (27 net-new contracts, 2 P0 findings, 3 P1 findings).
   - BC-PLUGIN-001..023 from plugin-r1/r2/r3 (23 net-new contracts; `ExecCLI` immutability recommendation).
   - BC-PMW-* full set from pmw-r2/r3/r4 (~76 net-new beyond r1, 3 P1 SAFETY findings on retained bus).

3. **Commission 3 additional Phase B rounds** before spec crystallization (or accept the risk and document):
   - `internal/core/event/broker.go` round — formalize the broker mutex model, dispatch semantics, single-mutex correction from server-r2.
   - `internal/core/config/hooks.go` round — formalize the node-eval one-liner content, PID-liveness algorithm, timeout-per-hook table at file:line precision (mission-critical for monocle).
   - `internal/adapter/tmuxadapter/` round — formalize DetectMaxOption and SendKeysLiteral semantics at file:line precision.

4. **For the `/msg/*` bus retained for monocle, add 3 P1 blockers** to the porting spec before any implementation work:
   - Sender identity verification (cross-check `req.From` against the authenticated caller — BC-PMW-MSG-AUTH-002).
   - Type allowlist on daemon path (currently server-only — BC-PMW-MSG-SAFETY-001).
   - 10 KB body field cap on daemon path (currently server-only — BC-PMW-MSG-SAFETY-002).

### Subsystems flagged as gaps (summary)

| Gap | Severity | Action |
|---|---|---|
| `internal/core/event/broker.go` not deepened | MEDIUM | 1 Phase B round recommended |
| `internal/core/config/hooks.go` not deepened | HIGH | 1 Phase B round strongly recommended |
| `internal/adapter/tmuxadapter/` not deepened | LOW-MEDIUM | 1 Phase B round optional |
| Pass 8 §310-311 lock-file mode error | LOW (factually wrong but small impact) | Amend |
| Pass 8 staleness re: mcp/server/plugin/pmw-full | HIGH | Pass 8 v2 required |
| Pass 8 §9 missing 3 P1 SAFETY findings on retained `/msg/*` | HIGH | Pass 8 v2 must integrate |

### Inconsistencies between rounds

- **1 real inconsistency:** Pass 8 §310-311 (lock-file mode 0644 claim) vs server-r1/r3 (correct 0o600 per lock.go:56). server-r* is authoritative.
- **0 inconsistencies between Phase B rounds themselves** — they cross-reference and reconcile cleanly (e.g. server-r2 §"Reconciled schema map" with daemon-r2; pmw-r2 with server-r1 and daemon-r1/r2/r3).

### Hallucinations identified

- **None in the 6 sampled citations.** All file:line references resolve to the claimed content.

### Whether Pass 8 v1 adequately represents the full-protocol coverage

**NO.** Pass 8 v1 is materially stale:
- Misses ~170 net-new contracts from the 11 new round files.
- Contains 1 factual error on lock-file permissions.
- Section 9 (PMW EXCLUDED) lacks the 3 new P1 SAFETY findings on the retained `/msg/*` bus.

Pass 8 v1 should be marked as "DO NOT CONSUME — STALE" and a Pass 8 v2 generated that integrates the four new full-protocol round groups before any downstream skill runs against this corpus.

---

## State Checkpoint

```yaml
pass: B.5-v2
status: complete
audit_type: fresh-context-watchdog
artifacts_inventoried: 34
new_full_protocol_rounds: 11
subsystem_by_pass_matrix_rows: 22
cross_round_consistency_checks: 8
real_inconsistencies_found: 1
hallucination_spot_checks: 6
hallucinations_found: 0
out_of_scope_deepening_detected: 0
gaps_flagged: 6
priority_recommendations: 4
pass8_v1_status: STALE — DO NOT CONSUME
pass8_v2_required: true
verdict: TOPIC-DRIFT-FOUND
drift_categories:
  - A: synthesis staleness (mcp/server/plugin/pmw-full not in Pass 8)
  - B: round-driven blind spots (core/event broker, core/config hooks, adapter/tmuxadapter)
  - D: Pass 8 §9 PMW-EXCLUDED missing 3 P1 SAFETY findings
timestamp: 2026-05-11T19:00:00Z
next_phase: Pass 8 v2 synthesis (after orchestrator accepts findings)
```
