//! The second thing this crate asks the machine to do, one method wide.
//!
//! A person signing in needs the directory that holds what an undo would put
//! back, and **only a privileged process can make it**: it lives in a folder
//! root owns, and giving a directory away needs `CAP_CHOWN`
//! ([ADR 0119](../../../docs/decisions/0119-the-capability-to-give-a-directory-away-goes-to-the-smallest-thing-that-can-hold-it.md),
//! [ADR 0120](../../../docs/decisions/0120-the-undo-folder-is-roots-own-and-not-inside-the-persons.md)).
//! The session cannot make it, and image build time does not know who the
//! people will be — which is
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) term 8's
//! own reasoning for why a privileged step at sign-in is where it goes.
//!
//! This is a trait for the reason [`crate::TheirDesktop`] is: whether the
//! folder is asked for, and in what order, is arithmetic on numbers that the
//! workspace gate can run on any host. `crate::starting_their_units` is the
//! half that is really systemd over the system bus.
//!
//! # It holds nothing, so it is `&self`
//!
//! This asks for something to be made and then has nothing. A directory
//! outlives every process involved, so `&self`.
//!
//! # Why a failure here is not a refusal
//!
//! [`Opening::heard`](crate::Opening::heard) does **not** turn a person away
//! when their folder cannot be made. Three reasons, and the third is the one
//! that decides it:
//!
//! 1. **The folder is not what they came for.** They came to use their machine.
//!    A sign-in refused because a directory for a feature nothing yet writes to
//!    could not be made is a machine that will not let somebody in over
//!    something they have never heard of.
//! 2. **There is already an honest answer for a missing folder.**
//!    `alo-letting-go` answers *not yet on this machine* when there is nothing
//!    kept, and ADR 0045's sixth term makes that a real answer rather than an
//!    error. A person whose folder is missing is in exactly the state of a
//!    person on a machine installed before the filesystem was decided.
//! 3. **`CLAUDE.md` law 5.** Refusing the sign-in would take the machine away
//!    from its owner to protect a feature, and *a change that takes a choice
//!    away from the person is a bug, whatever the reason given for it.*
//!
//! So the refusal goes to the journal, where whoever administers the machine
//! reads it, and the person signs in. **That is a decision and not an
//! oversight**, which is why it is written here and tested in
//! `crate::opening`'s own tests rather than left to be inferred from a `let _`.

use crate::refusing::NotTheirs;

/// What can make a person their place for what an undo would put back.
pub trait TheirFolder {
    /// Make account `person` their directory, and give it to them.
    ///
    /// **Asking twice is not an error.** A person signs in more than once, and
    /// what the privileged step does on a second call is put the mode and the
    /// owner back — so this is called at every sign-in rather than guarded by
    /// something remembering whether it has run. That is what makes one call
    /// site enough for both the first account and every account that already
    /// existed, and the reason `alo-giving-a-folder` was built to repair rather
    /// than to refuse.
    ///
    /// # Errors
    ///
    /// [`NotTheirs`], which is every way the machine can answer no. The caller
    /// writes it to the journal and signs the person in anyway — see this
    /// module's own documentation for why.
    fn made_for(&self, person: u32) -> Result<(), NotTheirs>;
}
