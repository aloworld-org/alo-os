//! **Where the right to decode came from**, which is the whole of ADR 0051's
//! second half.
//!
//! The decision's sentence: *the right to decode comes from the hardware, or
//! from somebody who licensed redistribution — never from shipping a decoder we
//! have no right to distribute.* Those are different things, and keeping them
//! apart is why this is a type rather than a boolean.
//!
//! A machine may play an H.264 file **because the chip in it holds a licence**,
//! or **because a company that paid for redistribution gave us a binary to pass
//! on**. It may not play one because we compiled a decoder and hoped.
//!
//! # Why a refusal is one of the answers
//!
//! [`Right::None`] is not an error state. It is the last step of the order, and
//! it is reached on a real machine that a real person bought: no hardware
//! decoder, no redistributable decoder, an HEVC film from a newer phone. **Some
//! people will meet it**, which is why the refusal it turns into is the one they
//! already know from documents rather than a new one for video.
//!
//! [ADR 0058] made that rarer than ADR 0051 left it — every audio codec in the
//! list now decodes in software, so sound never fails on its own, and H.264
//! reaches `openh264` where there is no silicon. What is left at the refusal is
//! HEVC without a hardware decoder.
//!
//! [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md

/// **Where this machine's right to decode a codec comes from**, if it has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Right {
    /// The codec carries no royalty to anybody, so the right needs no source.
    ///
    /// First in the order because it is not really a step: nothing is being
    /// licensed and nothing can be withdrawn.
    RoyaltyFree,
    /// **The patents ran out**, so the image may carry a software decoder for
    /// it without licensing anything from anybody.
    ///
    /// Separate from [`Self::RoyaltyFree`] because the two are true for
    /// different reasons and only one of them is arguable. A format that was
    /// made free was free on the day it was published; a format whose term
    /// expired was encumbered until a date, and *which* date depends on which
    /// filings are counted and where. [ADR 0058] names the one row of its table
    /// that rests on this, so that it is argued about on purpose.
    ///
    /// [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
    ByExpiry,
    /// **The silicon.** A hardware decoder on this machine — VA-API on a
    /// graphics card, V4L2 on an embedded decoder. The licence was paid for
    /// with the chip, and the file never touches a decoder we distributed.
    FromTheSilicon,
    /// **A redistributable licensed decoder**, such as Cisco's `openh264`,
    /// published under terms that let it be passed on with the royalty already
    /// paid by its publisher.
    FromALicensedDecoder,
    /// **None.** This machine has no right to decode this, and says so.
    None,
}

impl Right {
    /// Every answer, in the order the right to decode exists at all.
    ///
    /// **The order is not a preference.** Each step is reached only because the
    /// one above it was absent, which is what `crate::deciding` walks.
    pub const IN_ORDER: [Self; 5] = [
        Self::RoyaltyFree,
        Self::ByExpiry,
        Self::FromTheSilicon,
        Self::FromALicensedDecoder,
        Self::None,
    ];

    /// Whether this machine may decode at all.
    #[must_use]
    pub const fn may_decode(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether the right came from something that could be taken away.
    ///
    /// True of a licensed decoder, whose publisher pays the royalty and could
    /// stop; false of the silicon, which is bolted to the machine, and of a free
    /// codec, where there is nothing to withdraw. Nothing in this crate reads
    /// it: it is here because the difference is the reason these are two answers
    /// rather than one, and a type that records a difference nobody can read has
    /// not recorded it.
    #[must_use]
    pub const fn could_be_withdrawn(self) -> bool {
        matches!(self, Self::FromALicensedDecoder)
    }

    /// Whether this right rests on a patent term having run out.
    ///
    /// True of exactly one answer, and it is the one a lawyer is being asked
    /// about. Everything else is true because nobody ever charged for it, or
    /// because somebody else is paying, or because the answer is no — none of
    /// which turn on a date.
    #[must_use]
    pub const fn rests_on_a_patent_term(self) -> bool {
        matches!(self, Self::ByExpiry)
    }
}

/// **The software decoders the image ships**, which is the question ADR 0051
/// left open and [ADR 0058] closed.
///
/// The rule underneath the list, which is what a codec arriving later is
/// measured against: **we ship a decoder when nobody can charge us for shipping
/// it** — because it was made free, because the patents expired, or because a
/// company that paid the royalty published a binary for us to pass on.
/// Everything else is the silicon's job, and where there is no silicon the
/// machine says so.
///
/// So the image carries no H.264 decoder of its own (that is `openh264`'s job,
/// and Cisco's bill) and no HEVC decoder at all (nobody publishes one we may
/// pass on, and the patents are nowhere near term).
///
/// [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoftwareDecoders {
    /// **Everything free by design, everything whose patents have expired, and
    /// nothing we would need a licence to distribute.**
    FreeByDesignAndExpired,
}

impl SoftwareDecoders {
    /// The video codecs the image decodes in software.
    ///
    /// Both of them free by design. Nothing expired is here: baseline H.264 is
    /// the closest, and [ADR 0058] still leaves it to the silicon or to
    /// `openh264`, because the H.264 family's terms differ by profile and by
    /// filing in a way AAC-LC's do not.
    ///
    /// [ADR 0058]: ../../../docs/decisions/0058-which-software-decoders-the-image-ships.md
    pub const VIDEO: [crate::codec::Video; 2] =
        [crate::codec::Video::Av1, crate::codec::Video::Vp9];

