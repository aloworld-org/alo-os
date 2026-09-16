//! An adapter, loaded against the contract — held to each clause of the plan's
//! acceptance, with each refusal beside what it refuses.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 5:
//!
//! - **an adapter is loaded as declared data, and one whose verb takes a
//!   script, a command or free text that becomes code is refused** —
//!   [`an_adapter_whose_verb_takes_a_script_is_refused`] and the tests after it;
//! - **each adapter verb is approved and recorded like any verb, and has a
//!   by-hand road or is refused for lacking one** —
//!   [`an_adapter_verb_is_approved_once_sent_once_and_recorded`],
//!   [`every_refusal_of_an_adapter_verb_is_recorded`],
//!   [`a_verb_with_no_by_hand_road_is_refused`];
//! - **one reference adapter, for an application a fresh machine ships** —
//!   [`the_reference_adapter_is_for_an_application_a_fresh_machine_ships`], and
//!   on a real bus in `the_reference_adapter_on_a_real_bus.rs`.
//!
//! The application here is a stand-in that keeps every message it is handed
//! ([`AnApplication`]), so what was sent is what it received rather than what
//! this crate claims to have sent.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_adapters::{
    Adapter, AdapterArg, AdapterVerb, Adapters, Argument, DBusMethod, Delivers, Driving,
    Invocation, Kind, Mechanism, Message, NotDelivered, NotLoaded, ONLY_ITS_APPLICATION, Part,
    Reaches, TEXT_EDITOR, adapter_verbs, load, shipped_adapters,
};
use alo_applications::{Application, Installed};
use alo_capability::{
    Approvals, Authorised, Effect, Given, Grant, Grantee, Grants, Proposal, ProposalError, Reach,
    Refused, Requires, Verbs,
};
use alo_record::{Asking, Entry, Only, Record};
use alo_software::{Role, Shipped};
use alo_strings::{Strings, Vocabulary, Word};

// ---------------------------------------------------------------------------
// The machine these tests look at.
// ---------------------------------------------------------------------------

/// A fixed moment, so expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long grants and questions last here.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent.
fn agent() -> Grantee {
    Grantee::named("@alo")
}

/// The words a shell has: this crate's, beside the capability model's and the
/// applications'.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    alo_adapters::words::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A grant to the agent over each of these, made at noon for an hour.
fn granting(reaches: &[Reach]) -> Grants {
    let mut grants = Grants::default();
    for reach in reaches {
        grants.grant(Grant::checked("@alo", reach.clone(), noon(), hour()).unwrap());
    }
    grants
}

/// The text editor, and a folder of notes.
fn the_editor_and_the_notes() -> Grants {
    granting(&[
        Reach::Application("org.gnome.TextEditor".to_owned()),
        Reach::Folder(PathBuf::from("/home/anna/Notes")),
    ])
}

/// A machine with the text editor installed.
fn with_the_editor() -> Installed {
    Installed::holding([Application::called("org.gnome.TextEditor", "Text Editor").unwrap()])
}

/// An application standing in for the text editor: it keeps every message it
/// is handed, and answers as it is told to.
struct AnApplication {
    handed: RefCell<Vec<Message>>,
    answers: Result<(), NotDelivered>,
}

impl AnApplication {
    fn answering(answers: Result<(), NotDelivered>) -> Self {
        Self {
            handed: RefCell::new(Vec::new()),
            answers,
        }
    }
}

impl Delivers for AnApplication {
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
        self.handed.borrow_mut().push(message.clone());
        self.answers
    }
}

/// A call of a shipped adapter verb.
fn calling(verb: &str, given: &[(&str, Given)]) -> alo_capability::Call {
    adapter_verbs().unwrap().call(verb, given).unwrap()
}

/// Open this document.
fn opening(document: &str) -> alo_capability::Call {
    calling(
        "text_editor.open_document",
        &[("document", Given::text(document))],
    )
}

