---
type: research
topic: tui-term 0.3.4 scrollback-viewport rendering API
date: 2026-06-15
for: S-043 (scrollback navigation), ADR-0011 (embedded-PTY stack)
versions-pinned: portable-pty 0.9.0, vt100 0.16.2, tui-term =0.3.4
confidence: HIGH
status: complete
---

# tui-term 0.3.4 Scrollback-Viewport Rendering API — Authoritative Findings

## TL;DR (the answer story-writer can pin in S-043)

**tui-term 0.3.4's `PseudoTerminal` widget does NOT accept a scrollback/viewport offset
directly.** Scrollback is driven entirely on the `vt100::Screen` via
`set_scrollback(offset)`, then the (already-scrolled) screen is handed to
`PseudoTerminal::new(&screen)`. Path (b) in the S-043 draft is the canonical answer.

### Concrete, version-pinned call sequence (offset N rows up; 0 = live bottom)

```rust
// parser: &mut vt100::Parser created with a non-zero scrollback_len, e.g.
//   let mut parser = vt100::Parser::new(rows, cols, scrollback_len /* > 0 */);
// (scrollback_len MUST be > 0 at Parser construction or there is no history to view)

// 1. Drive the scrollback offset on the Screen (mutates which rows the Screen reports).
parser.screen_mut().set_scrollback(n);   // n = rows scrolled up; 0 == live bottom; clamped to actual scrollback size

// 2. Hand the now-scrolled Screen (immutable ref) to the widget and render.
let widget = tui_term::widget::PseudoTerminal::new(parser.screen())
    .block(/* optional Block */)
    .style(/* optional Style */)
    .cursor(/* optional Cursor */);
frame.render_widget(widget, area);

// 3. To return to live view, set offset back to 0:
parser.screen_mut().set_scrollback(0);
```

`set_scrollback` changes the return values of subsequent cell/content reads on the
`Screen`, which is exactly what `PseudoTerminal`'s `Widget` impl consumes during render.
No tui-term-side viewport state exists.

---

## Verified facts (each against the EXACT pinned version)

### 1. tui-term 0.3.4 `PseudoTerminal` API — verified against docs.rs/0.3.4

Source: https://docs.rs/tui-term/0.3.4/tui_term/widget/struct.PseudoTerminal.html

```rust
#[non_exhaustive]
pub struct PseudoTerminal<'a, S> { /* private fields */ }

impl<'a, S> PseudoTerminal<'a, S> {
    pub fn new(screen: &'a S) -> Self
    pub fn block(self, block: Block<'a>) -> Self
    pub fn cursor(self, cursor: Cursor) -> Self
    pub fn style(self, style: Style) -> Self
    pub const fn screen(&self) -> &S
}
```

- **No** `.scrollback(n)`, `.viewport(...)`, `.offset(...)`, or `.scroll(...)` method exists
  on the 0.3.4 widget. Builder surface is exactly `block`, `cursor`, `style` (plus the
  `screen` getter and `new` ctor).
- Generic over `S`; in practice `S = vt100::Screen`. Widget is rendered via the `Widget`
  impl on both `&PseudoTerminal` and `PseudoTerminal`.
- The "`.scroll((scroll, 0))`" snippet seen in some third-party code (e.g. skim) is a
  ratatui `Paragraph` method, NOT a tui-term `PseudoTerminal` method. Do not pin against it.

### 2. tui-term 0.3.4 renders from `vt100::Screen`, depends on vt100 ^0.16.2 — verified against crates.io/0.3.4

