---
document_type: adr
adr_id: ADR-0005
status: accepted
date: 2026-05-17
subsystems_affected: ["SS-01"]
supersedes: null
superseded_by: null
level: L3
section: "adr"
version: "1.0.2"
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-17T19:00:00Z
inputs: [product-brief.md, architecture/SS-daemon-lifecycle.md, behavioral-contracts/ss-01/BC-2.01.009.md, semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md]
input-hash: "1119b6f"
traces_to: "T-128m dispatch; BC-HOOK-016 deep ingest; CAP-001 v1.2 §P2; F-R105-6 + GAP-R44-2 BA closure"
project: monocle
---

# ADR-0005: Auth Header Dual-Accept — Canonical `X-Monocle-Authorization` with `X-Claude-Code-Ide-Authorization` Compatibility Alias

## Status

Accepted

## Context

### The Interop Problem

monocle's daemon requires `X-Monocle-Authorization: monocle-v1:<64-hex>` on all
authenticated endpoints (BC-2.01.009, SS-daemon-lifecycle.md §Start Sequence).
This is monocle's canonical project-scoped auth header — intentionally distinct
from Claude Code's header, since monocle is its own product, not a Claude Code
fork.

Real Claude Code's hook scripts (the primary Phase 1 harness) have the header name
`X-Claude-Code-Ide-Authorization` **hardcoded** in Go source (`hooks.go:31`). This
is confirmed by deep ingest BC-HOOK-016:

> "The auth header name is `X-Claude-Code-Ide-Authorization` (NOT `X-Auth-Token`)"
> Evidence: hooks.go:31 `'X-Claude-Code-Ide-Authorization': srvToken`

The header name is NOT configurable by the user. Claude Code reads the auth token
from the daemon's lock file `authToken` field (raw 64-char hex, no prefix) and sends
it verbatim in `X-Claude-Code-Ide-Authorization`. It never sends `X-Monocle-Authorization`.

Without a compatibility decision, real Claude Code hooks cannot authenticate to the
monocle daemon:
- Claude Code sends: `X-Claude-Code-Ide-Authorization: <64-hex>` (raw token, no prefix)
- monocle router requires: `X-Monocle-Authorization: monocle-v1:<64-hex>` (prefixed)
- Result: HTTP 401 `{"error":"missing_auth_token"}` on every real Claude Code hook call

### Architecture Background

SS-daemon-lifecycle.md §Body Limit and Router Design (lines 141-172) describes a
two-router architecture. Lines 147-149 say:

> "the Claude Code IDE token (`X-Claude-Code-Ide-Authorization`) is checked
> per-handler inside the hook handlers, not as a separate router-level layer,
> because the IDE token is optional and absent on non-hook requests."

This description implies `X-Claude-Code-Ide-Authorization` is an optional, secondary
check — but that model does not solve the primary auth gap: Claude Code cannot pass the
router-level `X-Monocle-Authorization` middleware before reaching any hook handler.
The per-handler description is architecturally under-specified; this ADR resolves it.

### Domain Invariant DI-005

DI-005 states: "A monocle daemon MUST NOT accept an auth token that does not begin
with the canonical prefix for its version."

DI-005 governs the **token prefix format** (`monocle-v1:`) — not the HTTP header name.
Dual-accept is consistent with DI-005 provided monocle-aware tools always present the
prefixed form in `X-Monocle-Authorization`. Real Claude Code sends the raw 64-hex
token (no prefix) in `X-Claude-Code-Ide-Authorization`; the daemon must handle the
format difference at the compatibility layer (see §Decision below).

## Decision

**Option (a): Dual-accept at router middleware** with canonical `X-Monocle-Authorization`.

### Auth Middleware Dual-Accept Protocol

The router-level auth middleware for the authenticated router (hook endpoints +
`/status` + `/shutdown`) is revised to check headers in the following priority order:

**Priority 1 — Canonical (monocle-aware tools):**
If `X-Monocle-Authorization` is present:
- Validate with prefix check: value MUST begin with `monocle-v1:`.
- Strip prefix; constant-time compare hex suffix against stored secret.
- On success: proceed. On failure: HTTP 401 per BC-2.01.009 two-body taxonomy.

**Priority 2 — Compatibility alias (real Claude Code hooks):**
If `X-Monocle-Authorization` is absent AND `X-Claude-Code-Ide-Authorization` is present:
- Emit deprecation log: `WARN: hook auth via X-Claude-Code-Ide-Authorization (compatibility alias); monocle-aware harness should use X-Monocle-Authorization`
- Validate value as raw 64-hex (no prefix — Claude Code sends the lock file `authToken` field verbatim, which has no prefix).
- Constant-time compare against stored secret.
- On success: proceed. On failure: HTTP 401 `{"error":"invalid_auth_token"}`.

**Neither header present:**
- Return HTTP 401 `{"error":"missing_auth_token"}` (unchanged from BC-2.01.009).

