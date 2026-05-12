# Pass B Deep: `internal/adapter/tmuxadapter` — Round 1

**Scope:** `internal/adapter/tmuxadapter/{detect.go, sendkeys.go, detect_test.go, sendkeys_test.go}` (126 source LOC + 294 test LOC).

**Mission:** Close the B5-audit "LOW-MEDIUM gap" by writing first-class BC-TMUXADAPTER-* contracts at file:line precision. The adapter is small but load-bearing — every Claude permission-dialog response goes through it.

**Prior context consumed:**
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-3-behavioral-contracts.md` — no BC-TMUXADAPTER-* exists; only BC-TMUX-CTL-* (control mode) and BC-TMUX-EXEC-* (exec adapter — see deep-tmux-r1).
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B-deep-tmux-r1.md` — covered the lower layer (`internal/core/tmux/{client,exec,control,mock}.go`).
- `/Users/jmagady/Dev/monocle/.factory/semport/any-context-lazyclaude/any-context-lazyclaude-pass-B5-coverage-audit-v2.md:34,130,157,180,510,523` — explicit gap notice.

---

## 0. Framing-correction up front (brief had a misnomer)

The orchestrator brief says:

> DetectMaxOption is used for tmux 3.4+ feature detection (display-popup -b rounded requires it).

**This is incorrect** and would lead a porter astray. `DetectMaxOption` has **nothing to do with tmux version detection or `display-popup -b rounded`**. It parses *Claude Code's permission-dialog options* out of a captured pane buffer — it counts how many numbered options (`1.`, `2.`, `3.`) are presented to the user, so the controller can clamp a chosen index to the valid range.

Verified by reading both files: no `version`, `3.4`, `popup`, or `rounded` tokens appear in `detect.go` or `sendkeys.go` (verified via `awk '/version|3\.4|popup|rounded/i'` — zero hits).

Tmux-version-dependent code (display-popup `-b rounded`, etc.) lives elsewhere (not in this adapter). This deep dive corrects that framing and proceeds on the actual semantics.

---

## 1. File inventory and structural shape

| File | Lines | Public symbols | Purpose |
|------|-------|----------------|---------|
| `detect.go` | 69 | `DetectMaxOption(paneContent string) int` | Parse last consecutive numbered-options block from a pane snapshot |
| `sendkeys.go` | 58 | `SendToPane(ctx, client, window, choice) error` | Translate a `choice.Choice` to a clamped digit keypress on a tmux pane |
| `detect_test.go` | 175 | 15 table-driven cases + 2 benchmarks | Unit + bench coverage of `DetectMaxOption` |
| `sendkeys_test.go` | 121 | 7 cases | Unit coverage of `SendToPane` (uses `tmux.MockClient`) |

Package surface: **2 exported functions, 2 unexported package-level vars (`optionPattern`, `ansiEscape`), 1 unexported const (`defaultMaxOption = 3`), 1 unexported const (`sessionPrefix = "lazyclaude"`), 1 unexported map (`choiceToKey`)**.

Consumers (full repository search via `awk` — no `grep`/`rg` available):
- `internal/server/server.go:497` — `tmuxadapter.DetectMaxOption(content)` inside `dispatchToolNotification`, supplying `MaxOption` to `model.ToolNotification`.
- `cmd/lazyclaude/local_provider.go:150` — `tmuxadapter.SendToPane(ctx, p.tmux, window, choice.Choice(choiceVal))` is the implementation of `localDaemonProvider.SendChoice`.

So the adapter has **exactly two consumers**: the in-process MCP server (read side) and the local daemon provider (write side). That is the entire blast radius.

---

## 2. `DetectMaxOption` — algorithm spec (ready for Rust port)

### 2.1 Inputs & outputs

```go
func DetectMaxOption(paneContent string) int
```

- Input: arbitrary string (intended to be the output of `tmux capture-pane -ep`, i.e. ANSI-preserving capture; supplied at `server.go:496`).
- Output: integer `≥ 1`; defaults to `3` when no valid options block is found (`detect.go:17, 64-66`).

### 2.2 Regex contract

Two compiled regexes (package-level, compiled at init):

