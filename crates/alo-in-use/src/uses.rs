//! One current use of the screen, the camera or the microphone.
//!
//! What and who, and the number the media server records it under. Nothing
//! else: not when it started, not how long it has run, not what it produced.
//! This indicator answers *what is watching or listening, right now*, and every
//! field that is not part of that answer is a field somebody would eventually
//! want to keep.
//!
//! # Why there is a number on it
//!
//! Two applications reading one camera are two uses and two lines, and a person
//! stopping one of them later (the plan's task 4 puts *stop* on the indicator
//! itself) has to be able to say which. The number is the media server's own,
//! not one this crate invents, so what the person points at and what the server
//! knows about are the same thing.
//!
//! # And no clock
//!
//! Nothing here reads the time, as everywhere else in this repository. A use is
//! read from the server's record at the moment somebody looks; what happened
//! *earlier* is a different question and it belongs to a record rather than to
//! an indicator.

use crate::by::By;
use crate::line::Line;
use crate::used::Used;

/// The number the media server records one use under.
///
/// Unique across the server's record at one moment, which is what lets two uses
/// of one camera be told apart. Not stable across a restart of the machine, and
/// nothing here pretends otherwise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UseId(u32);

impl UseId {
    /// The number the media server recorded.
    #[must_use]
    pub const fn recorded(number: u32) -> Self {
        Self(number)
    }

    /// The number itself, for showing and for pointing at.
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }
}

/// One thing using the screen, the camera or the microphone at this moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Use {
    /// Which use the media server says this is.
    at: UseId,
    /// What is being used.
    what: Used,
    /// Who is using it.
    by: By,
}

impl Use {
    /// One use, as the media server records it.
    ///
    /// Public because [`crate::Streams`] is a port: whatever reaches the
    /// machine's media server has to be able to say what it found. The
    /// guarantee this crate makes is not about who may build one of these — it
    /// is that [`crate::InUse`] can be built no other way than by reading them
    /// all off a server, and that nothing between the two drops one.
    #[must_use]
    pub const fn of(at: UseId, what: Used, by: By) -> Self {
        Self { at, what, by }
    }

    /// Which use the media server says this is.
    #[must_use]
    pub const fn at(&self) -> UseId {
        self.at
    }

    /// What is being used.
    #[must_use]
    pub const fn what(&self) -> Used {
        self.what
    }

    /// Who is using it.
    #[must_use]
    pub const fn by(&self) -> &By {
        &self.by
    }

    /// The line the indicator shows for this use.
    #[must_use]
    pub fn line(&self) -> Line {
        Line::of(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{a_video_call, the_camera_by};

    /// **A use carries what and who and nothing else.** The test is the
    /// accessors: anything a later reader wanted to keep about a use — when it
    /// started, what it recorded — would have to be added here, in front of the
    /// argument in this file's documentation for why it is not.
    #[test]
    fn a_use_is_what_and_who_and_the_number_it_is_recorded_under() {
        let one = the_camera_by(By::an_application(a_video_call()), 42);
        assert_eq!(one.at(), UseId::recorded(42));
        assert_eq!(one.at().as_u32(), 42);
        assert_eq!(one.what(), Used::Camera);
        assert_eq!(one.by().application().map(|it| it.identifier()), {
            Some("com.example.VideoCall")
        });
    }

    /// **Two uses of one thing are two uses**, told apart by the server's own
    /// number — which is what a person stopping one of them later points at.
    #[test]
    fn two_things_using_one_camera_are_two_uses() {
        let first = the_camera_by(By::an_application(a_video_call()), 42);
        let second = the_camera_by(By::alo_os_itself(), 43);
        assert_ne!(first, second);
        assert_ne!(first.at(), second.at());
        assert_eq!(first.what(), second.what());
    }
}