    /// The audio codecs the image decodes in software, which is all of them.
    ///
    /// Four free by design, and two — MP3 and AAC-LC — whose patents ran out.
    pub const AUDIO: [crate::codec::Audio; 6] = crate::codec::Audio::EVERY;

    /// **The one row of the list that rests on a patent term**, and therefore
    /// the one sentence a lawyer is being asked to confirm.
    ///
    /// Named here rather than left in prose because it is the part most likely
    /// to be forgotten once playback works. MP3 rests on a term too, but on one
    /// that ran out in 2017 with the licensor publicly closing its programme —
    /// a fact with a date, not a reading. AAC-LC is the reading.
    pub const THE_ONE_FOR_COUNSEL: crate::codec::Audio = crate::codec::Audio::AacLc;

    /// What the image ships today.
    ///
    /// A function rather than a constant, so that changing the answer is an
    /// edit in one place with every caller still compiling.
    #[must_use]
    pub const fn in_the_image() -> Self {
        Self::FreeByDesignAndExpired
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The order is the order the right exists in**, written out so that
    /// reordering it is a change to a test somebody reads.
    #[test]
    fn the_order_is_free_then_expired_then_silicon_then_a_licensed_decoder_then_a_refusal() {
        assert_eq!(
            Right::IN_ORDER,
            [
                Right::RoyaltyFree,
                Right::ByExpiry,
                Right::FromTheSilicon,
                Right::FromALicensedDecoder,
                Right::None,
            ]
        );
    }

    /// **Exactly one answer is a refusal, and it is the last.**
    #[test]
    fn only_the_last_answer_refuses() {
        let refusing: Vec<Right> = Right::IN_ORDER
            .into_iter()
            .filter(|right| !right.may_decode())
            .collect();
        assert_eq!(refusing, vec![Right::None]);
    }

    /// **Only a licensed decoder's right could be withdrawn.**
    ///
    /// The silicon cannot be taken back out of a machine somebody owns, and a
    /// free codec has nobody to withdraw anything.
    #[test]
    fn only_a_licensed_decoders_right_can_be_taken_away() {
        let withdrawable: Vec<Right> = Right::IN_ORDER
            .into_iter()
            .filter(|right| right.could_be_withdrawn())
            .collect();
        assert_eq!(withdrawable, vec![Right::FromALicensedDecoder]);
    }

    /// **Exactly one answer rests on a patent term**, and it is the one a
    /// lawyer is being asked about.
    #[test]
    fn only_expiry_rests_on_a_patent_term() {
        let resting: Vec<Right> = Right::IN_ORDER
            .into_iter()
            .filter(|right| right.rests_on_a_patent_term())
            .collect();
        assert_eq!(resting, vec![Right::ByExpiry]);
    }

    /// **A term that ran out cannot be un-run**, so nothing about expiry is
    /// withdrawable.
    #[test]
    fn an_expired_patent_is_not_a_right_that_can_be_taken_back() {
        assert!(!Right::ByExpiry.could_be_withdrawn());
        assert!(Right::ByExpiry.may_decode());
    }

    /// **The image ships no decoder it would need a licence to distribute.**
    ///
    /// Held as the rule rather than as the list: every codec named in
    /// [`SoftwareDecoders`] is free by design or past term, and the two that
    /// are neither — H.264 and HEVC — are absent.
    #[test]
    fn the_image_ships_only_what_nobody_can_charge_for() {
        for codec in SoftwareDecoders::VIDEO {
            assert!(
                codec.is_royalty_free() || codec.has_expired(),
                "{codec:?} is in the image and somebody could charge for it"
            );
        }
        for codec in SoftwareDecoders::AUDIO {
            assert!(
                codec.is_royalty_free() || codec.has_expired(),
                "{codec:?} is in the image and somebody could charge for it"
            );
        }
        assert!(!SoftwareDecoders::VIDEO.contains(&crate::codec::Video::H264Baseline));
        assert!(!SoftwareDecoders::VIDEO.contains(&crate::codec::Video::H264Main));
        assert!(!SoftwareDecoders::VIDEO.contains(&crate::codec::Video::Hevc));
    }

    /// **The question left for a lawyer is one codec**, and it is one the image
    /// actually ships — an open question about something we are not doing would
    /// not need asking.
    #[test]
    fn the_one_for_counsel_is_shipped_and_rests_on_a_term() {
        assert!(SoftwareDecoders::AUDIO.contains(&SoftwareDecoders::THE_ONE_FOR_COUNSEL));
        assert!(SoftwareDecoders::THE_ONE_FOR_COUNSEL.has_expired());
        assert!(!SoftwareDecoders::THE_ONE_FOR_COUNSEL.is_royalty_free());
    }

    /// **The answer is the answer**, and changing it is an edit somebody reads.
    #[test]
    fn the_image_ships_what_is_free_by_design_and_what_has_expired() {
        assert_eq!(
            SoftwareDecoders::in_the_image(),
            SoftwareDecoders::FreeByDesignAndExpired
        );
    }
}
