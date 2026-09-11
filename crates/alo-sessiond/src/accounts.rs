//! The other thing this crate asks the machine, and it is one question with a
//! yes-or-no answer.
//!
//! **Whether this machine has an account with this number.** Not who they are,
//! not what they are called, and emphatically not whether a password is right:
//! `alo-accounts` authenticates and this component does not, which is ADR 0018's
//! *one privileged component* argument kept where it is easiest to lose. A
//! privileged process that could verify a password is a privileged process with
//! a password on the way into it.
//!
//! # Why it is asked at all
//!
//! Because it is what makes *it cannot be asked to open a session for a number
//! the caller has not authenticated* true across a process boundary. The
//! surface authenticated somebody against `/etc/alo/accounts.toml`; this asks
//! the same file whether the number on the wire is one of the accounts in it.
//! The set of numbers this component will ever open a session for is therefore
//! exactly the set of people this machine has — a number nobody here could have
//! signed in as is refused before `logind` is asked anything at all.
//!
//! # And why it is asked every time
//!
//! An account made after this process started is an account this machine has.
//! A snapshot taken at start-up would refuse the first person to use a machine
//! that shipped with no accounts — which is every machine, because ADR 0024
//! settled that the image ships none and `crates/alo-image` tests it.

use crate::refusing::NotAsked;

/// The accounts this machine has, asked one number at a time.
pub trait TheAccounts {
    /// Whether this machine has an account with this number.
    ///
    /// # Errors
    ///
    /// [`NotAsked`] when the file would not read. A machine that cannot say who
    /// it has accounts for is not a machine that has none: the difference is
    /// the whole reason this answers with a `Result` rather than with `false`,
    /// because the second would turn an unreadable file into a person being
    /// told they do not exist.
    fn an_account_numbered(&self, person: u32) -> Result<bool, NotAsked>;
}
