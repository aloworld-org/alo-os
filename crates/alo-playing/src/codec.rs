//! **The codecs [ADR 0051](../../../docs/decisions/0051-what-this-machine-encodes-is-royalty-free-and-what-it-plays-is-a-separate-question.md)
//! names**, and nothing else.
//!
//! A closed set, deliberately. A codec nobody decided about is not a codec this
//! machine quietly tries: the decision's whole shape is *what we encode we
//! choose, what we decode somebody already sent*, and both halves are lists a
//! person can read rather than whatever a rented library happened to be built
//! with.
//!
//! **Royalty-free is a property of the codec, not of this machine.** It is
//! recorded here because it is what decides the encoding half outright and what
//! makes the decoding half a question at all — and because it is a fact about
//! the format that does not change when the machine does.

/// **A video codec this decision names.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Video {
    /// AV1: royalty-free, and what this machine encodes where it can.
    Av1,
    /// VP9: royalty-free, and what it encodes where AV1 is too slow.
    Vp9,
    /// H.264, baseline profile: the common case in what people are sent.
    H264Baseline,
    /// H.264, main profile and above.
    H264Main,
    /// HEVC: licensed across more than one pool.
    Hevc,
}

impl Video {
    /// Every video codec this decision names, most free first.
    pub const EVERY: [Self; 5] = [
        Self::Av1,
        Self::Vp9,
        Self::H264Baseline,
        Self::H264Main,
        Self::Hevc,
    ];

    /// Whether this codec carries no royalty to anybody.
    ///
    /// The encoding half of ADR 0051 is exactly this predicate: alo OS encodes
    /// what is free and nothing else.
    #[must_use]
    pub const fn is_royalty_free(self) -> bool {
        matches!(self, Self::Av1 | Self::Vp9)
    }
}

/// **An audio codec this decision names.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Audio {
    /// Opus: royalty-free, and the only thing this machine encodes.
    Opus,
    /// Vorbis: royalty-free, and older.
    Vorbis,
    /// FLAC: royalty-free, and lossless.
    Flac,
    /// Uncompressed samples, as a WAVE file carries them.
    Pcm,
    /// MP3: its patents have expired.
    Mp3,
    /// AAC, low complexity: the common case in what people are sent.
    AacLc,
}

impl Audio {
    /// Every audio codec this decision names, most free first.
    pub const EVERY: [Self; 6] = [
        Self::Opus,
        Self::Vorbis,
        Self::Flac,
        Self::Pcm,
        Self::Mp3,
        Self::AacLc,
    ];

    /// Whether this codec carries no royalty to anybody.
    ///
    /// **MP3 is here and AAC-LC is not.** MP3's last patents expired in 2017,
    /// which is a fact with a date rather than a reading of filings; AAC-LC is
    /// the half of the open question in ADR 0051 that a lawyer has to answer,
    /// and `crate::right` is where that shows.
    #[must_use]
    pub const fn is_royalty_free(self) -> bool {
        matches!(
            self,
            Self::Opus | Self::Vorbis | Self::Flac | Self::Pcm | Self::Mp3
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every codec is in its own list once.**
    ///
    /// A closed set that lists something twice, or leaves something out, is a
    /// list nobody can reason about — and both halves of this decision are
    /// reasoning about a list.
    #[test]
    fn each_list_holds_each_codec_once() {
        for (place, codec) in Video::EVERY.iter().enumerate() {
            assert_eq!(
                Video::EVERY.iter().filter(|other| *other == codec).count(),
                1,
                "{codec:?} at {place} is listed more than once"
            );
        }
        for (place, codec) in Audio::EVERY.iter().enumerate() {
            assert_eq!(
                Audio::EVERY.iter().filter(|other| *other == codec).count(),
                1,
                "{codec:?} at {place} is listed more than once"
            );
        }
    }

    /// **What is royalty-free is exactly what ADR 0051 says is.**
    ///
    /// Written as the two lists rather than as the predicate, so that a change
    /// to the predicate has to be a change to a list somebody reads.
    #[test]
    fn the_free_codecs_are_the_ones_the_decision_names() {
        let free_video: Vec<Video> = Video::EVERY
            .into_iter()
            .filter(|codec| codec.is_royalty_free())
            .collect();
        assert_eq!(free_video, vec![Video::Av1, Video::Vp9]);

        let free_audio: Vec<Audio> = Audio::EVERY
            .into_iter()
            .filter(|codec| codec.is_royalty_free())
            .collect();
        assert_eq!(
            free_audio,
            vec![
                Audio::Opus,
                Audio::Vorbis,
                Audio::Flac,
                Audio::Pcm,
                Audio::Mp3
            ]
        );
    }

    /// **AAC-LC is not royalty-free here, and that is the open question showing
    /// through.**
    ///
    /// ADR 0051 leaves *which software decoders may ship* to counsel, and the
    /// wrong way to be ready for the answer is to assume it. Saying AAC-LC is
    /// free would be assuming it in the permissive direction, which is the more
    /// expensive mistake of the two.
    #[test]
    fn aac_is_not_claimed_free_while_counsel_has_not_answered() {
        assert!(!Audio::AacLc.is_royalty_free());
        assert!(!Video::H264Baseline.is_royalty_free());
        assert!(!Video::Hevc.is_royalty_free());
    }
}
