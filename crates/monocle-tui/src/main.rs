//! `monocle-tui` binary entry point.
//!
//! Responsibilities (AC-001, AC-009):
//! 1. Install a panic hook that restores the terminal before unwinding.
//! 2. Put the terminal into raw mode and switch to the alternate screen.
//! 3. Drive the main application event loop via [`monocle_tui::app::run`].
//! 4. Restore the terminal on all exit paths (normal, error, panic).

use anyhow::Result;
use crossterm::{
    cursor, execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use monocle_tui::event_loop::{setup_keyboard_enhancement, teardown_keyboard_enhancement};
use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Install a panic hook that restores the terminal to normal mode before
/// the default panic handler runs (AC-009).
///
/// Without this hook, a panic in raw-mode leaves the terminal corrupted,
/// requiring the user to run `reset` or close the terminal emulator.
///
/// `kitty_active` is an `Arc<AtomicBool>` set by `setup_terminal()` after calling
/// `crossterm::terminal::supports_keyboard_enhancement()`. The panic hook captures it so `teardown_keyboard_enhancement`
/// correctly conditionalises `PopKeyboardEnhancementFlags` even when called from
/// a panic context.
fn install_panic_hook(kitty_active: Arc<AtomicBool>) {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Best-effort terminal restore — errors are ignored because:
        // (a) the process is panicking and about to exit, and
        // (b) logging may not be functional during unwinding.
        //
        // Keyboard enhancement teardown BEFORE disable_raw_mode (symmetric with setup).
        // AC-008 / BC-2.09.004 Invariant 1: must pop Kitty flags (if active) + disable
        // bracketed paste on ALL exit paths including panics, or the parent terminal
        // inherits the flags.
        teardown_keyboard_enhancement(kitty_active.load(Ordering::Relaxed));
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
        let _ = io::stdout().flush();
        // Invoke the original (default) panic handler after restoring the terminal
        // so the panic message is visible to the user.
        original_hook(panic_info);
    }));
}

/// Restore the terminal to normal mode: conditionally pop keyboard enhancement flags,
/// disable bracketed paste, disable raw mode, leave the alternate screen, restore cursor
/// visibility, and flush stdout. Called on all exit paths.
///
/// Keyboard enhancement teardown runs BEFORE disable_raw_mode so the terminal
/// is still in raw mode when the control sequences are sent (AC-008 / BC-2.09.004
/// Invariant 1: flags must be popped on all TUI exit paths).
///
/// `kitty_active` must match the value returned by `setup_terminal()`.
///
/// Errors from teardown are logged at WARN level and ignored — the process
/// is exiting and there is nothing useful to do with a teardown failure.
fn restore_terminal(kitty_active: bool) {
    // Pop Kitty keyboard enhancement flags (if active) + disable bracketed paste FIRST,
    // before disabling raw mode (symmetric with setup order in setup_terminal).
    teardown_keyboard_enhancement(kitty_active);
    if let Err(e) = disable_raw_mode() {
        tracing::warn!(error = %e, "failed to disable raw mode during terminal restore");
    }
    if let Err(e) = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show) {
        tracing::warn!(error = %e, "failed to leave alternate screen during terminal restore");
    }
    // Flush stdout after leaving the alternate screen so the cursor-show command
    // and any buffered output reach the terminal before the process exits.
    if let Err(e) = io::stdout().flush() {
        tracing::warn!(error = %e, "failed to flush stdout during terminal restore");
    }
}

/// Initialise the terminal: enable raw mode, enter the alternate screen, probe for Kitty
/// keyboard protocol support via `crossterm::terminal::supports_keyboard_enhancement()`, and install global keyboard enhancement flags +
/// bracketed paste.
///
/// Returns the `kitty_active` bool from `supports_keyboard_enhancement()`. Callers must store this and
/// pass it to every `teardown_keyboard_enhancement` call (normal and panic paths).
///
/// # Errors
///
/// Returns `Err` if either step fails. On partial failure (raw mode enabled
/// but alternate-screen entry fails), raw mode is cleaned up before returning
/// the error, preventing a terminal raw-mode leak (F-S025-ADV2-HIGH-001).
///
/// `setup_keyboard_enhancement` failures (PushKeyboardEnhancementFlags I/O error)
/// are propagated; if the terminal does not support Kitty protocol,
/// `PushKeyboardEnhancementFlags` is not called (AC-007 / EC-216 — no panic).
fn setup_terminal() -> Result<bool> {
    enable_raw_mode()?;
    if let Err(e) = execute!(io::stdout(), EnterAlternateScreen) {
        // raw_mode succeeded; alt-screen failed — clean up raw mode before propagating
        // so the caller's terminal is left in a usable state.
        let _ = disable_raw_mode();
        return Err(e.into());
    }
    // Detect Kitty protocol via supports_keyboard_enhancement() and conditionally install enhancement flags +
    // bracketed paste. Returns kitty_active bool (BC-2.09.004 Invariant 1 /
    // BC-2.09.005 Invariant 1: global, not gated on mode).
    match setup_keyboard_enhancement() {
        Ok(kitty_active) => Ok(kitty_active),
        Err(e) => {
            // Enhancement failed (I/O error writing to stdout) — restore and propagate.
            let _ = disable_raw_mode();
            let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
            Err(e)
        }
    }
}

/// Binary entry point. Wires terminal setup/teardown around the async app loop.
#[tokio::main]
async fn main() -> Result<()> {
    // Install structured tracing subscriber for structured logging (AC-001 / conventions).
    // Best-effort — if tracing setup fails we still proceed without logging rather than crash.
    let _ = tracing_subscriber::fmt()
        .with_target(false)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    // `kitty_active` must be shared between:
    //   - the panic hook (captures Arc clone, reads on panic)
    //   - main (reads after setup_terminal returns; passes to app::run and restore_terminal)
    // Arc<AtomicBool> is the minimal-overhead shared-state primitive for this pattern.
    let kitty_active_arc = Arc::new(AtomicBool::new(false));
    install_panic_hook(Arc::clone(&kitty_active_arc));

    let kitty_active = setup_terminal()?;
    // Store in the Arc so the panic hook (already registered) can read the correct value
    // even if a panic occurs inside app::run().
    kitty_active_arc.store(kitty_active, Ordering::Relaxed);

    let result = monocle_tui::app::run(kitty_active).await;

    // Restore terminal on all exit paths (AC-009): conditionally pop Kitty flags,
    // disable bracketed paste, disable raw mode, leave alternate screen, show cursor,
    // flush stdout — BEFORE returning the result.
    restore_terminal(kitty_active);

    result
}
