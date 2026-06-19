## Summary

Chore: update a stale doc-comment citation in `crates/monocle-runtime/src/session_manager/mod.rs`.

**Change:** single-line comment update — `// BC-2.08.007 v1.5.5` → `// BC-2.08.007 v1.5.6`

- Matches the version now recorded in `version-pin-registry.yaml` after the S-038 arch-source cascade (F-S038-PASS1-001 finding resolution).
- Preserves the human-authored commit 4638006 (Joshua Magady) via the standard PR/CI flow per project policy.
- **No behavioral change. No code logic change. No security surface.** Comment only.

## Why

`POL-11` cross-checks comment-embedded version pins against `version-pin-registry.yaml`. After BC-2.08.007 was advanced to v1.5.6 in the registry during the S-038 cascade, the comment at line 3394 of `session_manager/mod.rs` still cited v1.5.5, causing a POL-11 mismatch. This chore resolves it.

## Traceability

- BC: `BC-2.08.007 v1.5.6`
- Finding: `F-S038-PASS1-001` (arch-source cascade, version-pin registry update)
- Registry file: `.factory/specs/architecture/version-pin-registry.yaml`

## Pre-Merge Checklist

- [x] Comment-only change — no behavioral delta
- [x] POL-11 will pass (v1.5.6 matches registry)
- [x] No new tech debt introduced
- [x] Human-authored commit preserved via proper PR flow
- [x] CI required: all 11 checks must pass before merge
