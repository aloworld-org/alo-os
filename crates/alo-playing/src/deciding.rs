//! **Whether this machine plays a file, and where the right came from.**
//!
//! The order in ADR 0051, walked exactly once per track:
//!
//! 1. the codec is royalty-free, so there is nothing to have a right to;
//! 2. **the codec's patents ran out**, so the image's own software decoder may
//!    run — [ADR 0058], which closed the question ADR 0051 left open;
//! 3. **the machine's own hardware decoder** — the licence was paid for with the
//!    silicon;
//! 4. **a redistributable licensed decoder**, whose publisher paid the royalty;
//! 5. **a plain refusal**, naming what would play it.
//!
//! **The order is not a preference.** It is the order in which the right to
//! decode exists at all, and each step is reached only because the one above it
//! was absent. There is deliberately no sixth step: *a software decoder we built
//! for a format somebody still licenses* is what the decision refuses, and no
//! arrangement of these five reaches it.
//!
//! [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md

use crate::codec::{Audio, Video};
use crate::inside::Inside;
use crate::machine::AMachine;
use crate::right::Right;

/// **Whether a whole file plays**, and what each of its tracks decided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plays {
    /// What decided the picture, where there is one.
    pub picture: Option<Right>,
    /// What decided each sound track, in the file's own order.
    pub sound: Vec<Right>,
}

impl Plays {
    /// Whether every track in the file plays.
    ///
    /// **Every track, not any.** A film whose picture decodes and whose sound
    /// does not is a film somebody watches in silence and thinks is broken; the
    /// honest answer is that this machine cannot play it, with what would.
    #[must_use]
    pub fn all_of_it(&self) -> bool {
        self.picture.is_none_or(Right::may_decode)
            && self.sound.iter().copied().all(Right::may_decode)
    }

    /// The tracks this machine has no right to decode.
    ///
    /// What a diagnosis reads; a person reads `crate::refusing`.
    #[must_use]
    pub fn what_it_cannot_decode(&self) -> usize {
        usize::from(self.picture == Some(Right::None))
            + self.sound.iter().filter(|r| **r == Right::None).count()
    }
}

/// **Where this machine's right to decode this video codec comes from.**
#[must_use]
pub fn the_right_to_video(machine: &AMachine, codec: Video) -> Right {
    if codec.is_royalty_free() {
        return Right::RoyaltyFree;
    }
    if codec.has_expired() {
        return Right::ByExpiry;
    }
    if machine.decodes_video_in_hardware(codec) {
        return Right::FromTheSilicon;
    }
    if machine.has_a_licensed_decoder_for(codec) {
        return Right::FromALicensedDecoder;
    }
    Right::None
}

/// **Where this machine's right to decode this audio codec comes from.**
///
/// There is no licensed-decoder step here, and the absence is deliberate: the
/// redistributable decoders that exist are for video, and inventing a step for a
/// thing nobody publishes would be a list that looks decided and is not.
#[must_use]
pub fn the_right_to_sound(machine: &AMachine, codec: Audio) -> Right {
    if codec.is_royalty_free() {
        return Right::RoyaltyFree;
    }
    if codec.has_expired() {
        return Right::ByExpiry;
    }
    if machine.decodes_audio_in_hardware(codec) {
        return Right::FromTheSilicon;
    }
    Right::None
}

