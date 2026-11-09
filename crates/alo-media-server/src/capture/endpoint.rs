//! Reach the compositor that actually owns the session's Wayland socket.

use super::{Pending, pending};
use alo_capture_protocol::{
    CAPTURE_SOCKET, Request, Response, Source, frame_bytes, read_message, write_message,
};
use std::{
    io,
    os::unix::{fs::MetadataExt, net::UnixStream},
    path::{Path, PathBuf},
    time::Duration,
};

/// Read this compositor's actual catalogue on a bounded worker.
///
/// # Errors
/// Worker admission or creation failed. Socket and protocol errors are its result.
pub fn sources(wayland: PathBuf) -> io::Result<Pending<Vec<Source>>> {
    pending(move |_cancelled, sender| {
        let _ = sender.send(catalogue(&wayland));
    })
}

/// Authenticate both sockets to the same living, same-user compositor process.
pub(super) fn connect(wayland: &Path) -> io::Result<UnixStream> {
    let parent = wayland
        .parent()
        .filter(|_| wayland.is_absolute())
        .ok_or_else(|| io::Error::other("an absolute Wayland socket is required"))?;
    let about = std::fs::symlink_metadata(parent)?;
    if !about.is_dir()
        || about.uid() != rustix::process::geteuid().as_raw()
        || about.mode() & 0o077 != 0
    {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the capture directory is not private",
        ));
    }
    let display = UnixStream::connect(wayland)?;
    let compositor = rustix::net::sockopt::socket_peercred(&display)?;
    let held = rustix::process::pidfd_open(compositor.pid, rustix::process::PidfdFlags::empty())?;
    let stream = UnixStream::connect(parent.join(CAPTURE_SOCKET))?;
    let capture = rustix::net::sockopt::socket_peercred(&stream)?;
    if compositor.pid != capture.pid || capture.uid != rustix::process::geteuid() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "the capture socket belongs to another process",
        ));
    }
    let mut held = [rustix::event::PollFd::new(
        &held,
        rustix::event::PollFlags::IN,
    )];
    if rustix::event::poll(
        &mut held,
        Some(&rustix::event::Timespec {
            tv_sec: 0,
            tv_nsec: 0,
        }),
    )? != 0
    {
        return Err(io::Error::other("the compositor ended"));
    }
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    Ok(stream)
}

/// Refuse oversized catalogues and malformed sources rather than guessing caps.
fn catalogue(wayland: &Path) -> io::Result<Vec<Source>> {
    let mut stream = connect(wayland)?;
    write_message(&mut stream, &Request::Sources {})?;
    let Response::Sources { sources } = read_message(&mut stream)? else {
        return Err(io::Error::other(
            "the compositor did not provide capture sources",
        ));
    };
    if sources.len() > alo_capture_protocol::MAX_SOURCES {
        return Err(io::Error::other("the source catalogue is too large"));
    }
    let mut identities = std::collections::BTreeSet::new();
    for source in &sources {
        frame_bytes(source.width, source.height)?;
        if source.id.is_empty()
            || source.id.len() > 128
            || !identities.insert(&source.id)
            || source.name.len() > 1024
        {
            return Err(io::Error::other("invalid capture source identity"));
        }
    }
    Ok(sources)
}
