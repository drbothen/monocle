---
document_type: architecture-section
level: L3
section: "conventions-anti-patterns"
subsystem: cross-cutting
version: "1.32.4"
status: complete
producer: architect
phase: phase-3
timestamp: 2026-05-29T12:00:00Z
inputs: [product-brief.md, research/domain-monocle-vision-synthesis.md]
input-hash: "0351fc8"
traces_to: architecture/ARCH-INDEX.md
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

## Non-Exhaustive Structs with Public Constructors

`#[non_exhaustive]` on a `pub struct` prevents external crate consumers from constructing
the struct via struct-literal syntax. For internal workspace crates, `pub fn new(...)`
positional constructors are an acceptable alternative when three criteria are met:

1. **Internal workspace scope:** Not published to crates.io; not consumed outside this workspace.
2. **External protocol anchor:** Models an external wire protocol where field additions require
   coordinated, intentional changes (Claude Code version bump + monocle BC revision + story).
3. **All required fields present as positional parameters.** No `Default` substitution.

The 5 hook event inner structs (`SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`,
`NotificationEvent`, `StopEvent`) and `HookEventRecord` meet all three criteria (see ADR-0006).

### Breaking-change discipline for new() on non_exhaustive structs

Adding a new **required field** to a `#[non_exhaustive]` struct with `pub fn new(...)` is a
**breaking change**. When this occurs:

- Add the field as a new positional parameter to `new()`.
- Update ALL call sites in the SAME PR. Use `cargo check --workspace` to surface every gap.
- Add a §Trace entry in the owning architecture spec documenting the new field, its source,
  and the rationale.
- Add a BC revision if the field addition changes wire behavior.

Adding a new **optional field** (`Option<T>`) is NOT a breaking change — initialize to `None`
in the constructor body.

Code review MUST reject a `new()` constructor addition that:
- Is missing from the Cross-Crate Constructor Audit Table in `SS-engine-module.md`.
- Is applied to a struct that may be published externally in Phase 4 without a new ADR.
- Has positional parameters that omit any required field.

See ADR-0006 for full rationale and the authoritative struct table.

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
  { path = "std::println", reason = "use tracing::info!() with structured fields; println! forbidden in production code per §Convention Checklist" },
  { path = "std::eprintln", reason = "use tracing::error!() or tracing::warn!() with structured fields; eprintln! forbidden in production code per §Convention Checklist" },
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
# cargo-deny 0.19 schema: [advisories] no longer has vulnerability/unmaintained/yanked/notice
# severity lint-level fields. Those were removed in the 0.17 schema migration
# (see https://github.com/EmbarkStudios/cargo-deny/pull/611). cargo-deny 0.19 uses the
# advisory database directly for RUSTSEC detection; all vulnerabilities are denied by default.
db-path = "~/.cargo/advisory-db"
db-urls = ["https://github.com/rustsec/advisory-db"]
ignore = []

[licenses]
# cargo-deny 0.19 schema: [licenses] no longer has unlicensed/copyleft/allow-osi-fsf-free/exceptions
# top-level keys (removed in 0.17 schema migration). Confidence-threshold and allow/deny lists remain.
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
confidence-threshold = 0.8

[bans]
multiple-versions = "warn"
# wildcards = "deny" is intentionally NOT set here. In cargo-deny 0.19, path-based workspace
# member dependencies (e.g. `monocle-core = { path = "../monocle-core" }`) are treated as
# version wildcards because they have no explicit version specifier. This is a standard Rust
# workspace pattern; denying it would block all intra-workspace path deps. The anti-pattern we
# want to prevent (unbounded version ranges in registry deps) is enforced by the EXACT-pin policy
# in SS-deps-pin-manifest.md and by `cargo clippy` + code review rather than cargo-deny wildcards.
wildcards = "allow"
deny = [
    { name = "openssl", reason = "use rustls; openssl is a deployment-complexity liability" },
    { name = "openssl-sys", reason = "see openssl" },
    { name = "tokio", version = "<1.52", reason = "RUSTSEC remediated 1.52+" },
    { name = "russh", version = "<0.60", reason = "transitive rsa pre-release RUSTSEC-2023-0071" },
]
# skip entries for transitive duplicates forced by EXACT-pinned crates in Phase 1 workspace.
# These are human-accepted duplicates; resolution requires future wave decisions.
skip = [
    # getrandom: rand =0.8.6 EXACT pin requires getrandom 0.2.x (via rand_core 0.6);
    # tempfile 3 + prost-build 0.14 require getrandom 0.4.x (via wasi/rustix path).
    # Resolution requires rand 0.9+ (moves OsRng to a feature flag; ergonomic regressions;
    # deferred per SS-deps-pin-manifest §Patch-Pinning Policy rationale for rand).
    { name = "getrandom", reason = "duplicate forced by rand =0.8.6 EXACT pin (0.2.x) vs tempfile/prost-build transitive (0.4.x); resolve when rand 0.9 ergonomics stabilize" },
    # wit-bindgen: getrandom 0.4 pulls in wasip2 v1.0 + wasip3 v0.4 which each depend on
    # different wit-bindgen versions (0.57.1 vs 0.51.0). Purely transitive through tempfile.
    { name = "wit-bindgen", reason = "duplicate forced by wasip2/wasip3 divergence in getrandom 0.4 transitive path via tempfile; no direct monocle dependency" },
]
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

**Rationale for `wildcards = "allow"` (A2 decision):**

The original spec set `wildcards = "deny"` to prevent unbounded version ranges in registry dependencies. In cargo-deny 0.19, however, intra-workspace `path = "../monocle-*"` references are classified as wildcard version selectors because they carry no explicit version number — this is the standard Rust workspace pattern and is not the anti-pattern the rule targeted. Setting `wildcards = "deny"` in 0.19 would block every intra-workspace `path =` dep, making the workspace inoperable. The protection originally provided by the wildcard ban is fully delivered by: (a) the EXACT-pin / caret-pin policy in `SS-deps-pin-manifest.md`, (b) `cargo clippy` lint enforcement, and (c) PR code review. No enforcement gap is introduced by `wildcards = "allow"`.

**Rationale for `skip` entries (A3 / A4 decisions):**

Two transitive duplicate crates are accepted as known multi-version state for Phase 1:

- **getrandom (A3):** rand `=0.8.6` (EXACT-pinned per RUSTSEC-2026-0007 prost mitigation) transitively requires `rand_core 0.6` which requires `getrandom 0.2.x`. `tempfile 3` and `prost-build 0.14` transitively require `getrandom 0.4.x` via the wasi/rustix path. Two getrandom versions coexist in the graph. Resolution requires migrating to `rand 0.9` which moves `OsRng` behind a feature flag and has documented ergonomic regressions. This migration is deferred to a post-Wave-2 decision per `SS-deps-pin-manifest.md` §Patch-Pinning Policy rationale for rand.

- **wit-bindgen (A4):** A cascade of the getrandom 0.4 transitive path. `getrandom 0.4` pulls in `wasip2 v1.0` and `wasip3 v0.4` which depend on incompatible wit-bindgen versions (`0.57.1` vs `0.51.0`). monocle has no direct wit-bindgen dependency; this duplicate is entirely transitive through `tempfile`. Resolution is coupled to the getrandom/rand migration in A3. Surfaced by devops-engineer during S-001 fix PR; ratified alongside A3.

**Rationale for MPL-2.0 inclusion:**

MPL-2.0 (Mozilla Public License 2.0) is file-level copyleft: it requires modifications to MPL-licensed files to be released under MPL, but does not propagate to files in different compilation units. monocle uses nucleo 0.5 (matcher/scorer library; see ADR-0002) which is MPL-2.0. Because monocle links nucleo as an unmodified library crate and does not modify nucleo source files, the file-level copyleft does not impose redistribution obligations on monocle's own code. Any new MPL-2.0 additions will be surfaced by cargo-deny's license check against the explicit allow list.

**`targets = []` during Phase 1:**

