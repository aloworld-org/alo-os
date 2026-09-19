//! **What alo OS produces**, which is the half of ADR 0051 that was accepted
//! outright.
//!
//! A screen recording is a file this machine makes from nothing. Nobody is
//! waiting for it in a particular format and every constraint is ours to set, so
//! the answer is the free one: **AV1, or VP9 where AV1 cannot be encoded fast
//! enough; Opus; Matroska, or WebM where a file is meant for the web.**
//!
//! **AV1 only where hardware can encode it — measured, not preferred.** That
//! sentence is the reason [`Produces::decided_for`] takes a machine rather than
//! answering from a constant: a lane that picked AV1 because it is the better
//! format would have shipped a machine that takes four minutes to save a
//! thirty-second recording.

use crate::codec::{Audio, Video};

/// **Where a recording is going**, which is the only thing that changes the
/// container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Meant {
    /// Kept as a file, opened by whatever opens files.
    AsAFile,
    /// Put on the web, where the container has to be the one browsers name.
    ForTheWeb,
}

/// **The container a recording is written in.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Container {
    /// Matroska — `.mkv`.
    Matroska,
    /// WebM, which is Matroska under a name the web uses.
    Webm,
}

/// **What this machine will encode**, decided for one machine and one purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Produces {
    /// The video codec.
    pub video: Video,
    /// The audio codec, which is never anything but Opus.
    pub audio: Audio,
    /// The container.
    pub container: Container,
}

impl Produces {
    /// **What this machine encodes**, given whether its hardware can encode AV1
    /// quickly enough and where the recording is going.
    ///
    /// `av1_in_hardware` is an argument rather than something read here for the
    /// reason `crate::machine` gives: what a machine has is measured by whoever
    /// can measure it, and a decision that reads its own facts cannot be tested
    /// against a machine nobody has.
    #[must_use]
    pub const fn decided_for(av1_in_hardware: bool, meant: Meant) -> Self {
        Self {
            video: if av1_in_hardware {
                Video::Av1
            } else {
                Video::Vp9
            },
            audio: Audio::Opus,
            container: match meant {
                Meant::AsAFile => Container::Matroska,
                Meant::ForTheWeb => Container::Webm,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Everything this machine produces is royalty-free**, on every machine
    /// and for every purpose.
    ///
    /// The exhaustive form matters: this is the acceptance of ADR 0051's first
    /// half, and *we checked the common case* is not the same claim.
    #[test]
    fn nothing_this_machine_encodes_carries_a_royalty() {
        for av1 in [true, false] {
            for meant in [Meant::AsAFile, Meant::ForTheWeb] {
                let produces = Produces::decided_for(av1, meant);
                assert!(
                    produces.video.is_royalty_free(),
                    "{produces:?} encodes video somebody must be paid for"
                );
                assert!(
                    produces.audio.is_royalty_free(),
                    "{produces:?} encodes audio somebody must be paid for"
                );
            }
        }
    }

    /// **AV1 is chosen by what the hardware can do, and never by preference.**
    #[test]
    fn av1_only_where_the_hardware_encodes_it() {
        assert_eq!(
            Produces::decided_for(true, Meant::AsAFile).video,
            Video::Av1
        );
        assert_eq!(
            Produces::decided_for(false, Meant::AsAFile).video,
            Video::Vp9
        );
    }

    /// **The sound is Opus whatever else is true.**
    #[test]
    fn the_sound_is_always_opus() {
        for av1 in [true, false] {
            for meant in [Meant::AsAFile, Meant::ForTheWeb] {
                assert_eq!(Produces::decided_for(av1, meant).audio, Audio::Opus);
            }
        }
    }

    /// **Where a file is going decides the container, and nothing else does.**
    #[test]
    fn the_web_gets_webm_and_a_file_gets_matroska() {
        assert_eq!(
            Produces::decided_for(true, Meant::ForTheWeb).container,
            Container::Webm
        );
        assert_eq!(
            Produces::decided_for(true, Meant::AsAFile).container,
            Container::Matroska
        );
    }
}
