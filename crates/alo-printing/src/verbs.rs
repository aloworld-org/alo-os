//! The printing verb, declared.
//!
//! `docs/features.md` promises printing at v0.5, and ★ *Printers, solved*.
//! What an agent may ask for is **one verb**, `print_document`: a change,
//! because paper comes out of a machine in the room and a person approves the
//! sentence first, and it requires a grant over the document, because a file
//! nobody granted is not one an agent may send anywhere — to a printer least of
//! all, since a printer across the network is somewhere else.
//!
//! # What is deliberately not a verb
//!
//! **Finding and setting up a printer.** A printer is a place documents can go,
//! and adding one is a person choosing that place: [`crate::find`] and
//! [`crate::set_up`] are what a settings surface draws, and no call an agent
//! can make reaches them. An agent that could add a printer could make a place
//! for documents to leave to that nobody chose, which is the one thing
//! `docs/autonomy/v0-5-documents-and-paper-plan.md` says never happens. A test
//! at the bottom of this file holds the list to the one verb.
//!
//! **Which printer.** v0.5 is one printer that works, so the verb prints on the
//! printer this machine prints on and takes no argument naming one — an
//! argument the model filled with a queue name would be machinery reaching a
//! sentence a person approves.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::words;

/// The name the verb is asked for by.
pub const PRINT_DOCUMENT: &str = "print_document";

/// The argument naming the document.
pub const DOCUMENT: &str = "document";

/// Why the printing verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The printing verb, as a list of its own.
///
/// # Errors
/// [`Declaring`], which the verb as written cannot cause.
pub fn printing_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the printing verb on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already has a verb of this name.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    verbs.declare(print_document()?)?;
    Ok(())
}

/// Print a document on this machine's printer.
fn print_document() -> Result<Verb, VerbError> {
    Verb::checked(
        PRINT_DOCUMENT,
        words::VERB_PURPOSE,
        Effect::Change,
        vec![Arg::taking(DOCUMENT, words::VERB_DOCUMENT, Takes::Path)],
        Requires::grants_over([DOCUMENT]),
        words::VERB_SENTENCE,
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_capability::{CallError, Given};

    /// **One verb, a change, over a granted document.**
    #[test]
    fn printing_is_one_verb_a_change_over_a_granted_document() {
        let verbs = printing_verbs().unwrap();
        assert_eq!(verbs.len(), 1);
        let verb = verbs.of(PRINT_DOCUMENT).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert_eq!(verb.requires(), &Requires::grants_over([DOCUMENT]));
    }

    /// **No agent can add a printer.** There is no verb for it on the list, so
    /// asking for one is asking for something that does not exist.
    #[test]
    fn no_verb_an_agent_can_ask_for_adds_a_printer() {
        let verbs = printing_verbs().unwrap();
        for name in ["set_up_printer", "add_printer", "find_printers"] {
            assert!(matches!(
                verbs.call(name, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
        assert!(verbs.all().all(|verb| verb.name() == PRINT_DOCUMENT));
    }

    /// A name already taken is not replaced.
    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = printing_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
        assert!(
            verbs
                .call(PRINT_DOCUMENT, &[(DOCUMENT, Given::text("relative/path"))])
                .is_err()
        );
    }
}