The empty targets list instructs cargo-deny to analyze the dependency graph without restricting to a specific platform triple. This is correct during spec and early implementation phases when the exact deployment targets have not been finalized. During `/vsdd-factory:create-architecture` workspace initialization, the targets list will be populated with the pinned platform triples (e.g., `x86_64-unknown-linux-gnu`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`) to enable platform-specific dependency filtering.

#### GitHub Actions wiring

Use the `EmbarkStudios/cargo-deny-action` composite action, pinned to SHA. Place this step between `cargo test` and `cargo audit` in the CI workflow:

```yaml
- name: cargo deny check
  uses: EmbarkStudios/cargo-deny-action@<SHA>
  with:
    log-level: warn
    command: check all
    arguments: --workspace --all-features
```

Note: pin the `uses:` line to a full commit SHA when creating the workflow file (per the project's action-pinning requirement). The `@v2` reference above is illustrative; the devops-engineer must resolve the SHA at workflow creation time using `gh api repos/EmbarkStudios/cargo-deny-action/git/refs/tags/v2`.

### SBOM Generation

SBOM (Software Bill of Materials) generation via `cargo sbom` (or `cargo cyclonedx`) is run per-release tag in CI. Output format: CycloneDX JSON 1.6 (CISA-compliant; EU CRA-compliant). SBOM is attached to the GitHub Release artifact. Per-PR runs are NOT gated on SBOM (only on cargo-deny); SBOM is a release-time artifact, not a merge gate. License audit per merge is handled by cargo-deny (above).

## R-001 Monitoring Workflow

### Purpose

Weekly scheduled check of Anthropic agent-view release notes against the 4 R-001 re-eval trigger conditions. Opens a labeled GitHub Issue when any condition matches. Per brief §Competitive Positioning.

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

The keyword patterns in §Companion Script Contract above are anchored to Anthropic's terminology as of brief v1.4.6 at spec authoring time. Anthropic may introduce new wording (e.g., referring to hook-protocol ingestion under a new product name) that does not match current patterns, creating false-negative drift.

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
dtu-assessment.md v1.7.5 §monocle-canonical column, verified against SS-core-types-and-abi.md
v1.2.13 §Non-Exhaustive Inner Structs. Re-validation grep: `grep -rn 'field X.*all.*hook' .factory/specs/`"

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

**PG-D042-DTU-SCOPE (codified v1.21, round-52.2 — 8th META-pattern recurrence root-cause):**
The two patterns above match only `SS-[a-z-]*`-prefixed filenames. Non-SS-prefixed spec
artifacts that have versioned frontmatter — `dtu-assessment.md`, `domain-monocle-vision-synthesis.md`,
`product-brief.md` — are NOT caught by the primary or secondary patterns. This was confirmed
as the root cause of F-R52R-1 (round-52.2): `dtu-assessment.md` was bumped to v1.5 in
round-52.1 but three citation sites in SS-forward-compatibility.md remained stale at v1.4
because the R52.1 D-042 sweep ran only the `SS-[a-z-]*` primary pattern.

The D-042 recipe MUST include these sibling grep patterns run in addition to the primary
and secondary patterns:

**Sibling pattern — dtu-assessment.md version citations:**
```
grep -rn "dtu-assessment\.md v" .factory/specs/
```

**Sibling pattern — vision document version citations:**
```
grep -rn "domain-monocle-vision[^ ]*\.md v" .factory/specs/
```

Run all four patterns (primary, secondary, dtu sibling, vision sibling) before any version
bump of `dtu-assessment.md`, `domain-monocle-vision-synthesis.md`, or any SS-* document
that triggers a cascade. The sibling patterns produce few false positives; classify each
hit as a current-pointer or a historical §Trace pinpoint before declaring CLEAN.

Note on product-brief.md citations: `product-brief.md` version citations in CLAUDE.md are
governed by D-041 (brief edits route through product-owner) and are exempt from the D-042
automated sweep per Q-3 standing disposition. Do NOT add a product-brief sibling pattern
to the automated D-042 recipe; human-routed brief version maintenance is the correct control.

**PG-D042-WITHIN-FILE (codified v1.23, round-54.1 — 10th META-pattern recurrence, first
within-file-partial cascade variant):**
The D-042 recipe and cascade workflow above cover cross-file citation staleness. A single
file can also contain MIXED citations to the same source document — some historical pinpoints
(which must be preserved) and some current-pointer citations (which must be updated). If a
sibling-file bump triggers a cascade and the architect runs the D-042 grep selectively by
section (updating one section while missing another in the same file), the within-file
partial cascade produces the same inconsistency as a cross-file miss but is harder to detect
because both the stale and the correct value coexist in one file.

**Sub-rule:** When a sibling-file bump triggers a D-042 cascade, the architect MUST run
the D-042 grep at the **file level** (not selectively by section) for every file that
contains any citation to the bumped source document. For each match, explicitly classify
as:
- **Historical pinpoint** (preserve): the citation records a specific version at which
  a decision was made; it is an immutable audit record. Example: a Disposition column entry
  recording `locked in SS-daemon-lifecycle.md v1.0.6 per human authorization` remains at <!-- version-pin-historical: illustrative example in §Citation Discipline -->
  v1.0.6 even after SS-daemon-lifecycle.md is bumped to v1.0.7, because v1.0.6 was the
  version at lock-in time.
- **Current-pointer** (update): the citation asserts the current authoritative version
  of a fact or spec. Example: a Phase 1 Spec Change column entry `specified in
  SS-daemon-lifecycle.md v1.0.6 §Drain` <!-- version-pin-historical: illustrative example in §Citation Discipline --> must be updated to match the current version.

**Selective within-file updates are FORBIDDEN** — either update all current-pointer
citations in the file atomically in a single burst, or annotate why specific citations
are intentionally historical. A file may NOT have some current-pointer citations at the
old version and others at the new version after a cascade sweep.

The grep patterns above (primary, secondary, dtu sibling, vision sibling) are the
correct tool for the file-level scan. Run them scoped to the individual citing file,
not the full spec tree, to minimize false positives:

```
grep -n "SS-daemon-lifecycle\.md v" .factory/specs/architecture/SS-forward-compatibility.md
```

For each hit, look at the surrounding prose to classify as historical or current-pointer
before updating.

The grep pattern for hook body field presence (schema-fact validation):

```
grep -rn "session_id.*all.*hook\|all.*hook.*session_id\|present in all 5\|in all 5 hook" .factory/specs/
```

Run this grep in addition to the D-042 version-citation greps before any version bump of
`dtu-assessment.md` or `SS-core-types-and-abi.md`.

**PG-D042-BACK-CASCADE (codified v1.29.4, round-114 F-R114-1 root-cause closure):**
D-042 cascade has historically been applied in the forward direction only: when file X is bumped,
scan for citations to X in sibling files and refresh them. The back-cascade direction — when file X
is bumped, also check whether X itself cites OTHER files whose versions have since advanced — was
not codified, producing citation-staleness within the file that receives the cascade, not just in the
files that source it. F-R114-1 found that SS-forward-compatibility.md carried stale citations to
`dtu-assessment.md v1.7` and `SS-core-types-and-abi.md v1.2.8` <!-- version-pin-historical: version at F-R114-1 finding time --> even after those files had
advanced to v1.7.5 and v1.2.13 respectively.

**Explicit back-cascade obligation:** When bumping file X (the cascade recipient), ALSO run:

```
grep -rn 'X\.md v' .factory/specs/ | grep -v '§Trace'
```

where `X` is each document cited inside the file being updated. Classify every hit as
historical-pinpoint (immutable) or current-pointer (must update). Complete all current-pointer
refreshes in the same dispatch before committing.

**Mnemonic:** D-042 is bidirectional — forward (who cites X?) AND backward (what does X cite?).
Both directions run in every cascade sweep.

## Phantom-ID Convention

Added in v1.14 (round-47, F-R46-2 root-cause closure).

**Rule:** Every BC ID, VP ID, or other VSDD artifact ID referenced in a spec document MUST be
attested — verifiable in at least one of: (a) the pre-staged BC table in SS-forward-compatibility.md
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
SS-engine-module.md v1.1.12 <!-- version-pin-historical: version at fix authoring time --> also corrected `below` → `above` for the audit table reference
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
grep -A1 "^## §?Trace" <file> | grep -A9999 "^## §?Trace" | grep -E '\(L[0-9]+\)|paragraph at L[0-9]+|this file L[0-9]+|L[0-9]+-L[0-9]+'
```

Any match that lacks a version prefix (e.g., `in vX.Y.Z, L...`) is a forbidden current-state
pinpoint and MUST be replaced with the referenced section's heading name.

**Note (heading-agnostic, added v1.27, round-59.1):** The recipe pattern `^## §?Trace` matches
both `## §Trace` (canonical form, required by §Trace-Heading-Convention) and `## Trace`

(legacy or drift form). This makes the defense robust even when a file's §Trace heading is
missing the `§` prefix — the recipe still runs correctly. The §Trace-Heading-Convention (see
§Trace-Heading-Convention below) mandates `## §Trace` as the required heading form. Belt-and-suspenders: convention requires `## §Trace`; recipe accepts both forms to prevent silent bypass.

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

**Enhanced self-audit addendum (F-R58-1-cons, v1.26, round-58.1):** The pre-commit
`grep -nE 'L[0-9]+'` MUST be applied to the FULL text of every newly authored §Trace
version block — not just the changed lines, but every bullet and sentence within the new
block. A block header such as `v1.N changes (...)` does NOT make the bare L-numbers inside
that block compliant. Each individual prose sentence must independently satisfy the
version-prefix requirement. This addendum closes the "check #14 missed sites" failure
pattern: in round-57.1 the PG-3-TRACE-NEW-ENTRY self-audit was considered satisfied after
the block header named the version, but two bullets inside that block contained bare L-numbers
(`§Context` entry and `§Consequences` entry) that remained uncaught. Correct interpretation:
every individual bullet or sentence within the new block is a separate unit of §Trace prose
subject to the PG-3 constraint. The self-audit is only PASSED when the grep returns zero
matches on the entire new version block with no version-prefixed exception required.

## §Section-Anchor Citation Convention (PG-4)

Added in v1.19 (round-51.1, F-R51-adv-1 root-cause closure).

**Rule:** Cross-document `§<Name>` references MUST point to an actual `#`/`##`/`###`/`####`
heading in the cited document. Inline prose mentions of `<Name>` — bold labels
(`**Name:**`), paragraph prefixes, or any non-heading text — do NOT satisfy this convention.
Citations to non-heading content must use the closest enclosing actual heading plus a
position-free description of the content.

**Scope:** PG-4 enforcement applies to the versioned spec artifacts enumerated in
§META-Rule Recipe Sibling-Pattern Convention (PG-RECIPE-SCOPE). Citations in
non-versioned project documentation (e.g., `CLAUDE.md`) where the citation target is
an enumerated item within a list (e.g., `§Rule 1`, `§Rule 3`, `§Canonical Principle`
referencing numbered items under `### Six rules` or `## CANONICAL PRINCIPLE`) are exempt
provided the enumeration item is unambiguous in context.

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

**Pre-commit grep (PG-4 sweep — expanded scope, codified v1.22, round-53.1):**

**Pattern 1 — SS-*.md §-anchor citations (original v1.19 scope):**
```
grep -nE 'SS-[a-z-]+\.md §[A-Z][a-zA-Z0-9 -]+' <file>
```

**Pattern 2 — brief.md §-anchor citations (sibling, codified v1.22):**
```
grep -nE 'brief[^ ]*\.md §[A-Z][a-zA-Z0-9 -]+|brief v[0-9.]+ §[A-Z][a-zA-Z0-9 -]+' <file>
```
Or using the simpler form that catches inline brief citations without .md extension:
```
grep -nE 'brief[^.]*§[A-Z][a-zA-Z0-9 -]+' <file>
```

**Pattern 3 — dtu-assessment.md §-anchor citations (sibling, codified v1.22):**
```
grep -nE 'dtu-assessment[^ ]*\.md §[A-Z][a-zA-Z0-9 -]+' <file>
```

**Pattern 4 — domain-monocle-vision §-anchor citations (sibling, codified v1.22):**
```
grep -nE 'domain-monocle-vision[^ ]*\.md §[A-Z][a-zA-Z0-9 -]+|vision §[A-Z][a-zA-Z0-9 -]+' <file>
```

**Pattern 5 — ADR-N.md §-anchor citations (sibling, codified v1.22):**
```
grep -nE 'ADR-[0-9]+[^ ]*\.md §[A-Z][a-zA-Z0-9 -]+' <file>
```

For each match from any pattern, open the cited file and verify the §-named portion
corresponds to an actual `#`/`##`/`###`/`####` heading. Use:
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
the class: any `§<Name>` must resolve to a real heading, not prose. The original v1.19
recipe covered only SS-*.md citations; F-R53-adv-2 (round-53.1) confirmed at least 5
brief-§-anchor mis-anchors across the corpus that the SS-only pattern never caught. The
sibling patterns above extend PG-4 sweep to all versioned spec artifacts.

**Relationship to PG-1/PG-2/PG-3/PG-RECIPE-SCOPE/PG-5:** PG-4 is the fourth META-pattern class:
- PG-1: Schema-fact citations must include version pin (§Schema-Fact Citation Convention)
- PG-2: Narrative counts must match structural reality (§Schema-Fact Citation Convention, generalized PG-2 sub-rule)
- PG-3: Directional qualifiers above/below must be position-accurate (§Cross-Section Directional Reference Convention)
- PG-4: §<Name> citations must resolve to actual headings (this section)
- PG-RECIPE-SCOPE: Every new META-rule's sweep recipe must include sibling patterns for all versioned spec artifacts (§META-Rule Recipe Sibling-Pattern Convention)
- PG-5: Cross-artifact version citations in main-body prose must be current-pointer OR historical-anchor OR version-free (§Historical-Anchor Framing Convention)

## §META-Rule Recipe Sibling-Pattern Convention (PG-RECIPE-SCOPE)

Added in v1.22 (round-53.1, F-R53-adv-2 root-cause closure at the META-META level).

**Rule:** When codifying any new META-rule with a pre-commit grep recipe, the recipe MUST
include sibling patterns covering ALL versioned spec artifacts, not just `SS-*` files, as
part of the initial codification — not as a follow-up burst.

**Versioned spec artifacts in this corpus:**
- `SS-*.md` (7 architecture spec files)
- `dtu-assessment.md`
- `domain-monocle-vision-synthesis.md`
- `product-brief.md` (D-041 routing — read-only for automated sweeps; cited by architecture)
- `ADR-N-*.md` (architecture decision records)

**Root cause:** D-042 (v1.18 PG-D042-BURST-SKIP closure → v1.21 PG-D042-DTU-SCOPE) and PG-4
(v1.19 codification → v1.22 sibling extension) both required follow-up bursts to add sibling
patterns. The defect is structural: an SS-only recipe matches the bulk of the corpus but
silently excludes sibling artifacts. PG-RECIPE-SCOPE prevents recurrence at the META-META
level — the 9th META-pattern recurrence root-cause closure.

**Self-audit at codification time:** Before committing a new META-rule with a sweep recipe,
run the recipe against all versioned spec artifact classes. If the recipe pattern would
match `SS-foo.md` but not `product-brief.md`, `dtu-assessment.md`,
`domain-monocle-vision-synthesis.md`, or `ADR-N-*.md`, the recipe is INCOMPLETE — add the
sibling patterns before merge.

**Anti-pattern:**
```
# WRONG — SS-only scope, silently excludes sibling artifacts
grep -nE 'SS-[a-z-]+\.md §[A-Z][a-zA-Z0-9 -]+' <file>
```

**Correct form (PG-RECIPE-SCOPE compliant):** Include a sibling pattern for each artifact
class in the corpus. See §Section-Anchor Citation Convention §PG-4 sweep for the reference
implementation with all 5 patterns.

**Historical recurrence log:**
- Round-51.1 (v1.19): PG-4 codified with SS-only grep. No sibling patterns.
- Round-53.1 (v1.22): F-R53-adv-2 confirmed 5+ brief-§-anchor mis-anchors that the
  SS-only pattern never caught. Sibling patterns added; PG-RECIPE-SCOPE codified.
- Analogous to D-042 scope hole: v1.18 D-042 recipe was SS-only; v1.21 PG-D042-DTU-SCOPE
  added dtu-assessment.md and vision sibling patterns after F-R52R-1.

## §Historical-Anchor Framing Convention (PG-5)

Added in v1.24 (round-56.1, F-R56-2 root-cause closure). Scope extended in v1.25
(round-57.1, F-R57-2 frontmatter carve-out codification + sweep-evidence checklist).

**Rule:** Cross-artifact version citations in main-body prose MUST use one of these three
forms. Bare `<artifact>.md vX.Y §<heading>` where `vX.Y` is neither current nor explicitly
framed as historical FAILS PG-5.

**Form 1 — Current-pointer:** `<artifact>.md vX.Y §<heading>` where `vX.Y` matches the
CURRENT frontmatter `version:` of `<artifact>.md`. Valid only as long as the referenced file
is at `vX.Y`. When the file is bumped, a D-042 cascade MUST update all current-pointer
citations.

**Form 2 — Historical anchor:** `<artifact>.md vX.Y at scan time §<heading>` OR
`per <artifact>.md vX.Y at time of <event>` OR `as of <artifact>.md vX.Y at spec authoring
time §<heading>`. Use when `vX.Y` is the version at the moment the citation was authored
(intentional audit record). Does NOT need updating when the file is bumped — the version
is a provenance marker, not a navigation pointer.

**Form 3 — Version-free section reference:** `<artifact>.md §<heading>` or `brief §<heading>`
(no version qualifier). Use when the referenced section is stable across versions and the
version detail adds no navigational or provenance value. Section stability must be verified
before removing a version qualifier.

**Anti-pattern:** Bare `<artifact>.md vX.Y §<heading>` where `vX.Y` is neither the current
version of the file nor explicitly qualified as historical. Creates false-currency ambiguity:
a reader cannot determine whether `vX.Y` is an intentional historical anchor or an
accidentally stale current-pointer.

**Pre-commit sweep recipe (PG-RECIPE-SCOPE compliant — 5 artifact classes):**
```
# SS-* architecture specs
grep -nE 'SS-[a-z-]+\.md v[0-9]+(\.[0-9]+)*' <file>

# dtu-assessment.md
grep -nE 'dtu-assessment\.md v[0-9]+(\.[0-9]+)*' <file>

# domain-monocle-vision-synthesis.md
grep -nE 'domain-monocle-vision[^ ]*\.md v[0-9]+(\.[0-9]+)*' <file>

# product-brief.md (D-041 routing — read-only for automated sweeps; cited by architecture)
grep -nE '(product-brief|brief)\.md v[0-9]+(\.[0-9]+)*' <file>

# ADR-N-*.md
grep -nE 'ADR-[0-9]+-[a-z-]+\.md v[0-9]+(\.[0-9]+)*' <file>
```

For each match: classify as current-pointer (verify version matches actual frontmatter) OR
historical-anchor (verify the qualifier "at scan time", "at time of", "at spec authoring
time", or similar is present) OR version-free (no version qualifier — verify section is
stable). If none of the three: FAILS PG-5.

**Carve-outs:** §Trace prose entries (governed by PG-3-TRACE-NEW-ENTRY), §Revision History
rows, §Amendment History rows, §Closure Log entries, §Provenance entries, and code-block
examples are exempt from PG-5 — all are historical by section semantic. All frontmatter
audit fields are also exempt (F-R57-2 + F-R58-1-adv, v1.26 Option B ruling — extended from
`traces_to:` only to all audit-record frontmatter fields): `traces_to:`, `producer:`,
`inputs:`, `subsystems_affected:`, `supersedes:`, `superseded_by:`, and any other
frontmatter field whose semantic is provenance or audit record rather than navigation.
Every such field is inherently a historical anchor by its field semantic — it records what
was current at authoring time, not a navigation pointer. The false-currency hazard PG-5
guards against (ambiguity whether vX.Y is intentional or accidentally stale) does not apply
to structured frontmatter read as an audit log. D-042 cascade discipline already excludes
brief version citations from automated cascade (`product-brief.md` excluded from recipe per
Q-3), confirming frontmatter brief-version citations cannot be held to current-pointer
obligations. The generalization closes the sibling-field gap: the rationale that applies to
`traces_to:` applies identically to `producer:` (provenance of who extracted what at
authoring time), `inputs:` (snapshot of inputs at authoring time), and supersedes/superseded
fields (point-in-time ADR lineage record).

**Sweep-evidence checklist (F-R57-2, v1.25):** When claiming a "comprehensive PG-5 sweep,"
the sweep record MUST emit per-class evidence counts. Asserting "CLEAN" without per-class
counts is insufficient. Required format:
```
PG-5 sweep evidence:
- SS-*: N files swept, M violations found, M fixed
- brief: 1 file swept, M violations (D-041 read-only — no edits to brief permitted)
- dtu-assessment: 1 file swept, M violations found, M fixed
- vision: 1 file swept, M violations found, M fixed
- ADR-N: N files swept, M violations found, M fixed
```
If a class is entirely clean, state "0 violations found." If a class is exempt from edit
(e.g., brief under D-041), state the exemption rationale. A sweep that omits the ADR-N
class is incomplete under PG-RECIPE-SCOPE. A sweep that reports counts without listing
each class is also incomplete.

**Relationship to D-042 and PG-1:**
- D-042 targets current-pointer staleness: when a file is bumped, citations to the old
  version must be refreshed (D-042 cascade discipline).
- PG-1 requires schema-fact citations to include a version pin (form 1 or form 2).
- PG-5 closes the gap D-042 does not cover: bare versioned citations that are neither
  current nor explicitly historical. PG-5 does not relax D-042 — it adds a complementary
  classification obligation at authoring time.

**Historical recurrence log:**
- Round-55.1 (v1.2.11 of SS-forward-compatibility.md): §Scope cited "brief v1.4.5 and
  vision v1.1.2"; R55.1 burst correctly identified brief side as false-currency (v1.4.5 vs
  current v1.4.23) but incorrectly claimed "vision v1.1.2 does not exist" without reading
  the vision frontmatter. R56.1 corrects both the factual error and applies PG-5 framing
  to 5 sibling sites in SS-core-types-and-abi.md, SS-conventions-anti-patterns.md, and
  dtu-assessment.md.