/// **Whether this machine plays this file**, track by track.
///
/// A file with no tracks at all does not play: see [`Inside::is_empty`].
#[must_use]
pub fn plays(machine: &AMachine, inside: &Inside) -> Plays {
    Plays {
        picture: inside
            .picture
            .map(|codec| the_right_to_video(machine, codec)),
        sound: inside
            .sound
            .iter()
            .map(|codec| the_right_to_sound(machine, *codec))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A free codec plays on a machine with nothing at all.**
    ///
    /// Which is the encoding half paying off: everything alo OS produces plays
    /// on every alo OS machine, with no hardware and no licence anywhere.
    #[test]
    fn what_this_machine_makes_plays_on_a_machine_with_nothing() {
        let machine = AMachine::with_nothing();
        let ours = Inside::a_film(Video::Av1, Audio::Opus);
        let decided = plays(&machine, &ours);
        assert!(decided.all_of_it());
        assert_eq!(decided.picture, Some(Right::RoyaltyFree));
        assert_eq!(decided.sound, vec![Right::RoyaltyFree]);
    }

    /// **The silicon is reached before a licensed decoder, and only when the
    /// codec is not free.**
    #[test]
    fn the_hardware_is_asked_before_a_licensed_decoder() {
        let machine = AMachine::with_nothing()
            .decoding_in_hardware(&[Video::H264Baseline])
            .with_a_licensed_decoder_for(&[Video::H264Baseline]);
        assert_eq!(
            the_right_to_video(&machine, Video::H264Baseline),
            Right::FromTheSilicon
        );
    }

    /// **A licensed decoder is reached only when the hardware is absent.**
    #[test]
    fn a_licensed_decoder_is_reached_when_there_is_no_hardware() {
        let machine = AMachine::with_nothing().with_a_licensed_decoder_for(&[Video::H264Baseline]);
        assert_eq!(
            the_right_to_video(&machine, Video::H264Baseline),
            Right::FromALicensedDecoder
        );
    }

    /// **A free codec never reaches the hardware step**, on any machine.
    ///
    /// Not an optimisation: a free codec that answered *from the silicon* would
    /// say a machine's right to play AV1 came from a chip, and a machine without
    /// that chip would then look like it had lost a right it never needed.
    #[test]
    fn a_free_codec_is_free_whatever_the_machine_has() {
        let everything = AMachine::with_nothing()
            .decoding_in_hardware(&Video::EVERY)
            .with_a_licensed_decoder_for(&Video::EVERY);
        for codec in Video::EVERY {
            if codec.is_royalty_free() {
                assert_eq!(
                    the_right_to_video(&everything, codec),
                    Right::RoyaltyFree,
                    "{codec:?} on a machine that has everything"
                );
            }
        }
    }

    /// **The common case refuses on the picture alone, and that is the decision
    /// working.**
    ///
    /// A plain laptop, no video hardware, an H.264 film from somebody's phone.
    /// **One refusing track is enough** — but only one of the two refuses now:
    /// ADR 0058 put AAC-LC in the image, so the sound decodes even here. The
    /// count matters, because a film refused for one reason and a film refused
    /// for two are different problems to fix.
    #[test]
    fn a_plain_machine_refuses_an_h264_film_from_a_phone() {
        let machine = AMachine::with_nothing();
        let from_a_phone = Inside::a_film(Video::H264Baseline, Audio::AacLc);
        let decided = plays(&machine, &from_a_phone);
        assert_eq!(decided.picture, Some(Right::None));
        assert_eq!(decided.sound, vec![Right::ByExpiry]);
        assert!(!decided.all_of_it());
        assert_eq!(decided.what_it_cannot_decode(), 1);
    }

    /// **The same film plays outright once the machine has the silicon.**
    ///
    /// The other half of the case above, and the reason AAC-LC was worth
    /// deciding: with a hardware H.264 decoder the picture is licensed by the
    /// chip and the sound is licensed by nobody, so an ordinary phone video
    /// plays end to end with nothing bought.
    #[test]
    fn the_same_film_plays_on_a_machine_that_has_the_silicon() {
        let machine = AMachine::with_nothing().decoding_in_hardware(&[Video::H264Baseline]);
        let decided = plays(&machine, &Inside::a_film(Video::H264Baseline, Audio::AacLc));
        assert_eq!(decided.picture, Some(Right::FromTheSilicon));
        assert_eq!(decided.sound, vec![Right::ByExpiry]);
        assert!(decided.all_of_it());
    }

    /// **A film whose sound cannot be decoded does not play**, however good the
    /// picture is.
    ///
    /// Silence with a picture is the failure that looks like success, and it is
    /// the one a person reports as *your machine broke my video*.
    ///
    /// **No codec in the closed list can produce this any more** — ADR 0058
    /// leaves every audio codec the decision names either free by design or
    /// past term — so the decision is built by hand rather than played out of a
    /// file. That is deliberate: the rule outlives the list, and the day an
    /// encumbered audio codec is added the rule has to already be right.
    #[test]
    fn a_film_with_a_picture_and_no_sound_does_not_play() {
        let decided = Plays {
            picture: Some(Right::FromTheSilicon),
            sound: vec![Right::None],
        };
        assert!(!decided.all_of_it());
        assert_eq!(decided.what_it_cannot_decode(), 1);
    }

    /// **Every audio codec the decision names now decodes on a machine with
    /// nothing**, which is ADR 0058 stated as the thing a person would notice.
    #[test]
    fn every_sound_the_decision_names_plays_on_a_bare_machine() {
        let bare = AMachine::with_nothing();
        for codec in Audio::EVERY {
            assert!(
                the_right_to_sound(&bare, codec).may_decode(),
                "{codec:?} does not decode on a machine with nothing"
            );
        }
    }

    /// **Every codec on every machine reaches exactly one answer**, and the
    /// answer is always one the order names.
    ///
    /// The exhaustive sweep: three machine shapes across every codec, which is
    /// the whole of what this decision can be asked.
    #[test]
    fn every_codec_on_every_machine_lands_on_one_of_the_answers() {
        let machines = [
            AMachine::with_nothing(),
            AMachine::with_nothing().decoding_in_hardware(&Video::EVERY),
            AMachine::with_nothing().with_a_licensed_decoder_for(&Video::EVERY),
        ];
        for machine in &machines {
            for codec in Video::EVERY {
                let right = the_right_to_video(machine, codec);
                assert!(
                    Right::IN_ORDER.contains(&right),
                    "{codec:?} landed outside the order"
                );
            }
            for codec in Audio::EVERY {
                let right = the_right_to_sound(machine, codec);
                assert!(
                    Right::IN_ORDER.contains(&right),
                    "{codec:?} landed outside the order"
                );
            }
        }
    }
}
