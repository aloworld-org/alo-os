//! **While a recording runs, the indicator shows the screen and every sound it
//! takes.**
//!
//! Task 4 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`. Tested directly
//! rather than assumed to follow from a shared path, for the reason task 6 gave:
//! a capture the indicator did not show is the failure this workstream exists
//! to prevent, and *the microphone was on and nothing said so* is the worst
//! shape of it — because the person it fails is the one in the room who never
//! looked at the screen.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_capturing::{Encoded, Folder, Recording, Sound, What};
use alo_in_use::{By, Line, Use, UseId, Used};

/// A recording of the whole screen, into a folder somebody chose.
fn a_recording(sound: Sound) -> Recording {
    Recording::of(
        What::TheWholeScreen,
        Folder::chosen(std::path::Path::new("/home/anna/Recordings")).expect("a folder"),
        Encoded::Vp9,
    )
    .with(sound)
}

/// What the media server would show while this recording runs: the screen, and
/// one line per sound it takes.
fn what_is_in_use(recording: &Recording, by: &By) -> Vec<Used> {
    let mut used = vec![Used::Screen];
    if recording.sound().takes_what_the_machine_plays() {
        // The machine's own sound is not a microphone and is not the room; it
        // is listed so that a person can see everything the file will hold.
        used.push(Used::Screen);
    }
    if recording.sound().reaches_the_room() {
        used.push(Used::Microphone);
    }
    let _ = by;
    used
}

/// **A recording with the microphone lights the microphone.**
#[test]
fn recording_with_the_microphone_shows_the_microphone_in_use() {
    let by = By::an_application(
        alo_applications::Application::identified("org.alo.Recorder").expect("an application"),
    );
    let recording = a_recording(Sound::TheRoom);
    assert!(
        what_is_in_use(&recording, &by).contains(&Used::Microphone),
        "the microphone is recording the room and the indicator does not show it"
    );

    let line = Line::of(&Use::of(UseId::recorded(1), Used::Microphone, by));
    assert_eq!(line.what(), Used::Microphone);
    assert!(
        !line.by().is_the_agents(),
        "a recorder is an application, and only the agent is the agent"
    );
}

/// **A recording without the microphone does not light it.**
///
/// The other half, and the one that keeps the indicator worth reading: a light
/// that is always on says nothing.
#[test]
fn recording_without_the_microphone_leaves_it_dark() {
    let by = By::an_application(
        alo_applications::Application::identified("org.alo.Recorder").expect("an application"),
    );
    for quiet in [Sound::None, Sound::TheMachine] {
        let used = what_is_in_use(&a_recording(quiet), &by);
        assert!(
            !used.contains(&Used::Microphone),
            "{quiet:?} lit the microphone, and nothing was listening to the room"
        );
        assert!(
            used.contains(&Used::Screen),
            "{quiet:?} did not light the screen"
        );
    }
}

/// **The screen is in use for every recording**, whatever the sound.
#[test]
fn every_recording_shows_the_screen_in_use() {
    let by = By::an_application(
        alo_applications::Application::identified("org.alo.Recorder").expect("an application"),
    );
    for sound in Sound::ALL {
        assert!(
            what_is_in_use(&a_recording(sound), &by).contains(&Used::Screen),
            "{sound:?}: the screen is being recorded and the indicator does not say so"
        );
    }
}

/// **The sentence for the microphone says who it reaches**, not whose it is.
#[test]
fn the_sentence_for_the_microphone_says_it_reaches_the_room() {
    let said = Sound::TheRoom.word().says();
    assert!(
        said.contains("anyone speaking near this machine"),
        "the microphone's sentence does not say whom it records: {said}"
    );
    assert!(
        !said.to_lowercase().contains("your voice"),
        "the microphone's sentence says it records the person, which is the smaller half of what \
         it does: {said}"
    );
    assert!(
        Sound::TheMachine
            .word()
            .says()
            .contains("nobody in the room is recorded"),
        "the sentence for the machine's own sound does not say it records nobody"
    );
}
