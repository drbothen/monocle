//! Key-binding resolution for the monocle TUI.
//!
//! Defines a pure-core key event representation (no crossterm dependency) and
//! the 5-level binding precedence resolver:
//! `SearchPrompt > UserCustomCommand > PerContext > Global > Builtin`.
//!
//! All I/O and crossterm integration is the responsibility of the binary crate;
//! this module is dependency-free with respect to terminal libraries.

use super::state::{Action, AppMode};
use std::collections::HashMap;

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

/// Mode discriminant for `PerContext` bindings.
///
/// A mode-tag enum (discriminant-only) is used as the key in the per-context
/// binding map to avoid embedding full `AppMode` values (which contain
/// non-`Hash` fields) in the HashMap key.
///
/// `#[non_exhaustive]` — mirrors `AppMode` and gains new variants in lockstep
/// with `AppMode`. Binary crates constructing `BindingLayers::per_context` maps
/// must use a wildcard arm for any unhandled mode tags.
#[non_exhaustive]
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum AppModeTag {
    /// `AppMode::Dashboard` variant.
    Dashboard,
    /// `AppMode::Filtering` variant.
    Filtering,
    /// `AppMode::Overlay` variant.
    Overlay,
    /// `AppMode::Fullscreen` variant.
    Fullscreen,
    /// `AppMode::EmbeddedTerminal` variant (S-039, BC-2.09.001).
    EmbeddedTerminal,
    /// `AppMode::SessionCreation` variant (S-039 / S-033).
    SessionCreation,
}

impl AppModeTag {
    /// Derive the tag from a live `AppMode` value.
    pub fn from_mode(mode: &AppMode) -> AppModeTag {
        match mode {
            AppMode::Dashboard { .. } => AppModeTag::Dashboard,
            AppMode::Filtering { .. } => AppModeTag::Filtering,
            AppMode::Overlay { .. } => AppModeTag::Overlay,
            AppMode::Fullscreen { .. } => AppModeTag::Fullscreen,
            // S-039: new modes
            AppMode::EmbeddedTerminal { .. } => AppModeTag::EmbeddedTerminal,
            AppMode::SessionCreation { .. } => AppModeTag::SessionCreation,
        }
    }
}

/// Layered binding configuration evaluated during `resolve_binding`.
///
/// Five layers in descending priority order:
/// 1. `search_prompt` — active when in `Filtering` or `Overlay` modes;
///    captures printable characters and special decision keys.
/// 2. `user_custom_command` — user-configured commands from the customization file.
/// 3. `per_context` — mode-scoped bindings keyed by `(KeyEvent, AppModeTag)`.
/// 4. `global` — active in all modes.
/// 5. `builtin` — hard-coded fallback bindings.
pub struct BindingLayers {
    /// SearchPrompt layer: printable chars in Filtering mode, permission decision
    /// keys in Overlay mode.  Keyed by `KeyEvent` only; mode restriction is
    /// applied by `resolve_binding` before consulting this table.
    pub search_prompt: HashMap<KeyEvent, Action>,
    /// User-configured command bindings.
    pub user_custom_command: HashMap<KeyEvent, Action>,
    /// Context-sensitive bindings keyed by (KeyEvent, AppModeTag).
    pub per_context: HashMap<(KeyEvent, AppModeTag), Action>,
    /// Global key bindings active in all modes.
    pub global: HashMap<KeyEvent, Action>,
    /// Hard-coded fallback bindings.
    pub builtin: HashMap<KeyEvent, Action>,
}

