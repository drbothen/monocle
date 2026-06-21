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

/// Install a panic hook that restores the terminal to normal mode before
/// the default panic handler runs (AC-009).
///
/// Without this hook, a panic in raw-mode leaves the terminal corrupted,
/// requiring the user to run `reset` or close the terminal emulator.
fn install_panic_hook() {
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        // Best-effort terminal restore — errors are ignored because:
        // (a) the process is panicking and about to exit, and
        // (b) logging may not be functional during unwinding.
        //
        // Keyboard enhancement teardown BEFORE disable_raw_mode (symmetric with setup).
        // AC-008 / BC-2.09.004 Invariant 1: must pop Kitty flags + disable bracketed paste
        // on ALL exit paths including panics, or the parent terminal inherits the flags.
        teardown_keyboard_enhancement();
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
        let _ = io::stdout().flush();
        // Invoke the original (default) panic handler after restoring the terminal
        // so the panic message is visible to the user.
        original_hook(panic_info);
    }));
}

/// Restore the terminal to normal mode: pop keyboard enhancement flags, disable
/// bracketed paste, disable raw mode, leave the alternate screen, restore cursor
/// visibility, and flush stdout. Called on all exit paths.
///
/// Keyboard enhancement teardown runs BEFORE disable_raw_mode so the terminal
/// is still in raw mode when the control sequences are sent (AC-008 / BC-2.09.004
/// Invariant 1: flags must be popped on all TUI exit paths).
///
/// Errors from teardown are logged at WARN level and ignored — the process
/// is exiting and there is nothing useful to do with a teardown failure.
fn restore_terminal() {
    // Pop Kitty keyboard enhancement flags + disable bracketed paste FIRST,
    // before disabling raw mode (symmetric with setup order in setup_terminal).
    teardown_keyboard_enhancement();
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

/// Initialise the terminal: enable raw mode, enter the alternate screen, and
/// install global Kitty keyboard enhancement flags + bracketed paste.
///
/// # Errors
///
/// Returns `Err` if either step fails. On partial failure (raw mode enabled
/// but alternate-screen entry fails), raw mode is cleaned up before returning
/// the error, preventing a terminal raw-mode leak (F-S025-ADV2-HIGH-001).
///
/// `setup_keyboard_enhancement` failures (PushKeyboardEnhancementFlags I/O error)
/// are propagated; if the terminal does not support Kitty protocol,
/// `PushKeyboardEnhancementFlags` silently no-ops (AC-007 / EC-216 — no panic).
fn setup_terminal() -> Result<()> {
    enable_raw_mode()?;
    if let Err(e) = execute!(io::stdout(), EnterAlternateScreen) {
        // raw_mode succeeded; alt-screen failed — clean up raw mode before propagating
        // so the caller's terminal is left in a usable state.
        let _ = disable_raw_mode();
        return Err(e.into());
    }
    // Install Kitty keyboard enhancement flags + bracketed paste globally at startup
    // (BC-2.09.004 Invariant 1 / BC-2.09.005 Invariant 1: global, not gated on mode).
    // On unsupported terminals, PushKeyboardEnhancementFlags silently no-ops (AC-007).
    if let Err(e) = setup_keyboard_enhancement() {
        // Enhancement failed (I/O error writing to stdout) — restore and propagate.
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, cursor::Show);
        return Err(e);
    }
    Ok(())
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

    install_panic_hook();
    setup_terminal()?;

    let result = monocle_tui::app::run().await;

    // Restore terminal on all exit paths (AC-009): raw mode off, leave alternate
    // screen, show cursor, flush stdout — BEFORE returning the result.
    restore_terminal();

    result
}