## §Trace-Heading-Convention

Added in v1.27 (round-59.1, F-R59-adv-2 root-cause closure at META class level).

**Rule:** Every spec artifact's §Trace section MUST use the heading form `## §Trace` (with
`§` prefix). The heading form `## Trace` (missing `§`) is a PG-3-TRACE-NEW-ENTRY defense
bypass and is FORBIDDEN.

**Enforcement rationale:** The PG-3-TRACE-NEW-ENTRY post-write self-audit recipe (§Cross-Section
Directional Reference Convention) targets `^## §?Trace` (updated to heading-agnostic form in
v1.27). However, using `## Trace` instead of `## §Trace` caused the pre-v1.27 recipe
`^## §Trace` to return zero matches on that file — silently bypassing the entire defense layer
for that artifact. F-R59-adv-2 caught SS-permissions-phase1.md as the sole corpus divergence;
corrected to `## §Trace` in v1.4 of that file.

**Scope:** All versioned spec artifacts in the corpus (per PG-RECIPE-SCOPE): `SS-*.md`,
`dtu-assessment.md`, `domain-monocle-vision-synthesis.md`, `product-brief.md`, `ADR-N-*.md`.

**Pre-commit verification recipe:**
```
grep -nE '^## §?Trace' <file>
```
If the match uses `## Trace` (no `§`), add the `§` prefix before committing. If the grep
returns no matches, the file has no §Trace section — verify this is intentional (only
versioned spec artifacts require a §Trace section).

**Corpus audit (v1.27, round-59.1):** All 7 SS-*.md files plus dtu-assessment.md checked:
- SS-conventions-anti-patterns.md: `## §Trace` — COMPLIANT
- SS-engine-module.md: `## §Trace` — COMPLIANT
- SS-core-types-and-abi.md: `## §Trace` — COMPLIANT
- SS-daemon-lifecycle.md: `## §Trace` — COMPLIANT
- SS-forward-compatibility.md: `## §Trace` — COMPLIANT
- SS-deps-pin-manifest.md: `## §Trace` — COMPLIANT
- SS-permissions-phase1.md: `## Trace` → `## §Trace` (F-R59-adv-2, fixed in v1.4)
- dtu-assessment.md: `## §Trace` — COMPLIANT

Post-fix: 8/8 files COMPLIANT.

**Relationship to PG-3/PG-3-TRACE-NEW-ENTRY:** §Trace-Heading-Convention is a structural
pre-condition for PG-3-TRACE-NEW-ENTRY: if the heading is missing the `§` prefix, the
PG-3 recipe silently skips the file's §Trace section entirely. The convention and the
heading-agnostic recipe update (v1.27) together close the bypass class.

## §Corpus-Wide-Sweep Convention (F-R60-corpus-sweep META rule)

Added in v1.28 (round-60.1, F-R60-corpus-sweep process-gap root-cause closure).

**Rule:** When fixing a count-drift, false-currency, or sibling-propagation defect in spec
prose, the fix burst MUST execute a corpus-wide grep BEFORE declaring closure. Five mandatory
steps:

1. Grep `.factory/specs/` recursively for the stale value pattern.
2. Classify EVERY match as: `historical-correct` (count was accurate at the time and the site
   does not assert current truth); `stale-historical` (count was wrong even at write time);
   `active-stale` (count claims current truth but does not match actual state).
3. Fix every `stale-historical` and `active-stale` instance in the same atomic commit.
4. Document the per-class evidence count in the §Trace entry (analogous to PG-5
   sweep-evidence checklist): e.g., `historical-correct: N, stale-historical: M,
   active-stale: P — M+P sites fixed`.
5. Self-grep the corpus AGAIN after the fix to confirm 0 stale matches remain (or all
   remaining matches are `historical-correct`).

**Anti-pattern:** Asserting "comprehensive sweep" in §Trace when only the immediately-known
sites were corrected. R59.1 did this: it fixed 2 sites in SS-conventions §Trace v1.22 and
v1.19, claimed "comprehensive corpus-wide sweep," and missed 2 additional sibling sites
(SS-conventions §Trace v1.18 L1703 and SS-forward-compatibility §Trace v1.2.9 L333).
R60 independent consistency + adversary passes both caught the recurrence — stress-test
confirmation that the partial-sweep pattern recurs without a structural countermeasure.

