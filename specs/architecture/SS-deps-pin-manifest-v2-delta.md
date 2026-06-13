---
document_type: architecture-section-delta
level: L3
section: "deps-pin-manifest-v2-delta"
subsystem: cross-cutting
version: "1.0.1"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - specs/architecture/SS-deps-pin-manifest.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "10c134b"
traces_to: architecture/ARCH-INDEX.md
project: monocle
---

# SS-deps-pin-manifest v2 Delta

## Purpose

This document specifies additions to `SS-deps-pin-manifest.md` (the canonical dependency manifest)
required by the v1A control-center pivot. All existing Phase-1 pins and policies in
SS-deps-pin-manifest.md (v1.2.1) remain in effect. The implementer applies both documents.

**When SS-deps-pin-manifest.md is updated to incorporate these changes, this delta document's
version becomes SUPERSEDED.**

---

## New Crates — Phase 1 Pin Manifest Additions (v1A)

All three crates were selected in embedded-pty-evaluation.md §7.1, ratified at D-237, and
formalized by ADR-0011. All versions verified against crates.io on 2026-06-03.

| Crate | Version Pin | Crate Location | Role | License | RUSTSEC | MSRV | Pin Policy |
|-------|-------------|---------------|------|---------|---------|------|------------|
| `portable-pty` | `"0.9"` (caret) | `monocle-session-host` | PTY pair creation, child spawn, master read/write | MIT | none (2026-06-03) | <1.88 | Caret — not on untrusted-input path; not security-sensitive boundary |
| `vt100` | `"0.16"` (caret) | `monocle-session-host`, `monocle-tui` | ANSI/VT100 parse → in-memory screen state | MIT | none (2026-06-03) | <1.88 | Caret — same rationale |
| `tui-term` | `"=0.3.4"` (exact) | `monocle-tui` | ratatui widget rendering `vt100::Screen` | MIT | none (2026-06-03) | 1.86 | Exact — WIP-label maturity caveat (ADR-0011 §Q-7) |

### Compatibility verification (mandatory at Cargo-init)

Before any v1A story implementation begins, the devops-engineer MUST run:

```
cargo tree -d --workspace
```

and confirm:
- Exactly ONE resolved version of `ratatui-core` (expected: `0.1.x` matching `^0.1.0`).
- Exactly ONE resolved version of `ratatui-widgets` (expected: `0.3.x` matching `^0.3.0`).
- Exactly ONE resolved version of `vt100` (expected: `0.16.x`).

Zero duplicates is a hard gate. If duplicates appear, raise to architect before proceeding.
Root cause is typically a transitive ratatui-core version mismatch; see embedded-pty-evaluation.md
§2.1 for resolution guidance.

### Cargo.toml placement

```toml
# crates/monocle-session-host/Cargo.toml
[dependencies]
portable-pty  = "0.9"
vt100         = "0.16"
tokio         = { workspace = true }
serde         = { workspace = true }
serde_json    = { workspace = true }
nix           = { workspace = true }      # setsid, kill
thiserror     = { workspace = true }
tracing       = { workspace = true }
chrono        = { workspace = true }
uuid          = { workspace = true }

# crates/monocle-tui/Cargo.toml
[dependencies]
# (existing deps unchanged)
tui-term      = "=0.3.4"                  # exact-pinned per ADR-0011; no unstable feature
vt100         = "0.16"
```

**Explicit constraint:** Do NOT declare `tui-term` with the `unstable` feature:
```toml
# CORRECT:
tui-term = "=0.3.4"
# WRONG (do not use):
tui-term = { version = "=0.3.4", features = ["unstable"] }
```
The `unstable` feature activates a tui-term spawn helper that depends on `portable-pty` from
the TUI crate. monocle spawns via `portable-pty` in `monocle-session-host`; the TUI does not
own PTY spawn.

---

## New Binary Crate — monocle-session-host

`crates/monocle-session-host/` is a new binary crate in the workspace.

**Workspace manifest change (Cargo.toml root):**

