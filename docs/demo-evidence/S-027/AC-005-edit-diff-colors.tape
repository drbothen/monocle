# AC-005 — Edit payload: unified diff with color-coded lines via similar
# BC-2.06.010 PC-1 (Edit diff: added lines green, removed lines red, context default)
#
# Evidence via TestBackend render tests in diff_preview.rs asserting:
#   - Insert lines: Color::Green on cells
#   - Delete lines: Color::Red on cells
#   - Equal/context lines: neither Red nor Green
#   - Edit:<path> title appears in diff Block (in overlay body, not header)
#   - Empty old_content → all insertions green (no panic)
#   - Empty new_content → all deletions red (no panic)
#   - Height cap (area_height - 8) truncates without panic
#   - Wrap { trim: false } preserves leading whitespace
#   - Prefix char and content share the same style (single-token rendering)

Output docs/demo-evidence/S-027/AC-005-edit-diff-colors.gif
Output docs/demo-evidence/S-027/AC-005-edit-diff-colors.webm

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

# Success path: insert=green, delete=red, equal=default, Edit path in title
Type "cargo test -p monocle-tui --test diff_preview -- test_BC_2_06_010_diff_preview_insert_lines_rendered_green test_BC_2_06_010_diff_preview_delete_lines_rendered_red test_BC_2_06_010_diff_preview_equal_lines_rendered_default_color test_BC_2_06_010_diff_preview_edit_path_shown_in_diff_block_title test_BC_2_06_010_diff_prefix_and_content_share_same_style_single_token 2>&1 | tail -12"
Enter
Sleep 12s

# Edge paths: empty inputs, height cap, wrap preservation
Type "cargo test -p monocle-tui --test diff_preview -- test_BC_2_06_010_diff_preview_empty_old_content_all_insert_lines_green test_BC_2_06_010_diff_preview_empty_new_content_all_delete_lines_red test_BC_2_06_010_diff_preview_height_cap_truncates_without_panic test_BC_2_06_010_diff_preview_wrap_trim_false_preserves_leading_spaces test_BC_2_06_010_equal_line_content_cells_not_red_or_green test_BC_2_06_010_diff_height_cap_equals_area_height_minus_8 2>&1 | tail -12"
Enter
Sleep 12s
Sleep 2s
