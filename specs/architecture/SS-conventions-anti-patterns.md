---
document_type: architecture-section
level: L3
section: "conventions"
version: "1.20"
status: complete
producer: architect
phase: pre-phase-1-architecture
timestamp: 2026-05-14T08:00:00Z
inputs:
  - /Users/jmagady/Dev/monocle/.factory/specs/product-brief.md
  - /Users/jmagady/Dev/monocle/.factory/specs/research/domain-monocle-vision-synthesis.md
input-hash: "[live-state]"
traces_to: "adversary re-audit 0bd4ba9 §Top 8 CRITICAL/IMPORTANT item 3; canonical principle CLAUDE.md commit 3366d58; brief v1.4 commit 70286e1; vision v1.1 commit 0e4b0f4; adversary F-NEW-08 cargo-deny CI gate; ADR-0003 license selection; adversary F-R6-002 + consistency G-02 (round-6 bec535d); human Q-3 weekly R-001 monitoring; brief v1.4.6 §Competitive Positioning; v1.4 round-24 F-R24-adv-5: Test Conventions section added mandating temp-env for all env-mutating tests; v1.5 round-27: F-R26-adv-2 semgrep env-mutation pattern expanded (path-sensitive idioms); F-R26-adv-3 positive-coverage fixture corpus requirement added (POL-11); F-R26-adv-6 Test Conventions semgrep rule consolidated into §Semgrep Rules; v1.6 round-30: F-R30-3 monocle-non-exhaustive-struct-audit-completeness semgrep rule added + fixture corpus entry; v1.7 round-33: F-R32-2 fixture corpus dual-shape requirement for production-code attribute cluster + rule pattern hardening; F-R32-4 Python script edge-case contract (header/separator handling, missing file, malformed delimiters, duplicate delimiters, empty table); v1.8 round-35: F-R34-1 line-anchored delimiter regex for duplicate-detection (defense-in-depth with SS-engine-module §Trace prose de-quoting); F-R34-2 Shape B wildcard corrected from #[...] to #[$ATTR(...)] (standard semgrep metavariable); F-R34-3 paths.include expanded from 4 to 11 workspace crates + binary; v1.9 round-37: F-R36-2 §Trace v1.6 entry de-quoted (Partial-Fix Regression Discipline S-7.01 — v1.8 convention introduction failed to propagate no-verbatim-quoting rule to existing §Trace entries in same file); v1.10 round-39: F-R38-1 Option B — clause 4 Convention rule amended with explicit exception for regex constant definitions in §Trace (delimiter pattern string IS the spec content; cannot be expressed by name alone; narrowly scoped to code-specification blocks defining the constants, not narrative prose); v1.11 round-41: F-R40-1 Option A — CLI --include glob removed from Step 3 check_audit_table.py invocation; the rule's paths.include (expanded to all 12 workspace paths in F-R34-3) is the authoritative scope governor; CLI --include \"monocle-*/src/**/*.rs\" silently excluded the binary crate monocle/src/**/*.rs (no hyphen after monocle); v1.12 round-43: F-R42-adv-1 S-7.01 propagation — F-R32-2 dual-shape fixture discipline propagated to 3 sibling rules (monocle-no-shell-injection, monocle-no-naked-fs-write, monocle-no-raw-env-mutation-in-tests); all-arm fixture coverage made mandatory; expected counts computed from arm counts; Step 1 CI assertion language made normative (MUST); optional CI arm-count sanity check added; v1.13 round-45: F-R44-adv-1 paths.include fixture path + F-R44-adv-2/3/4 narrative count drift; v1.14 round-47: F-R46-2 MEDIUM phantom BC-HOOK-001-006 replaced with attested gene-source reference; F-R46-3 LOW stale step-6 pinpoint reworded to position-free; PG-1 D-042 extension: schema-fact citation convention added; PG-2 META rule extended to cover step-renumbering events; v1.15 round-48: F-R48-cons-1 LOW directional typo in §Trace v1.14 corrected (Convention below → Convention above); v1.16 round-48: F-NEW-PG-1-direction LOW directional typo in §Trace v1.14 PG-1 entry corrected (Citation Convention below → above); PG-3 §Cross-Section Directional Reference Convention codified; SS-engine-module.md v1.1.12 co-edit (audit table rows below → above); v1.17 round-47.3: F-R48R-1 + F-R48R-2 L-number pinpoint sweep; PG-3 §Trace-prose authoring sub-rule codified; SS-engine-module.md v1.1.13 co-edit (§Trace L-numbers → position-free); v1.18 round-49: F-R48-adv-1 PG-2 noun-agnostic generalization + count fix (five→seven); F-R48-adv-2 PG-3 all-prose expansion + main-body L-number sweep; F-R48-adv-3 BC-HOOK-018 gene-source qualifier; D-042 scope codified (.factory/specs/ recursive); SS-engine-module.md v1.1.14 + SS-core-types-and-abi.md v1.2.4 co-edits; v1.19 round-51.1: F-R51-adv-1 PG-4 §-heading-existence convention codified; §Option A mis-anchor (2 sites) → §Trace; §Future audit maintenance mis-anchor → §Cross-Crate Constructor Audit; comprehensive PG-4 sweep across corpus; SS-engine-module.md v1.1.15 + SS-core-types-and-abi.md v1.2.5 + SS-forward-compatibility.md v1.2.6 co-edits; v1.20 round-52.1: F-R52-cons-1 PG-3 self-violation in §Trace v1.19 item (3) fixed (L487 dropped); PG-3-TRACE-NEW-ENTRY META-rule application discipline codified; R51.1 §Trace sweep: SS-core-types-and-abi.md v1.2.6 (at L487 dropped) + SS-forward-compatibility.md v1.2.7 ((L55)/(L57)/(L73) → position-free) + dtu-assessment.md v1.5 (D-042 cascade) co-edits"
project: monocle
---

# Architecture: Code Conventions

## [Section Content]

This file records code-review enforcement rules, triple-confirmed across 8
gene-source ingest syntheses. These conventions are non-negotiable in Phase 1;
violations block PR merge. Enforcement is wired via clippy, semgrep, and the
PR checklist defined in this document.

## Naming Convention

Product name: lowercase `monocle` in code identifiers (crate names such as `monocle-core`,
module names, type names such as `MonocleConfig`); capitalized `Monocle` in prose
headings and documentation. This is the authoritative rule; any prior usage of
`Monocle` in code identifiers is a defect to be fixed before merge.

## Anti-Patterns to Reject in Code Review

The following patterns were observed as failure modes in gene-source repositories
and are explicitly forbidden in monocle's codebase:

- **Shell injection via template strings**: No `Command::new("sh").arg("-c").arg(template_string)` or equivalent `shell=True` pattern. Use `Command::new(binary).args([...])` arg-array form. Shell interpolation of user-controlled strings is a command-injection vector regardless of runtime context.

- **Naked config file writes**: No `std::fs::write` / `tokio::fs::write` for config files. Use `tempfile::persist` (write to temp, then atomic rename). Direct writes leave a corruption window on crash between open and close.

- **Unbounded event channels**: No `tokio::sync::mpsc::unbounded_channel`. Use bounded channel with a drop counter surfaced in the status bar. Unbounded channels mask backpressure and lead to unbounded memory growth under high-frequency hook events.

- **Package-level mutable globals for theme/config**: No `static mut` or `once_cell::Lazy<Mutex<Theme>>` at package level. Use `Arc<RwLock<Theme>>` threaded through the application context. Globals for visual state cause race conditions in multi-thread renders and make theme hot-reload impossible.

- **Single-popup overlay field**: No `Option<PromptModal>` field for the permission overlay. Use `VecDeque<PromptModal>` to support concurrent prompts without silent drop. The single-Option pattern causes the second concurrent prompt to replace the first with no acknowledgment.

## Test-Time Enforcement

All seven mechanisms below are wired in CI and block merge on failure. See §CI Wiring for step ordering.

### Clippy `disallowed_methods` Configuration

Add to workspace `Cargo.toml`:

```toml
[workspace.lints.clippy]
disallowed_methods = [
  { path = "tokio::sync::mpsc::unbounded_channel", reason = "Use bounded mpsc::channel(N) with surfaced drop counter per anti-patterns table" },
  { path = "std::fs::write", reason = "Use tempfile::persist for atomic config writes per anti-patterns table" },
  { path = "tokio::fs::write", reason = "Use tempfile::persist for atomic config writes per anti-patterns table" },
]
```

### Semgrep Rules

Write to `.semgrep.yml` at workspace root. All five rules below are authoritative; the
fifth rule (`monocle-non-exhaustive-struct-audit-completeness`) was added in v1.6 (F-R30-3
audit-completeness check). The fourth rule (`monocle-no-raw-env-mutation-in-tests`) was
added in v1.5 (consolidation of the §Test Conventions CI enforcement rule to create a
single source of truth). Cross-references: §Test Conventions below cites this list as the
canonical rule location.

```yaml
rules:
  - id: monocle-no-shell-injection
    pattern-either:
      - pattern: Command::new("sh")
      - pattern: Command::new("bash")
    message: "Shell injection vector. Use Command::new(binary).args([...]) arg-array form. See conventions.md anti-patterns."
    severity: ERROR
    languages: [rust]
  - id: monocle-no-naked-fs-write
    pattern-either:
      - pattern: std::fs::write(...)
      - pattern: tokio::fs::write(...)
    message: "Naked file write leaves corruption window on crash. Use tempfile::persist. See conventions.md anti-patterns."
    severity: ERROR
    languages: [rust]
  - id: monocle-no-unbounded-channel
    pattern: tokio::sync::mpsc::unbounded_channel(...)
    message: "Unbounded channel masks backpressure. Use bounded mpsc::channel(N) with drop counter. See conventions.md anti-patterns."
    severity: ERROR
    languages: [rust]
  - id: monocle-no-raw-env-mutation-in-tests
    # Covers fully-qualified, module-relative, and use-alias import forms.
    # Rationale for pattern-either expansion (F-R26-adv-2):
    #   - `std::env::set_var(...)` matches only the fully-qualified path.
    #   - `use std::env; env::set_var(...)` is a common Rust idiom that writes
    #     `env::set_var(...)` — NOT matched by the fully-qualified pattern alone.
    #   - Bare-import form (`use std::env::set_var; set_var(...)`) is NOT covered
    #     because semgrep cannot disambiguate `set_var(...)` from a user-defined
    #     function of the same name without full type information. The bare-import
    #     form is documented as discouraged in §Test Conventions prose but is not
    #     enforced by semgrep (noise risk outweighs coverage). Developers using
    #     the bare-import form should use `#[allow]` sparingly only for legitimate
    #     use of a user-defined function named `set_var` or `remove_var`.
    pattern-either:
      - pattern: std::env::set_var($X, $Y)
      - pattern: std::env::remove_var($X)
      - pattern: env::set_var($X, $Y)
      - pattern: env::remove_var($X)
    paths:
      include:
        - "**/tests/**/*.rs"
        - "**/*_test.rs"
        - "**/*tests*.rs"
    message: "Raw env mutation in tests is unsafe in multi-threaded Rust harnesses (Rust 1.86+
      marks set_var/remove_var unsafe). Use temp_env::with_vars (sync) or
      temp_env::async_with_vars (async) from temp-env ^0.3. See SS-conventions-anti-patterns.md
      §Test Conventions."
    severity: ERROR
    languages: [rust]
  - id: monocle-non-exhaustive-struct-audit-completeness
    # Matches any #[non_exhaustive] attribute applied to a pub struct definition.
    # This rule does NOT assert correctness by itself — it is used as the SOURCE
    # for CI audit-completeness checking (see note below).
    #
    # CI script contract (devops-engineer Phase 1 deliverable):
    #   1. Run semgrep with --json to get the list of matched struct names.
    #   2. Parse SS-engine-module.md §Cross-Crate Constructor Audit Table between
    #      the <!-- BEGIN: Cross-Crate Constructor Audit Table --> and
    #      <!-- END: Cross-Crate Constructor Audit Table --> HTML delimiters.
    #   3. Extract struct names from the first column of every data row in the table.
    #   4. For each struct name in the semgrep output, verify it appears in the table.
    #   5. Fail CI if any struct is missing from the table; emit:
    #      "Audit table gap: `<StructName>` carries #[non_exhaustive] but is absent
    #      from the Cross-Crate Constructor Audit Table in SS-engine-module.md.
    #      Update the table and add a constructor if any cross-crate construction
    #      site exists or is anticipated."
    #
    # Semgrep itself does NOT fail the CI step — it only produces the match list.
    # The Python script (not semgrep) fails the step on a gap. This two-step design
    # means the semgrep fixture-corpus assertion (Step 1 in §CI assertions) remains
    # a pure "does this rule match the fixture?" check, separate from the table-gap logic.
    #
    # Scope: only monocle crate source directories. Spec files (.factory/specs/) are
    # excluded — they contain #[non_exhaustive] in code blocks that are prose, not
    # compiled Rust. The path include list is kept narrowly scoped to avoid false
    # matches on fixture files or generated code.
    pattern-either:
      - pattern: |
          #[non_exhaustive]
          pub struct $NAME { ... }
      - pattern: |
          #[non_exhaustive]
          #[$ATTR(...)]
          pub struct $NAME { ... }
    # pattern-either rationale (F-R32-2 / F-R34-2): semgrep's behavior for Rust attribute clusters is not
    # externally documented as strict-or-liberal with respect to intermediate attributes. The first
    # arm matches the minimal shape (no intervening attributes); the second arm matches the
    # production-code shape (#[derive(...)] interposed between #[non_exhaustive] and pub struct).
    # Both arms are required to guarantee the rule fires on actual monocle production structs.
    # F-R34-2: `#[$ATTR(...)]` is the standard semgrep metavariable form for "any attribute with a
    # parenthesized argument list." `$ATTR` matches any identifier (e.g., `derive`, `serde`, `repr`);
    # `(...)` matches any argument list including multi-arg derives like `#[derive(Debug, Clone)]`
    # and key-value attributes like `#[serde(rename_all = "snake_case")]`. The prior `#[...]`
    # form is NOT a documented semgrep wildcard — it is not guaranteed to match any attribute and
    # was replaced in v1.8. Note: `#[$ATTR(...)]` requires the intermediate attribute to have
    # parentheses; a bare intermediate attribute (e.g., a hypothetical `#[copy]` with no args)
    # would not be matched by the second arm. All monocle production structs use parenthesized
    # intermediate attributes (`#[derive(...)]`), so this form is correct for the monocle codebase.
    # If a production struct acquires a bare intermediate attribute, a third arm must be added.
    # See §Semgrep Coverage Hardening — fixture corpus dual-shape requirement for enforcement.
    paths:
      include:
        # All 11 named workspace crates (source: SS-deps-pin-manifest.md §Phase 1 vs Pinned-But-Unused Crates + workspace graph)
        - "monocle-core/src/**/*.rs"
        - "monocle-runtime/src/**/*.rs"
        - "monocle-tui/src/**/*.rs"
        - "monocle-proto/src/**/*.rs"
        - "monocle-ipc/src/**/*.rs"
        - "monocle-config/src/**/*.rs"
        - "monocle-plugin-sdk/src/**/*.rs"
        - "monocle-workflow/src/**/*.rs"
        - "monocle-static/src/**/*.rs"
        - "monocle-fuzz/src/**/*.rs"
        - "monocle-test-harness/src/**/*.rs"
        # Binary crate (monocle — not a monocle-* crate; sits in monocle/ subdirectory per workspace layout)
        - "monocle/src/**/*.rs"
        # Fixture corpus (F-R44-adv-1): semgrep-fixtures/ MUST be included so Step 1 (fixture
        # corpus scan) can target this rule against its fixture file. Without this entry, Step 1
        # runs semgrep against semgrep-fixtures/ but the rule's paths.include rejects all fixture
        # files — producing 0 findings vs expected 2 and causing CI to fail on every run from day 1.
        # The fixture file contains AuditFixtureMinimal and AuditFixtureDerived structs which are
        # NOT production structs; their names are excluded from Step 2 and Step 3 by name-based
        # filtering (see Step 2 special case and Step 3 description below).
        - "semgrep-fixtures/**/*.rs"
        # Spec files (.factory/) are excluded because they never match the above source globs.
    message: "Found #[non_exhaustive] pub struct `$NAME`. Verify it appears in the Cross-Crate Constructor Audit Table in SS-engine-module.md §Cross-Crate Constructor Audit. CI script will fail if absent."
    severity: WARNING
    languages: [rust]
