//! What a person may call a machine this one is paired with.
//!
//! A pairing names the other machine by its identity — thirty-two hexadecimal
//! characters nobody chose — and a list of those is a list a person cannot
//! read. So the person gives each machine a name, and this file is the rule a
//! name is held to. It is one rule for the person's door and for the file the
//! names are kept in: a name the door would refuse is a name the file is
//! refused for holding.
//!
//! # A name decides nothing, so the rule is about what a person reads
//!
//! The identity is what a pairing, a proof and a grant are about (ADR 0003,
//! ADR 0031); a name is only ever shown. Nothing in the rule widens anything,
//! and nothing in it could: what it refuses is a name that would make what the
//! person reads untrue or unreadable.
//!
//! - **Something in it.** An empty name is not a name, and a machine with none
//!   is spoken of by its identity.
//! - **At most [`LONGEST_NAME`] characters.** A name is a label on a list, on
//!   an indicator and in a record entry, and a name that fills a screen is a
//!   name that hides what is beside it.
//! - **No control characters** — no line break, no tab, no escape. A record
//!   entry and an indicator are read a line at a time, and a name with a line
//!   break in it is a name that can draw a second line nobody wrote.
//! - **Not a machine's identity**, spelt bare or as a grantee
//!   (`machine:<identity>`). A person reading an identity reads *that*
//!   machine; a name spelt as another machine's identity would put one
//!   machine's name on another's evidence.

use alo_nearby::MachineId;

/// The most characters a machine's name may hold.
pub const LONGEST_NAME: usize = 64;

/// What a grantee naming a machine begins with, which a name may not borrow.
const A_MACHINE: &str = "machine:";

/// Why something is not a name a person may give a machine.
///
/// English, for this crate's reason: what reaches the person is the daemon's
/// sentence for each arm, in their own language. This is what a service log
/// says about a names file somebody edited into holding one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAName {
    /// Nothing but white space.
    #[error("a machine's name has nothing in it")]
    Empty,
    /// Longer than [`LONGEST_NAME`] characters.
    #[error("a machine's name is {characters} characters, and a name holds at most {LONGEST_NAME}")]
    TooLong {
        /// How many characters it held.
        characters: usize,
    },
    /// A line break, a tab or another control character.
    #[error("a machine's name holds a control character, such as a line break")]
    AControlCharacter,
    /// A machine's identity, bare or as a grantee.
    #[error("a machine's name reads as a machine's identity")]
    AnIdentity,
}

/// A name a person gave a machine, held to the rule above.
///
/// Made only by [`MachineName::checked`], so holding one is holding a name the
/// rule allowed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineName(String);

impl MachineName {
    /// This text as a name, trimmed, or why it is not one.
    ///
    /// # Errors
    ///
    /// [`NotAName`], one arm for each part of the rule, asked in the order the
    /// module's header gives it.
    pub fn checked(said: &str) -> Result<Self, NotAName> {
        let name = said.trim();
        if name.is_empty() {
            return Err(NotAName::Empty);
        }
        let characters = name.chars().count();
        if characters > LONGEST_NAME {
            return Err(NotAName::TooLong { characters });
        }
        if name.chars().any(char::is_control) {
            return Err(NotAName::AControlCharacter);
        }
        // Asked in lower case, because a person reading an identity in
        // capitals reads the same machine.
        let lower = name.to_ascii_lowercase();
        let bare = lower.strip_prefix(A_MACHINE).unwrap_or(&lower).trim();
        if MachineId::read(bare).is_ok() {
            return Err(NotAName::AnIdentity);
        }
        Ok(Self(name.to_owned()))
    }

    /// The name, as the person gave it, trimmed.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MachineName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An ordinary name is a name**, trimmed, in any script a person writes
    /// in.
    #[test]
    fn an_ordinary_name_in_any_language_is_a_name() {
        for said in [
            "the studio machine",
            "  Anna's laptop ",
            "Χαρτογράφος",
            "Kontor – Empfang",
        ] {
            assert_eq!(MachineName::checked(said).unwrap().as_str(), said.trim());
        }
        let longest = "a".repeat(LONGEST_NAME);
        assert!(MachineName::checked(&longest).is_ok());
    }

    /// **Each part of the rule refuses in its own arm**: nothing in it, too
    /// long, a line break, and a machine's identity bare, as a grantee and in
    /// capitals.
    #[test]
    fn each_part_of_the_rule_refuses_in_its_own_words() {
        assert_eq!(MachineName::checked("   "), Err(NotAName::Empty));
        assert_eq!(
            MachineName::checked(&"é".repeat(LONGEST_NAME + 1)),
            Err(NotAName::TooLong {
                characters: LONGEST_NAME + 1
            })
        );
        for said in ["the studio\nmachine", "tab\there", "bell\u{7}"] {
            assert_eq!(
                MachineName::checked(said),
                Err(NotAName::AControlCharacter),
                "{said:?}"
            );
        }
        for said in [
            "0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            "machine:0f1e2d3c4b5a69788796a5b4c3d2e1f0",
            "MACHINE:0F1E2D3C4B5A69788796A5B4C3D2E1F0",
        ] {
            assert_eq!(
                MachineName::checked(said),
                Err(NotAName::AnIdentity),
                "{said}"
            );
        }
    }
}
