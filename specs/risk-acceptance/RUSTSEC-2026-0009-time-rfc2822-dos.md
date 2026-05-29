---
document_type: risk-acceptance
id: RA-001
advisory: RUSTSEC-2026-0009
cve: CVE-2026-25727
crate: time
vulnerable_range: ">=0.3.6, <0.3.47"
locked_version: "0.3.37"
patched_version: ">=0.3.47"
patched_msrv: "1.88.0"
project_msrv: "1.88"
status: resolved
severity: MEDIUM
filed_by: devops-engineer
date: 2026-05-28
reviewed_by: architect
phase: phase-3-wave-6
story: S-025
expires_at: phase-3-wave-7-start
resolution_path: path-b-selected-wave-6-msrv-bumped-to-1.88
---

# Risk Acceptance: RUSTSEC-2026-0009 — time crate RFC 2822 DoS

## Advisory Summary

**RUSTSEC-2026-0009 / CVE-2026-25727**: The `time` crate versions 0.3.6..0.3.46 allow
stack exhaustion via crafted RFC 2822 input. Affected functions: `time::Date::parse`,
`time::OffsetDateTime::parse`, `time::PrimitiveDateTime::parse`, `time::Time::parse`,
`time::UtcDateTime::parse`, `time::UtcOffset::parse`, `time::parsing::Parsed::parse_item`.
Patched in 0.3.47 (released 2026-02-05). CVSS 6.8 (MEDIUM).

## Why the vulnerability is not exploitable in monocle

The `time` crate appears in `Cargo.lock` at `0.3.37` as a transitive dependency via:

```
monocle-tui → ratatui 0.30 → ratatui-widgets 0.3.0 → time 0.3.37 (optional, calendar feature)
```

**The `calendar` feature is not activated in the monocle dependency graph.** Specifically:

1. The workspace `Cargo.toml` declares `ratatui` with `default-features = false` and
   explicitly lists features: `["crossterm", "underline-color", "macros", "layout-cache"]`.
   The `all-widgets` feature (which activates `widget-calendar`) is NOT included.

2. No monocle crate (`monocle-tui`, `monocle-core`, `monocle-runtime`, or any other workspace
   member) imports or calls any `time` crate API. `grep -r "time::" crates/` returns zero
   hits for the `time` crate's types.

3. The `time` crate's vulnerable functions (`Date::parse`, `OffsetDateTime::parse`, etc.) are
   only reachable if the `calendar` feature is activated AND user-provided RFC 2822 strings are
   passed to those parse functions. Neither condition holds in monocle.

**The `time` entry in `Cargo.lock` is a residual artifact of `ratatui-widgets v0.3.0`
declaring `time` as `optional = true` with `uses_default_features = true`.** Cargo resolves
optional dependencies into the lockfile even when no crate in the workspace activates the
feature that gates them. The compiled monocle binary does not contain any `time` crate code.

## Why the patched version (0.3.47) cannot be applied immediately

`time 0.3.47` bumped MSRV to 1.88.0. The project MSRV is 1.86 (Phase 1 floor per
`SS-deps-pin-manifest.md §MSRV Policy`). Applying 0.3.47 would:

1. Break the CI `Lint toolchain pin (AC-004)` check, which asserts `channel = "1.86"`.
2. Require a workspace-wide MSRV bump — an architectural decision requiring architect approval
   and a CHANGELOG breaking-change entry (per SS-deps-pin-manifest §MSRV Policy).

Phase 3 MSRV is already planned to bump to 1.92 at the Phase 3 boundary (wasmtime 44 requirement).
Bumping to 1.88 ahead of that boundary requires an unplanned intermediate MSRV commit with no
other benefit, since `time` is not compiled into the binary in any case.

## Risk assessment

| Factor | Assessment |
|--------|-----------|
| Exploitability in monocle | Zero — `time` parse functions not called, calendar feature not activated |
| Attack surface | None — monocle has no code path that passes user input to `time` RFC 2822 parsers |
| Presence in compiled binary | No — `time` is lockfile-resident but not compiled into the monocle binary |
| CVSS severity | 6.8 MEDIUM |
| Risk to monocle users | Negligible — not present in binary |

**Accepted risk level: LOW.** The vulnerability is present in the lockfile but not in any
compiled artifact or reachable code path. The residual risk is theoretical: if a future
story activates the `calendar` feature without updating the toolchain first, the advisory
would become relevant. The expiry condition below addresses this.

## Expiry condition and resolution path

This acceptance expires at the start of Phase 3 Wave 7 (whichever comes first):

- **Resolution path A (preferred):** When `rust-toolchain.toml` is bumped toward Phase 3
  MSRV 1.92 (wasmtime 44 requirement), also update `time` to `>=0.3.47` and remove this
  risk-acceptance file + the `deny.toml` ignore entry. The MSRV bump to 1.92 subsumes 1.88.

- **Resolution path B (if calendar feature is added before Phase 3 MSRV bump):** The story
  implementing the calendar widget MUST first bump `rust-toolchain.toml` to `>=1.88` and
  update `time` to `>=0.3.47`. This risk-acceptance file is INVALID for any build where the
  `calendar` feature is activated.

- **Resolution path C (emergency):** If a security team assesses monocle's actual risk
  exposure as higher than LOW (e.g., a monocle deployment somehow exposes `time` parse calls),
  escalate to architect for immediate MSRV bump to 1.88.

## deny.toml change required

The following ignore entry must be present in `deny.toml` while this acceptance is active:

```toml
[advisories]
ignore = [
    { id = "RUSTSEC-2026-0009", reason = "time calendar feature not activated; not compiled into binary; see .factory/specs/risk-acceptance/RUSTSEC-2026-0009-time-rfc2822-dos.md; expires at Phase 3 Wave 7 or MSRV bump to 1.88+" }
]
```

## Resolution — Path B Applied (2026-05-28)

**Status: RESOLVED.** Human selected Path B during Wave 6 (S-025 pre-work). The project MSRV
has been bumped from 1.86 to 1.88 in the spec layer (SS-deps-pin-manifest.md §Trace v1.2.0).

Implementation actions owned by devops-engineer:
1. `rust-toolchain.toml`: update `channel = "1.86"` → `channel = "1.88"`.
2. `Cargo.toml` workspace `rust-version`: update `"1.86"` → `"1.88"`.
3. CI AC-004 check: update assertion to `channel = "1.88"`.
4. Once `cargo update` resolves `time ≥ 0.3.47` under the 1.88 toolchain,
   remove the `deny.toml` ignore entry for RUSTSEC-2026-0009.
5. Remove this file from `.factory/specs/risk-acceptance/` (or retain as historical record
   — no functional impact either way once the advisory is no longer present in `cargo deny` output).

This risk-acceptance file is retained as a historical record of the advisory lifecycle:
accepted by devops-engineer (RA-001, 2026-05-28), resolved same day via Path B human selection.
The deny.toml ignore entry above is the last operational artifact that must be removed by
devops-engineer after the toolchain and Cargo.lock updates are applied.