```go
optionPattern = regexp.MustCompile(`^\s*(?:[>❯➜]\s+)?(\d+)[.)]\s+(.+)`)   // detect.go:12
ansiEscape    = regexp.MustCompile(`\x1b\[[0-9;]*[a-zA-Z]`)              // detect.go:15
```

**`optionPattern` grammar:**
- `^\s*` — leading whitespace allowed (any amount).
- `(?:[>❯➜]\s+)?` — optional cursor marker (3 alternatives: ASCII `>`, U+276F `❯`, U+279C `➜`) followed by **one or more whitespace chars**. **No alternative marker chars exist** — porter must use exactly this trio.
- `(\d+)` — capture-group 1: the option number. Note: `\d` in Go's `regexp` package (RE2) means ASCII `[0-9]` only — full-width digits are not matched.
- `[.)]` — separator: literal dot or close-paren (no comma, no colon, no `:`, no `]`).
- `\s+` — at least one whitespace between separator and option label.
- `(.+)` — capture-group 2: option label (never used — captured but discarded; pure-source code would drop this group entirely).

**`ansiEscape` grammar:** matches CSI sequences `ESC [ ... letter` only. Does NOT strip:
- OSC sequences (`ESC ]`)
- DCS sequences (`ESC P`)
- C0/C1 control bytes except via being part of CSI
- Bracketed-paste markers (`ESC [ 200 ~` IS matched, OK because trailing `~` is `[a-zA-Z]`? **No** — `~` is not in `[a-zA-Z]`. Bracketed-paste markers would NOT be stripped.) ✱ See finding **F-TMUXADAPTER-006** below.

### 2.3 Three-state finite parser

For each line in `strings.Split(paneContent, "\n")`:

1. Strip ANSI via `ansiEscape.ReplaceAllString(line, "")`.
2. Apply `optionPattern.FindStringSubmatch(clean)`.
3. State machine (`current` = currently-building block, `lastBlock` = last completed block):

| Line type | `current` action | `lastBlock` action |
|-----------|------------------|---------------------|
| Non-matching | If `len(current) > 0`: `lastBlock = current; current = nil`. Else: no-op. | Updated when current is non-empty |
| Match, `n == 1` | `current = []int{1}` (resets) | Untouched |
| Match, `n == len(current)+1` and `len(current) > 0` | `current = append(current, n)` | Untouched |
| Match, any other `n` | No-op (silently ignored) | Untouched |

Post-loop (`detect.go:60-62`): if `current` is non-empty, `lastBlock = current` (final flush).

Result: `len(lastBlock)` if `> 0`, else `defaultMaxOption = 3`.

### 2.4 Subtle invariants the test suite implicitly pins

From `detect_test.go`:

| Test (line) | Invariant | Witness |
|-------------|-----------|---------|
| `non-sequential numbers ignored` (132-135) | A line `"  3. No"` immediately after `"  1. Yes"` is rejected — block stays `[1]`, result `1`. | The branch `n == len(current)+1` fails (3 != 2) and there is no `else` capture. |
| `stale output then current dialog` (118-129) | **Last block wins**, not first. Old block of size 3 is overwritten by a fresh block of size 2. | Result `2`, not `3`. |
| `cursor arrow on option` (90-95) — only line 1 has `>` | Marker is optional; subsequent lines without it still match. | All 3 lines matched, result `3`. |
| `ANSI escape codes in options` (137-140) | ANSI in the marker / label is stripped before regex. | `"\x1b[1m❯\x1b[0m 1. Yes"` parses as if it were `"❯ 1. Yes"`. |
| `mixed content with numbers in text` (97-103) | Numbers in narrative prose ("File has 42 lines", "number 99") do **not** false-match because they fail the post-number `[.)]\s+(.+)` shape. | Result `2` (the real dialog), not `1`. |
| `empty string` (72-75) | Empty input returns the default `3`, NOT `0`. | `defaultMaxOption` is the floor. |
| `4-option dialog` (158-164) | No upper cap. The function returns whatever count was found, even if the consumer only knows about 3 choices. | Result `4`. Clamping is the **consumer's** responsibility (cf. SendToPane). |

### 2.5 Untested behaviors (gaps for porter awareness)

| Scenario | Expected behavior | Tested? |
|----------|-------------------|---------|
| Block ending exactly at EOF without trailing `\n` | Captured by post-loop flush (line 60-62). | **Not directly** (most tests end with `\n` or rely on the final line being non-matching). The flush exists but is implicit. |
| Window-resize-induced wrapping (option label wraps to next line) | Wrapped label is treated as a non-matching line → ends the current block early. | **Not tested**. The actual Claude Code dialog can wrap on narrow panes. |
| Multiple cursor markers concatenated (`❯❯ 1.`) | Would fail the `(?:[>❯➜]\s+)?` alternation (a single marker, not two). | Not tested. |
| CRLF line endings | `strings.Split(s, "\n")` would leave a trailing `\r` in each line; regex `(.+)$` would match through it, so likely benign. | Not tested. |
| Lines with `>` indent followed by no marker space (`>1. Yes`) | The marker group requires `\s+` after; falls through to the optional-marker branch, so `"1. Yes"` still matches because the leading `>` would not be captured. Actually wait — `^\s*(?:[>❯➜]\s+)?(\d+)` — if the input starts with `>1` (no space after `>`), the optional group requires `[>❯➜]\s+` which fails; then the regex tries to match `^\s*(\d+)`, but the literal `>` is not whitespace and is not skipped → **the whole pattern fails**. So `>1. Yes` would NOT match. | Not tested. |
| `0)`-leading line (option index 0) | `n == 1` fails, `n == len(current)+1` fails when `current` empty, so silently ignored. Block never starts. | Not tested. |
| Multibyte/CJK in label | Match succeeds (`.+` is per-byte for ASCII regex; RE2 in Go is Unicode-aware for `.` — actually `.` matches any rune by default in Go's RE2). | Not tested. |
| Tab characters between number and label (`1.\tYes`) | `\s+` matches tab, so OK. | Not tested. |

### 2.6 Complexity / performance

- **Per-call:** O(N) over input length where N = bytes (one pass over lines + per-line ANSI regex + option regex).
- **Allocations:** `strings.Split` allocates a `[]string`; each match potentially allocates submatch slices; `ansiEscape.ReplaceAllString` allocates a new string per line (always — even when no match).
- **Hot-path test:** `BenchmarkDetectMaxOption_LargePane` (detect_test.go:31-41) uses a 190-line prefix + the standard 12-line dialog, deliberately targeting the no-match path on the prefix. No microsecond budget pinned in tests.
- **Consumer cadence:** Called once per Claude permission notification, which is human-interactive (sub-Hz). Not in any inner loop.

---

## 3. `SendToPane` — semantics spec

### 3.1 Signature & control flow

```go
func SendToPane(ctx context.Context, client tmux.Client, window string, c choice.Choice) error
```

`sendkeys.go:29-58`. Steps:

1. **Cancel short-circuit (`:30-32`):** if `c == choice.Cancel` (value `0`), return `nil` immediately. No I/O whatsoever — including no capture-pane call.
2. **Choice → key map (`:34-37`):** lookup `choiceToKey[c]`. If not present, return `nil` (silent). The map at `:16-20` covers only `Accept=1` → `"1"`, `Allow=2` → `"2"`, `Reject=3` → `"3"`.
3. **Target normalization (`:39-42`):** if `window` does not contain `:`, prepend `"lazyclaude:"`. Otherwise pass through unchanged.
4. **Capture pane (`:45`):** `client.CapturePaneANSI(ctx, target)`. On error, **send the key anyway** (best-effort path, `:46-49`) with `key` taken from the original choice — no clamping is applied. Returns whatever `SendKeys` returns.
5. **Clamp (`:51-55`):** `maxOpt := DetectMaxOption(paneContent); if int(c) > maxOpt { key = fmt.Sprintf("%d", maxOpt) }`. Note: clamping uses `int(c)` (the *raw choice value* 1/2/3), not the looked-up key.
6. **Send (`:57`):** `client.SendKeys(ctx, target, key)`.

### 3.2 Key alphabet — much narrower than the brief assumed

The orchestrator brief asked about handling of `Up`, `Down`, `Enter`, `Tab`, `Escape`, `BSpace`, `Space`, `C-c`, `M-x`, Unicode, and bracketed-paste in `SendKeys`.

**Reality (`sendkeys.go:16-20`):** the adapter's entire output alphabet for `SendKeys` is the three single-character ASCII strings `"1"`, `"2"`, `"3"`. There is **no** named-key support, **no** modifier-combo support, **no** Unicode in the keystroke payload, **no** bracketed-paste pathway. The adapter does not call `SendKeysLiteral`, `PasteToPane`, or any other key-injection variant.

Therefore the entire orchestrator question about quoting/escaping rules for `Up`, `Down`, `Enter`, `Tab`, `Escape`, `BSpace`, `Space`, `C-c`, `M-x`, Unicode, and bracketed-paste **collapses to a single answer for this adapter**: those concerns live one layer down, in `internal/core/tmux/{exec,control}.go`, and are already documented as BC-TMUX-CTL-001/002/003 (Pass 3) and BC-TMUX-EXEC-010 (deep-tmux-r1). The adapter inherits whatever validation/escaping the chosen `Client` impl applies — which is:

- `ExecClient.SendKeys` (exec.go:323-327): builds argv `["send-keys", "-t", target, "1"]` (or "2" / "3"). No string mutation. The digit travels through `exec.Command` as a literal arg → tmux receives it as the key name (which, for `1`, equals "the literal `1` keystroke").
- `ControlClient.SendKeys` (control.go:131-141 — see deep-tmux-r1 BC-TMUX-CTL-001/011): validates the digit-string for spaces/`;`/newlines (it passes trivially), then writes `send-keys -t <target> 1\n` over the control connection.
- `MockClient.SendKeys` (mock.go:181-187): appends to `m.SentKeys[target]`. No validation.

For the Rust port: `SendToPane`'s key alphabet is closed under `{"1","2","3"}` — this is a *small finite enumeration*, not a string-encoding problem. The Rust impl should validate at the type level (`enum DialogKey { One, Two, Three }`) rather than pass-through strings.

### 3.3 Window-target syntax accepted

From `sendkeys.go:39-42` + sendkeys_test.go:

| Input shape | Result | Test witness |
|-------------|--------|--------------|
| `"@3"` (bare window ID, no colon) | Becomes `"lazyclaude:@3"` | `TestSendToPane_Accept` (sendkeys_test.go:14-25), `TestSendToPane_PrependsSessionName` (61-71) |
| `"mysession:@2"` (already has session) | Passed through unchanged | `TestSendToPane_AlreadyHasSession` (111-120) |
| `"3"` (numeric index, no `@`, no colon) | Becomes `"lazyclaude:3"` — tmux would resolve as window-index 3 in session `lazyclaude` | Not tested |
| `"name:window"` (named session and window) | Passed through | Not tested |
| `""` (empty) | Becomes `"lazyclaude:"` — tmux would send to the session's active pane | Not tested |
| Any string containing `:` | Passed through (no further validation) | The colon-detection rule (`strings.Contains(window, ":")`) is the entire heuristic |

**Sharp edge for porter:** the colon-presence check is the entire session-vs-bare detection rule. Strings like `"::"`, `"foo:bar:baz"`, or even literal `":"` would all be treated as "already qualified" and passed through. The exec/control layer below validates `:` is allowed in target strings (only `\n`, `\r`, `;`, ` ` are rejected by `validateControlTarget` — control.go:190-198), so injection through this codepath is constrained by that validator, not by this adapter.

### 3.4 Failure modes & error propagation

| Failure | Behavior | Notes |
|---------|----------|-------|
| `c == Cancel` | Return `nil` | Documented: "safe no-op" (sendkeys.go:23, 32). |
| `c` not in `choiceToKey` (e.g. `Choice(99)`) | Return `nil` | Silent. The map covers 1,2,3 only. This is brittle — a future Choice value would be silently dropped. **Finding F-TMUXADAPTER-002**. |
| `CapturePaneANSI` returns error | **Best-effort send with unclamped key** (sendkeys.go:46-49) | The send is attempted with the original key (`"1"`, `"2"`, or `"3"`), bypassing clamp. If the dialog has only 2 options and choice was Reject(3), the user's intent is **lost** in this path: tmux receives `"3"` which is a no-op key on a 2-option dialog. **Finding F-TMUXADAPTER-003**. |
| `SendKeys` returns error | Returned to caller | Standard error propagation. |
| `ctx` already cancelled | Capture fails → fall through to best-effort send, which will also fail context → error returned | The function does not check `ctx.Err()` explicitly. |
| Clamping triggers (choice > maxOpt) | Key replaced with `maxOpt` digit | E.g. Reject(3) on a 2-option dialog → sends `"2"` (which is Reject's semantic position in a 2-option dialog where 1=Yes, 2=No). **This is the core load-bearing behavior** — without it, the user's "Reject" intent would be sent as a non-existent option. |

### 3.5 Race conditions

- **Stale capture race:** `CapturePaneANSI` runs at time T0; `SendKeys` runs at T0+ε. If the dialog updates (Claude redraws) between T0 and T0+ε with a different option count, the clamp may be wrong. There is no `tmux wait-for` or generation token. In practice the dialogs are stable during human reaction time, so this is theoretical.
- **Concurrent SendToPane on same window:** safe with respect to the adapter itself (no shared state), but two concurrent calls would race on the underlying tmux pane — tmux serializes its own input queue. The map `choiceToKey` is read-only after init.
- **Session-name collision:** the hardcoded `"lazyclaude"` prefix (`:12`) means if a user has a non-lazyclaude tmux session that happens to be named `lazyclaude`, the adapter would try to send to that. The launching pathway (BC-LIFE-* from Pass 3) ensures the lazyclaude tmux server uses a dedicated socket (`-L lazyclaude`), so the collision is constrained to the lazyclaude-socket scope only. The `tmux.Client` impl injected here is configured for that socket.

---

## 4. Interaction with `internal/core/tmux/control.go:176-179` (the Unicode TODO)

The brief asked: does `tmuxadapter` inherit or fix this TODO?

**Answer: NEITHER — the TODO is unreachable from the adapter.**

Verified by reading both files:

1. `tmuxadapter` calls only `client.CapturePaneANSI` (read) and `client.SendKeys` (single-digit write). It never calls `SendKeysLiteral`, `PasteToPane`, or any literal-text-injection path.
2. The TODO at `control.go:176-179` is inside `ControlClient.SendKeysLiteral`, which handles arbitrary user-typed text (used by GUI paste / typing pathways — see GUI and PMW deepenings).
3. Since the adapter's payload is restricted to `{"1","2","3"}`:
   - These are all ASCII, single-byte, with no combining characters.
   - They contain no `\`, `"`, `;`, space, newline, or any character that the SendKeysLiteral escape logic touches.
   - They take the `SendKeys` (non-literal) path which uses the digit as a tmux *key name*, not as literal text. `tmux send-keys -t <target> 1` types the `1` keystroke; it has no quoting issue.

**Conclusion:** The Unicode/combining-character TODO at `control.go:176-179` has **zero impact** on `tmuxadapter`. A Rust port of `tmuxadapter` does not need to solve, mitigate, or even acknowledge that TODO. (It does, however, need to acknowledge it when porting `core/tmux/control.go` — see deep-tmux-r1.)

---

## 5. Bracketed paste (the synthesis-flagged limitation)

The brief asked specifically about bracketed paste. **The adapter does not use bracketed paste at any layer**:

- `SendToPane` calls `client.SendKeys`, not `client.PasteToPane`.
- `PasteToPane` (`exec.go:344-357`) is the only path that uses `paste-buffer -p` (the `-p` flag is what enables bracketed paste markers around the buffer text). It is invoked for clipboard-paste from the GUI (see PMW deepening), not from this adapter.
- The pane content captured by `CapturePaneANSI` is read-only — bracketed paste is irrelevant for capture.

So a Rust monocle implementation of `SendToPane` has no bracketed-paste compatibility concern with `lazyclaude`. (Again, the GUI paste path does — but that's a different subsystem.)

---

## 6. Behavioral contracts (BC-TMUXADAPTER-*)

### BC-TMUXADAPTER-001: `DetectMaxOption` parses the LAST consecutive numbered-options block in the pane snapshot
**Preconditions:** `paneContent` is any string (may be empty).
**Postconditions:** Returns `len(lastBlock)` where `lastBlock` is the most recent run of lines matching `optionPattern` with sequentially-increasing numbers starting from 1. Returns `defaultMaxOption = 3` when no such block exists.
**Evidence:** `detect.go:27-68`. Test: `stale output then current dialog (uses last block)` (detect_test.go:118-129) — pins the "last wins" behavior.
**Confidence:** HIGH (multiple test cases).

### BC-TMUXADAPTER-002: A non-matching line ends the in-progress block and assigns it to `lastBlock`
**Preconditions:** A line did not match `optionPattern` after ANSI stripping.
**Postconditions:** If `current` was non-empty: `lastBlock = current; current = nil`. Subsequent matching lines start a fresh block (requires `n == 1`).
**Evidence:** `detect.go:36-43`. Test: `mixed content with numbers in text` (detect_test.go:97-103).
**Confidence:** HIGH.

### BC-TMUXADAPTER-003: A line matching with `n == 1` always resets `current` (even mid-block)
**Preconditions:** Match found, `n == 1`.
**Postconditions:** `current = []int{1}`, discarding any in-progress block. The previously in-progress block is **not** flushed to `lastBlock` first.
**Evidence:** `detect.go:50-52`. **Untested directly**, but logically implied. Concrete edge case: input `"1.\n2.\n1.\n2.\n3.\n"` (two back-to-back blocks with no separator) would still yield `3` because each `n==1` resets and the final flush captures `[1,2,3]`. **Not in test suite.**
**Confidence:** MEDIUM (logic-derived, not test-witnessed).

### BC-TMUXADAPTER-004: A line matching with `n != 1 && n != len(current)+1` is silently ignored
**Preconditions:** Match found, n is neither 1 nor `len(current)+1`.
**Postconditions:** No state change. `current` and `lastBlock` unchanged.
**Evidence:** `detect.go:53-57` (the `if/else if` has no final `else` branch). Test: `non-sequential numbers ignored` (detect_test.go:131-135).
**Confidence:** HIGH.

### BC-TMUXADAPTER-005: ANSI CSI sequences (`ESC [ ... [a-zA-Z]`) are stripped per-line before regex match
**Preconditions:** Line contains CSI escape sequences.
**Postconditions:** CSI sequences are removed; the remaining text is matched. Non-CSI escape sequences (OSC, DCS, bracketed-paste markers ending in `~`) are **not** stripped — see F-TMUXADAPTER-006.
**Evidence:** `detect.go:15, 33`. Test: `ANSI escape codes in options` (detect_test.go:137-140).
**Confidence:** HIGH for CSI; MEDIUM for non-CSI (not tested either direction).

### BC-TMUXADAPTER-006: `defaultMaxOption = 3` is the floor when no block is found
**Preconditions:** No consecutive run starting from `1` is found in the input.
**Postconditions:** Returns `3`.
**Evidence:** `detect.go:17, 64-66`. Tests: `empty string` (detect_test.go:72-75), `no options found` (66-70).
**Confidence:** HIGH. Note for porter: the default is deliberately the most-permissive count — consumers can always clamp down with `min(c, maxOpt)`, but if the floor were 0 or 1, valid user choices would be erroneously suppressed.

### BC-TMUXADAPTER-007: The cursor marker alphabet is exactly `{>, ❯ (U+276F), ➜ (U+279C)}`
**Preconditions:** Line begins with optional whitespace, then cursor marker.
**Postconditions:** Only these three runes are recognized as cursor markers. Other arrows (`→`, `▶`, `▸`, `►`) would fail the optional-marker branch — but the line would still match if it's of the form `\s*<digit>[.)]`.
**Evidence:** `detect.go:12`. Tests: `unicode marker ❯` (105-110), `unicode marker ➜` (112-116), `cursor arrow on option` (90-95).
**Confidence:** HIGH.

### BC-TMUXADAPTER-008: Option-number separator is exactly `.` or `)` followed by `\s+`
**Preconditions:** A digit run is captured.
**Postconditions:** The next byte must be `.` or `)`, followed by ≥1 whitespace, followed by ≥1 character of label. `:`, `,`, `]`, or no separator all fail.
**Evidence:** `detect.go:12`. Tests: `options with dot separator` (77-81), `options with paren separator` (83-88).
**Confidence:** HIGH.

### BC-TMUXADAPTER-009: `DetectMaxOption` returns `len(lastBlock)` not `max(lastBlock)`
**Preconditions:** A block exists.
**Postconditions:** Return value is the count of consecutive options found, which equals the highest sequential number reached (since they're 1..N). **No upper cap** — a 99-option dialog would return 99.
**Evidence:** `detect.go:67`. Test: `4-option dialog` (158-164) returns `4`. **Upper-bound behavior not explicitly tested**.
**Confidence:** HIGH.

### BC-TMUXADAPTER-010: `SendToPane` returns `nil` immediately and performs no I/O on `Choice.Cancel`
**Preconditions:** `c == choice.Cancel` (value `0`).
**Postconditions:** Function returns `nil`. No call to `CapturePaneANSI`, no call to `SendKeys`. Pane is unaffected.
**Evidence:** `sendkeys.go:30-32`. Test: `TestSendToPane_Cancel_NoSend` (sendkeys_test.go:52-59) asserts `mock.SentKeys` is empty.
**Confidence:** HIGH.

### BC-TMUXADAPTER-011: `SendToPane` returns `nil` for any `Choice` value not in `{Accept, Allow, Reject}`
**Preconditions:** `c` is not in the keys of `choiceToKey` (and not `Cancel`, which is handled separately).
**Postconditions:** Returns `nil`. Silent no-op.
**Evidence:** `sendkeys.go:34-37`. **Not tested.** A future addition of e.g. `choice.Defer = 4` would be silently dropped here — porter should consider whether this is desired.
**Confidence:** MEDIUM (logic-derived).

### BC-TMUXADAPTER-012: `SendToPane` prepends `"lazyclaude:"` to bare window strings (no colon present)
**Preconditions:** `!strings.Contains(window, ":")`.
**Postconditions:** Target becomes `"lazyclaude:" + window`.
**Evidence:** `sendkeys.go:39-42`. Tests: `TestSendToPane_PrependsSessionName` (sendkeys_test.go:61-71), `TestSendToPane_AlreadyHasSession` (111-120).
**Confidence:** HIGH.

### BC-TMUXADAPTER-013: On `CapturePaneANSI` error, `SendToPane` falls through to a best-effort send with the unclamped key
**Preconditions:** `client.CapturePaneANSI(ctx, target)` returns a non-nil error.
**Postconditions:** `client.SendKeys(ctx, target, key)` is called with the original (unclamped) digit. The return value is whatever `SendKeys` returns. The capture error is **not** surfaced — but the original (potentially out-of-range) keystroke is sent. **Risk:** if the dialog has fewer options than `c`, the user's intent is lost (the dialog ignores out-of-range digits).
**Evidence:** `sendkeys.go:45-49`. **Not tested** (no error-injection test for `ErrCapture`). Mock supports `ErrCapture` (mock.go:32, 164-166) but no `tmuxadapter` test exercises this path.
**Confidence:** HIGH (code) / **GAP in test coverage**.

### BC-TMUXADAPTER-014: On successful capture, `SendToPane` clamps the key to `DetectMaxOption(paneContent)`
**Preconditions:** Capture succeeded, `c ∈ {Accept, Allow, Reject}`, `int(c) > maxOpt`.
**Postconditions:** `key` is replaced with `fmt.Sprintf("%d", maxOpt)`. **Clamping uses `int(c)` (raw enum value 1/2/3), not the map-looked-up key** — but for the current enum these coincide.
**Evidence:** `sendkeys.go:51-55`. Tests: `TestSendToPane_ClampTo2Options` (sendkeys_test.go:39-50), `TestSendToPane_RejectOn2OptionBashDialog` (73-95).
**Confidence:** HIGH.

### BC-TMUXADAPTER-015: The key alphabet sent by `SendToPane` is exactly the 3-element set `{"1", "2", "3"}` (no named keys, no modifiers, no Unicode)
**Preconditions:** any.
**Postconditions:** Every byte string passed to `client.SendKeys` is one of `"1"`, `"2"`, `"3"`. Therefore no quoting/escaping is possible or required at this layer.
**Evidence:** `sendkeys.go:16-20` (the only key sources) + `sendkeys.go:54` (clamp also produces a single digit via `fmt.Sprintf("%d", maxOpt)`).
**Confidence:** HIGH. **This refutes the orchestrator-brief premise** that SendToPane needs to deal with Up/Down/Enter/Tab/Escape/BSpace/Space/C-c/M-x/Unicode/bracketed-paste.

### BC-TMUXADAPTER-016: `SendToPane` does not validate or sanitize the `window` string itself
**Preconditions:** any.
**Postconditions:** The window string flows through unchanged (modulo prefix) to the underlying `Client`. Validation is the lower-layer's responsibility (`validateControlTarget` in control.go:190-198 rejects spaces/`;`/newlines; ExecClient does not validate beyond what `exec.Command` provides).
**Evidence:** `sendkeys.go:39-57`. **Not tested** at the adapter level (no test passes a malformed window like `"foo;bar"`).
**Confidence:** HIGH (code).

---

## 7. Findings (new P0/P1 risks for monocle port)

### F-TMUXADAPTER-001 (P3 — informational): Brief's tmux-3.4 framing is wrong
DetectMaxOption is not version detection. The Rust port should not couple it to tmux version checks.
**Severity:** Informational / framing fix.

### F-TMUXADAPTER-002 (P2): Unknown `Choice` values are silently dropped
`sendkeys.go:34-37` — `if !ok { return nil }`. A future addition like `choice.Defer = 4` would be a no-op with no logging. For monocle, recommend logging at debug level or returning `errors.New("unsupported choice")`.
**Severity:** Low (defensive coding gap).

### F-TMUXADAPTER-003 (P1): Best-effort send on capture error bypasses clamping
`sendkeys.go:45-49` — if `CapturePaneANSI` fails (e.g. tmux server hiccup, pane killed), the adapter sends the **unclamped** digit. On a 2-option dialog, choice=Reject(3) becomes `tmux send-keys ... 3`, which tmux delivers as the keystroke `3`, which the Claude dialog ignores (out-of-range). **The user's "Reject" intent is silently lost.** A safer behavior: surface the capture error to the caller and let the consumer decide whether to retry or fail. Monocle port should consider this.
**Severity:** Medium (silent intent loss under transient tmux errors).

### F-TMUXADAPTER-004 (P1): Stale-capture race window is unbounded
Between `CapturePaneANSI` (T0) and `SendKeys` (T0+ε) tmux may redraw. No generation token, no `wait-for`, no retry. In practice human latency keeps this safe, but an automated agent firing SendToPane while Claude is mid-redraw could clamp against a stale dialog. **Severity:** Low in practice, but worth noting for any future "auto-pilot" mode.

### F-TMUXADAPTER-005 (P2): Hardcoded `sessionPrefix = "lazyclaude"`
`sendkeys.go:12`. The string `"lazyclaude"` is duplicated across the codebase (BC-LIFE-* in Pass 3 has the matching session-creation side). A Rust monocle port should derive this from a shared `const` or config to avoid drift when renaming.
**Severity:** Tech-debt / DRY.

### F-TMUXADAPTER-006 (P3): `ansiEscape` regex does not strip non-CSI sequences
`detect.go:15` — `\x1b\[[0-9;]*[a-zA-Z]` matches only CSI sequences ending in an ASCII letter. Bracketed-paste markers (`ESC[200~` / `ESC[201~`), OSC sequences (`ESC]...BEL`), and DCS sequences (`ESC P ... ESC \`) are not stripped. In Claude Code's dialog capture, this is probably safe (the dialog is plain text + CSI colors), but if Claude ever emits OSC for terminal-title updates between dialog lines, the line would be polluted. **Severity:** Low (no known triggering case).

### F-TMUXADAPTER-007 (P2): No test exercises the `CapturePaneANSI` error path
The mock supports `ErrCapture` (mock.go:32) but `sendkeys_test.go` never sets it. BC-TMUXADAPTER-013 is code-grounded but not test-grounded. Monocle's Rust port should add this test.
**Severity:** Test-coverage gap.

### F-TMUXADAPTER-008 (P3): `DetectMaxOption` `0)`-prefixed and re-numbered blocks not tested
Inputs like `"0) Skip\n1) Yes\n2) No"` (0-indexed dialog) would yield `2`, because the `0)` line is silently dropped (BC-TMUXADAPTER-004). Probably correct, but untested. **Severity:** Low.

---

## 8. Rust port summary (the consumable artifact)

### `detect_max_option(pane_content: &str) -> u32`

```rust
// Pseudocode — porter-ready
const DEFAULT_MAX_OPTION: u32 = 3;
// Single regex with two captures; ANSI-strip per line; same state machine.
//
// option_pattern: ^\s*(?:[>❯➜]\s+)?(\d+)[.)]\s+.+
// ansi_escape:    \x1b\[[0-9;]*[A-Za-z]
//
// State: current: Vec<u32>, last_block: Vec<u32>
// For each line:
//   clean = ansi_escape.replace_all(line, "")
//   match option_pattern.captures(&clean):
//     None: if !current.is_empty() { last_block = current; current = Vec::new(); }
//     Some(caps):
//       n = caps[1].parse::<u32>().ok()? // skip on parse error
//       if n == 1 { current = vec![1]; }
//       else if !current.is_empty() && n == (current.len() as u32) + 1 { current.push(n); }
//       // else: silently ignore
// // Final flush:
// if !current.is_empty() { last_block = current; }
// if last_block.is_empty() { DEFAULT_MAX_OPTION } else { last_block.len() as u32 }
```

### `send_to_pane(client: &dyn TmuxClient, window: &str, c: Choice) -> Result<()>`

```rust
const SESSION_PREFIX: &str = "lazyclaude";

fn send_to_pane(ctx: &Ctx, client: &dyn TmuxClient, window: &str, c: Choice) -> Result<()> {
    let key = match c {
        Choice::Cancel => return Ok(()),
        Choice::Accept => "1",
        Choice::Allow  => "2",
        Choice::Reject => "3",
        _ => return Ok(()), // Mirror lazyclaude's silent-drop; consider logging.
    };
    let target = if window.contains(':') {
        Cow::Borrowed(window)
    } else {
        Cow::Owned(format!("{}:{}", SESSION_PREFIX, window))
    };
    let mut effective_key = key.to_string();
    if let Ok(content) = client.capture_pane_ansi(ctx, &target) {
        let max_opt = detect_max_option(&content);
        if (c as u32) > max_opt {
            effective_key = max_opt.to_string();
        }
    } // On error: fall through with unclamped key (mirror, but consider F-TMUXADAPTER-003)
    client.send_keys(ctx, &target, &[&effective_key])
}
```

Notes:
- The Choice enum's discriminants must match Go: `Cancel=0, Accept=1, Allow=2, Reject=3`.
- `TmuxClient::send_keys` takes a slice of strings (matching tmux `send-keys` variadic args) but this caller always passes exactly one element.
- Consider tightening F-TMUXADAPTER-003 in monocle (surface capture errors) — note in the spec as a deliberate divergence if you do.

---

## 9. Delta Summary

- **New BC contracts added:** 16 (BC-TMUXADAPTER-001 through 016)
- **New findings:** 8 (F-TMUXADAPTER-001 through 008)
- **Framing corrections to upstream prompt:** 1 (DetectMaxOption is not tmux-version detection)
- **Existing items refined:** Pass 8's "BC-MCPSRV-004 (DetectMaxOption use)" is now backed by first-class BC-TMUXADAPTER-001..009; the SendChoice/SendToPane consumer wiring at `local_provider.go:150` and `server.go:497` now has explicit contracts for both consumer paths.
- **Remaining gaps:** None within the adapter package. All 4 files fully read; 2 exported functions both have full contracts; both consumers identified and cited at file:line.
- **Refutations of brief assumptions:**
  - DetectMaxOption is not version detection → BC-TMUXADAPTER-001
  - SendKeys does not handle named keys / modifiers / Unicode at this layer → BC-TMUXADAPTER-015
  - Bracketed paste is not in this adapter → §5
  - The control.go:176-179 Unicode TODO does not propagate here → §4

## 10. Novelty Assessment

**Novelty: SUBSTANTIVE** for round 1.

Justification: This adapter previously had zero BC-TMUXADAPTER-* contracts (B5 audit confirmed). Round 1 has produced:
- 16 new file:line-grounded behavioral contracts.
- 4 explicit refutations of orchestrator-brief assumptions (which themselves change how a Rust porter would approach the work — they prevent over-engineering of features that aren't in this adapter).
- 3 medium-severity findings (F-TMUXADAPTER-002, F-TMUXADAPTER-003, F-TMUXADAPTER-007) the spec author will want to address as deliberate design decisions for monocle.
- A complete Rust-ready pseudocode summary.

Would removing this round's findings change how you'd spec the system? **Yes** — without this round, the spec would either (a) lack contracts for the choice→key path entirely, or (b) inherit the orchestrator-brief's incorrect framing about tmux version detection and Unicode keystroke handling, leading to misallocated porter effort.

## 11. Convergence Declaration

**Pass B `internal/adapter/tmuxadapter` has converged after Round 1.**

Justification:
- The package is 126 source LOC across 2 files. All lines have been read and every behavior is now contract-backed.
- Both exported functions are fully spec'd with preconditions, postconditions, evidence, and confidence levels.
- All 7 unit tests + 2 benchmarks have been mapped to their corresponding contracts.
- Both call sites (server.go:497, local_provider.go:150) have been identified and the data flow into/out of the adapter is documented.
- No code paths remain unexamined.
- The 8 findings raised are spec-author-level decisions (whether to mirror Go behavior or deliberately diverge in Rust), not "we don't understand the code yet" gaps.

A round 2 would, at this point, only produce documentation-style refinements (better prose, additional table entries for already-known behaviors). That meets the NITPICK definition: removing those refinements would not change how the system is spec'd.

The brief's cap of 3 rounds is therefore not exercised.

## 12. State Checkpoint

```yaml
pass: B
subsystem: internal/adapter/tmuxadapter
round: 1
status: complete
files_read_full: [detect.go, sendkeys.go, detect_test.go, sendkeys_test.go]
files_read_supporting:
  - internal/core/tmux/control.go (re-read 150-210 for SendKeysLiteral and TODO verification)
  - internal/core/tmux/exec.go (re-read 300-360 for SendKeys / CapturePaneANSI signatures)
  - internal/core/tmux/mock.go (full, for test-mock semantics)
  - internal/core/choice/choice.go (full, for Choice discriminants)
  - internal/server/server.go (485-510, consumer)
  - cmd/lazyclaude/local_provider.go (130-160, consumer)
contracts_drafted: 16  # BC-TMUXADAPTER-001..016
findings: 8           # F-TMUXADAPTER-001..008
total_tmux_contracts: 40  # 8 BC-TMUX-CTL (Pass 3) + 16 BC-TMUX-EXEC/CLIENT (deep-tmux-r1) + 16 BC-TMUXADAPTER (this round)
brief_assumption_corrections: 4
timestamp: 2026-05-11T19:05:00Z
novelty: SUBSTANTIVE
convergence: PASS-B-TMUXADAPTER CONVERGED (round 1 — honest NITPICK reached for round 2)
followups_for_spec_author:
  - F-TMUXADAPTER-003 (capture-error → unclamped send)
  - F-TMUXADAPTER-005 (hardcoded session prefix)
  - F-TMUXADAPTER-007 (missing test coverage for ErrCapture)
next_subsystem: (none; B5 audit's adapter gap closed)
```
