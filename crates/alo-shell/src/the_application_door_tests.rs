//! The application door, and what it does with everyone who is not the daemon
//! writing one good line.
//!
//! **The refusals are the subject.** ADR 0108 puts no token on the wire, so
//! what stands between the agent and this door is one question — *is the
//! kernel's account of the caller the person?* — and what stands between a rude
//! caller and a frozen compositor is that nothing here waits. Both are tested
//! by making the machine really be in that state.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{ErrorKind, Read as _, Write as _};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use alo_acting::{Acting, Answer, NotActed};
use alo_applications::Application;

use crate::the_application_door::{
    AS_MANY_AS_WAIT, CarriesOutAnApplicationVerb, PATIENCE, TheApplicationDoor,
};
use crate::the_application_doors_place::ThePlace;

/// The user this test is really running as.
fn us() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// A place of this test's own with the person's directory already made.
fn a_place(what: &str) -> (PathBuf, ThePlace) {
    let root = std::env::temp_dir().join(format!("alo-doorway-{}-{what}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let place = ThePlace::beneath(&root, us());
    std::fs::create_dir_all(place.directory()).unwrap();
    (root, place)
}

/// Everything asked of it, and whatever it was told to answer.
struct AHand {
    /// Every act it was handed, in order.
    carried: Vec<Acting>,
    /// What to answer each time.
    answering: Answer,
}

impl AHand {
    fn answering(answering: Answer) -> Self {
        Self {
            carried: Vec::new(),
            answering,
        }
    }
}

impl CarriesOutAnApplicationVerb for AHand {
    fn carrying_out(&mut self, acting: &Acting) -> Answer {
        self.carried.push(acting.clone());
        self.answering.clone()
    }
}

/// The mail client, as an identifier.
fn mail() -> Application {
    Application::identified("org.gnome.Geary").unwrap()
}

/// Read one line, with a bound so a test cannot hang on a quiet socket.
fn one_line(talking: &mut UnixStream) -> Option<String> {
    talking
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let mut heard = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        match talking.read(&mut byte) {
            Ok(0) => return None,
            Ok(_) if byte[0] == b'\n' => return Some(String::from_utf8(heard).unwrap()),
            Ok(_) => heard.push(byte[0]),
            Err(_) => return None,
        }
    }
}

/// **One line in, one line out, and the act really reached the hand.**
#[test]
fn a_request_is_carried_out_and_answered() {
    let (root, place) = a_place("answered");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    let mut daemon = UnixStream::connect(place.socket()).unwrap();
    daemon.write_all(b"open org.gnome.Geary\n").unwrap();

    assert_eq!(door.answered(&mut hand), 1);
    assert_eq!(hand.carried, vec![Acting::Open(mail())]);
    assert_eq!(one_line(&mut daemon).as_deref(), Some("acted"));

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **A refusal crosses as its own word**, and the compositor never invents a
/// sentence: what a person reads was decided in their language long before.
#[test]
fn a_refusal_crosses_as_its_word() {
    let (root, place) = a_place("refused");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::NotActed(NotActed::NoWindow));

    let mut daemon = UnixStream::connect(place.socket()).unwrap();
    daemon.write_all(b"focus org.gnome.Geary\n").unwrap();

    // Nothing was *acted*, which is what the count means.
    assert_eq!(door.answered(&mut hand), 0);
    assert_eq!(hand.carried, vec![Acting::Focus(mail())]);
    assert_eq!(one_line(&mut daemon).as_deref(), Some("no-window"));

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **A stranger is closed with nothing written on it, and before a byte of
/// theirs is read.**
///
/// This is the one thing standing between the agent and this door. The line
/// the stranger wrote is good — so a door that answered it would act — and the
/// assertion is that nothing came back, nothing reached the hand, and the
/// connection is gone.
#[test]
fn a_stranger_is_closed_with_nothing_written_and_nothing_read() {
    let (root, place) = a_place("stranger");
    let listening = place.prepared(us()).unwrap();
    // The door admits somebody who is not us; we are therefore the stranger.
    let somebody_else = us().wrapping_add(1);
    let mut door = TheApplicationDoor::admitting(place.clone(), listening, somebody_else);
    let mut hand = AHand::answering(Answer::Acted);

    let mut stranger = UnixStream::connect(place.socket()).unwrap();
    stranger.write_all(b"close org.gnome.Geary\n").unwrap();

    assert_eq!(door.answered(&mut hand), 0);
    assert!(
        hand.carried.is_empty(),
        "a stranger's line reached the machine: {:?}",
        hand.carried
    );
    assert_eq!(
        one_line(&mut stranger),
        None,
        "something was written to a stranger, which tells them alo OS is here"
    );
    assert_eq!(
        door.how_many_are_waiting(),
        0,
        "a stranger is still being held"
    );

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **A line that is not a request gets no answer at all**, and the connection
/// goes. The three things `NotRead` tells apart are for whoever is fixing a
/// caller; this wire has one word out, and `could-not-ask` would blame the
/// machine for a caller's bug.
#[test]
fn a_line_that_is_not_a_request_is_answered_with_nothing() {
    let (root, place) = a_place("not-a-request");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    for rubbish in [
        "open\n",
        "open org.gnome.Geary and.another\n",
        "arrange org.gnome.Geary\n",
        "OPEN org.gnome.Geary\n",
        "open \n",
        "\n",
    ] {
        let mut caller = UnixStream::connect(place.socket()).unwrap();
        caller.write_all(rubbish.as_bytes()).unwrap();
        assert_eq!(door.answered(&mut hand), 0, "{rubbish:?} was acted on");
        assert_eq!(one_line(&mut caller), None, "{rubbish:?} was answered");
    }
    assert!(
        hand.carried.is_empty(),
        "something that is not a request reached the machine: {:?}",
        hand.carried
    );

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **`arrange` is not on this wire**, and that is the fourth verb's absence
/// rather than an oversight: it takes a *where*, and where a window goes is
/// `alo-arranging`'s and the design's. Said as a test so the absence is
/// checkable.
#[test]
fn the_fourth_verb_is_not_on_this_wire() {
    let (root, place) = a_place("no-arrange");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    let mut caller = UnixStream::connect(place.socket()).unwrap();
    caller.write_all(b"arrange org.gnome.Geary\n").unwrap();

    assert_eq!(door.answered(&mut hand), 0);
    assert!(hand.carried.is_empty());
    assert_eq!(one_line(&mut caller), None);

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **A caller cannot make the compositor hold a line while it writes one.**
/// More than a request may be is finished with rather than truncated: a
/// truncated line could be a different, valid request.
#[test]
fn a_caller_that_says_more_than_a_request_may_be_is_finished_with() {
    let (root, place) = a_place("too-long");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    let mut caller = UnixStream::connect(place.socket()).unwrap();
    // A good request, then enough rubbish to go past the bound before any
    // newline arrives.
    let mut saying = b"open org.gnome.Geary".to_vec();
    saying.extend(std::iter::repeat_n(b'x', alo_acting::Request::THE_MOST));
    saying.push(b'\n');
    caller.write_all(&saying).unwrap();

    assert_eq!(door.answered(&mut hand), 0);
    assert!(
        hand.carried.is_empty(),
        "a line over the bound was acted on"
    );
    assert_eq!(one_line(&mut caller), None);

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **Half a line is read again next visit**, which is what makes nothing here
/// wait. A caller writing a request in two pieces is served, not dropped.
#[test]
fn half_a_line_is_finished_on_a_later_visit() {
    let (root, place) = a_place("half-a-line");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    let mut daemon = UnixStream::connect(place.socket()).unwrap();
    daemon.write_all(b"open org.gnome").unwrap();
    assert_eq!(door.answered(&mut hand), 0, "half a line was acted on");
    assert_eq!(door.how_many_are_waiting(), 1, "the caller was dropped");

    daemon.write_all(b".Geary\n").unwrap();
    assert_eq!(door.answered(&mut hand), 1);
    assert_eq!(hand.carried, vec![Acting::Open(mail())]);
    assert_eq!(one_line(&mut daemon).as_deref(), Some("acted"));

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **A caller that says nothing at all is dropped when the patience runs out**,
/// so connecting and going quiet costs this door one slot for a second rather
/// than for as long as the machine is on.
#[test]
fn a_caller_that_says_nothing_is_eventually_dropped() {
    let (root, place) = a_place("quiet");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    let _quiet = UnixStream::connect(place.socket()).unwrap();
    assert_eq!(door.answered(&mut hand), 0);
    assert_eq!(door.how_many_are_waiting(), 1);

    for _ in 0..PATIENCE {
        door.answered(&mut hand);
    }
    assert_eq!(
        door.how_many_are_waiting(),
        0,
        "a caller that never spoke is still held"
    );

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **The number held at once is bounded**, so a process opening connections in
/// a loop costs a fixed amount of memory and no time at all.
#[test]
fn the_number_of_quiet_callers_held_at_once_is_bounded() {
    let (root, place) = a_place("bounded");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    let mut rude = Vec::new();
    for _ in 0..(AS_MANY_AS_WAIT * 4) {
        match UnixStream::connect(place.socket()) {
            Ok(connected) => rude.push(connected),
            // A kernel backlog that is full refuses, which is the same
            // protection one layer down.
            Err(why) if why.kind() == ErrorKind::ConnectionRefused => break,
            Err(why) => panic!("{why}"),
        }
    }
    door.answered(&mut hand);

    assert!(
        door.how_many_are_waiting() <= AS_MANY_AS_WAIT,
        "{} callers are held, over the bound of {AS_MANY_AS_WAIT}",
        door.how_many_are_waiting()
    );

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **A door nobody is at answers nothing and does not wait**, which is the
/// ordinary case every frame.
#[test]
fn a_door_nobody_is_at_costs_nothing() {
    let (root, place) = a_place("quiet-door");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    for _ in 0..100 {
        assert_eq!(door.answered(&mut hand), 0);
    }
    assert_eq!(door.how_many_are_waiting(), 0);
    assert!(hand.carried.is_empty());

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}

/// **Every act reaches the hand as itself**, so the wire's three words and the
/// vocabulary's three variants cannot drift apart.
#[test]
fn each_of_the_three_acts_reaches_the_machine_as_itself() {
    let (root, place) = a_place("three-acts");
    let mut door = TheApplicationDoor::served(place.clone(), us()).unwrap();
    let mut hand = AHand::answering(Answer::Acted);

    for (line, expected) in [
        ("open org.gnome.Geary\n", Acting::Open(mail())),
        ("focus org.gnome.Geary\n", Acting::Focus(mail())),
        ("close org.gnome.Geary\n", Acting::Close(mail())),
    ] {
        let mut daemon = UnixStream::connect(place.socket()).unwrap();
        daemon.write_all(line.as_bytes()).unwrap();
        assert_eq!(door.answered(&mut hand), 1, "{line:?}");
        assert_eq!(one_line(&mut daemon).as_deref(), Some("acted"), "{line:?}");
        assert_eq!(hand.carried.last(), Some(&expected), "{line:?}");
    }
    assert_eq!(hand.carried.len(), 3);

    door.taken_away();
    let _ = std::fs::remove_dir_all(root);
}
