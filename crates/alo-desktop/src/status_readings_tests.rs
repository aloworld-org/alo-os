//! A slow or unavailable device reader must not stop frames or change the clock.

use super::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::channel,
};

/// A measured speaker level, separate from the locally formatted time.
fn measured(clock: &str, volume: u16) -> Result<StatusItems, String> {
    Ok(StatusItems::shown(
        clock.into(),
        None,
        alo_networks::Reaching::reported(
            alo_networks::HowFar::AllOfIt,
            alo_networks::Metered::NotSaid,
        ),
        Some(alo_sound::Volume::of(volume).map_err(|error| error.to_string())?),
    ))
}

/// The fixed locale is irrelevant to injected hardware values.
fn formats() -> TheirFormats {
    TheirFormats::of(alo_formats::Chosen::default(), Some("en"))
}

/// Receive a completed worker without allowing the next request to start.
fn drain(
    reader: &mut StatusReadings,
    status: &mut StatusItems,
    formats: &TheirFormats,
    now: Instant,
) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(3);
    while reader.pending.is_some() {
        reader.refresh_with(now, status, formats, |_| Err("retry is unavailable".into()));
        if Instant::now() >= deadline {
            return Err("the test reader did not finish".into());
        }
        std::thread::yield_now();
    }
    Ok(())
}

#[test]
fn a_blocked_reader_leaves_refresh_and_regional_clock_changes_responsive() -> Result<(), String> {
    let now = Instant::now();
    let formats = formats();
    let mut reader = StatusReadings::at(now);
    let mut status = measured("09:41", 35)?;
    let (started, reached) = channel();
    let (release, blocked) = channel();
    let timed_out = Arc::new(AtomicBool::new(false));
    let failed = timed_out.clone();
    let calls = Arc::new(AtomicUsize::new(0));
    let invoked = calls.clone();
    reader.refresh_with(now, &mut status, &formats, move |_| {
        invoked.fetch_add(1, Ordering::SeqCst);
        started.send(()).map_err(|error| error.to_string())?;
        if blocked.recv_timeout(Duration::from_secs(2)).is_err() {
            failed.store(true, Ordering::SeqCst);
            return Err("the compositor waited on its reader".into());
        }
        measured("old region", 75)
    });
    reached
        .recv_timeout(Duration::from_secs(3))
        .map_err(|error| error.to_string())?;
    assert!(!timed_out.load(Ordering::SeqCst));
    status.set_clock("5:42 PM".into());
    reader.refresh_with(
        now + Duration::from_millis(10),
        &mut status,
        &formats,
        |_| Err("a second outstanding reader must not start".into()),
    );
    assert!(!timed_out.load(Ordering::SeqCst));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(status.clock(), "5:42 PM");
    release.send(()).map_err(|error| error.to_string())?;
    drain(
        &mut reader,
        &mut status,
        &formats,
        now + Duration::from_millis(20),
    )?;
    assert_eq!(status.clock(), "5:42 PM");
    assert_eq!(status.volume().map(alo_sound::Volume::hundredths), Some(75));
    Ok(())
}

#[test]
fn a_stalled_or_late_reading_becomes_unknown_without_starting_more_workers() -> Result<(), String> {
    let now = Instant::now();
    let formats = formats();
    let mut reader = StatusReadings::at(now);
    let mut status = measured("09:41", 35)?;
    let (release, blocked) = channel();
    reader.refresh_with(now, &mut status, &formats, move |_| {
        blocked
            .recv_timeout(Duration::from_secs(3))
            .map_err(|error| error.to_string())?;
        measured("old clock", 90)
    });
    let expired = now + STALE + Duration::from_millis(1);
    reader.refresh_with(expired, &mut status, &formats, |_| {
        Err("the outstanding query still owns the reader".into())
    });
    assert!(reader.pending.is_some());
    assert!(status.volume().is_none());
    assert_eq!(status.clock(), "09:41");
    status.set_clock("09:42".into());
    release.send(()).map_err(|error| error.to_string())?;
    drain(&mut reader, &mut status, &formats, expired)?;
    assert!(status.volume().is_none());
    assert_eq!(status.clock(), "09:42");
    Ok(())
}

#[test]
fn an_unavailable_reader_cannot_retain_a_measured_volume() -> Result<(), String> {
    let now = Instant::now();
    let formats = formats();
    let mut reader = StatusReadings::at(now);
    let mut status = measured("09:41", 35)?;
    reader.refresh_with(now, &mut status, &formats, |_| {
        Err("server unavailable".into())
    });
    drain(
        &mut reader,
        &mut status,
        &formats,
        now + Duration::from_millis(20),
    )?;
    assert!(status.volume().is_none());
    assert_eq!(status.clock(), "09:41");

    let (sender, receiver) = sync_channel(1);
    reader.pending = Some(receiver);
    drop(sender);
    reader.refresh_with(
        now + Duration::from_millis(30),
        &mut status,
        &formats,
        |_| Err("the retry interval has not elapsed".into()),
    );
    assert!(reader.pending.is_none());
    assert!(status.volume().is_none());
    assert_eq!(status.clock(), "09:41");
    Ok(())
}
