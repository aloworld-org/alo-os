//! **The refusal, which is not this crate's own.**
//!
//! ADR 0051 decided the shape before any of this was written: a film this
//! machine cannot play is reported through **`alo_opening::Cannot::NothingHereOpens`**
//! — *it is recognised, and nothing on this machine opens or converts it* — and
//! **not a second shape for video**.
//!
//! A person meets one sentence for *I cannot open this*, whether what they
//! double-clicked was a document or a film, with the same shape of *and here is
//! what would*. A second refusal written for media would drift from the first,
//! and the person reading it would have to learn two ways of being told the same
//! thing.
//!
//! So there is no vocabulary in this crate and there will not be one. This file
//! is the one place the decision reaches the sentence, and it is four lines
//! because that is all it should be.

use alo_opening::{Cannot, Kind};

use crate::deciding::Plays;

/// **How a file that does not play is reported.**
///
/// `None` where it plays: a file that plays has no refusal, and returning one
/// that callers are expected to ignore is how a *player that shows nothing*
/// starts.
///
/// # Panics on nothing, and refuses nothing of its own
///
/// A kind that is not a media kind is still reported honestly — it is
/// recognised, and nothing here opens it — because that sentence is true of it
/// too. Asking this crate about a spreadsheet is a caller's mistake, and a
/// mistake that produces a true sentence is not one worth a second error type.
#[must_use]
pub fn reported(kind: Kind, plays: &Plays) -> Option<Cannot> {
    if plays.all_of_it() {
        return None;
    }
    Some(Cannot::NothingHereOpens(kind))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use alo_opening::Would;

    use super::*;
    use crate::codec::{Audio, Video};
    use crate::deciding::plays;
    use crate::inside::Inside;
    use crate::machine::AMachine;

    /// **A film this machine plays produces no refusal at all.**
    #[test]
    fn nothing_is_reported_about_a_film_that_plays() {
        let decided = plays(
            &AMachine::with_nothing(),
            &Inside::a_film(Video::Av1, Audio::Opus),
        );
        assert_eq!(reported(Kind::MatroskaVideo, &decided), None);
    }

    /// **A film this machine cannot play is reported as the sentence a document
    /// is reported as**, and the remedy is the same one.
    ///
    /// Held here rather than trusted: the two roads being one is the decision,
    /// and a test that only checked the variant would still pass the day
    /// somebody gave media its own remedy.
    #[test]
    fn a_film_is_refused_the_way_a_document_is() {
        let decided = plays(
            &AMachine::with_nothing(),
            &Inside::a_film(Video::H264Baseline, Audio::AacLc),
        );
        let refusal = reported(Kind::Mp4Video, &decided).expect("a film nothing here plays");
        assert_eq!(refusal, Cannot::NothingHereOpens(Kind::Mp4Video));
        assert_eq!(refusal.would(), Would::AnotherMachineOrFormat);
        assert_eq!(
            refusal.would(),
            Cannot::NothingHereOpens(Kind::WordDocument).would(),
            "a film and a document are being told two different things"
        );
    }

    /// **A film whose sound alone cannot be decoded is refused, not played
    /// silently.**
    ///
    /// **No codec in the closed list reaches this any more** — ADR 0058 leaves
    /// every audio codec the decision names either free by design or past term
    /// — so the decision is built by hand rather than played out of a file. The
    /// rule has to outlive the list: the day an encumbered audio codec is
    /// added, a picture that decodes must not turn a refusal into silence.
    #[test]
    fn a_film_that_would_play_without_sound_is_still_refused() {
        let decided = crate::deciding::Plays {
            picture: Some(crate::right::Right::FromTheSilicon),
            sound: vec![crate::right::Right::None],
        };
        assert_eq!(
            reported(Kind::Mp4Video, &decided),
            Some(Cannot::NothingHereOpens(Kind::Mp4Video))
        );
    }

    /// **Every media kind refuses with the same sentence**, so no kind has
    /// quietly grown a road of its own.
    #[test]
    fn every_media_kind_is_refused_with_the_one_sentence() {
        let decided = plays(
            &AMachine::with_nothing(),
            &Inside::a_film(Video::Hevc, Audio::AacLc),
        );
        let media = [
            Kind::MatroskaVideo,
            Kind::WebmVideo,
            Kind::Mp4Video,
            Kind::Mp4Audio,
            Kind::AviVideo,
            Kind::OggMedia,
            Kind::Mp3Audio,
            Kind::WaveAudio,
            Kind::FlacAudio,
        ];
        for kind in media {
            assert!(kind.is_played(), "{kind:?} is not a kind this crate is for");
            assert_eq!(
                reported(kind, &decided),
                Some(Cannot::NothingHereOpens(kind)),
                "{kind:?} was refused some other way"
            );
        }
    }
}
