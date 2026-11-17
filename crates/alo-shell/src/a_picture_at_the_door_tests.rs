use std::io::Read as _;
use std::os::unix::net::UnixStream;

use super::*;
use alo_applications::Application;

/// A connection as the door holds one — not blocking — and the daemon's end.
fn a_connection() -> (UnixStream, UnixStream) {
    let (door, daemon) = UnixStream::pair().unwrap();
    door.set_nonblocking(true).unwrap();
    (door, daemon)
}

fn asked() -> ToPicture {
    ToPicture {
        application: Application::identified("net.supertuxkart.SuperTuxKart").unwrap(),
    }
}

/// Send on until done or gone, reading the daemon's end between, and answer
/// everything the daemon read.
fn sent_whole(mut sending: SendingAPicture, daemon: &mut UnixStream) -> Vec<u8> {
    daemon.set_nonblocking(true).unwrap();
    let mut read = Vec::new();
    let mut buffer = vec![0_u8; 64 * 1024];
    for _ in 0..100_000 {
        let next = sending.sent_on();
        loop {
            match daemon.read(&mut buffer) {
                Ok(0) | Err(_) => break,
                Ok(count) => read.extend_from_slice(&buffer[..count]),
            }
        }
        match next {
            Some(more) => sending = more,
            None => break,
        }
    }
    loop {
        match daemon.read(&mut buffer) {
            Ok(0) | Err(_) => break,
            Ok(count) => read.extend_from_slice(&buffer[..count]),
        }
    }
    read
}

/// **A picture larger than any socket holds arrives whole**: the head line,
/// then exactly the PNG's bytes, sent on across as many turns as it takes and
/// never waiting.
#[test]
fn a_picture_larger_than_the_socket_arrives_whole() {
    let (door, mut daemon) = a_connection();
    let png: Vec<u8> = (0..4 * 1024 * 1024_u32)
        .map(|at| u8::try_from(at % 251).unwrap())
        .collect();
    let answer = Pictured::Picture {
        bytes: png.len(),
        wide: 1280,
        tall: 800,
    };
    let sending = AskedToPicture::of(asked(), door).answered(answer, png.clone());
    let read = sent_whole(sending, &mut daemon);
    let head = format!("{}\n", answer.written());
    assert_eq!(&read[..head.len()], head.as_bytes());
    assert!(read[head.len()..] == png[..], "the picture did not arrive whole");
}

/// **A refusal is its one line and nothing after it**, whatever bytes were
/// handed with it.
#[test]
fn a_refusal_is_one_line_and_nothing_else() {
    let (door, mut daemon) = a_connection();
    let sending =
        AskedToPicture::of(asked(), door).answered(Pictured::NotInFront, b"not sent".to_vec());
    assert_eq!(sent_whole(sending, &mut daemon), b"not-in-front\n");
}

/// **A daemon that has gone ends the sending**, rather than leaving it to be
/// tried for ever.
#[test]
fn a_daemon_that_has_gone_ends_the_sending() {
    let (door, daemon) = a_connection();
    drop(daemon);
    let png = vec![7_u8; 1024 * 1024];
    let answer = Pictured::Picture {
        bytes: png.len(),
        wide: 10,
        tall: 10,
    };
    let mut sending = Some(AskedToPicture::of(asked(), door).answered(answer, png));
    let mut turns = 0;
    while let Some(now) = sending {
        sending = now.sent_on();
        turns += 1;
        assert!(turns < 10, "a gone daemon was written to for ever");
    }
}
