//! The one verb an agent proposes an installation through.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`: *an agent may propose
//! installing an application through a verb a person approves, never install
//! one by itself.* So there is **one verb**, `install_application`, and it is a
//! change — it waits for one approval of the sentence *install
//! org.gnome.TextEditor from flathub*, and that approval installs once.
//!
//! # What is deliberately not a verb
//!
//! **Updating and removing.** Neither is on the plan as something an agent
//! proposes, and a verb that removed an application would be a verb that ends
//! grants a person made; the list is held to the one verb by a test at the
//! bottom of this file.
//!
//! **Looking at what is installed, or what could be.** A read that listed a
//! person's applications would be the background reader `CLAUDE.md` calls a
//! bug, and a fingerprint of who they are besides — `alo-applications` makes
//! the same refusal about its own list.
//!
//! # Why it needs no grant
//!
//! Every other change an agent proposes is over something a person granted it.
//! This one is over nothing on the machine: the application is not here yet, it
//! arrives granted nothing ([`mod@crate::installing`]), and no agent can reach it
//! afterwards without a grant over it. What protects the person is the approval
//! itself — one sentence, naming the application and the place, answered once —
//! and the reason is written into the declaration where the next reader of the
//! verb list will find it.
//!
//! # From an approval to an installation
//!
//! [`approved`] is the road from an `alo_capability::Authorised` to a
//! [`Wanted`]: it accepts only this verb, and only an authority that came from
//! an approval. A read never reaches `Authorised` for a change
//! (`alo_capability::Authorised::read` refuses it), so an agent's own call can
//! never become an installation by itself.

use alo_applications::Application;
use alo_capability::{
    Arg, Authorised, Effect, Requires, Takes, Value, Verb, VerbError, Verbs, VerbsError,
};

use crate::installing::Wanted;
use crate::source::SourceName;
use crate::words;

/// The name the verb is asked for by.
pub const INSTALL_APPLICATION: &str = "install_application";

/// Why the software verb could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// An authority that is not an approved installation.
///
/// English and a `Display`, like `alo_capability::VerbError`: it cannot happen
/// because of anything a person did, only because an executor handed this
/// crate the wrong authority, and its reader is fixing that executor.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotAnInstallation {
    /// The authority is for another verb.
    #[error("{verb} is not {INSTALL_APPLICATION}, so it installs nothing")]
    AnotherVerb {
        /// The verb it was for.
        verb: String,
    },
    /// The authority did not come from an approval.
    #[error("{INSTALL_APPLICATION} is a change, and this authority came from no approval")]
    NotApproved,
    /// An argument the declaration requires is missing or of the wrong kind.
    #[error("{INSTALL_APPLICATION} arrived without a valid {argument}")]
    Malformed {
        /// Which argument.
        argument: &'static str,
    },
}

/// The software verb, as a list of its own.
///
/// # Errors
/// [`Declaring`], which the verb as written cannot cause.
pub fn software_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the software verb on an existing list.
///
/// # Errors
/// [`Declaring::List`] if the list already has a verb of this name.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    verbs.declare(install_application()?)?;
    Ok(())
}

/// Install an application from a place this machine installs from.
fn install_application() -> Result<Verb, VerbError> {
    Verb::checked(
        INSTALL_APPLICATION,
        words::VERB_PURPOSE,
        Effect::Change,
        vec![
            Arg::taking(
                words::APPLICATION,
                words::VERB_APPLICATION,
                Takes::Application,
            ),
            Arg::taking(
                words::SOURCE,
                words::VERB_SOURCE,
                Takes::name(SourceName::LONGEST),
            ),
        ],
        Requires::nothing_because(
            "the application is not on this machine yet, it arrives granted nothing, and the \
             person approves this one installation by the sentence naming it and its place",
        ),
        words::VERB_SENTENCE,
    )
}

/// The installation a person approved, from the authority that approval
/// became.
///
/// # Errors
/// [`NotAnInstallation`] for another verb's authority, one that came from no
/// approval, or one missing an argument.
pub fn approved(authorised: &Authorised) -> Result<Wanted, NotAnInstallation> {
    if authorised.verb() != INSTALL_APPLICATION {
        return Err(NotAnInstallation::AnotherVerb {
            verb: authorised.verb().to_owned(),
        });
    }
    if authorised.from_approval().is_none() {
        return Err(NotAnInstallation::NotApproved);
    }
    let call = authorised.call();
    let Some(Value::Application(identifier)) = call.value(words::APPLICATION) else {
        return Err(NotAnInstallation::Malformed {
            argument: words::APPLICATION,
        });
    };
    let Some(Value::Name(source)) = call.value(words::SOURCE) else {
        return Err(NotAnInstallation::Malformed {
            argument: words::SOURCE,
        });
    };
    let application =
        Application::identified(identifier).map_err(|_| NotAnInstallation::Malformed {
            argument: words::APPLICATION,
        })?;
    Ok(Wanted::by_hand(application, source))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;
    use alo_capability::{CallError, Given};

    #[test]
    fn installing_is_one_verb_a_change_that_needs_no_grant_and_says_why() {
        let verbs = software_verbs().unwrap();
        assert_eq!(verbs.len(), 1);
        let verb = verbs.of(INSTALL_APPLICATION).unwrap();
        assert_eq!(verb.effect(), Effect::Change);
        assert!(
            matches!(verb.requires(), Requires::Nothing { reason } if reason.contains("approves"))
        );
    }

    /// **The sentence names the application and the place**, filled from the
    /// validated arguments.
    #[test]
    fn the_sentence_a_person_approves_names_the_application_and_the_place() {
        let call = software_verbs()
            .unwrap()
            .call(
                INSTALL_APPLICATION,
                &[
                    (words::APPLICATION, Given::text("org.gnome.TextEditor")),
                    (words::SOURCE, Given::text("flathub")),
                ],
            )
            .unwrap();
        assert!(call.waits_for_approval());
        assert_eq!(
            call.sentence(&in_english()).text(),
            "install org.gnome.TextEditor from flathub"
        );
    }

    /// **No verb updates, removes or lists anything**, so an agent can ask for
    /// none of them.
    #[test]
    fn no_verb_an_agent_can_ask_for_updates_removes_or_lists() {
        let verbs = software_verbs().unwrap();
        for name in [
            "update_application",
            "remove_application",
            "list_applications",
            "list_sources",
            "add_source",
        ] {
            assert!(matches!(
                verbs.call(name, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
    }

    /// **An argument that is not a name or an identifier never becomes a
    /// call**, so nothing shaped like a command or an option reaches the
    /// sentence, let alone the rented tool.
    #[test]
    fn arguments_shaped_like_anything_but_names_are_refused_at_the_boundary() {
        let verbs = software_verbs().unwrap();
        for (application, source) in [
            ("org.gnome.TextEditor", "flathub/../../etc"),
            ("org.gnome.TextEditor", "flathub\nreboot"),
            ("org gnome", "flathub"),
            ("/usr/bin/sh", "flathub"),
            ("org.gnome.TextEditor", ""),
        ] {
            assert!(
                verbs
                    .call(
                        INSTALL_APPLICATION,
                        &[
                            (words::APPLICATION, Given::text(application)),
                            (words::SOURCE, Given::text(source)),
                        ],
                    )
                    .is_err(),
                "{application:?} from {source:?} became a call"
            );
        }
    }

    #[test]
    fn a_name_already_taken_is_not_replaced() {
        let mut verbs = software_verbs().unwrap();
        assert!(matches!(declare_into(&mut verbs), Err(Declaring::List(_))));
    }
}
