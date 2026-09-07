//! Completion identity, submission refusal and descriptor integration tests.

use super::*;
use std::{
    collections::BTreeSet, io::Write, os::fd::AsFd, os::unix::net::UnixStream, time::Duration,
};

/// Nonzero fixture CRTC without unchecked conversions.
fn crtc(value: u32) -> io::Result<NonZeroU32> {
    NonZeroU32::new(value).ok_or_else(|| io::ErrorKind::InvalidInput.into())
}

/// A synthetic completion retaining the real gate-issued identity.
fn event(cookie: u64, crtc: NonZeroU32) -> DisplayEvent {
    DisplayEvent::FlipComplete(FlipComplete {
        cookie,
        crtc,
        sequence: u32::MAX,
        timestamp: Duration::from_secs(7),
    })
}

#[test]
fn acceptance_is_not_completion_and_completion_is_exactly_once() -> io::Result<()> {
    let output = crtc(3)?;
    let mut gate = FlipGate::new(output);
    let mut calls = 0;
    let cookie = gate.submit(|cookie| {
        calls += 1;
        assert_ne!(cookie.get(), 0);
        Ok(())
    })?;
    assert_eq!(calls, 1);
    assert!(gate.is_pending());
    assert!(gate.complete(event(cookie.get(), output)).is_some());
    assert!(!gate.is_pending());
    assert!(gate.complete(event(cookie.get(), output)).is_none());
    let next = gate.submit(|_| Ok(()))?;
    assert_ne!(next, cookie);
    assert!(gate.complete(event(cookie.get(), output)).is_none());
    assert!(gate.is_pending());
    assert!(gate.complete(event(next.get(), output)).is_some());
    Ok(())
}

#[test]
fn pending_refuses_second_transport_and_foreign_events_do_not_clear_it() -> io::Result<()> {
    let output = crtc(3)?;
    let mut gate = FlipGate::new(output);
    let cookie = gate.submit(|_| Ok(()))?;
    let mut called = false;
    let result = gate.submit(|_| {
        called = true;
        Ok(())
    });
    assert_eq!(
        result.err().map(|e| e.kind()),
        Some(io::ErrorKind::WouldBlock)
    );
    assert!(!called);
    for foreign in [
        DisplayEvent::Other(1),
        event(0, output),
        event(cookie.get(), crtc(4)?),
    ] {
        assert!(gate.complete(foreign).is_none());
        assert!(gate.is_pending());
    }
    assert!(gate.complete(event(cookie.get(), output)).is_some());
    Ok(())
}

#[test]
fn failed_submission_preserves_errno_burns_cookie_and_allows_new_submission() -> io::Result<()> {
    let output = crtc(3)?;
    let mut gate = FlipGate::new(output);
    let mut refused = 0;
    let error = gate.submit(|cookie| {
        refused = cookie.get();
        Err(io::Error::from_raw_os_error(16))
    });
    assert_eq!(error.err().and_then(|e| e.raw_os_error()), Some(16));
    assert!(!gate.is_pending());
    assert!(gate.complete(event(refused, output)).is_none());
    let accepted = gate.submit(|_| Ok(()))?;
    assert_ne!(accepted.get(), refused);
    assert!(gate.complete(event(refused, output)).is_none());
    assert!(gate.is_pending());
    Ok(())
}

#[test]
fn reacquired_session_with_same_crtc_rejects_old_completion() -> io::Result<()> {
    let output = crtc(3)?;
    let old = {
        let mut previous = FlipGate::new(output);
        previous.submit(|_| Ok(()))?
    };
    let mut current = FlipGate::new(output);
    assert!(current.complete(event(old.get(), output)).is_none());
    let new = current.submit(|_| Ok(()))?;
    assert_ne!(old, new);
    assert!(current.complete(event(old.get(), output)).is_none());
    assert!(current.is_pending());
    assert!(current.complete(event(new.get(), output)).is_some());
    Ok(())
}

