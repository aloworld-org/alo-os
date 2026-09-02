//! The application verbs, declared.
//!
//! `docs/features.md` promises **open, focus, arrange, close** at v0.01 and
//! `docs/contracts/agent-verbs.md` says what a verb has to be. Three of the
//! four are here; `arrange` is not, and the reason is written down at the
//! bottom of this file rather than left for somebody to notice.
//!
//! **Nothing is declared by default.** [`application_verbs`] hands back a list;
//! [`declare_into`] puts them on somebody else's.
//!
//! # All three are changes, and there is no read
//!
//! No verb here answers a question, and the absence is the design rather than
//! an omission. A verb that listed the running applications, or the open
//! windows, would be a background reader: `CLAUDE.md` says context is offered
//! at invocation and never watched, so what is open reaches an agent as
//! *context*, for that turn, and not as something it can ask for whenever it
//! likes. Adding a `list_applications` would quietly undo that, which is why
//! this list has no way to look at anything.
//!
//! **Focus is a change, and that is not pedantry.** Bringing a window to the
//! front while somebody is typing sends the next keystrokes somewhere they did
//! not choose, and a window that can put itself in front of the one being typed
//! into is the oldest trick there is. So it waits for an approval like any other
//! change, and the sentence says what will happen.
//!
//! # Closing asks; nothing here kills anything
//!
//! `close_application` does what pressing the close button does: the
//! application is *asked*, it gets to put up its own *save your changes?*, and
//! the person answers it. It never takes the window away.
//!
//! The reason is what an approval means. A person approving *ask Blender to
//! close* has approved closing an application; they have not approved
//! discarding the model they have not saved, and one approval covers one
//! sentence and nothing beyond it. Everything else an agent does here is
//! recoverable — an application that was opened can be closed, one brought to
//! the front can be sent back — and unsaved work is the one thing on this list
//! that is gone for good. So the word **ask** is in the sentence a person
//! approves, where a translator can see that it matters and a reader cannot
//! miss it, rather than only in this documentation.
//!
//! # Why `arrange` is not here
//!
//! It needs an argument saying *where*, which is a `Takes::Choice` — and a
//! choice's chosen option goes into the approval sentence as the stable
//! identifier the model picked it by. *Put Blender on the `left_half`* is a
//! sentence with a piece of untranslated English in the middle of it, in the
//! one string the whole capability model is built around. That is a hole in
//! item 9g's guarantee rather than a thing this crate can word its way out of,
//! and closing it is a change to `alo_capability::Takes`. It is queue item 11a.

