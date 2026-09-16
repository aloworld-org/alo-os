//! Two open files handed across a socket, rather than two names.
//!
//! ADR 0039 §A: the verb's thread opens the original and creates the copy,
//! inside the turn's boundary and under the grant, and **passes both
//! descriptors** to the service, which never learns a path and could not open
//! one. On Linux that is a Unix socket's `SCM_RIGHTS`, and this is the only file
//! in the crate that names `rustix` for it.
//!
//! A request line travels with its descriptors in the same message; the
//! descriptors arrive attached to its first byte. Anything more than the two a
//! convert request carries is closed on arrival and makes the request one the
//! service does not understand.

use std::io::{self, IoSlice, IoSliceMut};
use std::mem::MaybeUninit;
use std::os::fd::{BorrowedFd, OwnedFd};
use std::os::unix::net::UnixStream;

use rustix::net::{
    RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, SendAncillaryBuffer,
    SendAncillaryMessage, SendFlags, recvmsg, sendmsg,
};

/// The most descriptors a request carries.
pub const MOST_DESCRIPTORS: usize = 2;

/// Send a request line with the descriptors it carries.
///
/// # Errors
/// Whatever the machine said, or [`io::ErrorKind::InvalidInput`] for more
/// descriptors than a request carries.
pub fn send(socket: &UnixStream, line: &[u8], descriptors: &[BorrowedFd<'_>]) -> io::Result<()> {
    if descriptors.len() > MOST_DESCRIPTORS {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(MOST_DESCRIPTORS))];
    let mut control = SendAncillaryBuffer::new(&mut space);
    if !descriptors.is_empty() && !control.push(SendAncillaryMessage::ScmRights(descriptors)) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let mut sent = sendmsg(
        socket,
        &[IoSlice::new(line)],
        &mut control,
        SendFlags::NOSIGNAL,
    )?;
    // The descriptors went with the first bytes; whatever the kernel did not
    // take of the line follows on its own.
    while let Some(rest) = line.get(sent..).filter(|rest| !rest.is_empty()) {
        let mut none = SendAncillaryBuffer::default();
        sent += sendmsg(
            socket,
            &[IoSlice::new(rest)],
            &mut none,
            SendFlags::NOSIGNAL,
        )?;
    }
    Ok(())
}

/// Receive one request line, at most `longest` bytes, with every descriptor
/// that came with it.
///
/// # Errors
/// Whatever the machine said; [`io::ErrorKind::InvalidData`] for a line longer
/// than `longest` or one the other end did not finish.
pub fn receive(socket: &UnixStream, longest: usize) -> io::Result<(Vec<u8>, Vec<OwnedFd>)> {
    let mut line = Vec::new();
    let mut descriptors = Vec::new();
    let mut buffer = vec![0_u8; longest];
    loop {
        let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(MOST_DESCRIPTORS))];
        let mut control = RecvAncillaryBuffer::new(&mut space);
        let wanted = longest.saturating_sub(line.len()).max(1);
        let slice = buffer.get_mut(..wanted).ok_or(io::ErrorKind::InvalidData)?;
        let received = recvmsg(
            socket,
            &mut [IoSliceMut::new(slice)],
            &mut control,
            RecvFlags::CMSG_CLOEXEC,
        )?;
        for message in control.drain() {
            if let RecvAncillaryMessage::ScmRights(arrived) = message {
                descriptors.extend(arrived);
            }
        }
        if received.bytes == 0 {
            return Err(io::ErrorKind::InvalidData.into());
        }
        line.extend_from_slice(slice.get(..received.bytes).unwrap_or_default());
        if line.ends_with(b"\n") {
            return Ok((line, descriptors));
        }
        if line.len() >= longest || line.contains(&b'\n') {
            return Err(io::ErrorKind::InvalidData.into());
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{Read, Seek, SeekFrom, Write};
    use std::os::fd::AsFd;

    /// **A file handed across arrives as the same open file**, and what is
    /// written through one end is read through the other.
    #[test]
    fn a_file_handed_across_is_the_same_open_file() {
        let (one, other) = UnixStream::pair().unwrap();
        let at =
            std::env::temp_dir().join(format!("alo-converting-passing-{}", std::process::id()));
        let mut file = File::options()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&at)
            .unwrap();
        std::fs::remove_file(&at).unwrap();

        send(&one, b"alo-converting 1 ready\n", &[file.as_fd()]).unwrap();
        let (line, descriptors) = receive(&other, 128).unwrap();
        assert_eq!(line, b"alo-converting 1 ready\n");
        assert_eq!(descriptors.len(), 1);

        let mut arrived = File::from(descriptors.into_iter().next().unwrap());
        arrived.write_all(b"written across").unwrap();
        file.seek(SeekFrom::Start(0)).unwrap();
        let mut read = String::new();
        file.read_to_string(&mut read).unwrap();
        assert_eq!(read, "written across");
    }

    /// **A line with no end, or too long, is not a request.**
    #[test]
    fn a_line_too_long_or_unfinished_is_refused() {
        let (one, other) = UnixStream::pair().unwrap();
        send(&one, &[b'a'; 300], &[]).unwrap();
        assert!(receive(&other, 128).is_err());

        let (one, other) = UnixStream::pair().unwrap();
        send(&one, b"no end", &[]).unwrap();
        drop(one);
        assert!(receive(&other, 128).is_err());
    }
}
