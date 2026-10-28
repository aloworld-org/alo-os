//! The third thing this crate asks the machine to do, one method wide: give a
//! person their own name, where their desktop can read it.
//!
//! [ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)
//! is the decision. The name a person typed when they made their account is in
//! the account store, which their desktop must not read, and the desktop shows
//! it — the terminal's strip says `Local · anna`. So at every sign-in a small
//! privileged unit, `alo-giving-a-name@<number>.service`, copies that one name
//! into a file only that person can read.
//!
//! # No name reaches this component
//!
//! What crosses here is the **number**, as it does for
//! [`crate::their_folder::TheirFolder`]. The unit reads the name from the store
//! itself. This crate's header promises *no password reaches it, and no name*,
//! and that is still true: it asks for a name to be given without ever holding
//! one.
//!
//! # Why a failure here is not a refusal
//!
//! [`Opening::heard`](crate::Opening::heard) signs the person in whether or not
//! their name was given, for [`crate::their_folder`]'s reasons and one of its
//! own: **the cost of failing is a word**. A desktop with no name file shows
//! `Local` alone where it would have shown `Local · anna`, which is what every
//! machine showed before this existed. Refusing the sign-in over it would take
//! the machine away from its owner to protect a label — `CLAUDE.md` law 5.
//!
//! So the failure goes to the journal, and
//! `a_name_that_was_not_given_refuses_nobody` holds it.

use crate::refusing::NotTheirs;

/// What can give a person their own name, where their desktop can read it.
pub trait TheirName {
    /// Give account `person` their name, as the account store has it.
    ///
    /// **Asked at every sign-in**, so a name changed in the store reaches the
    /// desktop at the next one, and asking again for an unchanged name writes
    /// the same line.
    ///
    /// # Errors
    ///
    /// [`NotTheirs`], which is every way the machine can answer no. The caller
    /// writes it to the journal and signs the person in anyway.
    fn given_to(&self, person: u32) -> Result<(), NotTheirs>;
}
