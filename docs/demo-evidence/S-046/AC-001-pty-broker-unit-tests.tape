# S-046 AC-001 — PtyOutput Fan-out Broker Unit Tests
#               Bounded INPUT channel(1024) backpressure, fan-out via shared SubscriberList,
#               1-strike disconnect, pty_drop_counter NOT incremented on graceful close or
#               broadcast failure, PtyReset NOT emitted on graceful input close,
#               biased-select source-guard, no-unbounded-channel source-guard.
# Covers: BC-2.05.009 all 8 acceptance criteria (AC-001..AC-007 + AC-004 dual test)
# Test file: crates/monocle-runtime/tests/pty_broker.rs
# 8 tests — all passing

Output AC-001-pty-broker-unit-tests.webm

Require cargo

Set Shell "bash"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 13
Set Width 1280
Set Height 720
Set Theme "Dracula"
Set Padding 16
Set PlaybackSpeed 1.0

Hide
Type "cd /Users/jmagady/Dev/monocle/.worktrees/S-046"
Enter
Sleep 2s
Show

Type "# S-046: PTY fan-out broker unit tests (BC-2.05.009 — all 8 ACs)"
Enter
Sleep 500ms

Type "cargo test -p monocle-runtime --test pty_broker 2>&1"
Enter
Sleep 30s