**Pre-commit self-application (Step 5 required):**
```
grep -rn "8 architecture spec files\|across 8 architecture\|all 8 architecture\|across 8 spec\|all 8 spec" .factory/specs/
```
After any count-drift fix, self-grep with the stale value; confirm all remaining hits are
`historical-correct` (quoted as old-wrong-value in descriptions of the fix, or denominator is
different from the claimed pattern).

**Scope:** All versioned spec artifacts in `.factory/specs/` recursively (per PG-RECIPE-SCOPE
and D-042 canonical scope: not `.factory/specs/architecture/` only).

**Relationship to PG-2/PG-RECIPE-SCOPE/PG-5:**
- PG-2: META-rule scope / step-renumbering events.
- PG-RECIPE-SCOPE: Every new META-rule's sweep recipe must include sibling patterns for all versioned spec artifacts.
- PG-5: Cross-artifact version citations must be current-pointer, historical-anchor, or version-free.
- F-R60-corpus-sweep: Count-drift and sibling-propagation fix bursts require exhaustive corpus grep + per-class classification before closure. Complements PG-5's sweep-evidence checklist.

## §BC-INDEX Conventions

> F-R110-18 (Round 9B): Cross-listing from `behavioral-contracts/BC-INDEX.md §Conventions`. These conventions apply globally to all BC authoring. Canonical source: BC-INDEX.md §Conventions; this section is a cross-reference for architects and implementers.

### EC Namespace Convention (F-R109-17)

Edge case IDs (EC-NNN) are scoped **per-BC**. EC-013 in BC-2.01.009 and EC-013 in BC-2.02.001 are distinct and NOT in conflict — per-BC scoping is intentional and sound. No global EC namespace exists or is required.

**Rationale:** EC IDs serve as local cross-reference labels within a BC file (cited in test vectors, preconditions, and invariants within the same BC). Global uniqueness would require coordinating EC sequences across 22+ independent BC files without providing additional semantic value — per-BC scoping is the correct granularity for behavioral edge cases.

**Enforcement:** When authoring or modifying a BC, EC-NNN is allocated within that BC's own sequence. Cross-BC EC references use the fully-qualified form `BC-S.SS.NNN EC-NNN` (e.g., `BC-2.01.007 EC-002`) to unambiguously scope the reference.

### Test Name Convention (F-R109-21)

BC test function names use stable legacy-form prefixes (e.g., `test_BC_AUTH_002_...`, `test_BC_DAEMON_003_...`) for test continuity across the BC renumbering event (BC-INDEX §Renumbering Map). These names are **immutable** — renaming them to the new BC-S.SS.NNN form would break test history in CI, coverage reports, and log analysis.

**Rationale:** Test names are stable identifiers in CI systems. The cost of renaming (CI history breakage, grep script updates, log grep pattern updates) exceeds the benefit (alignment to new BC IDs).

**Enforcement:** New BCs authored after the renumbering event (BC-INDEX v1.1+) SHOULD use the new-form prefix `test_BC_2_SS_NNN_...` for new test functions. Existing tests with legacy-form names are NOT renamed.

### Anchor Parenthetical Non-Contradiction (PG-5, F-R110-16)

Any parenthetical appended to a BC-INDEX title MUST NOT contradict the anchor target's H1 title. If a parenthetical adds policy-relevant context, that context must be moved INTO the BC H1 heading (per `bc_h1_is_title_source_of_truth` policy), not left as index-only context.

**Enforcement:** The adversary treats a parenthetical that contradicts or diverges from the BC H1 as a MEDIUM-severity finding.

---

### Architecture Source Pin-Symmetry Convention (F-R117-3, SE-17e)

When a BC Traceability `Architecture Source` cell or a VP Traceability `Architecture Source` cell references **multiple** architecture documents (semicolon-separated), ALL referenced documents MUST carry explicit version pins in the form `SS-name.md vN.M.P` or `ADR-NNNN vN.M.P`. A cell where some references are pinned and others are unpinned is a **pin-symmetry violation** — MED severity per F-R110-8 (originally codified for VP Architecture Source cells; extended to BC Architecture Source cells via SE-17e sibling-propagation in R16C; now codified here in SS-conventions-anti-patterns per F-R118-5 architect dispatch obligation).

**Single-reference cells** (one SS doc) have no symmetry requirement — a single-reference cell is trivially symmetric. Pin-symmetry only activates for two-or-more references.

**Canonical SS version table** (authoritative per R17D; update when architect bumps any SS document):

| SS Document | Canonical Version |
|-------------|-------------------|
| SS-daemon-lifecycle.md | v1.0.32 |
| SS-forward-compatibility.md | v1.2.19 |
| SS-engine-module.md | v1.1.20 |
| SS-core-types-and-abi.md | v1.2.13 |
| SS-deps-pin-manifest.md | v1.1.17 |
| SS-conventions-anti-patterns.md | v1.30.2 |

**Enforcement:** The adversary is instructed to flag any BC or VP Architecture Source cell where ≥2 architecture documents are cited and at least one lacks a `vN.M.P` pin. Such findings are MED severity. A pre-commit grep for `SS-[a-z-]+\.md(?!\s+v\d)` patterns within Architecture Source cell contexts provides automated detection.

**Cross-reference:** Originating authoritative copy in `behavioral-contracts/BC-INDEX.md §Conventions` (BC-INDEX v1.10+). This subsection is the architecture-side restatement for developers authoring BCs, VPs, and architecture citations. Originating finding: F-R117-3 (R117 adversary pass, R16C BC-INDEX dispatch closure). META-discipline parent: SE-17e (sibling-propagation); SE-22 (sibling-sweep META, fourth cycle).

## §Citation Discipline (ADR-0007)

Added in v1.32.0 (D-204 architect-escalation tripwire closure, ADR-0007).

Resolves the 7-instance META-pattern version-pin staleness species. Full decision rationale
in `adr/ADR-0007-version-pin-citation-discipline.md`. This section is the operative convention;
the ADR is the decision record.

### Citation Forms — Permitted and Forbidden

When citing a versioned artifact in any spec, story, BC, VP, ADR, or code file that is NOT
in `.factory/cycles/` (closed cycle records):

**Form 1 — Version-free (PREFERRED for new artifacts):**
```
BC-2.06.005 §Postconditions
SS-deps-pin-manifest.md §Phase-1-Pins
SS-tui.md §AppMode-State-Machine
ADR-0006
```
No version literal. The citation resolves to whatever the canonical current version is.
Section-anchor form `§Name` is encouraged for navigability but not required.

**Form 2 — Historical anchor (PERMITTED where provenance matters):**
```
at time of S-025 authoring, SS-deps-pin-manifest.md v1.2.0
BC-2.06.005 v1.0.5 at S-025 inputs[] declaration time (2026-05-27)
implemented against SS-tui.md v1.8.2 at S-025 spec time
```
Version literal IS present but qualified with a time anchor. This form is frozen at
authoring and MUST NOT be updated as the cited document evolves. It is a provenance record.

**Forbidden (active version-pin literal):**
```
SS-deps-pin-manifest.md v1.2.0          ← no time qualifier   # version-pin-historical
BC-2.06.005 v1.0.5                      ← no time qualifier   # version-pin-historical
inputs: [SS-tui.md v1.8.2]              ← frontmatter active pointer  # version-pin-historical
see SS-engine-module.md v1.1.26 §...    ← active pointer  # version-pin-historical
```
Active version-pin literals in artifact bodies introduce drift pressure: the citation
becomes stale on the next version bump of the cited document.

### Historical Anchor Classification

A citation is a historical anchor (frozen, exempt from the CI freshness check) when it
meets at least ONE of:

1. It appears inside a `## §Trace` section.
2. It is annotated with `# version-pin-historical` (Rust/TOML/YAML) or
   `<!-- version-pin-historical -->` (Markdown) on the same line.
3. It contains a time qualifier: "at time of", "at S-NNN authoring time",
   "at T-NNN dispatch time", "at spec authoring time", "at time of ratification",
   "at initial authoring", or equivalent unambiguous temporal anchor.

A citation that does NOT meet any criterion above is classified as an active pointer
subject to the CI freshness check.

**Carve-out — `.factory/cycles/` directory:** Closed adversarial cycle records are
exempt from the freshness check. They are sealed at closure; their version citations
are historical by directory convention.

### Version-Pin Registry

`.factory/specs/version-pin-registry.yaml` is the machine-readable source of truth
for canonical current versions. The CI lint reads this registry to verify active pointers.

**State-manager obligation:** When committing any document version bump to factory-artifacts,
state-manager MUST update the registry in the SAME commit. The registry and the bumped
document version are atomic (Single-Commit Burst Protocol applies).

**DevOps obligation:** The `monocle-version-pin-freshness` pre-commit hook and CI step
verify that every active version-pin literal in staged files matches the registry
`current_version` for the cited artifact ID. Implementation: devops-engineer Phase 3
deliverable (dispatched at D-204).

### Migration — Legacy Active Pointers

Existing artifacts containing active version-pin literals are NOT migrated all-at-once.
Migration is opportunistic: when any artifact is touched for another reason, convert any
active version-pin literals in that artifact's non-§Trace body to Form 1 (version-free)
in the same edit. This is a per-touch obligation from D-204 onward.

Full corpus migration target: Phase 5 (formal hardening) for VPs; Phase 7 (convergence)
for remaining BCs and stories. The CI gate catches remaining active pointers immediately
even before migration completes.

### Relationship to PG-5

PG-5 (§Historical-Anchor Framing Convention) governs main-body prose version citations.
§Citation Discipline (this section) supersedes PG-5 for the forward-going citation
authoring rule: all new artifacts use Form 1 (version-free) by default, reducing PG-5's
classification obligation to legacy corpus sweep only. PG-5 Form 2 (historical anchor)
is preserved and aligned with the historical-anchor classification in this section.

PG-5 remains operative for legacy active pointers in the corpus until migrated;
PG-5 current-pointer classification (Form 1) becomes obsolete for new artifacts
post-D-204 (new artifacts do not carry active pointers).

## §Structural-Claim Discipline (ADR-0008)

Added in v1.32.2 (D-206 structural-spec drift tripwire closure, ADR-0008).

Addresses the structural-claim sub-species of the broader authoring-time documentation
drift META-pattern. Full decision rationale in
`adr/ADR-0008-structural-claim-discipline.md`. This section is the operative convention;
the ADR is the decision record.

**Related:** §Citation Discipline (ADR-0007) governs version-pin literal citations
(`vN.M.P` form). This section governs structural claims (type names, column counts,
variant lists). Both apply simultaneously.

### Structural Claim Definition

A **structural claim** is any artifact-body statement that asserts the shape, type, or
count of a code-level entity, where the canonical source of truth is a compiled Rust type,
a BC postcondition, or an architecture spec struct declaration. Structural claims appear as:

- **Type-identifier claims:** `sessions: Vec<EnrichedSession>` in story Tasks checklists
  or Downstream Consumer Contract code blocks
- **Table-shape claims:** Markdown tables in code doc-comments that enumerate column names
  corresponding to BC postcondition column lists
- **Count claims:** Prose asserting "N postconditions", "N columns", "N hook endpoints"

### Canonical Source Registry

> **Self-application:** This §Canonical Source Registry table is itself subject to POL-12. Stale entries (citing the wrong line range, deprecated section anchors, or removed canonical sources) will be detected by POL-12 against the cited canonical document's actual content. The architect dispatch in ADR-0008 §Implementation Plan row 4 ("When a new canonical source is added or moved, update this registry") explicitly includes registry-maintenance as a POL-12 closure dependency.

