//! One machine the person has added, and what they call it.
//!
//! The identity is [`alo_nearby::MachineId`] and is not restated here. That
//! crate's own record argues why it is random rather than derived — a serial
//! outlives a reinstall and is therefore a tracker, and a hostname a person
//! chose is a person's name on every network they carry it onto. A second
//! spelling of *which machine* in this crate would be a second thing to keep in
//! step with that argument.
//!
//! What this crate adds is the half that is the person's: **the name they gave
//! it.** That name never crosses the wire. It is how they tell *the one in the
//! studio* from *the one at home*, and it is exactly the kind of thing ADR 0003
//! keeps off a local network.

use alo_nearby::MachineId;

use crate::refusing::NotElsewhere;

/// The longest name a person may give one of their machines.
///
/// Not a technical limit — nothing here is a DNS label, because this name never
/// leaves the machine. It is the point past which a name stops being a name and
/// starts being a sentence, and a list of them stops being scannable.
pub const AT_MOST: usize = 64;

/// What the person calls one of their machines.
///
/// **Never sent anywhere.** It is written down on this machine, beside the
/// identity, and read back by the person who wrote it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TheName(String);

impl TheName {
    /// The name a person typed, with the spaces either side taken off.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::Unnamed`] when nothing is left after trimming, because a
    /// machine with no name is one the person cannot tell from another, and
    /// [`NotElsewhere::NameTooLong`] past [`AT_MOST`].
    pub fn given(said: &str) -> Result<Self, NotElsewhere> {
        let trimmed = said.trim();
        if trimmed.is_empty() {
            return Err(NotElsewhere::Unnamed);
        }
        let how_long = trimmed.chars().count();
        if how_long > AT_MOST {
            return Err(NotElsewhere::NameTooLong {
                how_long,
                at_most: AT_MOST,
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// The name, as the person wrote it.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A machine the person has added to their own machines.
///
/// Holding the identity and the name together is what makes it impossible to
/// show a person a list of identities: there is no machine on this list without
/// a name they gave it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AMachine {
    /// Which machine, which is `alo-nearby`'s identity and crosses the wire.
    which: MachineId,
    /// What the person calls it, which never leaves this machine.
    called: TheName,
}

impl AMachine {
    /// A machine, added.
    #[must_use]
    pub const fn added(which: MachineId, called: TheName) -> Self {
        Self { which, called }
    }

    /// Which machine this is.
    #[must_use]
    pub const fn which(&self) -> &MachineId {
        &self.which
    }

    /// What the person calls it.
    #[must_use]
    pub const fn called(&self) -> &TheName {
        &self.called
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn an_identity() -> MachineId {
        MachineId::read("0123456789abcdef0123456789abcdef")
            .expect("thirty-two hexadecimal characters is an identity")
    }

    /// **A name is the person's, so what they typed is what they get** — minus
    /// the spaces either side, which are a typing accident rather than a choice.
    #[test]
    fn the_spaces_either_side_are_not_part_of_the_name() {
        let name = TheName::given("  the one in the studio  ").expect("a name");
        assert_eq!(name.as_str(), "the one in the studio");
    }

    /// A machine with no name is one the person cannot tell from another.
    #[test]
    fn a_machine_cannot_be_added_without_a_name() {
        assert_eq!(TheName::given("   "), Err(NotElsewhere::Unnamed));
        assert_eq!(TheName::given(""), Err(NotElsewhere::Unnamed));
    }

    /// The limit counts characters rather than bytes, or a European name would
    /// be refused earlier than an English one of the same length.
    #[test]
    fn the_limit_is_in_characters_not_bytes() {
        let sixty_four = "é".repeat(AT_MOST);
        assert!(TheName::given(&sixty_four).is_ok(), "{AT_MOST} characters");

        let one_too_many = "é".repeat(AT_MOST + 1);
        assert_eq!(
            TheName::given(&one_too_many),
            Err(NotElsewhere::NameTooLong {
                how_long: AT_MOST + 1,
                at_most: AT_MOST,
            })
        );
    }

    /// There is no machine on this list without a name the person gave it.
    #[test]
    fn a_machine_carries_its_name_with_it() {
        let machine = AMachine::added(an_identity(), TheName::given("at home").expect("a name"));
        assert_eq!(machine.called().as_str(), "at home");
        assert_eq!(machine.which().as_str(), "0123456789abcdef0123456789abcdef");
    }
}
