//! The application verbs, declared.
//!
//! `docs/features.md` promises **open, focus, arrange, close** at v0.01 and
//! `docs/contracts/agent-verbs.md` says what a verb has to be. All four are
//! here.
//!
//! **Nothing is declared by default.** [`application_verbs`] hands back a list;
//! [`declare_into`] puts them on somebody else's.
//!
//! # All four are changes, and there is no read
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
//! change, and the sentence says what will happen. Arranging is a change for the
//! same reason and one more: a window that moved is a window something else is
//! now underneath.
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
//! the front can be sent back, a window that was moved can be moved again — and
//! unsaved work is the one thing on this list that is gone for good. So the word
//! **ask** is in the sentence a person approves, where a translator can see that
//! it matters and a reader cannot miss it, rather than only in this
//! documentation.
//!
//! # Three arrangements, and the two that are not here
//!
//! `arrange_application` offers the left half, the right half and the whole
//! screen. Two windows put on opposite halves is what `docs/features.md`
//! promises as *tile* at v0.01, and the whole screen is *maximise*; **quarters
//! are v0.5** and are deliberately absent, along with the split that holds while
//! you work, because the scope gate is a gate.
//!
//! Minimising is not an arrangement and is not here either. It is on the v0.01
//! *window management* list, which is what a person does with their own
//! keyboard and mouse; this verb says where a window **goes**, and *out of the
//! way* is not a place. A verb for it would be a verb of its own, with its own
//! sentence, and nothing in `docs/features.md` asks for one.
//!
//! **The option a person approves is a word, not an identifier** (item 11a).
//! `Takes::Choice` holds `alo_capability::Offered`s, so *put Blender on the left
//! half of the screen* is one sentence in the reader's language rather than a
//! German sentence with `left_half` in the middle of it.

use alo_capability::{Arg, Effect, Offered, Requires, Takes, Verb, VerbError, Verbs, VerbsError};

use crate::words;

/// Why the application verbs could not be declared.
///
/// Neither of these can happen to the four as they are written — the tests in
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
/// ]).expect("three of the four take one application each");
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
/// [`Declaring`], which the four as written cannot cause.
pub fn application_verbs() -> Result<Verbs, Declaring> {
    let mut verbs = Verbs::default();
    declare_into(&mut verbs)?;
    Ok(verbs)
}

/// Put the application verbs on an existing list.
///
/// All four or none of them, as `alo-files` does with its six: a name already
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
        arrange_application()?,
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

/// Put an application's window somewhere on the screen.
///
/// The only verb in this crate with two arguments, and the only one anywhere in
/// this workspace that offers a choice. The grant is over the application and
/// not over the arrangement: *where* is not a thing a grant could be about, and
/// `alo_capability::Takes::can_be_a_grant` refuses a declaration that says
/// otherwise.
fn arrange_application() -> Result<Verb, VerbError> {
    Verb::checked(
        "arrange_application",
        words::ARRANGE_APPLICATION,
        Effect::Change,
        vec![
            Arg::taking(
                "application",
                words::ARRANGE_APPLICATION_APPLICATION,
                Takes::Application,
            ),
            Arg::taking(
                "where",
                words::ARRANGE_APPLICATION_WHERE,
                Takes::choice(arrangements()),
            ),
        ],
        Requires::grants_over(["application"]),
        words::ARRANGE_APPLICATION_SENTENCE,
    )
}

