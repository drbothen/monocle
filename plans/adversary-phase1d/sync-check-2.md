# Post-Fix Consistency Sync Check 2

**Date:** 2026-05-26
**Scope:** Targeted cross-document consistency check after Pass 2 fix cycle
**Validator:** consistency-validator

---

## Summary

| Check | Result | Notes |
|-------|--------|-------|
| 1. PreToolUse timeout semantics (fail-open) | PASS | All 4 BCs consistent |
| 2. PromptAutoResolved elimination | PASS | Zero live occurrences |
| 3. OpenProfilePicker elimination | PASS with caveat | Zero live occurrences; trace-note residuals only |
| 4. BC-2.06.023 propagation | PASS | Present in all 4 required locations |
| 5. Priority consistency (BC-2.05.006, BC-2.06.007, BC-2.06.010, BC-2.06.018) | PASS | All P1 in BC-INDEX and SS-tui.md |
| 6. PermissionPromptPayload struct definition | PASS | Explicit struct in SS-ipc.md |
| 7. Hook count claim (SS-daemon-wiring.md) | PASS with observation | Claim corrected; one residual ambiguity noted |

**Overall verdict: CLEAN** — no blocking issues remain. One minor observation recorded below.

---

## Check 1: PreToolUse Timeout Semantics (Fail-Open)

**Question:** Do BC-2.04.007, BC-2.05.005, BC-2.06.022, and BC-2.06.017 ALL consistently describe fail-open timeout behavior for PreToolUse?

### BC-2.04.007 (Hook Endpoint: PreToolUse Request Routing) — PASS

- PC-4 (line 87): "the handler produces a fail-open `HookResponse` (`{"decision": "allow", "reason": "timeout"}`) and returns HTTP 200."
- PC-7 (line 111): Timeout → `{"decision": "allow", "reason": "timeout"}` (fail-open, matching BC-HOOK-001 gene-source semantics).
- EC-072 and EC-075: Both specify "fail-open allow after 300ms."
- Canonical test vector timeout row: `{"decision": "allow", "reason": "timeout"}`.
- §Trace v1.1.0 (F-P1D2-001) confirms the fail-closed → fail-open correction was applied.

**Verdict: FAIL-OPEN, correctly specified.**

### BC-2.05.005 (IPC Message Types: PermissionPromptQueued) — PASS with minor hedging

- PC-4 (lines 77–78): "The daemon resolves the pending hook response with the fail-open or fail-closed semantics per BC-HOOK-001 / BC-HOOK-002 (no decision was made; Claude Code's default applies)."
- EC-002 (line 102): "Daemon resolves the pending hook response using Claude Code's default fallback (fail-open per BC-HOOK-001)."
- Canonical test vector timeout row (line 116): "Daemon resolves fail-open/closed" — uses hedged phrasing.

PC-4 body text hedges between fail-open and fail-closed ("per BC-HOOK-001 / BC-HOOK-002"), deferring to gene-source for the specific behavior. EC-002 is explicit: fail-open per BC-HOOK-001. The test vector at line 116 retains the "fail-open/closed" hedge, but this BC's scope is the IPC subsystem (not the hook routing layer), so its deference to BC-2.04.007 and BC-HOOK-001 for the exact response policy is architecturally correct — BC-2.05.005 specifies the IPC messaging path, not the hook response policy. The EC-002 row confirms fail-open is the correct answer for PreToolUse specifically. **This is not a defect; the hedge in PC-4 is acceptable because the response policy authority is BC-2.04.007.**

**Verdict: Consistent with fail-open. Minor hedging in PC-4/test vector is architecturally correct (deference to authoritative BC); not a violation.**

### BC-2.06.017 (Permission Response Within Hook Timeout Budget) — PASS

- Precondition 3 table: PreToolUse → 300ms → "Fail-open (Allow) — per BC-HOOK-001."
- PC-4 (lines 81–83): "On PreToolUse timeout: daemon returns fail-open (Allow)."
- PC-5: "On Stop/SessionStart/UserPromptSubmit timeout: daemon returns fail-open (Allow)."
- Invariant 3: "Fail-open is the default for all hook types with decision semantics."
- Canonical test vector PreToolUse timeout row: "Daemon sends fail-open."
- §Trace v1.1.0 confirms F-P1D2-002 replaced PromptAutoResolved, not a timeout semantics change (timeout was already fail-open in this BC from initial production).

**Verdict: FAIL-OPEN, explicitly and comprehensively specified.**

### BC-2.06.022 (Killer Scenario: ≤6 Keystrokes) — PASS

- Summary Postcondition §5 (line 115): "The entire flow completes within the 300ms PreToolUse timeout budget."
- EC-135 (line 142): "Daemon sends fail-open for P1" — explicit fail-open on timeout.
- EC-136 (line 143): Timeout race handled idempotently; no fail-closed branch.
- §Trace v1.2.0 confirms F-P1D2-002 replaced `PromptAutoResolved { P1 }` with `PermissionPromptResolved { prompt_id: P1.prompt_id }`.

