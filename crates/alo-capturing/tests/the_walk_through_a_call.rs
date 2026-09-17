//! **One walk: a screenshot, a call, and thirty seconds with the microphone.**
//!
//! Task 7 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`, and the last of
//! that plan. Every other test in these two crates holds one sentence, one
//! refusal, or one line of the indicator. This holds **the sequence** — what a
//! person actually reads and sees, in order, through one afternoon.
//!
//! A machine can pass every test of its parts and still make no sense read
//! through, and these parts are unusually easy to get right one at a time: each
//! sentence is careful, and the question is whether they are careful *together*.
//!
//! **Nothing here re-decides what the sentences say.** It reads them.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capturing::{
    Encoded, Folder, Mark, Marks, Picture, Recording, Region, Shared, Sound, What,
};
use alo_in_use::{By, Line, Use, UseId, Used};
use alo_strings::{Filling, Strings};

/// The vocabulary the whole machine holds.
fn what_this_machine_can_say() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().expect("the machine's vocabulary"))
}

/// A recorder, as an application the media server would name.
fn the_recorder() -> By {
    By::an_application(
        alo_applications::Application::identified("org.alo.Recorder").expect("an application"),
    )
}

/// **The walk, in the order a person meets it.**
#[test]
fn the_whole_afternoon_reads_in_order() {
    let strings = what_this_machine_can_say();
    let say = |word: alo_strings::Word| {
        strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_owned()
    };
    let mut walked: Vec<(String, String)> = Vec::new();

    // A screenshot of part of the screen, with a password blurred out.
    let marked = Marks::on(Picture::of(vec![1, 2, 3, 4]).expect("a picture")).and(Mark::Blur {
        over: Region::of(10, 40, 200, 20).expect("the field with the password in it"),
    });
    walked.push((
        "they pick the blur tool".to_owned(),
        say(Mark::Blur {
            over: Region::of(0, 0, 1, 1).expect("a region"),
        }
        .word()),
    ));
    assert!(marked.hides_anything());

    // They join a call and share one window.
    let sharing = Shared::the_person_picked(What::TheWholeScreen);
    walked.push((
        "before they share anything".to_owned(),
        say(Shared::nothing().word()),
    ));
    walked.push((
        "they share their whole screen".to_owned(),
        say(sharing.word()),
    ));
    assert!(
        !sharing.notifications_may_be_shown(),
        "a message from outside the meeting could be drawn in front of it"
    );

    // And they record thirty seconds with the microphone.
    let recording = Recording::of(
        What::TheWholeScreen,
        Folder::chosen(std::path::Path::new("/home/anna/Recordings")).expect("a folder"),
        Encoded::Vp9,
    )
    .with(Sound::TheRoom);
    walked.push((
        "they choose what the recording will hear".to_owned(),
        say(Sound::TheRoom.word()),
    ));

    // What the indicator shows while all this runs.
    for (what, said) in [
        (Used::Screen, "the screen is being read"),
        (Used::Microphone, "the microphone is listening"),
    ] {
        let line = Line::of(&Use::of(UseId::recorded(1), what, the_recorder()));
        walked.push((said.to_owned(), line.said(&strings).text().to_owned()));
    }

    assert!(recording.sound().reaches_the_room());
    assert_eq!(walked.len(), 6, "the walk is six things a person meets");

    for (doing, said) in &walked {
        println!("{doing} — {said}");
        assert!(!said.is_empty(), "{doing}: says nothing");
        assert!(
            !said.contains("capturing.") && !said.contains("in-use."),
            "{doing}: reaches a person as a key rather than a sentence — {said}"
        );
    }
}

/// **Nothing in the walk names the machinery.**
///
/// Not the media server, not the portal, not the encoder, not the format. A
/// person sharing a window has no use for the word *PipeWire*, and a person
/// reading *VP9* learns only that somebody left a note to themselves in the
/// product.
#[test]
fn no_sentence_in_this_walk_names_the_machinery() {
    let strings = what_this_machine_can_say();
    let mut named: Vec<String> = Vec::new();
    for word in alo_capturing::words::EVERY_WORD
        .iter()
        .chain(alo_in_use::words::EVERY_WORD.iter())
    {
        let said = strings
            .say(&word.key(), &Filling::nothing())
            .text()
            .to_lowercase();
        for machinery in [
            "pipewire",
            "portal",
            "wayland",
            "vp9",
            "av1",
            "opus",
            "matroska",
            "codec",
            "encoder",
            "ffmpeg",
            "gstreamer",
            "xdg",
        ] {
            if said.contains(machinery) {
                named.push(format!("{} says `{machinery}`", word.key()));
            }
        }
    }
    assert!(named.is_empty(), "the walk names the machinery: {named:#?}");
}

/// **Every sentence these two crates say has a translator's note.**
#[test]
fn every_sentence_tells_a_translator_what_it_is_for() {
    let mut without: Vec<String> = Vec::new();
    for word in alo_capturing::words::EVERY_WORD
        .iter()
        .chain(alo_in_use::words::EVERY_WORD.iter())
    {
        if word.note().is_none() {
            without.push(word.key().to_string());
        }
    }
    assert!(
        without.is_empty(),
        "sentences with nothing to tell a translator: {without:#?}"
    );
}
