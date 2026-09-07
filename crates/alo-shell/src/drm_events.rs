//! Bounded, cookie-preserving reads of the native DRM event ABI.

use std::{io, num::NonZeroU32, os::fd::BorrowedFd, time::Duration};

#[cfg(test)]
#[path = "drm_events_tests.rs"]
mod tests;

/// A kernel page-flip completion, not proof that a particular owned buffer retired.
///
/// The scanout owner must match both cookie and CRTC against its pending commit
/// on this same session descriptor before releasing anything. Sequence numbers
/// wrap and are not commit identities. Events from an earlier session are stale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlipComplete {
    /// The entire opaque user_data value supplied to the atomic commit.
    pub cookie: u64,
    /// Explicit kernel CRTC ID. Legacy zero-ID events are refused.
    pub crtc: NonZeroU32,
    /// Kernel vblank sequence; may wrap at u32::MAX.
    pub sequence: u32,
    /// Kernel timestamp, not wall time or a duration between frames.
    /// Its clock depends on the device's DRM_CAP_TIMESTAMP_MONOTONIC capability.
    pub timestamp: Duration,
}

/// One framed DRM event. Unhandled types never become flip completions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayEvent {
    /// A validated DRM_EVENT_FLIP_COMPLETE payload retaining its commit cookie.
    FlipComplete(FlipComplete),
    /// A structurally valid event of another type, including vblank notifications.
    /// Only its type is retained; this reader does not implement those payloads.
    Other(u32),
}

/// Read and validate one batch of at most 4096 bytes from a DRM session descriptor.
///
/// Call after readability notification, with exclusive ownership of event reads
/// and descriptor flags. The descriptor must already have O_NONBLOCK: this method
/// never changes shared file-description flags, closes the fd, waits, or retries.
/// `WouldBlock` is ordinary readiness loss; EOF is `UnexpectedEof`. Kernel errors
/// retain errno. Invalid framing, truncated flips, zero CRTC IDs and invalid
/// microseconds return `InvalidData`, discarding the entire consumed batch.
///
/// DRM reads return whole events. Larger unsupported events may cause a kernel
/// error at this fixed bound; the caller must report it, never infer completion.
/// A malformed/failed read must not release pending scanout resources. Retire
/// them by successful synchronous disable or quarantine until device retirement.
/// Successful decoding alone does not authenticate a pending commit: match the
/// cookie, CRTC and session before acting. This reader does not submit flips.
pub fn read_display_events(fd: BorrowedFd<'_>) -> io::Result<Vec<DisplayEvent>> {
    if !rustix::fs::fcntl_getfl(fd)?.contains(rustix::fs::OFlags::NONBLOCK) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "DRM event descriptor must already be nonblocking",
        ));
    }
    let mut bytes = [0; 4096];
    let count = rustix::io::read(fd, &mut bytes)?;
    if count == 0 {
        return Err(io::ErrorKind::UnexpectedEof.into());
    }
    decode(bytes.get(..count).ok_or_else(malformed)?)
}

/// Parse native-endian UAPI bytes without alignment assumptions or unsafe casts.
fn decode(mut bytes: &[u8]) -> io::Result<Vec<DisplayEvent>> {
    let mut events = Vec::new();
    while !bytes.is_empty() {
        let kind = word(bytes, 0)?;
        let length = usize::try_from(word(bytes, 4)?).map_err(|_| malformed())?;
        if length < 8 || length > bytes.len() {
            return Err(malformed());
        }
        let (event, rest) = bytes.split_at(length);
        events.push(if kind == 2 {
            // drm_event_vblank: header, u64 user_data, seconds, microseconds,
            // sequence, crtc_id. Accept a future tail, never reinterpret it.
            let cookie = u64::from_ne_bytes(
                event
                    .get(8..16)
                    .ok_or_else(malformed)?
                    .try_into()
                    .map_err(|_| malformed())?,
            );
            let seconds = word(event, 16)?;
            let micros = word(event, 20)?;
            let sequence = word(event, 24)?;
            let crtc = NonZeroU32::new(word(event, 28)?).ok_or_else(malformed)?;
            if micros >= 1_000_000 {
                return Err(malformed());
            }
            DisplayEvent::FlipComplete(FlipComplete {
                cookie,
                crtc,
                sequence,
                timestamp: Duration::new(u64::from(seconds), micros * 1000),
            })
        } else {
            DisplayEvent::Other(kind)
        });
        bytes = rest;
    }
    Ok(events)
}

/// Read one unaligned native-endian field, refusing a short event.
fn word(bytes: &[u8], offset: usize) -> io::Result<u32> {
    Ok(u32::from_ne_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or_else(malformed)?
            .try_into()
            .map_err(|_| malformed())?,
    ))
}

/// A corrupt batch is not a partial completion stream.
fn malformed() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "malformed DRM event batch")
}
