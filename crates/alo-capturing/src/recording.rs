//! **Recording the screen, with or without the room.**
//!
//! Task 4 of `docs/autonomy/v0-5-capture-and-the-room-plan.md`, and
//! [ADR 0051](../../../docs/decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
//! decides what comes out: **VP9 where software must encode, AV1 where hardware
//! can, Opus, in Matroska** — royalty-free, because an image distributed across
//! the EU cannot ship encoders it does not own the right to produce.
//!
//! # Sound is the part people forget is in the room
//!
//! Recording the screen with the microphone records **whoever is speaking near
//! the machine**: a colleague at the next desk, somebody on a call across the
//! room, a child in the background. None of them is looking at the screen, none
//! of them agreed, and the indicator that makes this honest for the person
//! holding the machine is invisible to everybody else in the room.
//!
//! That is a constraint on this type rather than a warning somewhere:
//!
//! - **[`Sound::None`] is where every recording starts.** There is no
//!   constructor that begins with the microphone on.
//! - **It is chosen for this recording**, never remembered from the last one —
//!   there is nowhere here to remember it, because the room is not the same
//!   room.
//! - **[`Sound::TheMachine`] records nobody**, and is what most screen
//!   recordings actually want.
//! - Nothing here listens between recordings: a [`Recording`] exists while one
//!   runs, and `alo-in-use` shows the microphone exactly then.
//!
//! # A machine that cannot keep up says so before it starts
//!
//! ADR 0051's rule, and ADR 0008's before it: never a silent fallback. A
//! recording that cannot be encoded at the screen's size and rate is
//! [`TooMuchToEncode`] **before anything is written**, naming what would work —
//! not begun and quietly dropped to fifteen frames a second, which a person
//! discovers when they watch it back, usually once, usually when it mattered.

use crate::folder::Folder;
use crate::screen::Screen;
use crate::what::What;
use crate::words::{self, Word};

/// **What a recording carries besides the picture.**
///
/// Four states and no default that listens: [`Sound::None`] is what a recording
/// has until a person says otherwise, for this recording only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Sound {
    /// No sound at all.
    #[default]
    None,
    /// What this machine is playing — nobody in the room is recorded.
    TheMachine,
    /// The microphone: **anybody speaking near this machine**.
    TheRoom,
    /// Both.
    TheMachineAndTheRoom,
}

impl Sound {
    /// All four, in the order a person meets them — quietest first.
    pub const ALL: [Self; 4] = [
        Self::None,
        Self::TheMachine,
        Self::TheRoom,
        Self::TheMachineAndTheRoom,
    ];

    /// **Whether this records anybody who is not looking at the screen.**
    ///
    /// The question the sentence beside it answers, and the one that decides
    /// whether the microphone's line is lit.
    #[must_use]
    pub const fn reaches_the_room(self) -> bool {
        matches!(self, Self::TheRoom | Self::TheMachineAndTheRoom)
    }

    /// Whether the machine's own sound is recorded.
    #[must_use]
    pub const fn takes_what_the_machine_plays(self) -> bool {
        matches!(self, Self::TheMachine | Self::TheMachineAndTheRoom)
    }

    /// What a person reads when they choose it.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::None => words::NO_SOUND,
            Self::TheMachine => words::THIS_MACHINES_SOUND,
            Self::TheRoom => words::THE_MICROPHONE,
            Self::TheMachineAndTheRoom => words::BOTH_SOUNDS,
        }
    }
}

/// **What this machine writes**, per ADR 0051.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Encoded {
    /// AV1, where the hardware encodes it.
    Av1,
    /// VP9, where software must encode — which is most machines today.
    Vp9,
}

impl Encoded {
    /// **What this machine can do**, which is a measurement rather than a
    /// setting: AV1 only where hardware encodes it.
    #[must_use]
    pub const fn on_this_machine(hardware_encodes_av1: bool) -> Self {
        if hardware_encodes_av1 {
            Self::Av1
        } else {
            Self::Vp9
        }
    }

    /// The file this ends up in. Matroska either way, and the sound is always
    /// Opus (ADR 0051).
    #[must_use]
    pub const fn in_a_file_called(self) -> &'static str {
        "mkv"
    }
}

/// **A recording that has not started**, and everything decided before it does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    /// The screen, a window or a region.
    of: What,
    /// Where the file goes.
    into: Folder,
    /// What sound it carries — [`Sound::None`] until somebody says otherwise.
    sound: Sound,
    /// What it will be encoded as.
    encoded: Encoded,
}

impl Recording {
    /// **A recording of this, into that folder, with no sound.**
    #[must_use]
    pub fn of(what: What, into: Folder, encoded: Encoded) -> Self {
        Self {
            of: what,
            into,
            sound: Sound::None,
            encoded,
        }
    }

    /// **With this sound, for this recording.**
    ///
    /// Taken every time a recording is made, because there is nowhere here to
    /// remember it from last time.
    #[must_use]
    pub const fn with(mut self, sound: Sound) -> Self {
        self.sound = sound;
        self
    }

    /// What is being recorded.
    #[must_use]
    pub const fn what(&self) -> &What {
        &self.of
    }

    /// Where the file goes.
    #[must_use]
    pub const fn into(&self) -> &Folder {
        &self.into
    }