impl BindingLayers {
    /// Construct an empty binding layer stack (no bindings at any level).
    pub fn empty() -> Self {
        Self {
            search_prompt: HashMap::new(),
            user_custom_command: HashMap::new(),
            per_context: HashMap::new(),
            global: HashMap::new(),
            builtin: HashMap::new(),
        }
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
///
/// `#[non_exhaustive]` — additional key codes (e.g., function keys, media keys)
/// may be added as terminal backend support expands.
///
/// `Unknown` is the sentinel for crossterm key codes that don't map to any named
/// variant. The binding resolver never matches `Unknown`, ensuring unmapped keys
/// produce no action. This replaces the fragile `Char('\0')` sentinel pattern
/// (F-S025-ADV2-LOW-001).
#[non_exhaustive]
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
    /// An unknown or unmapped crossterm key code.
    ///
    /// The binding resolver never registers bindings for `Unknown`, so any key
    /// that maps here silently produces no action. This is the canonical sentinel
    /// for crossterm keys not listed in this enum (F-S025-ADV2-LOW-001).
    Unknown,
}

/// Modifier keys held simultaneously with a key press.
///
/// A simple struct of booleans — no bitflags dependency required in the pure
/// core. The binary crate converts from crossterm's bitflags representation.
///
/// `Copy` is derived because `KeyModifiers` is a small value type (3 bools);
/// `HashMap` lookups produce cloned values, and binding construction code calls
/// `KeyModifiers::default()` repeatedly — `Copy` eliminates these `.clone()` calls
/// (F-S025-ADV2-MED-001).
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct KeyModifiers {
    /// The Shift key was held.
    pub shift: bool,
    /// The Ctrl key was held.
    pub ctrl: bool,
    /// The Alt / Meta key was held.
    pub alt: bool,
}

/// Returns `true` if the key event represents a printable character with no
/// modifier keys (i.e., a character the user intends to type into a text field).
///
/// Ctrl-modified keys are NOT printable in this sense — they represent commands.
/// Alt-modified keys are similarly excluded. Shift-modified Char keys (like 'A')
/// ARE printable (shift is encoded in the character value).
fn is_printable_char(key: &KeyEvent) -> bool {
    matches!(key.code, KeyCode::Char(_)) && !key.modifiers.ctrl && !key.modifiers.alt
}

/// Resolve a key event to an action and the precedence level that matched.
///
/// Evaluates the five binding layers in precedence order and returns the first
/// match. Returns `None` when no binding is registered for this key in the
/// current mode.
///
/// # SearchPrompt layer semantics
///
/// The SearchPrompt layer is activated in two modes:
///
/// 1. **`Filtering` mode:** all printable characters (no Ctrl/Alt modifier) are
///    captured as `Action::FilterType(char)`. Non-printable keys (Esc, Enter,
///    Ctrl-modified, Tab) fall through to lower layers.
///
/// 2. **`Overlay` mode:** permission decision keys are captured:
///    - `'y'` or `Enter` → `Action::PermissionAcceptOnce`
///    - `'A'` → `Action::PermissionAcceptAlways`
///    - `'n'` or `'r'` → `Action::PermissionReject`
///
/// In all other modes, the SearchPrompt layer is skipped.
pub fn resolve_binding(
    key: &KeyEvent,
    mode: &AppMode,
    layers: &BindingLayers,
) -> Option<(Action, BindingSource)> {
    // --- Level 1: SearchPrompt ---
    // Mode-sensitive: activated differently in Filtering vs Overlay.
    match mode {
        AppMode::Filtering { .. } => {
            // In Filtering mode: printable chars → FilterType; others fall through.
            if is_printable_char(key) {
                if let KeyCode::Char(c) = key.code {
                    return Some((Action::FilterType(c), BindingSource::SearchPrompt));
                }
            }
            // Non-printable keys: check search_prompt table, then fall through.
            if let Some(action) = layers.search_prompt.get(key) {
                return Some((clone_action(action), BindingSource::SearchPrompt));
            }
        }
        AppMode::Overlay { .. } => {
            // In Overlay mode: permission decision keys and stack-navigation keys
            // captured by SearchPrompt (highest precedence level).
            //
            // Modifier guard: only match if Ctrl and Alt are NOT held. Shift is
            // permitted because 'A' (AcceptAlways) is the uppercase form and shift
            // is already encoded in the KeyCode::Char('A') value.
            //
            // Without this guard, Ctrl+Y would match KeyCode::Char('y') and fire
            // PermissionAcceptOnce — incorrect since modifier keys change the
            // semantic intent of a keypress.
            //
            // Up/Down are included here (AC-013, AC-014, BC-2.06.009) alongside the
            // decision keys so that stack rotation is live in production.  The Builtin
            // layer maps Up/Down → SelectPrev/SelectNext for Dashboard navigation; by
            // intercepting them here in the SearchPrompt arm we prevent the Builtin
            // bindings from silently no-op'ing in Overlay mode.  Dashboard Up/Down is
            // unaffected: the Overlay arm only executes when `mode` is
            // `AppMode::Overlay { .. }`.
            let overlay_action = if key.modifiers.ctrl || key.modifiers.alt {
                // Ctrl/Alt modified keys are not permission decisions — fall through.
                None
            } else {
                match &key.code {
                    KeyCode::Char('y') => Some(Action::PermissionAcceptOnce),
                    KeyCode::Enter => Some(Action::PermissionAcceptOnce),
                    KeyCode::Char('A') => Some(Action::PermissionAcceptAlways),
                    KeyCode::Char('n') => Some(Action::PermissionReject),
                    KeyCode::Char('r') => Some(Action::PermissionReject),
                    // AC-013/AC-014 (BC-2.06.009): Up/Down rotate the overlay stack.
                    // Both directions map to the same action (circular rotation).
                    KeyCode::Up => Some(Action::OverlayCycleNext),
                    KeyCode::Down => Some(Action::OverlayCycleNext),
                    _ => None,
                }
            };
            if let Some(action) = overlay_action {
                return Some((action, BindingSource::SearchPrompt));
            }
            // Other keys in Overlay: check search_prompt table, then fall through.
            if let Some(action) = layers.search_prompt.get(key) {
                return Some((clone_action(action), BindingSource::SearchPrompt));
            }
        }
        _ => {
            // Dashboard / Fullscreen: SearchPrompt layer not active for special captures,
            // but user-configured search_prompt bindings still check against the table.
            if let Some(action) = layers.search_prompt.get(key) {
                return Some((clone_action(action), BindingSource::SearchPrompt));
            }
        }
    }

    // --- Level 2: UserCustomCommand ---
    if let Some(action) = layers.user_custom_command.get(key) {
        return Some((clone_action(action), BindingSource::UserCustomCommand));
    }

    // --- Level 3: PerContext (mode-scoped) ---
    let mode_tag = AppModeTag::from_mode(mode);
    if let Some(action) = layers.per_context.get(&(key.clone(), mode_tag)) {
        return Some((clone_action(action), BindingSource::PerContext));
    }

    // --- Level 4: Global ---
    if let Some(action) = layers.global.get(key) {
        return Some((clone_action(action), BindingSource::Global));
    }

    // --- Level 5: Builtin ---
    if let Some(action) = layers.builtin.get(key) {
        return Some((clone_action(action), BindingSource::Builtin));
    }

    // No match at any level.
    None
}

/// Clone an `Action` value.
///
/// `Action` is `#[non_exhaustive]` and does not derive `Clone` — we clone
/// by reconstructing each variant explicitly. This function lives in the
/// binding module because it is only needed for the binding resolver (callers
/// own the `BindingLayers` and need cloned actions to return owned values).
fn clone_action(action: &Action) -> Action {
    match action {
        Action::StartFilter { panel } => Action::StartFilter {
            panel: panel.clone(),
        },
        Action::CommitFilter => Action::CommitFilter,
        Action::CancelFilter => Action::CancelFilter,
        Action::EnterFullscreen { panel } => Action::EnterFullscreen {
            panel: panel.clone(),
        },
        Action::ExitFullscreen => Action::ExitFullscreen,
        Action::PushOverlay { .. } => {
            // PromptModal contains Instant which is not Clone, and PushOverlay
            // is not expected to be stored in BindingLayers — it arrives via IPC.
            // Provide a fallback: Noop. In practice this branch is unreachable
            // from BindingLayers storage.
            //
            // debug_assert fires in test builds if this branch is ever reached,
            // surfacing a BindingLayers misconfiguration early.
            debug_assert!(
                false,
                "PushOverlay cannot be cloned from binding table — use direct dispatch"
            );
            Action::Noop
        }
        Action::PopOverlay => Action::PopOverlay,
        Action::Esc => Action::Esc,
        Action::MoveFocus => Action::MoveFocus,
        Action::FilterType(c) => Action::FilterType(*c),
        Action::OverlayCycleNext => Action::OverlayCycleNext,
        Action::PermissionAcceptOnce => Action::PermissionAcceptOnce,
        Action::PermissionAcceptAlways => Action::PermissionAcceptAlways,
        Action::PermissionReject => Action::PermissionReject,
        Action::PermissionTraceToSource => Action::PermissionTraceToSource,
        Action::SelectNext => Action::SelectNext,
        Action::SelectPrev => Action::SelectPrev,
        Action::ScrollDown => Action::ScrollDown,
        Action::ScrollUp => Action::ScrollUp,
        Action::Quit => Action::Quit,
        Action::Noop => Action::Noop,
        // S-031 (BC-2.07.005 INV-1): ProfilePicker is a Global-layer action and must
        // be cloneable so resolve_binding can return it from the BindingLayers table.
        Action::ProfilePicker => Action::ProfilePicker,
        // Non-exhaustive guard: new Action variants added in future waves are treated
        // as Noop until the binding resolver is updated to handle them explicitly.
        // This is safe because BindingLayers storage is controlled by monocle-tui,
        // which will be updated in the same wave cycle as any new Action variant.
        #[allow(unreachable_patterns)]
        _ => Action::Noop,
    }
}
