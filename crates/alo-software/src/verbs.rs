//! The two software verbs: what a person could install, and the installation an
//! agent proposes.
//!
//! `docs/autonomy/software-and-the-web-plan.md`: *an agent may propose
//! installing an application through a verb a person approves, never install
//! one by itself.* So `install_application` is a **change** — it waits for one
//! approval of the sentence *install org.gnome.TextEditor from flathub*, and
//! that approval installs once.
//!
//! `what_can_be_installed` is a **read**, so it answers inside the turn with no
//! approval at all (ADR 0001 §5), and what it answers is
//! [`crate::WhatTheCatalogueHolds`].
//!
//! # The read was refused here until 2026-10-17, and the refusal was too wide
//!
//! This header said:
//!
//! > **Looking at what is installed, or what could be.** A read that listed a
//! > person's applications would be the background reader `CLAUDE.md` calls a
//! > bug, and a fingerprint of who they are besides — `alo-applications` makes
//! > the same refusal about its own list.
//!
//! **That is one sentence covering two different questions, and the reason it
//! gives is only about the first.** `alo-applications`' own words are *a verb
//! that listed the running applications, or the open windows, would be a
//! background reader* — which is true, and is about **the state of somebody's
//! machine**. The catalogue is not the state of anybody's machine: it is
//! `include_str!` of `catalogue.toml`, byte-identical on every alo machine
//! sold, and `Catalogue::decided` checks it against `shipped.toml`, which is
//! also shipped. Reading it tells an agent nothing whatever about the person.
//!
//! So *what is installed* stays refused, for the reason that was always the
//! right one, and *what could be* is a verb. **The line is held by a type and
//! not by this paragraph:** `WhatTheCatalogueHolds` has no constructor taking
//! an `Installed` or a `Tool`, so there is no way to answer this read with
//! something read off the machine.
//!
//! The owner's instruction of 2026-10-17 is why this was looked at rather than
//! obeyed: *we should not follow the laws that we made if they make our agent
//! or system weak.* An agent that cannot be shown a shipped list of eight
//! applications cannot help a person find one, and nothing was protected by
//! refusing it.
//!
//! # What is still deliberately not a verb
//!
//! **Updating and removing.** Neither is on the plan as something an agent
//! proposes, and a verb that removed an application would be a verb that ends
//! grants a person made; the list is held to these two by a test at the bottom
//! of this file.
//!
//! **Looking at what is installed.** Unchanged, and now for its own reason
//! rather than inside somebody else's sentence.
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
    Arg, Authorised, Effect, Offered, Requires, Takes, Value, Verb, VerbError, Verbs, VerbsError,
};

use crate::category::Category;
use crate::installing::Wanted;
use crate::source::SourceName;
use crate::words;

/// The name the verb is asked for by.
pub const INSTALL_APPLICATION: &str = "install_application";

/// The name the read is asked for by.
pub const WHAT_CAN_BE_INSTALLED: &str = "what_can_be_installed";

/// The option meaning *do not narrow at all*.
///
/// **A name a model sends, beside the eight `Category::as_an_option` names.** It
/// is not a `Category`, because narrowing to nothing is not a kind of work —
/// `crate::Category`'s list says what a person makes, and *everything* is the
/// absence of that question rather than an answer to it.
pub const EVERYTHING: &str = "everything";

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

/// An authority that is not a look at the catalogue.
///
/// English and a `Display`, for [`NotAnInstallation`]'s reason exactly: it
/// cannot happen because of anything a person did, only because an executor
/// handed this crate the wrong authority.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotALook {
    /// The authority is for another verb.
    #[error("{verb} is not {WHAT_CAN_BE_INSTALLED}, so it looks at nothing")]
    AnotherVerb {
        /// The verb it was for.
        verb: String,
    },
    /// The authority came from an approval, so something treated a read as a
    /// change.
    ///
    /// **The mirror of [`NotAnInstallation::NotApproved`], and it is here for
    /// the same reason that one is.** `alo_capability::Proposal::checked`
    /// refuses a read offered for approval and `Authorised::read` refuses a
    /// change offered as a read, so this cannot be reached from outside — and
    /// an authority for this verb that *had* come from an approval would mean a
    /// person had been asked to approve a question, which is the one thing
    /// ADR 0001 §5 is about.
    #[error("{WHAT_CAN_BE_INSTALLED} is a read, and this authority came from an approval")]
    Approved,
    /// The kind of work is missing, or is not one of the nine.
    #[error("{WHAT_CAN_BE_INSTALLED} arrived without a valid {argument}")]
    Malformed {
        /// Which argument.
        argument: &'static str,
    },
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
    verbs.declare(what_can_be_installed()?)?;
    verbs.declare(install_application()?)?;
    Ok(())
}

