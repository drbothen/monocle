---
document_type: behavioral-contract
level: L3
version: "1.1.0"
status: active
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T00:00:00Z
phase: 1a
inputs: [prd-expansion-scope.md, architecture/SS-tui.md, architecture/ARCH-INDEX.md]
input-hash: "e1ed8bb"
traces_to: prd.md
origin: greenfield
subsystem: SS-06
capability: CAP-006
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.2.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# Behavioral Contract BC-2.06.024: Permission Overlay: ToolPayload Body Rendering by Variant

## Description

When the permission overlay renders a `PromptModal`, the overlay body section shows
tool-specific content based on the `ToolPayload` variant stored in `PromptModal.tool_payload`.
For `ToolPayload::Bash`, the body displays the command string. For `ToolPayload::Read`, the
body displays the file path. For `ToolPayload::Generic`, the body displays the tool name and
a JSON excerpt of `tool_input`. For `ToolPayload::Edit`, the body invokes the diff renderer
(BC-2.06.010); Edit rendering is NOT in scope of this contract. This BC covers the three
non-Edit variants only.

Note: In Phase 1, `payload_to_modal()` produces `ToolPayload::Generic` for ALL `"Edit"` and
`"Write"` tool prompts because the daemon sends `old_content: None, new_content: None` for
every deferred prompt (rich diff content is S-027 scope). Therefore the `Generic` rendering
path is the predominant path for file-editing tools in Phase 1, not a rare fallback. This BC's
`Generic` rendering (PC-3) must correctly handle `tool_name` values of `"Edit"` and `"Write"`
— the `tool:` line shows the tool name verbatim, and the `input:` line shows the `tool_input`
JSON (which includes the path field), giving the user meaningful context about which file is
being modified.

## Preconditions

