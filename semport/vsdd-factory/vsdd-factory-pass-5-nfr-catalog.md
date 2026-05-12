# Pass 5 — Scoped NFR Catalog: vsdd-factory

NFRs that monocle must respect/inherit.

## Performance

| ID | NFR | Source | Implication for monocle |
|---|---|---|---|
| P-001 | factory-dispatcher startup ~100 ms | `engine.rs:1-12` ("The engine is expensive to build (~100 ms)"). | Monocle should not invoke the dispatcher to read state — read `.factory/` files directly. |
| P-002 | Per-hook 10s timeout (template default) | `hooks/hooks.json.template:9` (`"timeout": 10000` per event entry). | Monocle's own hooks (if it ever has them) should fit this budget. |
| P-003 | Per-plugin 5s default timeout, 10M fuel cap | `registry.rs:153-161` (RegistryDefaults). | Plugin output is bounded; events arrive in soft real-time. |
| P-004 | Epoch tick = 10 ms (timeout resolution) | `engine.rs:22-23`. | Events appear within 10-20 ms of plugin completion. |
| P-005 | Async drain window = `ASYNC_DRAIN_WINDOW_MS` (constant in `lib.rs`) | `main.rs:23-31`. | Telemetry-only async plugins may complete slightly after the dispatcher exits. |

## Security

| ID | NFR | Source | Implication |
|---|---|---|---|
| S-001 | Capabilities deny-by-default | `registry.rs:80-92`. | Plugin permissions are explicit; monocle never relies on implicit access. |
| S-002 | `shell_bypass_acknowledged` required for shell interpreters | `registry.rs:104-110`. | Bash hooks via legacy-bash-adapter MUST set this flag. |
| S-003 | `VSDD_SINK_FILE` is debug-only (SEC-003) | `main.rs:65-77` (`#[cfg(debug_assertions)]` gate). | Monocle SHOULD NOT depend on `VSDD_SINK_FILE` for production reads. |
| S-004 | Path allow-lists rooted at `CLAUDE_PROJECT_DIR` | `registry.rs:121-124`, `:134-136`. | Project boundary is enforced; monocle reads should respect the same root. |
| S-005 | Schema-version mismatch fails CLOSED | `registry.rs:30-35`, `main.rs:131-138`. | Monocle SHOULD validate `schema_version` before parsing any registry. |
| S-006 | No AI attribution in commits | root `CLAUDE.md:32-33`. | Monocle's commit messages MUST omit `Co-Authored-By: Claude`. |
| S-007 | Information asymmetry walls enforced via context.exclude | `code-delivery.lobster:118-135`, `:285-300`. | Monocle session view MUST NOT expose excluded artifacts to adversary agents. |
| S-008 | Branch protection: release branches MUST target `main` | root `CLAUDE.md:23-25`. | Monocle should treat release branch naming as a constraint, not enforce it. |

## Observability

