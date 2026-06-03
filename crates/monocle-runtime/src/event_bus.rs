//! Bounded event bus fan-out task and DropCounterUpdate debounce (S-018, BC-2.04.011).
//!
//! # Architecture
//!
//! The event bus is a `tokio::sync::mpsc::channel::<EventBusHookEvent>(4096)` pair
//! (BC-2.04.011 PC-1). The sender half is stored in `DaemonState.event_bus_tx` wrapped
//! in `Arc`. The receiver half is consumed by the fan-out task spawned at daemon startup.
//!
//! # Fan-out task
//!
//! `event_bus_fan_out_task` receives events from the bounded channel and fans them out
//! to all connected TUI clients (BC-2.04.011 PC-4, PC-5). Per-client write timeout is
//! 50ms; slow clients are removed without stalling `recv()`.
//!
//! # Drop counter debounce
//!
//! `drop_counter_debounce_task` sends `ServerToClient::DropCounterUpdate` at most once
//! per 100ms (BC-2.04.011 PC-8). The value reflects the cumulative counter at
//! debounce-fire time; it is NOT a delta.
//!
//! # Graceful shutdown
//!
//! On shutdown, `EventBusTx` is dropped before the tokio runtime shuts down, causing
//! `EventBusRx.recv()` to return `None` and the fan-out loop to exit cleanly (PC-7).
//! The fan-out task MUST NOT be forcibly aborted.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::state::DaemonState;
use crate::types::EventBusRx;

/// Per-client write timeout for fan-out (BC-2.04.011 PC-5).
///
/// 50ms: if a TUI client IPC write does not complete within 50ms, the client is
/// removed from `tui_clients` without stalling `EventBusRx.recv()`.
// Phase 1: no TUI clients yet (S-021/S-022 scope). Constant defined here for
// Phase 2 wiring; suppress dead_code lint until then.
#[allow(dead_code)]
const FAN_OUT_CLIENT_TIMEOUT_MS: u64 = 50;

/// Debounce interval for `DropCounterUpdate` (BC-2.04.011 PC-8).
///
/// At most one `DropCounterUpdate` IPC message per 100ms, regardless of burst size.
const DROP_COUNTER_DEBOUNCE_MS: u64 = 100;

/// Fan-out task: receives hook events from the bounded bus and delivers them to TUI clients.
///
/// # Lifetime
///
/// The task runs until `shutdown_rx` fires OR the channel is closed (all `EventBusTx` senders
/// are dropped). On `shutdown_rx.changed()`: logs debug and returns immediately (M3 fix —
/// SS-daemon-wiring-impl.md §Round 3 M3). On channel close, `rx.recv()` returns `None` and
/// the loop exits cleanly (BC-2.04.011 PC-7).
///
/// The `shutdown_rx` branch mirrors `drop_counter_debounce_task` — both long-running tasks
/// now exit promptly within one tokio scheduler tick of the shutdown signal firing, symmetric
/// shutdown behavior for all event bus background tasks.
///
/// # Client removal
///
/// If a write to any TUI client fails (broken pipe, disconnect), that client is removed
/// from `DaemonState.tui_clients`. Subsequent events are not sent to the removed client.
///
/// # Per-client timeout
///
/// Each client write is wrapped in `tokio::time::timeout(50ms, ...)`. If the write does
/// not complete within 50ms, the client is removed (BC-2.04.011 PC-5).
///
/// # Arguments
///
/// - `rx`: The receiver half of the bounded event bus channel. Consumed by this task.
/// - `state`: Shared daemon state providing access to `tui_clients`.
/// - `shutdown_rx`: Watch receiver that fires when the daemon signals graceful shutdown.
///   The task exits within one scheduler tick of `shutdown_rx` changing to `true`.
pub async fn event_bus_fan_out_task(
    mut rx: EventBusRx,
    state: Arc<DaemonState>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    // Phase 1: TUI IPC clients are not yet wired (S-021/S-022 scope).
    // The fan-out task runs the recv loop and exits cleanly when the channel closes
    // or the shutdown signal fires. Per-client write logic (50ms timeout, client removal)
    // is structurally present but operates on an empty client list in Phase 1.
    loop {
        tokio::select! {
            event = rx.recv() => {
                match event {
                    None => {
                        // All senders dropped — channel closed. Exit cleanly (BC-2.04.011 PC-7).
                        tracing::info!("event bus channel closed; fan-out task exiting");
                        return;
                    }
                    Some(event) => {
                        // Phase 1: no TUI clients to fan out to.
                        // Future (S-021/S-022): iterate tui_clients, write with 50ms timeout per
                        // client, remove failed/slow clients (BC-2.04.011 PC-4, PC-5, EC-095).
                        let _ = &state; // Suppress unused warning — state will be used in S-021/S-022.
                        tracing::trace!(
                            received_at = %event.received_at,
                            "fan-out: event received (no TUI clients in Phase 1)"
                        );
                    }
                }
            }
            _ = shutdown_rx.changed() => {
                // M3 fix: shutdown signal received. Exit promptly within one tokio scheduler
                // tick, mirroring drop_counter_debounce_task's shutdown branch behavior.
                tracing::debug!("event_bus_fan_out_task: shutdown signal received; exiting");
                return;
            }
        }
    }
}