```

### Semgrep Coverage Hardening (POL-11 positive-coverage requirement)

Semgrep rules that return zero findings on every CI run are unverifiable — the rule may be
silently broken (wrong path glob, incompatible pattern for the semgrep version in use, or
path scope that never matches any file). This section specifies the fixture corpus and CI
assertion requirements that give each rule a positive signal on every run.

#### Fixture corpus

The devops-engineer creates `semgrep-fixtures/` at the project root (NOT under `tests/`,
which is the Rust integration test crate). Each semgrep rule has exactly one corresponding
fixture file containing a deliberate violation of that rule.

| Rule ID | Fixture file | Violations required (all arms) | Expected count |
|---------|-------------|--------------------------------|---------------|
| `monocle-no-shell-injection` | `semgrep-fixtures/shell_injection.rs` | **Arm 1:** `Command::new("sh").arg("-c").arg("echo hi");` — **Arm 2:** `Command::new("bash").arg("-c").arg("echo hi");` — both `pattern-either` arms MUST be present | 2 |
| `monocle-no-naked-fs-write` | `semgrep-fixtures/naked_fs_write.rs` | **Arm 1:** `std::fs::write("/tmp/x", b"data").unwrap();` — **Arm 2:** `tokio::fs::write("/tmp/x", b"data").await.unwrap();` — both `pattern-either` arms MUST be present | 2 |
| `monocle-no-unbounded-channel` | `semgrep-fixtures/unbounded_channel.rs` | `tokio::sync::mpsc::unbounded_channel::<u8>();` (single pattern; no `pattern-either`) | 1 |
| `monocle-no-raw-env-mutation-in-tests` | `semgrep-fixtures/tests/raw_env_mutation.rs` | All four `pattern-either` arms MUST be present: **Arm 1:** `std::env::set_var("HOME", "/tmp");` — **Arm 2:** `std::env::remove_var("HOME");` — **Arm 3:** `env::set_var("HOME", "/tmp");` — **Arm 4:** `env::remove_var("HOME");` | 4 |
| `monocle-non-exhaustive-struct-audit-completeness` | `semgrep-fixtures/non_exhaustive_struct.rs` | Both `pattern-either` arms MUST be present: **Shape A** — minimal: `#[non_exhaustive] pub struct AuditFixtureMinimal { pub field: u32 }` (no intervening attribute); **Shape B** — production-code shape: `#[non_exhaustive] #[derive(Debug, Clone)] pub struct AuditFixtureDerived { pub field: u32 }` (`#[derive(...)]` interposed between `#[non_exhaustive]` and `pub struct`, mirroring every real monocle production struct). Rationale: see note below. | 2 |

**Note — why two fixture shapes are required for `monocle-non-exhaustive-struct-audit-completeness` (F-R32-2):**

Semgrep's Rust pattern matching is tree-sitter-based (AST-level), but semgrep's behavior when
matching multi-attribute clusters is not unambiguously documented for the case where intervening
attributes exist between the matched attribute and the `pub struct` keyword. The rule pattern:

```yaml
pattern: |
  #[non_exhaustive]
  pub struct $NAME { ... }
```

In tree-sitter-rust, a `struct_item` node's attributes are represented as sibling `attribute_item`
nodes preceding the `struct` keyword. The pattern above may be interpreted strictly — matching only
a struct whose first and only outer attribute is `#[non_exhaustive]` — or liberally — matching any
struct that carries `#[non_exhaustive]` as one of its attributes regardless of order. This behavior
is NOT verifiable from semgrep's public documentation alone (confirmed via research, F-R32-2
finding rationale).

Every monocle production struct that carries `#[non_exhaustive]` ALSO carries `#[derive(...)]`
interposed between the two, matching Shape B above. If semgrep's matching is strict (position-
sensitive on attribute order), then the current rule pattern would match Shape A (the minimal
fixture already in v1.6) but FAIL to match Shape B (the production-code shape), producing a
false-green: the fixture passes, the production code is never matched, the audit-completeness
check never fires on real structs, and the POL-11 coverage guarantee is worthless.

The dual-shape fixture requirement ensures the CI catches this failure mode: if Shape B produces
zero findings during the fixture corpus step (Step 1), the expected count (2) will not be met and
CI fails with a clear message indicating the rule does not match production-code attribute ordering.

**Rule pattern hardening:** The rule pattern is updated (see §Semgrep Rules above) to use
`pattern-either` to cover both attribute orderings explicitly:

```yaml
pattern-either:
  - pattern: |
      #[non_exhaustive]
      pub struct $NAME { ... }
  - pattern: |
      #[non_exhaustive]
      #[$ATTR(...)]
      pub struct $NAME { ... }
```

This makes the rule correct regardless of semgrep's strict-vs-liberal attribute-cluster semantics.
The second arm uses `#[$ATTR(...)]` — the standard semgrep metavariable form for "any attribute
with a parenthesized argument list." `$ATTR` matches any attribute identifier; `(...)` matches any
argument list, including multi-arg derives like `#[derive(Debug, Clone, Serialize)]`. This arm
covers the production-code shape where exactly one parenthesized intermediate attribute
(`#[derive(...)]`, `#[serde(...)]`, etc.) is interposed between `#[non_exhaustive]` and
`pub struct`. If a production struct acquires a bare intermediate attribute (no parentheses),
a third `pattern-either` arm must be added — the fixture corpus dual-shape requirement will catch
this gap as a CI failure when the expected match count (2) is not reached.
The fixture corpus dual-shape requirement ensures that any regression in rule matching is caught in CI before
it affects production scans.

Each fixture file contains ONLY the violation pattern (plus minimal Rust syntax to make it
parse). Fixture files are NOT part of the Rust workspace (`Cargo.toml` workspace members list
does not include `semgrep-fixtures/`); they exist solely as semgrep targets.

#### CI assertions (three steps)

The following CI step specifications are normative requirements for the devops-engineer to
wire into the GitHub Actions workflow at implementation time. The exact YAML belongs in
`.github/workflows/` (devops-engineer territory); this section specifies the behavioral
contract that YAML must satisfy.

**Step 1 — Fixture corpus scan (positive-coverage assertion):**

Run semgrep against `semgrep-fixtures/` only. For each rule, assert the finding count
equals the expected value defined in the table above. Emit a log line per rule:
`Fixture corpus: N violation(s) detected for rule <rule-id> (expected N) — PASS` or
`Fixture corpus: N violation(s) detected for rule <rule-id> (expected M) — FAIL`.
Fail the CI step if any rule's actual count does not equal the expected count.

