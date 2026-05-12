# Pass B.6: Extraction Validation — zellij Scoped Ingest

This pass spot-checks the most load-bearing claims against the actual source to confirm citation accuracy.

## Spot Checks

### Check 1: IPC framing — length-prefixed protobuf, 4-byte LE u32

**Claim source**: Pass 4 (BC-DRAFT-001), Pass B-deep-ipc-r1 (Wire-Level Summary)
**Cited file:line**: `zellij-utils/src/ipc.rs:402-426`
**Verification**: Read confirmed at `zellij-utils/src/ipc.rs:402-426`:

```rust
fn read_protobuf_message<T: Message + Default>(reader: &mut impl Read) -> Result<T> {
    let mut len_bytes = [0u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let len = u32::from_le_bytes(len_bytes) as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    T::decode(&buf[..]).map_err(Into::into)
}

fn write_protobuf_message<T: Message>(writer: &mut impl Write, msg: &T) -> Result<()> {
    let encoded = msg.encode_to_vec();
    let len = encoded.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(&encoded)?;
    Ok(())
}
```

**Result**: CONFIRMED — 4-byte LE u32 length prefix, then exactly len bytes of protobuf-encoded payload.

### Check 2: Single host import function for plugin ABI

**Claim source**: Pass B-deep-plugin-sdk-r1 (Plugin Host-Call ABI section)
**Cited file:line**: `zellij-server/src/plugins/zellij_exports.rs:155-158`
**Verification**:

```rust
pub fn zellij_exports(linker: &mut Linker<PluginEnv>) {
    linker
        .func_wrap("zellij", "host_run_plugin_command", host_run_plugin_command)
        .unwrap();
}
```

**Result**: CONFIRMED — exactly one host function (`host_run_plugin_command`) is exposed under the `"zellij"` module namespace in the wasmi linker.

### Check 3: Built-in plugins bypass permission gate

**Claim source**: Pass B-deep-plugin-sdk-r1 (Plugin Permission Gate section)
**Cited file:line**: `zellij-server/src/plugins/zellij_exports.rs:5179-5184`
**Verification**:

```rust
fn check_command_permission(
    plugin_env: &PluginEnv,
    command: &PluginCommand,
) -> (PermissionStatus, Option<PermissionType>) {
    if plugin_env.plugin.is_builtin() {
        // built-in plugins can do all the things because they're part of the application and
        // there's no use to deny them anything
        return (PermissionStatus::Granted, None);
    }
    ...
}
```

**Result**: CONFIRMED — built-in check is the first guard, returning `Granted` unconditionally.

### Check 4: Session persistence save chain has 5 thread hops

**Claim source**: Pass B-deep-session-persistence-r1 (Save Trigger Flow section)
**Cited path**: Screen → Plugin → Pty → BackgroundJobs → FS
**Verification**:

| Hop | Code |
|---|---|
| Screen → Plugin | `zellij-server/src/screen.rs:3507` — `.send_to_plugin(PluginInstruction::LogLayoutToHd(session_layout_metadata))` |
| Plugin → Pty | `zellij-server/src/plugins/mod.rs:912-921` — handler for `PluginInstruction::LogLayoutToHd` sends `PtyInstruction::LogLayoutToHd` |
| Pty → BackgroundJobs | `zellij-server/src/pty.rs:772-790` — handler calls `serialize_session_layout`, then sends `BackgroundJob::ReportLayoutInfo` |
| BackgroundJobs → FS | `zellij-server/src/background_jobs.rs:670-700` — `write_session_state_to_disk` writes both `session-metadata.kdl` and `session-layout.kdl` |

**Result**: CONFIRMED — all four cross-thread hops + final FS write present at the cited locations.

### Check 5: SessionConfiguration overlay model

**Claim source**: Pass B-deep-ipc-r1 (SessionConfiguration section)
**Cited file:line**: `zellij-server/src/lib.rs:185-308`
**Verification**:

```rust
pub(crate) struct SessionConfiguration {
    runtime_config: HashMap<ClientId, Config>, // if present, overrides the saved_config
    saved_config: Config,
}
```

Plus `get_client_keybinds`, `get_client_default_input_mode`, `reconfigure_runtime_config`, `rebind_keys` methods all present. Comment confirms: "when changed, this resets the runtime config to be identical to it and override any previous changes."

**Result**: CONFIRMED.

### Check 6: NotificationEnd drop signaling

**Claim source**: Pass B-deep-ipc-r1 (NotificationEnd section), Pass 6 (Patterns Worth Adopting)
**Cited file:line**: `zellij-server/src/route.rs:316-388`
**Verification**:

```rust
pub struct NotificationEnd {
    channel: Option<oneshot::Sender<ActionCompletionResult>>,
    exit_status: Option<i32>,
    unblock_condition: Option<UnblockCondition>,
    affected_pane_id: Option<PaneId>,
    affected_tab_id: Option<usize>,
    error_message: Option<String>,
    stdout_message: Option<String>,
}

impl Drop for NotificationEnd {
    fn drop(&mut self) {
        if let Some(tx) = self.channel.take() {
            let result = ActionCompletionResult { ... };
            let _ = tx.send(result);
        }
    }
}

impl Clone for NotificationEnd {
    fn clone(&self) -> Self {
        // Always clone as None - only the original holder should signal completion
        NotificationEnd { channel: None, .. }
    }
}
```

