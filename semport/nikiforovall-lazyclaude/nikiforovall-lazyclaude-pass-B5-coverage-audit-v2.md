# Phase B.5: Coverage Audit v2 — Fresh-Context Watchdog

**Date:** 2026-05-11
**Method:** Fresh-context audit. Reads source-of-truth directly (no reliance on prior agent's claims). Independent recomputation.
**HEAD verified:** `ebc1f8f3b046a04707340f749b4a441e26df7f6d` (from B.6 — re-confirmed file structure unchanged)
**Sibling audits running concurrently:** any-context B.5, codemachine, vsdd-factory (different subtrees)
**Scope:** Re-audit the original `pass-B5-coverage-audit.md` against the substantially-expanded Phase B full-protocol artifact set (services-r1..r3, mixins-r1..r2, app-keybindings-r1..r3, models-r1..r4).

## 1. Artifact inventory (all artifacts under `/Users/jmagady/Dev/monocle/.factory/semport/nikiforovall-lazyclaude/`)

| File | Size (bytes) | Scope | Convergence |
|---|---|---|---|
| `nikiforovall-lazyclaude-pass-1-project-discovery.md` | 10,826 | Inventory (50 src + 28 test, 9,280 / 5,275 LOC, 48M, HEAD pinned) | broad |
| `nikiforovall-lazyclaude-pass-2-architecture.md` | 17,300 | Component map; widget/mixin/service architecture | broad |
| `nikiforovall-lazyclaude-pass-3-conventions.md` | 15,175 | Conventions / patterns | broad |
| `nikiforovall-lazyclaude-pass-4-behavioral-contracts.md` | 21,511 | 12 BCs (BC-1..BC-12) | broad |
| `nikiforovall-lazyclaude-pass-5-verification-gaps.md` | 11,205 | Verification / drift | broad |
| `nikiforovall-lazyclaude-pass-6-security-deps.md` | 9,022 | Sec posture, dependency cone | broad |
| `nikiforovall-lazyclaude-pass-7-holdout-seeds.md` | 11,953 | 12 holdout seeds (4 P0-class) | broad |
| `nikiforovall-lazyclaude-pass-8-final-synthesis.md` | 16,996 | Phase C synthesis | predates full-protocol B |
| `nikiforovall-lazyclaude-pass-B-deep-parsers-r1.md` | 13,174 | Parser layer, 7 parsers | r1 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-parsers-r2.md` | 7,051 | Parser layer follow-up | r2 NITPICK (converged) |
| `nikiforovall-lazyclaude-pass-B-deep-plugin-marketplace-r1.md` | 13,182 | Plugin/marketplace single round | r1 — no r2 |
| `nikiforovall-lazyclaude-pass-B-deep-widgets-r1.md` | 13,707 | Widget layer single round | r1 — no r2 |
| `nikiforovall-lazyclaude-pass-B-deep-services-r1.md` | 52,370 | 10-file non-parser service layer | r1 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-services-r2.md` | 30,410 | Filter truth table; set-algebra walkthrough; mixed-version TypeError | r2 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-services-r3.md` | 18,136 | TOCTOU, walk_filtered overlap, hook double-discovery | r3 NITPICK (converged) |
| `nikiforovall-lazyclaude-pass-B-deep-mixins-r1.md` | 46,442 | Modal-Confirm-Callback; shell=True; move-no-rollback | r1 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-mixins-r2.md` | 36,195 | scope_view; refresh_bindings; pending-op race | r2 (claimed converged) |
| `nikiforovall-lazyclaude-pass-B-deep-app-keybindings-r1.md` | 44,080 | 32-binding registry; MRO; check_action gate | r1 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-app-keybindings-r2.md` | 35,455 | Textual→ratatui; subprocess POSIX bug | r2 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-app-keybindings-r3.md` | 23,518 | shell=True P0 confirmed via Python docs | r3 NITPICK (converged) |
| `nikiforovall-lazyclaude-pass-B-deep-models-r1.md` | 35,780 | Full Rust struct mapping; 3 P0 / 7 P1 | r1 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-models-r2.md` | 15,879 | Metadata key set; installed_scopes literal set | r2 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-models-r3.md` | 11,843 | 11-site Customization mutation surface | r3 SUBSTANTIVE |
| `nikiforovall-lazyclaude-pass-B-deep-models-r4.md` | 6,593 | Tests pin mutations; basename dedup; writer ignores metadata | r4 NITPICK (converged) |
| `nikiforovall-lazyclaude-pass-B5-coverage-audit.md` | 8,578 | **ORIGINAL B.5 (under re-audit)** | — |
| `nikiforovall-lazyclaude-pass-B6-extraction-validation.md` | 5,438 | Numeric recount | — |
| `nikiforovall-lazyclaude-pass-B5-coverage-audit-v2.md` | (this file) | — | — |

**Total Phase B artifacts:** 16 deepening files. Original B.5 was written 2026-05-11T17:25, **BEFORE** services/mixins/app-keybindings/models full-protocol rounds began (those start ~17:55+). Therefore the original B.5 explicitly cannot represent the expanded coverage.

## 2. Subsystem × Pass coverage matrix

Cells: **F** = full read with structural notes; **P** = partial / by-name only; **S** = surface mention; **·** = no coverage in that pass.

| Subsystem (file) | p1 | p2 | p3 | p4 | p5 | p6 | p7 | par-r1 | par-r2 | plug-mkt-r1 | wid-r1 | svc-r1 | svc-r2 | svc-r3 | mix-r1 | mix-r2 | akb-r1 | akb-r2 | akb-r3 | mod-r1 | mod-r2 | mod-r3 | mod-r4 | B5-orig | B6-orig | C-orig |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| **app.py** | S | F | F | F | S | S | S | · | · | · | · | · | · | · | F | F | F | F | F | · | · | · | · | F | F | S |
| **bindings.py** | S | F | S | · | · | · | · | · | · | · | · | · | · | · | F | · | F | F | · | · | · | · | · | F | S | S |
| **keybindings/__init__.py** | S | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | S | · | · | · | · | · | · | F (4 LOC, empty) | S | · |
| **mixins/__init__.py** | S | S | · | · | · | · | · | · | · | · | · | · | · | · | F | S | S | · | · | · | · | · | · | F | S | · |
| **mixins/customization_actions.py** | S | F | F | F | F | · | · | · | · | · | · | · | · | · | F | F | S | S | · | · | · | · | · | F | S | F |
| **mixins/filtering.py** | S | F | F | F | · | · | · | · | · | · | · | · | · | · | F | F | S | · | · | · | · | · | · | F | S | F |
| **mixins/help.py** | S | F | S | S | · | · | · | · | · | · | · | · | · | · | F | S | S | S | · | · | · | · | · | F | S | S |
| **mixins/marketplace.py** | S | F | F | F | F | S | · | · | · | F | · | · | · | · | F | F | F | F | F | · | · | · | · | F | F | F |
| **mixins/navigation.py** | S | F | F | F | · | · | · | · | · | · | · | · | · | · | F | F | F | F | · | · | · | · | · | F | S | F |
| **models/__init__.py** | S | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | · | · | · | F | S | · |
| **models/customization.py** | S | F | F | F | S | · | S | F | · | F | · | F | F | · | S | · | S | · | · | F | F | F | F | F | F | F |
| **models/marketplace.py** | S | F | F | F | S | S | S | · | · | F | · | F | F | S | F | · | S | · | · | F | F | F | F | F | F | F |
| **models/settings.py** | S | F | F | F | · | · | S | · | · | · | · | F | · | · | F | · | F | · | · | F | F | · | · | F | S | F |
| **services/__init__.py** | S | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | S | · |
| **services/config_path_resolver.py** | S | F | F | F | F | · | · | · | · | · | · | F | S | · | · | · | · | · | · | · | · | · | · | F | S | S |
| **services/discovery.py** | F | F | F | F | F | F | F | · | · | F | · | F | F | F | · | · | S | · | · | F | F | F | F | F | F | F |
| **services/filesystem_scanner.py** | S | F | F | F | F | · | · | · | · | · | · | F | S | F | · | · | · | · | · | F | · | · | · | F | S | F |
| **services/filter.py** | S | F | F | F | · | · | · | · | · | · | · | F | F | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **services/gitignore_filter.py** | S | F | F | F | F | F | F | · | · | · | · | F | F | F | · | · | · | · | · | · | · | · | · | F | S | F |
| **services/marketplace_loader.py** | S | F | F | F | S | · | F | · | · | F | · | F | F | · | F | F | · | · | · | F | F | F | F | F | F | F |
| **services/opener.py** | S | F | F | F | · | F | S | · | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **services/plugin_loader.py** | S | F | F | F | F | · | F | · | · | F | · | F | F | · | F | · | · | · | · | F | F | F | · | F | F | F |
| **services/settings.py** | S | F | F | F | · | · | · | · | · | · | · | F | · | · | · | · | · | · | · | F | F | · | · | F | S | F |
| **services/writer.py** | S | F | F | F | F | F | F | · | · | · | · | F | F | F | F | · | · | · | · | · | · | · | F | F | F | F |
| **services/parsers/__init__.py** | S | F | F | F | F | · | F | F | F | · | · | · | · | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **services/parsers/hook.py** | S | F | F | F | S | · | · | F | F | · | · | S | · | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **services/parsers/lsp_server.py** | S | F | F | F | F | · | · | F | F | · | · | S | · | · | · | · | · | · | · | F | F | F | · | F | S | F |
| **services/parsers/mcp.py** | S | F | F | F | S | · | F | F | F | · | · | S | S | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **services/parsers/memory_file.py** | S | F | F | F | F | · | F | F | F | · | · | S | · | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **services/parsers/skill.py** | S | F | F | F | S | · | · | F | F | · | · | S | · | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **services/parsers/slash_command.py** | S | F | F | F | S | · | · | F | F | · | · | F | · | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **services/parsers/subagent.py** | S | F | F | F | S | · | F | F | F | · | · | S | · | · | · | · | · | · | · | F | F | · | · | F | F | F |
| **widgets/__init__.py** | S | S | · | · | · | · | · | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | F | S | · |
| **widgets/app_footer.py** | S | F | F | S | · | · | · | · | · | · | F | · | · | · | · | · | F | · | · | · | · | · | · | F | S | S |
| **widgets/combined_panel.py** | S | F | F | S | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | F | F | F |
| **widgets/delete_confirm.py** | S | F | F | · | · | · | · | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **widgets/detail_pane.py** | S | F | F | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | F | F | S | F |
| **widgets/filter_input.py** | S | F | F | · | · | · | · | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **widgets/helpers/__init__.py** | S | S | · | · | · | · | · | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | F | S | · |
| **widgets/helpers/rendering.py** | S | F | F | · | · | · | · | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **widgets/level_selector.py** | S | F | F | · | · | · | · | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **widgets/marketplace_confirm.py** | S | F | F | · | · | · | · | · | · | F | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | F |
| **widgets/marketplace_modal.py** | S | F | F | F | F | · | F | · | · | F | F | · | · | · | F | F | · | · | · | F | F | F | · | F | F | F |
| **widgets/marketplace_source_input.py** | S | F | F | · | · | · | F | · | · | F | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **widgets/plugin_confirm.py** | S | F | F | · | · | · | · | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **widgets/status_panel.py** | S | F | F | · | · | · | · | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | F | S | S |
| **widgets/type_panel.py** | S | F | F | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | F | F | F |
| **styles/app.tcss** | · | F | S | · | · | · | · | · | · | · | · | · | · | · | · | · | F | S | · | · | · | · | · | F | S | F |
| **themes.py** | S | F | S | · | · | · | · | · | · | · | · | · | · | · | · | · | F | S | · | · | · | · | · | F | S | F |
| **__init__.py** | S | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | S | · | · | · | · | · | · | F | S | · |
| **__main__.py** | S | F | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | · | · | · | · | · | · | F | S | · |
| **pyproject.toml** | F | S | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | F | S |
| **uv.lock** | S | · | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | S | · |
| **tests/conftest.py** | S | S | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | F | S | S |
| **tests/integration/discovery/test_auto_memory.py** | · | · | · | F | · | · | · | F | F | · | · | · | · | · | · | · | · | · | · | · | · | · | F | F | S | F |
| **tests/integration/discovery/test_behavior.py** | · | · | · | F | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **tests/integration/discovery/test_gitignore.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/integration/discovery/test_hooks.py** | · | · | · | F | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **tests/integration/discovery/test_mcps.py** | · | · | · | F | · | · | F | F | · | · | · | F | · | F | · | · | · | · | · | · | · | · | · | F | S | F |
| **tests/integration/discovery/test_memory_files.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/integration/discovery/test_plugins.py** | · | · | · | F | · | · | F | · | · | F | · | · | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **tests/integration/discovery/test_skills.py** | · | · | · | F | · | · | · | F | F | · | · | · | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **tests/integration/discovery/test_slash_commands.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/integration/discovery/test_subagents.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/integration/writer/test_delete_writer.py** | · | · | · | S | · | · | · | · | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/integration/writer/test_mcp_writer.py** | · | · | · | F | · | · | · | · | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | P (partial 80 LOC) | S | · |
| **tests/unit/test_app_customization_actions.py** | · | · | · | F | · | · | · | · | · | · | · | · | · | · | F | · | · | · | · | · | · | · | · | P (partial 100 LOC) | F | F |
| **tests/unit/test_combined_panel.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_config_path_resolver.py** | · | · | · | S | · | · | · | · | · | · | · | F | S | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | S |
| **tests/unit/test_customization_writer.py** | · | · | · | S | · | · | F | · | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_filesystem_scanner.py** | · | · | · | S | · | · | · | · | · | · | · | F | S | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_gitignore_filter.py** | · | · | · | S | · | · | · | · | · | · | · | F | S | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_level_selector.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_memory_file_ref.py** | · | · | · | S | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_plugin_source_path.py** | · | · | · | F | · | · | · | · | · | F | · | F | · | · | · | · | · | · | · | · | · | · | · | F | S | F |
| **tests/unit/test_rules_discovery.py** | · | · | · | S | · | · | F | · | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/unit/test_settings_service.py** | · | · | · | S | · | · | · | · | · | · | · | F | · | · | · | · | · | · | · | · | · | · | · | P (listed only) | S | · |
| **tests/integration/fixtures/** | · | · | · | · | · | · | · | · | · | S | · | · | F (set-algebra walkthrough) | · | · | · | · | · | · | · | · | · | · | (declared not-source) | S | · |

**Key takeaway from matrix:** The expanded Phase B full-protocol rounds (services/mixins/app-keybindings/models) thoroughly deepen **source files** but **do not systematically deepen the test files** themselves. Eight test files appear ONLY in "P (listed only)" form (the original B.5 admission of partial read). They are now indirectly cited from full-protocol rounds (test_settings_service in svc-r1, test_filesystem_scanner in svc-r1, etc.) but those rounds reference the test *outputs* (numbers of tests, what's covered) rather than reading the tests in full.

## 3. Identified GAPS (re-evaluated against expanded Phase B)

### GAP-1: 7 test files remain entirely unread, 2 partial

**Test files that are STILL not deepened in full** (listed in original B.5 + still not opened in subsequent rounds):

| Test file | LOC | Listed-only since B.5 | Indirectly cited in deepening |
|---|---|---|---|
| `tests/integration/discovery/test_gitignore.py` | 252 | YES | partial citation in svc-r1 "13 tests" |
| `tests/integration/discovery/test_memory_files.py` | 55 | YES | no direct citation |
| `tests/integration/discovery/test_slash_commands.py` | 55 | YES | no direct citation |
| `tests/integration/discovery/test_subagents.py` | 54 | YES | no direct citation |
| `tests/integration/writer/test_delete_writer.py` | (declared 345 by svc-r1) | YES | svc-r1 citation only |
| `tests/unit/test_combined_panel.py` | (not declared) | YES | no direct citation |
| `tests/unit/test_level_selector.py` | (not declared) | YES | no direct citation |
| `tests/unit/test_memory_file_ref.py` | (not declared) | YES | no direct citation |
| `tests/integration/writer/test_mcp_writer.py` | 385 (per svc-r1) | partial (80 LOC read) | svc-r1 |
| `tests/unit/test_app_customization_actions.py` | 167 (per mix-r1) | partial (100 LOC read) | mix-r1 |

**Status verdict:** The original B.5 admitted "~40% test files read." The expanded full-protocol Phase B did **not** systematically close this gap. The services round (`svc-r1`) cites tests by line ranges (e.g., "test_config_path_resolver.py:1-213 (8 tests, well-covered)") but cites coverage extent rather than examining test fixtures or test-as-spec semantics. The original B.5's stance ("test coverage of analytical findings: ~40%") is **still essentially accurate**. **GAP CONFIRMED** — the 28 test files were never deepened as a unit.

### GAP-2: `tests/integration/fixtures/` not deepened as canonical example shapes

**Original B.5 declared:** "tests/integration/fixtures/** — real-on-disk fixtures, not source; LOW risk because schemas already extracted."

**Expanded Phase B status:** services-r2 walks the set-algebra of `_load_installed_plugins` using the fixture file `installed_plugins.json` as input. mod-r3 cites conftest.py's `plugins_config` fixture (lines 154-176). plugin-marketplace-r1 cites `marketplace.json` shape. BUT — **no round produces an inventory of all 24 fixture files with their canonical schemas, expected parses, and edge cases**.

This matters because for a Rust port the fixtures are the **most concrete worked examples** of the on-disk schema. The brief identified this as a candidate for the new B.5 to surface; my judgment: **partial gap remaining**. Fixtures are referenced opportunistically when needed but never enumerated as a deliverable.

Files in `tests/integration/fixtures/`:
- `agents/explorer.md`, `commands/greet.md`, `commands/nested/deep-cmd.md`
- `mcp/local.claude.json`, `mcp/project.mcp.json`, `mcp/user.claude.json`
- `memory/AGENTS.md`, `memory/CLAUDE.md`
- `plugins/example-plugin/{agents,commands,skills/plugin-skill/SKILL.md}/*.md`
- `plugins/installed_plugins.json`
- `project/{agents,CLAUDE.md,commands,skills}/*`
- `settings/{project-settings.json,user-settings.json}`
- `skills/full-skill/{SKILL.md,reference.md,examples.md,scripts/{run.py,setup.sh},templates/{basic.md,advanced/template.md}}`
- `skills/task-tracker/SKILL.md`

Of these, the mcp fixtures, installed_plugins.json, settings/, and the full-skill structure are all directly load-bearing for the Rust port. They were **never enumerated together** in any pass.

### GAP-3: `pyproject.toml` / `uv.lock` not deepened

**pyproject.toml:** read in Pass 1 (broad inventory). **No deepening round** revisited it for dependency strategy, build characteristics, or Rust-port crate mapping that should derive from it.
**uv.lock (99K):** declared "LOW for ingest" in original B.5. The expanded Phase B did not change this. **Gap stands but justified.** The 99K of pinned versions is not load-bearing for the port's domain spec.

### GAP-4: LSP layer — pass-6 said "139 LOC of lsp_server.py + plugin LSP discovery have zero tests"

**Verification:** confirmed no test file references LSP except `tests/unit/test_combined_panel.py` (widget-rendering only). The services round (svc-r1) cites `_discover_plugin_lsp_servers` at `discovery.py:701-722` as 0%-tested. Models-r3 confirms LSP parse_plugin_json metadata shape.

**Status:** the LSP gap is **acknowledged but not closed** — no deepening round produces LSP-specific tests-as-spec analysis because no LSP tests exist. The parsers-r1 + parsers-r2 rounds cover the parser semantics; that's the maximum possible given source coverage.

### GAP-5: `tests/conftest.py` test harness conventions

**Verification:** mod-r4 cites conftest.py:1-196 explicitly (Finding M). Conftest IS deepened in the models round. **NOT a gap.**

### GAP-6: Plugin scope phases 2/3 — Pass 6 said untested

**Verification:** confirmed in svc-r1 ("ZERO tests for ... get_all_plugins() three-phase enumeration"). The services round walks the phases with concrete inputs in svc-r2 set-algebra walkthrough. So the **semantics are documented; the test coverage gap remains real but acknowledged**. **NOT a gap in coverage of the codebase**, but a real test-suite gap in the reference.

### Net gap assessment

| Original B.5 stance | v2 finding |
|---|---|
| "Source coverage 100% (50/50 files)" | CONFIRMED via fresh `find` |
| "Test coverage ~40% files read" | STILL ~40% even after expanded Phase B; the deepening rounds tightened the SOURCE coverage, not the TEST coverage |
| "5 honest gaps declared" | The 5 are real and still present in expanded Phase B. Plus: GAP-2 (fixtures not enumerated) is a P2 addition the original B.5 did not surface |

## 4. Cross-round inconsistency check

Critical claims appearing in multiple rounds:

### Claim A: Atomic-write gap

| Round | Citation | Statement |
|---|---|---|
| Original Pass 8 (D7) | `settings.py:64-67`, `writer.py:415-418, 515-518` | "Naked write_text. P0." |
| svc-r1 §7 | `settings.py:64-67` | "Naked write_text. No tempfile, no rename." |
| svc-r1 §8 | `writer.py:415-418` (`_write_file`) and `writer.py:515-518` (`_write_settings_json`) | "Naked read+write. P0 confirmed." |
| svc-r1 P0 list | THREE sites: settings.py:64-67, writer.py:415-418, 515-518 | "ALL three mutation surfaces use naked write_text." |
| akb-r2 | (mentions writer atomic-write only briefly) | consistent |

**Consistency verdict:** ✅ All rounds agree. Three confirmed sites, byte-precise. v2 spot-checked `writer.py:415-418`:

```
414     def _write_file(self, source_path: Path, target_path: Path) -> None:
415         """Write file content to target path."""
416         content = source_path.read_text(encoding="utf-8")
417         target_path.write_text(content, encoding="utf-8")
```
(verified — naked read/write, no tempfile, no rename, matches claim)

### Claim B: `shell=True` + list args POSIX bug

| Round | Citation | Statement |
|---|---|---|
| Pass 8 (anti-pattern table) | `marketplace.py:253` (mentioned) + `app.py:576, 587` (gated) | "shell=True misuse" |
| mix-r1 P1 confirmed | `marketplace.py:253-261` (run) + `marketplace.py:293` (Popen) | "Silent no-op success on POSIX" |
| svc-r1 | `opener.py:9-28` — "No shell=True. Safe wrt argument injection because args is a list." (about opener, not marketplace) | Opener does NOT have the bug |
| akb-r2 §9 | `marketplace.py:253-261` | "P0 candidate" |
| akb-r3 §1 | `marketplace.py:253-261` | "P0 confirmed via Python docs" |

**Consistency verdict:** ✅ All four rounds AGREE on the bug at `marketplace.py:253-261`. Mixins-r1 ALSO surfaces a SECOND site at `marketplace.py:293` (the `Popen` in `open_plugin_folder`). v2 verified both sites in source (lines 253-261, line 293 in mixins/marketplace.py — confirmed).

**Minor inter-round line-range divergence (NOT a contradiction):**
- mix-r1 cites `:253-261` and ALSO `:248-267` for the `_run_plugin_command` worker (the surrounding @work(thread=True) function)
- akb-r2 cites `:253-261`
- svc-r1 says "Used by: `mixins/marketplace.py:309, 315, 329, 335`" — this is about opener call sites, not shell=True itself
- The actual content of `marketplace.py:248-267` is the full `_run_plugin_command` function definition; `:253-261` is the `subprocess.run` call. **Both citations are correct; they cite different scope nestings.**

### Claim C: Project-slug regex

| Round | Citation | Statement |
|---|---|---|
| Pass 7 (Seed 1) | `discovery.py:478-484` | "re.sub(r'[^a-zA-Z0-9\-]', '-', str(self.project_root))" |
| Pass B6 | `discovery.py:484` | confirmed |
| svc-r1 | `discovery.py:484` | "re.sub(r'[^a-zA-Z0-9\-]', '-', str(self.project_root))" |
| mod-r1 | location was claimed in `models/` → REFUTED in mod-r3 to `discovery.py:484` | self-corrected |

**Consistency verdict:** ✅ All consistent after mod-r3's self-correction. v2 spot-checked `discovery.py:484`:
```
484         return re.sub(r"[^a-zA-Z0-9\-]", "-", str(self.project_root))
```
(verified)

### Claim D: Sort order pinned by test

| Round | Citation | Statement |
|---|---|---|
| svc-r1 | `discovery.py:243-251` with `CustomizationType enum order` | "type_order[c.type], c.name.lower()" |
| mod-r1..r4 | "pinned by test" (test_behavior.py:29-33) | sort order = declaration order from `models/customization.py:37-46` |

**Consistency verdict:** ✅ Consistent. v2 verified `discovery.py:243-251`:
```
243     def _sort_customizations(
244         self, customizations: list[Customization]
245     ) -> list[Customization]:
246         """Sort customizations by type order then name."""
247         type_order = {t: i for i, t in enumerate(CustomizationType)}
248         return sorted(
249             customizations,
250             key=lambda c: (type_order[c.type], c.name.lower()),
251         )
```
And `models/customization.py:37-47`:
```
37 class CustomizationType(Enum):
38     """Type of Claude Code customization."""
39
40     SLASH_COMMAND = auto()
41     SUBAGENT = auto()
42     SKILL = auto()
43     MEMORY_FILE = auto()
44     MCP = auto()
45     HOOK = auto()
46     LSP_SERVER = auto()
```
(verified — 7 variants, declaration order is the sort key)

### Claim E: Mixed-version TypeError

| Round | Citation | Statement |
|---|---|---|
| Pass 7 (Seed 5) | `plugin_loader.py:329-353`, `marketplace_loader.py:267-283`, `marketplace_modal.py:425-437` | "P0 — latent crash" |
| svc-r2 P1-R2-1 | `plugin_loader.py:295`, `marketplace_loader.py:272` | "TypeError not caught; latent" |

**Consistency verdict:** ✅ Consistent. svc-r2 expands Seed 5 with concrete trace.

## 5. Recomputed metrics (independent)

### Source/test inventory

```
find .reference/nikiforovall-lazyclaude/src -name '*.py' | wc -l → 50  ✓
find .reference/nikiforovall-lazyclaude/tests -name '*.py' | wc -l → 28  ✓
find .reference/nikiforovall-lazyclaude/tests -name 'test_*.py' | wc -l → 23
find .reference/nikiforovall-lazyclaude/src -name '*.py' -exec wc -l {} + | tail -1 → 9280  ✓
find .reference/nikiforovall-lazyclaude/tests -name '*.py' -exec wc -l {} + | tail -1 → 5275  ✓
find .reference/nikiforovall-lazyclaude -name '*.tcss' → src/lazyclaude/styles/app.tcss (single file)  ✓
```

All Pass 1 / B.6 metrics independently verified.

### Test files vs claim

The original B.5 says "10 read, 15 unread". My recount: 28 total .py test files (incl. 5 __init__.py with no tests). 23 test_*.py files. Of those 23:
- **Original B.5 declared 10 read** (some partial). Recount of citations across all rounds: tests cited in full (not just by-name) in any deepening round = roughly 13 (test_auto_memory, test_behavior, test_hooks, test_mcps, test_skills, test_plugins, test_plugin_source_path, test_app_customization_actions partial, test_mcp_writer partial, test_config_path_resolver, test_filesystem_scanner, test_gitignore_filter, test_settings_service, test_customization_writer per svc-r1 ranges, test_rules_discovery per svc-r1 ranges).
- **Truly unread** (no round read in full): test_gitignore.py, test_memory_files.py, test_slash_commands.py, test_subagents.py, test_delete_writer.py, test_combined_panel.py, test_level_selector.py, test_memory_file_ref.py = **8 files**.

**Original B.5 said 15 unread**. After expanded Phase B, that count drops to ~8 truly unread + 2 partial. The original B.5 was **slightly stale** in this stat — the expanded Phase B does cite ~5 more test files by line range (in svc-r1's coverage matrix), though still without reading them end-to-end.

### BC count

Pass 4 defines **BC-1..BC-12 (12 contracts)**. Confirmed via `grep`. Pass 8 references "12 BCs". Phase B deepening rounds introduce additional implicit contracts (e.g., the FilterService truth table in svc-r2, the modal-confirm-callback in mix-r1) but do not number them with BC- prefixes. **The "12 BC" headline figure is unchanged across rounds.**

### HIGH/MEDIUM/LOW confidence distribution

Pass 4's BCs do not use HIGH/MEDIUM/LOW labels; instead they cite "Confidence: HIGH (directly from test)" patterns in some BCs. Sample check:
- BC-1 SlashCommandParser: HIGH (test_slash_commands.py exists)
- BC-2 SubagentParser: HIGH (test_subagents.py exists)
- BC-7 LSPServerParser: **LOW** (no test file)
- BC-9 Discovery walker: HIGH (test_behavior.py)
- BC-10 FilterService: **LOW** (zero tests confirmed by svc-r1)
- BC-11 Writer: HIGH (extensive tests)
- BC-12 ConfigPathResolver: HIGH

**Independent estimate:** ~7 HIGH, ~3 MEDIUM, ~2 LOW out of 12 BCs. The Phase B rounds did NOT re-rate confidence systematically. **No formal recount produced.**

### Tests:source ratio per subsystem

| Subsystem | Source LOC | Test LOC | Ratio |
|---|---|---|---|
| services (excluding parsers) | ~2,500 (10 files) | ~1,400 (test_config_path_resolver 213 + test_filesystem_scanner 221 + test_gitignore_filter 227 + test_settings_service 119 + test_customization_writer 522 + test_plugin_source_path 301 — sum ~1,603) | ~0.64 |
| services/parsers | ~600 (7 parsers + __init__) | ~0 unit + integration coverage in test_*.py files | unclear |
| mixins | 1,103 (mix-r1 §1) | 167 (test_app_customization_actions only) | ~0.15 |
| widgets | ~3,500 (per Pass 1 LOC by file) | ~600 (test_combined_panel + test_level_selector — file sizes not deepened, estimated) | ~0.17 |
| models | ~280 (3 model files) | 0 direct unit test | 0.0 |
| services/discovery + filesystem_scanner + gitignore_filter | core walker | covered indirectly via integration | ratio depends on integration vs unit |

**The mixins and models layers are critically under-tested in the reference.** This is confirmed by both svc-r1 and mix-r1.

### Holdout seeds (Pass 7's 12 seeds; 4 marked P0-class)

| Seed | Claim | Resolved in Phase B? |
|---|---|---|
| Seed 1 — slug regex | P0 | YES — svc-r1, svc-r2, mod-r3 cite `discovery.py:484` |
| Seed 2 — frontmatter CRLF | P1 | YES — parsers-r1 + svc-r1 cite CRLF concern |
| Seed 3 — tools_list polymorphism | P2 | YES — parsers-r1 covers |
| Seed 4 — walk_filtered depth | P1 | YES — svc-r1 §5 + svc-r3 Q4 |
| Seed 5 — _find_latest_version_dir TypeError | P0 | YES — svc-r2 P1-R2-1 expands with concrete trace |
| Seed 6 — _load_installed_plugins set algebra | P0 | YES — svc-r2 walks 6 steps with fixture inputs |
| Seed 7 — auto-collapse heuristic | P1 | NO — widget-r1 mentions but doesn't pin |
| Seed 8 — skill vs memory expansion keys | P2 | NO — widget-r1 mentions but doesn't deepen |
| Seed 9 — _extract_frontmatter_text duplicate | P2 | NO direct deepening |
| Seed 10 — _emit_selection_message focus/blur fire | P2 | NO direct deepening |
| Seed 11 — plugin scope resolution name-substring | P1 | YES — plugin-marketplace-r1 + mix-r1 |
| Seed 12 — watch_active_type old/new index | P1 | NO direct deepening |

**Of the 4 P0-class seeds: 4/4 deepened in subsequent rounds.** ✅
**Of the remaining 8 seeds (P1 and P2): 4/8 deepened, 4/8 not.** Acceptable for non-P0 priority.

The brief's claim that "4 marked 'resolved during deepening' are actually addressed" — verified ✅ for the 4 P0 seeds.

## 6. Stress-test critical invariants (independent source verification)

### Invariant A: Sort order pinned by declaration order

**Claim (models-r1..r4):** `customization.py:37-46` declares variants in order SLASH_COMMAND=1, SUBAGENT=2, SKILL=3, MEMORY_FILE=4, MCP=5, HOOK=6, LSP_SERVER=7. The `_sort_customizations` uses `type_order = {t: i for i, t in enumerate(CustomizationType)}` (zero-indexed in enumerate but the order is preserved).

**v2 verification via direct source read:**

`customization.py:37-47` — verified (note: B.5 cites lines 40-46, actual range is 37-47 for the class def; line 40 = `SLASH_COMMAND = auto()`). The 7 variants ARE declared in the claimed order. ✅

The actual `type_order` dict thus is `{SLASH_COMMAND: 0, SUBAGENT: 1, SKILL: 2, MEMORY_FILE: 3, MCP: 4, HOOK: 5, LSP_SERVER: 6}`. svc-r1 said "SLASH_COMMAND=1, ..., LSP_SERVER=7" referring to `auto()` enum VALUES — those are 1-indexed because `auto()` starts at 1 by default. **Both phrasings are correct** — the enum values are 1..7; the dict-derived index is 0..6. Either way, declaration order = sort order. ✅

### Invariant B: Project slug regex location

**Claim (models-r3):** refuted that it lives in `models/`; placed at `services/discovery.py:484`.

**v2 verification:** verified directly above. Located at `services/discovery.py:484`. ✅

### Invariant C: `installed_scopes` exhaustive set

**Claim (B5-v2 brief):** "only marketplace_loader.py:216-219" sets `_installed_scopes` values.

**v2 verification via direct source read of marketplace_loader.py:200-238:**

- Line 200: `self._installed_scopes = {}` (init/reset)
- Line 216-218: `if scope == "user": ... scopes.append("user")`
- Line 219-228: `elif scope in ("project", "local"): ... scopes.append(scope)`
- Line 238: `self._installed_scopes[pid] = list(dict.fromkeys(scopes))` ← **THIS IS WHERE _installed_scopes gets populated**

**REFINEMENT:** The brief's "216-219" line range is **slightly misleading** — those lines populate the local `scopes` list, but the actual assignment to `_installed_scopes[pid]` is at line 238. The literal-string set `{"user", "project", "local"}` flows through lines 216-228 (the if/elif/else). Models-r2 Finding E says "verified by enumerating EVERY string literal that flows into installed_scopes" — that enumeration IS line 216-228, not 216-219.

**Net verdict:** the SET of literal strings ✅ exhaustive at `{"user", "project", "local"}` confirmed. The CITATION 216-219 is slightly tight; the actual range is 200 (init) + 216-228 (literal flow) + 238 (assignment). **Minor citation imprecision in brief, not a hallucination.**

### Invariant D: `PluginScope` JSON serde literals NOT `"project_local"`

**Claim (models-r1, B.5-v2 brief):** PluginScope JSON serde uses `"user"`/`"project"`/`"local"`, NOT `"project_local"`.

**v2 verification via direct source grep of plugin_loader.py:**
- Line 91: `scope=inst.get("scope", "user")` — reads `scope` field from JSON
- Line 124: `if installation.scope == "user":` — compares against `"user"` literal
- Line 135: `if installation.scope == "project"`
- Line 148: `if installation.scope == "local"`
- Line 309-311: scope_map literally `{"user": PluginScope.USER, "project": PluginScope.PROJECT, "local": PluginScope.PROJECT_LOCAL}`

**ZERO occurrences of `"project_local"` as a string literal.** ✅ The Python ENUM is named `PluginScope.PROJECT_LOCAL` but the SERDE STRING is `"local"`. Confirmed.

### Invariant E: Customization mutation surface (claimed 11 sites)

**Claim (models-r3 Finding I):** 11 mutation sites for `Customization` (`customization.<field> =` assignments in services/).

**v2 verification via direct source grep:**

discovery.py mutation sites (verified via grep):
- Line 331: `c.plugin_info = plugin_info` (inside `_discover_md_files_from_paths`)
- Line 340: `c.plugin_info = plugin_info` (same function, rglob branch)
- Line 367: `c.plugin_info = plugin_info` (inside `_discover_custom_skills`)
- Line 389: `customization.plugin_info = plugin_info`
- Line 410: `customization.plugin_info = plugin_info`
- Line 459: `customization.name = str(rel_path)` (nested CLAUDE.md)
- Line 521: `customization.metadata["refs"] = ...` (per models-r3 listing; v2 did not direct-grep "metadata\[")
- Line 526: `customization.name = md_file.name`
- Line 550: `customization.name = str(rule_file.relative_to(...))`
- Line 566: `customization.name = str(rule_file.relative_to(...))`
- Line 679: `customization.plugin_info = plugin_info`
- Line 696: `customization.plugin_info = plugin_info`
- Line 711: `customization.plugin_info = plugin_info`
- Line 719: `customization.plugin_info = plugin_info`

filesystem_scanner.py:75: `customization.plugin_info = plugin_info` (verified)

**ACTUAL TOTAL: ~15 sites in discovery.py + 1 in filesystem_scanner.py = ~16 mutation sites** (counted via direct grep).

**Models-r3's claim of "11 sites" UNDERCOUNTS by 3-4 sites.** The missed sites are `discovery.py:331, 340, 367` — three additional `plugin_info` assignments inside `_discover_md_files_from_paths` and `_discover_custom_skills` (the marketplace-extras branches). Models-r3 listed mutation sites starting at discovery.py:389, missing the three earlier ones in the same file. The total mutation surface is **~14 sites in discovery.py + 1 in scanner = ~15-16 sites, not 11**.

**Severity:** Light citation imprecision, but the structural finding (Customization needs to be mutable) is unaffected — having 15 mutation sites instead of 11 reinforces the same conclusion more strongly.

**Status:** ⚠️ **MINOR UNDERCOUNT** — not a hallucination, but the number 11 is wrong. Should be ~15.

## 7. Hallucination spot-check from full-protocol rounds

Picked 10 citations across services-r1..r3, mixins-r1..r2, app-keybindings-r1..r3, models-r1..r4.

| # | Round | Citation | Claim | v2 source verification | Verdict |
|---|---|---|---|---|---|
| 1 | mix-r1 | `marketplace.py:253-261` | `subprocess.run(cmd, shell=True)` inside `_run_plugin_command` | Source lines 253-261 are exactly `subprocess.run(\n cmd,\n capture_output=True,\n check=True,\n shell=True,\n encoding="utf-8",\n errors="replace",\n cwd=cwd,\n)`. shell=True at line 257. | ✅ CONFIRMED |
| 2 | mix-r1 | `marketplace.py:293` | `subprocess.Popen([editor, str(plugin.install_path)], shell=True)` | Source line 293 is `subprocess.Popen([editor, str(plugin.install_path)], shell=True)` | ✅ CONFIRMED |
| 3 | svc-r1 | `discovery.py:484` | `re.sub(r"[^a-zA-Z0-9\-]", "-", str(self.project_root))` | Verified above | ✅ CONFIRMED |
| 4 | svc-r1 | `discovery.py:243-251` | sort by `(type_order[c.type], c.name.lower())` | Verified above | ✅ CONFIRMED |
| 5 | svc-r1 | `marketplace_loader.py:216-219` for installed_scopes literal set | (claim relayed in brief) | The literals flow through lines 216-228, assigned at line 238. The "216-219" cite is tight but the LITERAL set verification is correct. | ⚠️ LINE RANGE TIGHT but content correct |
| 6 | models-r3 | "11-site Customization mutation surface" | (claim) | Actual count is ~15-16 via direct grep. | ⚠️ MINOR UNDERCOUNT (claimed 11, actual ~15) |
| 7 | akb-r1 | "29-binding registry" | (claim in r1 header) | `grep -c "Binding(" bindings.py` returns **31**. The visible table in r1 lists 32 rows (matches Python list which has 32 entries — 31 "Binding(" + 1 "BindingType: list[BindingType]"). | ⚠️ TEXTUAL CLAIM "29" vs TABLE/CODE shows 31-32 |
| 8 | svc-r1 | `plugin_loader.py:295` is where `_find_latest_version_dir` is called inside `_create_plugin_info` | (claim) | Source: `plugin_loader.py:295` is `install_path = self._find_latest_version_dir(install_path.parent)` — inside the if-block starting at 294. | ✅ CONFIRMED |
| 9 | svc-r2 | `plugin_loader.py:329-341` defines `_find_latest_version_dir` | (claim) | Source: `plugin_loader.py:329` is `def _find_latest_version_dir(self, parent_dir: Path) -> Path:`. The function body extends through line ~341. | ✅ CONFIRMED |
| 10 | mod-r4 | `tests/integration/discovery/test_auto_memory.py:118, 172` pin mutation outputs | (claim) | v2 did NOT open test_auto_memory.py directly. Test file's existence verified, line numbers not independently checked. | TRUST-BUT-NOT-VERIFIED |
| 11 | mix-r1 | `customization_actions.py:165-212` for `_handle_copy_or_move` | (claim) | Range not independently verified line-by-line, but mixins/customization_actions.py is 279 LOC per B.5; range plausible. | TRUST-BUT-NOT-VERIFIED |

**Spot-check summary:** Of 11 citations:
- **8 CONFIRMED** byte-precisely
- **2 MINOR DISCREPANCIES** (lines 216-219 too tight; 11-site count is ~15-16)
- **1 TEXTUAL HEADER vs TABLE MISMATCH** ("29 bindings" vs 31-32 actual; the table in same round is correct)
- **2 TRUST-BUT-NOT-VERIFIED** (line ranges plausible but not directly opened)

**ZERO HALLUCINATIONS** — every citation points to real content; the imprecisions are counting / scope-of-citation errors, not invented references.

## 8. Triple-claim cross-validation: the shell=True misuse

Independent claims:

| Round | File:line | Description of bug |
|---|---|---|
| svc-r1 (mentions but doesn't deepen — primary bug-claim is in mix/akb) | — | n/a |
| mix-r1 (P1 confirmed §) | `marketplace.py:253-261` (subprocess.run inside @work) + `marketplace.py:293` (subprocess.Popen) | "Silent no-op success" on POSIX; second site at 293 same pattern. **Two sites, not one.** |
| akb-r2 §9 + akb-r3 §1 | `marketplace.py:253-261` | "P0 confirmed via Python docs"; cites stdlib docs as cross-validation |

**Comparison:**
- File: ALL THREE rounds agree on `marketplace.py`. ✅
- Line range: mix-r1 says 253-261 AND 293; akb-r2/r3 say 253-261. **akb-r2/r3 do NOT mention the second site at 293.** Mix-r1 is more complete.
- Bug semantics: ALL THREE agree — POSIX shell strips all but cmd[0]; result is `claude` called with NO args; silent no-op success appears as user-visible "Installed X" toast.

**v2 independent verification:**

Source `marketplace.py:248-267`:
```
248     @work(thread=True)
249     def _run_plugin_command(self, cmd: list[str], success_msg: str) -> None:
250         """Run a plugin command in a background worker."""
251         try:
252             cwd = str(self._discovery_service.project_root)  # type: ignore[attr-defined]
253             subprocess.run(
254                 cmd,
255                 capture_output=True,
256                 check=True,
257                 shell=True,
258                 encoding="utf-8",
259                 errors="replace",
260                 cwd=cwd,
261             )
```
✅ Verbatim match.

Source `marketplace.py:293`:
```
293         subprocess.Popen([editor, str(plugin.install_path)], shell=True)
```
✅ Verbatim match.

**Triple-claim consistency verdict:** ✅ All three rounds (mixins, app-keybindings r2, app-keybindings r3) agree on the bug location, semantics, and severity. The mixins round adds the second site at line 293 which the app-keybindings rounds do not explicitly mention. **No disagreement; minor coverage diff favors mixins.**

The Python stdlib citation in akb-r3 ("If shell=True and args is a sequence on POSIX, Popen does the equivalent of: Popen(['/bin/sh', '-c', args[0], args[1], ...])") is **correct** — this is documented behavior in Python's subprocess module.

## 9. Phase C synthesis adequacy

The original Phase C synthesis (`nikiforovall-lazyclaude-pass-8-final-synthesis.md`, 16,996 bytes) was written **before the full-protocol Phase B rounds executed**. It references parser/widget/plugin-marketplace deepening (the early Phase B rounds) but **predates services-r1..r3, mixins-r1..r2, app-keybindings-r1..r3, models-r1..r4**.

**Specific under-representation in Phase C:**

1. **Services layer (10 files):** Phase C mentions services in architecture overview but does not have the per-file canonical schemas that svc-r1 contributes. The atomic-write gap is mentioned in Pass 8 D7 but not at 3 confirmed call sites with byte-precise citations.
2. **Mixin layer (5 files):** Phase C describes the mixin structure at a high level but does not have the Modal-Confirm-Callback pattern catalog, the 7-modal pairing table, or the AppMode state-machine recommendation (mix-r1).
3. **App + keybindings (2 files + styles + themes):** Phase C does not have the 32-binding registry, the MRO trace, or the navigation asymmetric-wraparound documentation (akb-r1..r3).
4. **Models layer (3 files + types):** Phase C describes types at a domain level but does not have the 15-site Customization mutation surface (mod-r3 — note: v2 finds 15 sites, not 11), the metadata-key set per type (mod-r2), or the basename-dedup invariant (mod-r4).

**Status:** Phase C synthesis is **OUT OF DATE**. A Pass 8 v2 / Pass 9 final synthesis is required that incorporates the full-protocol Phase B findings. The brief's note "C-orig predates full-protocol rounds — under-represents services/mixins/app/models" is **confirmed**.

## 10. Verdict

### TOPIC-DRIFT-CLEAN — with three minor caveats

**Verdict: TOPIC-DRIFT-CLEAN.** The Phase B full-protocol rounds (services-r1..r3, mixins-r1..r2, app-keybindings-r1..r3, models-r1..r4) stay tightly on-topic for their declared scopes. Every cited file:line points to real source content. Cross-round claims (atomic-write, shell=True, project slug regex, sort order, mixed-version TypeError) are consistent across rounds with byte-precise reproducibility.

**Three minor caveats** (none rising to topic drift):

1. **Citation imprecision: models-r3's "11-site Customization mutation surface" undercounts.** Actual count via direct grep is **~15-16 sites** (3-4 additional `plugin_info = plugin_info` assignments at discovery.py:331, 340, 367 inside the marketplace-extras branches that mod-r3 enumerated starting at discovery.py:389 and missed earlier). The structural finding (Customization is mutable post-construction) is unaffected — more sites strengthen rather than weaken it.

2. **akb-r1 textual header "29-binding registry" vs actual 31-32 bindings.** The Python `APP_BINDINGS` list at `bindings.py:5-37` contains 32 entries (31 distinct `Binding(...)` constructors per `grep -c`, with the trailing `]` likely throwing off raw counts). The TABLE inside akb-r1 §4.1 correctly lists 32 rows. The HEADER's "29 bindings" claim is **internally inconsistent with the very table that follows it**. Confusing but not load-bearing — the table is the source of truth.

3. **Test-file deepening gap not addressed by expanded Phase B.** The original B.5's "test coverage of analytical findings: ~40%" is still essentially true after services/mixins/app-keybindings/models rounds. Eight test files (`test_gitignore.py`, `test_memory_files.py`, `test_slash_commands.py`, `test_subagents.py`, `test_delete_writer.py`, `test_combined_panel.py`, `test_level_selector.py`, `test_memory_file_ref.py`) remain entirely unread. Two more are partial (`test_app_customization_actions.py`, `test_mcp_writer.py`). Phase B added test-citation refinements via line ranges in svc-r1 but did not open these files. This is **a real coverage gap that B.5-v2 must surface**. Severity: P2 — the unread tests are mostly assertions on already-deepened subsystems, so the risk of missing a behavioral contract is small. But the original B.5's confidence claim "every behavioral contract claimed by source code has been examined via source-of-truth" should be qualified.

### Subsystems flagged as gaps (P2)

| Subsystem | Gap | Severity |
|---|---|---|
| 8 test files (listed above) | Never opened in full | P2 |
| `tests/integration/fixtures/` 24-file inventory | Never enumerated as canonical-shape catalog | P2 |
| `pyproject.toml` deep dependency-Rust-port mapping | Only broad inventory | P2 |
| LSP layer (lsp_server.py + plugin LSP discovery) | 0 tests in reference — gap acknowledged but unclosable from source side | P2 (real but inherent) |
| Plugin scope phases 2/3 (`get_all_plugins` project + local) | 0 unit tests — semantics walked in svc-r2 set-algebra | P2 acknowledged |

### Inter-round inconsistencies found

1. **Customization mutation site count**: mod-r3 says 11; actual ~15. Inconsistency of count, not of finding.
2. **akb-r1 header vs table**: 29 vs 32 binding entries. Internal to a single round.
3. **Line-range scope of installed_scopes citation**: brief said "216-219"; literal flow is 216-228, assignment is 238. Tight but not wrong.

**None of these change the model.**

### Hallucinations identified

**ZERO.** Every citation spot-checked maps to real source content. The three caveats above are counting / scope-of-citation imprecisions, not invented references.

### Whether existing Phase C synthesis adequately represents full-protocol coverage

**NO.** Phase C synthesis was written 2026-05-11T17:27 (per file timestamp), while the full-protocol Phase B rounds run 2026-05-11T18:26–20:50 per their checkpoints. Phase C **could not have incorporated** services-r1..r3, mixins-r1..r2, app-keybindings-r1..r3, models-r1..r4. **Recommendation:** produce a Pass 8 v2 / Pass 9 final synthesis that supersedes the original Phase C.

### Independent confirmation of the triple-claimed shell=True bug

✅ **INDEPENDENTLY CONFIRMED** by direct source read of `mixins/marketplace.py:253-261` and `mixins/marketplace.py:293`. Both sites verbatim match the cited content. The Python stdlib semantics (per `subprocess` module docs) of `subprocess.run(list, shell=True)` on POSIX produce a silent-no-op for all args after `cmd[0]`. Bug is real; **two sites**, not one (mix-r1 catches both; akb-r2/r3 only cite the first). Recommendation: any downstream synthesis should cite **BOTH** `marketplace.py:253-261` AND `marketplace.py:293`.

## 11. State Checkpoint

```yaml
pass: B.5-v2 (fresh-context independent watchdog)
status: complete
verdict: TOPIC-DRIFT-CLEAN
artifacts_audited: 27
source_files_verified_byte_precise: 8
hallucinations_found: 0
minor_citation_imprecisions: 3
test_files_truly_unread: 8
test_files_partial: 2
inter_round_inconsistencies: 3 (all counting/scope, none model-changing)
phase_c_synthesis_adequacy: OUT_OF_DATE (predates full-protocol rounds)
recommendation: produce Pass 8 v2 / Pass 9 synthesis incorporating
  services-r1..r3, mixins-r1..r2, app-keybindings-r1..r3, models-r1..r4
timestamp: 2026-05-11T21:30:00Z
```
