//! **The one verb by which an agent may see the screen.**
//!
//! `docs/contracts/agent-verbs.md`, *the screen verb*. Declared here because
//! this is the crate that takes the picture, as `alo-software` declares the
//! installing verb and `alo-printing` the printing one.
//!
//! # Why it takes no arguments
//!
//! There is no version of this that is narrower than *your screen*. A verb with
//! a region argument would read as *a picture of this corner*, and what the
//! machine would actually hold is a picture of everything, cut down afterwards
//! by the same program that asked for it. The sentence a person approves says
//! the whole of what happens.
//!
//! # Why it requires no grant
//!
//! This was written first as *requires the `screen-once` facility*, which
//! `alo-capability` refuses: **ADR 0040 keeps facilities to applications and
//! never to agents**, because *a durable grant to the camera would be a
//! background reader by another name* — `GrantError::NotForAnAgent` is that
//! rule in code, and it is the same rule as task 5's *no application may answer
//! the picking in advance*.
//!
//! So an agent holds nothing standing over the screen, and **the approval of
//! the sentence is the authority**: one picture, this once, and again only by
//! asking again. It is the second verb in this contract that requires no grant,
//! for a different reason from the first — `install_application` needs none
//! because there is nothing on the machine yet to grant over; this needs none
//! because what it would be over is something an agent may never hold.

use alo_capability::{Effect, Requires, Verb, VerbError, Verbs, VerbsError};

use crate::words;

/// What the verb is called, everywhere.
pub const PICTURE_OF_THE_SCREEN: &str = "picture_of_the_screen";

/// Why the screen verb could not be declared. Neither can happen as it is
/// written; the type exists because `declare_into` is a public surface and a
/// caller deserves to be told which half failed.
#[derive(Debug, thiserror::Error)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// Put the screen verb on a registry.
///
/// # Errors
/// [`Declaring::List`] where the registry already has a verb of this name.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    verbs.declare(picture_of_the_screen()?)?;
    Ok(())
}

/// A picture of the screen, for the turn that asked.
fn picture_of_the_screen() -> Result<Verb, VerbError> {
    Verb::checked(
        PICTURE_OF_THE_SCREEN,
        words::VERB_PURPOSE,
        Effect::Change,
        Vec::new(),
        Requires::nothing_because(
            "ADR 0040 keeps the screen, the camera and notifications to applications and never \
             to agents — a durable grant to the screen would be a background reader by another \
             name — so there is nothing standing for an agent to hold here. The approval of the \
             sentence is the whole of the authority, one picture at a time, and \
             `alo_capturing::ForTheAgent::approved` refuses an authority that came from anything \
             but an approval",
        ),
        words::VERB_SENTENCE,
    )
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **It is a change, so it waits for one approval**, and it takes nothing
    /// to fill in.
    #[test]
    fn the_screen_verb_is_a_change_with_nothing_to_fill_in() {
        let verb = picture_of_the_screen().expect("the verb is declarable");
        assert_eq!(verb.name(), PICTURE_OF_THE_SCREEN);
        assert_eq!(verb.effect(), Effect::Change);
        assert!(
            verb.args().is_empty(),
            "a narrower-sounding argument would describe a picture the machine does not take"
        );
    }

    /// **The reason it names no argument grant is written down**, and it names
    /// what does check the grant.
    #[test]
    fn the_reason_it_requires_no_argument_grant_names_what_checks_it() {
        let verb = picture_of_the_screen().expect("the verb is declarable");
        match verb.requires() {
            Requires::Nothing { reason } => {
                assert!(reason.contains("ADR 0040"), "{reason}");
                assert!(reason.contains("ForTheAgent"), "{reason}");
            }
            Requires::Grants(over) => panic!("the screen is not an argument: {over:?}"),
        }
    }

    /// **One registry, one screen verb.**
    #[test]
    fn it_goes_on_a_registry_once() {
        let mut verbs = Verbs::default();
        declare_into(&mut verbs).expect("a fresh registry takes it");
        assert!(
            declare_into(&mut verbs).is_err(),
            "a second screen verb could be declared beside the first"
        );
        assert!(verbs.all().any(|verb| verb.name() == PICTURE_OF_THE_SCREEN));
    }
}