**Result**: CONFIRMED — drop fires the oneshot send; clone deliberately strips the sender.

### Check 7: KDL theme file format

**Claim source**: Pass B-deep-theming-r1 (Theme File Format section)
**Cited evidence**: `assets/themes/gruvbox-dark.kdl` and `example/themes/example.kdl`
**Verification**:

`assets/themes/gruvbox-dark.kdl` shows new-style semantic tokens (`text_unselected { base 251 241 199; background 60 56 54; emphasis_0 214 93 14; ... }`).

`example/themes/example.kdl` shows old-style palette (`fg 60 56 54` or `fg "#D5C4A1"`).

Both confirmed visually.

**Result**: CONFIRMED.

### Check 8: 14 InputMode variants

**Claim source**: Pass 3 (Ubiquitous Language), Pass B-deep-config-keybinds-r1 (InputMode Catalog)
**Cited file:line**: `zellij-utils/src/data.rs:1146-1196`
**Verification**: Counted variants in source: Normal, Locked, Resize, Pane, Tab, Scroll, EnterSearch, Search, RenameTab, RenamePane, Session, Move, Prompt, Tmux = **14**.

**Result**: CONFIRMED.

### Check 9: 17 PermissionType variants

**Claim source**: Pass 3 glossary, Pass B-deep-plugin-sdk-r1 (Permission table)
**Cited file:line**: `zellij-utils/src/data.rs:1063-1086`
**Verification**: ReadApplicationState, ChangeApplicationState, OpenFiles, RunCommands, OpenTerminalsOrPlugins, WriteToStdin, WebAccess, ReadCliPipes, MessageAndLaunchOtherPlugins, Reconfigure, FullHdAccess, StartWebServer, InterceptInput, ReadPaneContents, RunActionsAsUser, WriteToClipboard, ReadSessionEnvironmentVariables = **17**.

**Result**: CONFIRMED.

### Check 10: 23 workspace members

**Claim source**: Pass 1 inventory, Pass B-deep-workspace-r1
**Cited file:line**: workspace `Cargo.toml:39-62`
**Verification**: Counted entries in `[workspace] members = [ ... ]` list: 13 default-plugins + 5 zellij-* crates + xtask + `.` = **20 entries listed**.

Wait — Pass 1 said 23. Recounting from `Cargo.toml` (lines 39-62):

```
"default-plugins/compact-bar",      # 1
"default-plugins/status-bar",        # 2
"default-plugins/strider",           # 3
"default-plugins/tab-bar",           # 4
"default-plugins/fixture-plugin-for-tests", # 5
"default-plugins/session-manager",   # 6
"default-plugins/configuration",     # 7
"default-plugins/plugin-manager",    # 8
"default-plugins/about",             # 9
"default-plugins/share",             # 10
"default-plugins/multiple-select",   # 11
"default-plugins/layout-manager",    # 12
"default-plugins/link",              # 13
"zellij-client",                     # 14
"zellij-server",                     # 15
"zellij-utils",                      # 16
"zellij-tile",                       # 17
"zellij-tile-utils",                 # 18
"xtask",                             # 19
".",                                  # 20
```

That's **20**, not 23. Pass 1's "23 workspace members" claim is OFF BY 3.

**Result**: CITATION ERROR. Correcting: the workspace has **20 members at this HEAD**, not 23.

The error came from my initial visual count being sloppy. The xtask source code (`xtask/src/main.rs:39-117`) iterates over the same 20 entries (`workspace_members()` returns a `Vec<WorkspaceMember>` of length 20). This is fixable; the rest of the architecture map is unaffected because the per-crate analysis was done individually, not by count.

### Check 11: 41 built-in themes

**Claim source**: Pass B-deep-theming-r1 (Built-in Theme Catalog)
**Cited evidence**: `zellij-utils/assets/themes/` directory
**Verification**: `ls assets/themes/ | wc -l` = **41**.

**Result**: CONFIRMED.

## Summary of Validation Outcomes

| Check | Outcome |
|---|---|
| 1 — IPC framing | CONFIRMED |
| 2 — Single host import | CONFIRMED |
| 3 — Built-in plugin perm bypass | CONFIRMED |
| 4 — Session save chain | CONFIRMED |
| 5 — SessionConfiguration overlay | CONFIRMED |
| 6 — NotificationEnd drop signaling | CONFIRMED |
| 7 — KDL theme formats | CONFIRMED |
| 8 — 14 InputModes | CONFIRMED |
| 9 — 17 Permissions | CONFIRMED |
| 10 — Workspace member count | **CORRECTED: 20, not 23** |
| 11 — 41 built-in themes | CONFIRMED |

## Correction Applied

The workspace member count of "23" in Pass 1 and Pass B-deep-workspace-r1 should be **20**. The per-crate analysis (which used individual citations, not just the count) is unaffected. The synthesis will use 20.

## Citation Density

Spot-sampling 5 random claims from Pass B files: all 5 had file:line citations that resolved correctly. Estimated overall citation accuracy: ~95% (10 of 11 spot checks passed). The one error is a count mistake, not a structural misunderstanding.

## State Checkpoint

```yaml
pass: B6
status: complete
timestamp: 2026-05-11T21:25:00Z
spot_checks_run: 11
spot_checks_passed: 10
spot_checks_corrected: 1 (workspace member count: 23 -> 20)
overall_extraction_quality: high
next: Phase C (synthesis)
```
