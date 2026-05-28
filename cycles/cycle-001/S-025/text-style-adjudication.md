---
document_type: product-owner-decision
level: ops
cycle: cycle-001
story: S-025
issue: F-S025-ADV11-HIGH-001
version: "1.0"
status: binding
producer: vsdd-factory:product-owner
timestamp: 2026-05-28T14:00:00Z
phase: 3
traces_to: "Resolves F-S025-ADV11-HIGH-001 BC-2.06.016 PC-4 spec-impl drift."
---

# Decision — Daemon Status Indicator Text Style

## Adjudication

**Option B chosen: Production wins. BC-2.06.016 updated to match the bracketed style.**

Rationale:

The S-025 implementation uses a consistent bracketed-tag style across all three daemon status
indicators: `[disconnected] reconnecting...`, `[daemon: offline]`, and `[dropped: N] monocle`.
This consistency is the governing constraint. Updating the disconnect indicator to a prose style
(`"Daemon disconnected — reconnecting..."`) would require updating ALL three indicators to maintain
visual coherence — otherwise the UX shows a visible style break between indicators that appear in
the same status bar context. The bracketed style is:

1. Internally consistent across all three indicators (architecture-wide coherence).
2. Implicitly endorsed by 11 adversarial passes (Architect Passes 1 and 2 through ADV-11) that
   flagged the style drift in BC-2.06.016 PC-4 but did not reverse the implementation style.
3. Compatible with the tracing output pattern already used in the codebase.

The prose style in BC-2.06.016 PC-4 originated from the spec-writing phase before the bracketed
style was established in production. The BC is the artifact that must be corrected.

Note: SS-tui.md line 668 also cites the prose style
(`"Daemon disconnected — reconnecting..."`). That reference is under architect scope and must
be updated in a follow-up pass. Flagged below.

## Implementer Directive

**No production changes required.** app.rs line 304 is already correct:

```
"[disconnected] reconnecting..."
```

Keep this string exactly as-is. Do not change `[daemon: offline]` (line 620/636) or
`[dropped: N] monocle` (line 888) — they were correct before this adjudication and remain so.

The three canonical daemon-status strings for production are:

| Context | Canonical String |
|---------|-----------------|
| IPC transport disconnected, reconnect in progress | `"[disconnected] reconnecting..."` |
| Reconnect window exhausted, offline mode active | `"[daemon: offline]"` |
| Drop counter non-zero | `"[dropped: N] monocle"` (N = actual count) |

## Test-Writer Directive

Any test asserting the status bar text after a `TransportEvent::Disconnected` event must use
the exact string:

```
"[disconnected] reconnecting..."
```

Example assertion form:
```rust
assert_eq!(app.status_message, Some("[disconnected] reconnecting...".to_string()));
```

## BC-2.06.016 Update Required?

**Yes.** Version bump v1.0.7 → v1.0.8.

Lines to change (all occurrences of the prose form):

- **Description paragraph (line 35):**
  `"Daemon disconnected — reconnecting..."` → `"[disconnected] reconnecting..."`

- **Postcondition 4 (line 62):**
  `"Daemon disconnected — reconnecting..."` → `"[disconnected] reconnecting..."`

- **VP table (line 115):**
  `Status bar renders "Daemon disconnected — reconnecting..." on disconnect`
  → `Status bar renders "[disconnected] reconnecting..." on disconnect`

- **Canonical test vectors (lines 102–106) — all "reconnecting..." cells in the
  Expected Post-State column:**
  `status bar "reconnecting..."` → `status bar "[disconnected] reconnecting..."`

  Specifically the four test vector rows (rows 1–4 in the table) that contain
  `status bar "reconnecting..."` in their Expected Post-State column.

No other BC files reference this specific string literal and require changes.

## Cross-References

The following artifacts cite the prose form and must be updated in the same or immediately
following burst:

| Artifact | Location | Scope | Owner |
|----------|----------|-------|-------|
| `BC-2.06.016.md` | PC-4, Description, VP table, test vectors | Product Owner (this decision) | Updated below |
| `architecture/SS-tui.md` | Line 668 | Architect scope — **flag for architect** | Not updated here |

SS-tui.md line 668 (`"Daemon disconnected — reconnecting..."`) is the only other canonical
reference. Architect must update SS-tui.md to `"[disconnected] reconnecting..."` in a
follow-up spec pass. This is LOW severity (architecture doc trails implementation; the
production string is already correct and BC-2.06.016 will be corrected in this burst).