/// Propose, approve once and redeem.
fn approved(call: &alo_capability::Call, grants: &Grants) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(call, &agent(), grants, noon(), hour()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

// ---------------------------------------------------------------------------
// Declarations that break the contract, one rule each.
// ---------------------------------------------------------------------------

const PURPOSE: Word = Word::saying("adapters.test.purpose", "do one thing");
const WHICH: Word = Word::saying("adapters.test.which", "which one");
const SENTENCE: Word = Word::saying("adapters.test.sentence", "do it to {which}");
const SENTENCE_NONE: Word = Word::saying("adapters.test.sentence-none", "do it");
const BY_HAND: Word = Word::saying("adapters.test.by-hand", "do it yourself");
const UNDECLARED: Word = Word::saying("adapters.test.undeclared", "a word nobody declared");
const WORDS: [Word; 5] = [PURPOSE, WHICH, SENTENCE, SENTENCE_NONE, BY_HAND];

/// A method that passes every check.
const OPEN: DBusMethod = DBusMethod {
    object: "/org/example/Drawing",
    interface: "org.freedesktop.Application",
    method: "Open",
    parameters: &[Part::FileAddressOf("which"), Part::NoPlatformData],
};

/// One verb over a path, sound in every way.
const A_SOUND_VERB: AdapterVerb = AdapterVerb {
    name: "open_drawing",
    purpose: PURPOSE,
    effect: Effect::Change,
    args: &[AdapterArg {
        name: "which",
        purpose: WHICH,
        kind: Kind::Path,
    }],
    reaches: Reaches::Over(&["which"]),
    sentence: SENTENCE,
    by_hand: Some(BY_HAND),
    carried_out: Invocation::DBus(OPEN),
};

/// An adapter with that one verb, sound in every way.
const A_SOUND_ADAPTER: Adapter = Adapter {
    name: "drawing",
    application: "org.example.Drawing",
    releases: &["3"],
    mechanism: Mechanism::DBus,
    words: &WORDS,
    verbs: &[A_SOUND_VERB],
};

/// A verb that sends `which`, of this kind, to this parameter.
const fn taking(kind: Kind, part: Part) -> AdapterVerb {
    AdapterVerb {
        args: match kind {
            Kind::Path => &[AdapterArg {
                name: "which",
                purpose: WHICH,
                kind: Kind::Path,
            }],
            Kind::Text => &[AdapterArg {
                name: "which",
                purpose: WHICH,
                kind: Kind::Text,
            }],
            Kind::Command => &[AdapterArg {
                name: "which",
                purpose: WHICH,
                kind: Kind::Command,
            }],
            Kind::Script { .. } => &[AdapterArg {
                name: "which",
                purpose: WHICH,
                kind: Kind::Script { language: "python" },
            }],
            Kind::Name { .. } | Kind::Count { .. } | Kind::Choice(_) => &[AdapterArg {
                name: "which",
                purpose: WHICH,
                kind: Kind::Name { longest: 64 },
            }],
        },
        reaches: Reaches::OnlyItsApplication,
        carried_out: Invocation::DBus(DBusMethod {
            parameters: match part {
                Part::TextOf(_) => &[Part::TextOf("which")],
                Part::Evaluated { .. } => &[Part::Evaluated {
                    argument: "which",
                    by: "python",
                }],
                _ => &[Part::FileAddressOf("which")],
            },
            ..OPEN
        }),
        ..A_SOUND_VERB
    }
}

/// The sound adapter with this one verb in place of its own.
const fn with_verb(verb: &'static [AdapterVerb]) -> Adapter {
    Adapter {
        verbs: verb,
        ..A_SOUND_ADAPTER
    }
}

static TAKES_A_SCRIPT: [AdapterVerb; 1] = [taking(
    Kind::Script { language: "python" },
    Part::TextOf("which"),
)];
static TAKES_A_COMMAND: [AdapterVerb; 1] = [taking(Kind::Command, Part::TextOf("which"))];
static TAKES_FREE_TEXT: [AdapterVerb; 1] = [taking(Kind::Text, Part::TextOf("which"))];
static EVALUATES_A_NAME: [AdapterVerb; 1] = [taking(
    Kind::Name { longest: 64 },
    Part::Evaluated {
        argument: "which",
        by: "python",
    },
)];

static SCRIPTED: Adapter = with_verb(&TAKES_A_SCRIPT);
static COMMANDED: Adapter = with_verb(&TAKES_A_COMMAND);
static FREE_TEXT: Adapter = with_verb(&TAKES_FREE_TEXT);
static EVALUATED: Adapter = with_verb(&EVALUATES_A_NAME);

/// **An adapter whose verb takes a script is refused**, and so is one taking a
/// command, free text, or handing a typed name to something that interprets
/// it. Nothing of any of them reaches a list; the sound adapter beside them
/// does.
#[test]
fn an_adapter_whose_verb_takes_a_script_is_refused() {
    static SOUND: Adapter = A_SOUND_ADAPTER;
    assert_eq!(load(&SOUND).unwrap().verbs().len(), 1);

    for (adapter, kind) in [
        (&SCRIPTED, "script"),
        (&COMMANDED, "command"),
        (&FREE_TEXT, "text"),
    ] {
        let refused = load(adapter).unwrap_err();
        assert_eq!(
            refused,
            NotLoaded::TakesCode {
                verb: "drawing.open_drawing".to_owned(),
                argument: "which".to_owned(),
                kind,
            }
        );
        assert!(
            refused.to_string().contains("never takes a script"),
            "{refused}"
        );
    }

    assert_eq!(
        load(&EVALUATED).unwrap_err(),
        NotLoaded::BecomesCode {
            verb: "drawing.open_drawing".to_owned(),
            argument: "which".to_owned(),
            by: "python".to_owned(),
        }
    );
}

/// **A method named for running something, and an action a model would
/// choose, are refused** — the two ways a typed argument becomes code without
/// ever being declared as a script.
#[test]
fn a_method_that_runs_things_or_lets_an_argument_choose_the_action_is_refused() {
    static EVAL: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            interface: "org.example.Drawing.Scripting",
            method: "Evaluate",
            ..OPEN
        }),
        ..A_SOUND_VERB
    }];
    static RUNS: Adapter = with_verb(&EVAL);
    assert!(matches!(
        load(&RUNS).unwrap_err(),
        NotLoaded::RunsSomething { method, .. } if method == "org.example.Drawing.Scripting.Evaluate"
    ));

    // `ActivateAction` with the action's name taken from an argument.
    static CHOSEN: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            method: "ActivateAction",
            parameters: &[
                Part::TextOf("which"),
                Part::NoParameters,
                Part::NoPlatformData,
            ],
            ..OPEN
        }),
        args: &[AdapterArg {
            name: "which",
            purpose: WHICH,
            kind: Kind::Name { longest: 64 },
        }],
        reaches: Reaches::OnlyItsApplication,
        ..A_SOUND_VERB
    }];
    static CHOOSES: Adapter = with_verb(&CHOSEN);
    assert!(matches!(
        load(&CHOOSES).unwrap_err(),
        NotLoaded::ChoosesWhatRuns { .. }
    ));

    // The same method with the action written into the adapter loads.
    static WRITTEN: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            method: "ActivateAction",
            parameters: &[
                Part::Literal("new-window"),
                Part::NoParameters,
                Part::NoPlatformData,
            ],
            ..OPEN
        }),
        args: &[],
        reaches: Reaches::OnlyItsApplication,
        sentence: SENTENCE_NONE,
        ..A_SOUND_VERB
    }];
    static WRITTEN_IN: Adapter = with_verb(&WRITTEN);
    let loaded = load(&WRITTEN_IN).unwrap();
    assert!(matches!(
        loaded.verbs().first().unwrap().requires(),
        Requires::Nothing { reason } if reason == ONLY_ITS_APPLICATION
    ));
}

