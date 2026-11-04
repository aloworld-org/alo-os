//! Bounded setup replies remain interruptible while the native engine starts.

use std::{
    io::{self, Read},
    os::unix::net::UnixStream,
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

/// Keep partial headers across short reads without consuming a following frame.
pub(super) fn line(
    stream: &mut UnixStream,
    cancelled: &AtomicBool,
    limit: Duration,
) -> io::Result<Vec<u8>> {
    stream.set_read_timeout(Some(Duration::from_millis(50)))?;
    let until = Instant::now() + limit;
    let mut bytes = Vec::with_capacity(128);
    while bytes.len() < alo_capture_protocol::MAX_MESSAGE_BYTES {
        check(cancelled)?;
        if Instant::now() >= until {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "capture setup timed out",
            ));
        }
        let mut byte = [0];
        match stream.read_exact(&mut byte) {
            Ok(()) => {
                bytes.extend(byte);
                if byte == *b"\n" {
                    return Ok(bytes);
                }
            }
            Err(why)
                if matches!(
                    why.kind(),
                    io::ErrorKind::TimedOut | io::ErrorKind::WouldBlock
                ) => {}
            Err(why) => return Err(why),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "capture reply exceeds the message bound",
    ))
}

/// Check at each setup boundary before asking for frames or launching a publisher.
pub(super) fn check(cancelled: &AtomicBool) -> io::Result<()> {
    if cancelled.load(Ordering::Acquire) {
        Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "capture setup was cancelled",
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "unexpected transport results fail the cancellation test"
)]
mod tests {
    use super::*;
    use std::{io::Write, sync::Arc};

    /// Closing a request interrupts even a native worker's unfinished header.
    #[test]
    fn cancelled_setup_does_not_wait_for_a_complete_header() {
        let (mut read, mut write) = UnixStream::pair().unwrap();
        write.write_all(b"{").unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let stop = cancelled.clone();
        let (sent, received) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            sent.send(
                line(&mut read, &stop, Duration::from_secs(10))
                    .unwrap_err()
                    .kind(),
            )
            .unwrap();
        });
        cancelled.store(true, Ordering::Release);
        assert_eq!(
            received.recv_timeout(Duration::from_secs(1)).unwrap(),
            io::ErrorKind::Interrupted
        );
        worker.join().unwrap();
    }
}
