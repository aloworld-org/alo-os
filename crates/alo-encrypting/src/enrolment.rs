//! A machine whose disk is encrypted.
//!
//! One constructor, and it takes two things: how the disk opens, and the proof
//! that the person kept the recovery key. There is no second constructor, no
//! `Default`, and no field anybody outside this crate can set — so *this
//! machine's disk is encrypted* is a sentence that cannot be written in these
//! types without a [`WrittenDown`], and a [`WrittenDown`] cannot be made without
//! a person having typed their recovery key back.
//!
//! That is the whole of task 5's acceptance in
//! `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: **no road enrols
//! encryption without also producing the recovery key and requiring the person
//! to confirm they kept it.** It is held by there being no other road, and
//! `tests/no_road_enrols_without_a_recovery_key_the_person_kept.rs` reads this
//! crate's own public surface to say so rather than trusting this paragraph.

use crate::unlocking::HowItUnlocks;
use crate::written_down::WrittenDown;

/// This machine's disk, encrypted.
#[derive(Debug)]
pub struct Enrolment {
    /// How it opens.
    how: HowItUnlocks,
    /// That the person kept the key that recovers it.
    kept: WrittenDown,
}

impl Enrolment {
    /// The enrolment, from how the disk opens and the person having kept the
    /// recovery key.
    ///
    /// The second argument is not an option and is not a flag. It is a value
    /// only [`crate::RecoveryKey::written_back`] makes, and only from a typing
    /// that matched.
    #[must_use]
    pub fn made(how: HowItUnlocks, kept: WrittenDown) -> Self {
        Self { how, kept }
    }

    /// How the disk opens.
    #[must_use]
    pub const fn how_it_unlocks(&self) -> &HowItUnlocks {
        &self.how
    }

    /// That the person kept the recovery key.
    ///
    /// Reachable so that whatever finishes an install can say it in its own
    /// words, and so that a managed machine's escrow has somewhere to put its
    /// own witness beside this one at v1
    /// ([ADR 0004](../../../docs/decisions/0004-the-organisations-machine.md)).
    #[must_use]
    pub const fn the_person_kept_the_key(&self) -> WrittenDown {
        self.kept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chip::TheChip;
    use crate::pin::Pin;
    use crate::recovery_key::RecoveryKey;

    /// What the rented tool printed on a virtual disk in the pinned base.
    const AS_THE_TOOL_PRINTED_IT: &str =
        "lrhdhrji-vtdlgdkr-ulukcjff-rgckhvrd-kguhibgc-gvihrfgc-ghedrdlr-uihctkkt\n";

    /// **The whole road, walked**: a volume, a key from the tool, a person who
    /// typed it back, a PIN sealed to a ready chip, and an enrolment at the end
    /// of it.
    #[test]
    fn an_enrolment_is_made_from_a_kept_key_and_the_way_it_unlocks() {
        let Ok(key) = RecoveryKey::as_printed(AS_THE_TOOL_PRINTED_IT) else {
            unreachable!("the tool's own output is a recovery key")
        };
        let shown = key.as_it_is_shown().to_owned();
        let Ok(kept) = key.written_back(&shown) else {
            unreachable!("the key typed back exactly is the key")
        };
        let Ok(pin) = Pin::typed("123456", "123456") else {
            unreachable!("six characters twice is a PIN")
        };
        let Ok(how) = HowItUnlocks::sealed_to_the_chip(TheChip::ReadyToUse, pin) else {
            unreachable!("a chip that is ready can hold a key")
        };
        let enrolment = Enrolment::made(how, kept);
        assert!(enrolment.how_it_unlocks().is_sealed_to_the_chip());
        assert_eq!(enrolment.the_person_kept_the_key(), kept);
    }

    /// **No secret is in the line that describes an encrypted machine.**
    #[test]
    fn an_enrolment_says_nothing_about_the_secrets_in_it() {
        let Ok(key) = RecoveryKey::as_printed(AS_THE_TOOL_PRINTED_IT) else {
            unreachable!("the tool's own output is a recovery key")
        };
        let shown = key.as_it_is_shown().to_owned();
        let Ok(kept) = key.written_back(&shown) else {
            unreachable!("the key typed back exactly is the key")
        };
        let Ok(pin) = Pin::typed("123456", "123456") else {
            unreachable!("six characters twice is a PIN")
        };
        let Ok(how) = HowItUnlocks::sealed_to_the_chip(TheChip::ReadyToUse, pin) else {
            unreachable!("a chip that is ready can hold a key")
        };
        let said = format!("{:?}", Enrolment::made(how, kept));
        assert!(!said.contains("123456"), "{said}");
        assert!(!said.contains("lrhdhrji"), "{said}");
    }
}
