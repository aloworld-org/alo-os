//! The four ways a credential is not handed over, and none of them is a
//! fallback.
//!
//! ADR 0022 names them and requires that each be told apart, because they are
//! four different things for a person to do: sign in, unlock, add the key, or
//! find out why the store said no. A single *it did not work* would send
//! somebody to check a key that is correct.
//!
//! **Nothing here carries a credential, and nothing here carries what the store
//! said either.** A keyring's own error text is not in anybody's language and
//! may quote what it was asked about; `alo_choosing::NotToml` is the same
//! argument one file to the left, and `41c9f1e` is the day it stopped being
//! theoretical.

/// Why a key was not handed over.
///
/// **No `Display`**, for `alo_models::SecretError`'s reason: a type in this part
/// of the system with one is a sentence one `to_string()` away from a log line
/// written by somebody who never thought about credentials.
///
/// **And no words yet**, deliberately. Four sentences a person reads belong
/// beside the daemon that says them, and there is nothing here to say them
/// about until a lookup exists — a vocabulary declared now would be four strings
/// nothing renders, which `alo-strings` would carry and no translator could
/// place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotStored {
    /// There is no store to ask.
    ///
    /// Before anybody has signed in there is no `/run/user/<uid>`, and on a
    /// machine whose image ships no Secret Service there is nothing behind the
    /// bus. Both are this: **nothing was asked and nothing was sent.**
    Unavailable,

    /// The store is there and will not open.
    ///
    /// A locked keyring is a person's own decision most of the time — they have
    /// not typed their password yet — and it is a different thing to do about it
    /// than a missing key.
    Locked,

    /// The store is open and has nothing under this name.
    ///
    /// The provider is configured and its key was never added, or was added
    /// under another name. Neither is an error in the store.
    Missing,

    /// The store refused this caller.
    ///
    /// **Never retried anywhere else.** A refusal that went looking for a second
    /// store would be the worst reading of ADR 0008's *never a silent fallback*,
    /// with a credential attached.
    Denied,
}

impl NotStored {
    /// Whether anything at all was sent to the provider, which is **never**.
    ///
    /// A method rather than a comment because it is the claim the whole type
    /// exists to make, and a caller that has to remember it is a caller that
    /// one day will not. Every one of these happens *before* a question is put:
    /// there is no state here that means *it went and came back*.
    #[must_use]
    pub const fn nothing_was_sent(self) -> bool {
        match self {
            Self::Unavailable | Self::Locked | Self::Missing | Self::Denied => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Nothing was sent, in all four**, and the match has no wildcard: a fifth
    /// state does not fail this at runtime, it fails to compile it, and whoever
    /// adds one says whether a question had already left by then.
    #[test]
    fn no_refusal_here_has_already_sent_anything() {
        for refused in [
            NotStored::Unavailable,
            NotStored::Locked,
            NotStored::Missing,
            NotStored::Denied,
        ] {
            assert!(refused.nothing_was_sent(), "{refused:?}");
        }
    }

    /// **And the four are told apart**, because they are four different things
    /// for a person to do about them. A store that answered *it did not work*
    /// would send somebody to check a key that is correct.
    #[test]
    fn the_four_are_four_and_not_one() {
        let all = [
            NotStored::Unavailable,
            NotStored::Locked,
            NotStored::Missing,
            NotStored::Denied,
        ];
        for (which, one) in all.iter().enumerate() {
            for (other, another) in all.iter().enumerate() {
                assert_eq!(
                    which == other,
                    one == another,
                    "{one:?} and {another:?} are not told apart"
                );
            }
        }
    }
}
