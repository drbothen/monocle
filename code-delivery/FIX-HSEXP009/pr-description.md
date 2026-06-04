## fix(HS-EXP-009): emit runtime-dir-unresolvable hint to stderr on exit-70 fail-fast (start+stop)

### Summary

Closes Phase 4 holdout finding HS-EXP-009. `monocle daemon start` and `monocle daemon stop` correctly exited with code 70 when the runtime directory could not be resolved, but emitted **zero bytes to stderr**. A user seeing only an exit-70 with no diagnostic message had no recovery path.

**Fix:** Add a verbatim `eprintln!` to each fail-fast path, emitting the canonical hint before any tracing-subscriber or directory-dependent initialisation:

```
ERROR: cannot resolve runtime directory; set MONOCLE_RUNTIME_DIR to specify an explicit path
```

Both `cmd_daemon_start` and `cmd_daemon_stop` receive the fix symmetrically.

**Tests added:** Two new CLI integration tests (`cli_daemon_start.rs::test_hs_exp_009_start_runtime_dir_unresolvable_stderr_hint` and `cli_daemon_stop.rs::test_hs_exp_009_stop_runtime_dir_unresolvable_stderr_hint`) that drive the real binary with `HOME`/`XDG_*`/`MONOCLE_RUNTIME_DIR` stripped, asserting exit 70 **and** byte-exact stderr content.

---

### Architecture Changes

```mermaid
graph TD
    A[monocle binary: main.rs] --> B[cmd_daemon_start: resolve_runtime_dir]
    A --> C[cmd_daemon_stop: resolve_runtime_dir]
    B -->|Err| D["eprintln! hint → stderr\nreturn EXIT_RUNTIME_DIR_UNRESOLVABLE (70)"]
    C -->|Err| E["eprintln! hint → stderr\nreturn EXIT_RUNTIME_DIR_UNRESOLVABLE (70)"]
    D --> F[User sees: ERROR + MONOCLE_RUNTIME_DIR hint]
    E --> F
```

**Convention rationale (ADV-W4GATE-MED-002 precedent):** The `eprintln!` is placed before any tracing-subscriber initialisation. This is intentional and consistent with the pre-subscriber fail-fast pattern used in the tokio-runtime-init path and the panic-hook setup. Using `tracing::error!` here would be wrong — the subscriber may not be initialised at the point of runtime-dir resolution failure.

---

### Story Dependencies

```mermaid
graph LR
    BC2_04_004["BC-2.04.004 (daemon start exit codes)"] --> HS["HS-EXP-009 (holdout scenario)"]
    BC2_04_005["BC-2.04.005 (daemon stop exit codes)"] --> HS
    HS --> FIX["fix/hsexp009-runtime-dir-stderr-hint"]
    S026["S-026 (merged)"] --> FIX
    S027["S-027 (merged)"] --> FIX
```

This fix has no upstream PR dependencies — all referenced stories are merged to `develop`.

---

### Spec Traceability

```mermaid
flowchart LR
    BC_START["BC-2.04.004\nEC-2.04.004-06\nPC-8: stderr hint required"] --> HS_EXP["HS-EXP-009\nPhase 4 holdout:\ndaemon start AND stop\nmust emit hint + exit 70"]
    BC_STOP["BC-2.04.005\nEC-2.04.005-07\nPC-8: stderr hint required"] --> HS_EXP
    HS_EXP --> TEST_START["test_hs_exp_009_start_\nruntime_dir_unresolvable_stderr_hint"]
    HS_EXP --> TEST_STOP["test_hs_exp_009_stop_\nruntime_dir_unresolvable_stderr_hint"]
    TEST_START --> IMPL["main.rs\ncmd_daemon_start:\neprintln! + return 70"]
    TEST_STOP --> IMPL2["main.rs\ncmd_daemon_stop:\neprintln! + return 70"]
```

---

### Test Evidence

| Metric | Value |
|--------|-------|
| New tests added | 2 (cli_daemon_start, cli_daemon_stop) |
| Test scope | End-to-end CLI binary (assert_cmd) |
| Exit code assertion | exit 70 verified |
| Stderr assertion | byte-for-byte hint string verified via `predicate::str::contains` |
| CI pre-validation | `cargo test -p monocle` green on branch |
| Clippy | `cargo clippy --workspace --all-targets -- -D warnings` clean |
| Format | `cargo fmt --all` clean |
| POL-11 (version pins) | PASS (`python3 scripts/check_version_pins.py`) |
| POL-12 (structural claims) | PASS (`python3 scripts/check_structural_claims.py`) |

---

### Holdout Evaluation

HS-EXP-009 is the **holdout scenario this fix resolves**. The scenario specification requires:

1. `monocle daemon start` exits 70 when runtime dir unresolvable — **was already passing**
2. `monocle daemon start` emits the verbatim hint to stderr — **was FAILING (no output)**
3. `monocle daemon stop` exits 70 when runtime dir unresolvable — **was already passing**
4. `monocle daemon stop` emits the verbatim hint to stderr — **was FAILING (no output)**

Post-fix: all four assertions pass.

---

### Adversarial Review

N/A — evaluated at Phase 5. This is a targeted fix-PR for a Phase 4 holdout finding. The fix is mechanically simple (two `eprintln!` calls, two test assertions) with no architectural surface expansion.

---

### Security Review

**Verdict: CLEAN — no security findings.**

| Check | Result |
|-------|--------|
| CWE-134 format string injection | NONE — static literal, zero format parameters |
| Information disclosure (OWASP A05) | NONE — hint contains only static env var name, no paths or user data |
| Pre-subscriber stderr emission | INTENDED — correct fail-fast pattern (ADV-W4GATE-MED-002 precedent) |
| OWASP Top 10 applicability | N/A — CLI error path, no network surface, no auth, no data processing |
| Exit path control flow | UNCHANGED — eprintln! is additive before existing return |

---

### Risk Assessment

| Dimension | Assessment |
|-----------|------------|
| Blast radius | Minimal — two `eprintln!` calls on error paths only; no hot path changes |
| Performance impact | None — error path only, binary not started |
| Regression risk | Low — new tests assert both exit code and stderr; pre-existing exit-70 behavior preserved |
| Data sensitivity | None — the hint string contains no user data, paths, or secrets |
| Injection risk | None — static string literal, no format parameters |

---

### AI Pipeline Metadata

| Field | Value |
|-------|-------|
| Pipeline mode | greenfield-with-reference-ingest |
| Fix type | Phase 4 holdout finding resolution (fix-pr-delivery) |
| Branch | fix/hsexp009-runtime-dir-stderr-hint |
| Base | develop (90ae584) |
| Files changed | 3 (main.rs, cli_daemon_start.rs, cli_daemon_stop.rs) |
| Lines changed | +74 |

---

### Pre-Merge Checklist

- [x] PR description matches actual diff
- [x] Traceability chain complete: BC-2.04.004/005 → HS-EXP-009 → tests → implementation
- [x] `cargo test -p monocle` green on branch
- [x] `cargo clippy --workspace --all-targets -- -D warnings` clean
- [x] `cargo fmt --all` clean
- [x] POL-11 version-pin check PASS
- [x] POL-12 structural-claims check PASS
- [ ] CI checks green (pending merge)
- [ ] Security review complete
- [ ] PR reviewer APPROVE
