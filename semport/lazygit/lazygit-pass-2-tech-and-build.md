# Pass 2 — Tech & Build (SCOPED)

## Module declaration
`/Users/jmagady/Dev/monocle/.reference/lazygit/go.mod:1` — `module github.com/jesseduffield/lazygit`, `go 1.25.0`. The `ignore ./test` directive on line 7 keeps `gofumpt` out of the integration-test fixture tree.

## Build pipelines (all read-only inspection)
- `Makefile` (60 lines) — primary developer entry: `build`, `test`, `format`, `lint`, `generate-cheatsheet`, `bump-gocui`.
- `justfile` — alternative recipe runner mirroring make targets (`just build`, `just test`).
- `default.nix`, `flake.nix`, `flake.lock`, `shell.nix` — Nix dev-shell support.
- `Dockerfile` — small Alpine-based runtime container; not a TUI concern.
- `.goreleaser.yml` — multi-arch release pipeline (`build-source` set to `binaryBuild` for goreleaser builds, see `pkg/config/app_config.go:521`).

## Test framework
- `stretchr/testify` (assert / require) — declared in `AGENTS.md` as the preferred assertion style.
- `pkg/integration/` runs full TUI integration tests by driving the gocui event loop via `gui_driver.go:165`. **Out of scope** for this ingest — one mention only.
- Unit-style `_test.go` files live next to source (e.g. `pkg/gui/controllers/helpers/window_arrangement_helper_test.go` at 729 LOC is the canonical example of a high-value test file in scope).

## Key runtime substrates
| Substrate | Where | Role |
|---|---|---|
| gocui fork | `/Users/jmagady/Dev/monocle/.reference/lazygit/pkg/gocui/` | Terminal cell renderer, view abstraction, event loop, keybinding registry. |
| tcell/v3 | dep | Underlies gocui (Unicode-aware terminal I/O). |
| boxlayout | dep (`lazycore`) | Recursive box-tree layout solver (rows / columns / weights / sizes). |
| go-deadlock | dep | Drop-in `sync.Mutex` replacement that detects lock-order inversions. |

## ratatui-translation notes (saved here so later passes can refer back)
- gocui `*View` ~ ratatui `Buffer` + viewport state. Cursor / origin / wrap / frame are owned by the view, not by the painter, mirroring ratatui's stateful widgets (`StatefulWidget`).
- gocui event loop is single-threaded and serialises every handler on a UI thread (`pkg/gui/types/common.go:73` — `OnUIThread`). Ratatui's idiomatic loop is the same (single-thread + crossbeam channel for background tasks); the architectural translation is direct.
- `boxlayout.Box{Direction, Weight, Size, Children}` ≈ ratatui `Layout::default().direction(...).constraints([...])`. Weight ≈ `Constraint::Ratio`, Size ≈ `Constraint::Length`.

## State checkpoint
pass: 2
status: complete
scope: in-scope tech only (gocui, boxlayout, tcell, fuzzy, xdg)