| Structural claim type | Canonical source | Lookup |
|-----------------------|-----------------|--------|
| `App` struct field types | SS-tui.md §App struct (lines 833-864) | Read field declarations; compare cited type |
| Sessions panel column list | BC-2.06.005 §Postconditions PC-2 | Read PC-2 column table; count columns |
| `AppMode` variants | `monocle-core::tui::AppMode` enum | Read enum definition; compare variant list |
| `Action` variants | `monocle-core::tui::Action` enum | Read enum definition; compare variant list |
| Hook endpoint count | BC-HOOK-007 §Postconditions PC-1 | Read endpoint enumeration; count |

When authoring a structural claim about any entity in this table, read the canonical source
FIRST. Do not infer the type, column list, or count from memory or from prior artifacts.

### Permitted and Forbidden Structural-Claim Forms

**Permitted — structurally correct (matches canonical source):**
```rust
pub sessions: Vec<EnrichedSession>   // matches SS-tui.md §App struct line 845
```
```markdown
//! | Session ID | Project | Status | Tokens | Cost | Uptime | Drop |  ← 7 columns per BC-2.06.005 PC-2
```

**Forbidden — structurally stale:**
```rust
pub sessions: Vec<SessionState>      // WRONG: canonical type is EnrichedSession
```
```markdown
//! | Icon | Project | Status | Tokens | Cost | Uptime |  ← 6 columns; canonical PC-2 has 7
```

### Historical Anchor Classification for Structural Claims

A structural claim is a historical anchor (frozen, exempt from CI check) when it
meets at least ONE of:

1. It appears inside a `## §Trace` section.
2. It is annotated with `<!-- structural-claim-historical -->` on the same line or
   the adjacent line.
3. It contains a time qualifier establishing this as a record of past state:
   "at S-NNN authoring time", "as of vN.M.P", or equivalent unambiguous temporal anchor.

### CI Enforcement Gate (POL-12-structural-claim)

`monocle-structural-claim-check` — devops-engineer Phase 3 deliverable (dispatched at D-206).

**Scope:** `.factory/stories/*.md` Tasks checklists + Downstream Consumer Contract code
blocks; `.factory/specs/behavioral-contracts/**/*.md` postcondition prose;
`crates/**/*.rs` module-level doc-comment tables.

**Exempt:** `.factory/cycles/` (closed records); `## §Trace` sections; lines annotated
with `<!-- structural-claim-historical -->`.

**CI step ordering:** After `cargo test`, before `cargo deny check`. Separate CI step
from POL-11 `monocle-version-pin-freshness` — both must pass independently.

**Implementation notes for devops-engineer:**

Phase 1 (immediate): Scan `.factory/stories/*.md` for type identifiers in Tasks
checklists and Consumer Contract code blocks. Grep pattern:
```
grep -n 'Vec<\|VecDeque<\|Option<' .factory/stories/*.md | grep -v '§Trace\|historical'
```
For each match involving an `App` struct field, extract the type argument and compare
against SS-tui.md §App struct canonical declarations. Fail with:
`structural-claim mismatch: <file>:<line> cites App.<field> as <cited-type> but canonical SS-tui.md §App struct declares <canonical-type>`.

Phase 2 (Phase 5 scope): Module-level doc-comment table shape extraction from
`crates/**/*.rs`. Pattern: `grep -n '//! |' crates/**/*.rs | grep -E '\|.*\|.*\|'`.
Compare column count against the BC cited in the same doc-comment block.

## §ADR Authoring Discipline

**Codified D-ADV30 (F-S025-ADV30-HIGH-001 closure). Governs all ADR documents in `.factory/specs/architecture/adr/`.**

The §Trace-escaping-into-normative-content defect has appeared 4 times across the S-025
convergence cycle (ADR-0006 indirect path; ADR-0007 Pass 26 HIGH-001; ADR-0008 Pass 28
MED-002; ADR-0008 Pass 30 HIGH-001). Each instance required a fresh-context adversarial
pass to detect. The pattern: §Trace entry prose is accidentally inserted inside a normative
section (numbered list, table, or body paragraph) during an edit that adds a new §Trace
entry, instead of creating a new `## §Trace vN.M.P` section.

### Pre-Commit ADR Self-Consistency Checklist

Before committing any change to an ADR file, the author MUST verify all five points:

**1. §Trace section header ↔ entry label match:**
Every `## §Trace vN.M.P` section header must contain an entry labeled `**N.M.P**`
or `**vN.M.P**` (or a descriptive bold title beginning a paragraph). A header of
`## §Trace v1.0.2` with a body entry labeled `**1.0.3**` is a defect — the header
and label must agree on the version number. Check: after any §Trace section rename
or new-version addition, grep the section body for bold labels and verify they match
the header.

**2. No §Trace prose inside normative sections:**
Grep the edited ADR file for bold version labels: `grep -n '^\*\*[0-9]' <file>`.
Every match must be either (a) inside a `## §Trace` section (between that section's
header and the next `##` header), or (b) inside a fenced code block, or (c) an
annotated historical-anchor. If a match falls inside a numbered list, table, or
non-§Trace body paragraph, the §Trace prose has escaped — extract it into its own
`## §Trace vN.M.P` section before committing.

**3. Table cell pipe escape in regex patterns:**
Any regex or pattern string inside backticks that contains a `|` alternation operator
MUST escape it as `\|`. The `validate-table-cell-count` pre-commit hook counts `|`
characters as table structural pipes and will reject rows where the count does not match
the header. An unescaped `|` inside a backtick-delimited pattern (e.g., in an
Implementation Plan table cell) is a pre-commit hook violation.

**4. Numbered list continuity:**
After any ADR edit that touches a numbered list, verify the list reads 1, 2, 3, ...
without gaps. A gap (e.g., 1 followed directly by 3) indicates either: (a) an item's
text was removed but its number was not renumbered, or (b) a §Trace entry consumed
a list item's position by insertion between items. Check by rendering the list mentally
or with `grep -n '^[0-9]\+\. ' <file>`.

**5. Line-level self-references verified:**
Before citing a specific line number in the same file (e.g., "see lines 121-125"),
re-verify those lines exist and contain the referenced content. Off-by-N defects are
a recorded failure mode (ADR-0008 Pass 28 MED-002: off-by-2 at lines 831-864 → 833-864).
This is especially important when the ADR references its own section content by line range.

### Anti-Pattern Summary

| Anti-pattern | Correct form |
|-------------|-------------|
| `## §Trace v1.0.2` header with `**1.0.3**` labeled body entry | Rename header to match label, or rename label to match header |
| Bold version label `**1.0.2** (date) — ...` inserted between items 2 and 3 of a numbered list | Create a new `## §Trace v1.0.2` section; remove from numbered list |
| Regex in table cell: `(SS-[a-z-]+\.md|BC-[0-9.]+)` | Escape alternation: `(SS-[a-z-]+\.md\|BC-[0-9.]+)` |
| Numbered list reads 1, (blank), 3 after edit | Renumber to 1, 2, 3 after extracting the interloping content |
| "See lines 831-864" when struct body starts at line 833 | Re-read the file to verify line range before authoring |

**Related policies:** ADR-0007 §Implementation Plan "Immediate (D-ADV30)" items; ADR-0008 §Trace v1.0.4 (corrective entry for the 4th instance).

## §Trace

v1.32.4 changes (D-ADV30 F-S025-ADV30 remediation — ADR authoring discipline codified):

- NORMATIVE: §ADR Authoring Discipline section added. Codifies the pre-commit ADR
  self-consistency checklist (5 checks: §Trace header↔label match; no §Trace prose in
  normative sections; table cell pipe escape; numbered list continuity; line-level
  self-reference verification) and an Anti-Pattern Summary table. Motivated by the
  4th recorded instance of the §Trace-escaping-into-normative-content defect class.
  Cross-references ADR-0007 v1.0.5 §Implementation Plan (D-ADV30 items) and
  ADR-0008 v1.0.4 §Trace v1.0.4 (corrective entry).
- NORMATIVE: Version bump 1.32.3 → 1.32.4.

v1.32.3 changes (Pass 28 F-S025-ADV28-MED-002 propagation closure — ADR-0008 §Canonical Source Registry off-by-2 correction):

- NORMATIVE: §Structural-Claim Discipline §Canonical Source Registry `App` struct field types row line range corrected: `(lines 831-864)` → `(lines 833-864)`. Matches ADR-0008 v1.0.1 correction. Lines 831-832 of SS-tui.md are the code-block fence and filename comment; struct body begins at line 833.
- NORMATIVE: Explicit self-application policy note added above §Canonical Source Registry table: the registry is itself subject to POL-12 (stale entries detected by POL-12 against cited canonical document content). Mirrors the same note added to ADR-0008 v1.0.1.
- SE-16d PASS: 2026-05-29 — same calendar day as v1.32.2; sequential same-burst correction.

v1.32.2 changes (D-206 ADR-0008 structural-claim discipline — structural-spec drift tripwire closure):