/// See what this machine's catalogue holds, under one kind of work or all of
/// them.
///
/// **A read, so no approval**, and `Requires::nothing_because` says why in the
/// place the next reader of the verb list will look: there is nothing on the
/// machine to grant over, because the catalogue is a file this crate ships.
///
/// **One required argument with nine choices rather than an optional one.**
/// `alo_capability::Arg`'s own rule: *every argument is required … a verb that
/// needs to behave two ways declares a `Takes::Choice` and says so in its
/// sentence, or it is two verbs.* So *everything* is the ninth choice and not
/// an absent argument, which is also how the window a person opens draws it —
/// the chip for *Everything* sits beside the eight and is chosen first.
fn what_can_be_installed() -> Result<Verb, VerbError> {
    // **The options are built from `Category::EVERY`**, so a category added to
    // that list is offered here without anybody remembering to come back. The
    // name is `Category::as_an_option` and the word is `Category::word`:
    // `crate::Category`'s header is why those cannot be one thing, and
    // `as_an_option`'s own is why it is not `named` — `alo_capability` refuses
    // an option that is not an identifier, and the catalogue's spelling has a
    // hyphen in it. Every software verb test failed at once when `named` was
    // passed here, which is what a declaration-time refusal is for.
    let mut kinds = vec![Offered::called(EVERYTHING, words::EVERYTHING)];
    kinds.extend(
        Category::EVERY
            .into_iter()
            .map(|kind| Offered::called(&kind.as_an_option(), kind.word())),
    );
    Verb::checked(
        WHAT_CAN_BE_INSTALLED,
        words::READ_PURPOSE,
        Effect::Read,
        vec![Arg::taking(
            words::KIND_OF_WORK,
            words::READ_KIND_OF_WORK,
            Takes::choice(kinds),
        )],
        Requires::nothing_because(
            "the catalogue is a list this machine ships, the same on every alo machine, and \
             reading it says nothing about what this person has installed or has open",
        ),
        words::READ_SENTENCE,
    )
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
    )?
    .changing(words::VERB_CHANGES, Some(words::VERB_LEAVES))
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

