# Pass 1 — Scoped Project Discovery: vsdd-factory

## Self-Reference Note

This ingest IS being orchestrated by an orchestrator-style agent FROM a (different-version) install of this same vsdd-factory plugin. The reference tree being analyzed at `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/` is the `develop`-branch source; the dispatcher running monocle's own hooks is a marketplace-cached binary built from an earlier release tag. Source-vs-instance separation is intentional: the analysis here is on the v1.0.0-rc.16 source state, not the running engine.

## Repo Identification

| Field | Value |
|-------|-------|
| Path (absolute) | `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/` |
| Repo | `drbothen/vsdd-factory` |
| Branch | `develop` |
| HEAD | `99d2431529fa2839bc2b04702ad32404d06d0f99` |
| Plugin manifest version | `1.0.0-rc.16` (per `plugins/vsdd-factory/.claude-plugin/plugin.json:5`) |
| License | MIT |
| Total files | 1,572 (per user statement) |
| Total size | 232 MB on disk |
| Top languages | Rust 2.6 MB, JavaScript 33 KB, PowerShell 17 KB, HTML 8 KB, Python 4 KB, TypeScript 3 KB, plus ~1 MB Shell |

## Purpose

vsdd-factory is a Claude Code plugin that installs the **Verified Spec-Driven Development (VSDD)** pipeline. It's a "dark factory" for software: specs → tests → implementation → adversarial review → holdout eval → formal hardening → convergence → release. The pipeline is driven by **workflow files** (Lobster `.lobster` YAML), dispatched and policed by a **native Rust WASM-host dispatcher binary** that routes Claude Code hook events to plugin hooks (compiled WASM + a legacy-bash-adapter for unported bash hooks).

## Scope Boundary

This ingest is **SCOPED** to the genes monocle needs to inherit/detect:

### IN-SCOPE (deepened to NITPICK)

1. **Workflow format** — the `.lobster` YAML schema and its parser
2. **Factory dispatcher** — Rust binary + hooks-registry.toml protocol + Claude Code hook events
3. **Factory project pattern** — how monocle can DETECT a project uses vsdd-factory (or a compatible factory)
4. **STATE.md schema** — frontmatter + body sections, mutation patterns, recovery semantics
5. **Workflow execution primitives** — the small set of skills (`run-phase`, `next-step`, `validate-workflow`, `factory-dashboard`, `recover-state`, `check-state-health`, `state-update`) and the orchestrator agent that drive workflow execution

### OUT-OF-SCOPE (deliberately not deepened)

- `agents/` — 33+ specialist agents (only `orchestrator/orchestrator.md` read for context)
- `skills/` — 116 skills total; only the 7 listed above read
- `rules/` — orchestrator rule files (9 files)
- `fixtures/` — test fixtures
- `tests/` — bats test suites (520+ tests)
- `docs/` — extended methodology docs
- `hook-plugins/` — individual hook plugin sources (28+ Rust crates; we recorded the dispatcher's role but not each plugin's internal logic)
- `crates/` other than `factory-dispatcher/` (12 other crates: sink-*, hook-sdk, hook-sdk-macros, hook-plugins/, vsdd-context-resolvers)
- All language-specific tooling (Cargo.toml, scripts/, ci/, benches/) except where directly relevant
- `plugins/vsdd-factory/templates/` — 109 templates total; only 6 STATE/cycle templates read
- `plugins/vsdd-factory/hooks/*.sh` — 27 enforcement bash hooks; only `hooks.json.template` and registry entries summarized (no per-hook deep-dive)

## In-Scope File Manifest

### Category 1: Workflow Format (`.lobster` files)

| Absolute path | LOC | Role |
|---|---|---|
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/brownfield.lobster` | 400 | Brownfield mode workflow (Phase 0 ingest + greenfield overlay) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/code-delivery.lobster` | 436 | Per-story delivery sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/discovery.lobster` | 435 | Autonomous discovery mode |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/feature.lobster` | 1489 | Feature delta workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/greenfield.lobster` | 1408 | Greenfield workflow (reference path) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/maintenance.lobster` | 418 | Maintenance sweep workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/multi-repo.lobster` | 731 | Multi-repo coordination workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/planning.lobster` | 298 | Adaptive planning front-end |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-0-codebase-ingestion.lobster` | 146 | Phase 0 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-1-spec-crystallization.lobster` | 161 | Phase 1 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-2-story-decomposition.lobster` | 171 | Phase 2 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-3-tdd-implementation.lobster` | 158 | Phase 3 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-4-holdout-evaluation.lobster` | 38 | Phase 4 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-5-adversarial-refinement.lobster` | 54 | Phase 5 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-6-formal-hardening.lobster` | 91 | Phase 6 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/phase-7-convergence.lobster` | 175 | Phase 7 sub-workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/workflows/phases/per-story-delivery.md` | n/a | Per-story prose reference (not .lobster) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/bin/lobster-parse` | 51 | Bash YAML→JSON parser wrapper (yq + jq) |

**Total Lobster LOC: ~6,609** across 16 `.lobster` files.

### Category 2: Factory Dispatcher

