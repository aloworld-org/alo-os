//! **What pairing a device grants on this machine, which is nothing.**
//!
//! This file has no logic in it and it is the most important one in the crate.
//! It exists because two things in alo OS are called *pairing* and only one of
//! them is authority:
//!
//! | | What it is | What it grants |
//! |---|---|---|
//! | `alo-nearby`'s pairing | two **machines**, each person agreeing on their own machine (ADR 0003) | an agent on one may ask the other, under grants made on the machine it acts upon |
//! | this crate's pairing | a **headset**, a keyboard, a mouse | nothing |
//!
//! A headset is not a machine. It holds no keys of this machine's, it appears on
//! no list of what may reach a person's files, and a grant in the sense of
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md) or
//! [ADR 0031](../../../docs/decisions/0031-the-pairing-is-the-key.md)
//! is not created, widened or implied by pairing one. **The danger is the one
//! list**: if a person ever sees their headphones on the same list as a machine
//! they paired with, the word *paired* has stopped meaning anything, and the
//! list that says who may reach this machine is the last list in the product
//! that may become vague.
//!
//! What pairing a device does do is decide **what it is for** — sound out and
//! in, typing, pointing — which `crate::reported::Kind` says and which is not a
//! permission. A keyboard that types is a keyboard; this machine does not
//! thereby let it read anything.
//!
//! `tests/a_bluetooth_pairing_grants_nothing.rs` holds it against the two crates
//! that own the other meanings, by pairing a device and finding both untouched.

/// **What a Bluetooth pairing grants on this machine.**
///
/// There is one value, it has no fields, and nothing in this crate returns
/// anything else. A later change that wants a device to be able to do something
/// has to add a variant here, which is a change somebody will notice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum WhatAPairingGrants {
    /// Nothing at all.
    #[default]
    Nothing,
}

impl WhatAPairingGrants {
    /// What pairing the device in front of you grants.
    #[must_use]
    pub const fn pairing_one() -> Self {
        Self::Nothing
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Pairing a device grants nothing**, whatever the device is.
    #[test]
    fn pairing_a_device_grants_nothing() {
        assert_eq!(
            WhatAPairingGrants::pairing_one(),
            WhatAPairingGrants::Nothing
        );
        assert_eq!(WhatAPairingGrants::default(), WhatAPairingGrants::Nothing);
    }
}