/// **Screenshots and synthetic input are never loaded**, and the two
/// mechanisms nothing here carries out yet are refused rather than offered.
#[test]
fn screenshots_and_synthetic_input_are_refused_and_so_is_what_is_not_carried_out_here() {
    static SYNTHETIC: Adapter = Adapter {
        mechanism: Mechanism::Synthetic,
        ..A_SOUND_ADAPTER
    };
    assert!(matches!(
        load(&SYNTHETIC).unwrap_err(),
        NotLoaded::Synthetic { .. }
    ));
    static API: Adapter = Adapter {
        mechanism: Mechanism::Api,
        ..A_SOUND_ADAPTER
    };
    static ACCESSIBILITY: Adapter = Adapter {
        mechanism: Mechanism::Accessibility,
        ..A_SOUND_ADAPTER
    };
    for (adapter, mechanism) in [(&API, "api"), (&ACCESSIBILITY, "accessibility")] {
        assert_eq!(
            load(adapter).unwrap_err(),
            NotLoaded::NotCarriedOutHere {
                adapter: "drawing".to_owned(),
                mechanism,
            }
        );
    }
}

/// **A verb with no by-hand road is refused** (ADR 0009), with the verb named.
#[test]
fn a_verb_with_no_by_hand_road_is_refused() {
    static NONE: [AdapterVerb; 1] = [AdapterVerb {
        by_hand: None,
        ..A_SOUND_VERB
    }];
    static WITHOUT: Adapter = with_verb(&NONE);
    let refused = load(&WITHOUT).unwrap_err();
    assert_eq!(
        refused,
        NotLoaded::NoByHand {
            verb: "drawing.open_drawing".to_owned()
        }
    );
    assert!(refused.to_string().contains("ADR 0009"), "{refused}");
}

