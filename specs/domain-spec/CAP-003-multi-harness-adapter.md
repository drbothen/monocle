---
document_type: domain-spec-section
level: L2
section: "CAP-003 Multi-Harness Adapter Surface"
capability: CAP-003
version: "1.0"
status: active
producer: vsdd-factory:business-analyst
timestamp: 2026-05-17T14:00:00Z
phase: 1a
inputs:
  - product-brief.md
  - research/domain-monocle-vision-synthesis.md
input-hash: "55f92f9"
traces_to: L2-INDEX.md
subsystem: SS-03
bcs:
  - BC-2.03.001
  - BC-2.03.002
  - BC-2.03.003
  - BC-2.03.004
---

# CAP-003: Multi-Harness Adapter Surface

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`. This section
> describes the Multi-Harness Adapter Surface domain capability at the
> problem-domain level. Implementation contracts live in
> `behavioral-contracts/ss-03/`.

## Capability Statement

CAP-003 covers the domain obligation to abstract AI coding harness identity
behind a stable trait boundary, such that monocle can observe Claude Code
sessions in Phase 1 and add CodeMachine (Phase 4) and future harnesses without
changing the daemon's core logic. The same abstraction pattern applies to
factory-pattern workflow adapters: monocle must support vsdd-factory in Phase 1
and third-party factory adapters in Phase 3 without hardcoding either.

**Anchor justification:** CAP-003 covers this scope because the product brief
§Phase 1 Scope explicitly names `ClaudeCodeModule` as the built-in
`EngineModule` implementation and `VsddFactoryAdapter` as the built-in
`FactoryAdapter` implementation, while §Phase 4 names `CodeMachineModule` as
the second built-in — establishing that the abstraction must accommodate
multiple implementations from Phase 1 day one. Vision §Key Abstractions
defines the `EngineModule` trait as the "multi-harness gene from codemachine-cli"
and the `FactoryAdapter` trait as the workflow-plane plugin contract.

## Domain Entities

### EngineModule (trait)

The abstract contract that any AI coding harness must fulfill to participate in
monocle's session roster. The trait establishes the vocabulary monocle uses to
talk about harnesses without coupling to any specific harness.

| Method | I/O | Description |
|--------|-----|-------------|
| `id()` | → stable string | Unique harness identifier (e.g., "claude-code") |
| `metadata()` | → EngineMetadata | Human-readable display name, icon, config path, hook schema |
| `detect(proc)` | proc snapshot → bool | Decide if a process belongs to this harness; NO I/O allowed |
| `enrich(proc)` | proc snapshot → EnrichedSession | Add token counts, cost, phase tag from hook history; may do I/O |
| `on_hook(event)` | HookEvent → HookResponse | Handle a hook event and return the response to send back to the harness |

The `detect()` method's no-I/O rule is a domain invariant (DI-006), not an
implementation constraint: if `detect()` performs I/O, it blocks the process-
scan loop, which must complete within a render frame budget. The domain says
"detection is cheap identification"; enrichment is the expensive step.

### EngineMetadata

Descriptive data about a harness, surfaced in the sessions panel header.

| Attribute | Type | Description |
|-----------|------|-------------|
| display_name | string | Human-readable harness name (e.g., "Claude Code") |
| icon | char | Single character rendered in the sessions panel row |
| config_path | path | Root of harness configuration files (e.g., `~/.claude/`) |
| hook_schema | string | JSON schema identifier for validating hook payloads |

### ClaudeCodeModule

The Phase 1 built-in implementation of `EngineModule` for Claude Code. Identifies
Claude Code subprocesses by basename and enriches them with token counts, cost,
and phase tag extracted from hook event history.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | "claude-code" | Stable identifier |
| detection rule | strict-basename match | Process is Claude Code iff its executable basename is "claude" |
| hook_paths() | → list of paths | Paths where Claude Code hook scripts must be written (inherent method, not trait) |
| spawn() | → SessionHandle | Launch a managed Claude Code subprocess (inherent method) |
| preflight() | → Result | Verify Claude Code is installed and at a compatible version (inherent method) |

The strict-basename-match detection rule is a domain decision, not an
implementation shortcut: it mirrors the gene-source any-context
`lazyclaude` detection heuristic (brief §ClaudeCodeModule) and ensures
monocle does not misidentify processes whose path happens to contain "claude".

### HomeUnresolvable (error entity)

The error that `ClaudeCodeModule` returns when the home directory cannot be
resolved at hook-path derivation time. This is a domain entity because the
harness's config path is home-relative, and an unresolvable home means monocle
cannot configure Claude Code hook scripts — a blocking startup condition.

| Attribute | Type | Description |
|-----------|------|-------------|
| kind | HomeUnresolvable | Discriminant for this error class |
| message | string | Human-readable description for status bar display |

### FactoryAdapter (trait)

The abstract contract that any factory-pattern workflow system must fulfill to
appear in monocle's Workflow panel. The trait establishes the vocabulary monocle
uses to talk about factory state without coupling to vsdd-factory's internal
schema.

| Method | I/O | Description |
|--------|-----|-------------|
| `id()` | → stable string | Unique factory identifier (e.g., "vsdd-factory") |
| `detect(project_root)` | path → bool | Decide if a project uses this factory; must be synchronous and cheap |
| `read_state(project_root)` | path → FactoryState | Parse the factory state from the project directory |
| `on_change(project_root, changed)` | path × path → FactoryState | Respond to a file-watcher event; re-parse and return updated state |

The canonical detection signal for vsdd-factory is `document_type: pipeline-state`
in `.factory/STATE.md`. This is a domain rule: monocle does not parse
arbitrary YAML structures — it looks for the discriminator field that marks a
file as a vsdd-factory pipeline state document.

### VsddFactoryAdapter

The Phase 1 built-in implementation of `FactoryAdapter` for vsdd-factory
projects. Reads `.factory/STATE.md`, parses the YAML frontmatter and Phase
Progress table, and returns a `FactoryState`.

| Attribute | Type | Description |
|-----------|------|-------------|
| id | "vsdd-factory" | Stable identifier |
| detection signal | `document_type: pipeline-state` in `.factory/STATE.md` | Canonical discriminator |
| multi-repo signal | `.factory-project/` directory present alongside `.factory/` | Secondary signal |

In Phase 1, `VsddFactoryAdapter` is statically bundled in the daemon binary
(not loaded via WASM). Phase 3 promotes it to a WASM-loadable plugin using the
same `FactoryAdapter` ABI, without changing the adapter's domain behavior.

## Domain Processes

### P8: Harness Detection

1. The daemon's process scanner receives a new process event.
2. The scanner calls `detect(proc)` on each registered `EngineModule` in order.
3. The first `EngineModule` that returns `true` claims the process.
4. If no module claims it, the process is ignored.
5. The claimed process is enriched asynchronously via `enrich(proc)`.

### P9: Hook Routing

1. Daemon receives a hook POST on `/hooks/<type>`.
2. Daemon identifies the session from the hook payload's session ID.
3. Daemon looks up the `EngineModule` that owns the session.
4. Daemon calls `on_hook(event)` on the owning module.
5. Module returns a `HookResponse` that the daemon sends back to the harness.

### P10: Factory State Detection

1. Daemon's session enrichment identifies a session's working directory.
2. Daemon calls `detect(project_root)` on each registered `FactoryAdapter`.
3. The first adapter that returns `true` claims the project.
4. Daemon calls `read_state(project_root)` to populate the Workflow panel.
5. File watcher notifies daemon of STATE.md changes; daemon calls
   `on_change(project_root, changed)` on the owning adapter.

### P11: Home Directory Error Handling

1. `ClaudeCodeModule::hook_paths()` is called at daemon startup.
2. If the home directory cannot be resolved, the method returns
   `Err(HomeUnresolvable)`.
3. Daemon surfaces the error in the status bar and prevents hook-script
   configuration.
4. The daemon continues to run; sessions that have pre-configured hook scripts
   can still fire events.

## Domain Invariants

### DI-006: Stateless Detection Invariant

Every `EngineModule` implementation MUST be stateless with respect to process
detection. The `detect()` method MUST NOT perform I/O and MUST NOT mutate
shared state.

**Justification:** DI-006 is a business invariant because the process scanner
runs on every process event and must complete within the TUI render budget. An
I/O-performing `detect()` would block the hot path and cause visible latency
in the sessions panel. The domain says: detection is cheap identification;
enrichment (which may do I/O) is the separate step. Source: vision §Key
Abstractions EngineModule trait, `detect` method doc "Must be cheap (no I/O)".

### DI-007: Observe-Only Invariant

monocle MUST NOT write to any file owned by a harness or factory workflow system.

**Justification:** DI-007 is a business invariant because the product's value
proposition is observation without interference. Writing to Claude Code's
config files or vsdd-factory's STATE.md would turn monocle from an observer
into an actor in the harness's lifecycle, violating the "observe-only for state"
principle stated in the Vision Statement and enumerated in brief §Out of Scope
("Does NOT write STATE.md", "Does NOT route LLM API requests"). Source: vision
§Vision Statement, vision §Explicit Non-Goals.

## BC Cross-References

All 4 BCs in SS-03 operationalize CAP-003. See `behavioral-contracts/BC-INDEX.md`
§SS-03 for the full list.

| BC ID | Title | Operationalizes |
|-------|-------|-----------------|
| BC-2.03.001 | EngineModule Trait Definition | EngineModule entity |
| BC-2.03.002 | ClaudeCodeModule Implementation (Strict-Basename Detect) | ClaudeCodeModule entity, DI-006 |
| BC-2.03.003 | HomeUnresolvable Error Contract | HomeUnresolvable entity, P11 |
| BC-2.03.004 | ClaudeCodeModule Inherent Methods (hook_paths, spawn, preflight) | ClaudeCodeModule entity, P8 |
