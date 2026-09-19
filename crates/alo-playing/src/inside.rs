//! **What is inside a file**, which is not what the file is.
//!
//! ADR 0051's sharpest sentence about shape: *a kind is the wrapping, not the
//! codec*. One Matroska file holds AV1 that this machine plays and the next
//! holds something it may not. `alo_opening::Kind` answers *this is a film*;
//! this answers *and these are its tracks*, and only the second can decide
//! whether it plays.
//!
//! Getting this wrong is a machine that says *I can play `.mkv`* and then cannot
//! play the `.mkv` in front of somebody — which is worse than refusing, because
//! it refused after promising.

use crate::codec::{Audio, Video};

/// **The tracks inside one file.**
///
/// A film has a picture and usually sound. A sound recording has no picture, and
/// that is not a fault: `.m4a`, `.mp3` and `.flac` all arrive this way, and a
/// reader that treated a missing picture as damage would refuse most of what
/// people are sent.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Inside {
    /// The picture, where there is one.
    pub picture: Option<Video>,
    /// The sound tracks, in the order the file lists them.
    pub sound: Vec<Audio>,
}

impl Inside {
    /// A film: a picture, and one sound track.
    #[must_use]
    pub fn a_film(picture: Video, sound: Audio) -> Self {
        Self {
            picture: Some(picture),
            sound: vec![sound],
        }
    }

    /// A sound recording with no picture.
    #[must_use]
    pub fn a_recording(sound: Audio) -> Self {
        Self {
            picture: None,
            sound: vec![sound],
        }
    }

    /// Whether this file carries nothing at all.
    ///
    /// A container that parsed and listed no tracks is a real thing — a
    /// truncated download, a file somebody renamed — and it is **not** a file
    /// this machine can play. Answering *yes, playable* for a file with nothing
    /// in it is the *player that shows nothing* the decision refuses by name.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.picture.is_none() && self.sound.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A recording with no picture is a file, not a fault.**
    #[test]
    fn a_sound_recording_has_no_picture_and_is_not_empty() {
        let recording = Inside::a_recording(Audio::Mp3);
        assert!(recording.picture.is_none());
        assert!(!recording.is_empty());
    }

    /// **A container with nothing in it is empty**, and says so.
    #[test]
    fn a_file_with_no_tracks_at_all_is_empty() {
        assert!(Inside::default().is_empty());
        assert!(!Inside::a_film(Video::Av1, Audio::Opus).is_empty());
    }
}