/// **A verb cannot reach outside its grant**: a path no grant is required over
/// is refused, whether the verb names other grants or none.
#[test]
fn a_path_no_grant_is_required_over_is_refused() {
    static UNCOVERED: [AdapterVerb; 1] = [AdapterVerb {
        reaches: Reaches::OnlyItsApplication,
        ..A_SOUND_VERB
    }];
    static OUTSIDE: Adapter = with_verb(&UNCOVERED);
    assert_eq!(
        load(&OUTSIDE).unwrap_err(),
        NotLoaded::OutsideItsGrant {
            verb: "drawing.open_drawing".to_owned(),
            argument: "which".to_owned()
        }
    );
}

/// **The rest of the declaration is held too**: a person's own application, no
/// release, no verbs, an undeclared word, an argument that is never sent, a
/// parameter from nowhere or of the wrong kind, a malformed name, a bad adapter
/// name, and one application with two adapters.
#[test]
fn every_other_rule_of_the_contract_refuses_its_declaration() {
    static TERMINAL: Adapter = Adapter {
        application: "app.devsuite.Ptyxis",
        ..A_SOUND_ADAPTER
    };
    assert!(matches!(
        load(&TERMINAL).unwrap_err(),
        NotLoaded::APersonsOwn { .. }
    ));

    static NO_RELEASE: Adapter = Adapter {
        releases: &[],
        ..A_SOUND_ADAPTER
    };
    assert!(matches!(
        load(&NO_RELEASE).unwrap_err(),
        NotLoaded::NoReleases { .. }
    ));

    static NO_VERBS: Adapter = Adapter {
        verbs: &[],
        ..A_SOUND_ADAPTER
    };
    assert!(matches!(
        load(&NO_VERBS).unwrap_err(),
        NotLoaded::NoVerbs { .. }
    ));

    static BAD_NAME: Adapter = Adapter {
        name: "Drawing Tool",
        ..A_SOUND_ADAPTER
    };
    assert!(matches!(
        load(&BAD_NAME).unwrap_err(),
        NotLoaded::NotAName { .. }
    ));

    static BAD_APPLICATION: Adapter = Adapter {
        application: "/usr/bin/sh",
        ..A_SOUND_ADAPTER
    };
    assert!(matches!(
        load(&BAD_APPLICATION).unwrap_err(),
        NotLoaded::NotAnApplication { .. }
    ));

    static UNDECLARED_WORD: [AdapterVerb; 1] = [AdapterVerb {
        by_hand: Some(UNDECLARED),
        ..A_SOUND_VERB
    }];
    static UNSAID: Adapter = with_verb(&UNDECLARED_WORD);
    assert!(matches!(
        load(&UNSAID).unwrap_err(),
        NotLoaded::WordNotDeclared { key, .. } if key == "adapters.test.undeclared"
    ));

    static UNSENT: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            parameters: &[Part::NoPlatformData],
            ..OPEN
        }),
        ..A_SOUND_VERB
    }];
    static NOWHERE: Adapter = with_verb(&UNSENT);
    assert!(matches!(
        load(&NOWHERE).unwrap_err(),
        NotLoaded::GoesNowhere { .. }
    ));

    static FROM_NOWHERE: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            parameters: &[Part::FileAddressOf("which"), Part::TextOf("other")],
            ..OPEN
        }),
        ..A_SOUND_VERB
    }];
    static NO_SUCH: Adapter = with_verb(&FROM_NOWHERE);
    assert!(matches!(
        load(&NO_SUCH).unwrap_err(),
        NotLoaded::NoSuchArgument { argument, .. } if argument == "other"
    ));

    static WRONG: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            parameters: &[Part::CountOf("which")],
            ..OPEN
        }),
        ..A_SOUND_VERB
    }];
    static WRONG_KIND: Adapter = with_verb(&WRONG);
    assert!(matches!(
        load(&WRONG_KIND).unwrap_err(),
        NotLoaded::WrongKind { .. }
    ));

    static MALFORMED: [AdapterVerb; 1] = [AdapterVerb {
        carried_out: Invocation::DBus(DBusMethod {
            object: "org/example",
            ..OPEN
        }),
        ..A_SOUND_VERB
    }];
    static NOT_WELL_FORMED: Adapter = with_verb(&MALFORMED);
    assert!(matches!(
        load(&NOT_WELL_FORMED).unwrap_err(),
        NotLoaded::NotWellFormed { .. }
    ));

    // The verb contract itself still refuses what it refuses.
    static OMITS: [AdapterVerb; 1] = [AdapterVerb {
        sentence: SENTENCE_NONE,
        ..A_SOUND_VERB
    }];
    static OMITTING: Adapter = with_verb(&OMITS);
    assert!(matches!(
        load(&OMITTING).unwrap_err(),
        NotLoaded::Verb { .. }
    ));

    static TWICE: [AdapterVerb; 2] = [A_SOUND_VERB, A_SOUND_VERB];
    static SAME_NAME: Adapter = with_verb(&TWICE);
    assert!(matches!(load(&SAME_NAME).unwrap_err(), NotLoaded::Taken(_)));

    static FIRST: Adapter = A_SOUND_ADAPTER;
    static SECOND: Adapter = Adapter {
        name: "sketching",
        ..A_SOUND_ADAPTER
    };
    let mut adapters = Adapters::default();
    adapters.add(load(&FIRST).unwrap()).unwrap();
    assert!(matches!(
        adapters.add(load(&SECOND).unwrap()).unwrap_err(),
        NotLoaded::Twice { .. }
    ));
}

