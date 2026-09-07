//! Native ABI fixtures and real nonblocking descriptor integration.

use super::*;
use std::{
    fs::OpenOptions,
    io::Write,
    os::{
        fd::AsFd,
        unix::{fs::OpenOptionsExt, net::UnixStream},
    },
};

/// Build the wire layout independently of the decoder, including a 64-bit cookie.
fn flip(cookie: u64, crtc: u32) -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend(2_u32.to_ne_bytes());
    bytes.extend(32_u32.to_ne_bytes());
    bytes.extend(cookie.to_ne_bytes());
    for field in [17_u32, 999_999, u32::MAX, crtc] {
        bytes.extend(field.to_ne_bytes());
    }
    bytes
}

#[test]
fn full_cookie_crtc_sequence_and_timestamp_survive_batched_events() -> io::Result<()> {
    let mut bytes = flip(u64::MAX, 42);
    bytes.extend(1_u32.to_ne_bytes());
    bytes.extend(8_u32.to_ne_bytes());
    bytes.extend(flip(0, 77));
    let events = decode(&bytes)?;
    assert_eq!(events.len(), 3);
    assert_eq!(
        *events.first().ok_or_else(malformed)?,
        DisplayEvent::FlipComplete(FlipComplete {
            cookie: u64::MAX,
            crtc: NonZeroU32::MIN.saturating_add(41),
            sequence: u32::MAX,
            timestamp: Duration::new(17, 999_999_000),
        })
    );
    assert_eq!(
        *events.get(1).ok_or_else(malformed)?,
        DisplayEvent::Other(1)
    );
    assert!(
        matches!(*events.get(2).ok_or_else(malformed)?, DisplayEvent::FlipComplete(FlipComplete {cookie: 0, crtc, ..}) if crtc.get() == 77)
    );
    Ok(())
}

#[test]
fn unknown_and_extended_events_preserve_framing_without_alignment_assumptions() -> io::Result<()> {
    let mut bytes = Vec::new();
    bytes.extend(999_u32.to_ne_bytes());
    bytes.extend(9_u32.to_ne_bytes());
    bytes.push(0xff);
    let mut extended = flip(1 << 48, 3);
    extended
        .get_mut(4..8)
        .ok_or_else(malformed)?
        .copy_from_slice(&36_u32.to_ne_bytes());
    extended.extend([0xa5; 4]);
    bytes.extend(extended);
    let events = decode(&bytes)?;
    assert_eq!(events.len(), 2);
    assert_eq!(
        *events.first().ok_or_else(malformed)?,
        DisplayEvent::Other(999)
    );
    assert!(
        matches!(*events.get(1).ok_or_else(malformed)?, DisplayEvent::FlipComplete(FlipComplete {cookie, ..}) if cookie == 1 << 48)
    );
    Ok(())
}

#[test]
fn every_truncated_flip_and_invalid_length_refuses_the_entire_batch() -> io::Result<()> {
    let valid = flip(9, 2);
    for length in 1..32 {
        let mut short = valid.get(..length).ok_or_else(malformed)?.to_vec();
        if length >= 8 {
            short
                .get_mut(4..8)
                .ok_or_else(malformed)?
                .copy_from_slice(&(length as u32).to_ne_bytes());
        }
        let mut batch = valid.clone();
        batch.extend(short);
        assert_eq!(
            decode(&batch).err().ok_or_else(malformed)?.kind(),
            io::ErrorKind::InvalidData
        );
    }
    for length in [0_u32, 1, 7, 33, u32::MAX] {
        let mut batch = valid.clone();
        batch
            .get_mut(4..8)
            .ok_or_else(malformed)?
            .copy_from_slice(&length.to_ne_bytes());
        assert_eq!(
            decode(&batch).err().ok_or_else(malformed)?.kind(),
            io::ErrorKind::InvalidData
        );
    }
    Ok(())
}

#[test]
fn invalid_timestamp_and_legacy_zero_crtc_are_not_completions() -> io::Result<()> {
    for (offset, value) in [(20, 1_000_000_u32), (20, u32::MAX), (28, 0)] {
        let mut bytes = flip(9, 2);
        bytes
            .get_mut(offset..offset + 4)
            .ok_or_else(malformed)?
            .copy_from_slice(&value.to_ne_bytes());
        assert_eq!(
            decode(&bytes).err().ok_or_else(malformed)?.kind(),
            io::ErrorKind::InvalidData
        );
    }
    Ok(())
}