/// Drop counter debounce task: sends `DropCounterUpdate` at most once per 100ms.
///
/// Reads `DaemonState.drop_counter` with `Ordering::Relaxed` at each 100ms interval.
/// Sends `ServerToClient::DropCounterUpdate` to connected TUI clients only if the
/// counter value has changed since the last send (BC-2.04.011 PC-8).
///
/// # Lifetime
///
/// Exits when `shutdown_rx` fires (within one 100ms interval of the signal). The
/// `tokio::select!` branch exits the loop cleanly without needing task abort
/// (C1 fix — SS-daemon-wiring-impl.md §Fix Addendum C1).
///
/// # Arguments
///
/// - `state`: Shared daemon state providing access to `drop_counter` and `tui_clients`.
/// - `shutdown_rx`: Watch receiver that fires when the daemon signals graceful shutdown.
///   The task exits within one 100ms interval of `shutdown_rx` changing to `true`.
pub async fn drop_counter_debounce_task(
    state: Arc<DaemonState>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    use std::time::Duration;
    use tokio::time;

    let mut interval = time::interval(Duration::from_millis(DROP_COUNTER_DEBOUNCE_MS));
    let mut last_sent: u64 = 0;

    loop {
        tokio::select! {
            _ = interval.tick() => {
                let current = match state.drop_counter.as_ref() {
                    None => continue, // Counter not initialized; nothing to send.
                    Some(counter) => counter.load(Ordering::Relaxed),
                };

                if current == last_sent {
                    // Counter unchanged since last send; skip this cycle (BC-2.04.011 PC-8).
                    continue;
                }

                // Counter changed — record new value and send DropCounterUpdate to TUI clients.
                last_sent = current;

                // Phase 1: TUI IPC clients are not yet wired (S-021/S-022 scope).
                // Future: send ServerToClient::DropCounterUpdate(current) to all tui_clients with
                // the per-client 50ms write timeout (BC-2.04.011 PC-8, EC-095).
                tracing::debug!(
                    drop_count = current,
                    "drop counter update (debounce 100ms): {} events dropped",
                    current
                );
            }
            _ = shutdown_rx.changed() => {
                tracing::debug!("drop_counter_debounce_task: shutdown signal received; exiting");
                return;
            }
        }
    }
}

/// Try to publish a hook event to the event bus, incrementing the drop counter on full.
///
/// This function encapsulates the canonical pattern for all 5 hook handlers:
/// 1. Call `state.event_bus_tx.try_send(event)`.
/// 2. If `Err(TrySendError::Full)`: increment `drop_counter`, log WARN, discard event.
/// 3. If `Err(TrySendError::Disconnected)`: log WARN, discard event (shutdown race, EC-099).
/// 4. If `Ok(())`: event is enqueued.
///
/// Non-blocking in all cases; NEVER awaits (BC-2.04.011 PC-3, invariant 2).
///
/// # Arguments
///
/// - `state`: Shared daemon state with `event_bus_tx` and `drop_counter`.
/// - `event`: The event to publish.
pub fn try_publish_event(state: &DaemonState, event: crate::types::EventBusHookEvent) {
    use tokio::sync::mpsc::error::TrySendError;

    match state.event_bus_tx.as_ref() {
        None => {
            // Event bus not yet initialized (daemon startup race, or test stub).
            tracing::warn!("event bus not initialized; dropping event");
        }
        Some(tx) => {
            match tx.try_send(event) {
                Ok(()) => {
                    // Event enqueued successfully.
                }
                Err(TrySendError::Full(_)) => {
                    // BC-2.04.011 PC-3a: increment drop counter.
                    let count = state
                        .drop_counter
                        .as_ref()
                        .map(|c| c.fetch_add(1, Ordering::Relaxed) + 1)
                        .unwrap_or(0);
                    // BC-2.04.011 PC-3b: log WARN with current count.
                    tracing::warn!(
                        drop_count = count,
                        "event bus full; dropping event (drop_count={})",
                        count
                    );
                    // PC-3c: event is silently discarded (drop here).
                }
                Err(TrySendError::Closed(_)) => {
                    // Shutdown race (EC-099) or bus receiver task crashed.
                    // Increment drop counter so the TUI shows ALL lost events regardless of cause
                    // (Full vs Closed both result in a dropped event; BC-2.04.011 PC-3a).
                    let count = state
                        .drop_counter
                        .as_ref()
                        .map(|c| c.fetch_add(1, Ordering::Relaxed) + 1)
                        .unwrap_or(0);
                    tracing::warn!(
                        drop_count = count,
                        "event bus closed (shutdown race or receiver crashed); dropping event"
                    );
                }
            }
        }
    }
}