// ---------------------------------------------------------------------------
// The reference adapter.
// ---------------------------------------------------------------------------

/// **The reference adapter is for an application a fresh machine ships, at the
/// release it ships**, through the application's own interface, and every verb
/// it declares is on the machine's list.
#[test]
fn the_reference_adapter_is_for_an_application_a_fresh_machine_ships() {
    let shipped = Shipped::decided().unwrap();
    let editor = shipped.the(Role::TextEditor);
    assert_eq!(TEXT_EDITOR.application, editor.application().identifier());
    assert!(
        TEXT_EDITOR.supports(editor.version()),
        "the adapter does not support release {} of the text editor a fresh machine ships",
        editor.version()
    );
    assert_eq!(TEXT_EDITOR.mechanism, Mechanism::DBus);

    let adapters = shipped_adapters().unwrap();
    assert_eq!(adapters.all().count(), 1);
    let mut verbs = Verbs::default();
    adapters.declare_into(&mut verbs).unwrap();
    assert_eq!(verbs.len(), TEXT_EDITOR.verbs.len());
    for verb in TEXT_EDITOR.verbs {
        assert!(verbs.of(&TEXT_EDITOR.verb_named(verb)).is_some());
        assert!(verb.by_hand.is_some());
    }
}

/// **An adapter verb is approved once, sent once, and recorded like any verb**:
/// the sentence the person approves names the document, the message the
/// application receives is the file address of that document and nothing
/// else, the approval cannot be answered twice, and the record keeps the
/// approved sentence with its approval and its grants.
#[test]
fn an_adapter_verb_is_approved_once_sent_once_and_recorded() {
    let strings = in_english();
    let adapters = shipped_adapters().unwrap();
    let grants = the_editor_and_the_notes();
    let call = opening("/home/anna/Notes/march notes.txt");

    let mut approvals = Approvals::default();
    let proposal = Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap();
    assert_eq!(
        proposal.sentence(&strings).text(),
        "open /home/anna/Notes/march notes.txt in GNOME Text Editor"
    );
    let id = approvals.propose(proposal);
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(&grants, noon())
        .unwrap();
    assert!(approvals.approve(id, noon()).is_err(), "answered twice");

    let application = AnApplication::answering(Ok(()));
    let driving =
        Driving::of(authorised, &adapters, &grants, &with_the_editor(), &strings).unwrap();
    let driven = driving.deliver(&application, &strings).unwrap();
    assert!(!driven.unanswered());
    assert!(driven.said(&strings).is_none());

    let handed = application.handed.borrow();
    assert_eq!(handed.len(), 1, "one approval, one message");
    let message = handed.first().unwrap();
    assert_eq!(message.destination(), "org.gnome.TextEditor");
    assert_eq!(message.object(), "/org/gnome/TextEditor");
    assert_eq!(message.interface(), "org.freedesktop.Application");
    assert_eq!(message.method(), "Open");
    assert_eq!(
        message.arguments(),
        [
            Argument::Texts(vec!["file:///home/anna/Notes/march%20notes.txt".to_owned()]),
            Argument::NoPlatformData,
        ]
    );

    let mut record = Record::default();
    record.keep(Entry::ran(&driven.into_authorised(), &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), Some(id.as_u64()));
    // Against the document's grant: the application's is asked where it is
    // reached, and a call names only what its arguments reach.
    assert_eq!(entry.happened().against().len(), 1);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is("text_editor.open_document"));
    assert!(
        what.sentence()
            .is("open /home/anna/Notes/march notes.txt in GNOME Text Editor")
    );
}