1. `AppMode` is `Overlay { prior }` and `App.overlay_stack.len() >= 1` (at least one `PromptModal` queued in `monocle-tui`'s `App.overlay_stack`).
2. The front `PromptModal` (`App.overlay_stack.front()`) has `tool_payload` that is one of:
   `ToolPayload::Bash { command }`, `ToolPayload::Read { path }`, or
   `ToolPayload::Generic { tool_name, tool_input }`.
3. The overlay layout has sufficient height to render at least the header (2 rows), the
   body label(s) (1–2 rows), and the hint line (1 row).

## Postconditions

### PC-1: Bash tool display

When `tool_payload == ToolPayload::Bash { command }`:

1. The overlay body renders a single label line:
   ```
   command: <command>
   ```
   where `<command>` is the full value of the `command` field.
2. The label `command:` is rendered in the default terminal color (no special styling).
3. The command value is rendered in the default terminal color (no special styling).
4. If `command` is empty (should not occur in practice — `payload_to_modal` falls back to
   `Generic` if the command field is absent — but if it is empty by the time it reaches the
   renderer), the body renders `command: (empty)` as a safe fallback. No panic.
5. Long `command` values wrap using `Wrap { trim: false }` so the full command is visible
   without truncation.

### PC-2: Read tool display

When `tool_payload == ToolPayload::Read { path }`:

1. The overlay body renders a single label line:
   ```
   path: <path>
   ```
   where `<path>` is the full value of the `path` field.
2. The label `path:` is rendered in the default terminal color.
3. The path value is rendered in the default terminal color.
4. If `path` is empty (same edge note as PC-1), the body renders `path: (empty)`. No panic.
5. Long paths wrap using `Wrap { trim: false }`.

### PC-3: Generic tool display

When `tool_payload == ToolPayload::Generic { tool_name, tool_input }`:

1. The overlay body renders two label lines:
   ```
   tool: <tool_name>
   input: <tool_input_excerpt>
   ```
2. `<tool_name>` is the full value of `tool_name`.
3. `<tool_input_excerpt>` is a compact JSON representation of `tool_input`, truncated at
   256 characters (the same bounding used elsewhere in the system). If `tool_input` is a
   JSON object or array, `serde_json::to_string(&tool_input)` is used. If serialization
   fails (should not occur for valid `serde_json::Value`), the body renders `(unrepresentable)`.
4. Both labels and values render in the default terminal color.
5. Long values wrap using `Wrap { trim: false }`.

### PC-4: Edit payloads are not handled by this BC

When `tool_payload == ToolPayload::Edit { ... }`, the rendering is delegated to the diff
renderer (BC-2.06.010). This BC's rendering logic MUST NOT be invoked for Edit payloads.
The overlay renderer branches on `tool_payload` variant BEFORE dispatching:
```rust
match &modal.tool_payload {
    ToolPayload::Edit { old_content, new_content, .. } => render_diff(old_content, new_content, frame, area),
    ToolPayload::Bash { command } => render_bash_body(command, frame, area),
    ToolPayload::Read { path } => render_read_body(path, frame, area),
    ToolPayload::Generic { tool_name, tool_input } => render_generic_body(tool_name, tool_input, frame, area),
}
```

## Invariants

1. Each `ToolPayload` variant maps to exactly one body rendering function. There is no
   shared rendering path between Bash, Read, Generic, and Edit body rendering.
2. The body rendering functions for Bash, Read, and Generic have no side effects: they
   do not modify `AppMode`, do not write to disk, and do not send IPC messages.
3. The `similar` crate (used by the Edit diff renderer) is NOT imported or called by the
   Bash, Read, or Generic body rendering functions. The purity boundary defined in
   BC-2.06.001 and BC-2.06.010 is preserved: `similar` is only used in the Edit path.
4. All three body rendering functions use `Wrap { trim: false }` so long values wrap
   without silent truncation that could hide critical tool arguments.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `ToolPayload::Bash` with a 500-character command string | Body wraps at terminal width; full command is visible with scroll; no truncation |
| EC-002 | `ToolPayload::Read` with a deeply nested absolute path (`/a/b/c/.../z`) | Path wraps at terminal width; fully visible; no panic |
| EC-003 | `ToolPayload::Generic` where `tool_input` is a large JSON object (>256 chars) | `tool_input_excerpt` is truncated at 256 characters; truncation does not split a UTF-8 character; rendered as `input: <256-char excerpt>` |
| EC-004 | `ToolPayload::Generic` where `tool_name` is a namespaced MCP tool (e.g., `mcp__perplexity__perplexity_search`) | Full `tool_name` displayed on the `tool:` line; no truncation of the tool name itself |
| EC-005 | `ToolPayload::Generic` where `tool_input` is `serde_json::Value::Null` | `serde_json::to_string` produces `"null"`; body renders `input: null` |
| EC-006 | `ToolPayload::Bash` arrives when terminal height is minimal (e.g., 8 rows) | Header (2 rows) + body (1 row) + hint (1 row) = 4 rows minimum; body is truncated to fit; no panic |
| EC-007 | `ToolPayload::Generic` where `tool_input` serialization produces an error | Body renders `input: (unrepresentable)` as a safe fallback; error is logged at `WARN` level via `tracing::warn!`; no panic |
| EC-008 | `ToolPayload::Generic` with `tool_name == "Edit"` (Phase-1 normal path: daemon sent None/None content) | PC-3 applies: renders `tool: Edit` and `input: <tool_input_excerpt>`. The `tool_input` JSON contains the path field, so the user sees which file is being edited. This is the correct Phase-1 behavior; the Edit diff variant is produced only when content is available (S-027+). |
| EC-009 | `ToolPayload::Generic` with `tool_name == "Write"` (Phase-1 normal path: daemon sent None/None content) | Same as EC-008: PC-3 applies; renders `tool: Write` and `input: <tool_input_excerpt>`. `Write` is a distinct Claude Code tool (full file creation/overwrite) handled identically to `Edit` in `payload_to_modal()`. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `ToolPayload::Bash { command: "cargo test --workspace" }` | Body renders `command: cargo test --workspace` | happy-path |
| `ToolPayload::Read { path: "/Users/alice/Dev/project/src/main.rs" }` | Body renders `path: /Users/alice/Dev/project/src/main.rs` | happy-path |
| `ToolPayload::Generic { tool_name: "mcp__perplexity__perplexity_search", tool_input: json!({"query":"rust async"}) }` | Body renders `tool: mcp__perplexity__perplexity_search` and `input: {"query":"rust async"}` | happy-path |
| `ToolPayload::Bash { command: <500-char string> }` | Body renders wrapped command; no truncation; no panic | edge-case |
| `ToolPayload::Generic { tool_name: "unknown_tool", tool_input: json!({"x": "y".repeat(300)}) }` | `tool_input_excerpt` truncated at 256 chars; valid UTF-8 boundary respected | edge-case |
| `ToolPayload::Edit { ... }` arrives at non-Edit renderer | Compiler-enforced: `match` arm for Edit calls `render_diff`, not the body functions in this BC | N/A (compile-time guarantee) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | `ToolPayload::Bash` body renders `command: <value>` label | unit test (render output inspection via ratatui `TestBackend`) |
| VP-TBD | `ToolPayload::Read` body renders `path: <value>` label | unit test |
| VP-TBD | `ToolPayload::Generic` body renders `tool: <name>` and `input: <excerpt>` | unit test |
| VP-TBD | Generic `tool_input_excerpt` is truncated at 256 characters at a valid UTF-8 boundary | unit test (inject 300-char input) |
| VP-TBD | No `similar` crate usage in Bash/Read/Generic rendering functions | static analysis (grep or semgrep) |
| VP-TBD | Empty command/path fields render safe fallback text without panic | unit test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 |
| Capability Anchor Justification | CAP-006 ("User-facing TUI; AppMode state machine; keybinding dispatch; sessions panel; event ribbon; permission overlay stack; Ctrl-\ popup integration") per ARCH-INDEX §Capability Traceability SS-06 — this BC specifies the visual rendering of the "permission overlay stack" for the three non-Edit ToolPayload variants (Bash command display, Read path display, Generic JSON excerpt display), which is the user-visible content of the overlay for the majority of Claude Code tool calls |
| L2 Domain Invariants | DI-007 (monocle MUST NOT write to any file owned by a harness or factory workflow system — satisfied: overlay body rendering is a pure read-only computation producing ratatui `Line` values; no file writes occur) |
| Architecture Module | monocle-tui (overlay.rs — render_bash_body, render_read_body, render_generic_body functions) per ARCH-INDEX SS-06 |
| Architecture Source | SS-tui.md v1.8.2 §Permission Overlay §IPC Payload to PromptModal Conversion (ToolPayload variant selection); SS-tui.md v1.8.2 §Permission Overlay §Diff Preview (Edit is a separate path — this BC covers the non-Edit branches) |
| Cross-Ref | BC-2.06.010 (Edit diff rendering — the fourth ToolPayload variant; this BC explicitly excludes it); BC-2.06.001 (purity boundary — similar is monocle-tui only, not used in Bash/Read/Generic rendering); BC-2.06.008 (PromptModal arrives via this push path); BC-2.06.011/012/013 (decision keys active regardless of which ToolPayload variant is displayed) |
| Test File | `monocle-tui/tests/overlay_payload_rendering.rs` |
| Test Name | `test_BC_2_06_024_tool_payload_body_rendering_by_variant` |
| Stories | S-027 (filled by story-writer) |

## Related BCs

- [BC-2.06.010] — PARTITION: BC-2.06.010 covers `ToolPayload::Edit` diff rendering; this BC covers the three remaining variants. Together they form a complete rendering partition over all `ToolPayload` variants.
- [BC-2.06.008] — depends on: `PromptModal` carrying any `ToolPayload` variant arrives via the push path defined in BC-2.06.008
- [BC-2.06.001] — depends on: purity boundary; `similar` crate not used in non-Edit rendering paths
- [BC-2.06.011] — composes with: user decides via Accept-Once regardless of which ToolPayload variant is displayed
- [BC-2.06.012] — composes with: user decides via Accept-Always regardless of ToolPayload variant
- [BC-2.06.013] — composes with: user decides via Reject regardless of ToolPayload variant

## Architecture Anchors

- `architecture/SS-tui.md#ipc-payload-to-promptmodal-conversion` — ToolPayload variant selection logic (Bash, Read, Generic, Edit arms)
- `architecture/SS-tui.md#permission-overlay` — PromptModal type definition with ToolPayload enum
- `architecture/SS-tui.md#diff-preview` — Edit branch (contrast: this BC covers the non-Edit branches)

## Story Anchor

S-027 — Overlay Rendering + Diff Preview + Status Bar (this BC covers the Bash, Read, and Generic display ACs in S-027)

## VP Anchors

- VP-TBD — Unit tests for Bash/Read/Generic body rendering (render output inspection, truncation boundary, safe fallback for empty fields)

## §Trace v1.1.0

**F-S026-ADV1-MED-001 — Write tool coverage + Phase-1 Generic-fallback documentation** (2026-05-31T00:00:00Z):
- Finding: BC-2.06.024 did not reference the `Write` tool (`tool_name == "Write"` is a distinct
  Claude Code tool for full file creation/overwrite, documented at `monocle-core/src/permissions.rs:181`
  and `monocle-ipc/src/types.rs:255`). Additionally, the Description did not explain that in Phase 1
  ALL Edit and Write prompts arrive with `old_content: None, new_content: None`, making
  `ToolPayload::Generic` the predominant rendering path for file tools — not a rare edge case.
- Fix: Description extended with a Phase-1 note explaining that `ToolPayload::Generic` with
  `tool_name == "Edit"` or `"Write"` is the normal Phase-1 path. The note clarifies that PC-3
  (Generic rendering) must correctly display the `tool_input` JSON (which includes the path),
  giving the user meaningful context about which file is being modified.
- Fix: EC-008 added — `ToolPayload::Generic` with `tool_name == "Edit"` (Phase-1 normal path).
- Fix: EC-009 added — `ToolPayload::Generic` with `tool_name == "Write"` (Phase-1 normal path).
- No changes to H1 title, Preconditions, PC-1/PC-2/PC-3/PC-4, or Invariants — this BC covers
  rendering; the conversion guard (`is_some() || is_some()`) is in BC-2.06.008/AC-016 scope.
- SE-16d monotonicity: v1.1.0 timestamp 2026-05-31T00:00:00Z > v1.0.2 (2026-05-29). PASS.

## §Trace v1.0.1

**Architect Pass 2 HIGH-003 propagation — `Overlay { stack, prior }` shape removed** (2026-05-28T00:00:00Z):
- Resolves F-S025-ADV3-BLOCKER-002. Precondition 1: `Overlay { stack, prior }` → `Overlay { prior }` with `App.overlay_stack.len() >= 1`. Precondition 2: `stack.front()` → `App.overlay_stack.front()`. The overlay stack is exclusively in `App.overlay_stack`; `AppMode::Overlay` no longer carries a `stack` field.
- SE-16d monotonicity: v1.0.1 timestamp 2026-05-28T00:00:00Z > v1.0.0. PASS.

## §Trace v1.0.2

**ADV23-SCOPE-002 — Architecture Source pin updated: SS-tui.md v1.6.0 → v1.8.2** (2026-05-29T00:00:00Z):
- Architecture Source: both `SS-tui.md v1.6.0` occurrences → `SS-tui.md v1.8.2` per F-S025-ADV23-MED-001 Category 8 cascade closure.
- Classification: Category A plain version-pin refresh. No substantive content changes required:
  - v1.8.0 (Overlay shape): already propagated in §Trace v1.0.1 above.
  - v1.8.1 (Sessions Panel 6→7 columns): this BC covers ToolPayload rendering (Bash/Read/Generic variants); no Sessions Panel column table in scope.
  - v1.8.2 (disconnect bracketed-tag style): no disconnect rendering in scope for this BC.
  - v1.6.0 → v1.8.2 delta: v1.7.0 added keybinding canonicalization; v1.8.0 removed Overlay stack field (propagated in v1.0.1). The `payload_to_modal()` conversion sketch and `§Diff Preview` sections cited by this BC are unchanged in v1.7.0 and v1.8.x.
- SE-16d monotonicity: v1.0.2 timestamp 2026-05-29T00:00:00Z > v1.0.1. PASS.

## §Trace v1.0.0

**Initial production** (2026-05-27T00:00:00Z):
- BC-2.06.024 created to resolve adversarial finding: S-027 ACs for Bash/Read/Generic
  tool display were mis-anchored to BC-2.06.017 (Permission Response Within Hook Timeout
  Budget), whose postconditions are timing/latency contracts, not rendering contracts.
- Decision rationale: option (b) — new BC rather than extending BC-2.06.010. BC-2.06.010's
  scope is precisely "Diff Preview via `similar 3`" for `ToolPayload::Edit`. Extending it to
  cover Bash/Read/Generic body rendering would break its H1 title authority and mix two
  distinct rendering concerns (diff computation vs. label display). The new BC provides a
  clean, independently testable rendering contract for the three non-Edit variants.
- Reads: SS-tui.md v1.6.0 §Permission Overlay §IPC Payload to PromptModal Conversion
  (canonical ToolPayload variant selection with Bash/Read/Generic/Edit arms);
  SS-tui.md v1.6.0 §Permission Overlay §Diff Preview (Edit is the separate path);
  BC-2.06.010 (scope and postconditions confirmed — Edit only).
- Capability anchored to CAP-006 per ARCH-INDEX §Capability Traceability SS-06.
- DI-007 cited: rendering is pure read-only; no file writes.
- EC-004 covers MCP-namespaced tool names (`mcp__*`) which are the most common Generic
  variant in practice given monocle's purpose.
- EC-007 covers `serde_json::to_string` failure as a safe-fallback edge case.
- Invariant 3 explicitly prohibits `similar` import in non-Edit rendering functions, enforcing
  the purity boundary from BC-2.06.001 and BC-2.06.010.
- SE-16d PASS: 2026-05-27T00:00:00Z > chain high-water 2026-05-26T18:00:00Z (prior SS-06 BCs). ARITHMETICALLY TRUE.
