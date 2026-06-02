# AC-005 + AC-006 — Shared nucleo Matcher (INV-1) + EventRibbon panel rendering
# BC-2.06.006 INV-1 (App.matcher shared; NOT recreated per keystroke; scoring functional across calls)
# BC-2.06.018 PC-1 (EventRibbon panel: timestamp | hook_type | session | latency columns)
# BC-2.05.002 PC-2 (ring_tail pre-populates event_ribbon_events on connect)
# BC-2.05.004 streaming (HookEventReceived appends to event_ribbon_events; client-side filter)
#
# Evidence via filter_sessions.rs and event_ribbon.rs TestBackend tests:
#   - test_BC_2_06_006_invariant_matcher_not_recreated_per_keystroke (AC-005, INV-1 shared matcher)
#   - test_BC_2_06_006_INV1_render_uses_shared_matcher_not_fresh_per_render (structural audit)
#   - test_BC_2_05_002_ring_tail_prepopulates_event_ribbon (AC-006, ring_tail backfill path)
#   - test_BC_2_05_004_hook_event_received_appends_to_ribbon (AC-006, streaming append path)
#   - test_BC_2_06_018_client_side_session_filter (AC-006, INV-1: all sessions stored, display-filtered)
#
# Success path: shared matcher produces consistent scores across consecutive calls; ring_tail events appear.
# Error path (captured via structural test): source audit confirms no local_matcher = Matcher::new per render.

Output docs/demo-evidence/S-028/AC-005-006-shared-matcher-event-ribbon-panel.gif
Output docs/demo-evidence/S-028/AC-005-006-shared-matcher-event-ribbon-panel.webm

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

# AC-005: shared Matcher INV-1 — consistent scoring across consecutive calls
Type "cargo test -p monocle-tui --test filter_sessions -- test_BC_2_06_006_invariant_matcher_not_recreated_per_keystroke 2>&1 | tail -8"
Enter
Sleep 15s

# AC-005 structural audit: source does NOT contain local_matcher = Matcher::new
Type "cargo test -p monocle-tui --test event_ribbon_real_defects -- test_BC_2_06_006_INV1_render_uses_shared_matcher_not_fresh_per_render 2>&1 | tail -8"
Enter
Sleep 15s

# AC-006: ring_tail pre-population (BC-2.05.002 PC-2) and HookEventReceived streaming (BC-2.05.004)
Type "cargo test -p monocle-tui --test event_ribbon -- test_BC_2_05_002_ring_tail_prepopulates_event_ribbon test_BC_2_05_004_hook_event_received_appends_to_ribbon test_BC_2_06_018_client_side_session_filter 2>&1 | tail -10"
Enter
Sleep 15s
Sleep 2s
