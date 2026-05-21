---
document_type: adversarial-review
story: S-001
round: 2
scope: "PR #2 (8911431, 13 commits)"
producer: vsdd-factory:adversary
timestamp: 2026-05-20T23:00:00Z
verdict: FAIL
findings_summary:
  critical: 1
  high: 2
  medium: 4
  process_gaps: 1
---

# Adversarial Review — PR #2 (S-001 post-merge fix), Round 2

**Verdict: FAIL** — 1 CRIT time-bomb + 2 HIGH production-grade violations + 4 MED + 1 process-gap. None can ship per Production-Grade Default.

## Findings

### CRIT-1 — Semgrep Step 3 cannot read spec file in CI; passes today only because of empty-production short-circuit
File: `.worktrees/S-001-fix/.github/workflows/ci.yml:149-152`
Evidence: invokes `python3 scripts/check_audit_table.py --spec-file .factory/specs/architecture/SS-engine-module.md`. `.gitignore:24` ignores `.factory/` (orphan-branch worktree); `actions/checkout` on develop/PR refs doesn't materialize `.factory/`. Script (`scripts/check_audit_table.py:64-66`) exits 1 with FileNotFoundError if spec file missing. CI green today because `check_audit_table.py:203-205` short-circuits exit(0) when production_struct_names is empty — Phase 1 crates have ZERO #[non_exhaustive] structs in production. Step 3 never reaches spec-file read.
Time-bomb trigger: First real #[non_exhaustive] pub struct in S-002+ (HookResponse, EngineMetadata per audit table SS-engine-module.md:1109-1116) → FileNotFoundError every CI run.
Routing: devops-engineer. Fix options: (a) add factory-artifacts checkout step + path materialization, (b) vendor audit-table to worktree at scripts/audit-table.md, (c) ship audit-table verbatim with --spec-file pointing to it.
Confidence: HIGH.

### HIGH-1 — println! in production binary main.rs violates canonical convention
File: `.worktrees/S-001-fix/crates/monocle-runtime/src/main.rs:7` has `println!("monocle-runtime stub");`.
SS-conventions-anti-patterns.md:503: "No println! in production code paths (use tracing with structured fields)". CLAUDE.md §Conventions reiterates.
Production-Grade Default Rule 1 forbids "stub" rationalization. Either use tracing::info!() with structured fields + tracing_subscriber init, OR std::process::exit(0) with no output, OR amend the convention.
clippy.toml does NOT include std::println in disallowed_methods → rule has no CI enforcement (process-gap).
Routing: implementer (fix violation) + architect (close process-gap via clippy.toml addition).
Confidence: HIGH.

### HIGH-2 — main.rs missing `#![forbid(unsafe_code)]` + `#![deny(missing_docs)]`
File: `.worktrees/S-001-fix/crates/monocle-runtime/src/main.rs:1-9`
Every other crate root has both — monocle-core/src/lib.rs:6-7, monocle-proto/src/lib.rs:9-10, monocle-runtime/src/lib.rs:7-8. main.rs lacks both. Binary target compiled separately from lib; lib attrs do not propagate. A contributor adding unsafe to main.rs would compile without error.
Routing: implementer. (Partial-fix-regression-discipline S-7.01 sibling sweep gap.)
Confidence: HIGH.

### MED-1 — Stale canonical version refs (v1.1.18 and v1.30.0) across multiple files
Sites: Cargo.toml:59, .github/dependabot.yml:3+15+48, .github/workflows/audit.yml:45, crates/monocle-runtime/Cargo.toml:39 (all v1.1.18 → v1.1.19); deny.toml:1 (v1.30.0 → v1.30.1). S-7.01 partial-fix-regression gap. Blast radius 6 files.
Routing: devops-engineer.
Confidence: HIGH.

### MED-2 — workspace_structure.rs:187 stale comment "bytes 1.10 pin"
Assertion at L218 checks for "bytes = \"1.11\"" correctly; header comment was missed during d9fb512 bytes pin bump sweep.
Routing: devops-engineer or test-writer.
Confidence: HIGH.

### MED-3 — deny.toml [graph] targets = [] "for now" deferral
File: deny.toml:3-5 comment "Phase 1 targets — adjust at workspace init". CI matrix at ci.yml:163-170 pins targets to 3 known triples. deny.toml currently scans all platforms (empty-list default).
Production-Grade Default Rule 6 violation: "TODO for architect" / "adjust at workspace init" for a mechanical question. The 3 CI targets ARE the answer.
Fix: targets = [aarch64-apple-darwin, x86_64-unknown-linux-gnu, aarch64-unknown-linux-gnu].
Routing: devops-engineer.
Confidence: MEDIUM.

### MED-4 — Step 1 semgrep stderr suppression
File: .github/workflows/ci.yml:97 — `2>/dev/null` retained. CR-006 (Round 1) removed silent failure masking from Step 3 but Step 1 was not swept. S-7.01 sibling-sweep gap.
Routing: devops-engineer.
Confidence: MEDIUM.

### LOW-1 (deferred from Round 1)
Already accepted as deferred — informational only.

### Process-gap — clippy.toml omits println/eprintln
clippy.toml lists tokio::sync::mpsc::unbounded_channel, std::fs::write, tokio::fs::write but omits std::println, std::eprintln. SS-conventions §Convention Checklist L503 ban exists only in prose. Adding std::println + std::eprintln to disallowed_methods converts prose rule into hard CI lint.
Routing: architect (canonical clippy.toml extension is architect domain).

## Novelty: HIGH
None of these overlap with Round 1 findings. CRIT-1 new defect category. HIGH-1 + HIGH-2 missed by every prior pass.

## Confidence: HIGH for CRIT-1/HIGH-1/HIGH-2/MED-1/MED-2; MEDIUM for MED-3/MED-4.

## Mergeable per Production-Grade Default: NO. CRIT-1 deterministic future failure. HIGH-1 ships convention-violating println! with no clippy enforcement. HIGH-2 one-line sibling-sweep gap.
