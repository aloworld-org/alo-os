//! Bound the one-shot probe's initial connection without changing the daemon.

use std::{sync::mpsc, thread, time::Instant};

/// Attempt one operation within the probe's existing absolute deadline.
///
/// Only for a disposable process that exits immediately on error. A timed-out
/// blocking connect cannot be cancelled in Rust; process exit tears down that
/// worker. This never retries or joins a still-blocked worker. It must not be
/// used in a long-lived service. The private binary supplies one fixed socket;
/// the operation parameter permits independent local tests without a daemon.
///
/// # Errors
/// Refuse an expired deadline, worker failure, connection failure or timeout.
pub fn before_deadline<T: Send + 'static>(
    deadline: Instant,
    operation: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    if deadline <= Instant::now() {
        return Err("the connection deadline has expired".to_owned());
    }
    let (sender, receiver) = mpsc::sync_channel(1);
    let worker = thread::Builder::new()
        .name("studio-probe-connect".to_owned())
        .spawn(move || {
            let _ = sender.send(operation());
        })
        .map_err(|_| "the connection worker could not start".to_owned())?;
    let reply = receiver
        .recv_timeout(deadline.saturating_duration_since(Instant::now()))
        .map_err(|_| "the connection did not finish before its deadline".to_owned())?;
    if Instant::now() >= deadline {
        return Err("the connection deadline has expired".to_owned());
    }
    worker
        .join()
        .map_err(|_| "the connection worker failed".to_owned())?;
    reply
}
