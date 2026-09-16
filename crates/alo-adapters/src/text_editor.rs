//! The reference adapter: GNOME Text Editor, through its own interface.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 5: *one reference
//! adapter for an application task 2 ships is built end to end as the proof,
//! with the application's own automation interface.* The text editor is on the
//! fresh machine's list (`crates/alo-software/shipped.toml`, at release 50.1),
//! and it publishes itself on the session bus the way every GNOME application
//! does: it holds its own identifier as a bus name and answers
//! `org.freedesktop.Application` on `/org/gnome/TextEditor` — `Open` with a list
//! of file addresses, and `ActivateAction` with one of the actions it registers
//! (`new-window` among them, in `src/editor-application-actions.c` upstream).
//!
//! # Why this application, of the seven
//!
//! Its interface is **the application's own and addressed by its own name**.
//! The file manager answers a shared interface under a shared name that
//! whichever file manager is running holds, so a sentence saying *in Dolphin*
//! could be carried out by something else; the browser's automation interface
//! evaluates script, which is exactly what this crate refuses; and the terminal
//! is a person's own (ADR 0043). The text editor's published permissions reach
//! the person's files (`--filesystem=host`), so a file address it is handed is
//! one it can open.
//!
//! # Two verbs, and what is deliberately not one
//!
//! `open_document` opens a granted file in it; `new_window` opens an empty
//! window. Both change what is on the screen, so both are changes. **Not
//! quitting**: it would close whatever a person had open, and no sentence
//! about *quit* names what that is. **Not saving, and not typing**: the
//! interface offers neither as a typed method, and building one from its
//! actions with an argument choosing the action would be a verb whose meaning a
//! model chooses — refused ([`crate::NotLoaded::ChoosesWhatRuns`]).

use alo_capability::Effect;
use alo_strings::Word;

use crate::adapter::Adapter;
use crate::adapter_arg::{AdapterArg, Kind};
use crate::adapter_verb::{AdapterVerb, Reaches};
use crate::invocation::{DBusMethod, Invocation, Part};
use crate::mechanism::Mechanism;

/// The application, as the machine and its bus know it.
pub const APPLICATION: &str = "org.gnome.TextEditor";

/// The object it answers on.
const OBJECT: &str = "/org/gnome/TextEditor";

/// The interface every GNOME application answers.
const INTERFACE: &str = "org.freedesktop.Application";

/// The one argument `open_document` takes.
pub const DOCUMENT: &str = "document";

/// What `open_document` does.
pub const OPEN_DOCUMENT_PURPOSE: Word = Word::saying(
    "adapters.text-editor.open-document.purpose",
    "open a document in GNOME Text Editor",
)
.noting(
    "What one thing the agent can do with the text editor is for, shown in the list of what an \
     agent may do. GNOME Text Editor is the application's name and is not translated.",
);

/// What `document` is.
pub const OPEN_DOCUMENT_DOCUMENT: Word = Word::saying(
    "adapters.text-editor.open-document.document",
    "the document to open",
)
.noting("Shown beside the one thing the agent names when it asks to open a document.");

/// What a person approves.
pub const OPEN_DOCUMENT_SENTENCE: Word = Word::saying(
    "adapters.text-editor.open-document.sentence",
    "open {document} in GNOME Text Editor",
)
.noting(
    "The sentence a person approves before anything happens. {document} is the full path of a \
     file on this machine and is shown as it is. GNOME Text Editor is the application's name \
     and is not translated.",
);

/// How a person does it without the agent.
pub const OPEN_DOCUMENT_BY_HAND: Word = Word::saying(
    "adapters.text-editor.open-document.by-hand",
    "In GNOME Text Editor, choose Open and pick the document",
)
.noting(
    "How a person does the same thing themselves. \"Open\" is the name of the text editor's own \
     button, so use the word that application shows in this language.",
);

/// What `new_window` does.
pub const NEW_WINDOW_PURPOSE: Word = Word::saying(
    "adapters.text-editor.new-window.purpose",
    "open a new, empty GNOME Text Editor window",
)
.noting(
    "What one thing the agent can do with the text editor is for. GNOME Text Editor is the \
     application's name and is not translated.",
);

/// What a person approves.
pub const NEW_WINDOW_SENTENCE: Word = Word::saying(
    "adapters.text-editor.new-window.sentence",
    "open a new GNOME Text Editor window",
)
.noting(
    "The sentence a person approves before anything happens. It names nothing but the \
     application, because nothing else is opened. GNOME Text Editor is not translated.",
);

/// How a person does it without the agent.
pub const NEW_WINDOW_BY_HAND: Word = Word::saying(
    "adapters.text-editor.new-window.by-hand",
    "In GNOME Text Editor, open its menu and choose New Window",
)
.noting(
    "How a person does the same thing themselves. \"New Window\" is the text editor's own menu \
     item, so use the words that application shows in this language.",
);

/// Every word this adapter declares its verbs with.
pub const WORDS: [Word; 7] = [
    OPEN_DOCUMENT_PURPOSE,
    OPEN_DOCUMENT_DOCUMENT,
    OPEN_DOCUMENT_SENTENCE,
    OPEN_DOCUMENT_BY_HAND,
    NEW_WINDOW_PURPOSE,
    NEW_WINDOW_SENTENCE,
    NEW_WINDOW_BY_HAND,
];

/// GNOME Text Editor, as an adapter.
pub const TEXT_EDITOR: Adapter = Adapter {
    name: "text_editor",
    application: APPLICATION,
    releases: &["50"],
    mechanism: Mechanism::DBus,
    words: &WORDS,
    verbs: &[
        AdapterVerb {
            name: "open_document",
            purpose: OPEN_DOCUMENT_PURPOSE,
            effect: Effect::Change,
            args: &[AdapterArg {
                name: DOCUMENT,
                purpose: OPEN_DOCUMENT_DOCUMENT,
                kind: Kind::Path,
            }],
            reaches: Reaches::Over(&[DOCUMENT]),
            sentence: OPEN_DOCUMENT_SENTENCE,
            by_hand: Some(OPEN_DOCUMENT_BY_HAND),
            carried_out: Invocation::DBus(DBusMethod {
                object: OBJECT,
                interface: INTERFACE,
                method: "Open",
                parameters: &[Part::FileAddressOf(DOCUMENT), Part::NoPlatformData],
            }),
        },
        AdapterVerb {
            name: "new_window",
            purpose: NEW_WINDOW_PURPOSE,
            effect: Effect::Change,
            args: &[],
            reaches: Reaches::OnlyItsApplication,
            sentence: NEW_WINDOW_SENTENCE,
            by_hand: Some(NEW_WINDOW_BY_HAND),
            carried_out: Invocation::DBus(DBusMethod {
                object: OBJECT,
                interface: INTERFACE,
                method: "ActivateAction",
                parameters: &[
                    Part::Literal("new-window"),
                    Part::NoParameters,
                    Part::NoPlatformData,
                ],
            }),
        },
    ],
};
