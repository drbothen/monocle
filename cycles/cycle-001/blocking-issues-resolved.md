# Blocking Issues Resolved — Cycle 001

Resolved blocking issues archived from STATE.md durable_task_register.

## Resolved Items

### F-S026-ADV6-DEFER-001 — offline-break reconnect availability gap

- **id:** F-S026-ADV6-DEFER-001
- **status:** resolved
- **subject:** offline-break reconnect paths leave dead ipc_rx and hot-loop without re-attempting reconnect (availability gap)
- **resolution:** RESOLVED by PR #31 (merged develop @ 2a51a91). Wave-6-gate adversarial re-run elevated to CRITICAL (F-WAVE6-GATE-CRIT-001); fixed via extracted reconnect_from_offline() + 4 offline-break arms re-enter reconnect + offline_reconnect.rs mutation-verified integration test. Gate-3 re-run CLEAN. D-224.

### POINTS-TALLY-RECONCILE — STATE running-tally drift

- **id:** POINTS-TALLY-RECONCILE
- **status:** resolved
- **subject:** STATE running-tally points_complete drifted to 177 vs authoritative sprint-state sum 169; corrected D-225
- **resolution:** RESOLVED D-225 (2026-05-31): sprint-state summary.points_complete hand-incremented to 177 but authoritative per-story sum is 169 (28 done stories: Wave 1-6 complete). Also STATE prematurely declared 'Phase 3 COMPLETE' before Wave 7. Both corrected in factory-artifacts burst. GUARD: state-manager must re-sum sprint-state per-story points (never trust summary.points_complete) at every phase/wave transition.

### F-S025-ADV28-MED-001 — S-025 §Downstream Consumer Contract struct-shape

- **id:** F-S025-ADV28-MED-001
- **status:** closed
- **subject:** S-025 §Downstream Consumer Contract struct-shape (META 10th + structural-claim #3)
- **resolution:** CLOSED (D-207) via story-writer 344366d (Option B historical-anchor annotation at lines 225-231; tasks list line 144 clarified; S-025 v1.10→v1.11; STORY-INDEX v5.15→v5.16). System-level 3-way divergence (story 5 fields vs SS-tui.md 9 fields vs app.rs 7 fields) deferred to phase-5 as F-S025-ADV28-OBS-002.

### F-S025-ADV28-MED-002 — ADR-0008 §Canonical Source Registry off-by-2 line range

- **id:** F-S025-ADV28-MED-002
- **status:** closed
- **subject:** ADR-0008 §Canonical Source Registry off-by-2 line range (self-application defect)
- **resolution:** CLOSED (D-207) via architect 12170b4 (ADR-0008 v1.0.1: line-range 831-864 → 833-864; §Self-Application Policy explicit; SS-conventions v1.32.2→v1.32.3 same correction).
