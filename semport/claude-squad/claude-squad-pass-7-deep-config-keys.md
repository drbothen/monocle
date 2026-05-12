# Pass 7 (Deepening): Configuration + Keys + App glue — claude-squad

## Scope

Verify configuration layer (config.json vs state.json semantics), key bindings as a layer, and the cmd/Executor seam.

## Configuration Files

### `~/.claude-squad/config.json` — user-facing

| Field | Type | Default source | Notes |
|-------|------|----------------|-------|
| `default_program` | string | `GetClaudeCommand()` result, fallback `"claude"` | Either a profile name or a literal command |
| `auto_yes` | bool | false | |
| `daemon_poll_interval` | int | 1000 (ms) | |
| `branch_prefix` | string | `<lowercase-username>/` | Slash-suffixed prefix for new branches |
| `profiles` | []Profile (optional) | nil | Each: `{name string, program string}` |

If the file doesn't exist on first run, `DefaultConfig()` is generated and saved to disk (`/config/config.go:163-171`).

If the file exists but is invalid JSON, `LoadConfig` logs an error and returns `DefaultConfig()` WITHOUT overwriting the bad file. Subsequent runs would still read the broken file.

### `~/.claude-squad/state.json` — program-managed

```go
type State struct {
    HelpScreensSeen uint32          `json:"help_screens_seen"`
    InstancesData   json.RawMessage `json:"instances"`
}
```

`InstancesData` is a JSON blob (array of `InstanceData`). The fact that it's `json.RawMessage` means `State` doesn't unmarshal the instances itself — `Storage` does that lazily.

`HelpScreensSeen` is a bitmask of dismissed help screens.

### Interface decoupling

`config.InstanceStorage` (`/config/state.go:17-24`) declares Save/Get/DeleteAll for instance data. `State` implements it. `session.Storage` accepts it as an interface. This makes Storage testable without disk I/O, in theory — but there are no in-memory mocks of InstanceStorage in the test suite.

### Profile semantics

`GetProgram` (`/config/config.go:50-59`):
- If any profile's Name equals DefaultProgram → return that profile's Program
- Else → return DefaultProgram as literal

`GetProfiles` (`/config/config.go:64-82`):
- If no profiles → return single synthetic `[{Name: DefaultProgram, Program: DefaultProgram}]`
- Else → return profiles with the default-matching one moved to position 0

This sort-by-default-first is for the profile picker UX (default appears as the leftmost option).

### Claude-finding heuristic

`GetClaudeCommand` (`/config/config.go:113-153`):
1. Determine shell from `$SHELL` (fallback `/bin/bash`)
2. For zsh/bash: `source ~/.zshrc &>/dev/null || true; which claude`
3. For other shells: just `which claude`
4. If output contains `aliased to`, `->`, or `=`: parse the right-hand side as the actual path
5. Else: `exec.LookPath("claude")`
6. Else: error

This is unusually defensive code — it's trying to find claude even when it's only available as a shell alias (common because Anthropic distributes claude as a JS package that may be installed weirdly). The code reaches into `.zshrc`/`.bashrc` which is a non-trivial decision.

## Keys Layer

`/keys/keys.go` is 119 LOC with two global maps:

1. `GlobalKeyStringsMap` (string → KeyName): the actual binding source. E.g., `"k" → KeyUp`, `"D" → KeyKill`. 18 entries.
2. `GlobalkeyBindings` (KeyName → key.Binding): for help-text rendering. 13 entries.

**Asymmetry:** `KeyQuit` is in `GlobalKeyStringsMap` (mapped from `"q"`), but the actual quit logic is hardcoded `msg.String() == "q"` in `handleKeyPress` (`/app/app.go:597`). So the keymap entry is unused. Similarly, `KeySubmitName` is in GlobalkeyBindings only (used for menu rendering when in stateNew), and there's a comment in app.go acknowledging the asymmetry: `TODO: cleanup: when you press enter on stateNew, we use keys.KeySubmitName. We should unify the keymap` (`/app/app.go:374`).

This is a minor architectural debt — three different ways keys are mapped:
1. Through `GlobalKeyStringsMap` + the dispatch switch in `handleKeyPress`
2. Through hardcoded string compares (`q`, `ctrl+c`, `esc`, `tab`, etc.)
3. Through overlay-internal HandleKeyPress methods that handle their own key sets

## Executor Seam (`/cmd/cmd.go`)

```go
type Executor interface {
    Run(cmd *exec.Cmd) error
    Output(cmd *exec.Cmd) ([]byte, error)
}
type Exec struct{}
func (e Exec) Run(cmd *exec.Cmd) error            { return cmd.Run() }
func (e Exec) Output(cmd *exec.Cmd) ([]byte, error) { return cmd.Output() }
func MakeExecutor() Executor                       { return Exec{} }
```

32 LOC. Used only by `tmux.TmuxSession` (via `NewTmuxSessionWithDeps`). The git package does NOT use this seam — it constructs `exec.Cmd` directly and calls `.Run()`/`.CombinedOutput()` inline.

**Implication:** git operations cannot be mocked easily. Tests of git code paths require an actual git binary. This is why `/session/git/util_test.go` only tests `sanitizeBranchName` (pure function) — there's no test for `Setup`, `Cleanup`, `Push`, `Diff` etc.

The TUI tests that DO need git (preview_test.go, terminal_test.go) `git init` actual repos in `t.TempDir()`. Heavy but it works.

## App-Layer Glue (high level)

The `home` struct is large (10 fields just for UI components, 5 fields for state, 5 fields for config/storage). It mirrors a pattern in many BubbleTea apps: the top-level Model owns the world. There's no Redux-style state separation; there's no slice/reducer pattern. The Update method is one big switch with subcase handlers.

The flow of one keypress:

```
tea.KeyMsg
  ↓
home.Update (case tea.KeyMsg)
  ↓
home.handleKeyPress(msg)
  ↓
handleMenuHighlighting   (sets keyDown for visual feedback, sometimes returns early)
  ↓
state-dispatch:
  if stateHelp → handleHelpState
  if stateNew → inline switch on key types
  if statePrompt → delegate to textInputOverlay.HandleKeyPress
  if stateConfirm → delegate to confirmationOverlay.HandleKeyPress
  else → switch on KeyName (KeyUp, KeyDown, KeyTab, KeyKill, KeySubmit, KeyCheckout, KeyResume, KeyEnter, etc.)
  ↓
returns (tea.Model, tea.Cmd)
```

`handleKeyPress` is ~410 LOC long. Single function. Readable but dense.

## Cobra Layer

The cobra root has 4 subcommands (root/RunE, reset, debug, version). 3 flags on root: `--program`/`-p`, `--autoyes`/`-y`, `--daemon` (hidden).

Notable: `daemon` is a flag on the root command, not a subcommand. So `cs --daemon` works but `cs daemon` doesn't. This is because the daemon is internal-use-only.

## Delta Summary

- New items added: claude-finding heuristic detail, the three-ways-of-handling-keys debt, the Executor-not-used-in-git observation, the git-tests-must-use-real-git-binary consequence, the "broken config.json never auto-recovers" failure mode
- Existing items refined: state vs config file separation, the JSON-RawMessage lazy unmarshalling pattern
- Remaining gaps: none

## Novelty Assessment

Novelty: **NITPICK**

Justification: All Big-Picture features already named. Findings are refinements.

## Convergence Declaration

Configuration + Keys + App glue deepening has converged.

## State Checkpoint

```yaml
pass: 7
subsystem: config-keys-app
round: 1
status: complete
novelty: NITPICK
timestamp: 2026-05-11T19:55:00Z
```