| ID | NFR | Source | Implication |
|---|---|---|---|
| O-001 | Always-on internal log via `InternalLog` | `main.rs:81-82`, `internal_log.rs` (referenced). | Even on registry load failure, the dispatcher logs the error. |
| O-002 | Plugin lifecycle events emitted by executor | `main.rs:14-19`. | `plugin.invoked`, `plugin.completed`, `plugin.timeout`, `plugin.crashed` events exist. |
| O-003 | Dispatcher structured events | `main.rs:21-23`. | `dispatcher.schema_mismatch`, `dispatcher.registry_invalid`, `plugin.async_block_discarded`, `plugin.timeout` (async path), `internal.dispatcher_error`. |
| O-004 | Multi-sink architecture | `Cargo.toml:34-38`. | File sink (always-on, monocle's read path) + datadog/honeycomb/http/otel-grpc. |
| O-005 | Default log retention | `main.rs:81` (`internal_log.prune_old(DEFAULT_RETENTION_DAYS)`). | Old events are pruned; monocle should not rely on long history. |
| O-006 | Cost tracking metadata per workflow | feature.lobster:62-76, greenfield.lobster:21-25. | Cost summary is at `.factory/feature/cost-summary.md` (or analogous). |
| O-007 | factory-dashboard tool exists | `bin/factory-dashboard` (9 KB). | Monocle can shell out to it for a baseline rendering. |

## Reliability

| ID | NFR | Source | Implication |
|---|---|---|---|
| R-001 | Crash recovery via STATE.md + git worktrees | feature.lobster:41-53. | Monocle's own state should be reconstructible from on-disk artifacts. |
| R-002 | Single-commit burst protocol for STATE.md | `state-manager-checklist-template.md:87-111`. | Monocle MUST NOT write STATE.md (it's mutated by state-manager only). |
| R-003 | Pre-Tool/Stop hook checks for stale state | `wave-state-template.yaml:6-7`. | Stale wave-state.yaml = wave-gate violation = block. |
| R-004 | Workflow timeouts have hard ceilings | most workflows declare `defaults.timeout: 2h` and per-step `timeout: 30m/1h/4h`. | Long-running phases (Phase 0 ingest = 4h) are bounded. |
| R-005 | Wave gate `fail_action` semantics | wave-state-template.yaml + per-workflow gates. | `block` = halt pipeline; `warn` = continue with warning. |
| R-006 | `optional: true` step skip-on-fail | greenfield.lobster:75, brownfield.lobster:107. | Some steps are tolerated as missing. |
| R-007 | `max_retries: 2` default | all top-level workflows declare it. | Transient failures auto-retry twice. |
| R-008 | `on_failure: escalate` default | all top-level workflows. | Unrecoverable failures escalate to human, not silent skip. |

## Scalability

| ID | NFR | Source | Implication |
|---|---|---|---|
| Sc-001 | wasmtime single-thread runtime | `main.rs:79` (`#[tokio::main(flavor = "current_thread")]`). | Dispatcher does not pool threads; one process per hook call. |
| Sc-002 | Same-priority plugins fire in parallel | `routing.rs:64-86`. | Hot paths can register multiple plugins without serializing. |
| Sc-003 | Wave parallelism for stories | `feature.lobster:41-53`, `phase-3-tdd-implementation.lobster`. | Stories within a wave run in parallel worktrees. |
| Sc-004 | Multi-repo project parallelism | `multi-repo.lobster:156-164` (`parallel-foreach` step type). | Per-repo Phase 0 runs in parallel. |
| Sc-005 | STATE.md size cap = 500 lines hard, 200 warn | `state-template.md:21-22`. | Monocle UI should not assume large STATE.md is unusual. |

## Configurability

| ID | NFR | Source | Implication |
|---|---|---|---|
| C-001 | Registry defaults overridable per-entry | `registry.rs:143-162` + `RegistryEntry::priority/timeout_ms/fuel_cap/on_error`. | Plugins customize their slot in the dispatch graph. |
| C-002 | `cost_monitoring.thresholds` configurable | feature.lobster:69-71. | Cost gate triggers are project-defined. |
| C-003 | `merge-config.yaml` autonomy levels | code-delivery.lobster:413-416. | Merge decisions are policy-driven (Level 3 / 3.5 / 4). |
| C-004 | `discovery-config.yaml` schedule | discovery.lobster:31-37. | Cadences are configurable per-product. |
| C-005 | Workflow `inputs:` for sub-workflows | code-delivery.lobster:21-26. | Reusable sub-workflows with typed inputs. |

## Missing NFRs (notable absences)

- **No documented event-log SLA**. Monocle should not assume events appear synchronously; the async drain window can defer telemetry by 100 ms+.
- **No documented session correlation across sessions**. `session_id` is stable within a Claude session but not across. Monocle's "session view" should match Claude session boundaries.
- **No public API for monocle-style introspection**. The only programmatic interfaces today are `bin/lobster-parse`, `bin/factory-dashboard`, and direct file reads. There is no JSON-RPC endpoint.

## State Checkpoint

```yaml
pass: 5
status: complete
nfrs_cataloged: 37
categories: 6
timestamp: 2026-05-11T22:18:00Z
next_pass: 6-conventions
```
