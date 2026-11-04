//! A restricted media descriptor crosses once, with no ambient descriptors.

use rustix::net::{
    RecvAncillaryBuffer, RecvAncillaryMessage, RecvFlags, ReturnFlags, SendAncillaryBuffer,
    SendAncillaryMessage, SendFlags, recvmsg, sendmsg,
};
use std::{
    io::{self, IoSlice, IoSliceMut},
    mem::MaybeUninit,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
};

/// Send exactly one descriptor after its permission barriers have completed.
///
/// # Errors
/// The transport error, an unavailable control buffer, or a short transfer.
pub fn send_descriptor(socket: &impl AsFd, descriptor: BorrowedFd<'_>) -> io::Result<()> {
    let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(1))];
    let mut control = SendAncillaryBuffer::new(&mut space);
    let descriptors = [descriptor];
    if !control.push(SendAncillaryMessage::ScmRights(&descriptors)) {
        return Err(io::ErrorKind::InvalidInput.into());
    }
    let sent = sendmsg(
        socket,
        &[IoSlice::new(&[1])],
        &mut control,
        SendFlags::NOSIGNAL,
    )?;
    if sent != 1 {
        return Err(io::ErrorKind::WriteZero.into());
    }
    Ok(())
}

/// Receive one close-on-exec descriptor, refusing missing or extra descriptors.
///
/// # Errors
/// The transport error, truncation, an invalid marker, or the wrong descriptor count.
pub fn receive_descriptor(socket: &impl AsFd) -> io::Result<OwnedFd> {
    let mut space = [MaybeUninit::uninit(); rustix::cmsg_space!(ScmRights(2))];
    let mut control = RecvAncillaryBuffer::new(&mut space);
    let mut marker = [0];
    let received = recvmsg(
        socket,
        &mut [IoSliceMut::new(&mut marker)],
        &mut control,
        RecvFlags::CMSG_CLOEXEC,
    )?;
    let mut descriptors = Vec::new();
    for message in control.drain() {
        if let RecvAncillaryMessage::ScmRights(arrived) = message {
            descriptors.extend(arrived);
        }
    }
    if received.bytes != 1
        || marker != [1]
        || received.flags.contains(ReturnFlags::CTRUNC)
        || descriptors.len() != 1
    {
        return Err(io::ErrorKind::InvalidData.into());
    }
    descriptors
        .pop()
        .ok_or_else(|| io::ErrorKind::InvalidData.into())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "an unexpected descriptor failure is the test's failure"
)]
mod tests {
    use super::*;
    use std::{
        fs::File,
        io::{Read, Write},
        os::unix::net::UnixStream,
    };
    #[test]
    fn the_transferred_descriptor_stays_live_after_the_sender_closes() {
        let (sender, receiver) = UnixStream::pair().unwrap();
        let (mut written, read) = UnixStream::pair().unwrap();
        send_descriptor(&sender, read.as_fd()).unwrap();
        drop(read);
        let mut received = File::from(receive_descriptor(&receiver).unwrap());
        written.write_all(b"frame").unwrap();
        let mut bytes = [0; 5];
        received.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"frame");
    }
    #[test]
    fn a_success_marker_without_a_descriptor_is_not_a_remote() {
        let (mut sender, receiver) = UnixStream::pair().unwrap();
        sender.write_all(&[1]).unwrap();
        assert!(receive_descriptor(&receiver).is_err());
    }
}