- NORMATIVE (ADR-0008 ratification — Task #9 m.6 tripwire fired at Pass 27): §Structural-Claim
  Discipline section added above. Codifies POL-12-structural-claim as complement to ADR-0007
  POL-11-version-pin. Both address authoring-time documentation drift species; POL-11 governs
  literal `vN.M.P` pins, POL-12 governs type-identifier and table-shape claims.
- NORMATIVE: Canonical source registry added for App struct field types, sessions panel column
  list, AppMode/Action variant lists, hook endpoint count.
- NORMATIVE: Historical anchor classification for structural claims codified (at-least-one-of
  §Trace / structural-claim-historical annotation / time qualifier — mirrors ADR-0007 form).
- NORMATIVE: POL-12 CI enforcement gate specification added for devops-engineer Phase 3 delivery.
- NORMATIVE: Frontmatter version bumped v1.32.1 → v1.32.2 (patch: new discipline section, no
  changes to existing rules). Timestamp updated to 2026-05-29T12:00:00Z.
- INFORMATIONAL: §Citation Discipline (ADR-0007) is unchanged. §Structural-Claim Discipline
  (ADR-0008) is additive; both sections apply simultaneously.
- SE-16d PASS: 2026-05-29T12:00:00Z > chain high-water 2026-05-29T10:00:00Z (monotonic).

v1.32.1 changes (Pass 26 F-S025-ADV26-HIGH-001 + LOW-001 — ADR-0007 internal-consistency correction):

- NORMATIVE (cross-reference only): §Historical Anchor Classification body in this document
  was already correct (at-least-one-of formulation, lines 1617-1630 at v1.32.0 authoring time).
  ADR-0007 §Historical Anchor Classification has been corrected to match — see ADR-0007 v1.0.1
  §Trace entry for adjudication rationale (Option B selected). The two documents now agree.
- INFORMATIONAL: §Historical Anchor Classification body unchanged. No rule text modified in
  this file. Version bump to v1.32.1 records the alignment closure.
- SE-16d PASS: 2026-05-29T10:00:00Z.

v1.32.0 changes (D-204 ADR-0007 version-pin citation discipline):

- NORMATIVE (ADR-0007 ratification — architect-escalation tripwire fired at Pass 25):
  §Citation Discipline section added above (see §Citation Discipline). Codifies
  Option C-Refined (hybrid semantic anchors + CI registry enforcement). Operationalizes
  ADR-0007 decision as the citation convention rule binding all monocle spec artifacts.
- NORMATIVE: Frontmatter version bumped v1.31.1 → v1.32.0 (minor version: discipline change).
  Timestamp updated to 2026-05-29T08:00:00Z.
- INFORMATIONAL: §Citation Discipline does NOT migrate existing active version-pin literals
  in this file or any sibling (legacy corpus). Migration is opportunistic per ADR-0007
  §Migration plan. The §Architecture Source Pin-Symmetry Convention §Canonical SS version
  table in this file still carries legacy active pointers (SS-conventions v1.30.2 and
  SS-deps-pin-manifest v1.1.17) — these are in-scope for opportunistic migration when that
  table is next touched for other reasons.
- SE-16d PASS: 2026-05-29T08:00:00Z — first entry in this §Trace chain for D-204 burst;
  no prior chain entry to compare.
- PG-5 self-check on newly added §Citation Discipline prose: zero active-pointer citations
  introduced (the new section describes the citation rules but does not itself cite versioned
  artifacts with active-pointer form). PASS.
- PG-3 directional qualifier self-check: §Citation Discipline uses "above" zero times and
  "below" zero times. PASS (no directional qualifiers introduced).

v1.31.1 changes (S-022 cycle ADR-0006 BONUS SS-forward-compat.md fix):

v1.28 changes (round-60.1 F-R60-1 + F-R60-corpus-sweep META rule codified):

- F-R60-1 RESOLVED (MED — sibling-propagation gap from R59.1 narrow sweep): R59.1 corrected
  2 stale "8 architecture spec files" sites in §Trace v1.22 and v1.19 entries, but claimed
  "comprehensive corpus-wide sweep." R60 independent consistency + adversary passes (stress-test
  confirmation) found 2 additional sibling sites missed by R59.1:
  (1) §Trace v1.18 entry (round-49 F-R48-adv-3 BC-HOOK-018 sweep): "Sweep of all 8 architecture
  spec files" → "Sweep of all 7 architecture spec files" (this file, prior line).
  (2) SS-forward-compatibility.md §Trace v1.2.9 entry (round-53.1 PG-4 expanded-scope sweep):
  "verified across all 8 architecture spec files" → "verified across all 7 architecture spec
  files" (SS-forward-compat bumped v1.2.12→v1.2.13). Root cause: partial-sweep anti-pattern —
  fixing only known sites without corpus-wide grep, then asserting comprehensiveness in §Trace.

- F-R60-corpus-sweep META rule CODIFIED (process-gap closure at META class level):
  §Corpus-Wide-Sweep Convention added above (see §Corpus-Wide-Sweep Convention above).
  5-step protocol: (1) grep `.factory/specs/` recursively for stale value pattern; (2) classify
  every match as historical-correct / stale-historical / active-stale; (3) fix all stale-historical
  and active-stale in same atomic commit; (4) emit per-class evidence count in §Trace; (5) self-grep
  after fix to confirm 0 stale matches remain. Anti-pattern documented: asserting "comprehensive
  sweep" when only known sites were corrected (R59.1 example).

  F-R60-corpus-sweep self-application (Step 5): corpus-wide grep after fix:
  `grep -rn "8 architecture spec files\|across 8 architecture\|all 8 architecture" .factory/specs/`
  Per-class classification of all 4 initial matches:
  - SS-conventions L1408 (§Trace v1.27): `"corrected from '8 architecture spec files' to '7'"` —
    historical-correct (quotes the old-wrong value in a description of the prior fix; protected by
    §Trace carve-out).
  - SS-conventions L1483 (§Trace v1.25): `"listed '(8 architecture spec files)'"` —
    historical-correct (quotes the old-wrong value being described; protected by §Trace carve-out).
  - SS-conventions L1703 (§Trace v1.18): `"Sweep of all 8 architecture spec files"` →
    stale-historical; FIXED in this burst → "7".
  - SS-forward-compat L333 (§Trace v1.2.9): `"verified across all 8 architecture spec files"` →
    stale-historical; FIXED in this burst → "7".
  Evidence: historical-correct: 2, stale-historical: 2, active-stale: 0 — 2 sites fixed.
  Post-fix self-grep: 0 stale matches remain (L1408 + L1483 are historical-correct descriptions
  of prior wrong values; they contain the phrase "from '8'" / "'(8'"  in quoted form, not as
  current assertions).

- PG-5 sweep for this burst: SS-*: 2 files modified (this file v1.27→v1.28;
  SS-forward-compatibility.md v1.2.12→v1.2.13); no new PG-5 violations introduced.

v1.27 changes (round-59.1 F-R59-adv-1 + F-R59-adv-2 + §Trace-Heading-Convention codified):

- F-R59-adv-1 RESOLVED (MED — §Trace prose count drift, two sites): §Trace v1.22 entry
  for the comprehensive PG-4 sweep (brief §-anchor sweep) used the phrase "7 architecture
  spec files" corrected in v1.25 §META-Rule Recipe Sibling-Pattern Convention body, but the
  propagation missed two §Trace prose sites: one in the v1.22 §Trace entry describing the
  brief-§-anchor sweep, and one in the v1.19 §Trace entry describing the §-heading-existence
  sweep. Both sites corrected from "8 architecture spec files" to "7 architecture spec files."
  Root cause: S-7.01 partial-fix irony pattern — v1.25 fixed the active rule body but did not
  propagate to §Trace prose entries from prior rounds that contained the same stale assertion.

- F-R59-adv-2 RESOLVED (MED + process-gap — §Trace heading drift, defense-layer bypass):
  SS-permissions-phase1.md §Trace heading was `## Trace` (missing `§` prefix), causing the
  PG-3-TRACE-NEW-ENTRY self-audit grep recipe `^## §Trace` to return zero matches on that
  file. The defense layer was silently bypassed. Fixed in SS-permissions-phase1.md v1.4:
  heading renamed `## Trace` → `## §Trace`. Process-gap closed at META class level:
  §Trace-Heading-Convention codified (this file §Trace-Heading-Convention), mandating `## §Trace`
  across all versioned spec artifacts. PG-3-TRACE-NEW-ENTRY recipe updated to heading-agnostic
  form `^## §?Trace` (belt-and-suspenders: convention requires `## §Trace`; recipe accepts
  both).

- Corpus-wide §Trace-heading audit: all 7 SS-*.md + dtu-assessment.md checked; 7/8 already
  compliant; 1/8 fixed (SS-permissions-phase1.md as described above).

- PG-5 sweep for this burst:
  SS-*: 2 files modified (SS-conventions-anti-patterns.md v1.26→v1.27; SS-permissions-phase1.md
  v1.3→v1.4); no new PG-5 violations introduced.
  brief: 1 file swept (D-041 read-only — no edits permitted), 0 violations.
  dtu-assessment: 1 file swept, 0 violations.
  vision: 1 file swept, 0 violations.
  ADR-N: 4 files swept, 0 violations.

v1.26 changes (round-58.1 F-R58-1-adv + F-R58-1-cons + PG-3-TRACE-NEW-ENTRY enhanced self-audit):

- F-R58-1-adv RESOLVED (MEDIUM adversary finding, codified out at META class level): PG-5
  Option B carve-out in §Historical-Anchor Framing Convention was scoped to `traces_to:`
  only. The same historical-anchor-by-semantic rationale applies to all frontmatter audit
  fields. Carve-out extended to enumerate: `traces_to:`, `producer:`, `inputs:`,
  `subsystems_affected:`, `supersedes:`, `superseded_by:`, and any other frontmatter field
  whose semantic is provenance or audit record. Closes sibling-field gap at META class level
  rather than per-site. Corpus sweep: ADR-0001 `producer:` field (`product-owner (extracted
  from brief v1.1)`) is the only non-`traces_to:` frontmatter field with a version citation
  in the corpus; confirmed covered by extended carve-out without content change required.

- F-R58-1-cons RESOLVED (LOW META catalog-growth blocker, codified out): PG-3-TRACE-NEW-ENTRY
  enhanced self-audit addendum codified. The existing post-write self-audit (grep on newly
  added lines) was insufficient: a version block header naming the version does not make
  bare L-numbers inside individual bullets compliant. Each bullet within a new version block
  is a separate unit of §Trace prose subject to PG-3. Addendum states: the self-audit MUST
  cover the full text of each newly authored version block; block-header-does-not-count is
  now explicit precedent. Closes the check missed-sites pattern from round-57.1.

- SS-permissions-phase1.md co-edit: bumped to v1.3, §Trace v1.2 bare L-numbers removed
  (§Context entry and §Consequences entry rewritten to position-free section names).

- PG-5 sweep for this burst:
  SS-*: 1 file modified (SS-permissions-phase1.md v1.2→v1.3); no new PG-5 violations
  introduced; all new prose in §Historical-Anchor Framing Convention uses section names only.
  brief: 1 file swept (D-041 read-only — no edits permitted).
  dtu-assessment: 1 file swept, 0 violations.
  vision: 1 file swept, 0 violations.
  ADR-N: 4 files swept, 0 new violations (ADR-0001 producer field now covered by
  extended carve-out; no content change required).

v1.25 changes (round-57.1 F-R57-2 PG-5 scope extension + sweep-evidence checklist + corpus sweep):

- F-R57-2 RESOLVED (LOW META catalog-growth blocker, codified out): PG-5 was silent on
  frontmatter `traces_to` fields, creating the same false-currency hazard as main-body prose
  at 4+ corpus sites. Architect adjudication: Option B — frontmatter `traces_to` exempted
  with explicit rationale. Rationale: `traces_to` is a machine-readable audit record; every
  entry is inherently historical by the field's semantic definition (records what was current
  at authoring time, not navigation pointers). D-042 already excludes brief version citations
  from cascade scope, confirming these cannot be held to current-pointer obligations.
  Carve-outs clause extended to include `traces_to` frontmatter, §Amendment History rows,
  §Closure Log entries, and §Provenance entries (all historical by section semantic).

- Sweep-evidence checklist codified: every PG-5 sweep claim MUST emit per-class counts
  (SS-*: N/M, brief: 1/M, dtu: 1/M, vision: 1/M, ADR-N: N/M). A sweep without per-class
  counts is incomplete. Closes the R56.1 assertion-without-evidence gap identified by
  the R57 adversary as a process-gap finding.

- PG-RECIPE-SCOPE SS-* count corrected: §META-Rule Recipe Sibling-Pattern Convention
  listed "(8 architecture spec files)" — actual count is 7 (verified by ls SS-*.md).
  Corrected to "(7 architecture spec files)". Pre-existing error from v1.22 codification.

- Comprehensive corpus PG-5 sweep at corrected scope (5 classes + ADR-N class newly added):
  SS-*: 7 files swept, 4 violations found, 4 fixed (SS-deps-pin-manifest v1.1.7→v1.1.8 —
  L27 + L140 brief v1.4 historical-anchor fix; SS-permissions-phase1 v1.1→v1.2 — L28 +
  L271 brief v1.3/v1.4.3 historical-anchor fix; remaining 5 SS files: 0 violations).
  brief: 1 file swept (D-041 read-only — no edits permitted).
  dtu-assessment: 1 file swept, 0 violations (existing hits are §Trace entries — carve-out).
  vision: 1 file swept, 0 violations (existing hits are in §Closure Log / §Provenance —
  carve-out per extended clause).
  ADR-N: 4 files swept, 4 violations found, 4 fixed (ADR-0004 v1.0.2→v1.0.3 — L175 F-R57-1
  fix; ADR-0001 v1.0.1→v1.0.2 — L71 + L72 + L83 brief v1.1/v1.2 historical-anchor fix;
  ADR-0002 0 violations; ADR-0003 0 violations).

v1.24 changes (round-56.1 PG-5 codification + F-R56-2 sibling sites 3+4 + D-042 cascade):

- PG-5 §Historical-Anchor Framing Convention codified. Cross-artifact version citations in
  main-body prose MUST use current-pointer OR historical-anchor OR version-free form. Bare
  `<artifact>.md vX.Y §<heading>` that is neither current nor explicitly historical FAILS
  PG-5. Pre-commit sweep recipe enumerates all 5 versioned spec artifact classes per
  PG-RECIPE-SCOPE. PG-4 relationship list updated to include PG-5 as sixth META-pattern class.

