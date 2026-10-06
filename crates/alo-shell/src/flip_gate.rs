//! Session-scoped authorization of page-flip completion, independent of transport.

use crate::{DisplayEvent, FlipComplete, read_display_events};
use std::{
    io,
    num::{NonZeroU32, NonZeroU64},
    os::fd::BorrowedFd,
    sync::atomic::{AtomicU64, Ordering},
};

#[cfg(test)]
#[path = "flip_gate_tests.rs"]
mod tests;

/// Process-wide identities are never reused, including after failed submissions.
static NEXT_COOKIE: AtomicU64 = AtomicU64::new(1);

/// A single pending flip's completion gate for one CRTC in one session lifetime.
///
/// This is identity bookkeeping, not a resource owner or atomic ioctl transport.
/// Keep old and submitted buffers alive until a matching completion or successful
/// synchronous disable; on disable failure quarantine them until device retirement.
/// Dropping this gate does not authorize release. Create a fresh gate after session
/// reacquisition, and never share event streams across process lifetimes.
///
/// Cookies are unique across all gates in this process, even for reused CRTC IDs.
/// Sequence and timestamps are deliberately not identities. The caller exclusively
/// owns commits and reads on the session descriptor and must supply kernel events
/// from that descriptor. This API does not authenticate caller-fabricated events.
pub struct FlipGate {
    /// The CRTC selected by the caller's current session discovery.
    crtc: NonZeroU32,
    /// Reserved before transport runs; cleared only on refusal or matched completion.
    pending: Option<NonZeroU64>,
}

impl FlipGate {
    /// Create an idle gate for a freshly discovered CRTC in an active session.
    pub fn new(crtc: NonZeroU32) -> Self {
        Self {
            crtc,
            pending: None,
        }
    }

    /// Whether a submission may still own both the old and new scanout buffers.
    pub fn is_pending(&self) -> bool {
        self.pending.is_some()
    }

    /// Submit once with a fresh opaque cookie, refusing a second pending flip.
    ///
    /// The transport must submit exactly one nonblocking atomic page-flip request
    /// with this full cookie and CRTC, returning success only after kernel acceptance.
    /// An error must mean no commit was accepted. Errors retain their original errno;
    /// no retry occurs and the failed cookie is burned. Cookie exhaustion refuses
    /// before calling transport. The reservation remains pending if transport panics.
    /// Success is acceptance, not completion or permission to release either buffer.
    pub fn submit(
        &mut self,
        transport: impl FnOnce(NonZeroU64) -> io::Result<()>,
    ) -> io::Result<NonZeroU64> {
        if self.is_pending() {
            return Err(io::Error::new(
                io::ErrorKind::WouldBlock,
                "page flip pending",
            ));
        }
        let cookie = allocate_cookie(&NEXT_COOKIE)?;
        self.pending = Some(cookie);
        if let Err(error) = transport(cookie) {
            self.pending = None;
            return Err(error);
        }
        Ok(cookie)
    }

    /// Accept one exact pending cookie/CRTC pair, authorizing old-buffer retirement.
    ///
    /// Foreign, stale, duplicate and non-flip events leave ownership unchanged.
    /// The newly displayed buffer remains owned even when this returns `Some`.
    pub fn complete(&mut self, event: DisplayEvent) -> Option<FlipComplete> {
        let DisplayEvent::FlipComplete(flip) = event else {
            return None;
        };
        if flip.crtc != self.crtc || self.pending.map(NonZeroU64::get) != Some(flip.cookie) {
            return None;
        }
        self.pending = None;
        Some(flip)
    }

    /// Read one bounded batch and accept at most the single pending completion.
    ///
    /// Uses `read_display_events` with its borrowed-fd, exclusive-reader and
    /// nonblocking requirements. The entire batch is decoded before matching:
    /// malformed data, EOF, WouldBlock and other read failures preserve pending
    /// ownership. Unrelated events are consumed, so this reader is suitable only
    /// for a session with this one flip gate and no other DRM event consumers.
    pub fn read_completion(&mut self, fd: BorrowedFd<'_>) -> io::Result<Option<FlipComplete>> {
        let events = read_display_events(fd)?;
        Ok(events.into_iter().find_map(|event| self.complete(event)))
    }
}

/// Zero is a terminal exhaustion sentinel; wrapping never restarts allocation.
///
/// **Written as its own compare-and-exchange loop rather than `fetch_update`,
/// because that method is deprecated on some toolchains and not others.** It
/// was renamed to `try_update`; a checkout that used either name would be
/// warning-free on one Rust and `-D warnings` red on the next, and this
/// repository gates with warnings denied. Measured on 2026-10-06: the hosted
/// runner refused this line while the Mac lane's own toolchain compiled it
/// silently, so no local gate on that machine could have caught it.
///
/// The loop is what `fetch_update` is. `compare_exchange_weak` may fail
/// spuriously, which is why the answer it gives back on failure is used as
/// the next attempt's starting value rather than re-read.
fn allocate_cookie(next: &AtomicU64) -> io::Result<NonZeroU64> {
    let mut value = next.load(Ordering::Relaxed);
    loop {
        // Zero is terminal: once the space is exhausted it stays exhausted,
        // and wrapping back to one would hand out a cookie still in flight.
        let Some(raised) = (value != 0).then(|| value.wrapping_add(1)) else {
            return Err(io::Error::other("page-flip cookie space exhausted"));
        };
        match next.compare_exchange_weak(value, raised, Ordering::Relaxed, Ordering::Relaxed) {
            // The cookie is the value this call claimed, not the one left
            // behind — the same value `fetch_update` answered with.
            Ok(claimed) => {
                return NonZeroU64::new(claimed)
                    .ok_or_else(|| io::Error::other("page-flip cookie space exhausted"));
            }
            Err(taken) => value = taken,
        }
    }
}
