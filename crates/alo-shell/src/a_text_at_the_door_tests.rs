#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{Read as _, Write as _};
use std::os::unix::net::UnixStream;

use alo_acting::{ToType, Typed};
use alo_applications::Application;

use super::{ReadingText, SoFar};

fn a_text(bytes: usize) -> ToType {
    ToType::Text {
        application: Application::identified("com.vscodium.codium").unwrap(),
        bytes,
    }
}

fn pair() -> (UnixStream, UnixStream) {
    let (ours, theirs) = UnixStream::pair().unwrap();
    ours.set_nonblocking(true).unwrap();
    (ours, theirs)
}

#[test]
fn a_text_that_arrived_with_its_line_is_whole_at_once() {
    let (ours, _theirs) = pair();
    match ReadingText::begun(a_text(5), ours, b"h\xc3\xa9yo".to_vec()) {
        SoFar::Whole(asked) => assert_eq!(asked.text(), "héyo"),
        _ => panic!("the text was all there"),
    }
}

#[test]
fn a_text_arriving_over_several_frames_is_read_whole() {
    let (ours, mut theirs) = pair();
    let mut reading = match ReadingText::begun(a_text(6), ours, b"ab".to_vec()) {
        SoFar::Arriving(reading) => reading,
        _ => panic!("four bytes are still to come"),
    };
    theirs.write_all(b"cd").unwrap();
    reading = match reading.looked_at() {
        SoFar::Arriving(reading) => reading,
        _ => panic!("two bytes are still to come"),
    };
    theirs.write_all(b"\nf").unwrap();
    match reading.looked_at() {
        SoFar::Whole(asked) => {
            assert_eq!(asked.text(), "abcd\nf");
            asked.answered(Typed::Typed);
            let mut said = String::new();
            theirs.read_to_string(&mut said).unwrap();
            assert_eq!(said, "typed\n");
        }
        _ => panic!("the text is all there"),
    }
}

#[test]
fn more_than_was_promised_is_dropped() {
    let (ours, _theirs) = pair();
    assert!(matches!(
        ReadingText::begun(a_text(2), ours, b"abc".to_vec()),
        SoFar::Dropped
    ));
}

#[test]
fn bytes_that_are_not_text_are_dropped() {
    let (ours, _theirs) = pair();
    assert!(matches!(
        ReadingText::begun(a_text(2), ours, vec![0xff, 0xfe]),
        SoFar::Dropped
    ));
}

#[test]
fn a_caller_that_hangs_up_part_way_is_dropped() {
    let (ours, theirs) = pair();
    drop(theirs);
    assert!(matches!(
        ReadingText::begun(a_text(4), ours, b"ab".to_vec()),
        SoFar::Dropped
    ));
}

#[test]
fn a_caller_that_stops_sending_is_dropped_once_its_patience_runs_out() {
    let (ours, _theirs) = pair();
    let mut so_far = ReadingText::begun(a_text(4), ours, b"ab".to_vec());
    let mut frames = 0;
    while let SoFar::Arriving(reading) = so_far {
        frames += 1;
        assert!(frames <= 100, "never dropped");
        so_far = reading.looked_at();
    }
    assert!(matches!(so_far, SoFar::Dropped));
}

#[test]
fn a_save_has_no_text_and_anything_after_it_is_dropped() {
    let save = || ToType::Save {
        application: Application::identified("com.vscodium.codium").unwrap(),
    };
    let (ours, _theirs) = pair();
    assert!(matches!(
        ReadingText::begun(save(), ours, Vec::new()),
        SoFar::Whole(_)
    ));
    let (ours, _theirs) = pair();
    assert!(matches!(
        ReadingText::begun(save(), ours, b"x".to_vec()),
        SoFar::Dropped
    ));
}