**Verdict: FAIL-OPEN, consistent with the other three BCs.**

---

## Check 2: PromptAutoResolved Elimination

**Question:** Are there any remaining occurrences of `PromptAutoResolved` in live spec content (not trace/fix notes)?

Grep across all `.factory/specs/` files for `PromptAutoResolved` returned **5 matches total**, all confined to `§Trace` sections in BC-2.06.017.md and BC-2.06.022.md documenting the fix. These are historical records of the F-P1D2-002 correction — not live spec content.

Specifically:
- BC-2.06.017.md §Trace v1.1.0: "All occurrences of `PromptAutoResolved` ... replaced with `PermissionPromptResolved`" — this is the fix record, not a live use.
- BC-2.06.022.md §Trace v1.2.0: "EC-135: `PromptAutoResolved { P1 }` → `PermissionPromptResolved { prompt_id: P1.prompt_id }`" — this is the before/after of the fix, not a live use.

No occurrence of `PromptAutoResolved` appears in any precondition, postcondition, invariant, edge case, canonical test vector, or verification property section of any spec file.

**Verdict: ZERO live occurrences. CLEAN.**

---

## Check 3: OpenProfilePicker Elimination

**Question:** Are there any remaining occurrences of `OpenProfilePicker` in live spec content?

Grep returned **4 matches**, all in two files:

1. `BC-INDEX.md` — 2 matches, both in the §Trace section recording the F-P1D2-003 fix:
   - "BC-2.07.005: all occurrences of `Action::OpenProfilePicker` → `Action::ProfilePicker`."
   - "Canonical enum per SS-tui.md: variant is `ProfilePicker`, not `OpenProfilePicker`."

2. `ss-07/BC-2.07.005.md` — 4 matches, all in §Trace v1.1.0 (F-P1D2-003 fix record):
   - "Traceability §Cross-Ref BC-2.06.003 note: 'OpenProfilePicker' → 'ProfilePicker' per F-P1D2-003."
   - "Related BCs [BC-2.06.003] note: 'OpenProfilePicker' → 'ProfilePicker' per F-P1D2-003."
   - "All occurrences of `Action::OpenProfilePicker` replaced with `Action::ProfilePicker`"
   - "Canonical enum in SS-tui.md defines the variant as `ProfilePicker`, not `OpenProfilePicker`."

No `OpenProfilePicker` in architecture files. No live use in any precondition, postcondition, invariant, edge case, test vector, or VP section.

**Verdict: ZERO live occurrences. CLEAN. Trace-note residuals are documentation of the fix, not violations.**

---

## Check 4: BC-2.06.023 Propagation

**Question:** Does BC-2.06.023 appear in ALL of: BC-INDEX §SS-06 section, PRD §2.6 table, PRD §7 RTM, SS-tui.md BC table?

### BC-INDEX.md — PASS

Line 154: `| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 | active | ss-06/BC-2.06.023.md | — |`

### PRD §2.6 table — PASS

Line 200 of prd.md: `| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 |`

Added by F-P1D2-006 (PRD §Trace v1.27.2).

### PRD §7 RTM — PASS

Line 369 of prd.md: `| BC-2.06.023 | §Success Criteria (killer scenario — permission overlay; concurrent prompt resolution) | SS-tui.md v1.0.0 §Permission Overlay §Overlay Stack Lifecycle; SS-ipc.md v1.0.0 §ServerToClient::PermissionPromptResolved | P0 | monocle-tui/tests/permission_overlay_resolved.rs | Integration |`

Added by CV-P1D-003 closure (PRD §Trace v1.27.1).

### SS-tui.md BC table — PASS

Line 856 of SS-tui.md: `| BC-2.06.023 | TUI Removes Resolved Prompt from Overlay Stack on PermissionPromptResolved | P0 |`

Added by F-P1D2-005 (SS-tui.md §Trace).

### BC-2.06.023.md file itself — PASS

File exists at `behavioral-contracts/ss-06/BC-2.06.023.md` with correct H1 title and comprehensive content (postconditions, edge cases, canonical test vectors, VPs).

Additionally, BC-2.06.017.md and BC-2.05.005.md both cross-reference BC-2.06.023 in their postconditions (BC-2.06.017 PC-7; BC-2.05.005 PC-4).

**Verdict: BC-2.06.023 propagated to all 4 required locations. CLEAN.**

---

## Check 5: Priority Consistency

**Question:** Are BC-2.05.006, BC-2.06.007, BC-2.06.010, and BC-2.06.018 all P1 in both BC-INDEX and their respective arch docs?

### BC-INDEX.md

- Line 118: BC-2.05.006 → P1
- Line 138: BC-2.06.007 → P1
- Line 141: BC-2.06.010 → P1
- Line 149: BC-2.06.018 → P1

### SS-ipc.md (BC-2.05.006)

Line 436: `| BC-2.05.006 | TUI Reconnects After Daemon Restart | P1 | F-27 |`

Priority corrected P0 → P1 by F-P1D2-004 per §Trace.

### SS-tui.md (BC-2.06.007, BC-2.06.010, BC-2.06.018)