#[test]
fn real_descriptor_reads_events_and_preserves_wouldblock_eof_and_flags() -> io::Result<()> {
    let (reader, mut writer) = UnixStream::pair()?;
    reader.set_nonblocking(true)?;
    let flags = rustix::fs::fcntl_getfl(&reader)?;
    assert_eq!(
        read_display_events(reader.as_fd())
            .err()
            .ok_or_else(malformed)?
            .kind(),
        io::ErrorKind::WouldBlock
    );
    writer.write_all(&flip(u64::MAX - 1, 11))?;
    let events = read_display_events(reader.as_fd())?;
    assert!(
        matches!(events.as_slice(), [DisplayEvent::FlipComplete(FlipComplete {cookie, crtc, ..})] if *cookie == u64::MAX - 1 && crtc.get() == 11)
    );
    assert_eq!(rustix::fs::fcntl_getfl(&reader)?, flags);
    drop(writer);
    assert_eq!(
        read_display_events(reader.as_fd())
            .err()
            .ok_or_else(malformed)?
            .kind(),
        io::ErrorKind::UnexpectedEof
    );
    assert_eq!(rustix::fs::fcntl_getfl(&reader)?, flags);
    Ok(())
}

#[test]
fn blocking_descriptor_refusal_does_not_read_or_change_flags() -> io::Result<()> {
    let (reader, mut writer) = UnixStream::pair()?;
    writer.write_all(&flip(27, 9))?;
    let flags = rustix::fs::fcntl_getfl(&reader)?;
    assert_eq!(
        read_display_events(reader.as_fd())
            .err()
            .ok_or_else(malformed)?
            .kind(),
        io::ErrorKind::InvalidInput
    );
    assert_eq!(rustix::fs::fcntl_getfl(&reader)?, flags);
    reader.set_nonblocking(true)?;
    assert_eq!(read_display_events(reader.as_fd())?.len(), 1);
    Ok(())
}

#[test]
fn real_malformed_batch_is_consumed_without_delivering_partial_completion() -> io::Result<()> {
    let (reader, mut writer) = UnixStream::pair()?;
    reader.set_nonblocking(true)?;
    let mut bytes = flip(2, 3);
    bytes.extend([0; 8]);
    writer.write_all(&bytes)?;
    assert_eq!(
        read_display_events(reader.as_fd())
            .err()
            .ok_or_else(malformed)?
            .kind(),
        io::ErrorKind::InvalidData
    );
    assert_eq!(
        read_display_events(reader.as_fd())
            .err()
            .ok_or_else(malformed)?
            .kind(),
        io::ErrorKind::WouldBlock
    );
    Ok(())
}

#[test]
fn real_kernel_read_error_preserves_errno_and_caller_fd() -> io::Result<()> {
    let fd = OpenOptions::new()
        .write(true)
        .custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32)
        .open("/dev/null")?;
    let flags = rustix::fs::fcntl_getfl(&fd)?;
    let error = read_display_events(fd.as_fd())
        .err()
        .ok_or_else(malformed)?;
    assert_eq!(
        error.raw_os_error(),
        Some(rustix::io::Errno::BADF.raw_os_error())
    );
    assert_eq!(rustix::fs::fcntl_getfl(&fd)?, flags);
    Ok(())
}

#[test]
fn wire_layout_matches_pinned_kernel_uapi() {
    type Wire = drm_ffi::drm_event_vblank;
    assert_eq!(std::mem::size_of::<Wire>(), 32);
    assert_eq!(std::mem::offset_of!(Wire, user_data), 8);
    assert_eq!(std::mem::offset_of!(Wire, tv_sec), 16);
    assert_eq!(std::mem::offset_of!(Wire, tv_usec), 20);
    assert_eq!(std::mem::offset_of!(Wire, sequence), 24);
    assert_eq!(std::mem::offset_of!(Wire, crtc_id), 28);
}

#[test]
fn real_descriptor_accepts_a_full_bounded_batch() -> io::Result<()> {
    let (reader, mut writer) = UnixStream::pair()?;
    reader.set_nonblocking(true)?;
    let mut bytes = Vec::new();
    for cookie in 0..128 {
        bytes.extend(flip(cookie, 11));
    }
    assert_eq!(bytes.len(), 4096);
    writer.write_all(&bytes)?;
    let events = read_display_events(reader.as_fd())?;
    assert_eq!(events.len(), 128);
    for (cookie, event) in events.iter().enumerate() {
        assert!(matches!(event, DisplayEvent::FlipComplete(flip) if flip.cookie == cookie as u64));
    }
    assert_eq!(
        read_display_events(reader.as_fd())
            .err()
            .ok_or_else(malformed)?
            .kind(),
        io::ErrorKind::WouldBlock
    );
    Ok(())
}
