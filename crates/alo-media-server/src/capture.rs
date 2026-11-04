//! Fixed native capture workers, supervised outside the compositor and bus loops.
//!
//! Callers must identify the application, obtain a fresh source choice, judge its
//! grant, and record permission before starting. None of these operations grant
//! access or offer a model a way to execute a program.

mod endpoint;
mod workers;

pub use endpoint::sources;
pub use workers::{Stream, remote, start};

use std::{
    io,
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
};

/// Bound helper work even if a native backend or private socket stops answering.
static ACTIVE: AtomicUsize = AtomicUsize::new(0);

/// A cancellable native operation polled without blocking the session bus.
pub struct Pending<T> {
    /// At most one result; a disconnected worker is reported as an error.
    receiver: Receiver<io::Result<T>>,
    /// Dropping a pending request cancels setup and tears down any late result.
    cancelled: Arc<AtomicBool>,
    /// After delivery, an owned stream controls its own lifetime.
    delivered: bool,
}

impl<T> Pending<T> {
    /// Poll once. `None` means the worker is still preparing the result.
    pub fn try_result(&mut self) -> Option<io::Result<T>> {
        match self.receiver.try_recv() {
            Ok(result) => {
                self.delivered = true;
                Some(result)
            }
            Err(mpsc::TryRecvError::Empty) => None,
            Err(mpsc::TryRecvError::Disconnected) => {
                Some(Err(io::Error::other("capture worker stopped")))
            }
        }
    }
}

impl<T> Drop for Pending<T> {
    fn drop(&mut self) {
        if !self.delivered {
            self.cancelled.store(true, Ordering::Release);
        }
    }
}

/// An admission token remains on the same supervising thread as its child.
struct Admission;
impl Drop for Admission {
    fn drop(&mut self) {
        ACTIVE.fetch_sub(1, Ordering::AcqRel);
    }
}

/// Start bounded work; the worker may return its stream before supervising it.
fn pending<T: Send + 'static>(
    run: impl FnOnce(Arc<AtomicBool>, SyncSender<io::Result<T>>) + Send + 'static,
) -> io::Result<Pending<T>> {
    ACTIVE
        .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
            (n < 16).then_some(n + 1)
        })
        .map_err(|_| io::Error::other("too many capture operations"))?;
    let admission = Admission;
    let cancelled = Arc::new(AtomicBool::new(false));
    let stopping = cancelled.clone();
    let (sender, receiver) = mpsc::sync_channel(1);
    std::thread::Builder::new()
        .name("native-capture-worker".into())
        .spawn(move || {
            let _admission = admission;
            run(stopping, sender);
        })?;
    Ok(Pending {
        receiver,
        cancelled,
        delivered: false,
    })
}
