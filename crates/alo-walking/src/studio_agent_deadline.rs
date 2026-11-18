//! Cancel a single held daemon connection at its absolute deadline.

use std::{
    net::Shutdown,
    os::unix::net::UnixStream,
    sync::mpsc::{self, Sender},
    thread::{self, JoinHandle},
    time::Instant,
};

/// A wakeable watchdog over a clone of the one existing agent connection.
pub struct DeadlineGuard {
    /// Wake the worker when the caller has finished before the deadline.
    cancel: Sender<()>,
    /// Join ownership prevents a cancelled worker from outliving its turn.
    worker: Option<JoinHandle<()>>,
}

impl DeadlineGuard {
    /// Shut down both halves at `deadline`, even if a peer trickles partial data.
    /// Dropping this guard cancels and joins the worker without closing the turn.
    ///
    /// # Errors
    /// Refuse an exhausted deadline or inability to start the watchdog thread.
    pub fn on(stream: UnixStream, deadline: Instant) -> Result<Self, String> {
        if deadline <= Instant::now() {
            return Err("the private agent deadline has expired".to_owned());
        }
        let (cancel, stopped) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("studio-probe-deadline".to_owned())
            .spawn(move || {
                if matches!(
                    stopped.recv_timeout(deadline.saturating_duration_since(Instant::now())),
                    Err(mpsc::RecvTimeoutError::Timeout)
                ) {
                    let _ = stream.shutdown(Shutdown::Both);
                }
            })
            .map_err(|_| "the private agent deadline watchdog could not start".to_owned())?;
        Ok(Self {
            cancel,
            worker: Some(worker),
        })
    }
}

impl Drop for DeadlineGuard {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}
