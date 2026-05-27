//! Key-binding resolution for the monocle TUI.
//!
//! Defines a pure-core key event representation (no crossterm dependency) and
//! the 5-level binding precedence resolver:
//! `SearchPrompt > UserCustomCommand > PerContext > Global > Builtin`.
//!
//! All I/O and crossterm integration is the responsibility of the binary crate;
//! this module is dependency-free with respect to terminal libraries.

/// Source (precedence level) from which a key binding was resolved.
///
/// The five levels implement the canonical binding precedence defined in the
/// monocle product brief: SearchPrompt (highest) … Builtin (lowest).
///
/// `#[non_exhaustive]` — future precedence levels (e.g., plugin-supplied
/// bindings) may be inserted without a semver break.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum BindingSource {
    /// Transient bindings active only while a search/filter prompt is open.
    SearchPrompt,
    /// User-defined commands from the user customisation file.
    UserCustomCommand,
    /// Context-sensitive bindings scoped to the currently focused panel.
    PerContext,
    /// Global bindings active in any mode.
    Global,
    /// Hard-coded fallback bindings shipped with the binary.
    Builtin,
}

/// Layered binding configuration evaluated during `resolve_binding`.
///
/// Internal representation is deliberately opaque at the stub stage; the
/// implementer will fill in the concrete storage in S-024 implementation.
pub struct BindingLayers {
    _priv: (),
}

impl BindingLayers {
    /// Construct an empty binding layer stack (no bindings at any level).
    pub fn empty() -> Self {
        Self { _priv: () }
    }
}

/// A terminal-library-agnostic key event.
///
/// Defined here rather than re-exported from crossterm to keep `monocle-core`
/// free of I/O dependencies. The binary crate converts `crossterm::event::KeyEvent`
/// into this type before passing it to `resolve_binding`.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct KeyEvent {
    /// The key that was pressed.
    pub code: KeyCode,
    /// Modifier keys held during the key press.
    pub modifiers: KeyModifiers,
}

/// The physical (or logical) key that was pressed.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum KeyCode {
    /// A printable character key.
    Char(char),
    /// The Enter / Return key.
    Enter,
    /// The Escape key.
    Esc,
    /// The Up arrow key.
    Up,
    /// The Down arrow key.
    Down,
    /// The Left arrow key.
    Left,
    /// The Right arrow key.
    Right,
    /// The Tab key.
    Tab,
    /// The Backspace key.
    Backspace,
}

/// Modifier keys held simultaneously with a key press.
///
/// A simple struct of booleans — no bitflags dependency required in the pure
/// core. The binary crate converts from crossterm's bitflags representation.
#[derive(Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct KeyModifiers {
    /// The Shift key was held.
    pub shift: bool,
    /// The Ctrl key was held.
    pub ctrl: bool,
    /// The Alt / Meta key was held.
    pub alt: bool,
}

/// Resolve a key event to an action and the precedence level that matched.
///
/// Evaluates the five binding layers in precedence order and returns the first
/// match. Returns `None` when no binding is registered for this key in the
/// current mode.
pub fn resolve_binding(
    _key: &KeyEvent,
    _mode: &super::state::AppMode,
    _layers: &BindingLayers,
) -> Option<(super::state::Action, BindingSource)> {
    todo!()
}