- F-R56-2 RESOLVED (MEDIUM content — 2 sites in this file):
  (1) §R-001 Monitoring Workflow §Purpose cited `brief v1.4.6 §Competitive Positioning`.
  Neither current (brief at v1.4.23) nor explicitly historical. §Competitive Positioning is
  a stable brief section; version qualifier adds no navigational value. Fix: option (c) —
  version qualifier dropped. Now reads `per brief §Competitive Positioning`.
  (2) §Quarterly Keyword Maintainer Review cited "as of brief v1.4.6" without historical
  qualifier. This citation marks when the keyword patterns were authored — an intentional
  audit record. Fix: option (a) — historical-anchor framing added: "as of brief v1.4.6 at
  spec authoring time".

- D-042 CITATION REFRESH: PG-1 §Schema-Fact Citation Convention example citations updated:
  dtu-assessment.md v1.6 → v1.7 and SS-core-types-and-abi.md v1.2.7 → v1.2.8, both files
  bumped in this burst (F-R56-2 PG-5 fixes).

v1.23 changes (round-54.1 F-R54-adv-2 PG-4 scope clause + PG-D042-WITHIN-FILE codification):

- F-R54-adv-2 RESOLVED (LOW process-gap — adversary finding R54): PG-4 rule statement was
  unscoped ("Cross-document `§<Name>` references MUST point to an actual heading"), but
  PG-RECIPE-SCOPE limits PG-4 enforcement to versioned spec artifacts. Citations in
  non-versioned project documentation (e.g., `CLAUDE.md`) to enumerated list items
  (e.g., `§Rule 1`, `§Canonical Principle`) are not mis-anchors under PG-RECIPE-SCOPE
  scope but would be flagged under the unscoped rule statement. Scope clause added
  immediately after the Rule paragraph in §Section-Anchor Citation Convention (PG-4):
  enforcement limited to versioned spec artifacts (per PG-RECIPE-SCOPE); citations to
  non-versioned project documentation where the citation target is an enumerated item
  within a list are exempt provided the item is unambiguous. This closes the rule-statement
  vs PG-RECIPE-SCOPE alignment gap without changing actual enforcement behavior.

- PG-D042-WITHIN-FILE CODIFIED (10th META-pattern recurrence, first within-file-partial
  cascade variant): When a sibling-file bump triggers a D-042 cascade, the architect MUST
  run the D-042 grep at the file level (not selectively by section) for every file citing
  the bumped source. Each hit must be classified as a historical pinpoint (preserve) or a
  current-pointer (update). Selective within-file updates are FORBIDDEN. Sub-rule added to
  §Schema-Fact Citation Convention after the PG-D042-DTU-SCOPE block, before the schema-fact
  grep pattern. Root cause: R53.1 burst updated §Verdict in SS-forward-compatibility.md to
  cite SS-daemon-lifecycle.md v1.0.7 but ran the within-file scan selectively, missing the
  FC-01 and FC-06 table col 4 cells in the same file that still cited v1.0.6.

- SS-forward-compatibility.md bumped to v1.2.10 as co-edit partner (F-R54-adv-1 cascade fix).

v1.22 changes (round-53.1 F-R53-adv-2 PG-RECIPE-SCOPE codification + PG-4 sibling-pattern expansion):

- F-R53-adv-2 RESOLVED (MEDIUM process-gap — adversary finding R53): PG-4 pre-commit
  grep recipe was scoped to `SS-[a-z-]+` filenames only, silently excluding
  `product-brief.md`, `dtu-assessment.md`, `domain-monocle-vision-synthesis.md`, and
  `ADR-N-*.md`. Same root-cause class as D-042's pre-R51.1 scope hole (§PG-D042-DTU-SCOPE)
  and the META-META structural pattern confirmed across 9 recurrences. Recipe expanded with
  4 sibling patterns (brief, dtu-assessment, vision, ADR-N). Each sibling pattern verified
  against actual citation patterns in the corpus.

- PG-RECIPE-SCOPE META-META rule codified: new §META-Rule Recipe Sibling-Pattern Convention
  added. When codifying any new META-rule with a sweep recipe, the recipe MUST include
  sibling patterns for all versioned spec artifacts at codification time — not as a follow-up
  burst. Self-audit at codification time specified. This closes the 9th recurrence of the
  SS-only recipe scope hole at the META-META level.

- PG-4 §Relationship updated to reference PG-RECIPE-SCOPE as the 5th META-pattern class.

- Comprehensive brief-§-anchor sweep (expanded PG-4 scope): All brief §-anchor citations
  verified across 7 architecture spec files + ADR-0004. Mis-anchors corrected in:
  SS-daemon-lifecycle.md v1.0.6 → v1.0.7 (F-R53-adv-1; §Item P3-1 fix),
  SS-forward-compatibility.md v1.2.8 → v1.2.9 (F-R53-adv-3; 5 brief §-anchor fixes),
  SS-core-types-and-abi.md v1.2.6 → v1.2.7 (F-R53-adv-3 + F-R53-adv-4; 3 brief
  §-anchor fixes), ADR-0004 v1.0.1 → v1.0.2 (F-R53-adv-5; brief §Scope fix).

v1.21 changes (round-52.2 PG-D042-DTU-SCOPE codification + D-042 example citation refresh):

- PG-D042-DTU-SCOPE CODIFIED (8th META-pattern recurrence root-cause): D-042 grep recipe
  extended with two sibling patterns for non-SS-prefixed versioned spec artifacts. The
  existing primary (`grep -rn "SS-[a-z-]*\.md v"`) and secondary (`grep -rn "SS-[a-z-]*\.md.*v[0-9]"`)
  patterns matched only SS-prefixed filenames, silently excluding `dtu-assessment.md` and
  `domain-monocle-vision-synthesis.md` citations. Root cause: F-R52R-1 confirmed a D-042
  incomplete cascade — `dtu-assessment.md v1.4` citations persisted in
  SS-forward-compatibility.md after dtu-assessment.md was bumped to v1.5 in round-52.1.
  New sibling patterns added to §Schema-Fact Citation Convention §D-042 CANONICAL SCOPE:
  (1) `grep -rn "dtu-assessment\.md v" .factory/specs/`
  (2) `grep -rn "domain-monocle-vision[^ ]*\.md v" .factory/specs/`
  Product-brief.md version citations explicitly excluded from the recipe per Q-3 standing
  disposition (D-041 routing; human-controlled). Recurrence count now at 8 confirmed events.

- D-042 example citation refresh: §Schema-Fact Citation Convention §Correct form example
  updated from `dtu-assessment.md v1.4` → `v1.5` and `SS-core-types-and-abi.md v1.2.5`
  → `v1.2.6`. The example uses real current versions as the canonical illustration; both
  were stale. This citation was surfaced by the new `dtu-assessment\.md v` sibling grep
  pattern proving the rule works on its own example.

- SS-forward-compatibility.md bumped to v1.2.8 as co-edit partner (F-R52R-1 cascade fix
  and F-R52R-2 §Trace ordering correction).

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
  verified across 7 architecture spec files + dtu-assessment.md. Additional mis-anchors found
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
  SS-engine-module.md; no sibling sites required treatment. Sweep of all 7 architecture
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

**§Trace v1.29** (2026-05-17T11:00:00Z) — Template compliance Dispatch 1:
- NORMATIVE: `section` field corrected from `"conventions"` → `"conventions-anti-patterns"`
  (full, unambiguous section name matching filename per template convention).
- NORMATIVE: `subsystem: cross-cutting` added (conventions apply to all subsystems; cross-cutting
  file per ARCH-INDEX.md §Cross-Cutting Files).
- NORMATIVE: `traces_to` corrected to `architecture/ARCH-INDEX.md` (was long trace-history
  string; ARCH-INDEX.md created in this dispatch).
- NORMATIVE: `timestamp` bumped to 2026-05-17T11:00:00Z (>= chain high-water 2026-05-17T10:30:00Z;
  SE-16d PASS).
- INFORMATIONAL: `document_type` already `architecture-section` — no change required (audit §9
  confirmed PASS for conventions document_type).
- INFORMATIONAL: Version bump 1.28 → 1.29 records structural fix; no content changes.
- Audit reference: `.factory/plans/template-compliance-audit-r1.md` §9 (SS-conventions).
- SE-17g classification: all citations above NORMATIVE or INFORMATIONAL as labeled.
- SE-16d PASS: UTC ISO-8601 Z form, 2026-05-17T11:00:00Z >= chain high-water 2026-05-17T10:30:00Z.

**§Trace v1.30.0** (2026-05-20T21:00:00Z) — cargo-deny 0.19 schema migration ratified; 4 ambiguities resolved from S-001 fix PR #2:
- NORMATIVE (A1): §deny.toml configuration sample migrated from cargo-deny 0.16 schema to 0.19 schema. Removed `[advisories]` fields `vulnerability`, `unmaintained`, `yanked`, `notice` (dropped in 0.17 breaking schema migration; cargo-deny 0.19 denies all RUSTSEC vulnerabilities by default). Removed `[licenses]` fields `unlicensed`, `copyleft`, `allow-osi-fsf-free`, `exceptions` (dropped in same 0.17 migration). Retained `[licenses] allow`, `confidence-threshold`, `[advisories] db-path`, `db-urls`, `ignore`. Source: devops-engineer S-001 fix PR #2 commits 287c109 + 53b5d6e.
- NORMATIVE (A2): `[bans] wildcards = "deny"` → `wildcards = "allow"`. In cargo-deny 0.19, intra-workspace `path = "../monocle-*"` deps are classified as wildcard version selectors; `wildcards = "deny"` would block all intra-workspace path deps. The original anti-pattern protection (unbounded registry version ranges) is fully enforced by `SS-deps-pin-manifest.md` EXACT-pin/caret-pin policy + clippy + code review. No enforcement gap. Rationale added inline to deny.toml sample.
- NORMATIVE (A3): `[bans] skip` entry added for `getrandom`. `rand =0.8.6` EXACT pin (closed RUSTSEC-2026-0007) forces `getrandom 0.2.x`; `tempfile 3` + `prost-build 0.14` require `getrandom 0.4.x`. Known forced duplicate; resolution requires `rand 0.9` migration (deferred to post-Wave-2 per `SS-deps-pin-manifest.md` rationale).
- NORMATIVE (A4, 4th ambiguity not originally surfaced): `[bans] skip` entry added for `wit-bindgen`. Cascade of getrandom 0.4 transitive path: `wasip2 v1.0` + `wasip3 v0.4` require incompatible wit-bindgen versions (0.57.1 vs 0.51.0). No direct monocle dependency; purely transitive through tempfile. Resolution coupled to A3 (rand 0.9 migration). Devops correctly included this skip in PR #2; ratified in scope.
- NORMATIVE: `[licenses]` rationale paragraph updated to remove reference to removed `copyleft = "warn"` field (no longer in 0.19 schema; text now refers to explicit allow-list enforcement instead).
- NORMATIVE: Canonical SS version table (§Architecture Source Pin-Symmetry Convention) must advance `SS-conventions-anti-patterns.md` self-pin from v1.29.5 → v1.30.0 (SE-17f recursive self-revalidation obligation).
- NORMATIVE: `phase` frontmatter updated from `pre-phase-1-architecture` → `phase-3` (document modified during Phase 3; phase field tracks modification phase, not original authoring phase).
- SE-22 v2 sibling-sweep: Consumers of §deny.toml content: S-001 (references §CI Wiring step list — unchanged; step 5 command string unchanged); nfr-catalog.md (no deny.toml content reference); PRD §CI (references step list — unchanged); ARCH-INDEX §Cross-Cutting (version pin only). Zero cascade write-backs required. Canonical SS version table in §Architecture Source Pin-Symmetry Convention requires self-pin update (see NORMATIVE above).
- SE-16d PASS: 2026-05-20T21:00:00Z > chain high-water 2026-05-18T19:30:00Z (v1.29.5). ARITHMETICALLY TRUE.
- Source commits: PR #2 commits 287c109 (cargo-deny 0.19 schema) + 53b5d6e (bytes pin cascade update) on branch `s-001-workspace-ci`.