/// **The verb that reaches only its application sends the action the adapter
/// wrote**, never one a model named.
#[test]
fn a_new_window_is_the_action_written_into_the_adapter() {
    let strings = in_english();
    let grants = granting(&[Reach::Application("org.gnome.TextEditor".to_owned())]);
    let call = calling("text_editor.new_window", &[]);
    let application = AnApplication::answering(Ok(()));
    Driving::of(
        approved(&call, &grants),
        &shipped_adapters().unwrap(),
        &grants,
        &with_the_editor(),
        &strings,
    )
    .unwrap()
    .deliver(&application, &strings)
    .unwrap();
    let handed = application.handed.borrow();
    let message = handed.first().unwrap();
    assert_eq!(message.method(), "ActivateAction");
    assert_eq!(
        message.arguments(),
        [
            Argument::Text("new-window".to_owned()),
            Argument::NoParameters,
            Argument::NoPlatformData,
        ]
    );

    // And an agent cannot send it anything to choose with.
    assert!(
        adapter_verbs()
            .unwrap()
            .call("text_editor.new_window", &[("action", Given::text("quit"))])
            .is_err()
    );
}

/// **A document outside every grant is never put to a person**, and an agent
/// is offered an adapter's verbs only while it holds its application.
#[test]
fn what_is_not_granted_is_neither_offered_nor_proposed() {
    let grants = the_editor_and_the_notes();
    let outside = opening("/home/anna/Payroll/salaries.txt");
    assert!(matches!(
        Proposal::checked(&outside, &agent(), &grants, noon(), hour()).unwrap_err(),
        ProposalError::NotGranted(_)
    ));

    let adapters = shipped_adapters().unwrap();
    assert_eq!(
        adapters
            .offered_to(&agent(), &grants, noon())
            .unwrap()
            .len(),
        2
    );
    let only_the_notes = granting(&[Reach::Folder(PathBuf::from("/home/anna/Notes"))]);
    assert!(
        adapters
            .offered_to(&agent(), &only_the_notes, noon())
            .unwrap()
            .is_empty()
    );
    // Expired is gone.
    assert!(
        adapters
            .offered_to(&agent(), &grants, noon() + hour() * 2)
            .unwrap()
            .is_empty()
    );
}