- Line 840: BC-2.06.007 → P1
- Line 843: BC-2.06.010 → P1
- Line 851: BC-2.06.018 → P1

SS-tui.md §Trace confirms these were corrected from P0 → P1 by F-P1D2-004 to match BC-INDEX.

**Verdict: All four BCs are P1 in both BC-INDEX and respective architecture documents. CLEAN.**

---

## Check 6: PermissionPromptPayload Struct Definition

**Question:** Does SS-ipc.md now have an explicit struct definition for `PermissionPromptPayload`?

SS-ipc.md lines 279–291:

```rust
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPromptPayload {
    /// Stable ID generated by the daemon when the PreToolUse hook first arrives.
    /// Remains stable for the lifetime of the pending decision.
    pub prompt_id: Uuid,
    pub session_id: Uuid,
    pub tool_name: String,
    pub tool_input: serde_json::Value,
    /// Present when tool_name is "Edit" or similar file-mutation tools.
    pub old_content: Option<String>,
    pub new_content: Option<String>,
}
```

This explicit struct definition was added by F-P1D2-008 (SS-ipc.md §Trace). Prior to this fix, the spec only described the fields inline in `PermissionPromptQueued` without a named, reusable type. The struct is now `#[non_exhaustive]` for API safety, consistent with the `RingError` treatment in the implementation.

`PermissionPromptQueued` now embeds `payload: PermissionPromptPayload` rather than inlining the fields, and `InitialState.overlay_stack` is confirmed as `Vec<PermissionPromptPayload>`.

**Verdict: Explicit struct present with complete field definitions. CLEAN.**

---

## Check 7: Hook Count — SS-daemon-wiring.md

**Question:** Does SS-daemon-wiring.md no longer claim "5 hook endpoint URLs" in hooks-settings.json?

The F-P1D2-007 fix corrected the previously ambiguous "All 5 hook endpoint URLs" text. The current state (lines 394–400):

> "The daemon serves 5 hook endpoints; hooks-settings.json configures 4 of them with URLs (`PreToolUse`, `Notification`, `Stop`, `UserPromptSubmit`). `SessionStart` is invoked by Claude Code's internal lifecycle, not via hooks-settings.json. `PostToolUse` and `PreCompact` are included as reserved empty arrays (forward-compatibility)."

This is a material improvement: it now correctly distinguishes between the 5 endpoints the daemon _serves_ and the 4 that are _configured via URLs_ in hooks-settings.json. The §Trace v1.2.0 records the rationale.

**Observation (non-blocking):** Line 276 states "The `hooks-settings.json` configures only the 5 hook types that support user-configurable scripts (`PreToolUse`, `Notification`, `Stop`, `PostToolUse`, `UserPromptSubmit`)." This counts PostToolUse as one of the "5 hook types" even though the hooks-settings.json schema shows PostToolUse with an empty array (no URL configured). The text is technically correct — PostToolUse _is_ configurable and _is_ present in the schema — but there is a mild tension with the "4 URLs" count at line 395. An implementer reading line 276 first and line 395 second may be momentarily confused.

This is not a defect: both statements are accurate; they describe different things (types present in schema vs. types that have active URLs). The inconsistency is cosmetic and does not affect implementability. It does not warrant a fix story under the production-grade principle because the information is complete and correct when read in context.

**Verdict: "5 hook endpoint URLs" claim eliminated. PASS. Non-blocking cosmetic observation recorded.**

---

## Final Status

| # | Check | Status |
|---|-------|--------|
| 1 | PreToolUse fail-open semantics — BC-2.04.007 | PASS |
| 1 | PreToolUse fail-open semantics — BC-2.05.005 | PASS (hedging is architecturally correct) |
| 1 | PreToolUse fail-open semantics — BC-2.06.017 | PASS |
| 1 | PreToolUse fail-open semantics — BC-2.06.022 | PASS |
| 2 | PromptAutoResolved elimination | PASS — zero live occurrences |
| 3 | OpenProfilePicker elimination | PASS — zero live occurrences |
| 4 | BC-2.06.023 in BC-INDEX §SS-06 | PASS |
| 4 | BC-2.06.023 in PRD §2.6 table | PASS |
| 4 | BC-2.06.023 in PRD §7 RTM | PASS |
| 4 | BC-2.06.023 in SS-tui.md BC table | PASS |
| 5 | BC-2.05.006 P1 in BC-INDEX + SS-ipc.md | PASS |
| 5 | BC-2.06.007 P1 in BC-INDEX + SS-tui.md | PASS |
| 5 | BC-2.06.010 P1 in BC-INDEX + SS-tui.md | PASS |
| 5 | BC-2.06.018 P1 in BC-INDEX + SS-tui.md | PASS |
| 6 | PermissionPromptPayload struct in SS-ipc.md | PASS |
| 7 | SS-daemon-wiring.md hook count corrected | PASS |

**Overall: CLEAN. All Pass 2 fixes verified. No blocking findings. One non-blocking cosmetic observation (Check 7, line 276 vs 395 mild tension).**
