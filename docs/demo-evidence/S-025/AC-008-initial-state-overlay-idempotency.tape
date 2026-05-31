Output AC-008-initial-state-overlay-idempotency.gif
Output AC-008-initial-state-overlay-idempotency.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-008: InitialState handler populates VecDeque<PromptModal> via
# apply_permission_prompt_queued (BC-2.05.002 Invariant 4 idempotency on prompt_id).
# Non-empty overlay transitions to Overlay mode. Duplicate prompt_id is silently discarded.

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-025"
Enter
Sleep 1s
Show

Type "# AC-008: InitialState overlay population + idempotency (BC-2.05.002 Inv4)"
Enter
Sleep 0.5s
Type "cargo test -p monocle-tui --test startup_connect ac008 -- --nocapture"
Enter
Sleep 4s

Type "cargo test -p monocle-tui --test startup_connect inv4 -- --nocapture"
Enter
Sleep 4s