/// **Every refusal of an adapter verb is recorded, in the words the person
/// read, and nothing is sent**: the application not granted (identically
/// whether it is installed or not), the grant revoked between approval and
/// sending, the application not installed, not there, not offering it, and
/// refusing it.
#[test]
fn every_refusal_of_an_adapter_verb_is_recorded() {
    let strings = in_english();
    let adapters = shipped_adapters().unwrap();
    let mut record = Record::default();
    let mut keep = |refused: &Refused| {
        record.keep(Entry::refused(refused, &agent(), &strings, noon()));
    };
    let application = AnApplication::answering(Ok(()));

    // Not granted: the document is, the application is not — and the refusal
    // is the same whether it is installed or not.
    let only_the_notes = granting(&[Reach::Folder(PathBuf::from("/home/anna/Notes"))]);
    let call = opening("/home/anna/Notes/march.txt");
    let installed = Driving::of(
        approved(&call, &only_the_notes),
        &adapters,
        &only_the_notes,
        &with_the_editor(),
        &strings,
    )
    .unwrap_err();
    let not_installed = Driving::of(
        approved(&call, &only_the_notes),
        &adapters,
        &only_the_notes,
        &Installed::nothing(),
        &strings,
    )
    .unwrap_err();
    assert_eq!(
        installed, not_installed,
        "an ungranted application's refusal says whether it is installed"
    );
    assert!(
        installed
            .said(&strings)
            .text()
            .contains("has not been granted"),
        "{}",
        installed.said(&strings).text()
    );
    keep(&installed);

    // Revoked between approval and sending.
    let mut grants = the_editor_and_the_notes();
    let authorised = approved(&call, &grants);
    grants.revoke_everything_for(&agent());
    let revoked =
        Driving::of(authorised, &adapters, &grants, &with_the_editor(), &strings).unwrap_err();
    keep(&revoked);

    // Not installed.
    let grants = the_editor_and_the_notes();
    let missing = Driving::of(
        approved(&call, &grants),
        &adapters,
        &grants,
        &Installed::nothing(),
        &strings,
    )
    .unwrap_err();
    assert!(
        missing
            .said(&strings)
            .text()
            .contains("org.gnome.TextEditor"),
        "{}",
        missing.said(&strings).text()
    );
    keep(&missing);

    assert!(
        application.handed.borrow().is_empty(),
        "something was sent after a refusal"
    );

    // What the application says back.
    for (answer, says) in [
        (NotDelivered::NotThere, "could not be reached"),
        (
            NotDelivered::DoesNotOffer,
            "does not offer what was approved",
        ),
        (NotDelivered::Refused, "refused what was approved"),
    ] {
        let application = AnApplication::answering(Err(answer));
        let refused = Driving::of(
            approved(&call, &grants),
            &adapters,
            &grants,
            &with_the_editor(),
            &strings,
        )
        .unwrap()
        .deliver(&application, &strings)
        .unwrap_err();
        assert_eq!(application.handed.borrow().len(), 1, "sent more than once");
        assert!(
            refused.said(&strings).text().contains(says),
            "{}",
            refused.said(&strings).text()
        );
        keep(&refused);
    }

    // An authority for a verb that is not an adapter's sends nothing.
    let mut other = Verbs::default();
    alo_applications::declare_into(&mut other).unwrap();
    let focus = other
        .call(
            "focus_application",
            &[("application", Given::text("org.gnome.TextEditor"))],
        )
        .unwrap();
    let not_ours = Driving::of(
        approved(&focus, &grants),
        &adapters,
        &grants,
        &with_the_editor(),
        &strings,
    )
    .unwrap_err();
    keep(&not_ours);

    assert_eq!(record.len(), 7);
    let refusals = Asking::anything().only(Only::Refusals);
    let stopped: Vec<_> = record.answering(&refusals).collect();
    assert_eq!(stopped.len(), 7);
    for entry in stopped {
        assert!(entry.happened().was_stopped());
        assert!(!entry.happened().ran());
        assert!(entry.happened().why_stopped().is_some());
    }
}

/// **An application that did not answer is recorded as run, and the person is
/// told what is not known** — something was sent under the approval, and a
/// record that left it out would be a record of less than happened.
#[test]
fn an_unanswered_message_is_recorded_as_run_and_said_as_not_known() {
    let strings = in_english();
    let grants = the_editor_and_the_notes();
    let call = opening("/home/anna/Notes/march.txt");
    let application = AnApplication::answering(Err(NotDelivered::DidNotAnswer));
    let driven = Driving::of(
        approved(&call, &grants),
        &shipped_adapters().unwrap(),
        &grants,
        &with_the_editor(),
        &strings,
    )
    .unwrap()
    .deliver(&application, &strings)
    .unwrap();
    assert!(driven.unanswered());
    assert!(
        driven
            .said(&strings)
            .unwrap()
            .text()
            .contains("is not known")
    );
    let mut record = Record::default();
    record.keep(Entry::ran(&driven.into_authorised(), &strings));
    assert!(record.everything().next().unwrap().happened().ran());
}

/// **Every word a shipped adapter verb says is declared**, so nothing reaches a
/// person as a key where a sentence belongs.
#[test]
fn every_word_a_shipped_adapter_verb_says_is_declared() {
    let strings = in_english();
    for verb in adapter_verbs().unwrap().all() {
        assert!(!verb.purpose(&strings).is_a_bug(), "{}", verb.name());
        for arg in verb.args() {
            assert!(!arg.purpose(&strings).is_a_bug(), "{}", arg.name());
        }
    }
    let grants = the_editor_and_the_notes();
    let proposal = Proposal::checked(
        &opening("/home/anna/Notes/a.txt"),
        &agent(),
        &grants,
        noon(),
        hour(),
    )
    .unwrap();
    let sentence = proposal.sentence(&strings);
    assert!(!sentence.is_a_bug(), "{}", sentence.text());
}