Source: crates.io dependencies API for tui-term 0.3.4
(https://crates.io/api/v1/crates/tui-term/0.3.4/dependencies)

Normal deps (0.3.4):
- `ratatui-core` ^0.1.0 (required)
- `ratatui-widgets` ^0.3.0 (required)
- `portable-pty` ^0.9.0 (optional)
- **`vt100` ^0.16.2 (optional)**

This CONFIRMS ADR-0011's pin set (vt100 0.16.2, portable-pty 0.9.0). Crate-level example
(docs.rs/tui-term/0.3.4) shows the canonical construction:
`PseudoTerminal::new(parser.screen())` from a `vt100::Parser`.

> CORRECTION to an earlier (non-authoritative) finding: a `perplexity_research` pass read
> the docs.rs **/latest** pages and reported vt100 `^1.11.1`. That is the LATEST tui-term's
> dependency, NOT 0.3.4's. The version-pinned crates.io/0.3.4 source is authoritative and
> says **^0.16.2**. Always pin the explicit version in the URL.

### 3. vt100 0.16.2 scrollback API + offset semantics — verified against docs.rs/0.16.2

Source: https://docs.rs/vt100/0.16.2/vt100/struct.Screen.html
        https://docs.rs/vt100/0.16.2/vt100/struct.Parser.html

```rust
// vt100::Screen (0.16.2)
pub fn set_scrollback(&mut self, rows: usize)   // "offset from the top of the screen; 0 == normal screen in view"; clamped to actual scrollback size
pub fn scrollback(&self) -> usize               // "0 when the normal screen is in view"

// vt100::Parser (0.16.2)
pub fn new(rows: u16, cols: u16, scrollback_len: usize) -> Self
pub fn screen(&self) -> &Screen
pub fn screen_mut(&mut self) -> &mut Screen      // needed to call set_scrollback
```

- **Offset semantics:** `0` = live/normal screen (bottom). `N` = scrolled `N` rows up into
  history. Value is **clamped** to the actual scrollback size — safe to pass a large N to
  go "to top".
- `scrollback_len` passed to `Parser::new` MUST be `> 0` for any history to exist. The
  common docs example uses `0`, which disables scrollback entirely. S-043 must allocate a
  real scrollback budget at parser construction.
- `set_scrollback` requires `&mut Screen`; obtain via `Parser::screen_mut()`. Render-time
  pass the immutable `Parser::screen()` to `PseudoTerminal::new`.

---

## Production-grade guidance for S-043

- Pin the call sequence as path (b) above. Remove the "implementer-time investigation" /
  "check tui-term 0.3.4 API" language — the answer is settled.
- BC/spec note: scrollback capacity is a Parser-construction parameter (`scrollback_len`),
  not a render-time knob. S-043 should specify the scrollback budget (rows) explicitly.
- Clamping is handled by vt100, so the offset state machine in monocle can hold an
  unclamped logical offset and rely on vt100 to clamp at render, OR clamp against
  `Screen::scrollback()` for accurate UI affordances (e.g., "top reached"). Reading back
  `screen().scrollback()` after a set gives the effective (clamped) offset for status display.
- The widget itself is stateless w.r.t. scroll; do not attempt to persist viewport state in
  `PseudoTerminal`.

## Sources

- docs.rs/tui-term/0.3.4 PseudoTerminal: https://docs.rs/tui-term/0.3.4/tui_term/widget/struct.PseudoTerminal.html
- docs.rs/tui-term/0.3.4 crate root (example): https://docs.rs/tui-term/0.3.4/tui_term/
- crates.io tui-term 0.3.4 deps: https://crates.io/api/v1/crates/tui-term/0.3.4/dependencies
- docs.rs/vt100/0.16.2 Screen: https://docs.rs/vt100/0.16.2/vt100/struct.Screen.html
- docs.rs/vt100/0.16.2 Parser: https://docs.rs/vt100/0.16.2/vt100/struct.Parser.html
- upstream repo: https://github.com/a-kenji/tui-term (tag v0.3.4)

## Research Methods

| Tool | Queries | Purpose |
|------|---------|---------|
| Perplexity perplexity_research (PRIMARY) | 1 | Initial deep sweep of tui-term/vt100 API; FLAGGED for reading /latest not /0.3.4 — version-corrected below |
| Context7 resolve-library-id | 1 | Attempted tui-term resolution (no exact match returned; pivoted to version-pinned docs.rs) |
| WebFetch | 5 | Authoritative version-pinned verification: docs.rs/tui-term/0.3.4 (widget + crate root), crates.io/0.3.4 deps, docs.rs/vt100/0.16.2 (Screen + Parser) |
| Training data | 0 areas | None relied upon; all claims sourced to pinned docs |

**Total MCP tool calls:** 2 (1 perplexity_research, 1 context7)
**Training data reliance:** low — every API fact verified against the exact pinned-version docs.rs/crates.io pages. The one Perplexity version error (vt100 ^1.11.1) was caught and corrected by version-pinned WebFetch.
