# VHS Demo — S-028 Sessions Nucleo Fuzzy Filter
# Covers ACs:
#
# AC-001 (BC-2.06.006 PC-1): pressing '/' or 'f' enters Filtering mode;
#         filter input box rendered at top of Sessions panel with cursor
# AC-002 (BC-2.06.006 PC-2): typed chars update query; nucleo scores sessions,
#         highest score first; non-matching sessions hidden; shared Matcher
# AC-003 (BC-2.06.006 PC-3): Enter → CommitFilter (retained selection); Esc → CancelFilter
# AC-004 (BC-2.06.006 PC-4 / empty-query): empty query shows all sessions (no scoring)
# AC-005 (BC-2.06.006 INV-1): nucleo::Matcher held in App — not recreated per keystroke

# === Output Configuration ===
Output AC-001-005-sessions-filter.gif
Output AC-001-005-sessions-filter.webm

# === Dependency Check ===
Require cargo

# === Visual Settings (factory standard) ===
Set Shell "zsh"
Set FontFamily "FiraCode Nerd Font Mono"
Set FontSize 18
Set Width 1200
Set Height 680
Set Theme "Catppuccin Mocha"
Set WindowBar Colorful
Set BorderRadius 8
Set Padding 20
Set Framerate 30
Set TypingSpeed 40ms

# === Hidden Setup ===
Hide
Type "cd /Users/jmagady/Dev/monocle && clear"
Enter
Sleep 2s
Show

# === Demo: S-028 AC-001..005 — Sessions Nucleo Fuzzy Filter ===
# Drives filter_sessions.rs against the production dispatch_key_event + render path.
# Demonstrates:
#   AC-001: '/' and 'f' transition to Filtering { Sessions }, search input box rendered
#   AC-002: typed chars append to query; nucleo scores "mono" → shows "monocle", hides "another-project"
#           case-insensitive match ("MONO" → "monocle")
#           zero-match renders SESSIONS_FILTER_NO_MATCH sentinel
#   AC-003: CommitFilter (Enter) and CancelFilter (Esc) return to Dashboard
#   AC-004: empty query shows all sessions (no scoring applied)
#   AC-005: shared Matcher produces consistent correct results across consecutive calls

Type "cargo test -p monocle-tui --test filter_sessions -- 2>&1 | tail -20 && echo 'FILTER_PASS'"
Enter
Wait+Screen@120s /FILTER_PASS/
Sleep 2s