**§Trace v1.30.1** (2026-05-20T22:00:00Z) — §deny.toml GitHub Actions YAML example corrected; no semantic content change:
- NORMATIVE: §GitHub Actions wiring YAML example corrected. Original v1.30.0 form (`command: check` + `arguments: --all-features --workspace licenses bans advisories sources`) was malformed: `--all-features` and `--workspace` are cargo-deny global CLI flags (pre-command), while `licenses bans advisories sources` are positional args to the `check` subcommand (post-command). The `arguments:` input to `cargo-deny-action` is inserted PRE-command per `action.yml` SHA `6c8f9facfa5047ec02d8485b6bf52b587b7777d1`; mixing pre-command flags and post-command positional args in the same slot causes cargo-deny to parse `licenses` as the subcommand name — error: "unrecognized subcommand 'licenses'". Corrected form: `command: check all` (runs all 4 categories: advisories, bans, licenses, sources) + `arguments: --workspace --all-features` (scans all workspace members + feature-gated transitive deps). Final invocation: `cargo-deny --log-level warn --manifest-path ./Cargo.toml --workspace --all-features check all`. Source: PR #2 Round-1 adversary report (`.factory/plans/adversary-pass-PR2-pre-merge.md` MED-1).
- NORMATIVE: `name:` field corrected from `cargo-deny` → `cargo deny check` to match the corrected action form for clarity.
- SE-22 v2 sibling-sweep: scanned PRD, ARCH-INDEX, L2-INDEX, BC-INDEX, VP-INDEX, all 17 story files for `--all-features --workspace licenses` or `command: check` + `arguments:.*licenses`. ZERO citations of the malformed YAML form found outside this file. Cascade write-backs: NONE required.
- SE-16d PASS: 2026-05-20T22:00:00Z > chain high-water 2026-05-20T21:00:00Z (v1.30.0). ARITHMETICALLY TRUE.
- Source: adversary finding MED-1 from `.factory/plans/adversary-pass-PR2-pre-merge.md`.

**§Trace v1.31.0** (2026-05-27T00:00:00Z) — ADR-0006 ratification: §Non-Exhaustive Structs with Public Constructors section added:
- NORMATIVE (F-S022-ADV2-MED-003 HIGH — architect adjudication): Added §Non-Exhaustive Structs with Public Constructors section documenting the three criteria under which `pub fn new(...)` positional constructors are permitted on `#[non_exhaustive]` structs, and the breaking-change discipline for adding required fields.
- Triggered by adversarial Pass 2 finding F-S022-ADV2-MED-003 which routed to architect via durable task register item ADV-W5GATE-MED-003.
- Ratifies the 5 hook event inner struct constructors (`SessionStartEvent`, `UserPromptSubmitEvent`, `PreToolUseEvent`, `NotificationEvent`, `StopEvent`) and `HookEventRecord` constructor as meeting all three criteria (internal workspace, external protocol anchor, all required fields).
- See ADR-0006 for full rationale and authoritative struct table.
- SE-16d PASS: 2026-05-27T00:00:00Z > chain high-water 2026-05-20T23:00:00Z (v1.30.2). ARITHMETICALLY TRUE.

**§Trace v1.30.2** (2026-05-20T23:00:00Z) — §clippy.toml disallowed_methods extended with `std::println` + `std::eprintln`. Closes process-gap surfaced by adversary Round 2 on PR #2 (S-001 post-merge fix):
- NORMATIVE (CR-009 process-gap): Added `{ path = "std::println", reason = "..." }` and `{ path = "std::eprintln", reason = "..." }` to the `disallowed_methods` list in the §Clippy `disallowed_methods` Configuration section. The §Convention Checklist banned `println!`/`eprintln!` in production code in prose, but the clippy.toml enforcement block omitted these entries — making the prose rule unenforced at CI. This converts the prose-only rule into a hard CI lint.
- NORMATIVE: version bump 1.30.1 → 1.30.2 records disallowed_methods extension; no other content changed.
- SE-22 v2 sibling-sweep: scanned PRD, ARCH-INDEX, VP-INDEX, BC-INDEX, all 17 story files, CLAUDE.md, and all spec files for `disallowed_methods`, `std::println`, `std::eprintln`, `clippy.toml` consumer references. S-001 contains two `println!` occurrences in stub scaffold code (intentional compile stubs — not production code enforcement targets); S-002 contains one prose prohibition already aligned to the new enforcement. CLAUDE.md contains one aligned prose prohibition. Zero cascade write-backs required.
- SE-16d PASS: 2026-05-20T23:00:00Z > chain high-water 2026-05-20T22:00:00Z (v1.30.1). ARITHMETICALLY TRUE.
- Source: adversary Round 2 finding CR-009 from `.factory/plans/adversary-pass-PR2-round-2.md`.

**§Trace v1.29.5** (2026-05-18T19:30:00Z) — R17D F-R118-5 closure: Architecture Source Pin-Symmetry Convention added to §BC-INDEX Conventions:
- NORMATIVE (F-R118-5 HIGH): Added `### Architecture Source Pin-Symmetry Convention (F-R117-3, SE-17e)` subsection to §BC-INDEX Conventions section. BC-INDEX v1.10 (commit 9a02f5a, R16C) codified this convention in BC-INDEX §Conventions with parenthetical "(add at next architect dispatch)". This is that dispatch.
- NORMATIVE: Canonical SS version table updated — `SS-conventions-anti-patterns.md` self-pin advanced to v1.29.5 (reflects this burst's version bump).
- NORMATIVE: version bump 1.29.4 → 1.29.5 records addition of Pin-Symmetry subsection.
- SE-22 cycle-4 sweep (NORMATIVE vs INFORMATIONAL):
  - `BC-INDEX v1.1+` at line 1481 (pre-edit numbering): INFORMATIONAL — version-free floor marker for historical renumbering event boundary; not a normative pin requiring update.
  - No stale NORMATIVE pins to BC-INDEX, PRD, product-brief, L2-INDEX, ARCH-INDEX, or VP-INDEX found in the body of this file. 0 stale NORMATIVE pins caught beyond F-R118-5.
- SE-17a literal evidence: `grep -n "Pin-Symmetry\|pin-symmetry" .factory/specs/architecture/SS-conventions-anti-patterns.md` → new subsection at line 1490 (post-edit; per scoped-awk D-116 protocol).
- SE-17c BEFORE: 3 subsections in §BC-INDEX Conventions (EC Namespace, Test Name, Anchor Parenthetical).
- SE-17c AFTER: 4 subsections in §BC-INDEX Conventions (EC Namespace, Test Name, Anchor Parenthetical, Architecture Source Pin-Symmetry).
- SE-17d L-number revalidation: subsection heading verified present post-edit; §Trace appended at descending timestamp order (v1.29.5 inserted above v1.29.4, preserving descending sort in §Trace).
- SE-17f recursive self-revalidation: canonical SS version table in the new subsection lists `SS-conventions-anti-patterns.md v1.29.5` — matches this burst's version bump. Self-consistent.
- SE-17g NORMATIVE vs INFORMATIONAL: all changes above classified NORMATIVE. The BC-INDEX v1.1+ floor reference is INFORMATIONAL (unchanged).
- SE-16d PASS: 2026-05-18T19:30:00Z > chain high-water 2026-05-18T11:00:00Z (VP-INDEX v1.13 at 19:00:00Z; this burst at 19:30:00Z > 19:00:00Z — monotonic).
- Source of truth: BC-INDEX v1.10 §Conventions "Architecture Source Pin-Symmetry Convention (F-R117-3, SE-17e)".

**§Trace v1.29.4** (2026-05-18T11:00:00Z) — F-R114-1 + F-R114-2 D-042 back-cascade codification and example-pin refresh:
- NORMATIVE (F-R114-2 LOW): §Schema-Fact Citation Convention example pins refreshed:
  `dtu-assessment.md v1.7` → `v1.7.5`; `SS-core-types-and-abi.md v1.2.8` → `v1.2.13` in
  the **Correct form** example. Historical §Trace references to these versions are immutable
  pinpoints and are preserved unchanged.
- NORMATIVE (F-R114-1 MED — D-042 back-cascade obligation): Added `PG-D042-BACK-CASCADE`
  sub-rule to §D-042 CANONICAL SCOPE block. The rule codifies that D-042 sweeps are
  bidirectional: forward (who cites X after X is bumped?) AND backward (what does X cite,
  and are those citations stale?). Root cause: SS-forward-compatibility.md carried stale
  `dtu-assessment.md v1.7` and `SS-core-types-and-abi.md v1.2.8` citations even after those
  files advanced to v1.7.5 and v1.2.13.
- SE-17c BEFORE: `dtu-assessment.md v1.7` (1 site), `SS-core-types-and-abi.md v1.2.8` (1 site).
- SE-17c AFTER: `dtu-assessment.md v1.7.5` (1 site), `SS-core-types-and-abi.md v1.2.13` (1 site).
- SE-16d PASS: 2026-05-18T11:00:00Z > chain high-water 2026-05-18T06:00:00Z (monotonic).

**§Trace v1.29.3** (2026-05-18T06:00:00Z) — F-R110-18 BC-INDEX conventions cross-listing:
- F-R110-18 LOW RESOLVED: BC-INDEX §Conventions defines EC namespace, test name, and anchor parenthetical conventions that are relevant to all BC authoring. These conventions were not cross-listed in SS-conventions-anti-patterns (the canonical conventions doc). Added §BC-INDEX Conventions section with EC Namespace Convention (F-R109-17), Test Name Convention (F-R109-21), and Anchor Parenthetical Non-Contradiction (PG-5, F-R110-16) as cross-references to the BC-INDEX §Conventions canonical source.
- NORMATIVE: version bump 1.29.2 → 1.29.3 records addition of §BC-INDEX Conventions section; no existing content changed.
- SE-16d PASS: 2026-05-18T06:00:00Z > prior version timestamp (v1.29.2). ARITHMETICALLY TRUE.

**§Trace v1.31.1** (2026-05-29T00:00:00Z) — F-S025-ADV22-MED-001 sweep-wider: SS-forward-compat.md → SS-forward-compatibility.md (line 1043):
- F-S025-ADV22-MED-001 MED RESOLVED (sweep-wider pass): SS-conventions-anti-patterns.md line 1043 cited `SS-forward-compat.md` (abbreviated, non-existent filename). Canonical file is `SS-forward-compatibility.md`. Active spec reference — fixed per CLAUDE.md Principle 4.
- 1 site in active spec body updated: `SS-forward-compat.md §Cross-Phase Decisions` → `SS-forward-compatibility.md §Cross-Phase Decisions`.
- Historical §Trace entries referencing `SS-forward-compat` preserved as historical narrative (not updated).
- NORMATIVE: version bump 1.31.0 → 1.31.1.
- SE-16d PASS: 2026-05-29T00:00:00Z > 2026-05-18T11:00:00Z (prior §Trace v1.31.0 timestamp). PASS.