#[test]
fn exhaustion_never_wraps_or_reuses_the_last_cookie() -> io::Result<()> {
    let counter = AtomicU64::new(u64::MAX);
    assert_eq!(allocate_cookie(&counter)?.get(), u64::MAX);
    for _ in 0..3 {
        assert_eq!(
            allocate_cookie(&counter).err().map(|e| e.kind()),
            Some(io::ErrorKind::Other)
        );
        assert_eq!(counter.load(Ordering::Relaxed), 0);
    }
    Ok(())
}

#[test]
fn parallel_gates_have_distinct_cookies() -> io::Result<()> {
    let output = crtc(3)?;
    let workers: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(move || {
                (0..64)
                    .map(|_| FlipGate::new(output).submit(|_| Ok(())))
                    .collect::<io::Result<Vec<_>>>()
            })
        })
        .collect();
    let mut cookies = BTreeSet::new();
    for worker in workers {
        for cookie in worker
            .join()
            .map_err(|_| io::Error::other("cookie worker panicked"))??
        {
            assert!(cookies.insert(cookie));
        }
    }
    assert_eq!(cookies.len(), 512);
    Ok(())
}

/// Serialize the native kernel event layout through an actual Linux descriptor.
fn wire(cookie: u64, output: NonZeroU32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(2_u32.to_ne_bytes());
    bytes.extend(32_u32.to_ne_bytes());
    bytes.extend(cookie.to_ne_bytes());
    for word in [7, 0, u32::MAX, output.get()] {
        bytes.extend(word.to_ne_bytes());
    }
    bytes
}

#[test]
fn descriptor_batch_matches_once_after_foreign_events_and_preserves_fd() -> io::Result<()> {
    let output = crtc(3)?;
    let mut gate = FlipGate::new(output);
    let cookie = gate.submit(|_| Ok(()))?;
    let (reader, mut writer) = UnixStream::pair()?;
    reader.set_nonblocking(true)?;
    let flags = rustix::fs::fcntl_getfl(&reader)?;
    let mut bytes = wire(cookie.get(), crtc(4)?);
    bytes.extend(wire(0, output));
    bytes.extend(wire(cookie.get(), output));
    bytes.extend(wire(cookie.get(), output));
    writer.write_all(&bytes)?;
    assert_eq!(
        gate.read_completion(reader.as_fd())?.map(|e| e.cookie),
        Some(cookie.get())
    );
    assert!(!gate.is_pending());
    assert_eq!(
        gate.read_completion(reader.as_fd()).err().map(|e| e.kind()),
        Some(io::ErrorKind::WouldBlock)
    );
    assert_eq!(rustix::fs::fcntl_getfl(&reader)?, flags);
    Ok(())
}

#[test]
fn malformed_batch_and_read_refusals_never_authorize_retirement() -> io::Result<()> {
    let output = crtc(3)?;
    let mut gate = FlipGate::new(output);
    let cookie = gate.submit(|_| Ok(()))?;
    let (reader, mut writer) = UnixStream::pair()?;
    assert_eq!(
        gate.read_completion(reader.as_fd()).err().map(|e| e.kind()),
        Some(io::ErrorKind::InvalidInput)
    );
    assert!(gate.is_pending());
    reader.set_nonblocking(true)?;
    assert_eq!(
        gate.read_completion(reader.as_fd()).err().map(|e| e.kind()),
        Some(io::ErrorKind::WouldBlock)
    );
    assert!(gate.is_pending());
    let mut bytes = wire(cookie.get(), output);
    bytes.extend([0; 8]);
    writer.write_all(&bytes)?;
    assert_eq!(
        gate.read_completion(reader.as_fd()).err().map(|e| e.kind()),
        Some(io::ErrorKind::InvalidData)
    );
    assert!(gate.is_pending());
    drop(writer);
    assert_eq!(
        gate.read_completion(reader.as_fd()).err().map(|e| e.kind()),
        Some(io::ErrorKind::UnexpectedEof)
    );
    assert!(gate.is_pending());
    Ok(())
}
