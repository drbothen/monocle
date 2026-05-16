---
document_type: verification-property
level: L4
version: "1.0"
status: in-development
producer: vsdd-factory:formal-verifier
timestamp: 2026-05-17T13:30:00Z
phase: 1b
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, architecture/ARCH-INDEX.md]
input-hash: "[live-state]"
traces_to: prd.md
source_bc: BC-2.02.005
module: monocle-core
proof_method: integration-test+fuzz
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
# Lifecycle fields (DF-030)
lifecycle_status: active
introduced: v1.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-015: `VsddFactoryAdapter::new` + Self-Referential Detection; `None` for Absent Optionals

> **One-per-file:** Each verification property lives in its own file.
> Renumbered from VP-FACTORY-002 (PG-5 historical) per template-compliance Dispatch 5b.

## Property Statement

1. A public constructor `VsddFactoryAdapter::new(workspace_root: PathBuf)
   -> Self` exists.
2. The constructor performs NO validation (it does not panic, does not
   error, does not stat the filesystem); validation happens in `detect()`
   and `read_state()`.
3. The static method `VsddFactoryAdapter::detect(<monocle repo root>)`
   returns `Some(FactoryDetection)` where `display_name == "VSDD Factory"`
   (self-referential test).
4. For a `FactoryState` produced from a STATE.md file lacking
   `current_cycle:`, `state.cycle == None` (NOT `Some("unknown")` or any
   placeholder string).
5. For a `FactoryState` produced from a STATE.md file lacking a §Session
   Resume Checkpoint section, `state.convergence == None`.

## Source Contract

- **BC:** BC-2.02.005 — `VsddFactoryAdapter` (Phase 1 Reference Adapter
  + Frontmatter Parsing Contract).
- **Postcondition/Invariant:** BC-2.02.005 invariants 1-5 plus
  §Edge Case EC-061 (present-but-empty `current_cycle: ""` collapses to
  `None`); cross-property with VP-013 (orthogonal exhaustive-enum
  concern noted in monolithic line 1247).
- **Traces to (historical):** BC-FACTORY-002 (SS-core-types-and-abi.md
  §FactoryAdapter Trait; PRD v1.25 §BC-FACTORY-002 Verification subsection).

## Verification Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| Integration test (primary) | cargo test | Bounded — finite fixture set | Self-referential `detect()`; `None`/`Some` discrimination across 4 fixture STATE.md files; constructor no-op invariant |
| Fuzz (auxiliary) | cargo-fuzz | Unbounded UTF-8 byte space | `parse_frontmatter_field` + `parse_frontmatter_extra_fields` no-panic / no-OOM / flow-style + block-scalar skip invariants |

## Mechanism

Integration test (primary; harness located at
`monocle-core/tests/factory_self_referential.rs` — files in
`<crate>/tests/` are cargo integration tests; PRD v1.25 §7 RTM Test Type
column labels this BC `Integration`); fuzz (auxiliary) via
`cargo fuzz add fuzz_state_md_parser` feeding arbitrary UTF-8 byte
sequences into `parse_frontmatter_field` + `parse_frontmatter_extra_fields`
and asserting no-panic / no-OOM / flow-style + block-scalar skip
invariants.

## Pre-conditions

- `monocle-core` builds cleanly.
- Test fixture STATE.md files are checked in under
  `monocle-core/tests/fixtures/`:
  - `state_minimal.md` — has `current_cycle:` and §Session Resume Checkpoint.
  - `state_no_cycle.md` — lacks `current_cycle:` frontmatter key.
  - `state_no_checkpoint.md` — has `current_cycle:` but lacks §Session
    Resume Checkpoint section.
  - `state_empty_cycle.md` — has `current_cycle: ""` (present-but-empty
    string) — drives the §Post-condition 7 empty-string probe per
    I-R90-4 closure.

## Post-conditions

1. `VsddFactoryAdapter::new(PathBuf::from("/nonexistent/path"))` returns
   a value without error or panic.
2. `VsddFactoryAdapter::detect(&monocle_repo_root)` returns `Some(d)`
   where `d.display_name == "VSDD Factory"` and `d.state_file ==
   monocle_repo_root.join(".factory/STATE.md")`.
3. For fixture `state_no_cycle.md`, `state.cycle.is_none()`.
4. For fixture `state_no_checkpoint.md`, `state.convergence.is_none()`.
5. For fixture `state_minimal.md`, `state.cycle == Some("cycle-001".into())`
   (or whatever the fixture declares) — proves the `None` cases are
   discriminating, not a vacuous default.
6. `parse_frontmatter_field` returns `None` for: empty values, flow-style
   lists (`[...]`), block scalars (`|` or `>` lead), and continuation
   lines.
7. **Empty-string `current_cycle` fixture (per BC-2.02.005 §Edge Case
   EC-061):** For STATE.md frontmatter with `current_cycle: ""`
   (present-but-empty), `state.cycle` MUST equal `None` (NOT
   `Some("".into())`). Verified by integration test loading the
   empty-fixture and asserting `state.cycle.is_none()`. Cross-property
   with `parse_frontmatter_field` empty-value handling (§Post-condition
   6) — the empty-value form is the canonical "absence of meaningful
   content" path and MUST collapse to `None` at the `state.cycle`
   surface, not just at the `parse_frontmatter_field` return.

