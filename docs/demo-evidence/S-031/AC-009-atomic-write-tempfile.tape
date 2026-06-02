Output AC-009-atomic-write-tempfile.gif
Output AC-009-atomic-write-tempfile.webm

Require cargo

Set FontFamily "FiraCode Nerd Font Mono"
Set Theme "Dracula"
Set Width 1200
Set Height 600
Set FontSize 14
Set Padding 20

# AC-009 (BC-2.07.005 INV-2):
# write_config() MUST use tempfile::persist (from monocle-config::write_config()).
# Direct std::fs::write for config is a forbidden pattern.
# monocle-no-direct-config-write semgrep rule enforces this.
#
# SUCCESS PATH: round-trip write+read via write_config succeeds
# SUCCESS PATH: no direct fs::write in profile_picker_widget.rs or app.rs

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-031"
Enter
Sleep 1s
Show

Type "# AC-009: atomic write_config round-trip (tempfile::persist)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-tui --test profile_switch BC_2_07_005_invariant_atomic_write_via_write_config -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Type "# AC-009: verify no direct std::fs::write in TUI profile code"
Enter
Sleep 0.3s
Type "grep -rn 'std::fs::write' crates/monocle-tui/src/ui/profile_picker_widget.rs crates/monocle-tui/src/app.rs 2>/dev/null && echo 'VIOLATION FOUND' || echo 'OK: no direct fs::write found'"
Enter
Sleep 3s

Type "# AC-009: all monocle-config tests pass (write_config foundation)"
Enter
Sleep 0.3s
Type "cargo test -p monocle-config -- --nocapture 2>&1 | tail -4"
Enter
Sleep 6s

Sleep 2s
