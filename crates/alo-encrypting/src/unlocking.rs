//! Which of the two roads this machine takes, and the one secret it asks for.
//!
//! [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md):
//! **the machine chooses, not the person.** A chip that is present and ready
//! means a PIN; anything else means a passphrase. The person is asked for one
//! secret and is never asked what LUKS is, what a TPM is, or which of them is
//! holding anything — which is what *enrols it during install without a person
//! understanding LUKS* means in practice.
//!
//! [`HowItUnlocks`] is a struct rather than an enum on purpose. A public enum's
//! variants are public constructors, and *sealed to a chip* is a sentence that
//! must not be constructible on a machine with no chip to seal to.

use std::fmt;

use crate::chip::TheChip;
use crate::passphrase::Passphrase;
use crate::pin::Pin;

/// The platform configuration registers the key is sealed against.
///
/// **Register 7 and nothing else.** It records what the firmware's Secure Boot
/// policy was, which is what ties the key to this chip in this machine — a disk
/// pulled out of a laptop is bytes. It is not what protects a machine somebody
/// switches on: the PIN is, and ADR 0054 says so at length rather than letting
/// a reader assume otherwise.
///
/// Binding to more would be what makes an update lock a person out, and the
/// alternative systemd offers for a boot chain that changes — a signed policy
/// over register 11 — needs a unified kernel image, which the pinned base does
/// not build. Both halves of that are measured in ADR 0054.
pub const THE_PCRS_IT_IS_SEALED_AGAINST: [u8; 1] = [7];

/// What the person is asked for on this machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatToAskFor {
    /// A PIN. The chip holds the key and counts the wrong answers.
    APin,
    /// A passphrase, typed at every start, because no chip here can hold a key.
    APassphrase,
}

impl WhatToAskFor {
    /// What to ask for, on a machine with this chip.
    #[must_use]
    pub fn on_a_machine_with(chip: TheChip) -> Self {
        if chip.can_hold_a_key() {
            Self::APin
        } else {
            Self::APassphrase
        }
    }
}

/// How this machine's disk opens.
pub struct HowItUnlocks {
    /// One of the two roads, kept private so that the sentence *sealed to a
    /// chip* is only ever written where there was a chip to seal to.
    which: Which,
}

/// The two roads, and there is no third.
enum Which {
    /// Sealed to the machine's own security chip and released when the person
    /// types their PIN.
    SealedToTheChipWithAPin(Pin),
    /// Typed at every start.
    APassphraseTypedAtEveryStart(Passphrase),
}

/// There was no chip to seal the key to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoChipToSealTo {
    /// What the machine has, which is one of the three that cannot hold a key.
    chip: TheChip,
}

impl NoChipToSealTo {
    /// What the machine has.
    #[must_use]
    pub const fn chip(&self) -> TheChip {
        self.chip
    }
}

impl fmt::Display for NoChipToSealTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the disk's key cannot be sealed to this machine's security chip: {:?}",
            self.chip
        )
    }
}

impl std::error::Error for NoChipToSealTo {}

impl HowItUnlocks {
    /// Sealed to the chip, released when the PIN is typed.
    ///
    /// # Errors
    /// [`NoChipToSealTo`], where the machine has no chip that can hold a key.
    /// A PIN on such a machine is a PIN that opens nothing, and this is the
    /// refusal that stops one being asked for.
    pub fn sealed_to_the_chip(chip: TheChip, pin: Pin) -> Result<Self, NoChipToSealTo> {
        if !chip.can_hold_a_key() {
            return Err(NoChipToSealTo { chip });
        }
        Ok(Self {
            which: Which::SealedToTheChipWithAPin(pin),
        })
    }

    /// Opened by a passphrase typed at every start.
    ///
    /// Always available. A machine with a chip that is ready takes the other
    /// road under ADR 0054, and this one stays reachable because the road a
    /// machine takes is [`WhatToAskFor::on_a_machine_with`]'s to decide and not
    /// this constructor's to police — an installer that has to recover from a
    /// chip that stopped answering halfway through is task 6's problem and it
    /// should not have to fight the types to do it.
    #[must_use]
    pub fn a_passphrase(passphrase: Passphrase) -> Self {
        Self {
            which: Which::APassphraseTypedAtEveryStart(passphrase),
        }
    }

    /// Whether the key is sealed to the machine's chip.
    #[must_use]
    pub fn is_sealed_to_the_chip(&self) -> bool {
        matches!(&self.which, Which::SealedToTheChipWithAPin(_))
    }

    /// What the person was asked for.
    #[must_use]
    pub fn what_was_asked_for(&self) -> WhatToAskFor {
        match &self.which {
            Which::SealedToTheChipWithAPin(_) => WhatToAskFor::APin,
            Which::APassphraseTypedAtEveryStart(_) => WhatToAskFor::APassphrase,
        }
    }

