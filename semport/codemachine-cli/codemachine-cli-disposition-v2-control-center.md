---
document_type: gene-source-disposition
project: monocle
producer: architect
status: draft
version: "1.0"
timestamp: 2026-06-03T00:00:00Z
gene_source: codemachine-cli
disposition_pass: v2 (D-236 control-center pivot)
supersedes: original disposition embedded in domain-monocle-vision-synthesis.md v1.1.2
traces_to: NEXT-SESSION-PIVOT.md §5
---

# Gene-Source Disposition v2: codemachine-cli (Control-Center Lens)

## Vision Lens Applied

monocle v1 (re-baselined): full TUI control center — multi-harness support via EngineModule
abstraction. codemachine-cli is the gene source for the EngineModule interface and the
second harness beyond Claude Code. The pivot makes EngineModule more critical by requiring
the trait to support spawn/launch lifecycle, not just detect/enrich/on_hook.

## Original Disposition Summary

The original vision adopted codemachine-cli as the primary source for:
- EngineModule trait (40-line interface defining what "be an AI coding harness" means)
- EngineRegistry (singleton with register/get/getAll — first-write-wins)
- Headless-CLI-as-transport (spawn as child process; no SDK integration)
- Multi-harness abstraction (Claude Code + CodeMachine as first two built-ins)
- 7-step routing decision tree (not adopted — CCR handles routing)

