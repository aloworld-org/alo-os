//! Deliver one immutable frame while a slow reader starts, checking its lifetime.

use std::{io, io::Write, os::unix::net::UnixStream, time::Duration};

/// Preserve partial writes across short waits without queuing another frame.
///
/// A media engine may take seconds to start reading its compositor socket.
/// Short socket deadlines let revocation interrupt that wait; they do not mean
/// the reader has failed. The caller retains only this frame and its newest slot.
pub(super) fn write_frame(
    stream: &mut UnixStream,
    mut remaining: &[u8],
    mut sharing: impl FnMut() -> bool,
) -> io::Result<()> {
    stream.set_write_timeout(Some(Duration::from_millis(50)))?;
    while !remaining.is_empty() {
        if !sharing() {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "screen sharing ended during frame delivery",
            ));
        }
        match stream.write(remaining) {
            Ok(0) => return Err(io::Error::from(io::ErrorKind::WriteZero)),
            Ok(written) => {
                remaining = remaining
                    .get(written..)
                    .ok_or_else(|| io::Error::other("invalid frame write length"))?;
            }
            Err(why)
                if matches!(
                    why.kind(),
                    io::ErrorKind::TimedOut
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::Interrupted
                ) => {}
            Err(why) => return Err(why),
        }
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "unexpected transport failures fail the real socket checks"
)]
mod tests {
    use super::*;
    use std::{
        io::Read,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
            mpsc,
        },
        thread,
    };

    /// A first-use engine delay must neither truncate nor duplicate a raw frame.
    #[test]
    fn a_reader_starting_after_one_second_receives_the_complete_frame() {
        let (mut writer, mut reader) = UnixStream::pair().unwrap();
        rustix::net::sockopt::set_socket_send_buffer_size(&writer, 4096).unwrap();
        reader
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        let pixels: Vec<_> = (0..1024 * 1024).map(|at| (at % 251) as u8).collect();
        let expected = pixels.clone();
        let sending = thread::spawn(move || write_frame(&mut writer, &pixels, || true).unwrap());
        thread::sleep(Duration::from_millis(1200));
        let mut received = Vec::new();
        reader.read_to_end(&mut received).unwrap();
        sending.join().unwrap();
        assert_eq!(received, expected);
    }

    /// A blocked socket must not hold capture open after its source is revoked.
    #[test]
    fn revocation_interrupts_a_reader_that_never_drains_its_socket() {
        let (mut writer, _reader) = UnixStream::pair().unwrap();
        rustix::net::sockopt::set_socket_send_buffer_size(&writer, 4096).unwrap();
        let live = Arc::new(AtomicBool::new(true));
        let sharing = live.clone();
        let (sent, received) = mpsc::channel();
        let sending = thread::spawn(move || {
            let result = write_frame(&mut writer, &vec![0; 1024 * 1024], || {
                sharing.load(Ordering::Acquire)
            });
            sent.send(result.unwrap_err().kind()).unwrap();
        });
        thread::sleep(Duration::from_millis(100));
        live.store(false, Ordering::Release);
        assert_eq!(
            received.recv_timeout(Duration::from_secs(1)).unwrap(),
            io::ErrorKind::Interrupted
        );
        sending.join().unwrap();
    }
}