    /// The PIN, where this machine has one.
    #[must_use]
    pub fn pin(&self) -> Option<&Pin> {
        match &self.which {
            Which::SealedToTheChipWithAPin(pin) => Some(pin),
            Which::APassphraseTypedAtEveryStart(_) => None,
        }
    }

    /// The passphrase, where this machine has one.
    #[must_use]
    pub fn passphrase(&self) -> Option<&Passphrase> {
        match &self.which {
            Which::APassphraseTypedAtEveryStart(passphrase) => Some(passphrase),
            Which::SealedToTheChipWithAPin(_) => None,
        }
    }
}

impl fmt::Debug for HowItUnlocks {
    /// Says which road, and never the secret on it. The secrets have their own
    /// silent `Debug`, and this one does not rely on them having it.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.which {
            Which::SealedToTheChipWithAPin(_) => f.write_str("SealedToTheChipWithAPin"),
            Which::APassphraseTypedAtEveryStart(_) => f.write_str("APassphraseTypedAtEveryStart"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PIN long enough to be one.
    fn a_pin() -> Pin {
        match Pin::typed("123456", "123456") {
            Ok(pin) => pin,
            Err(why) => unreachable!("six characters twice is a PIN: {why}"),
        }
    }

    /// A passphrase long enough to be one.
    fn a_passphrase() -> Passphrase {
        match Passphrase::typed("correct horse battery", "correct horse battery") {
            Ok(passphrase) => passphrase,
            Err(why) => unreachable!("twenty-one characters twice is a passphrase: {why}"),
        }
    }

    /// **The machine chooses**: a ready chip means a PIN, and the other three
    /// answers mean a passphrase.
    #[test]
    fn a_ready_chip_means_a_pin_and_anything_else_means_a_passphrase() {
        assert_eq!(
            WhatToAskFor::on_a_machine_with(TheChip::ReadyToUse),
            WhatToAskFor::APin
        );
        for chip in [TheChip::NotReady, TheChip::Absent, TheChip::NotRead] {
            assert_eq!(
                WhatToAskFor::on_a_machine_with(chip),
                WhatToAskFor::APassphrase,
                "{chip:?}"
            );
        }
    }

    /// A key is sealed to a chip that is ready, and the secret comes back out
    /// for the rented tool to be given.
    #[test]
    fn a_ready_chip_takes_the_key_and_the_pin() {
        let Ok(how) = HowItUnlocks::sealed_to_the_chip(TheChip::ReadyToUse, a_pin()) else {
            unreachable!("a chip that is ready can hold a key")
        };
        assert!(how.is_sealed_to_the_chip());
        assert_eq!(how.what_was_asked_for(), WhatToAskFor::APin);
        assert_eq!(
            how.pin().map(Pin::as_the_person_typed_it),
            Some("123456"),
            "the rented tool has to be given it"
        );
        assert!(how.passphrase().is_none());
    }

    /// **No key is sealed to a chip that cannot hold one**, whichever of the
    /// three reasons it cannot.
    #[test]
    fn no_key_is_sealed_to_a_chip_that_cannot_hold_one() {
        for chip in [TheChip::NotReady, TheChip::Absent, TheChip::NotRead] {
            let refused = HowItUnlocks::sealed_to_the_chip(chip, a_pin());
            match refused {
                Ok(_) => unreachable!("{chip:?} sealed a key"),
                Err(why) => {
                    assert_eq!(why.chip(), chip);
                    assert!(why.to_string().contains("cannot be sealed"));
                }
            }
        }
    }

    /// A passphrase is the other road, and carries no PIN.
    #[test]
    fn a_passphrase_is_the_other_road() {
        let how = HowItUnlocks::a_passphrase(a_passphrase());
        assert!(!how.is_sealed_to_the_chip());
        assert_eq!(how.what_was_asked_for(), WhatToAskFor::APassphrase);
        assert!(how.pin().is_none());
        assert_eq!(
            how.passphrase().map(Passphrase::as_the_person_typed_it),
            Some("correct horse battery")
        );
    }

    /// **Neither secret is in the line that mentions how the disk opens.**
    #[test]
    fn neither_secret_is_in_the_debug_of_how_it_unlocks() {
        let Ok(sealed) = HowItUnlocks::sealed_to_the_chip(TheChip::ReadyToUse, a_pin()) else {
            unreachable!("a chip that is ready can hold a key")
        };
        let said = format!("{sealed:?}");
        assert!(!said.contains("123456"), "{said}");
        let typed = format!("{:?}", HowItUnlocks::a_passphrase(a_passphrase()));
        assert!(!typed.contains("correct horse"), "{typed}");
    }

    /// One register, and it is register 7.
    #[test]
    fn the_key_is_sealed_against_register_seven_and_nothing_else() {
        assert_eq!(THE_PCRS_IT_IS_SEALED_AGAINST, [7]);
    }
}