The original vision left behind:
- Declarative workflow templates (vsdd-factory is the workflow gene)
- Workflow FSM (not monocle's concern)
- Three-axis scenario dispatcher (not monocle's concern)
- Two signal channels (MCP + filesystem) (not monocle's concern)
- Zero tests: flagged as P0 risk; monocle's Rust port must add tests

## Disposition by Capability Area

### 1. EngineModule Trait (originally ADOPT, partially built)

**New verdict: ADOPT + EXTEND (lifecycle methods required).**

The original EngineModule in monocle's vision (vision-synthesis §EngineModule):
```rust
pub trait EngineModule: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn metadata(&self) -> EngineMetadata;
    fn detect(&self, proc: &ProcessSnapshot) -> bool;
    async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession;
    async fn on_hook(&self, event: HookEvent) -> HookResponse;
}
```

The control-center pivot requires EngineModule to provide the SPAWN RECIPE — the information
needed to launch a session of this harness type:

```rust
pub trait EngineModule: Send + Sync + 'static {
    fn id(&self) -> &'static str;
    fn metadata(&self) -> EngineMetadata;

    // EXISTING: detection + observation
    fn detect(&self, proc: &ProcessSnapshot) -> bool;
    async fn enrich(&self, proc: &ProcessSnapshot) -> EnrichedSession;
    async fn on_hook(&self, event: HookEvent) -> HookResponse;

    // NEW: spawn recipe (what the daemon needs to launch a session)
    fn spawn_recipe(&self, opts: &SpawnOptions) -> SpawnRecipe;
    // SpawnRecipe { binary: PathBuf, args: Vec<String>, env: HashMap<String, String> }
    // The daemon's session manager takes SpawnRecipe + worktree path → portable-pty spawn
}

pub struct SpawnOptions {
    pub worktree_path: PathBuf,
    pub hooks_settings_path: PathBuf,  // injected via --settings
    pub initial_prompt: Option<String>,
    pub model: Option<String>,         // overrides profile default
    pub extra_env: HashMap<String, String>,
}

pub struct SpawnRecipe {
    pub binary: PathBuf,    // e.g., which("claude") → /usr/local/bin/claude
    pub args: Vec<String>,  // e.g., ["--settings", "/path/to/hooks-settings.json"]
    pub env: HashMap<String, String>,  // ANTHROPIC_BASE_URL if CCR, etc.
}
```

Rationale for the trait split (spawn_recipe on trait vs. on session manager):
- The EngineModule knows its own binary name, required CLI flags, and environment variables.
- The session manager knows the worktree path, hooks config path, and PTY sizing.
- The two combine: `session_manager.launch(module.spawn_recipe(opts), worktree, pty_size)`.
- This mirrors codemachine's `EngineRunOptions` (which includes workingDir, env, timeout)
  passed to `EngineModule.run()` — except monocle's version is for PTY spawn, not headless
  subprocess spawn.

From codemachine's 7-engine inventory:
- `ClaudeCodeModule.spawn_recipe()` → `binary=which("claude"), args=["--settings", hooks_path]`
- `CodeMachineModule.spawn_recipe()` → `binary=which("cm"), args=[...]` (TBD for Phase 2)
- Future engines: same pattern.

### 2. EngineRegistry (originally ADOPT)

**New verdict: ADOPT (confirmed).** The singleton registry with first-write-wins duplicate
prevention and `getAll()` sorted by `order` remains correct for monocle. No change needed.

In Rust: `OnceLock<RwLock<BTreeMap<&'static str, Box<dyn EngineModule>>>>` keyed by `id()`,
sorted by metadata.display_order (a new field for UI ordering, like codemachine's `order`).

### 3. Headless-CLI-as-Transport (originally ADOPT)

**New verdict: PARTIALLY CHANGED.**

codemachine uses headless CLI spawn (Bun.spawn with non-interactive flags, JSON stream output)
because CodeMachine reads the LLM response programmatically. monocle does NOT do this — monocle
embeds the terminal for the USER to read. The headless-CLI concept applies for the binary spawn
decision but NOT for the output handling:
- monocle spawns `claude` as a PTY child (not headless subprocess) so the user can see the
  terminal output in the embedded terminal pane.
- The `onData` / `onTelemetry` / `onSessionId` callbacks from codemachine's EngineRunOptions
  are NOT needed — monocle reads PTY bytes via the vt100 parser, not structured JSON streams.
- The only structured channel monocle uses from the session is the HOOK protocol (HTTP POSTs
  to the daemon). This is already built.

The headless-CLI pattern is LEFT BEHIND for session PTY embedding. It may be revisited if
monocle ever wants to inspect or automate harness output (not a v1 requirement).

### 4. Declarative Workflow Templates + FSM (originally LEAVE BEHIND)

**CONFIRMED Leave-behind.** The pivot does not bring workflow execution into scope.
vsdd-factory is the workflow gene; monocle observes workflow state (Workflow plane) but
never executes it.

### 5. Three-Axis Scenario Dispatcher (originally LEAVE BEHIND)

**CONFIRMED Leave-behind.** monocle does not need the three-axis routing logic (interactive
× autoMode × chainedPrompts → 8 scenarios). Sessions are launched by the user from the TUI;
the routing decision is the user's.

### 6. MCP Router / Signal Channels (originally LEAVE BEHIND)

**CONFIRMED Leave-behind.** codemachine's in-process MCP server that fans out to built-in
and user-defined MCP servers is CodeMachine-specific orchestration. monocle does not need
an MCP router.

### 7. Zero Tests Risk (originally P0 flagged)

**CONFIRMED P0 for Rust port.** All monocle implementations of EngineModule must have
integration tests via the DTU-clone + PtySpawner mock seam. The Rust port inherits
CodeMachine's behavioral patterns but adds the test coverage that CodeMachine lacks.

ClaudeCodeModule.spawn_recipe() and its integration with the session manager (mock PTY spawn
→ assert correct binary/args/env) must be integration-tested.

### 8. CodeMachineModule as Second EngineModule (originally Phase 2 scope)

**New verdict: Phase 2 scope (confirmed).** The re-baselined v1 delivers ClaudeCodeModule
with full launch/manage/PTY embedding. CodeMachineModule as a second EngineModule comes in
a follow-on phase. The trait extensibility makes adding it mechanical.

## Summary Table

| Capability | Original Verdict | New Verdict | Change? |
|-----------|-----------------|-------------|---------|
| EngineModule trait | ADOPT (built) | ADOPT + EXTEND (spawn_recipe method required) | Extended |
| EngineRegistry | ADOPT | ADOPT (confirmed) | Confirmed |
| Headless-CLI spawn | ADOPT | PARTIALLY LEFT BEHIND (PTY spawn, not headless; hook protocol is the structured channel) | Clarified |
| Workflow templates + FSM | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Three-axis scenario dispatcher | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| MCP router | LEAVE BEHIND | LEAVE BEHIND | Confirmed |
| Zero tests (P0) | ADOPT fix | ADOPT fix (PtySpawner mock seam + DTU discipline) | Confirmed |
| CodeMachineModule | ADOPT (Phase 2) | ADOPT (Phase 2 confirmed) | Confirmed |

## Net Assessment

codemachine-cli's primary contribution is the EngineModule abstraction, which the control-
center pivot makes more concrete: the trait gains a `spawn_recipe()` method that provides
the daemon with the binary, args, and env needed to launch a PTY session for any harness.

This is the most significant architectural extension this disposition introduces:
`SpawnRecipe` (binary + args + env) as a new EngineModule output. It is clean, testable,
and directly enables the multi-harness launch capability of the control-center.