## Counter-examples

1. The constructor stats `workspace_root` and panics on absent path —
   fails post-condition 1.
2. `read_state` substitutes `"unknown"` for absent `current_cycle:` —
   fails post-condition 3.
3. `parse_frontmatter_field` accepts `awaiting: [a, b]` and returns
   `Some("[a, b]")` — fails post-condition 6 (the v1.2.3 round-20 fix
   site; this VP's permanent fuzz harness prevents recurrence).
4. `parse_frontmatter_field` accepts `phase: |` block-scalar marker and
   returns `Some("|")` — fails post-condition 6.
5. Self-referential detect fails because the `document_type:
   pipeline-state` substring check is too strict (e.g., requires exact
   line equality including trailing whitespace) — fails post-condition 2.
6. **Probe-matrix exhaustiveness regression (empty-string
   `current_cycle`):** `read_state` returns `Some("".into())` for
   frontmatter `current_cycle: ""` (present-but-empty value) instead of
   collapsing to `None` — fails post-condition 7. This is a regression
   class where an implementer treats "absent key" and "present-but-empty
   value" as different paths but only the absent-key path correctly
   yields `None`; the present-but-empty path leaks through as `Some("")`.
   `cargo-mutants` mutation-test rationale: this probe is the leverage
   point for the asymmetric `None`-collapse mutation surface in the
   frontmatter parser.

## Fuzz Harness

`cargo fuzz add fuzz_state_md_parser`. The fuzz target feeds arbitrary
UTF-8 byte sequences into `parse_frontmatter_field(content, "phase")` and
`parse_frontmatter_extra_fields(content, &known_keys)` and asserts: no
panic; no allocation > 1 MiB; flow-style and block-scalar inputs produce
`None` (frontmatter_field) or are skipped (extra_fields). The fuzzer is
seeded with `state_minimal.md`, `state_no_cycle.md`, `state_empty_cycle.md`,
and adversarial malformed corpora (truncated frontmatter, mismatched
quotes, Unicode direction overrides, deep nesting markers). Permanent
fuzz harness — prevents recurrence of the v1.2.3 (F-R20-2) regression
site.

## Probe Matrix

| Probe | Setup | Expected outcome |
|-------|-------|------------------|
| 15.a | `new(PathBuf::from("/nonexistent"))` | No panic; no error; returns adapter |
| 15.b | `detect(&monocle_repo_root)` from `state_minimal.md` | `Some(d)`; `d.display_name == "VSDD Factory"` |
| 15.c | `read_state` over `state_no_cycle.md` | `state.cycle.is_none()` |
| 15.d | `read_state` over `state_no_checkpoint.md` | `state.convergence.is_none()` |
| 15.e | `read_state` over `state_minimal.md` | `state.cycle == Some("cycle-001".into())` |
| 15.f | `parse_frontmatter_field` over `awaiting: [a, b]` | Returns `None` (flow-style reject) |
| 15.g | `parse_frontmatter_field` over `phase: \|` | Returns `None` (block-scalar reject) |
| 15.h | `read_state` over `state_empty_cycle.md` (`current_cycle: ""`) | `state.cycle.is_none()` — present-but-empty collapses to `None` (EC-061) |
| 15.i | Fuzz: arbitrary UTF-8 input | No panic; no >1 MiB allocation |

## Harness Location

- `monocle-core/tests/factory_self_referential.rs` (integration test)
- `fuzz/fuzz_targets/fuzz_state_md_parser.rs` (fuzz harness)
- Test name: `test_BC_FACTORY_002_vsdd_adapter_self_referential_detection`
  (per PRD v1.25 §BC-FACTORY-002, Verification subsection — to be
  migrated to `test_BC_2_02_005_vsdd_adapter_self_referential_detection`
  post BC renumber propagation into source).

## References

- Current as of `2026-05-17T13:30:00Z` (Dispatch 5b).
- Predecessor: monolithic VP-FACTORY-002 at
  `.factory/specs/verification-properties.md` v1.35 (commit 842402c —
  pre-Dispatch-5a state; retired in Dispatch 5b).
- Source contract: `behavioral-contracts/ss-02/BC-2.02.005.md`.
- Architecture: `architecture/SS-core-types-and-abi.md` §FactoryAdapter Trait.
- PRD: `.factory/specs/prd.md` v1.26 §BC-2.02.005 (Dispatch 4 commit 1030c65).
- Cross-VP: VP-013 (orthogonal exhaustive-enum concern for `BlockingSeverity` etc.);
  VP-014 (trait surface this adapter implements).
- Mutation-test pairing: per monolithic §Coverage Matrix, `fuzz` is the
  auxiliary mechanism here because `parse_frontmatter_field` was the
  v1.2.3 (F-R20-2) regression site; permanent fuzz harness prevents
  recurrence.
