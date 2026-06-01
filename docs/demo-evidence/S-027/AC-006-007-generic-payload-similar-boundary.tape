# AC-006 + AC-007 — Generic tool payload display + similar purity boundary
# BC-2.06.024 PC-3 (Generic: "tool: " + "input: " labels, 256-char truncation)
# BC-2.06.015 INV-2 (similar ONLY in monocle-tui; absent from monocle-core)
#
# Evidence via TestBackend render tests asserting:
#   AC-006: "tool: " label + tool_name value in buffer
#   AC-006: "input: " label + JSON excerpt (compact) in buffer
#   AC-006: tool_input excerpt truncated at 256 chars (300-char value does not appear verbatim)
#   AC-006: scroll hint ("scroll" / "↑" / "↓") when JSON exceeds area height
#   AC-007: "similar" absent from monocle-core/Cargo.toml (checked by test scanning file)
#   AC-007: "use similar" / "extern crate similar" absent from monocle-core/src/**

Output docs/demo-evidence/S-027/AC-006-007-generic-payload-similar-boundary.gif
Output docs/demo-evidence/S-027/AC-006-007-generic-payload-similar-boundary.webm

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

# AC-006: tool/input labels, 256-char truncation, scroll hint on overflow
Type "cargo test -p monocle-tui --test overlay_render -- test_BC_2_06_024_render_generic_payload_shows_tool_and_input_labels test_BC_2_06_024_render_generic_payload_truncates_input_excerpt_at_256_chars test_BC_2_06_024_render_generic_payload_shows_scroll_hint_when_content_exceeds_height 2>&1 | tail -12"
Enter
Sleep 10s

# AC-007: similar absent from monocle-core Cargo.toml and source files
Type "cargo test -p monocle-tui --test diff_preview -- test_BC_2_06_015_invariant_2_similar_crate_not_in_monocle_core_cargo_toml test_BC_2_06_015_invariant_2_similar_not_used_in_monocle_core_source 2>&1 | tail -10"
Enter
Sleep 10s
Sleep 2s