| Absolute path | LOC | Role |
|---|---|---|
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/dispatcher/bin/darwin-arm64/factory-dispatcher` | ~12 MB binary | Mach-O 64-bit executable arm64 (compiled Rust) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/dispatcher/bin/darwin-x64/factory-dispatcher` | binary | Intel Mac variant |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/dispatcher/bin/linux-arm64/factory-dispatcher` | binary | Linux ARM64 |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/dispatcher/bin/linux-x64/factory-dispatcher` | binary | Linux x86_64 |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/dispatcher/bin/windows-x64/factory-dispatcher.exe` | binary | Windows |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/src/main.rs` | 673 | Dispatcher CLI entry |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/src/registry.rs` | 1153 | hooks-registry.toml parser, types, validation |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/src/routing.rs` | 265 | Event/tool matcher + priority grouping |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/src/payload.rs` | 187 | HookPayload (stdin envelope) types |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/src/engine.rs` | 141 | wasmtime engine + epoch ticker |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/src/executor.rs` | 931 | Plugin execution (sync/async tiers) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/crates/factory-dispatcher/Cargo.toml` | 58 | Crate manifest |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks-registry.toml` | 996 | Plugin registry (schema_version=2; 35 legacy-bash entries + 21 native WASM) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/hooks.json.template` | 116 | Claude Code hook wiring template |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/hooks.json.darwin-arm64` | ~80 | Platform-specialised (resolved {{PLATFORM}}) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/hooks.json.darwin-x64` | ~80 | (same) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/hooks.json.linux-arm64` | ~80 | (same) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/hooks.json.linux-x64` | ~80 | (same) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/hooks/hooks.json.windows-x64` | ~80 | (same) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/resolvers-registry.toml` | 18 | Context resolver registry (sibling of hooks-registry) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/config/artifact-path-registry.yaml` | n/a | Artifact path registry (12 KB) |

### Category 3: Factory Project Pattern (Discriminator)

| Absolute path | LOC | Role |
|---|---|---|
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/.claude-plugin/plugin.json` | 20 | Plugin manifest — the canonical name discriminator |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/state-template.md` | 110 | STATE.md template |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/state-manager-checklist-template.md` | 306 | STATE.md mutation discipline |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/factory-project-state-template.md` | 50 | Multi-repo project-level STATE.md |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/wave-state-template.yaml` | 29 | Wave lifecycle tracker |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/cycle-manifest-template.md` | 59 | Cycle directory manifest |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/templates/factory-project-structure-template.md` | 56 | `.factory-project/` layout (multi-repo) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/CLAUDE.md` | 65 | Root project conventions |

NOTE: User listed `plugin.json` at the plugin root; **actual path is `.claude-plugin/plugin.json`**. Similarly the user listed `CLAUDE.md` "top-level project conventions" — that lives at the repo root, NOT inside `plugins/vsdd-factory/` (no `plugins/vsdd-factory/CLAUDE.md` exists).

### Category 4: Workflow Execution Primitives

| Absolute path | LOC | Role |
|---|---|---|
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/run-phase/SKILL.md` | 62 | Drives a phase from its `.lobster` file |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/next-step/SKILL.md` | 39 | "What should I do now?" from STATE.md + workflow |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/validate-workflow/SKILL.md` | 47 | Static checker for `.lobster` files |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/factory-dashboard/SKILL.md` | 69 | Live markdown dashboard (calls `bin/factory-dashboard`) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/recover-state/SKILL.md` | 164 | Rebuild STATE.md from on-disk artifacts |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/check-state-health/SKILL.md` | 118 | Validate STATE.md structure/size/numbering |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/skills/state-update/SKILL.md` | 89 | Internal STATE.md mutator |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/plugins/vsdd-factory/agents/orchestrator/orchestrator.md` | 437 | Orchestrator agent prompt |

### Category 5: Top-Level Documentation

| Absolute path | LOC | Role |
|---|---|---|
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/README.md` | 303 | Product README + Mermaid pipeline diagram |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/CLAUDE.md` | 65 | Root project conventions (self-referential, branching model, release process) |
| `/Users/jmagady/Dev/monocle/.reference/vsdd-factory/CHANGELOG.md` | 286,884 bytes | Release history (NOT deeply read — out of scope after scan-only confirmation) |

## In-Scope LOC Roll-up

| Bucket | LOC |
|---|---|
| Lobster workflows (16 files) | ~6,609 |
| Dispatcher Rust source (6 files in `src/`) | 3,350 |
| hooks-registry.toml + hooks.json.template | 1,112 |
| In-scope skill SKILL.md files (7) | 588 |
| Orchestrator agent | 437 |
| STATE-related templates (7) | 670 |
| `plugin.json` + `lobster-parse` + root `CLAUDE.md` + `README.md` | 439 |
| **In-scope total** | **~13,205 LOC** |

## State Checkpoint

```yaml
pass: 1
status: complete
files_scanned: ~46 in-scope files
in_scope_loc: ~13205
out_of_scope: explicit (see Scope Boundary section)
timestamp: 2026-05-11T22:00:00Z
next_pass: 2-architecture
```