## Rationale

| Criterion | Assessment |
|-----------|-----------|
| Phase 1 scope (Claude Code first) | Dual-accept is the ONLY option that makes real Claude Code work without changing Claude Code. Option (c) is infeasible; option (b) adds process complexity with zero security benefit. |
| DI-005 compliance | DI-005 governs token prefix format, not header name. Dual-accept canonical (prefix required) + alias (raw hex) satisfies DI-005: the canonical path enforces the prefix; the alias path cannot enforce the prefix because Claude Code sends raw hex, but the token value is still validated by constant-time comparison. |
| Security posture | Both code paths perform constant-time secret comparison. The compatibility alias emits a WARN-level deprecation log (observable in structured logs). Same security guarantee; no new attack surface. Daemon binds 127.0.0.1; same-user process assumption holds. |
| Future-extensibility (multi-harness Phase 3+) | When CodeMachine or other harnesses are added (Phase 3+), they can be onboarded via `X-Monocle-Authorization` (canonical). Legacy Claude Code retains its alias. No new ADR required for onboarding canonical-header harnesses. |
| Lowest complexity | No new components. One additional header check in the existing auth middleware. |
| BC-2.01.009 impact | BC-2.01.009 postcondition 1 "Missing header" semantics expand: "missing" means BOTH `X-Monocle-Authorization` AND `X-Claude-Code-Ide-Authorization` are absent. PO must update BC-2.01.009 to reflect dual-accept (see §BC Impact below). |

### Deprecation Timeline

The `X-Claude-Code-Ide-Authorization` compatibility alias is a **Phase 1 interop
necessity**, not a permanent first-class feature. Timeline:

| Phase | Status |
|-------|--------|
| Phase 1 | Alias active. WARN log on every use. DTU clone tests both paths. |
| Phase 3 | Multi-harness design may introduce a canonical hook-settings generator that monocle emits, configuring Claude Code-compatible wrappers to use `X-Monocle-Authorization`. Evaluate at Phase 3 architecture gate. |
| Phase 4 | Federation layer introduced. Alias re-evaluated. If upstream Claude Code adds a configurable header, alias can be removed with a version-gated deprecation cycle. |

### Lock File Interplay

monocle's lock file `authToken` field stores the raw 64-char hex secret (no prefix).
Real Claude Code reads `authToken` and sends it as-is in `X-Claude-Code-Ide-Authorization`.
The canonical path (`X-Monocle-Authorization`) expects `monocle-v1:<authToken>` — the
prefix is a **wire-format concern**, not a storage concern (per existing SS-daemon-lifecycle
design). Dual-accept resolves the format mismatch at the middleware layer without
changing the lock file contract.

### BC-2.01.009 Impact (PO Round 4 Surface)

BC-2.01.009 currently specifies `X-Monocle-Authorization`-only semantics. This ADR
changes postcondition 1:

- **Current:** "If the `X-Monocle-Authorization` header is absent entirely, return HTTP 401 `{"error":"missing_auth_token"}`"
- **Required update:** "If BOTH `X-Monocle-Authorization` and `X-Claude-Code-Ide-Authorization` are absent, return HTTP 401 `{"error":"missing_auth_token"}`"

Additionally, postconditions 2-3 require extension to cover the alias validation path
(invalid alias header → `{"error":"invalid_auth_token"}`).

The two-body error taxonomy (`missing_auth_token` / `invalid_auth_token`) is
**preserved**. Mapping:
- Neither header present → `missing_auth_token` (unchanged semantics)
- Either header present but fails validation → `invalid_auth_token` (extended to alias path)

**This ADR does NOT silently change BC-2.01.009.** It surfaces the required update to
PO for Round 4 follow-up. The BC update is a narrowing of the "missing" definition and
extension of validation to the alias path — not a fundamentally different contract.

### CAP-001 Compatibility Alias (BA Round 4 Surface)

CAP-001 v1.2 §P2 Hook Event Ingestion step 1 states:
> "A harness subprocess fires an HTTP POST to `POST /hooks/<type>` with the
> `X-Monocle-Authorization` header set to the token read from the lock file."

This is accurate for monocle-aware harnesses. For real Claude Code (Phase 1 primary),
the harness sends `X-Claude-Code-Ide-Authorization` instead. BA should update CAP-001
§P2 step 1 to document the dual-accept semantics and note the compatibility alias for
the Claude Code harness.

## Consequences

### Positive
- Real Claude Code hook calls authenticate to the monocle daemon without any Claude Code changes.
- `X-Monocle-Authorization` remains the project-canonical header; future harnesses use the canonical.
- WARN-level deprecation log provides visibility into alias usage without blocking.
- No additional processes, no configuration complexity.

### Negative
- Auth middleware has two code paths. Each path must be tested independently.
- The compatibility alias carries raw hex tokens (no `monocle-v1:` prefix), making the validation
  logic asymmetric between canonical and alias paths. This asymmetry must be clearly documented
  in the implementation and reviewed at Phase 3.

