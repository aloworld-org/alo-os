//! What this machine has in the way of a security chip, and whether a key may
//! be sealed to it.
//!
//! The installer already reads this before alo OS exists on the machine —
//! `alo_installer::SecurityChip` asks Windows the same question, in the same
//! four answers — and this is the same fact on the other side of the install,
//! written again rather than borrowed: that crate is the program a person runs
//! on the machine they are replacing, and a type in it is not a type about the
//! machine alo OS is on. The four answers agree deliberately, so that whoever
//! wires them together has nothing to translate.
//!
//! The decision this serves is [ADR 0054](../../../docs/decisions/0054-the-disk-is-sealed-to-this-machine-and-opened-with-a-pin.md):
//! *a chip we cannot see is a chip we will not make the only place the key
//! lives*. Three of the four answers therefore mean the same thing here.

/// What this machine has in the way of a security chip.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheChip {
    /// There is one, and it is ready to be used.
    ReadyToUse,
    /// There is one, and it is not ready — never initialised, owned by
    /// something else, or in a state the machine will not use it from.
    NotReady,
    /// There is none.
    Absent,
    /// Nothing on this machine would say. A chip that cannot be asked about is
    /// not a chip a key is given to.
    NotRead,
}

impl TheChip {
    /// Whether the disk's key may be sealed to it.
    ///
    /// True for exactly one of the four. The other three are *no* for the same
    /// reason and are kept apart because what a person is told about a machine
    /// with no chip is not what they are told about a chip that is not ready.
    #[must_use]
    pub fn can_hold_a_key(self) -> bool {
        matches!(self, Self::ReadyToUse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// One of the four answers can hold a key, and it is the one that says the
    /// chip is ready.
    #[test]
    fn only_a_chip_that_is_ready_holds_a_key() {
        assert!(TheChip::ReadyToUse.can_hold_a_key());
        for chip in [TheChip::NotReady, TheChip::Absent, TheChip::NotRead] {
            assert!(!chip.can_hold_a_key(), "{chip:?}");
        }
    }
}
