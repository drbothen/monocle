# Pass B.5 — Coverage Audit

## In-scope items requested vs covered

| Requested in-scope item | Covered where | Status |
|---|---|---|
| 1. pkg/gui/ TUI architecture | Pass 3, Pass B-tui-architecture-r1 | COVERED |
| 2. Key dispatch (context-aware bindings) | Pass 5 (BC 1-7), Pass B-key-dispatch-r1 | COVERED |
| 3. pkg/gui/popup popup/modal patterns | Pass 4 (MenuItem/CreatePopupPanelOpts), Pass B-popup-patterns-r1 | COVERED |
| 4. Help overlay (`?`) telescope-style | Pass B-help-overlay-and-filter-r1 § "The `?` help" | COVERED |
| 5. Filter system (`/` fzf-style) | Pass 5 (BC-18), Pass B-help-overlay-and-filter-r1 § "Filter ('/')" | COVERED |
| 6. Custom commands framework | Pass 5 (BC-21), Pass B-custom-commands-and-theming-r1 | COVERED |
| 7. Theming (runtime, tokens) | Pass 5 (BC-12), Pass B-custom-commands-and-theming-r1 § "Theming" | COVERED |
| 8. Status panel / app footer | Pass 5 (BC-16), Pass 3 § options bar | COVERED |
| 9. Log viewer / scrollback | Pass 5 (BC-19), Pass B-log-viewer-and-scrollback-r1 | COVERED |
| 10. Layout system | Pass 3 § "Window arrangement", Pass 5 (BC-15) | COVERED |
| 11. Top-level config loading | Pass 5 (BC-13, BC-14), Pass B-config-loading-r1 | COVERED |

## Out-of-scope items: confirmed cited and not deepened

| Out-of-scope | Cited where | Status |
|---|---|---|
| pkg/commands/* (git commands) | Pass 1 OUT-OF-SCOPE table | COMPLIANT |
| pkg/integration/ | Pass 1 + Pass 2 (one mention only) | COMPLIANT |
| vendor/ | Pass 1 | COMPLIANT |
| docs/ demos | Pass 1 | COMPLIANT |
| GitHub/GitLab integrations specifically | Pass 1 + Pass 6 (PR cache only as state-file example) | COMPLIANT |
| pkg/snake/ | Pass 1 | COMPLIANT |
| pkg/i18n/ | Pass 1 + Pass 4 (mentioned as TranslationSet seam only) | COMPLIANT |
| pkg/cheatsheet/, pkg/jsonschema/ | Pass 1 + Pass 6 (one mention each) | COMPLIANT |
| pkg/tasks/ | Pass 1 + Pass 6 (ViewBufferManager named, not opened) | COMPLIANT |

## Behavioural contracts inventory

25 BC drafts in Pass 5. Coverage by in-scope area:
- Key dispatch: BC-1, 2, 3, 4, 5, 6, 7 (7)
- Popups: BC-8, 9, 11, 24 (4)
- Context stack: BC-10, 22 (2)
- Theming: BC-12 (1)
- Config: BC-13, 14 (2)
- Layout: BC-15 (1)
- Footer/status bar: BC-16 (1)
- Help/menu: BC-17, 23 (2)
- Filter/search: BC-18 (1)
- Log/extras: BC-19, 25 (2)
- Toast: BC-20 (1)
- Custom commands: BC-21 (1)

All categories covered.

## File:line citation density check
Spot-check 12 random claims in the deepening rounds:

- `pkg/gui/keybindings.go:448-454` (SetKeybinding) ✓ verified
- `pkg/gui/keybindings.go:411` (custom-prepend) ✓ verified
- `pkg/gui/context.go:91-95` (side-context push semantics) ✓ verified
- `pkg/gui/controllers/helpers/confirmation_helper.go:198-205` (popup serialise) ✓ verified
- `pkg/gui/controllers/helpers/confirmation_helper.go:116-122` (cascade offset) ✓ verified
- `pkg/gui/services/custom_commands/handler_creator.go:47-131` (prompt chain) ✓ verified
- `pkg/gui/services/custom_commands/handler_creator.go:295-340` (Output routing) ✓ verified
- `pkg/gui/controllers/options_menu_action.go:64-77` (3-section bucketing) ✓ verified
- `pkg/gui/context/menu_context.go:83-91` (@ prefix detection) ✓ verified
- `pkg/gui/controllers/helpers/search_helper.go:223` (no debounce) ✓ verified
- `pkg/gui/context.go:117-119` (search context exception) ✓ verified
- `pkg/config/app_config.go:269-277` (renames list) ✓ verified

All twelve verified.

## Missing or low-density areas

- The gocui internals (`pkg/gocui/`) are referenced but not deepened. **By design** (it's a vendored fork; monocle uses ratatui+crossterm instead). One-line summary in Pass 2 is sufficient.
- `pkg/tasks/ViewBufferManager` — mentioned, not deepened. Acceptable: it's a perf optimisation, not a UX pattern.
- `pkg/utils/yaml_utils/` migration helpers — mentioned indirectly via `migrateUserConfig`. The transform-node API is a useful pattern but easily reconstructible.

None of these gaps block monocle's ability to translate the lazy* TUI conventions.

## Compliance with constraints

- Absolute paths: every cited file uses `/Users/jmagady/Dev/monocle/.reference/lazygit/...` form. ✓
- File:line citations: dense throughout (avg ~3 citations per finding). ✓
- Table cells match headers: spot-checked Pass 1, Pass 4. ✓
- Reserved adversarial headers (`## Novelty Assessment`, `## Delta Summary`) avoided in Phase C. Will be replaced with neutral wording. ✓ planned
- No commits made to anywhere. ✓
- No path overlap with other sibling agents (zellij, claude-squad, claude-code-router) — they write to their own subdirs. ✓

## Audit verdict
Coverage is complete for the SCOPED ingest. Ready for Pass B.6 (extraction validation) and Pass 8 (final synthesis).
