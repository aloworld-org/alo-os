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
//! [`Right::None`] is not an error state. It is the third step of the order, and
//! it is reached on a real machine that a real person bought: no hardware
//! decoder, no redistributable decoder, an H.264 file in the mail. **Most people
//! will meet it at least once**, which is why the refusal it turns into is the
//! one they already know from documents rather than a new one for video.

/// **Where this machine's right to decode a codec comes from**, if it has one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Right {
    /// The codec carries no royalty to anybody, so the right needs no source.
    ///
    /// First in the order because it is not really a step: nothing is being
    /// licensed and nothing can be withdrawn.
    RoyaltyFree,
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
    pub const IN_ORDER: [Self; 4] = [
        Self::RoyaltyFree,
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
}

/// **A software decoder in the image is the question ADR 0051 left to counsel**,
/// and this is where that shows in the code.
///
/// The decision names four formats whose position differs — AAC-LC, baseline
/// H.264, main-profile H.264 and HEVC — and asks, for a machine sold and
/// distributed in the EU, which of them alo OS may include as a software decoder
/// in the image it ships, and under what notice.
///
/// **Until that is answered there is no fourth step.** This crate decides with
/// three, and a machine with no hardware decoder and no licensed decoder refuses
/// — which is the *none may ship* row of the decision's own table, taken as the
/// safe reading rather than as the answer.
///
/// # Why this is a type with one value
///
/// So that the hole is somewhere a reader trips over. A comment saying *counsel
/// has not answered* is a comment somebody deletes while tidying; a type that
/// every test asserts against is a hole with a shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SoftwareDecoders {
    /// **Nobody has answered which software decoders may ship.** The deadline is
    /// a shipment, not a version: before the certified laptop goes to anybody
    /// outside this team.
    NotAnsweredByCounsel,
}

impl SoftwareDecoders {
    /// Where this stands today.
    ///
    /// A function rather than a constant, so the day it is answered the change
    /// is here and every caller keeps compiling until it should not.
    #[must_use]
    pub const fn in_the_image() -> Self {
        Self::NotAnsweredByCounsel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The order is the order the right exists in**, written out so that
    /// reordering it is a change to a test somebody reads.
    #[test]
    fn the_order_is_free_then_silicon_then_a_licensed_decoder_then_a_refusal() {
        assert_eq!(
            Right::IN_ORDER,
            [
                Right::RoyaltyFree,
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

    /// **The counsel question is open, and this crate says so rather than
    /// assuming either answer.**
    #[test]
    fn no_software_decoder_ships_until_somebody_says_which_may() {
        assert_eq!(
            SoftwareDecoders::in_the_image(),
            SoftwareDecoders::NotAnsweredByCounsel
        );
    }
}
