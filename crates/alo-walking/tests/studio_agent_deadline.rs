//! An absolute whole-connection deadline, including silent or trickled input.
//!
//! These local Unix streams never connect to an agent daemon. The watchdog
//! must shut down its cloned handle of this same stream, and Drop must cancel
//! the watchdog without shutting down a still-live connection.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    reason = "failure of local stream setup or a watchdog thread is a test failure"
)]

use std::{
    io::{Read, Write},
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

use alo_walking::studio_agent_launch::DeadlineGuard;

#[test]
fn a_silent_peer_is_cut_off_at_the_absolute_deadline_on_the_same_connection() {
    let (mut client, mut peer) = UnixStream::pair().expect("local test pair");
    client
        .set_read_timeout(Some(Duration::from_secs(3)))
        .expect("test safety bound");
    peer.set_read_timeout(Some(Duration::from_secs(3)))
        .expect("test safety bound");
    let began = Instant::now();
    let guard = DeadlineGuard::on(
        client.try_clone().expect("same connection handle"),
        began + Duration::from_millis(150),
    )
    .expect("watchdog starts");
    let mut byte = [0_u8; 1];
    assert_eq!(
        client
            .read(&mut byte)
            .expect("watchdog causes EOF, not per-read timeout"),
        0
    );
    assert!(
        began.elapsed() < Duration::from_secs(2),
        "absolute deadline must precede socket safety timeout"
    );
    assert_eq!(
        peer.read(&mut byte)
            .expect("Both shutdown ends peer's read too"),
        0
    );
    assert!(client.write_all(b"must not send after deadline").is_err());
    drop(guard);
}

#[test]
fn a_partial_reply_trickled_within_each_receive_timeout_still_ends_at_the_absolute_deadline() {
    let (mut client, mut peer) = UnixStream::pair().expect("local test pair");
    client
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("per-read timeout");
    let began = Instant::now();
    let guard = DeadlineGuard::on(
        client.try_clone().expect("same connection handle"),
        began + Duration::from_millis(500),
    )
    .expect("watchdog starts");
    let writer = std::thread::spawn(move || {
        // Never finish the protocol line. Each byte arrives sooner than the
        // receive timeout; resetting that timeout alone would keep it alive.
        for _ in 0..30 {
            if peer.write_all(b"x").is_err() {
                return;
            }
            std::thread::sleep(Duration::from_millis(40));
        }
    });
    let mut bytes = Vec::new();
    client
        .read_to_end(&mut bytes)
        .expect("watchdog must end read before a receive timeout");
    assert!(
        !bytes.is_empty(),
        "exercise partial incoming data, not just silence"
    );
    assert!(
        bytes.len() < 30,
        "trickling cannot restart the absolute budget"
    );
    assert!(
        began.elapsed() < Duration::from_secs(2),
        "deadline must not follow the last partial byte"
    );
    writer
        .join()
        .expect("test writer exits after watchdog shutdown");
    drop(guard);
}

#[test]
fn dropping_a_live_watchdog_cancels_and_joins_promptly_without_shutting_down_the_connection() {
    let (mut client, mut peer) = UnixStream::pair().expect("local test pair");
    client
        .set_read_timeout(Some(Duration::from_secs(1)))
        .expect("test safety bound");
    peer.set_read_timeout(Some(Duration::from_secs(1)))
        .expect("test safety bound");
    let guard = DeadlineGuard::on(
        client.try_clone().expect("same connection handle"),
        Instant::now() + Duration::from_secs(300),
    )
    .expect("watchdog starts");
    let began = Instant::now();
    drop(guard);
    assert!(
        began.elapsed() < Duration::from_secs(1),
        "Drop must wake and join, not wait out300 seconds"
    );
    // Also cross a cancelled deadline. Returning promptly while leaving a
    // detached watchdog to shut down the connection later is not cancellation.
    let short = DeadlineGuard::on(
        client.try_clone().expect("same connection handle"),
        Instant::now() + Duration::from_millis(100),
    )
    .expect("short watchdog starts");
    drop(short);
    std::thread::sleep(Duration::from_millis(200));
    peer.write_all(b"still live").expect("peer can still write");
    let mut received = [0_u8; 10];
    client
        .read_exact(&mut received)
        .expect("client still receives");
    assert_eq!(&received, b"still live");
    client.write_all(b"ok").expect("client can still write");
    let mut reply = [0_u8; 2];
    peer.read_exact(&mut reply).expect("peer still receives");
    assert_eq!(&reply, b"ok");
}