```toml
[workspace]
members = [
    # ... existing 9 crates ...
    "crates/monocle-session-host",   # NEW v1A
]
```

This expands the workspace from 9 crates to 10 crates.

The `monocle-session-host` binary is packaged alongside `monocle` in the release bundle.
The release CI job MUST verify that BOTH binaries are present in the release archive.

---

## MSRV Impact

**No change.** Phase-1 MSRV remains Rust 1.88.

- `portable-pty 0.9.0`: no declared MSRV; compiles cleanly on 1.88 (verified in
  embedded-pty-evaluation.md §2).
- `vt100 0.16.2`: no declared MSRV; compiles on 1.88 (embedded-pty-evaluation.md §2).
- `tui-term 0.3.4`: declared MSRV 1.86 < monocle's 1.88 floor. No floor raise.

---

## Security Audit (cargo audit additions)

The three new crates are automatically covered by the existing CI `cargo audit --deny warnings`
gate. No additional configuration required. The weekly RUSTSEC advisory scan already covers
all workspace dependencies.

**Current RUSTSEC status (2026-06-03):**
- `portable-pty 0.9.0`: no advisory.
- `vt100 0.16.2`: no advisory.
- `tui-term 0.3.4`: no advisory.

---

## Version-pin-registry.yaml entries (state-manager action required)

<!-- version-pin-historical: HISTORICAL — authoring-time registry action, completed at D-239/D-240;
     current pins live in version-pin-registry.yaml. The block below is preserved as an audit trail
     of the initial registration instructions; do NOT act on the version literals listed here. -->

State-manager MUST add the following entries to `version-pin-registry.yaml` in the same
commit that creates the `monocle-session-host` crate and bumps SS-deps-pin-manifest.md:

```yaml
SS-session-manager:
  path: specs/architecture/SS-session-manager.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"

SS-embedded-pty:
  path: specs/architecture/SS-embedded-pty.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"

SS-engine-module-v2-delta:
  path: specs/architecture/SS-engine-module-v2-delta.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"

SS-deps-pin-manifest-v2-delta:
  path: specs/architecture/SS-deps-pin-manifest-v2-delta.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"

ADR-0009:
  path: specs/architecture/adr/ADR-0009-native-session-host-process-model.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"

ADR-0010:
  path: specs/architecture/adr/ADR-0010-pty-bytes-over-shared-uds-ipc.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"

ADR-0011:
  path: specs/architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md
  current_version: "1.0.0"
  last_bump_commit: "[SHA — state-manager fills]"
  last_bump_date: "2026-06-03"
```

<!-- version-pin-historical: S27-001 fix — this authoring-time instruction was completed at D-239/D-240; ARCH-INDEX was bumped to v1.0.27 at that time. This line is retained as a historical record only and must NOT be actioned again. -->

---

## §Trace v1.0.1

**S27-001 — stale authoring-time ARCH-INDEX bump instruction annotated as completed** (2026-06-13):

- **Finding (S27-001):** Line ~196 contained `"ARCH-INDEX must also be bumped in the same commit (from v1.0.26 → v1.0.27)."` — an authoring-time instruction for state-manager, sitting outside the `version-pin-historical` comment block. The current ARCH-INDEX is v1.0.28, so the instruction has been completed. Leaving it as live instruction text risks a future agent misreading it as an open action item.
- **Fix:** Line annotated with `<!-- version-pin-historical: ... completed at D-239/D-240 -->` to mark it as a historical record. No behavioral change; no version bump to ARCH-INDEX is triggered by this patch.
- Semver: patch (v1.0.0 → v1.0.1) — comment annotation only; no normative content change.

## §Trace v1.0.0

**Initial production** (2026-06-03T23:00:00Z):
- SS-deps-pin-manifest v2 delta authored for v1A architecture delta.
- Three PTY-stack crates added with pin policy justifications.
- monocle-session-host new binary crate documented.
- MSRV confirmed unchanged at 1.88.
- Version-pin-registry additions listed for state-manager.
- SE-16d PASS: 2026-06-03T23:00:00Z (new artifact).
