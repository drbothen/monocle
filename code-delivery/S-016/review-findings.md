---
document_type: review-findings
story_id: S-016
pr_number: 19
producer: vsdd-factory:pr-manager
timestamp: 2026-05-27T18:30:00Z
verdict: MERGED
convergence_cycles: 1
---

# Review Findings — S-016: Daemon Binary Crate Init + CLI Subcommands

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining | Verdict |
|-------|----------|----------|-------|-----------|---------|
| 1 | 5 | 3 | 5 | 0 | APPROVE |
| **Total** | **5** | **3** | **5** | **0** | **MERGED** |

**Converged in 1 cycle.** No blocking findings at merge.

---

## Cycle 1 Findings

### Important (Blocking)

**I1: Wrong `ProjectDirs::from` organization argument**
- Location: `crates/monocle-runtime/src/lifecycle.rs:98`
- Category: spec-fidelity
- Evidence: `from("", "monocle", "monocle")` should be `from("", "", "monocle")` per BC-2.04.006 PC-4
- Resolution: Fixed in commit `13e1667` — changed to `ProjectDirs::from("", "", "monocle")`
- Status: FIXED

**I2: Direct `std::process::exit()` bypasses `exit_with()` convention**
- Location: `crates/monocle-runtime/src/main.rs:63, 69, 81`
- Category: SS-conventions-anti-patterns.md violation
- Evidence: 3 direct `process::exit(1)` calls bypass `exit_with` invariant (lifecycle.rs:24-26)
- Resolution: Fixed in commit `13e1667` — replaced all 3 with `exit_with(DaemonExit::StartupFailure)`
- Status: FIXED

**I3: Level 2 INFO log missing from `resolve_runtime_dir()`**
- Location: `crates/monocle-runtime/src/lifecycle.rs:102-103`
- Category: spec-fidelity (BC-2.04.006 PC-6, INV-6)
- Evidence: No log when `proj.runtime_dir()` returns `Some` (Linux XDG path)
- Resolution: Fixed in commit `13e1667` — added `tracing::info!("runtime_dir from ProjectDirs::runtime_dir()")`
- Status: FIXED

### Suggestions (Non-blocking)

**S1: Level 3 log format doesn't match spec literal**
- Location: `crates/monocle-runtime/src/lifecycle.rs:105-108`
- Category: spec-compliance (BC-2.04.006 PC-9)
- Resolution: Fixed in commit `13e1667` — platform now interpolated inline in message body
- Status: FIXED

**S2: Stale doc comments reference `ProjectDirs::new()`**
- Location: `crates/monocle-runtime/src/lifecycle.rs:9, 12, 39, 45`
- Category: documentation accuracy
- Resolution: Fixed in commit `13e1667` — all references updated to `ProjectDirs::from("", "", "monocle")`
- Status: FIXED

---

## Merge Information

- **PR:** #19
- **Merge commit:** `87ac91fc7d416a14d02817fa9de72df26a3895ce`
- **Merged to:** `develop`
- **Final test count:** 37/37 PASS
- **Security review:** CLEAN
- **CI:** Pre-existing protoc infrastructure failure (not S-016 regression; same failure on develop since Wave 1)