### Test Implications
- BC-2.01.009 integration test suite (`auth_header_rejection.rs`) must gain alias-path test vectors.
- DTU clone (`dtu-claude-code-hooks-v1`) tests the alias path (it sends `X-Claude-Code-Ide-Authorization`).
- A monocle-aware tool (e.g., CLI `monocle hook-test`) tests the canonical `X-Monocle-Authorization` path.

## Alternatives Considered

**(a) Dual-accept at router middleware** — Chosen. See §Decision above.

**(b) Translation shim (separate process):** A separate adapter process receives Claude
Code's header, validates, and re-issues `X-Monocle-Authorization` to the daemon. Daemon
only accepts canonical. Rejected: adds process complexity with zero security benefit over
option (a).

**(c) Reconfigure Claude Code:** Real Claude Code is configured (env var or settings)
to send `X-Monocle-Authorization`. Not feasible — BC-HOOK-016 confirms the header name
is hardcoded in Go source; no user-facing override exists.

**(d) Multi-harness generic header pattern:** Daemon accepts `X-Harness-{NAME}-Authorization`
where NAME is derived from session_id or lock file `app` field. Out-of-scope for Phase 1;
each harness brings unknown header conventions; premature generalization.

## Source / Origin

- **Behavioral contract:** `BC-2.01.009` (daemon auth header semantics) — this ADR extends its postconditions to cover dual-accept.
- **Deep ingest evidence:** `semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-hooks-r1.md` BC-HOOK-016 — confirms `X-Claude-Code-Ide-Authorization` is hardcoded at `hooks.go:31`; no user-facing override.
- **Architecture section:** `SS-daemon-lifecycle.md` lines 141-172 (router design) and lines 147-149 (IDE token description) — under-specified; this ADR resolves the ambiguity.
- **Originating context:** T-128m R105 closure chain Round 3 (2026-05-17); interop gap first surfaced by BA in T-128f scan; frontmatter `traces_to:` cross-reference: "T-128m dispatch; BC-HOOK-016 deep ingest; CAP-001 v1.2 §P2; F-R105-6 + GAP-R44-2 BA closure".

## §Trace v1.0.2

**F-R106-14 inputs path normalization — Round 5E** (2026-05-17T22:00:00Z):
- NORMATIVE: Frontmatter `inputs:` third entry corrected.
  SE-17f BEFORE: `specs/behavioral-contracts/ss-01/BC-2.01.009.md`
  SE-17f AFTER: `behavioral-contracts/ss-01/BC-2.01.009.md`
  Defect: spurious `specs/` prefix. All `inputs:` entries are relative to `.factory/specs/`;
  the other three entries (product-brief.md, architecture/SS-daemon-lifecycle.md, semport/…)
  correctly omit the `specs/` prefix. Normalized to match convention.
- SE-16d PASS: 2026-05-17T22:00:00Z > chain high-water 2026-05-17T20:30:00Z (monotonic).

## §Trace v1.0.1

**T-128q R4-005 LOW heading hierarchy normalization** (2026-05-17T20:30:00Z):
- SE-17f BEFORE: `### Rationale` at H3 under `## Decision`; `### Options Considered` at H3 under `## Context`; `## Source / Origin` absent.
- SE-17f AFTER: `## Rationale` promoted to top-level H2 (template canonical position between Decision and Consequences); `## Alternatives Considered` promoted to top-level H2 (template canonical name and position after Consequences); `## Source / Origin` added as new top-level H2 with provenance citation.
- SE-17c-d body-scope grep: all substantive content from the original `### Rationale`, `### Options Considered`, and inner subsections preserved verbatim — no text deleted, restructure only.
- SE-16d PASS: 2026-05-17T20:30:00Z > prior chain high-water 2026-05-17T19:00:00Z.

---

**T-128m architectural decision — F-R105 closure chain Round 3** (2026-05-17T19:00:00Z):
- NORMATIVE: ADR-0005 authored. Decision: dual-accept (option a).
  - Resolves interop gap surfaced by BA in T-128f: real Claude Code cannot send
    `X-Monocle-Authorization`; its header is hardcoded per BC-HOOK-016.
  - SS-daemon-lifecycle.md line 147 description updated from "optional per-handler
    IDE token check" to "router-level dual-accept middleware".
  - dtu-assessment.md: 10 `X-Claude-Code-Ide-Authorization` occurrences annotated
    with ADR-0005 compatibility alias rationale; no removals (DTU tests alias path).
  - BC-2.01.009 update: surfaced to PO for Round 4 (dual-accept semantics on
    postcondition 1; alias validation on postconditions 2-3).
  - CAP-001 compatibility alias update: surfaced to BA for Round 4 (§P2 step 1).
- SE-16d PASS: 2026-05-17T19:00:00Z > prior chain high-water 2026-05-17T18:00:00Z.
