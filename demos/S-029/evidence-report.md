# Demo Evidence Report — S-029

**Story:** S-029: Killer Scenario Integration Test — Permission Prompt E2E Round-Trip
**BC:** BC-2.06.022 (killer scenario: permission prompt E2E round-trip)
**Date:** 2026-06-02
**Recorder:** vsdd-factory:demo-recorder
**Format:** VHS terminal recording (GIF + WebM)

---

## Coverage Map

| AC | Description | Test Scenario | Recording |
|----|-------------|---------------|-----------|
| AC-001 | Test setup: MockDaemon + UDS + TestInputDriver | (infrastructure — visible in all runs) | AC-002-007-killer-scenario.gif |
| AC-002 | `killer_scenario_accept` — accept decision E2E (8-step happy path) | `test_BC_2_06_022_killer_scenario_accept` | AC-002-007-killer-scenario.gif |
| AC-003 | `killer_scenario_multi_prompt` — two-prompt FIFO stacking | `test_BC_2_06_022_killer_scenario_multi_prompt` | AC-002-007-killer-scenario.gif |
| AC-004 | `killer_scenario_disconnect` — disconnect clears overlay mid-prompt | `test_BC_2_06_022_killer_scenario_disconnect` | AC-002-007-killer-scenario.gif |
| AC-005 | `killer_scenario_esc_no_reject` — Esc is identity, no PermissionDecision sent | `test_BC_2_06_022_killer_scenario_esc_no_reject` | AC-002-007-killer-scenario.gif |
| AC-006 | `killer_scenario_edit_diff` — Edit diff rendered in ratatui TestBackend | `test_BC_2_06_022_killer_scenario_edit_diff` | AC-002-007-killer-scenario.gif |
| AC-007 | Test isolation via `tempfile::TempDir` — parallel-safe | `test_BC_2_06_022_killer_scenario_isolation_parallel_safe` | AC-002-007-killer-scenario.gif |
| BC-KS | `killer_scenario_accept_always` — AcceptAlways variant (BC-2.06.022 KS-001/KS-002) | `test_BC_2_06_022_killer_scenario_accept_always` | AC-002-007-killer-scenario.gif |

**Total:** 7 test scenarios shown green in recording.

---

## Recordings

| File | Format | Size | Purpose |
|------|--------|------|---------|
| `AC-002-007-killer-scenario.gif` | GIF | 130 KB | PR embed / inline viewing |
| `AC-002-007-killer-scenario.webm` | WebM | 125 KB | Archival / high-fidelity playback |
| `AC-002-007-killer-scenario.tape` | VHS tape script | 1.4 KB | Reproducible re-recording source |

---

## Command Demonstrated

```
cargo test -p monocle-tui --test killer_scenario -- --test-threads=4 2>&1
```

**Expected terminal output (confirmed in recording):**

```
running 7 tests
test test_BC_2_06_022_killer_scenario_disconnect ... ok
test test_BC_2_06_022_killer_scenario_accept_always ... ok
test test_BC_2_06_022_killer_scenario_accept ... ok
test test_BC_2_06_022_killer_scenario_multi_prompt ... ok
test test_BC_2_06_022_killer_scenario_edit_diff ... ok
test test_BC_2_06_022_killer_scenario_isolation_parallel_safe ... ok
test test_BC_2_06_022_killer_scenario_esc_no_reject ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.51s
```

---

## Demo Format Rationale

S-029 is a test-only story — its deliverable is `monocle-tui/tests/killer_scenario.rs`
(7 integration scenarios). There is no new interactive UI surface to record. The
appropriate demo evidence is the test execution proving the killer scenario passes
end-to-end. The recording shows the `cargo test` invocation and the final
`test result: ok. 7 passed; 0 failed` line, which constitutes sufficient regression
evidence for BC-2.06.022.

---

## Validated Holdout

Recording covers HS-EXP-008 validation path: the killer scenario E2E round-trip
is the primary evidence for holdout HS-EXP-008 (permission prompt 6-keystroke
resolution). The holdout evaluator will consume this evidence alongside the test
suite run on develop after wave-7 gate.