Each rule's fixture MUST contain at least one violation per `pattern-either` arm. Expected
counts for Step 1 (computed from each rule's arm count):

- `monocle-no-shell-injection` — 2 arms (`Command::new("sh")`, `Command::new("bash")`); expected count: **2**
- `monocle-no-naked-fs-write` — 2 arms (`std::fs::write`, `tokio::fs::write`); expected count: **2**
- `monocle-no-unbounded-channel` — 1 pattern (no `pattern-either`); expected count: **1**
- `monocle-no-raw-env-mutation-in-tests` — 4 arms (`std::env::set_var`, `std::env::remove_var`, `env::set_var`, `env::remove_var`); expected count: **4**
- `monocle-non-exhaustive-struct-audit-completeness` — 2 arms (Shape A minimal, Shape B with `#[derive(...)]`); expected count: **2**

If any rule's actual finding count does not equal the expected count, CI MUST fail with an
explicit error message identifying the missing arm, for example:
`Fixture corpus: 1 violation(s) detected for rule monocle-no-shell-injection (expected 2) — FAIL: arm 'Command::new("bash")' produced no findings; verify fixture semgrep-fixtures/shell_injection.rs contains this pattern`.

**CI sanity check — future arm additions:** The CI script MUST also assert that for each
`pattern-either` rule, the declared expected count in the CI configuration is less than or
equal to the actual fixture finding count. When a new `pattern-either` arm is added to any
rule in `.semgrep.yml`, the corresponding fixture file MUST be updated with a violation for
that arm and the expected count in the CI step MUST be incremented. Failure to do both in
the same PR will be caught on the next fixture corpus run: the new arm produces zero findings
in the stale fixture, causing the count to fall below the updated expected value — CI fails.

**Step 2 — Production scan (zero-findings assertion):**

Run semgrep against the production Rust source (`src/`, `crates/`, or equivalent workspace
source directories — NOT `semgrep-fixtures/`, NOT `tests/` for the production-code rules).
Assert zero findings for each rule. Emit a log line per rule:
`Production scan: 0 violations for rule <rule-id> (clean)` or
`Production scan: N violations for rule <rule-id> — FAIL (see semgrep output above)`.
Fail the CI step if any rule returns a non-zero count.

**Special case — `monocle-non-exhaustive-struct-audit-completeness` (audit-completeness rule):**

This rule uses `severity: WARNING` (not `ERROR`) and does NOT participate in Step 2's
zero-findings assertion — it is expected to match every `#[non_exhaustive] pub struct` in
the codebase (by design). Instead, its production-scan output is consumed by a separate
CI step.

**Fixture-name exclusion for Step 2 (F-R44-adv-1):** The rule's `paths.include` now
includes `semgrep-fixtures/**/*.rs` to enable Step 1 fixture corpus scanning (see
`paths.include` rationale above). This means a production-scope semgrep run (Step 2)
without path filtering would also scan `semgrep-fixtures/non_exhaustive_struct.rs` and
return findings for `AuditFixtureMinimal` and `AuditFixtureDerived`. Since Step 2 is
skipped for this rule (it uses WARNING severity and is not a zero-findings rule), this has
no direct impact on Step 2's assertion logic. However, to document the exclusion contract
for future maintainers: if the rule's severity is ever promoted to ERROR or a zero-findings
assertion is added, the CI script MUST filter out struct names `AuditFixtureMinimal` and
`AuditFixtureDerived` from the Step 2 production-scan output before asserting zero findings.
These names are fixture-only; their presence in `semgrep-fixtures/non_exhaustive_struct.rs`
is intentional and must not be treated as a production violation.

**Step 3 — Audit-table gap check (Python script):**

After Step 2, run `scripts/check_audit_table.py` (devops-engineer Phase 1 deliverable):
```
python scripts/check_audit_table.py \
  --semgrep-json <(semgrep --config .semgrep.yml --json) \
  --spec-file .factory/specs/architecture/SS-engine-module.md \
  --rule-id monocle-non-exhaustive-struct-audit-completeness
```

The script:
1. Parses the semgrep JSON output to extract all struct names matched by the rule.
2. **Removes fixture struct names from the semgrep output set (F-R44-adv-1):** Before
   computing the set difference, the script MUST remove the known fixture struct names
   `AuditFixtureMinimal` and `AuditFixtureDerived` from the semgrep-enumerated set. These
   names are present because `semgrep-fixtures/**/*.rs` is included in `paths.include` to
   enable Step 1 fixture corpus scanning. The fixture structs are not production structs and
   must not be looked up in the audit table. Exclusion is by exact struct name (string
   equality after stripping whitespace and backticks), not by file path, so this remains
   correct even if the fixture file is moved. If either name is found in the semgrep output,
   it is silently dropped before step 3 below. The exclusion list is:
   `FIXTURE_STRUCT_NAMES = {"AuditFixtureMinimal", "AuditFixtureDerived"}`
   This constant MUST be defined as a named set in the script (not an inline literal) so
   that future fixture struct additions are a single-location update.
3. Opens `SS-engine-module.md` and locates the audit table using the line-anchored regexes
   defined in clause 4 of §Contract edge cases:
   `BEGIN_DELIMITER_REGEX = r'^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$'`
   `END_DELIMITER_REGEX   = r'^<!-- END: Cross-Crate Constructor Audit Table -->$'`
   Reads all lines between the first line matching `BEGIN_DELIMITER_REGEX` and the first
   subsequent line matching `END_DELIMITER_REGEX`.
4. Extracts the struct name from the first column of each data row (text between the first
   `|` and second `|` on each row, stripped of surrounding backticks and whitespace).
5. Computes the set difference: structs in (semgrep output minus fixture names) but NOT in
   the table.
6. Fails with exit code 1 and prints the gap list if the difference is non-empty:
   `Audit table gap: following structs carry #[non_exhaustive] but are absent from the
   Cross-Crate Constructor Audit Table: <list>. Update SS-engine-module.md §Cross-Crate
   Constructor Audit and add a constructor if any cross-crate construction site exists.`
7. Exits 0 with `Audit table: complete (N structs declared, N structs found by semgrep).`
   if the sets are equal. N reflects the post-exclusion count (production structs only).

**Contract edge cases (F-R32-4):** The following behaviors are normative requirements for the
`check_audit_table.py` implementation. The devops-engineer MUST implement all five. No
implementer's-choice behavior is permitted.

1. **Header and separator row handling.** When iterating lines between the delimiter markers,
   the script MUST skip two categories of non-data rows before attempting to extract struct names:
   - Separator rows: skip any line matching the regex `r'^\|[-: |]+\|$'` (a row whose cells
     contain only hyphens, colons, spaces, and pipe characters — the markdown table separator).
   - Header rows: skip any line whose first cell (text between the first `|` and second `|`,
     stripped) equals the literal string `Struct` or begins with `**` (bold-formatted header).
   Lines that are neither separator nor header are treated as data rows. Struct names are extracted
   from data rows only.

2. **Missing spec file.** If the file path supplied via `--spec-file` does not exist, the script
   MUST exit with status code 1 and emit exactly:
   `Error: spec file not found: <path>` (where `<path>` is the value passed to `--spec-file`).
   No further processing occurs.

3. **Malformed delimiter pairs.** The script MUST validate delimiter pairing before reading table
   data:
   - If `<!-- BEGIN: Cross-Crate Constructor Audit Table -->` is found but no subsequent
     `<!-- END: Cross-Crate Constructor Audit Table -->` exists, exit with status 1 and emit:
     `Error: found BEGIN delimiter with no matching END delimiter in <path>`.
   - If `<!-- END: Cross-Crate Constructor Audit Table -->` is found but no preceding
     `<!-- BEGIN: Cross-Crate Constructor Audit Table -->` exists, exit with status 1 and emit:
     `Error: found END delimiter with no preceding BEGIN delimiter in <path>`.

4. **Duplicate delimiters.** Delimiter detection MUST use line-anchored regex to avoid false
   positives from prose mentions of the delimiter strings (e.g., in §Trace text or backtick-quoted
   examples). The canonical regexes are:

   ```python
   BEGIN_DELIMITER_REGEX = r'^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$'
   END_DELIMITER_REGEX   = r'^<!-- END: Cross-Crate Constructor Audit Table -->$'
   ```

   Line-anchoring (`^...$`, using `re.match` or `re.fullmatch` on stripped lines, or
   `re.search` with the anchors) excludes:
   - Backtick-wrapped prose mentions (the marker is not the full line content).
   - Indented prose references (leading whitespace breaks the `^` anchor when matching the
     raw line; strip the line before matching to handle trailing newlines only — do NOT strip
     leading whitespace, as leading whitespace itself is the discriminant).
   - Mid-line narrative occurrences (additional text before or after the marker breaks anchoring).

   The line-anchored regex is the sole mechanism for counting delimiter occurrences. If
   `BEGIN_DELIMITER_REGEX` matches more than once in the spec file, the script MUST exit with
   status 1 and emit:
   `Error: multiple BEGIN delimiters found in <path>; spec file is ambiguous`.
   Similarly, if `END_DELIMITER_REGEX` matches more than once, exit with status 1 and emit:
   `Error: multiple END delimiters found in <path>; spec file is ambiguous`.
   Duplicate-delimiter detection runs before any table content is read.

   **Convention (defense in depth):** Do NOT quote the audit-table delimiter strings verbatim
   in §Trace prose or any spec narrative. Refer to them by name (e.g., "the BEGIN/END delimiter
   markers wrapping the audit table" or "the HTML comment markers defined in SS-conventions
   §Semgrep Coverage Hardening"). Verbatim inline quoting is the root cause the line-anchored
   regex must guard against; the convention prevents the regression even if the regex is
   loosened in a future edit.

   **Exception — regex constant definitions in §Trace:** A §Trace entry that introduces or
   modifies `BEGIN_DELIMITER_REGEX` or `END_DELIMITER_REGEX` MAY include the full constant
   assignment expression (e.g., `BEGIN_DELIMITER_REGEX = r'^...$'`) because the delimiter
   pattern string IS the specification content — it cannot be expressed by name alone.
   This exception is narrowly scoped: it applies only to code-specification blocks that define
   the regex constants themselves, not to any narrative prose that incidentally references the
   delimiter text. All other §Trace prose and spec narrative remains subject to the no-verbatim-
   quoting rule without exception.

5. **Empty table.** If the table between the delimiters contains zero data rows (i.e., only
   separator and/or header rows, or no rows at all), the script MUST exit with status 1 and emit:
   `Error: Cross-Crate Constructor Audit Table in <path> has no data rows; this is a spec gap`.
   Rationale: an empty table indicates the audit table has not been populated, which is a spec
   defect — not a legitimate "zero structs declared" state. A codebase with zero
   `#[non_exhaustive]` structs would still produce an empty semgrep output, which is handled
   by the normal set-difference logic (0 structs declared = 0 structs found = complete). The
   empty-table check is distinct from the zero-semgrep-findings case.

The fixture-corpus assertion for this rule (Step 1 above) must match **2 findings** in
`semgrep-fixtures/non_exhaustive_struct.rs` — one for `AuditFixtureMinimal` (Shape A, minimal
form) and one for `AuditFixtureDerived` (Shape B, production-code form with `#[derive(...)]`
interposed). Expected count: 2. If CI reports 1 instead of 2, the rule's `pattern-either` second
arm is not matching the production-code attribute shape — the rule must be corrected before
proceeding. If CI reports 0, the rule is entirely non-functional. Both count-mismatch conditions
block CI (Step 1 fails; Steps 2 and 3 are skipped per step-ordering rule).
The production scan expected count is NOT zero (not a conventional zero-findings rule) —
the fixture-corpus assertion is the only POL-11 coverage check for this rule.

**Step ordering in CI workflow:**

All three steps run after `cargo clippy` and before `cargo test`. They are separate CI steps
(distinct `name:` entries) so failures are individually addressable in the GitHub Actions
UI. The fixture-corpus step (Step 1) runs first; if it fails (rule broken), Steps 2 and 3
are skipped to avoid misleading results from a non-functioning rule.

**Note on scope:** The CI wiring (actual `.github/workflows/` YAML, fixture file content,
`scripts/check_audit_table.py` implementation, semgrep version pin) is the devops-engineer's
Phase 1 deliverable. This section specifies the behavioral requirement with enough precision
that the implementer can wire it without round-trips to the architect.

### PR Template Checklist

Write to `.github/PULL_REQUEST_TEMPLATE.md`:

```markdown
## Monocle Convention Checklist
- [ ] All channel instantiations use `mpsc::channel(N)` with explicit bound N
- [ ] All config writes use `tempfile::persist` (or equivalent atomic-rename pattern)
- [ ] No `Command::new("sh")` or `Command::new("bash")` — use arg-array form
- [ ] No package-level mutable globals for theme/config — use `Arc<RwLock<>>` threaded through context
- [ ] All permission overlays use `VecDeque<PromptModal>`, not `Option<PromptModal>`
- [ ] All public APIs have explicit error taxonomy via `thiserror`
- [ ] No `println!` in production code paths (use `tracing` with structured fields)
```

### Channel-Drop Integration Test Skeleton

Add to `monocle-test-harness`:

```rust
#[tokio::test]
async fn synthetic_high_frequency_hook_load_does_not_overflow() {
    let (tx, mut rx) = mpsc::channel(1024);
    let drop_counter = Arc::new(AtomicU64::new(0));
    // sustain 1000 events/sec for 30s via spawned producer
    // assert: drop_counter.load(Ordering::Relaxed) <= 50
    // assert: rx drains all non-dropped events
}
```

Target: 1000 events/sec sustained for 30 seconds; drop counter must remain at or below 50 drops. This is the integration test anchor for the bounded channel anti-pattern enforcement.

### CI Wiring

GitHub Actions step ordering (all block merge on failure):

1. `cargo fmt --check` — block merge on format deviation
2. `cargo clippy --workspace -- -D warnings` — block on any lint, including `disallowed_methods`
3. `semgrep --config .semgrep.yml --error` — block on any rule match
4. `cargo test --workspace` — block on any test failure, including channel-drop integration test
5. `cargo deny check licenses bans advisories sources` — block on license policy violation, banned crate, active RUSTSEC advisory, or non-registry source; uses `deny.toml` at workspace root (see §deny.toml configuration below)
6. `cargo audit` — block on new RUSTSEC advisory affecting pinned versions; run weekly scheduled via `cargo audit --json`

#### deny.toml configuration

Place `deny.toml` at the workspace root. This file operationalizes ADR-0003 (MIT/Apache-2.0 dual-license selection) and enforces supply-chain hygiene.

```toml
[graph]
# Phase 1 targets — adjust at workspace init during /vsdd-factory:create-architecture
# when platform targets are pinned (e.g. ["x86_64-unknown-linux-gnu", "aarch64-apple-darwin"])
targets = []
all-features = false

[advisories]
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
vulnerability = "deny"
unmaintained = "warn"
yanked = "deny"
notice = "warn"
ignore = []

[licenses]
unlicensed = "deny"
allow = [
    "MIT",
    "Apache-2.0",
    "Apache-2.0 WITH LLVM-exception",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Unicode-DFS-2016",
    "Unicode-3.0",
    "MPL-2.0",  # nucleo; file-level copyleft, does not propagate to binary
    "Zlib",
]
deny = ["GPL-1.0", "GPL-2.0", "GPL-3.0", "AGPL-1.0", "AGPL-3.0", "LGPL-2.0", "LGPL-2.1", "LGPL-3.0"]
copyleft = "warn"
allow-osi-fsf-free = "either"
confidence-threshold = 0.8
exceptions = []

[bans]
multiple-versions = "warn"
wildcards = "deny"
deny = [
    { name = "openssl", reason = "use rustls; openssl is a deployment-complexity liability" },
    { name = "openssl-sys", reason = "see openssl" },
    { name = "tokio", version = "<1.52", reason = "RUSTSEC remediated 1.52+" },
    { name = "russh", version = "<0.60", reason = "transitive rsa pre-release RUSTSEC-2023-0071" },
]
skip = []
skip-tree = []

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
allow-git = []
```

**Rationale for `[bans]` deny entries:**

- **`openssl` / `openssl-sys`**: OpenSSL introduces system-library linkage that breaks hermetic builds and creates version-mismatch failures in CI containers and musl targets. monocle uses rustls (pure-Rust TLS) throughout; any transitive pull of openssl indicates a dependency tree problem to be resolved at the source, not papered over with feature flags.
- **`tokio < 1.52`**: RUSTSEC advisories were remediated starting in tokio 1.52. Any version below this floor represents a known-vulnerable async runtime. The `SS-deps-pin-manifest.md` already pins tokio at 1.52 for Phase 1 — this ban acts as a floor guard that will fire if a transitive dep drags in a pre-remediation version.
- **`russh < 0.60`**: russh versions below 0.60 pull in a pre-release `rsa` crate affected by RUSTSEC-2023-0071 (RSA key recovery via Marvin attack). russh is not a direct monocle dependency but may appear transiently through plugin SDK paths in Phase 3; the ban prevents accidental introduction.

**Rationale for MPL-2.0 inclusion:**

MPL-2.0 (Mozilla Public License 2.0) is file-level copyleft: it requires modifications to MPL-licensed files to be released under MPL, but does not propagate to files in different compilation units. monocle uses nucleo 0.5 (matcher/scorer library; see ADR-0002) which is MPL-2.0. Because monocle links nucleo as an unmodified library crate and does not modify nucleo source files, the file-level copyleft does not impose redistribution obligations on monocle's own code. The `copyleft = "warn"` setting ensures any new MPL-2.0 additions surface for human review.

**`targets = []` during Phase 1:**

The empty targets list instructs cargo-deny to analyze the dependency graph without restricting to a specific platform triple. This is correct during spec and early implementation phases when the exact deployment targets have not been finalized. During `/vsdd-factory:create-architecture` workspace initialization, the targets list will be populated with the pinned platform triples (e.g., `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`) to enable platform-specific dependency filtering.

#### GitHub Actions wiring

Use the `EmbarkStudios/cargo-deny-action` composite action, pinned to SHA. Place this step between `cargo test` and `cargo audit` in the CI workflow:

```yaml
- name: cargo-deny
  uses: EmbarkStudios/cargo-deny-action@v2
  with:
    log-level: warn
    command: check
    arguments: --all-features --workspace licenses bans advisories sources
```

Note: pin the `uses:` line to a full commit SHA when creating the workflow file (per the project's action-pinning requirement). The `@v2` reference above is illustrative; the devops-engineer must resolve the SHA at workflow creation time using `gh api repos/EmbarkStudios/cargo-deny-action/git/refs/tags/v2`.

### SBOM Generation

SBOM (Software Bill of Materials) generation via `cargo sbom` (or `cargo cyclonedx`) is run per-release tag in CI. Output format: CycloneDX JSON 1.6 (CISA-compliant; EU CRA-compliant). SBOM is attached to the GitHub Release artifact. Per-PR runs are NOT gated on SBOM (only on cargo-deny); SBOM is a release-time artifact, not a merge gate. License audit per merge is handled by cargo-deny (above).

## R-001 Monitoring Workflow

### Purpose

Weekly scheduled check of Anthropic agent-view release notes against the 4 R-001 re-eval trigger conditions. Opens a labeled GitHub Issue when any condition matches. Per brief v1.4.6 §Competitive Positioning.

This section specifies the workflow contract. The actual implementation files (`/.github/workflows/r001-monitor.yml` and `scripts/r001-monitor.py`) are created during Phase 1 `/vsdd-factory:create-architecture` by the devops-engineer. This is a SPEC, not the implementation.

### Trigger Conditions

Four conditions, any single match opens a GitHub Issue:

| ID | Condition | Rationale |
|---|---|---|
| (a) | Anthropic announces hook-protocol ingestion as a first-class agent-view capability | Directly commoditizes monocle's hook-event pipeline (gene-source canonical 5-hook matrix: BC-HOOK-007) |
| (b) | Anthropic ships diff-preview or cascaded permission-queue functionality inside agent view | Directly commoditizes monocle's permission overlay plane (VecDeque\<PromptModal\>) |
| (c) | Anthropic extends agent view beyond Claude Code (e.g., supports a non-Claude harness) | Attacks monocle's multi-harness federation differentiator |
| (d) | Anthropic publishes a multi-harness session-management spec or RFC | Pre-empts monocle's Phase 4 harness-federation roadmap |

### Workflow Specification — `.github/workflows/r001-monitor.yml`

```yaml
name: R-001 Monitor (Anthropic agent-view trigger conditions)

on:
  schedule:
    - cron: '0 14 * * 1'  # Every Monday at 14:00 UTC (10am ET / 7am PT)
  workflow_dispatch:  # Allow manual trigger for ad-hoc checks

permissions:
  contents: read
  issues: write

concurrency:
  group: r001-monitor
  cancel-in-progress: false

jobs:
  check-anthropic-release-notes:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Python
        uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - name: Install dependencies
        run: pip install requests beautifulsoup4
      - name: Fetch agent-view docs and release notes
        env:
          ANTHROPIC_DOCS_URL: 'https://code.claude.com/docs/en/agent-view'
          ANTHROPIC_RELEASE_NOTES_URL: 'https://docs.claude.com/en/release-notes/claude-code'
        run: python scripts/r001-monitor.py
      - name: Open issue on trigger match
        if: env.R001_TRIGGER_MATCHED == 'true'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const triggerReport = fs.readFileSync('r001-trigger-report.md', 'utf8');
            await github.rest.issues.create({
              owner: context.repo.owner,
              repo: context.repo.repo,
              title: `[R-001 TRIGGER] Anthropic agent-view release matches re-eval condition`,
              body: triggerReport,
              labels: ['risk', 'r-001', 'competitive-monitoring']
            });
```

**Action pinning requirement:** The devops-engineer MUST resolve `actions/checkout@v4`, `actions/setup-python@v5`, and `actions/github-script@v7` to full commit SHAs at workflow creation time. The `@vN` references above are illustrative version pointers; the final file must pin to SHA per the project action-pinning requirement (see §CI Wiring).

### Companion Script Contract — `scripts/r001-monitor.py`

The script is specced here; the full Python implementation is the devops-engineer's Phase 1 deliverable.

**Inputs:** Two public URLs (set via env vars in the workflow):
- `ANTHROPIC_DOCS_URL`: `https://code.claude.com/docs/en/agent-view`
- `ANTHROPIC_RELEASE_NOTES_URL`: `https://docs.claude.com/en/release-notes/claude-code`

**Processing:**
1. Fetch both URLs via `requests`; extract visible text via BeautifulSoup HTML parsing.
2. For each of the 4 trigger conditions, evaluate keyword/regex patterns against the combined text corpus:
   - **(a)** keywords: `hook protocol`, `hook ingestion`, `hooks api`, `PreToolUse`, `PostToolUse`, `PermissionPrompt`
   - **(b)** keywords: `diff preview`, `diff overlay`, `permission queue`, `cascaded permission`, `permission stack`
   - **(c)** keywords: any harness name OTHER than `Claude Code` — e.g., `Codex`, `Aider`, `Cursor`, `Continue`, `GitHub Copilot` (case-insensitive, whole-word match to reduce false positives)
   - **(d)** keywords: `multi-harness`, `harness-agnostic`, `session management spec`, `session management RFC`, `agent-view spec`

**Outputs:**
- If ANY condition matches: set GitHub Actions env var `R001_TRIGGER_MATCHED=true` and write `r001-trigger-report.md` containing: matched condition ID(s), matched text excerpt(s) (up to 200 chars each), source URL, and fetched UTC timestamp (ISO 8601).
- Always (match or no match): append a JSON log entry to `r001-monitor-log/YYYY-MM-DD.json` recording: run timestamp, conditions checked, match results (true/false per condition), and fetch status (ok/unreachable/parse-error) per URL. This log file is committed back to a maintenance branch (`chore/r001-monitor-log`) so the historical trail is preserved even when no trigger fires.

### Failure Modes

| Failure | Behavior |
|---|---|
| URL unreachable | Log warning; do NOT set `R001_TRIGGER_MATCHED`; do NOT open trigger issue (no false positive) |
| HTML structure change (parsing yields empty or implausible content) | Log warning; open a SEPARATE maintenance issue labeled `r-001-monitor-broken` (distinct from trigger label `r-001`) |
| Cron timeout / workflow failure | Retry once via manual `workflow_dispatch` within 24h; if second run also fails, open maintenance issue labeled `r-001-monitor-broken` |

The maintenance issue label `r-001-monitor-broken` must be created in the repository at initialization time (different from the trigger label `r-001`). The devops-engineer creates both labels during Phase 1 repo setup.

### Quarterly Keyword Maintainer Review

The keyword patterns in §Companion Script Contract above are anchored to Anthropic's current terminology as of brief v1.4.6. Anthropic may introduce new wording (e.g., referring to hook-protocol ingestion under a new product name) that does not match current patterns, creating false-negative drift.

**Required cadence:** Review and update keyword patterns quarterly (every 13 weeks). Track via a recurring maintenance issue tagged `r-001-keywords-review`, created automatically by the workflow at each calendar quarter boundary (1 January, 1 April, 1 July, 1 October). The quarterly creation logic is embedded in `scripts/r001-monitor.py` using the run date.

**Review checklist:**
- [ ] Check Anthropic release notes for new terminology covering hook-protocol, diff-preview, multi-harness, or session-management surface areas.
- [ ] Update keyword lists in `scripts/r001-monitor.py` to cover new wording.
- [ ] Run workflow manually (`workflow_dispatch`) after update to verify no spurious matches on prior release notes.
- [ ] Commit updated script and close the `r-001-keywords-review` issue.

### Cost

GitHub Actions free tier covers weekly cron (`0 14 * * 1`) plus light Python compute (typically under 30 seconds per run). No external API costs — both source URLs are public Anthropic documentation. No GitHub API rate-limit risk: one issue creation per trigger event, well within the 5,000 requests/hour ceiling for `GITHUB_TOKEN`.

## Test Conventions

### Environment Variable Mutation in Tests

Tests that mutate `std::env` MUST use `temp-env 0.3+` (`with_vars` for sync closures;
`async_with_vars` for async closures). Raw `std::env::set_var` / `std::env::remove_var` is
**forbidden** in tests — it is unsound in multi-threaded Rust test harnesses (Rust 1.86+
marks these functions `unsafe` for exactly this reason), and lacks panic-safe cleanup.
`temp-env` provides RAII cleanup that fires on both normal return and panic exit.

**Forbidden pattern:**

```rust
// FORBIDDEN: raw env mutation — race-prone, leaks on panic
std::env::set_var("HOME", "/tmp/test-home");
// ... test body ...
std::env::remove_var("HOME"); // never executes if test panics
```

**Required pattern (sync closure):**

```rust
// REQUIRED: RAII cleanup on normal return AND panic
temp_env::with_vars(
    [("HOME", None::<&str>)],
    || {
        // test body — env is restored when closure exits
    },
);
```

**Required pattern (async closure — requires `features = ["async_closure"]`):**

```rust
// REQUIRED for async test bodies
temp_env::async_with_vars(
    [("HOME", None::<&str>)],
    async {
        // async test body — env is restored when future completes or panics
    },
).await;
```

**Dependency declaration** (`monocle-runtime/Cargo.toml` `[dev-dependencies]`):

```toml
temp-env = { version = "^0.3", features = ["async_closure"] }
```

The `async_closure` feature must always be enabled even if only sync closures are used in
a given crate's tests, to keep the dev-dependency declaration consistent across all crates
that add `temp-env`.

**Canonical usage example:** BC-ENGINE-002-ERR test in
`monocle-runtime/tests/engine_module.rs` (see SS-engine-module.md §Behavioral Contracts).

**CI enforcement:** Rule `monocle-no-raw-env-mutation-in-tests` is the 4th semgrep rule
in §Semgrep Rules above — it is the canonical single-source-of-truth location for this
rule. The rule uses `pattern-either` to cover both the fully-qualified form
(`std::env::set_var`, `std::env::remove_var`) and the module-relative form
(`env::set_var`, `env::remove_var`), scoped to test file paths only via `paths.include`.
See §Semgrep Rules for the full rule definition and §Semgrep Coverage Hardening for the
positive-coverage fixture corpus requirement (POL-11).

**Bare-import form** (`use std::env::set_var; set_var(...)`): not enforced by semgrep
(cannot disambiguate from a user-defined function without full type resolution). This form
is also discouraged; developers using it must use `temp_env::with_vars` anyway. If a
false-positive suppression is needed for a legitimate user-defined `set_var` function,
add a `# nosemgrep: monocle-no-raw-env-mutation-in-tests` comment — NOT `#[allow]`.

## Gene-Source Citations

| Anti-Pattern | Gene-Source Evidence |
|---|---|
| Shell injection | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §mixins-r1: Python `subprocess(shell=True)` with template string in CLAUDE.md injection path |
| Naked config file writes | `nikiforovall-lazyclaude-pass-8-final-synthesis-v2.md` §services-r1..r3: atomic-write gap finding; `write_text` with no temp-file intermediary |
| Unbounded channels | `any-context-lazyclaude-pass-8-final-synthesis-v2.md` §broker-r1: BC-BROKER-003 documents unbounded channel as a confirmed failure mode in the broker subsystem; broker drops are completely silent (no log, no metric, no counter) per BC-BROKER-006 |
| Theme globals | `lazygit-pass-8-final-synthesis.md` §pkg/gui: package-level theme globals causing render-thread contention |
| Single-popup overlay | `lazygit-pass-8-final-synthesis.md` §pkg/gui popup: `Option<Popup>` drop-on-concurrent pattern; concurrent modal opens silently drop the pending prompt |
| Raw env mutation in tests | `monocle` round-21 adversary trace + round-24 F-R24-adv-1: `std::env::set_var`/`remove_var` is unsound in multi-threaded test harnesses; data-race on `HOME` between concurrent test threads causes non-deterministic failures; cleanup leaks on panic; `temp-env` is the canonical RAII fix |

## Schema-Fact Citation Convention

Added in v1.14 (round-47, PG-1 D-042 extension). Closes the root-cause class exposed by F-R46-1.

**Rule:** When a document asserts a factual claim about the schema content of another document's
table, matrix, or struct (e.g., "field X is present in all N hook body schemas"), the assertion
MUST:

1. Cite a single canonical anchor document as the source of truth (not a generic descriptor).
2. Name the specific column or table within that anchor (e.g., "monocle-canonical column of
   dtu-assessment.md §Endpoint matrix" — not "the DTU endpoint matrix").
3. Include a grep re-validation pattern in the §Trace of the citing document that can be run
   to re-verify the claim against the canonical anchor at any future version bump of either
   the citing or the cited document.

**Anti-pattern:** "field X is already present in all 5 hook body schemas per the DTU endpoint
matrix." (Ambiguous anchor; does not distinguish gene-source vs monocle-canonical; no grep
validation pattern.)

**Correct form:** "field X is present in all 5 monocle-canonical hook body schemas per
dtu-assessment.md v1.4 §monocle-canonical column, verified against SS-core-types-and-abi.md
v1.2.5 §Non-Exhaustive Inner Structs. Re-validation grep: `grep -rn 'field X.*all.*hook' .factory/specs/`"

**D-042 integration:** The D-042 workflow (primary and secondary grep patterns, documented in
SS-forward-compatibility.md §Trace §D-042 WORKFLOW RULE corrected) covers version-citation
staleness. Schema-fact citation validation requires an ADDITIONAL grep before every version
bump of any document containing a schema-fact assertion.

**D-042 CANONICAL SCOPE (codified v1.18, round-49 PG-D042-BURST-SKIP closure):**
The D-042 grep MUST be run at `.factory/specs/` recursive scope — NOT `.factory/specs/architecture/`
only. Restricting to `architecture/` silently excludes `.factory/specs/product-brief.md`,
`.factory/specs/dtu-assessment.md`, `.factory/specs/research/domain-monocle-vision-synthesis.md`,
and all other spec artifacts above the `architecture/` subtree. This was the confirmed root
cause of seven recurrences of citation-staleness (rounds 26, 32, 36, 38, 40, 42, and the
PG-D042-BURST-SKIP pattern itself). The corrected scope is documented in
SS-forward-compatibility.md §Trace §D-042 WORKFLOW RULE corrected (v1.2.3); this entry
canonicalizes it in SS-conventions for pre-commit self-check use.

**Correct D-042 primary grep (strict form):**
```
grep -rn "SS-[a-z-]*\.md v" .factory/specs/
```

**Correct D-042 secondary grep (anchor-tolerant):**
```
grep -rn "SS-[a-z-]*\.md.*v[0-9]" .factory/specs/
```

Both patterns MUST use `.factory/specs/` as the root — not `.factory/specs/architecture/`.

The grep pattern for hook body field presence (schema-fact validation):

```
grep -rn "session_id.*all.*hook\|all.*hook.*session_id\|present in all 5\|in all 5 hook" .factory/specs/
```

Run this grep in addition to the D-042 version-citation greps before any version bump of
`dtu-assessment.md` or `SS-core-types-and-abi.md`.

## Phantom-ID Convention

Added in v1.14 (round-47, F-R46-2 root-cause closure).

**Rule:** Every BC ID, VP ID, or other VSDD artifact ID referenced in a spec document MUST be
attested — verifiable in at least one of: (a) the pre-staged BC table in SS-forward-compat.md
§Cross-Phase Decisions, (b) the gene-source canonical BC set cited by name with document
provenance, or (c) a BC-INDEX.md or VP-INDEX.md row.

**Forbidden:** Citing a BC ID range (e.g., "BC-HOOK-001–006") that appears in no index, no
pre-staged table, and no named gene-source reference. Such citations are phantom forward-references
that block implementer resolution.

**Prevention grep (run before every commit touching BC ID references):**

```
grep -rn "BC-[A-Z]*-[0-9]" .factory/specs/ | grep -v "gene-source\|any-context\|BC-HOOK-007\|BC-RING\|BC-ABI\|BC-TYPES\|BC-FACTORY\|BC-PROTO\|BC-AUTH\|BC-LOCK\|BC-ENGINE"
```

Any BC ID that does not appear in the allowlist above must be verified against the pre-staged
table in SS-forward-compatibility.md or a named gene-source reference before committing.

## Cross-Section Directional Reference Convention

Added in v1.16 (round-48, F-NEW-PG-1-direction + PG-3 root-cause closure).

**Rule:** Cross-section directional qualifiers (`above` / `below`) in cross-section references
(e.g., `(see §Foo below)`, `(see §Foo above)`) MUST be re-verified at every version bump of the
citing document, because section insertions, reorderings, and §Trace growth can invert the
positional truth of an existing directional qualifier without touching its citing line.

**Preferred form:** Position-free reference: `(see §Foo)` or `(see §Foo, defined elsewhere in
this document)`. This form is immune to section reordering and requires no re-verification at
version bumps. Use it whenever the directional qualifier is not load-bearing for navigation
(i.e., when the reader can find the section via the heading name alone).

**When directional qualifiers are retained:** If a directional qualifier is retained for reader
convenience, the author MUST verify its accuracy at the time of writing by checking the target
section's line number against the citing line number.

**Anti-pattern:** Hard-coding `above` or `below` in a §Trace entry or cross-section reference
where future section insertions (especially §Trace growth) can silently invert the directional
truth — producing a reference that points the reader in the wrong direction without any
validation signal.

**Recurrence pattern that motivated this rule:** F-R48-cons-1 (v1.15) fixed `below` → `above`
in the §Trace v1.14 PG-2 entry (§Phantom-ID Convention was above §Trace, not below).
F-NEW-PG-1-direction (v1.16) fixed `below` → `above` in the §Trace v1.14 PG-1 entry
(§Schema-Fact Citation Convention was above §Trace, not below). Both were introduced together
in v1.14 as new §Trace entries; neither was verified against section positions at write time.
SS-engine-module.md v1.1.12 also corrected `below` → `above` for the audit table reference
(§Future audit maintenance paragraph is below the delimiter-bounded audit table block it describes).

**Version-bump self-check grep (mandatory before any version bump of any document containing
`above` or `below` directional qualifiers in cross-section references):**

```
grep -nE '\(see §[^)]*\b(above|below)\b[^)]*\)|\(§[^)]*\b(above|below)\b[^)]*\)' <file>
```

For each match: confirm the referenced section heading is genuinely above (lower line number)
or below (higher line number) the citing line. Fix any misdirection before committing the
version bump.

**Relationship to D-042:** D-042 targets version-citation staleness; PG-3 targets positional
qualifier staleness. Both are triggered by version bumps of any SS-*.md document. Run both
checks in the same pre-commit pass.

**All-prose L-number convention (expanded v1.18, round-49, F-R48-adv-2 root-cause):**
Current-state cross-doc L-number pinpoints are FORBIDDEN in ANY spec prose — not only
in §Trace entries. This generalizes the §Trace-prose sub-rule (added v1.17) to the full
document body. A "cross-doc L-number pinpoint" is any reference of the form "SS-foo.md
lines N-M", "SS-foo.md line N", "vision.md lines N-M", or "(lines N-M)" citing a current
position in another document, where that position is not prefixed with a version pin.

**Carve-outs (unchanged from v1.17):**
- Version-prefixed historical references (e.g., "in v1.14, the rule was at L904") are
  ACCEPTABLE provided the L-number was accurate at the cited version.
- Gene-source file references (e.g., "any-context-lazyclaude-pass-B-deep-hooks-r1.md
  lines 412–428") are ACCEPTABLE — gene-source files are read-only ingest artifacts that
  do not shift after ingestion.
- Code-block examples and anti-pattern fenced code blocks that use line numbers as
  illustrative values (not as navigational pointers) are ACCEPTABLE.
- Rust doc-comment code-specification blocks that define regex constants or similar
  specification content that cannot be expressed by name alone are ACCEPTABLE per the
  clause-4 exception in §Semgrep Coverage Hardening.

**Pre-commit grep — whole-file scan (primary, §Trace and main-body):**
```
grep -nE 'SS-[a-z-]+\.md (line|lines) [0-9]|domain-monocle-vision[^ ]* (line|lines) [0-9]|\(lines? [0-9]+[–-][0-9]+\)' <file>
```
Filter out gene-source file names (contain `-pass-` or `-synthesis`) and version-prefixed
matches (preceded by "in vX.Y.Z"). Any remaining match is a forbidden current-state
pinpoint and MUST be replaced with the referenced section's heading name.

**§Trace prose sub-rule (original v1.17 scope, preserved as special case):**
§Trace entries describing changes MUST use position-free references (section names) rather
than current-state L-numbers. Current-state L-numbers in §Trace prose are FORBIDDEN because
§Trace entries are written at version-bump time but read after subsequent edits shift line
numbers, making any current-state pinpoint a guaranteed staleness vector. Historical-state
L-numbers prefixed with their version (e.g., "in v1.14, the rule was at L904") are
ACCEPTABLE provided the L-number was accurate at the cited version. Pre-commit grep to
detect current-state L-number violations in §Trace prose specifically:

```
grep -A1 "^## §Trace" <file> | grep -A9999 "^## §Trace" | grep -E '\(L[0-9]+\)|paragraph at L[0-9]+|this file L[0-9]+|L[0-9]+-L[0-9]+'
```

Any match that lacks a version prefix (e.g., `in vX.Y.Z, L...`) is a forbidden current-state
pinpoint and MUST be replaced with the referenced section's heading name.

**META-rule application discipline (PG-3-TRACE-NEW-ENTRY, added v1.20, round-52.1):**
When applying or codifying any META rule (PG-1, PG-2, PG-3, PG-4, D-042), the §Trace entry
documenting the application MUST itself comply with ALL sibling META rules currently active in
the document. The §Trace prose describing a fix is subject to the same PG-1/PG-2/PG-3/PG-4
constraints as any other spec prose — META rules do not grant an exemption to the artifact that
records their application. Root-cause: the S-7.01 partial-fix irony pattern occurs when a META
rule is applied to existing prose while the §Trace documenting that application introduces a
sibling META violation (e.g., applying PG-4 to fix a §-heading mis-anchor while the §Trace
entry for that fix introduces a PG-3 bare L-number pinpoint).

**Post-write self-audit (mandatory after every §Trace entry authoring):** After writing a new
§Trace version entry, run the following grep on the NEWLY ADDED lines before committing:

```
grep -nE 'L[0-9]+' <newly-added-lines>
```

Any bare `L[0-9]+` token in §Trace prose is a candidate violation. Evaluate each match:
- If it is a version-prefixed historical reference (e.g., `in v1.17, the rule was at L904`):
  ACCEPTABLE — the version pin anchors the number to a past-state document.
- If it is a cross-doc current-state pinpoint (e.g., `SS-foo.md §Section rustdoc L487`):
  FORBIDDEN — drop the L-number; the section heading is sufficient for navigation.
- If it is a cross-doc or intra-doc positional L-number without version prefix:
  FORBIDDEN — replace with position-free section name.

This self-audit supplements (does not replace) the existing §Trace-prose sub-rule grep above.

## §Section-Anchor Citation Convention (PG-4)

Added in v1.19 (round-51.1, F-R51-adv-1 root-cause closure).

**Rule:** Cross-document `§<Name>` references MUST point to an actual `#`/`##`/`###`/`####`
heading in the cited document. Inline prose mentions of `<Name>` — bold labels
(`**Name:**`), paragraph prefixes, or any non-heading text — do NOT satisfy this convention.
Citations to non-heading content must use the closest enclosing actual heading plus a
position-free description of the content.

**Prefix-match policy:** A `§<Name>` citation is satisfiable by a heading whose text starts
with `<Name>` (e.g., `§Item P3-1` resolves to `#### Item P3-1: \`monocle-core\` trait
stability for WASM ABI`), provided the prefix uniquely identifies a single heading. Ambiguous
prefixes must be extended to distinguish.

**Anti-patterns:**

| Citation | Why it fails | Correct form |
|----------|-------------|--------------|
| `SS-permissions-phase1.md §Option A` | No heading "Option A" exists; prose at §Status and §Trace | `SS-permissions-phase1.md §Trace` |
| `SS-daemon-lifecycle.md §HookEventRecord` | No heading; struct defined in §Drain prose | `SS-daemon-lifecycle.md §Drain (HookEventRecord struct)` |
| `SS-engine-module.md §Future audit maintenance` | No heading; bold paragraph label in §Cross-Crate Constructor Audit | `SS-engine-module.md §Cross-Crate Constructor Audit (audit-maintenance paragraph)` |
| `SS-deps-pin-manifest.md §Phase 4 Additions` | No heading; actual heading is `### Phase 4 — Federation, MCP Bridge` | `SS-deps-pin-manifest.md §Phase 4 — Federation, MCP Bridge` |
| `SS-forward-compatibility.md §Analysis — Sealed trait` | No heading; bold label within `#### Item P3-1` | `SS-forward-compatibility.md §Item P3-1` |

**Exemption — meta-prose grep examples:** When a §-citation appears inside a code fence or
grep-pattern description as an illustrative example of what the pattern catches (not as a
navigational target), it is exempt from PG-4 enforcement. The string must be clearly within
a grep/code context. Example: the secondary D-042 grep description quotes
`"SS-daemon-lifecycle.md §HookEventRecord at v1.0.5"` as a sample match string — this is
meta-prose describing a pattern, not a navigation citation.

**Pre-commit grep (PG-4 sweep):**
```
grep -nE 'SS-[a-z-]+\.md §[A-Z][a-zA-Z0-9 -]+' <file>
```
For each match, open the cited SS-*.md file and verify the §-named portion corresponds to an
actual heading. Use:
```
grep -n "^#\|^##\|^###\|^####" <cited-file> | grep "<Name>"
```
If zero results: the citation is a mis-anchor — replace with the closest enclosing heading
and a position-free description. If one result: PASS. If multiple results: verify the prefix
uniquely identifies the intended one.

**Root-cause context:** F-R51-adv-1 (round-51) found that `SS-permissions-phase1.md §Option A`
appeared at 4 sites across 2 files after the R49 fix burst. The phrase "Option A" existed
only as inline prose in §Status ("human-directed per Q-A-permission-enum Option A") and as a
paragraph prefix in §Trace ("Option A. Gene-source: ... BC-HOOK-018"). No `##` or `###`
heading named "Option A" existed. This was missed by two prior CLEAN consistency audits and
one CLEAN adversary pass; only a fresh-context Semantic Anchoring Audit caught it. PG-4 closes
the class: any `§<Name>` must resolve to a real heading, not prose.

**Relationship to PG-1/PG-2/PG-3:** PG-4 is the fourth META-pattern class:
- PG-1: Schema-fact citations must include version pin (§Schema-Fact Citation Convention)
- PG-2: Narrative counts must match structural reality (§Schema-Fact Citation Convention, generalized PG-2 sub-rule)
- PG-3: Directional qualifiers above/below must be position-accurate (§Cross-Section Directional Reference Convention)
- PG-4: §<Name> citations must resolve to actual headings (this section)

## §Trace

v1.20 changes (round-52.1 F-R52-cons-1 PG-3 self-violation fix + PG-3-TRACE-NEW-ENTRY discipline):

- F-R52-cons-1 RESOLVED (LOW — PG-3 self-violation in R51.1 §Trace prose): §Trace v1.19
  item (3) referenced `SS-core-types-and-abi.md §FactoryAdapter Trait rustdoc L487` with a
  bare current-state L-number pinpoint. The `L487` token was dropped; the section heading
  `§FactoryAdapter Trait rustdoc` is sufficient for navigation per PG-3 §Trace-prose sub-rule.
  Root-cause irony: the R51.1 §Trace documented a PG-4 §-heading fix while itself introducing
  a PG-3 L-number violation — the S-7.01 partial-fix irony pattern.

- PG-3-TRACE-NEW-ENTRY discipline CODIFIED (META-rule application discipline): New sub-rule
  added to PG-3 §Cross-Section Directional Reference Convention. When applying or codifying
  any META rule, the §Trace entry documenting the application MUST comply with all sibling
  META rules. Post-write self-audit grep recipe (grep `L[0-9]+` on newly-added §Trace lines)
  specified. Prevents recurrence of the S-7.01 partial-fix irony pattern at the class level.

- R51.1 §Trace sweep (5 files): SS-core-types-and-abi.md v1.2.5 §Trace `at L487` (intra-doc
  bare L-number, dropped — bumped to v1.2.6); SS-forward-compatibility.md v1.2.6 §Trace
  `(L55)`, `(L57)`, `(L73)` (cross-doc positional L-numbers pointing to own body sections,
  replaced with position-free section-name descriptions — bumped to v1.2.7); dtu-assessment.md
  v1.4 §Trace: CLEAN (no bare L-numbers). D-042 cascade: dtu-assessment.md 3 body citations
  of SS-core-types-and-abi.md v1.2.5 updated to v1.2.6 (bumped to v1.5).

- SS-core-types-and-abi.md bumped to v1.2.6; SS-forward-compatibility.md bumped to v1.2.7;
  dtu-assessment.md bumped to v1.5 as co-edit partners for sweep fixes and D-042 cascade.

v1.19 changes (round-51.1 F-R51-adv-1 PG-4 §-heading-existence convention + comprehensive sweep):

- F-R51-adv-1 RESOLVED (MEDIUM — PG-4 §-heading-existence mis-anchor, two sites in this
  file): §Trace v1.18 F-R48-adv-3 entry cited `SS-permissions-phase1.md §Option A` at two
  locations. No heading "Option A" exists in SS-permissions-phase1.md; the text appears only
  as inline prose in §Status and as a paragraph prefix in §Trace. Both sites corrected to
  `SS-permissions-phase1.md §Trace` — the actual heading under which the BC-HOOK-018
  attestation and Q-A-permission-enum Option A resolution reside.

- PG-4 §Section-Anchor Citation Convention codified: cross-document `§<Name>` references
  must resolve to an actual `#`/`##`/`###`/`####` heading in the cited document. Inline prose
  mentions (bold labels, paragraph prefixes) do not satisfy. Pre-commit grep recipe added.
  Anti-pattern table with 5 confirmed historical mis-anchors. Prefix-match policy documented.
  Exemption for meta-prose grep examples documented.

- Comprehensive §-heading-existence sweep (PG-4 recipe, round-51.1): all cross-doc §-anchors
  verified across 8 architecture spec files + dtu-assessment.md. Additional mis-anchors found
  and corrected:
  (1) This file §Trace v1.17 PG-3 entry: `SS-engine-module.md §Future audit maintenance`
  corrected to `SS-engine-module.md §Cross-Crate Constructor Audit (audit-maintenance
  paragraph)` — `Future audit maintenance` is a bold paragraph label, not a heading.
  (2) SS-forward-compatibility.md §Item P4-3 body: `SS-deps-pin-manifest.md §Phase 4
  Additions` corrected to `SS-deps-pin-manifest.md §Phase 4 — Federation, MCP Bridge` —
  no heading "Phase 4 Additions" exists; actual heading is `### Phase 4 — Federation, MCP
  Bridge` (corrected in SS-forward-compatibility.md v1.2.6 companion edit).
  (3) SS-core-types-and-abi.md §FactoryAdapter Trait rustdoc: `§Analysis — Sealed
  trait §Item P3-1` corrected to `§Item P3-1` — `Analysis — Sealed trait` is a bold label
  within Item P3-1, not a heading (corrected in SS-core-types-and-abi.md v1.2.5 companion
  edit).
  (4) SS-engine-module.md §Trace v1.1.8 F-R28-4 entry: `SS-daemon-lifecycle.md
  §HookEventRecord` corrected to `§Drain (HookEventRecord struct)` (corrected in
  SS-engine-module.md v1.1.15 companion edit).

- SS-engine-module.md bumped to v1.1.15; SS-forward-compatibility.md bumped to v1.2.6
  (was v1.2.5 from prior burst); SS-core-types-and-abi.md bumped to v1.2.5 as co-edit
  partners for PG-4 sweep fixes.

v1.18 changes (round-49 F-R48-adv-1/2/3 root-cause fixes + PG-2 generalization + PG-3 all-prose expansion + D-042 scope codify):

- F-R48-adv-1 RESOLVED (LOW process-gap — PG-2 vocabulary gap, root-cause fix):
  §Test-Time Enforcement lead sentence at L51 read "All five mechanisms below" — the
  section now contains seven subsections (Clippy, Semgrep Rules, Semgrep Coverage Hardening,
  PR Template, Channel-Drop Test, CI Wiring, SBOM). Count corrected to "All seven mechanisms
  below"; reference also updated to position-free "§CI Wiring" (was "CI Wiring section").
  Root-cause fix: PG-2 META-pattern rule in this §Trace generalized from vocabulary-specific
  grep ("N rules", "N steps", "N entries") to noun-agnostic syntactic-shape grep covering
  "(All|These|The) <number-word> <noun>", "<number> <noun> (above|below)", and
  "<ordinal> <noun>" patterns. The grep recipe now catches "All five mechanisms",
  "All seven sections", "These three steps", "The fourth rule", etc. regardless of the
  noun used — closing the "mechanism" vocabulary gap exposed by this finding.
  Recurrence lineage: F-R44-adv-2 (v1.13) → F-R44-adv-3 (v1.13) → F-R46-3 (v1.14) →
  F-R48-adv-1 (v1.18). Each recurrence found the same root cause with a slightly different
  noun. The noun-agnostic recipe closes the class.

- F-R48-adv-2 RESOLVED (LOW process-gap — PG-3 scope expansion, root-cause fix):
  Cross-doc current-state L-number pinpoints found in main-body prose (outside §Trace):
  (1) §Test-Time Enforcement §Semgrep Rules YAML comment: "SS-deps-pin-manifest.md line 140"
  → "SS-deps-pin-manifest.md §Phase 1 vs Pinned-But-Unused Crates".
  Co-edited files (same L-number class, same PG-3 root cause):
  (2) SS-engine-module.md §Purpose: "SS-forward-compatibility.md lines 95–97" →
  "SS-forward-compatibility.md §Item P3-1 — Verdict on Sealed". (3) SS-engine-module.md
  §EngineModule Trait Signature signature block provenance prose: "lines 111–128" →
  position-free "§EngineModule". (4) SS-engine-module.md rustdoc comment for EngineModule
  trait: "(lines 95–97)" → "(§Item P3-1 — Verdict on Sealed)". (5) SS-engine-module.md
  §Trace v1.1 cross-reference block: two L-number pinpoints → position-free.
  (6) SS-core-types-and-abi.md §FactoryAdapter rustdoc and §BC-FACTORY-001 Traceability:
  "lines 95–97" → "§Item P3-1 — Verdict on Sealed" (two sites).
  Root-cause fix: PG-3 §Trace-prose sub-rule (v1.17) expanded to all-prose scope. The
  previous rule only covered §Trace entries; main-body prose carried the same staleness
  risk. All-prose PG-3 now covers §Trace + §Purpose + §Section Content + rustdoc comments
  + any other prose, with identical carve-outs (version-prefixed historical, gene-source
  files, code-block examples, regex constant definitions).

- F-R48-adv-3 RESOLVED (LOW — phantom-ID prevention, Option A applied uniformly):
  Inline code comment in SS-engine-module.md §Phase 1 Implementation (on_hook method) cited
  "BC-HOOK-018"
  without gene-source qualifier. BC-HOOK-018 IS attested (SS-permissions-phase1.md §Trace:
  gene-source BC-HOOK-007/BC-HOOK-018), but the prevention grep could not chain-resolve
  across files. Option A chosen: added explicit two-line gene-source qualifier
  "(gene-source: any-context-lazyclaude; attested in SS-permissions-phase1.md §Trace)"
  adjacent to the citation. Option B (soften grep to cross-file chain-resolution) rejected:
  adds grep complexity without marginal benefit; the line-grep is correct for unattested
  citations. Uniform application: BC-HOOK-018 is the only inline BC-HOOK-NNN citation in
  SS-engine-module.md; no sibling sites required treatment. Sweep of all 8 architecture
  spec files confirmed no other inline BC-HOOK-NNN citations outside their attestation source.
  [§Option A corrected to §Trace in v1.19 per F-R51-adv-1 PG-4 sweep — no §Option A heading
  in SS-permissions-phase1.md; BC-HOOK-018 attestation resides in its §Trace heading.]

- PG-D042-BURST-SKIP CLOSURE (7th recurrence root-cause): D-042 grep scope explicitly
  codified in §Schema-Fact Citation Convention D-042 integration block (this file) as
  `.factory/specs/` recursive — NOT `.factory/specs/architecture/` only. The scope error
  was documented in SS-forward-compatibility.md §Trace v1.2.3 (round-43) but not propagated
  to this file, allowing the burst-skip pattern to recur in subsequent rounds. Correct D-042
  primary grep (`grep -rn "SS-[a-z-]*\.md v" .factory/specs/`) and secondary grep now
  embedded in §Schema-Fact Citation Convention for pre-commit use. The "architecture/"
  restriction gave false confidence that all citation sites had been swept.

- SS-engine-module.md bumped to v1.1.14; SS-core-types-and-abi.md bumped to v1.2.4 as
  co-edit partners for F-R48-adv-2 L-number fixes.

v1.17 changes (round-47.3 F-R48R-1 LOW + F-R48R-2 LOW + PG-3 §Trace-prose sub-rule):

- F-R48R-1 RESOLVED (LOW — stale intra-document §Trace L-number pinpoint): §Trace v1.15
  entry for F-R48-cons-1 described §Phantom-ID Convention as above §Trace with L-numbers.
  PG-3 insertion in v1.16 shifted §Trace from its v1.14 position, making the §Trace (L882)
  L-number stale (§Trace is now at a higher line number). Applied PG-3 preferred-form:
  dropped both L-numbers from the directional assertion, leaving position-free `§Phantom-ID
  Convention is above §Trace`. No content change; navigation aid accuracy fix.

- F-R48R-2 RESOLVED (LOW — stale cross-document §Trace L-number pinpoints): §Trace v1.16
  entries referenced SS-engine-module.md §Future audit maintenance by `L1141` in two
  locations: (a) v1.16 co-edit bullet and (b) PG-3 pre-commit self-check summary line.
  The actual paragraph (§Future audit maintenance) is at L1137 in SS-engine-module.md —
  the L1141 reference was overcounted by 4. Also fixed the HTML-delimited table range
  `(L1108-L1128)` in the same v1.16 bullet to a section-name reference. Applied PG-3
  preferred-form: all three current-state L-numbers replaced with stable section names
  (§Future audit maintenance; §Cross-Crate Constructor Audit). Additionally fixed stale
  L-numbers in v1.16 (PG-1 entry `L932` → position-free; v1.14 §Trace PG-3 sweep summary
  `this file L932; SS-engine-module.md L1141` → section-name references) and in v1.14
  (R-001 trigger `Line 641` → role description; SS-forward-compatibility.md `L235-252` →
  `§Cross-Phase Decisions table`).

- PG-3 §Trace-prose authoring sub-rule CODIFIED: §Cross-Section Directional Reference
  Convention extended with an explicit §Trace prose authoring sub-rule (added in PG-3
  rule body above §Trace heading). Rule summary: current-state L-numbers in §Trace prose
  are FORBIDDEN; only version-prefixed historical L-numbers are acceptable. Pre-commit
  grep provided. This codifies the M2 META-pattern observed in R47.2: §Trace entries
  written with current-state L-number pinpoints are guaranteed staleness vectors because
  any subsequent section insertion shifts them without touching the §Trace line.

- Comprehensive §Trace sweep across 8 target spec files completed (scope: all
  R47/R47.1/R47.2/R47.3-touched files). Current-state L-number drifts found and
  corrected: 5 in SS-conventions-anti-patterns.md (this file); 2 in SS-engine-module.md
  (§Trace v1.1.12 paragraph-at and delimiter-block range). Zero L-number drifts found
  in SS-forward-compatibility.md, SS-core-types-and-abi.md, SS-daemon-lifecycle.md,
  SS-permissions-phase1.md, SS-deps-pin-manifest.md, dtu-assessment.md.

- SS-engine-module.md bumped to v1.1.13 with §Trace entry recording the position-free
  conversion of its v1.1.12 §Trace L-numbers.

v1.16 changes (round-48 F-NEW-PG-1-direction LOW + PG-3 codification):

- F-NEW-PG-1-direction RESOLVED (LOW — directional typo, sibling to F-R48-cons-1): §Trace v1.14
  PG-1 entry read "Convention rule added (see §Schema-Fact Citation Convention below)."
  §Schema-Fact Citation Convention is above §Trace, not below — same class of bug as F-R48-cons-1
  (v1.15). Corrected "below" to "above". No content change; navigational accuracy fix only.
  Root cause: both F-R48-cons-1 and F-NEW-PG-1-direction were introduced together in the v1.14
  §Trace block without positional verification at write time.

- PG-3 RESOLVED (META-pattern codification — third META class this cycle): Two consecutive
  directional typos (F-R48-cons-1 in v1.15 and F-NEW-PG-1-direction in v1.16) with identical
  root cause — §Trace entries written with directional qualifiers that were not verified against
  actual section positions — constitute a META-pattern class requiring a codified prevention rule.
  §Cross-Section Directional Reference Convention added (see §Cross-Section Directional Reference
  Convention above). Rule summary: (1) preferred form is position-free `(see §Foo)` without
  directional qualifier; (2) when directional qualifiers are retained, MUST be verified at write
  time and re-verified at every version bump of the citing document; (3) pre-commit self-check
  grep provided. This is the third META-pattern class codified this cycle after PG-1
  (schema-fact citations) and PG-2 (step-renumbering events).

- SS-engine-module.md v1.1.12 co-edited: §Future audit maintenance paragraph read
  "audit table rows below" — the HTML-delimited §Cross-Crate Constructor Audit table block is
  above that paragraph. Corrected "below" to "above". Version bumped to v1.1.12 with §Trace
  entry.

- PG-3 pre-commit self-check grep applied to all 8 target files in the R47.2 sweep scope;
  2 misdirections found and corrected (this file §Trace v1.14 PG-1 entry;
  SS-engine-module.md §Cross-Crate Constructor Audit audit-maintenance paragraph); all other
  directional qualifiers verified accurate.

v1.15 changes (round-48 fix F-R48-cons-1 LOW):

- F-R48-cons-1 RESOLVED (LOW — consistency audit finding, single-word corrective edit):
  §Trace v1.14 L904 read "Prevention rule added in §Phantom-ID Convention below." §Phantom-ID
  Convention is above §Trace, not below. A reader following the cross-reference
  forward would scan past §Trace and not find the referenced section. Corrected "below" to
  "above". No content change; navigational accuracy fix only.

v1.14 changes (round-47 fixes F-R46-2 MEDIUM / F-R46-3 LOW / PG-1 / PG-2):

- F-R46-2 RESOLVED (MEDIUM — adversary finding, Option C chosen): The R-001 trigger
  condition (a) row referenced "monocle's hook-event pipeline (BC-HOOK-001–006)" — these monocle
  BC IDs do not exist. The 16 pre-staged BC IDs in the SS-forward-compatibility.md
  §Cross-Phase Decisions Required table contain
  no BC-HOOK-NNN entries; the gene-source any-context repo uses BC-HOOK-001..BC-HOOK-041 but
  those are gene-source IDs, not monocle IDs. "BC-HOOK-001–006" is an unattested phantom range.
  Option C chosen over A (forward-reference disclaimer) and B (remove parenthetical entirely):
  - Option A rejected: "to-be-authored in Phase 1 PRD" language creates a vague forward
    reference that won't match any specific BC IDs once the PRD authors them; the R-001
    trigger rationale becomes stale again as soon as Phase 1 IDs are assigned.
  - Option B rejected: removing the parenthetical reduces traceability — the trigger condition
    provides no observable hook-pipeline anchor for the weekly monitoring workflow.
  - Option C implemented: replaced "(BC-HOOK-001–006)" with "(gene-source canonical 5-hook
    matrix: BC-HOOK-007)". BC-HOOK-007 IS attested: it is cited in dtu-assessment.md and
    SS-core-types-and-abi.md as the gene-source canonical hook-endpoint-body matrix. It is an
    external observable fact (the any-context gene-source canonical reference) that does not
    depend on monocle Phase 1 PRD numbering finalization. The R-001 trigger rationale remains
    resolvable for both the weekly monitoring workflow and future readers.
  META-pattern (phantom forward-reference — new class): a BC ID range referenced as if anchored
  but unattested in any current artifact. Prevention rule added in §Phantom-ID Convention above.

- F-R46-3 RESOLVED (LOW — adversary finding, Option B chosen, intent: NOT historical
  preservation): §Trace v1.7 entry for F-R32-4 at L1069 stated "a 'Contract edge cases'
  paragraph added after Step 3's step 6." After F-R44-adv-1 (v1.13) inserted a new step 2
  into the Python script, renumbering subsequent steps from 1-6 to 1-7, the "step 6"
  pinpoint became an off-by-one (current location is after step 7). The §Trace entry is
  operationally consulted by current-state readers (they read the §Trace to understand where
  in the spec an item lives); a stale step number sends readers to the wrong location.
  Option B chosen over A (update to "step 7") and C (declare historical preservation):
  - Option A (update to "step 7") rejected: updates the count to match the current-state
    but remains fragile — any future step insertion will produce the same off-by-one. The
    root cause is position-coupling in §Trace prose, not the specific number.
  - Option C (historical preservation) rejected: the §Trace entry is NOT historical archival
    of a time-locked state; it is an active navigational aid. "Historical preservation" applies
    to rationale for a decision made at a past point in time — not to a pointer to where in
    the current spec to find a section. A position pointer must be current-state-accurate.
  - Option B implemented: "after Step 3's step 6" reworded to "after the final step of the
    Step 3 script description" — position-free, resilient to future step insertions.
  META-pattern (step-renumbering sub-class — PG-2): covered in updated META-pattern rule below.

- PG-1 RESOLVED (D-042 grep workflow scope extension): The D-042 workflow documented in
  SS-forward-compatibility.md v1.2.3 §Trace targets SS-*.md version-citation drift
  (primary: `grep -rn "SS-[a-z-]*\.md v" .factory/specs/`) but does NOT catch cross-doc
  factual claims about content matrices (e.g., "field X present in all N hook body schemas").
  F-R46-1 root cause: a sentence in SS-forward-compatibility.md asserted session_id was
  present in all 5 hook body schemas citing the DTU endpoint matrix — but the DTU matrix
  showed only gene-source fields, which lack session_id on 2 of 5 endpoints.
  Convention rule added (see §Schema-Fact Citation Convention above):
  - Schema-fact citations across docs MUST cite a single canonical anchor document AND cite
    the specific column/table name within that anchor.
  - Any schema-fact assertion of the form "field X in all N schemas" MUST include a grep
    pattern in §Trace that re-validates the fact against the canonical anchor at any version
    bump of either the citing or cited document.
  - The grep validation pattern for the session_id fact is now embedded in both
    dtu-assessment.md v1.3 §Trace and SS-forward-compatibility.md §Trace §D-042 WORKFLOW RULE.
    D-042 workflow rule extended: before any version bump of dtu-assessment.md or
    SS-core-types-and-abi.md, run:
    ```
    grep -rn "session_id.*all.*hook\|all.*hook.*session_id\|present in all 5\|in all 5 hook" .factory/specs/
    ```
    to enumerate all cross-doc factual claims referencing hook body field presence.

- PG-2 RESOLVED (META-pattern rule scope extension): The F-R44-adv-2 META-pattern rule
  (introduced v1.13) stated "Every rule addition, removal, or reordering event MUST include
  a proactive grep for 'N rules', 'Nth rule', 'N steps', 'Nth step'." This did not explicitly
  cover step-renumbering in procedural specs — F-R44-adv-1 (v1.13) inserted a Python script
  step, renumbering steps 1-6 to 1-7, but the META-pattern grep ("step N") was not applied
  to §Trace pinpoints. F-R46-3 was the result. Fix: META-pattern rule above expanded to
  include "step-renumbering events in any procedural spec" and "step N" variant in the grep
  list. Step-renumbering events are now an explicit trigger for the proactive grep.

v1.13 changes (round-45 fixes F-R44-adv-1 HIGH / F-R44-adv-2 MEDIUM / F-R44-adv-3 MEDIUM / F-R44-adv-4 LOW):

- F-R44-adv-1 RESOLVED (HIGH — adversary finding, Option B chosen): The
  `monocle-non-exhaustive-struct-audit-completeness` rule's `paths.include` listed 12
  production source paths (11 named crates + binary crate) but excluded `semgrep-fixtures/`
  — no glob matched it. Step 1 (fixture corpus scan) invokes semgrep against
  `semgrep-fixtures/` only; the rule's `paths.include` then rejected all fixture files,
  producing 0 findings against an expected count of 2. This caused guaranteed CI failure on
  every run from day one. Root cause: F-R34-3 (v1.8) expanded `paths.include` for
  production-scope correctness but was not cross-checked against the Step 1 invocation
  pattern introduced by F-R26-adv-3/POL-11 (v1.5). Two defense layers with a silent
  compatibility gap. Option B chosen over Option A (CLI --include override) and Option C
  (separate fixture-only rule):
    - Option A rejected: reintroduces the CLI --include override pattern that F-R40-1 (v1.11)
      explicitly removed; contradicts F-R40-1 reasoning.
    - Option C rejected: a separate fixture-only rule adds maintenance overhead (two rules to
      keep in sync per arm change) with no material benefit over Option B.
    - Option B implemented (three parts):
      (1) Added `semgrep-fixtures/**/*.rs` to `paths.include` with inline rationale comment
          explaining the F-R44-adv-1 compatibility constraint.
      (2) Added fixture-struct-name exclusion to Step 2 special-case prose: documents that
          `AuditFixtureMinimal` and `AuditFixtureDerived` are present in the semgrep output
          due to the fixture path inclusion; if Step 2 ever adds a zero-findings assertion for
          this rule, the CI script MUST filter these two names first.
      (3) Added fixture-struct-name exclusion as a normative Step 3 requirement: step 2 of the
          Python script description (renumbered; all subsequent steps incremented by 1) now
          specifies `FIXTURE_STRUCT_NAMES = {"AuditFixtureMinimal", "AuditFixtureDerived"}` as
          a named constant; the script MUST remove these names from the semgrep output set before
          computing the set difference against the audit table. Exclusion is by exact name (not
          by path) so it remains correct if the fixture file is moved. The named-constant
          requirement ensures future fixture additions are a single-location update.

- F-R44-adv-2 RESOLVED (MEDIUM — adversary finding): Lines 68-69 read "All four rules below
  are authoritative; the fourth rule (no-raw-env-mutation-in-tests) was added in v1.5."
  Current state is 5 rules (the fifth — `monocle-non-exhaustive-struct-audit-completeness`
  — was added in v1.6 per F-R30-3). Narrative count was not updated at v1.6 add-time, nor at
  any subsequent version. Fix: "All five rules below are authoritative; the fifth rule
  (`monocle-non-exhaustive-struct-audit-completeness`) was added in v1.6." The fourth rule
  reference is preserved as a secondary clause so both add-events remain traceable. META-pattern:
  narrative wrapper counts ("All N rules", "the Nth rule") are a distinct propagation layer from
  content counts (rule YAML entries). They must be explicitly audited on every rule/step add
  or remove event. This is now documented as a §Trace requirement: future rule additions must
  update BOTH the YAML content AND the narrative wrapper count in the same commit.

- F-R44-adv-3 RESOLVED (MEDIUM — adversary finding): Two stale step counts: (1) heading
  "CI assertions (two steps)" should be "three steps" — Steps 1, 2, and 3 have existed since
  F-R30-3/v1.6; the heading was never updated past "two steps." (2) prose "All four steps run
  after cargo clippy" should be "All three steps" — this refers to the three semgrep CI steps,
  not the 6-entry CI Wiring list. The "four" count had no basis in any version of the spec.
  Both narrative count errors follow the same root cause as F-R44-adv-2: narrative wrapper
  counts were not audited when the content they describe changed. Fix: heading updated to
  "three steps"; prose updated to "All three steps."

- F-R44-adv-4 AUTO-RESOLVED (LOW): Line 800 reads "Rule monocle-no-raw-env-mutation-in-tests
  is the 4th semgrep rule." This is numerically correct (rule 4 of 5); the reference did not
  require a change after F-R44-adv-2. Line 1062 "the 4th rule" is also correct for the same
  reason. Both confirmed clean post-fix.

- META-pattern (narrative wrapper count discipline — GENERALIZED v1.18, round-49):
  Any addition, removal, reordering, or renumbering of ANY countable item — rules, steps,
  mechanisms, sections, entries, checks, arms, criteria, or ANY other noun — MUST include
  a proactive noun-agnostic grep for narrative count wrappers across the file before
  declaring done. This is not optional — narrative wrapper counts are a separate propagation
  layer that semgrep and table validators do not cover.

  **PG-2 noun-agnostic grep recipe (run before every version bump or item-count change):**
  ```
  grep -nEi '(All|These|The|all)\s+(one|two|three|four|five|six|seven|eight|nine|ten|[0-9]+)\s+\w+|(one|two|three|four|five|six|seven|eight|nine|ten|[0-9]+)\s+\w+\s+(above|below)|(first|second|third|fourth|fifth|sixth|seventh|eighth|ninth|tenth)\s+\w+' <file>
  ```
  For each match: confirm the count or ordinal is still accurate against the current item
  list. Fix any mismatch before committing. The grep is syntactic-shape-based (not
  vocabulary-limited); it catches "All five mechanisms", "All seven sections", "These
  three steps", "The fourth rule", "six entries above", and all sibling variants regardless
  of noun.

  **Covered event classes (expanded from v1.13 → v1.14 → v1.18):**
  - Rule addition/removal/reordering (original F-R44-adv-2 scope)
  - Step-renumbering in any procedural spec (added round-47, F-R46-3)
  - Section/mechanism addition in any structured list (added round-49, F-R48-adv-1 root-cause)
  The S-7.01 Partial-Fix Regression Discipline explicitly includes this layer. Step-renumbering
  failures produce off-by-one pinpoints in §Trace entries; section-count drift produces
  stale lead sentences (e.g., "All five mechanisms" when seven exist). Both follow the
  same root cause: a human-readable count in narrative prose that is not verified against
  the actual item list at write time.

  PG-2 extension lineage: F-R44-adv-2 (round-45) introduced the rule for "N rules"; F-R46-3
  (round-47) extended to step-renumbering; F-R48-adv-1 (round-49) generalized to
  noun-agnostic syntactic shape after "All five mechanisms" drifted when the §Test-Time
  Enforcement section grew to seven subsections.

v1.12 changes (round-43 fix F-R42-adv-1 MEDIUM — S-7.01 propagation):

- F-R42-adv-1 RESOLVED (MEDIUM process-gap — adversary finding): The F-R32-2 fix (v1.7) applied
  dual-shape fixture corpus discipline to `monocle-non-exhaustive-struct-audit-completeness` —
  mandatory all-arm coverage, expected count computed from arm count, CI failure on mismatch.
  That discipline was NOT propagated to 3 sibling rules at the same fix point, violating
  Partial-Fix Regression Discipline (S-7.01): when a convention is introduced or strengthened,
  ALL existing occurrences of the gap in the same file must be remediated in the same edit.
  The 3 sibling rules carried the same POL-11 partial-arm-coverage gap:
  (1) `monocle-no-shell-injection` (2 `pattern-either` arms: `Command::new("sh")` and
  `Command::new("bash")`): fixture previously exercised only the first arm. If the
  `Command::new("bash")` arm's pattern syntax broke (semgrep version regression, YAML typo),
  CI would emit PASS because the `sh` arm still matched — the broken arm is unverified.
  (2) `monocle-no-naked-fs-write` (2 arms: `std::fs::write` and `tokio::fs::write`): fixture
  previously exercised only `std::fs::write`. The `tokio::fs::write` arm was unverified.
  (3) `monocle-no-raw-env-mutation-in-tests` (4 arms: `std::env::set_var`, `std::env::remove_var`,
  `env::set_var`, `env::remove_var`): fixture exercised 2 of 4. Step 1 CI assertion used "implicitly
  covered" language and "may add them to make explicit" — optional, not normative. The remaining
  2 arms were unverified; a pattern regression on either would go undetected.
  Fix (three parts, applied per S-7.01 all-at-once discipline):
  (1) Fixture corpus table updated with all-arm requirements: each of the 3 affected rules now lists
  ALL arms with MUST language, and expected counts computed from arm counts (shell_injection=2,
  naked_fs_write=2, raw_env_mutation=4). `monocle-non-exhaustive-struct-audit-completeness`
  unchanged (already dual-shape compliant from F-R32-2). `monocle-no-unbounded-channel` unchanged
  (single-pattern rule, no `pattern-either`; expected count remains 1).
  (2) Step 1 CI assertion language replaced: "implicitly covered" and "may" optional language
  removed. Normative MUST language added: each rule's fixture MUST contain at least one violation
  per `pattern-either` arm; expected counts table added; CI MUST fail with explicit arm-identifying
  error message on count mismatch.
  (3) CI sanity check added: the CI script MUST assert that declared expected count <= actual
  fixture finding count, preventing future arm additions from regressing to partial coverage without
  a corresponding fixture update. New arm + updated fixture + incremented expected count must ship
  in the same PR; the sanity check catches the omission on the next fixture corpus run.

v1.11 changes (round-41 fix F-R40-1 MEDIUM):

- F-R40-1 RESOLVED (MEDIUM — adversary finding): The Step 3 audit-table gap check CLI
  invocation used `--include "monocle-*/src/**/*.rs"` as a CLI-level file filter. This
  glob requires a hyphen after `monocle` (the `*` in `monocle-*` matches the crate-name
  suffix, not an empty string in standard semgrep/shell glob semantics). The binary crate
  is named `monocle` (no hyphen), so its source at `monocle/src/**/*.rs` does NOT match
  `monocle-*/src/**/*.rs`. The CLI `--include` is applied by the semgrep runner before
  rule-level `paths.include` is evaluated — structs defined in `monocle/src/**/*.rs` are
  never read, making the rule's `paths.include` entry for the binary crate a dead letter.
  The audit-completeness gap has zero CI signal: the Python script receives no semgrep
  findings from the binary crate, so it cannot report a table gap for structs defined there.
  Fix chosen: Option A — remove `--include` from the CLI invocation entirely. The rule's
  `paths.include` (expanded to all 12 workspace paths in F-R34-3/v1.8) is already the
  authoritative and complete scope governor. CLI `--include` was redundant and incorrect.
  Option B (glob `monocle*/src/**/*.rs`) was rejected as relying on glob semantics where `*`
  matches empty string — correct in Python fnmatch but not universally guaranteed across
  semgrep runner versions. Option C (dual `--include` flags) was rejected as a maintenance
  burden requiring update whenever a new crate is added. After fix, the runner reads all
  Rust source files in the workspace and the rule's `paths.include` governs which files the
  rule fires on — as intended by F-R34-3.

v1.10 changes (round-39 fix F-R38-1 MEDIUM):

- F-R38-1 RESOLVED (MEDIUM — adversary finding, Option B chosen): The adversary flagged that
  §Trace v1.8 lines containing `BEGIN_DELIMITER_REGEX = r'^...$'` and `END_DELIMITER_REGEX =
  r'^...$'` in the F-R34-1 narrative are borderline under the clause 4 no-verbatim-quoting
  Convention. Option B selected over Option A on production-grade grounds: the regex constant
  definitions in §Trace v1.8 cannot be expressed by name alone — the delimiter pattern string
  IS the specification content that clause 4 added. A reader of §Trace v1.8 must be able to
  see the full constant assignment (pattern value included) without round-tripping to clause 4;
  eliminating it reduces spec readability without improving correctness. Option A (rewrite §Trace
  to refer to constants by name only) would remove information. Option B (amend the convention
  rule with a narrowly scoped exception) preserves readability and enforces correctness.
  Fix: clause 4 Convention rule updated with explicit exception: regex constant definition
  blocks in §Trace MAY include the full assignment expression; all other narrative prose
  remains subject to the no-verbatim-quoting rule without exception.

v1.9 changes (round-37 fix F-R36-2 MEDIUM):

- F-R36-2 RESOLVED (MEDIUM — adversary finding, Partial-Fix Regression Discipline S-7.01): The
  v1.8 §Trace entry for F-R30-3 (v1.6 changes) quoted the audit-table delimiter markers verbatim
  in its §Trace prose — a direct self-violation of the no-verbatim-quoting convention introduced
  in the same v1.8 edit (clause 4, §Semgrep Coverage Hardening Layer 2). Partial-Fix Regression
  Discipline (S-7.01): when a convention rule is introduced, ALL existing occurrences of the
  prohibited pattern in the same file must be remediated in the same edit. The v1.8 fix correctly
  patched SS-engine-module.md §Trace but missed the parallel occurrence at v1.6 §Trace of this
  file. Fix: v1.6 §Trace point (1) rewritten to refer to the audit-table delimiter markers by
  name (referencing BEGIN_DELIMITER_REGEX and END_DELIMITER_REGEX in clause 4 of §Semgrep
  Coverage Hardening) without verbatim HTML comment quoting. Historical narrative meaning
  preserved: the boundary between where the markers live (SS-engine-module.md) and how they work
  (CI machine-parsing) remains clear.

v1.8 changes (round-35 fixes F-R34-1 CRITICAL / F-R34-2 IMPORTANT / F-R34-3 IMPORTANT):

- F-R34-1 RESOLVED (CRITICAL — adversary finding): The `check_audit_table.py` duplicate-delimiter
  detection in clause 4 of §Contract edge cases was specified as a substring search ("if the
  delimiter appears more than once"). A substring search would count occurrences in §Trace prose,
  backtick-quoted examples, or mid-line narrative text — exactly the scenario described in clause 4's
  own parenthetical "(e.g., because a §Trace example embedded the delimiter text)." This creates a
  self-DoS: if any §Trace entry quotes the delimiter string verbatim (as SS-engine-module.md
  §Trace v1.1.9 did at the adversary-reported lines 1183–1184), the Python script would detect a
  "duplicate" and exit 1 on first run, making Phase 1 CI permanently broken until a human noticed
  the spec prose was the cause. Fix applied in two parts (defense in depth):
  (a) Clause 4 now specifies line-anchored regex for all delimiter detection:
  `BEGIN_DELIMITER_REGEX = r'^<!-- BEGIN: Cross-Crate Constructor Audit Table -->$'` and
  `END_DELIMITER_REGEX = r'^<!-- END: Cross-Crate Constructor Audit Table -->$'`. Line-anchoring
  excludes backtick-wrapped mentions (non-full-line content), indented prose (leading whitespace
  breaks `^`), and mid-line occurrences. The Step 3 script description (steps 2–3) updated to
  reference the regex by name for consistency. All prior "find the delimiter" prose in §Step 3
  now refers to the line-anchored regex, ensuring the devops-engineer cannot implement a plain
  `str.find()` or `in` check for delimiter detection.
  (b) Clause 4 now includes a **Convention** rule: do NOT quote the audit-table delimiter strings
  verbatim in §Trace prose or any spec narrative. Refer to them by name. This prevents the
  regression even if the line-anchored regex is loosened in a future edit.
  Companion fix: SS-engine-module.md v1.1.10 §Trace prose at the adversary-reported lines rewritten
  to refer to the delimiter markers by name without verbatim quoting.

- F-R34-2 RESOLVED (IMPORTANT — adversary finding): The `pattern-either` second arm (Shape B) for
  rule `monocle-non-exhaustive-struct-audit-completeness` used `#[...]` as the intermediate-attribute
  wildcard. `#[...]` is NOT a documented semgrep wildcard form — its matching behavior is undefined
  and it may match nothing, silently breaking the Shape B arm. Standard semgrep form for "any
  attribute with a parenthesized argument list" is `#[$ATTR(...)]` where `$ATTR` is a named
  metavariable matching any identifier and `(...)` matches any argument list. Verification:
  `#[$ATTR(...)]` matches `#[derive(Debug, Clone)]` (`$ATTR=derive`, `(...)=(Debug, Clone)`),
  `#[derive(Debug, Clone, Serialize)]` (multi-arg; `(...)` is not restricted to single arguments),
  `#[serde(rename_all = "snake_case")]`, and `#[repr(C)]`. It does NOT match bare attributes with
  no parentheses (e.g., a hypothetical `#[copy]`). All monocle production structs use parenthesized
  intermediate attributes; no bare intermediate attributes exist in the codebase. If a production
  struct acquires a bare intermediate attribute, a third `pattern-either` arm must be added — the
  dual-shape fixture corpus (expected count 2) will catch the gap as a CI failure.
  Fix: `#[...]` replaced with `#[$ATTR(...)]` in both the §Semgrep Rules YAML block and the
  §Semgrep Coverage Hardening `pattern-either` reference block. Rule rationale comment expanded
  with documentation of `$ATTR`/`(...)` semantics and the bare-attribute limitation.

- F-R34-3 RESOLVED (IMPORTANT — adversary finding): The `monocle-non-exhaustive-struct-audit-completeness`
  semgrep rule `paths.include` listed only 4 of the 11 Phase 1 workspace crates (`monocle-core`,
  `monocle-runtime`, `monocle-tui`, `monocle-proto`). Per SS-deps-pin-manifest.md §Workspace
  Dependency Graph, the Phase 1 workspace has 11 named crates + 1 binary = 12 total. Any
  `#[non_exhaustive]` struct added to the 7 uncovered crates (`monocle-ipc`, `monocle-config`,
  `monocle-plugin-sdk`, `monocle-workflow`, `monocle-static`, `monocle-fuzz`,
  `monocle-test-harness`) or the binary crate (`monocle`) would be silently missed by the semgrep
  rule, creating an audit-completeness gap without any CI signal. The audit-completeness check
  (Step 3 Python script) only fires on structs that semgrep finds — structs in uncovered crates
  are invisible to the script. Fix: `paths.include` expanded from 4 to 12 paths covering all 11
  named crates and the `monocle/src/**` binary crate. Each path documented with its source
  (SS-deps-pin-manifest.md workspace graph). Audit-completeness rationale added as inline YAML
  comment explaining why full-workspace scope is required.

v1.7 changes (round-33 fixes F-R32-2 MEDIUM / F-R32-4 LOW):

- F-R32-2 RESOLVED (MEDIUM process-gap — adversary finding): The semgrep fixture corpus for
  `monocle-non-exhaustive-struct-audit-completeness` contained a single fixture struct
  (`#[non_exhaustive] pub struct AuditFixtureStruct { ... }`) with no `#[derive(...)]` attribute
  between `#[non_exhaustive]` and `pub struct`. Every actual monocle production struct has
  `#[derive(Debug, Clone)]` (or similar) interposed between these two. If semgrep's Rust tree-sitter
  pattern matching is position-sensitive on attribute clusters — i.e., `#[non_exhaustive]\npub
  struct $NAME { ... }` matches only a struct whose first outer attribute is `#[non_exhaustive]`
  with no intervening attributes — then the rule matches the minimal fixture (confirming the rule
  is functional) but fails to match any production struct (because production structs have an
  intermediate `#[derive(...)]` attribute). The Step 3 Python script then receives an empty semgrep
  JSON output and trivially exits 0: no struct names found, no table gap possible. This is the
  POL-11 false-green pattern identical to the prism PR #127 failure mode (a rule appears to work
  in CI because the fixture passes, but never fires on production code). Fix (three parts):
  (1) Fixture corpus updated to require two fixture structs in `semgrep-fixtures/non_exhaustive_struct.rs`:
  Shape A (minimal — no intermediate attribute, matches `AuditFixtureMinimal`) and Shape B
  (production-code shape — `#[derive(Debug, Clone)]` interposed, matches `AuditFixtureDerived`).
  Expected Step 1 match count updated from 1 to 2. If CI reports 1 instead of 2, the rule's
  second arm fails on the production-code attribute shape — this is a blocking CI failure.
  (2) The semgrep rule pattern updated from a single `pattern: |` block to a `pattern-either` with
  two arms — one for the minimal shape, one for the production-code shape with `#[...]` as a
  wildcard intermediate attribute. This makes the rule correct regardless of whether semgrep's
  attribute-cluster matching is strict or liberal. Rationale is documented inline in the rule YAML
  comment.
  (3) The fixture corpus rationale note added after the table explains WHY dual-shape is required
  (F-R32-2 META-GAP rationale + POL-11 production-shape requirement), enabling future readers to
  understand the non-obvious design decision.

- F-R32-4 RESOLVED (LOW process-gap — adversary finding): The `check_audit_table.py` Step 3
  contract (§Step 3 — Audit-table gap check) lacked specification for five edge cases that would
  leave the script with undefined/implementer-choice behavior: header/separator row skipping,
  missing spec file, malformed delimiter pairs (BEGIN without END, END without BEGIN), duplicate
  delimiters, and empty table. In each case a naive implementation could silently succeed with
  exit 0 despite the spec being in a broken state — a false-green identical to the POL-11 pattern.
  Fix: a "Contract edge cases" paragraph added after the final step of the Step 3 script description, specifying all five cases
  with exact exit codes, exact error message formats, and production-grade defaults (empty table =
  fail, not warn; missing file = fail; duplicate delimiters = fail with disambiguation). All five
  cases are normative requirements for the devops-engineer Phase 1 deliverable.

v1.6 changes (round-30 fix F-R30-3 MEDIUM):
- F-R30-3 RESOLVED (MEDIUM process-gap — adversary finding): The §Cross-Crate Constructor
  Audit table invariant in SS-engine-module.md was a passive policy with no machine enforcement.
  F-R30-1 demonstrated the policy was violated — 10 of 17 `#[non_exhaustive]` structs were
  missing while the invariant statement claimed completeness. Fix (split across two files):
  (1) SS-engine-module.md v1.1.9: HTML BEGIN/END delimiter markers (whose canonical line-anchored
  regex patterns are specified in BEGIN_DELIMITER_REGEX and END_DELIMITER_REGEX in clause 4 of
  §Semgrep Coverage Hardening) wrap the audit table rows,
  enabling a CI Python script to machine-parse the declared struct list. Audit table expanded
  from 7 to 17 structs (see SS-engine-module.md §Trace v1.1.9 for the complete expansion log).
  (2) This file v1.6: new semgrep rule `monocle-non-exhaustive-struct-audit-completeness` added
  as the 5th rule in §Semgrep Rules. The rule has `severity: WARNING` (unlike the 4 ERROR-severity
  anti-pattern rules) because it is not a direct defect detector — it is an enumeration tool whose
  output feeds a Python script (Step 3 — Audit-table gap check). The rule matches `#[non_exhaustive]
  pub struct $NAME { ... }` in monocle crate source directories; fixture file
  `semgrep-fixtures/non_exhaustive_struct.rs` added to §Semgrep Coverage Hardening table.
  (3) §CI assertions expanded with "Step 3 — Audit-table gap check" specifying the
  `scripts/check_audit_table.py` contract: parse semgrep JSON output, extract struct names from
  the delimiter-bounded table, compute set difference, fail CI if any struct is absent. The Python
  script implementation is the devops-engineer's Phase 1 deliverable.
  Mechanism choice rationale: semgrep + Python CI script over Rust `syn`-based integration test.
  Semgrep is already in the CI pipeline (4 existing rules, same two-step pattern, same fixture
  corpus). A `syn`-based test binary adds a new dev-dependency and compile step. The Python script
  fits the existing pattern and requires no compilation.

v1.5 changes (round-27 fixes F-R26-adv-2 MEDIUM / F-R26-adv-3 MEDIUM / F-R26-adv-6 LOW):
- F-R26-adv-2 RESOLVED (MEDIUM — adversary finding): The `monocle-no-raw-env-mutation-in-tests`
  semgrep rule previously matched only the fully-qualified path form `std::env::set_var(...)`.
  The common Rust idiom `use std::env; env::set_var(...)` writes `env::set_var(...)` and was
  not matched — creating a false-green where tests using the module-relative form silently
  bypassed CI enforcement. Fix: rule expanded from two `pattern-either` entries to four,
  adding `env::set_var($X, $Y)` and `env::remove_var($X)` alongside the fully-qualified
  forms. Bare-import form (`use std::env::set_var; set_var(...)`) is explicitly NOT covered
  by semgrep (documented in both §Semgrep Rules and §Test Conventions with rationale: would
  collide with user-defined functions of the same name; prose discourages it; nosemgrep
  comment is the suppression mechanism for legitimate user-defined functions).
- F-R26-adv-3 RESOLVED (MEDIUM process-gap — adversary finding): All four semgrep rules
  could produce zero findings on every CI run without indicating a problem — the rule may
  be silently broken (wrong path glob, incompatible semgrep pattern syntax for the pinned
  semgrep version, or pattern-either that matches nothing because of Rust AST differences).
  Fix: added §Semgrep Coverage Hardening subsection (between §Semgrep Rules and §PR Template
  Checklist) specifying: (1) `semgrep-fixtures/` directory with one fixture file per rule;
  (2) CI Step 1 — fixture corpus scan asserting each rule produces the expected non-zero
  finding count; (3) CI Step 2 — production scan asserting zero findings per rule; (4) log
  line format for both steps; (5) step ordering (fixture step first; skip production scan
  if fixture step fails). The CI wiring (actual workflow YAML, fixture file content, semgrep
  version pin) is the devops-engineer's Phase 1 deliverable — this section specifies the
  behavioral requirement with enough precision for implementation without architect round-trips.
- F-R26-adv-6 RESOLVED (LOW — adversary finding): The `monocle-no-raw-env-mutation-in-tests`
  semgrep rule was defined in two places: partially in §Semgrep Rules (via a trailing comment
  "Add this rule to .semgrep.yml") and fully in §Test Conventions (as a YAML block). This
  created a single-source-of-truth violation: a future editor might update one copy but not
  the other. Fix: the canonical rule definition is now in §Semgrep Rules as the 4th rule
  (complete YAML block). The §Test Conventions §CI enforcement paragraph now cross-references
  §Semgrep Rules as the canonical location rather than duplicating the YAML. The rule's
  expanded pattern-either (from F-R26-adv-2) is defined once in §Semgrep Rules only.

v1.4 changes (round-24 fix F-R24-adv-5):
- F-R24-adv-5 RESOLVED (LOW process-gap — adversary finding): BC-ENGINE-002-ERR
  established a precedent for `temp-env` usage in env-mutating tests, but the
  conventions doc did not codify the rule. Future tests for env-sensitive code paths
  (e.g., `MONOCLE_NO_AUTOSTART`, `CLAUDE_SESSION_ID`, daemon lock-file path resolution)
  could silently regress to raw `std::env::set_var`/`remove_var` — which Rust 1.86+
  marks `unsafe` in multi-threaded contexts and which lacks panic-safe cleanup. Fix:
  added §Test Conventions subsection with forbidden/required patterns, Cargo.toml
  declaration, canonical usage example pointer (BC-ENGINE-002-ERR), and a semgrep CI
  rule that rejects raw env mutation in test files. The semgrep rule is scoped to test
  file paths only to avoid false positives on production code that reads (but does not
  write) env vars (e.g., daemon start sequence `std::env::var("CLAUDE_SESSION_ID")`).
