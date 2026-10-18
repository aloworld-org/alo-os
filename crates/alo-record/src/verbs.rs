//! The record's one verb, declared.
//!
//! `docs/features.md` promises *find a command by what it did*, and the owner
//! decided on 2026-10-18 which surface asks it: **a question to alo**, which
//! alo answers from the record. An agent answering a question about the record
//! needs a verb, and `docs/contracts/agent-verbs.md` had none — so this file
//! is that verb, declared the way the contract says a verb has to be and the
//! way `alo-files` declares its six: a name, a purpose, an effect, typed
//! arguments, the grant it requires and the sentence a person reads.
//!
//! **Nothing is declared by default.** [`record_verbs`] hands back a list;
//! [`declare_into`] puts it on somebody else's. A registry nobody gave this to
//! cannot be asked anything about the record, which is the honest starting
//! state.
//!
//! # One verb, and why it is a read over a granted file
//!
//! [`crate::what_touched_a_file::THE_VERB`] answers a question and changes
//! nothing, so it is [`Effect::Read`]: it runs inside the turn, nobody is
//! asked to approve it, and the record's entry for it names no approval.
//!
//! It still requires a grant, over the file the question names — *read inside
//! the turn* is about approval and never about reach.
//! [`crate::what_touched_a_file`]'s header says at length why: an agent that
//! could ask about any path would learn whether a path is in the record by
//! asking about a guess, and that is an answer about a place nobody granted
//! it.
//!
//! # Why the argument is a path and not part of a name
//!
//! `Asking::touching` matches a recorded argument **exactly**. A part of a
//! name would have to be matched against paths, which is a search, and a
//! search over the record is a different promise with a different surface —
//! the record window, which refuses to narrow by design. One question, asked
//! about one file.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::what_touched_a_file::{THE_FILE, THE_VERB};
use crate::words;

/// Why the record's verb could not be declared.
///
/// Neither can happen to the verb as it is written — the tests in this file
/// say so. It is a `Result` because a library that panics on its own
/// declaration takes the daemon with it, and because [`declare_into`] can
/// genuinely fail against a list that already holds the name.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The record's verb, as a list of its own.
///
/// ```
/// use alo_capability::Given;
/// use alo_record::{record_verbs, record_words};
/// use alo_strings::Strings;
///
/// let verbs = record_verbs()?;
/// let call = verbs
///     .call("what_touched_this_file", &[("file", Given::text("/home/anna/invoice.pdf"))])
///     .expect("the verb takes one file");
///
/// let strings = Strings::of(record_words()?);
/// assert_eq!(
///     call.sentence(&strings).text(),
///     "look in the record for what was done to /home/anna/invoice.pdf",
/// );
/// // A read: it answers inside the turn, and nobody is asked to approve it.
/// assert!(!call.waits_for_approval());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// [`Declaring`], which the verb as written cannot cause.
pub fn record_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the record's verb on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already holds the name — a name means one
/// thing, so whoever took it first keeps it and nothing is silently replaced.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let asking = what_touched_this_file()?;
    if verbs.of(asking.name()).is_some() {
        return Err(Declaring::List(VerbsError::AlreadyDeclared {
            name: asking.name().to_owned(),
        }));
    }
    verbs.declare(asking)?;
    Ok(())
}

/// Look in the record for what was done to one file.
fn what_touched_this_file() -> Result<Verb, VerbError> {
    Verb::checked(
        THE_VERB,
        words::WHAT_TOUCHED_THIS_FILE,
        Effect::Read,
        vec![Arg::taking(
            THE_FILE,
            words::WHAT_TOUCHED_THIS_FILE_FILE,
            Takes::Path,
        )],
        Requires::grants_over([THE_FILE]),
        words::WHAT_TOUCHED_THIS_FILE_SENTENCE,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "verbs_tests.rs"]
mod tests;
