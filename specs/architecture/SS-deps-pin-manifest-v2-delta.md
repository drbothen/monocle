---
document_type: architecture-section-delta
level: L3
section: "deps-pin-manifest-v2-delta"
subsystem: cross-cutting
version: "1.0.2"
status: draft
producer: vsdd-factory:architect
phase: v1A-architecture-delta
timestamp: 2026-06-03T23:00:00Z
inputs:
  - specs/architecture/SS-deps-pin-manifest.md
  - specs/research/embedded-pty-evaluation.md
  - specs/architecture/adr/ADR-0011-pty-stack-native-portable-pty-vt100-tui-term.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "7d806f1"
traces_to: architecture/ARCH-INDEX.md
project: monocle
# Story inputs: pin policy for v1A stories — see §Story Pin Rule below.
---

# SS-deps-pin-manifest v2 Delta

## Purpose

This document specifies additions to `SS-deps-pin-manifest.md` (the canonical dependency manifest)
required by the v1A control-center pivot. All existing Phase-1 pins and policies in
`SS-deps-pin-manifest.md` remain in effect. The implementer applies BOTH documents together.
The base manifest's frontmatter `version:` field is the authoritative current version of that
document; the v1.2.1 reference at initial authoring time is an audit-trail marker only.

**When SS-deps-pin-manifest.md is updated to incorporate these changes, this delta document's
version becomes SUPERSEDED.**

## Story Pin Rule (v1A SS-08/SS-09 stories)

This rule governs the `inputs:` frontmatter of all v1A wave 8-9 stories (S-033..S-048+).

**Rule: BOTH manifests — base + v2-delta — in every v1A story `inputs:` block.**

Rationale: Every v1A story operates inside a workspace that instantiates ALL crates. An
implementer touching any v1A crate (including `monocle-runtime`, `monocle-ipc`, `monocle-tui`,
`monocle-session-host`) must be aware of the full pin set. The cost of an extra input reference
is zero; the cost of an implementer missing an exact pin (e.g. `tui-term = "=0.3.4"`) is a
workspace Cargo resolution error or a wrong crate version in the build.

Stories that touch only workspace-wide deps (tokio, serde, tracing, etc.) still need the
v2-delta pin because the workspace Cargo.toml already imports the v1A crates as members, and
the workspace resolver applies the full dep graph at resolution time. A story that declares
only the base manifest is missing normative coverage of the PTY-stack pins.

Per-story exception: if a story is explicitly scoped to a crate that predates v1A and has zero
transitive dependency on `monocle-session-host` or `monocle-tui::embedded_terminal`, a waiver
may be noted in the story file with explicit justification. No v1A SS-08/SS-09 stories meet
this exception.

**Canonical pin pair for all v1A stories:**
```yaml
inputs:
  - {path: .factory/specs/architecture/SS-deps-pin-manifest.md, version: "CURRENT"}
  - {path: .factory/specs/architecture/SS-deps-pin-manifest-v2-delta.md, version: "CURRENT"}
```
Replace `CURRENT` with the versions in `version-pin-registry.yaml` at story-write time.

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

## §Trace v1.0.2

**Story Pin Rule added** (2026-06-16):

- **Finding:** v1A wave 8-9 stories (S-033..S-048) were inconsistently pinning either the base
  manifest only, the v2-delta only, or neither — caused by the absence of an explicit Story Pin
  Rule in this document. Story-writers had no normative policy to cite, so each defaulted to
  whichever manifest seemed more immediately relevant to the story's primary crates.
- **Fix:** Added §Story Pin Rule section establishing BOTH manifests as mandatory inputs for all
  v1A stories, with rationale (full workspace dep graph at resolution time). Added frontmatter
  comment pointer `# Story inputs: pin policy for v1A stories — see §Story Pin Rule below.`
- **Scope:** No crate version changes. No normative pin policy changes. MSRV unchanged.
- **State-manager action:** update `version-pin-registry.yaml` `SS-deps-pin-manifest-v2-delta`
  entry to `current_version: "1.0.2"` in the same factory-artifacts commit.
  Also reconcile story inputs for S-033..S-048 per architect ruling (see ruling report).
- Semver: patch (v1.0.1 → v1.0.2) — normative policy addition (Story Pin Rule); no crate change.

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