/// Where a window may be put, at v0.01.
///
/// Declared from the words a translator is handed, as everything else in this
/// crate is: the name is what a model sends and the record keeps, and the word
/// is what a person approves. Quarters are v0.5 and are not here.
fn arrangements() -> [Offered; 3] {
    [
        Offered::called("left_half", words::LEFT_HALF),
        Offered::called("right_half", words::RIGHT_HALF),
        Offered::called("whole_screen", words::WHOLE_SCREEN),
    ]
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

    /// The four `docs/features.md` promises at v0.01, and no fifth that arrived
    /// with them.
    #[test]
    fn the_list_is_the_four_this_crate_declares() {
        let verbs = application_verbs().unwrap();
        let names: Vec<_> = verbs.all().map(Verb::name).collect();
        assert_eq!(
            names,
            [
                "open_application",
                "focus_application",
                "close_application",
                "arrange_application"
            ]
        );
    }

    /// One call of one verb, with the arguments the tests below all use.
    fn calling(named: &str) -> alo_capability::Call {
        let verbs = application_verbs().unwrap();
        let given = [
            ("application", Given::text("org.blender.Blender")),
            ("where", Given::text("left_half")),
        ];
        let takes_where = verbs
            .of(named)
            .is_some_and(|verb| verb.arg("where").is_some());
        verbs
            .call(named, if takes_where { &given } else { &given[..1] })
            .unwrap()
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
    ///
    /// **`arrange_application` takes a second argument and the grant is not
    /// over it**, which is the honest shape rather than an omission: *where* is
    /// not a thing anybody can grant, and a declaration claiming otherwise is
    /// refused by `Takes::can_be_a_grant`.
    #[test]
    fn every_application_verb_needs_a_grant_over_the_application_it_names() {
        for verb in application_verbs().unwrap().all() {
            assert_eq!(
                verb.requires(),
                &Requires::Grants(vec!["application".to_owned()]),
                "{}",
                verb.name()
            );
            assert_eq!(
                verb.args().first().map(Arg::takes),
                Some(&Takes::Application),
                "{}",
                verb.name()
            );
        }
        let verbs = application_verbs().unwrap();
        let arranging = verbs.of("arrange_application").unwrap();
        assert_eq!(arranging.args().len(), 2);
        assert!(matches!(
            arranging.args().get(1).map(Arg::takes),
            Some(&Takes::Choice(_))
        ));
        assert!(!Takes::choice(arrangements()).can_be_a_grant());
    }

    /// The sentence a person approves, for each of the four. It is generated
    /// from the arguments and it names all of them — a verb that broke either
    /// rule could not have been declared — so this asserts what somebody
    /// actually reads.
    #[test]
    fn what_a_person_would_be_approving_reads_as_a_sentence() {
        let strings = in_english();
        for (named, expected) in [
            ("open_application", "open org.blender.Blender"),
            (
                "focus_application",
                "bring org.blender.Blender to the front",
            ),
            ("close_application", "ask org.blender.Blender to close"),
            (
                "arrange_application",
                "put org.blender.Blender on the left half of the screen",
            ),
        ] {
            let call = calling(named);
            assert_eq!(call.sentence(&strings).text(), expected, "{named}");
            assert_eq!(call.asks().len(), 1, "{named}");
        }
    }

    /// **The arrangement a person approves is a phrase, never the name a model
    /// sent.** This is item 11a as a person meets it: before it, the sentence
    /// read *put org.blender.Blender left_half*.
    #[test]
    fn the_arrangement_reaches_the_sentence_as_words_and_the_value_as_a_name() {
        let verbs = application_verbs().unwrap();
        for (sent, reads) in [
            ("left_half", "on the left half of the screen"),
            ("right_half", "on the right half of the screen"),
            ("whole_screen", "across the whole screen"),
        ] {
            let call = verbs
                .call(
                    "arrange_application",
                    &[
                        ("application", Given::text("org.blender.Blender")),
                        ("where", Given::text(sent)),
                    ],
                )
                .unwrap();
            assert_eq!(
                call.sentence(&in_english()).text(),
                format!("put org.blender.Blender {reads}"),
                "{sent}"
            );
            // And the value underneath is still the name that was sent, which
            // is what the record keeps.
            assert_eq!(
                call.value("where").map(alo_capability::Value::describe),
                Some(sent.to_owned()),
                "{sent}"
            );
        }
    }

    /// **The whole line is one language.** A German machine reads a German
    /// sentence with a German arrangement in it — and one that has translated
    /// the sentence but not the arrangement says so rather than looking
    /// finished, which is what item 11a bought in `alo-strings`.
    #[test]
    fn the_arrangement_is_read_in_the_readers_own_language() {
        let whole = crate::testing::translated(&[
            (
                words::ARRANGE_APPLICATION_SENTENCE,
                "{application} {where} platzieren",
            ),
            (words::LEFT_HALF, "auf der linken Bildschirmhälfte"),
        ]);
        let said = calling("arrange_application").sentence(&whole);
        assert_eq!(
            said.text(),
            "org.blender.Blender auf der linken Bildschirmhälfte platzieren"
        );
        assert!(said.is_translated());
        assert!(said.unfilled().is_empty());

        let half = crate::testing::translated(&[(
            words::ARRANGE_APPLICATION_SENTENCE,
            "{application} {where} platzieren",
        )]);
        let mixed = calling("arrange_application").sentence(&half);
        assert_eq!(
            mixed.text(),
            "org.blender.Blender on the left half of the screen platzieren"
        );
        assert!(
            !mixed.is_translated(),
            "an untranslated arrangement is untranslated English on the line"
        );
        assert!(!mixed.is_a_bug(), "not translated yet is work, not a fault");
    }

    /// An arrangement nobody offered never becomes a call, and the refusal
    /// names **what has to be sent** rather than what a person would read: a
    /// call that never validated is about what arrived.
    #[test]
    fn an_arrangement_that_is_not_offered_is_refused_by_name() {
        let verbs = application_verbs().unwrap();
        for attempt in ["left", "LEFT_HALF", "top_left_quarter", "on the left half"] {
            let said = verbs
                .call(
                    "arrange_application",
                    &[
                        ("application", Given::text("org.blender.Blender")),
                        ("where", Given::text(attempt)),
                    ],
                )
                .unwrap_err()
                .said(&in_english());
            assert!(
                said.text().contains("left_half, right_half, whole_screen"),
                "{attempt}: {said}"
            );
        }
    }

    /// Quarters are v0.5, so they are absent rather than half-present under a
    /// name that suggests otherwise. The scope gate is a gate.
    #[test]
    fn a_quarter_is_not_offered_at_this_version() {
        assert_eq!(arrangements().len(), 3);
        assert!(
            !arrangements()
                .iter()
                .any(|offered| offered.name().contains("quarter"))
        );
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

        // And two arguments given the wrong way round are refused rather than
        // read as whichever one they happen to fit.
        let sideways = verbs
            .call(
                "arrange_application",
                &[
                    ("application", Given::text("left_half")),
                    ("where", Given::text("org.blender.Blender")),
                ],
            )
            .unwrap_err();
        assert!(matches!(sideways, CallError::Argument(_)), "{sideways:?}");
    }

    /// The list is closed, and these four do not open it.
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
        assert_eq!(verbs.len(), 4);

        // The clash is on the last of the four, and the three before it are not
        // added on the way to finding it.
        let theirs = application_verbs().unwrap();
        let mut mine = Verbs::default();
        mine.declare(theirs.of("arrange_application").unwrap().clone())
            .unwrap();
        let clash = declare_into(&mut mine).unwrap_err();
        assert!(clash.to_string().contains("arrange_application"), "{clash}");
        assert_eq!(mine.len(), 1);
        assert!(mine.of("open_application").is_none());
    }

    /// The four sit beside the file verbs on one list, which is the
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
        assert_eq!(verbs.len(), 5);
        assert!(verbs.of("open_application").is_some());
        assert!(verbs.of("list_folder").is_some());
    }

    /// **Every word the four are declared with is one this crate declares.**
    ///
    /// The risk item 9g introduced and the test that closes it, which
    /// `docs/contracts/agent-verbs.md` asks of every crate declaring verbs: a
    /// constant left out of [`crate::words`]'s list would compile, declare, and
    /// reach a person as a key in the place where the sentence they are
    /// approving belongs.
    ///
    /// **Item 11a widened it to the options**, which are declared the same way
    /// and can be left out the same way — and an arrangement nobody declared
    /// would reach a person as a key inside the sentence rather than in place
    /// of it, which is harder to notice rather than easier.
    #[test]
    fn everything_the_four_say_is_something_this_crate_declares() {
        let strings = in_english();
        let verbs = application_verbs().unwrap();
        for verb in verbs.all() {
            let purpose = verb.purpose(&strings);
            assert!(!purpose.is_a_bug(), "{}: {purpose}", verb.name());
            for arg in verb.args() {
                let said = arg.purpose(&strings);
                assert!(!said.is_a_bug(), "{} {}: {said}", verb.name(), arg.name());
            }
            let said = calling(verb.name()).sentence(&strings);
            assert!(!said.is_a_bug(), "{}: {said}", verb.name());
            assert!(said.unfilled().is_empty(), "{:?}", said.unfilled());
        }
        for offered in arrangements() {
            let said = offered.shown(&strings);
            assert!(!said.is_a_bug(), "{}: {said}", offered.name());
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
        assert_eq!(mine.len(), 4);
    }
}
