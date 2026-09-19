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

    /// Whether this codec's patents have run out, so a software decoder for it
    /// may ship without licensing anything.
    ///
    /// **False for every video codec, including baseline H.264**, and that is a
    /// decision rather than an oversight. Baseline H.264's core filings are old
    /// enough to argue about, but the H.264 family's terms differ by profile and
    /// by filing, and a file does not announce its profile before it is decoded.
    /// [ADR 0058] leaves H.264 to the silicon or to `openh264`, where somebody
    /// else has already answered the question.
    ///
    /// [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
    #[must_use]
    pub const fn has_expired(self) -> bool {
        false
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

    /// Whether this codec was **free by design** — never encumbered, by
    /// somebody's choice when it was published.
    ///
    /// **MP3 is not here any more, and that is the change [ADR 0058] made.** It
    /// carries no royalty, but for the other reason: its term ran out. The two
    /// used to be folded together, and folding them lost the only distinction
    /// that matters when a lawyer reads this — *was it always free* against
    /// *did it stop being encumbered on a date somebody could argue about*. See
    /// [`Self::has_expired`].
    ///
    /// [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
    #[must_use]
    pub const fn is_royalty_free(self) -> bool {
        matches!(self, Self::Opus | Self::Vorbis | Self::Flac | Self::Pcm)
    }

    /// Whether this codec's patents have run out, so a software decoder for it
    /// may ship without licensing anything.
    ///
    /// **MP3 and AAC-LC**, and they are not equally settled. MP3's last patents
    /// expired in 2017 and its licensor publicly closed its programme — a fact
    /// with a date. AAC-LC's core filings are from 1997 and are past a
    /// twenty-year term everywhere alo OS ships, but the pool that licenses the
    /// family still exists for the later extensions, so the position is *the
    /// part we want has expired, inside a family that has not*. That is the one
    /// sentence [ADR 0058] sends to a lawyer, and
    /// [`crate::right::SoftwareDecoders::THE_ONE_FOR_COUNSEL`] names it.
    ///
    /// [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
    #[must_use]
    pub const fn has_expired(self) -> bool {
        matches!(self, Self::Mp3 | Self::AacLc)
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
            vec![Audio::Opus, Audio::Vorbis, Audio::Flac, Audio::Pcm]
        );
    }

    /// **The expired codecs are named separately from the free ones**, because
    /// they are true for a different reason and only one of the two is
    /// arguable.
    #[test]
    fn the_expired_codecs_are_the_two_whose_terms_ran_out() {
        let expired_video: Vec<Video> = Video::EVERY
            .into_iter()
            .filter(|codec| codec.has_expired())
            .collect();
        assert_eq!(expired_video, Vec::<Video>::new());

        let expired_audio: Vec<Audio> = Audio::EVERY
            .into_iter()
            .filter(|codec| codec.has_expired())
            .collect();
        assert_eq!(expired_audio, vec![Audio::Mp3, Audio::AacLc]);
    }

    /// **Nothing is both free by design and expired**, which would mean the two
    /// predicates had stopped meaning different things.
    #[test]
    fn no_codec_is_free_by_design_and_also_expired() {
        for codec in Video::EVERY {
            assert!(
                !(codec.is_royalty_free() && codec.has_expired()),
                "{codec:?}"
            );
        }
        for codec in Audio::EVERY {
            assert!(
                !(codec.is_royalty_free() && codec.has_expired()),
                "{codec:?}"
            );
        }
    }

    /// **Nothing encumbered is called free**, which is the mistake that would
    /// cost the most.
    ///
    /// AAC-LC ships (ADR 0058) *because its term ran out*, not because it was
    /// free — and the difference is the whole of what a lawyer is being asked.
    /// Folding it into `is_royalty_free` would make the question unanswerable
    /// by making it invisible. H.264 and HEVC are neither, and ship in neither
    /// sense.
    #[test]
    fn nothing_encumbered_is_called_free() {
        assert!(!Audio::AacLc.is_royalty_free());
        assert!(Audio::AacLc.has_expired());

        for codec in [Video::H264Baseline, Video::H264Main, Video::Hevc] {
            assert!(!codec.is_royalty_free(), "{codec:?}");
            assert!(!codec.has_expired(), "{codec:?}");
        }
    }
}