use alo_capability::{Arg, Effect, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::words;

/// Why the application verbs could not be declared.
///
/// Neither of these can happen to the three as they are written — the tests in
/// this file are what say so. They are here because a `Result` that cannot fail
/// is still better than an unwrap in a library, and because [`declare_into`]
/// can genuinely fail against a list that already has one of these names.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Declaring {
    /// A declaration that does not satisfy the contract.
    #[error(transparent)]
    Verb(#[from] VerbError),
    /// A name already on the list.
    #[error(transparent)]
    List(#[from] VerbsError),
}

/// The application verbs, as a list of their own.
///
/// ```
/// use alo_applications::{application_verbs, application_words};
/// use alo_capability::Given;
/// use alo_strings::Strings;
///
/// let verbs = application_verbs()?;
/// let call = verbs.call("close_application", &[
///     ("application", Given::text("org.blender.Blender")),
/// ]).expect("the three take one application each");
///
/// // The sentence says *ask*, because that is what happens: an application
/// // with unsaved work still gets to ask its own question.
/// let strings = Strings::of(application_words()?);
/// assert_eq!(call.sentence(&strings).text(), "ask org.blender.Blender to close");
/// assert!(call.waits_for_approval());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// # Errors
/// [`Declaring`], which the three as written cannot cause.
pub fn application_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the application verbs on an existing list.
///
/// All three or none of them, as `alo-files` does with its six: a name already
/// taken is found before anything is added, so a list never ends up holding
/// half a set of capabilities that nobody chose.
///
/// # Errors
/// [`Declaring::List`] if the list already holds one of these names — a name
/// means one thing, so whoever took it first keeps it and nothing is silently
/// replaced.
pub fn declare_into(verbs: &mut Verbs) -> Result<(), Declaring> {
    let declaring = [
        open_application()?,
        focus_application()?,
        close_application()?,
    ];
    for verb in &declaring {
        if verbs.of(verb.name()).is_some() {
            return Err(Declaring::List(VerbsError::AlreadyDeclared {
                name: verb.name().to_owned(),
            }));
        }
    }
    for verb in declaring {
        verbs.declare(verb)?;
    }
    Ok(())
}

/// Start an application.
fn open_application() -> Result<Verb, VerbError> {
    Verb::checked(
        "open_application",
        words::OPEN_APPLICATION,
        Effect::Change,
        vec![Arg::taking(
            "application",
            words::OPEN_APPLICATION_APPLICATION,
            Takes::Application,
        )],
        Requires::grants_over(["application"]),
        words::OPEN_APPLICATION_SENTENCE,
    )
}

/// Bring an application to the front.
fn focus_application() -> Result<Verb, VerbError> {
    Verb::checked(
        "focus_application",
        words::FOCUS_APPLICATION,
        Effect::Change,
        vec![Arg::taking(
            "application",
            words::FOCUS_APPLICATION_APPLICATION,
            Takes::Application,
        )],
        Requires::grants_over(["application"]),
        words::FOCUS_APPLICATION_SENTENCE,
    )
}

/// Ask an application to close.
fn close_application() -> Result<Verb, VerbError> {
    Verb::checked(
        "close_application",
        words::CLOSE_APPLICATION,
        Effect::Change,
        vec![Arg::taking(
            "application",
            words::CLOSE_APPLICATION_APPLICATION,
            Takes::Application,
        )],
        Requires::grants_over(["application"]),
        words::CLOSE_APPLICATION_SENTENCE,
    )
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

    /// The three that are here, and no fourth that arrived with them.
    #[test]
    fn the_list_is_the_three_this_crate_declares() {
        let verbs = application_verbs().unwrap();
        let names: Vec<_> = verbs.all().map(Verb::name).collect();
        assert_eq!(
            names,
            ["open_application", "focus_application", "close_application"]
        );
        // The fourth the feature list promises is item 11a, and until it exists
        // it is not on the list under a name that half means it.
        assert!(verbs.of("arrange_application").is_none());
    }

    /// **Every one of them waits for an approval, and none of them answers a
    /// question.** Both halves matter: a change that ran inside the turn would
    /// be an unapproved change, and a read here would be a way to watch a
    /// machine.
    #[test]
    fn every_application_verb_is_a_change_and_none_of_them_is_a_read() {
        let verbs = application_verbs().unwrap();
        for verb in verbs.all() {
            assert_eq!(verb.effect(), Effect::Change, "{}", verb.name());
            assert!(verb.effect().waits_for_approval(), "{}", verb.name());
        }
        for asking in ["list_applications", "list_windows", "what_is_open"] {
            assert!(verbs.of(asking).is_none(), "{asking}");
        }
    }

    /// Each requires a grant, and it is over the application it names — the one
    /// thing `Takes::Application` can be a grant over.
    #[test]
    fn every_application_verb_needs_a_grant_over_the_application_it_names() {
        for verb in application_verbs().unwrap().all() {
            assert_eq!(
                verb.requires(),
                &Requires::Grants(vec!["application".to_owned()]),
                "{}",
                verb.name()
            );
            assert_eq!(verb.args().len(), 1, "{}", verb.name());
            assert_eq!(
                verb.args().first().map(Arg::takes),
                Some(&Takes::Application)
            );
        }
    }

    /// The sentence a person approves, for each of the three. It is generated
    /// from the argument and it names it — a verb that broke either rule could
    /// not have been declared — so this asserts what somebody actually reads.
    #[test]
    fn what_a_person_would_be_approving_reads_as_a_sentence() {
        let verbs = application_verbs().unwrap();
        let strings = in_english();
        for (named, expected) in [
            ("open_application", "open org.blender.Blender"),
            (
                "focus_application",
                "bring org.blender.Blender to the front",
            ),
            ("close_application", "ask org.blender.Blender to close"),
        ] {
            let call = verbs
                .call(
                    named,
                    &[("application", Given::text("org.blender.Blender"))],
                )
                .unwrap();
            assert_eq!(call.sentence(&strings).text(), expected, "{named}");
            assert_eq!(call.asks().len(), 1, "{named}");
        }
    }

    /// **Closing asks.** The sentence a person approves says so, so this is a
    /// test of the promise rather than of a comment: a change to the word here
    /// is a change to what somebody agreed to.
    #[test]
    fn closing_asks_the_application_rather_than_taking_it_away() {
        let call = application_verbs()
            .unwrap()
            .call(
                "close_application",
                &[("application", Given::text("org.blender.Blender"))],
            )
            .unwrap();
        let said = call.sentence(&in_english());
        assert!(said.text().starts_with("ask "), "{said}");
        assert!(said.text().ends_with(" to close"), "{said}");
    }

    /// An argument that is not an identifier never becomes a call, and it is
    /// refused before any grant is consulted or any list is looked at.
    #[test]
    fn a_call_that_does_not_survive_the_door_is_not_a_call() {
        let verbs = application_verbs().unwrap();
        for attempt in ["/usr/bin/blender", "org blender", "   "] {
            let err = verbs
                .call("open_application", &[("application", Given::text(attempt))])
                .unwrap_err();
            assert!(matches!(err, CallError::Argument(_)), "{attempt}: {err:?}");
        }
        let missing = verbs.call("focus_application", &[]).unwrap_err();
        let said = missing.said(&in_english());
        assert!(said.text().contains("application"), "{said}");
    }

    /// The list is closed, and these three do not open it.
    #[test]
    fn nothing_that_runs_something_is_on_the_list() {
        let verbs = application_verbs().unwrap();
        for asked in ["run_application", "exec", "open_application_with_arguments"] {
            assert!(verbs.of(asked).is_none(), "{asked}");
            assert!(matches!(
                verbs.call(asked, &[]),
                Err(CallError::NoSuchVerb { .. })
            ));
        }
    }

    /// A list that already holds one of these names keeps its own, and gets
    /// none of the others either.
    #[test]
    fn a_name_already_taken_is_not_quietly_replaced_and_the_rest_do_not_arrive() {
        let mut verbs = application_verbs().unwrap();
        let again = declare_into(&mut verbs).unwrap_err();
        assert!(matches!(again, Declaring::List(_)), "{again}");
        assert_eq!(verbs.len(), 3);

        // The clash is on the last of the three, and the two before it are not
        // added on the way to finding it.
        let theirs = application_verbs().unwrap();
        let mut mine = Verbs::default();
        mine.declare(theirs.of("close_application").unwrap().clone())
            .unwrap();
        let clash = declare_into(&mut mine).unwrap_err();
        assert!(clash.to_string().contains("close_application"), "{clash}");
        assert_eq!(mine.len(), 1);
        assert!(mine.of("open_application").is_none());
    }

    /// The three sit beside the file verbs on one list, which is the
    /// arrangement a daemon has: one registry, every crate declaring into it.
    #[test]
    fn they_join_a_list_that_already_holds_another_crates_verbs() {
        let mut verbs = Verbs::default();
        verbs
            .declare(
                Verb::checked(
                    "list_folder",
                    alo_strings::Word::saying(
                        "testing.verb.list-folder",
                        "list what is in a folder",
                    ),
                    Effect::Read,
                    vec![Arg::taking(
                        "folder",
                        alo_strings::Word::saying("testing.argument.folder", "the folder to list"),
                        Takes::Path,
                    )],
                    Requires::grants_over(["folder"]),
                    alo_strings::Word::saying(
                        "testing.sentence.list-folder",
                        "list what is in {folder}",
                    ),
                )
                .unwrap(),
            )
            .unwrap();
        declare_into(&mut verbs).unwrap();
        assert_eq!(verbs.len(), 4);
        assert!(verbs.of("open_application").is_some());
        assert!(verbs.of("list_folder").is_some());
    }

    /// **Every word the three are declared with is one this crate declares.**
    ///
    /// The risk item 9g introduced and the test that closes it, which
    /// `docs/contracts/agent-verbs.md` asks of every crate declaring verbs: a
    /// constant left out of [`crate::words`]'s list would compile, declare, and
    /// reach a person as a key in the place where the sentence they are
    /// approving belongs.
    #[test]
    fn everything_the_three_say_is_something_this_crate_declares() {
        let strings = in_english();
        let verbs = application_verbs().unwrap();
        for verb in verbs.all() {
            let purpose = verb.purpose(&strings);
            assert!(!purpose.is_a_bug(), "{}: {purpose}", verb.name());
            for arg in verb.args() {
                let said = arg.purpose(&strings);
                assert!(!said.is_a_bug(), "{} {}: {said}", verb.name(), arg.name());
            }
            let call = verbs
                .call(
                    verb.name(),
                    &[("application", Given::text("org.blender.Blender"))],
                )
                .unwrap();
            let said = call.sentence(&strings);
            assert!(!said.is_a_bug(), "{}: {said}", verb.name());
            assert!(said.unfilled().is_empty(), "{:?}", said.unfilled());
        }
    }

    /// Nothing arrives declared. A registry nobody gave these to has no
    /// application capabilities at all.
    #[test]
    fn nothing_is_declared_until_somebody_declares_it() {
        let empty = Verbs::default();
        assert!(empty.of("open_application").is_none());
        let mut mine = Verbs::default();
        declare_into(&mut mine).unwrap();
        assert_eq!(mine.len(), 3);
    }
}
