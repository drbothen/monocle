# AC-008 + AC-009 — Auto-scroll (pinned_top flag) + session-change ribbon reset
# BC-2.06.018 PC-8 / AC-008 (auto-scroll to row 0 when !pinned_top; suppress when pinned_top=true)
# BC-2.06.018 INV-1 / AC-009 (session change resets ribbon to row 0, clears pinned_top, client-side only)
#
# Evidence via event_ribbon.rs and event_ribbon_real_defects.rs TestBackend tests:
#   - test_BC_2_06_018_auto_scroll_follows_bottom_when_not_pinned (AC-008, !pinned → auto-scroll fires)
#   - test_BC_2_06_018_auto_scroll_suppressed_when_pinned_top (AC-008, pinned_top=true → no scroll)
#   - test_BC_2_06_018_session_change_resets_scroll (AC-009, session change → row 0, pinned_top=false)
#   - test_BC_2_06_018_session_change_no_ipc_request (AC-009, reset_on_session_change is pure, no IPC)
#   - test_BC_2_06_018_INV1_AC009_session_change_resets_ribbon_scroll_via_dispatch (AC-009 via dispatch)
#
# Success path: new event when !pinned_top moves selection to row 0; session change resets to row 0.
# Error path: pinned_top=true suppresses auto-scroll; session change does NOT issue IPC (struct guarantee).

Output docs/demo-evidence/S-028/AC-008-009-auto-scroll-session-change.gif
Output docs/demo-evidence/S-028/AC-008-009-auto-scroll-session-change.webm

Require cargo

Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 16
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 30ms

Hide
Type "clear"
Enter
Sleep 1s
Show

# AC-008: auto-scroll fires when !pinned_top; suppressed when pinned_top=true
Type "cargo test -p monocle-tui --test event_ribbon -- test_BC_2_06_018_auto_scroll_follows_bottom_when_not_pinned test_BC_2_06_018_auto_scroll_suppressed_when_pinned_top 2>&1 | tail -10"
Enter
Sleep 15s

# AC-009: session-change resets to row 0, clears pinned_top, no IPC request issued
Type "cargo test -p monocle-tui --test event_ribbon -- test_BC_2_06_018_session_change_resets_scroll test_BC_2_06_018_session_change_no_ipc_request 2>&1 | tail -10"
Enter
Sleep 15s

# AC-009 via production dispatch_key_event path (session SelectNext → reset)
Type "cargo test -p monocle-tui --test event_ribbon_real_defects -- test_BC_2_06_018_INV1_AC009_session_change_resets_ribbon_scroll_via_dispatch 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