    /// What sound it carries.
    #[must_use]
    pub const fn sound(&self) -> Sound {
        self.sound
    }

    /// What it is encoded as.
    #[must_use]
    pub const fn encoded(&self) -> Encoded {
        self.encoded
    }

    /// **Whether this machine can encode it as fast as it happens.**
    ///
    /// `frames_a_second_it_can_encode` is what the machine measured for this
    /// encoder at this size; `wanted` is the screen's own rate. Refused
    /// **before anything is written**, naming what would work — never begun and
    /// quietly degraded (ADR 0051, ADR 0008).
    ///
    /// # Errors
    /// [`TooMuchToEncode`], carrying both numbers and the size that would.
    pub fn can_be_encoded_live(
        &self,
        screen: Screen,
        wanted: u32,
        frames_a_second_it_can_encode: u32,
        and_at_this_size_it_could: Option<(u32, u32)>,
    ) -> Result<(), TooMuchToEncode> {
        if frames_a_second_it_can_encode >= wanted {
            return Ok(());
        }
        Err(TooMuchToEncode {
            width: screen.width(),
            height: screen.height(),
            wanted,
            it_can_do: frames_a_second_it_can_encode,
            but_at: and_at_this_size_it_could,
        })
    }
}

/// **This machine cannot encode that, at that size, as fast as it happens.**
///
/// A refusal rather than a slower recording. What it carries is what a person
/// needs to choose again: what they asked for, what the machine managed, and a
/// size that would work where there is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error(
    "this machine encodes {width}×{height} at {it_can_do} frames a second and the recording \
     wants {wanted}"
)]
pub struct TooMuchToEncode {
    /// How wide the picture is.
    pub width: u32,
    /// How tall it is.
    pub height: u32,
    /// How many frames a second the recording wanted.
    pub wanted: u32,
    /// How many this machine can encode.
    pub it_can_do: u32,
    /// A size it could manage, where there is one.
    pub but_at: Option<(u32, u32)>,
}

impl TooMuchToEncode {
    /// What a person reads.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self.but_at {
            Some(_) => words::TOO_MUCH_TO_ENCODE_BUT,
            None => words::TOO_MUCH_TO_ENCODE,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn a_recording() -> Recording {
        Recording::of(
            What::TheWholeScreen,
            Folder::chosen(std::path::Path::new("/home/anna/Recordings")).expect("a folder"),
            Encoded::Vp9,
        )
    }

    /// **A recording starts with no sound**, and nothing here remembers a
    /// choice from last time.
    #[test]
    fn a_recording_records_nobody_until_somebody_says_otherwise() {
        let recording = a_recording();
        assert_eq!(recording.sound(), Sound::None);
        assert!(!recording.sound().reaches_the_room());

        // And choosing it is per recording: a second one starts silent again.
        let with_the_room = a_recording().with(Sound::TheRoom);
        assert!(with_the_room.sound().reaches_the_room());
        assert_eq!(a_recording().sound(), Sound::None);
    }

    /// **The machine's own sound records nobody in the room.**
    #[test]
    fn the_machines_own_sound_does_not_reach_the_room() {
        assert!(!Sound::TheMachine.reaches_the_room());
        assert!(Sound::TheMachine.takes_what_the_machine_plays());
        assert!(Sound::TheRoom.reaches_the_room());
        assert!(!Sound::TheRoom.takes_what_the_machine_plays());
        assert!(Sound::TheMachineAndTheRoom.reaches_the_room());
    }

    /// **AV1 only where hardware encodes it** (ADR 0051, measured).
    #[test]
    fn software_encoding_is_vp9_and_hardware_may_be_av1() {
        assert_eq!(Encoded::on_this_machine(false), Encoded::Vp9);
        assert_eq!(Encoded::on_this_machine(true), Encoded::Av1);
        assert_eq!(Encoded::Vp9.in_a_file_called(), "mkv");
    }

    /// **A machine that cannot keep up refuses before it starts**, naming what
    /// would work.
    #[test]
    fn a_recording_this_machine_cannot_encode_is_refused_before_it_begins() {
        let screen = Screen::measuring(1920, 1080).expect("a screen");
        let recording = a_recording();

        assert_eq!(recording.can_be_encoded_live(screen, 30, 195, None), Ok(()));

        let refused = recording
            .can_be_encoded_live(screen, 30, 20, Some((1280, 720)))
            .expect_err("twenty frames a second is not thirty");
        assert_eq!(refused.wanted, 30);
        assert_eq!(refused.it_can_do, 20);
        assert_eq!(refused.but_at, Some((1280, 720)));
        assert_eq!(refused.word().key(), words::TOO_MUCH_TO_ENCODE_BUT.key());

        // And where nothing would work, it says that instead.
        let nothing_would = recording
            .can_be_encoded_live(screen, 30, 4, None)
            .expect_err("four frames a second is not thirty");
        assert_eq!(nothing_would.word().key(), words::TOO_MUCH_TO_ENCODE.key());
    }

    /// **Every sound choice has a sentence**, and no two share one.
    #[test]
    fn every_sound_a_person_can_choose_is_named() {
        let mut keys: Vec<String> = Sound::ALL
            .into_iter()
            .map(|sound| sound.word().key().to_string())
            .collect();
        keys.sort();
        keys.dedup();
        assert_eq!(keys.len(), Sound::ALL.len());
    }
}
