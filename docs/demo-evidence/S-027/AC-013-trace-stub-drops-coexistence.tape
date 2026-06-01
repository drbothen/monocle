# AC-013 — [t] trace-to-source stub + drops:N coexistence
# BC-2.06.015 PC-1 (exact canonical status_message after [t] press)
# BC-2.06.015 PC-2 (AppMode stays Overlay; overlay_stack unchanged)
# BC-2.06.015 PC-3 (no IPC message sent on [t] press)
# BC-2.06.015 PC-7 / INV-1 (Action::PermissionTraceToSource in state.rs + binding.rs)
# BC-2.06.015 EC-099 ([t] in Dashboard is unbound; status_message unchanged)
# BC-2.06.015 EC-098 (repeated [t] presses idempotent)
# BC-2.06.019 PC-7 / EC-129 (drops:N upper row + disconnect msg lower row, simultaneously)
# BC-2.06.019 PC-7 / EC-130 (drops:N upper row + [t] placeholder lower row, simultaneously)
#
# This is the critical coexistence test: mutual-exclusion "if msg { msg } else { drops }"
# is FORBIDDEN. Both signals are always visible on their respective rows.

Output docs/demo-evidence/S-027/AC-013-trace-stub-drops-coexistence.gif
Output docs/demo-evidence/S-027/AC-013-trace-stub-drops-coexistence.webm

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

# AC-013 success path: exact status_message, mode unchanged, no IPC, production binding
Type "cargo test -p monocle-tui --test overlay_stub -- test_BC_2_06_015_handler_sets_status_message_exact_canonical_text test_BC_2_06_015_handler_sets_status_message_mode_stays_overlay test_BC_2_06_015_handler_no_ipc_sent_on_trace_key test_BC_2_06_015_production_binding_t_in_overlay_resolves_trace_to_source 2>&1 | tail -12"
Enter
Sleep 10s

# AC-013 edge paths: EC-099 (t in Dashboard is no-op), EC-098 (idempotent)
Type "cargo test -p monocle-tui --test overlay_stub -- test_BC_2_06_015_ec099_t_in_dashboard_status_message_unchanged test_BC_2_06_015_ec098_repeated_t_press_idempotent_status_message 2>&1 | tail -10"
Enter
Sleep 10s

# AC-013 coexistence: [t] placeholder on lower row + drops:N on upper row simultaneously
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_019_pc7_coexistence_drops_and_trace_stub_both_visible_in_two_row_bar test_BC_2_06_019_pc7_coexistence_drops_and_disconnect_both_visible_in_two_row_bar 2>&1 | tail -10"
Enter
Sleep 10s
Sleep 2s