/// The kind of work a person wants to look under, from the authority their
/// agent's read became.
///
/// [`None`] is *everything*, which is the ninth choice and not a missing
/// argument — see this module's `what_can_be_installed`, which declares the
/// nine.
///
/// **This is the only reader of the call**, as `approved` is for the
/// installation: an executor that looked at an argument itself would be a
/// second opinion about what arrived.
///
/// # Errors
/// [`NotALook`] for another verb's authority, one that came from an approval,
/// and one whose kind of work is missing or is not one of the nine.
pub fn looked_at(authorised: &Authorised) -> Result<Option<Category>, NotALook> {
    if authorised.verb() != WHAT_CAN_BE_INSTALLED {
        return Err(NotALook::AnotherVerb {
            verb: authorised.verb().to_owned(),
        });
    }
    if authorised.from_approval().is_some() {
        return Err(NotALook::Approved);
    }
    let Some(Value::Choice { chosen, .. }) = authorised.call().value(words::KIND_OF_WORK) else {
        return Err(NotALook::Malformed {
            argument: words::KIND_OF_WORK,
        });
    };
    if chosen == EVERYTHING {
        return Ok(None);
    }
    // **Matched against the same list the options were built from**, so a
    // category offered and a category understood cannot be two different sets.
    Category::EVERY
        .into_iter()
        .find(|kind| kind.as_an_option() == *chosen)
        .map(Some)
        .ok_or(NotALook::Malformed {
            argument: words::KIND_OF_WORK,
        })
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

    /// **Two verbs, one a change and one a read, and each says why it needs no
    /// grant.**
    ///
    /// The count is asserted rather than the two names alone, because a third
    /// verb appearing here is the thing this test is for: `verbs.rs`' header
    /// says what is deliberately not on this list, and a list that grew without
    /// that paragraph growing with it is the drift that matters.
    #[test]
    fn there_are_two_verbs_a_change_and_a_read_and_each_says_why_it_needs_no_grant() {
        let verbs = software_verbs().unwrap();
        assert_eq!(verbs.len(), 2);

        let installing = verbs.of(INSTALL_APPLICATION).unwrap();
        assert_eq!(installing.effect(), Effect::Change);
        assert!(
            matches!(installing.requires(), Requires::Nothing { reason } if reason.contains("approves"))
        );

        let looking = verbs.of(WHAT_CAN_BE_INSTALLED).unwrap();
        assert_eq!(looking.effect(), Effect::Read);
        assert!(
            matches!(looking.requires(), Requires::Nothing { reason } if reason.contains("ships")),
            "the read does not say why it needs no grant"
        );
    }

    /// **The read offers nine kinds of work and nothing else**, which is what
    /// `Takes::Choice` is for: *the options exist before the model does, and
    /// anything else is refused.*
    #[test]
    fn the_read_offers_everything_and_the_eight_kinds_of_work_and_nothing_else() {
        let verbs = software_verbs().unwrap();
        let verb = verbs.of(WHAT_CAN_BE_INSTALLED).unwrap();
        let args = verb.args();

        assert_eq!(args.len(), 1, "the read takes more than the one argument");
        // **Matched rather than panicked on**, because `clippy::panic` is
        // denied in this crate and the assertion reads better anyway: what is
        // being claimed is that the argument *is* a choice.
        let takes = args.first().unwrap().takes();
        assert!(
            matches!(takes, Takes::Choice(_)),
            "the kind of work is {takes:?}, so free text reaches it"
        );
        let Takes::Choice(offered) = takes else {
            return;
        };
        assert_eq!(offered.len(), Category::EVERY.len() + 1);
        assert_eq!(offered.first().unwrap().name(), EVERYTHING);
        for kind in Category::EVERY {
            assert!(
                offered.iter().any(|one| one.name() == kind.as_an_option()),
                "{} is not offered",
                kind.named()
            );
        }
    }

    /// **The read answers inside the turn and waits for nobody**, which is the
    /// whole difference between it and the installation beside it.
    #[test]
    fn looking_at_the_catalogue_waits_for_no_approval() {
        let call = software_verbs()
            .unwrap()
            .call(
                WHAT_CAN_BE_INSTALLED,
                &[(words::KIND_OF_WORK, Given::text("page_layout"))],
            )
            .unwrap();

        assert!(
            !call.waits_for_approval(),
            "a person was asked to approve a question"
        );
        // **The sentence the record keeps says the word, not the identifier.**
        // `Category::as_an_option` is what a model sent; what a reader of the
        // record sees is what a person would read.
        assert_eq!(
            call.sentence(&in_english()).text(),
            "see what can be installed under Page layout"
        );
    }

    /// **A kind of work nobody offered never becomes a call**, including the
    /// catalogue file's own hyphenated spelling — which is the right spelling
    /// in that file and not a name a verb takes.
    #[test]
    fn a_kind_of_work_nobody_offered_is_refused_at_the_boundary() {
        let verbs = software_verbs().unwrap();
        for asked in [
            "page-layout",
            "photoshop",
            "everything_else",
            "installed",
            "",
            "PICTURES",
        ] {
            assert!(
                verbs
                    .call(
                        WHAT_CAN_BE_INSTALLED,
                        &[(words::KIND_OF_WORK, Given::text(asked))],
                    )
                    .is_err(),
                "{asked:?} became a call"
            );
        }
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
            // **What is installed is still refused**, and that is the half of
            // this crate's old refusal that was always right: a verb listing a
            // person's applications would be a fingerprint of who they are.
            // `what_can_be_installed` is the other half and is a different
            // question — see this module's header.
            "list_applications",
            "what_is_installed",
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
