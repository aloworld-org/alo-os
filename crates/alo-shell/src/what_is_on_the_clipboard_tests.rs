//! A pipe written into, and the characters that come out the other side.
//!
//! **A real pipe, not a stand-in.** What this file is about is reading
//! something another process writes at a time of its own choosing, so a test
//! that handed over a `Vec` would prove the filtering and nothing about the
//! reading.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::Write;
use std::os::unix::io::OwnedFd;

use super::{AT_MOST, TheClipboard};

/// A pipe, as the two halves a client and a compositor would hold.
fn a_pipe() -> (OwnedFd, OwnedFd) {
    let (reading, writing) = std::io::pipe().expect("a pipe");
    (OwnedFd::from(reading), OwnedFd::from(writing))
}

/// Wait for one paste, or give up.
///
/// **Bounded, because a test that waited forever on a thread that never posts
/// is a gate that hangs rather than a test that fails.** Two seconds is far
/// longer than a pipe this size takes and far shorter than anybody's patience.
fn what_came_back(clipboard: &TheClipboard) -> Vec<super::WhatWasPasted> {
    let until = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        let came = clipboard.whatever_was_pasted();
        if !came.is_empty() || std::time::Instant::now() > until {
            return came;
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
}

#[test]
fn what_a_client_wrote_comes_back_as_characters() {
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);

    // The client writes when it gets round to it, which here is now.
    let mut writing = std::fs::File::from(writing);
    writing
        .write_all(b"sk-proj-abcdefghijklmnop")
        .expect("the client writes");
    drop(writing);

    let came = what_came_back(&clipboard);
    assert_eq!(came.len(), 1);
    assert_eq!(
        came.first().expect("one paste came back").text(),
        "sk-proj-abcdefghijklmnop"
    );
}

#[test]
fn nothing_has_been_pasted_until_something_is() {
    // **The answer a frame loop gets almost every time it asks**, and it must
    // not block: this is called once per frame, sixty times a second.
    let clipboard = TheClipboard::new();
    assert!(clipboard.whatever_was_pasted().is_empty());
}

#[test]
fn a_key_copied_out_of_a_web_page_loses_the_newline_on_the_end() {
    // **The fault this is here for.** A key copied from a browser arrives with
    // a newline, and a field holding one looks right and does not work: the
    // provider answers that the key is wrong, and a person reading their own
    // screen can see nothing wrong with it.
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);

    let mut writing = std::fs::File::from(writing);
    writing.write_all(b"sk-a-real-key\n").expect("written");
    drop(writing);

    let came = what_came_back(&clipboard);
    assert_eq!(came.len(), 1);
    assert_eq!(
        came.first().expect("one paste came back").text(),
        "sk-a-real-key",
        "the newline is gone, and the key is otherwise untouched"
    );
}

#[test]
fn a_tab_and_a_carriage_return_go_the_same_way() {
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);

    let mut writing = std::fs::File::from(writing);
    writing.write_all(b"one\ttwo\r\n").expect("written");
    drop(writing);

    let came = what_came_back(&clipboard);
    assert_eq!(came.first().expect("one paste came back").text(), "onetwo");
}

#[test]
fn letters_that_are_not_ascii_arrive_whole() {
    // A provider somebody called "Fürs Büro", or a key kept in a note beside
    // Cyrillic. `text/plain` alone is read as ASCII by some clients, which is
    // why the mime type asked for names UTF-8.
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);

    let mut writing = std::fs::File::from(writing);
    writing
        .write_all("Fürs Büro — Привет — नमस्ते".as_bytes())
        .expect("written");
    drop(writing);

    let came = what_came_back(&clipboard);
    assert_eq!(
        came.first().expect("one paste came back").text(),
        "Fürs Büro — Привет — नमस्ते"
    );
}

#[test]
fn bytes_that_are_not_text_are_dropped_rather_than_repaired() {
    // Mending a credential is silently changing one, which is what
    // `alo_models::Secret` refuses for the same reason. A person pasting an
    // image gets nothing, and can see that nothing happened.
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);

    let mut writing = std::fs::File::from(writing);
    writing
        .write_all(&[0xff, 0xfe, 0x00, 0x01])
        .expect("written");
    drop(writing);

    // Waited for properly: a drop and a slow client look the same for two
    // seconds, and this asserts the first rather than catching the second.
    assert!(
        what_came_back(&clipboard).is_empty(),
        "nothing that is not text reaches a field"
    );
}

#[test]
fn a_client_that_writes_without_stopping_is_cut_off() {
    // **Bounded as it reads, not after.** A client may write as much as it
    // likes down that pipe, and a compositor that read it all before measuring
    // would be holding whatever it sent.
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);

    std::thread::spawn(move || {
        let mut writing = std::fs::File::from(writing);
        // More than the limit, in chunks, the way a stream arrives.
        let chunk = vec![b'x'; 8 * 1024];
        for _ in 0..32 {
            if writing.write_all(&chunk).is_err() {
                return;
            }
        }
    });

    let came = what_came_back(&clipboard);
    assert_eq!(came.len(), 1);
    assert_eq!(
        came.first().expect("one paste came back").text().len(),
        AT_MOST,
        "it stopped at the limit rather than taking everything offered"
    );
}

#[test]
fn a_client_that_writes_nothing_posts_nothing() {
    // An empty selection is not a paste. A field that took one would move the
    // caret and change nothing, which reads as the machine ignoring a press.
    let clipboard = TheClipboard::new();
    let (reading, writing) = a_pipe();
    clipboard.reading(reading);
    drop(writing);

    assert!(what_came_back(&clipboard).is_empty());
}

#[test]
fn two_pastes_both_come_back_in_the_order_they_were_read() {
    let clipboard = TheClipboard::new();

    let (first_read, first_write) = a_pipe();
    clipboard.reading(first_read);
    let mut first_write = std::fs::File::from(first_write);
    first_write.write_all(b"one").expect("written");
    drop(first_write);
    // Waited for, so the order is the order they were read rather than a race
    // between two threads.
    let first = what_came_back(&clipboard);

    let (second_read, second_write) = a_pipe();
    clipboard.reading(second_read);
    let mut second_write = std::fs::File::from(second_write);
    second_write.write_all(b"two").expect("written");
    drop(second_write);
    let second = what_came_back(&clipboard);

    assert_eq!(first.first().expect("the first paste").text(), "one");
    assert_eq!(second.first().expect("the second paste").text(), "two");
}

#[test]
fn the_mime_type_asked_for_names_utf8() {
    // Said as a test because it is the difference between a Polish provider
    // name arriving and arriving mangled, and it is one word in a string.
    assert_eq!(TheClipboard::as_text(), "text/plain;charset=utf-8");
}
