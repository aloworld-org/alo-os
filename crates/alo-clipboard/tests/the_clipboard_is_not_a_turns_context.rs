//! What a person copied is **not** what an agent is offered, and a turn cannot
//! reach it.
//!
//! ADR 0001 §4 offers an agent three things at the moment it is invoked and for
//! that turn only: the focused window, the highlighted text and the open
//! document. The clipboard is none of the three. `CLAUDE.md` makes the same
//! point as a standing rule — *context is offered, never watched* — and a turn
//! that read what a person had copied would be the background reader that rule
//! calls a bug in this product.
//!
//! It matters more here than the general case. A clipboard is where a password
//! manager puts a password, for the thirty seconds between one window and the
//! next. An agent that could read it would be an agent that sees every secret a
//! person moves, without a grant, without an approval, and without anything
//! appearing in a record — because nothing about a person copying is an agent
//! doing something.
//!
//! # What this file measures, and why one half of it reads a manifest
//!
//! The behaviour is measured first: a whole turn is begun over a real context
//! while there is a real selection on the clipboard, and the application that
//! owns that selection is never asked for anything.
//!
//! But a behaviour test can only show that today's turn does not reach it. What
//! stops tomorrow's is that **neither crate can name the other**: there is no
//! `alo-clipboard` in `alo-context`'s dependencies and no `alo-context` in this
//! crate's, so a turn cannot reach a clipboard because there is no clipboard in
//! scope to reach. That is checkable, it is the guarantee rather than the
//! evidence, and it is why the second test reads two manifests off the disk.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee, Grants};
use alo_clipboard::{Clipboard, CouldNotGive, Gives, Kind, Offer};
use alo_context::{Context, Document, Focused, Selection, Turn};
use alo_strings::Strings;

/// The moment this file is written against.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// What a person copied, which nothing in a turn may ever see.
const THE_PASSWORD: &str = "correct-horse-battery-staple";

/// The application that owns the selection, which answers if anybody asks — and
/// says so, so that a test can tell the difference between *nobody asked* and
/// *nothing came back*.
struct APasswordManager {
    /// Whether it was ever asked for anything.
    asked: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Gives for APasswordManager {
    fn give(&mut self, _form: &Kind) -> Result<Vec<u8>, CouldNotGive> {
        self.asked.set(true);
        Ok(THE_PASSWORD.as_bytes().to_vec())
    }
}

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .expect("this crate is two directories below the repository")
}

/// A manifest of this repository, read.
fn manifest(crate_named: &str) -> String {
    let at = the_repository()
        .join("crates")
        .join(crate_named)
        .join("Cargo.toml");
    fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// **A turn reaches nothing on the clipboard.**
///
/// Everything a turn is entitled to is offered: a focused window, a selection
/// the person highlighted, and an open document. Meanwhile a password manager
/// owns the clipboard. The turn begins, grants what it grants, and ends — and
/// the application that owns the selection is never asked for anything, because
/// there is no path from a turn to it.
#[test]
fn a_turn_reaches_nothing_on_the_clipboard() {
    let asked = std::rc::Rc::new(std::cell::Cell::new(false));
    let mut clipboard = Clipboard::nothing_copied_yet();
    clipboard.taken(
        Offer::copied(vec![Kind::text()]).expect("one form is an offer"),
        Box::new(APasswordManager {
            asked: std::rc::Rc::clone(&asked),
        }),
    );

    let context = Context::at_invocation(noon())
        .and_window(Focused::titled("org.gnome.TextEditor", "notes.md").expect("a window"))
        .and_selection(Selection::of("the paragraph they highlighted").expect("a selection"))
        .and_document(Document::open("/home/anna/notes.md").expect("a document"));

    let mut grants = Grants::default();
    let turn = Turn::beginning(context, "@files", Duration::from_secs(300), &mut grants)
        .expect("a turn with a document grants the document");

    // The one thing the turn granted is the document. Not the window, not the
    // highlighted text, and nothing of what is on the clipboard — which has no
    // shape a grant could even be written over.
    let agent = Grantee::named("@files");
    assert!(grants.permits(&agent, &Ask::path("/home/anna/notes.md"), noon()));
    assert!(!grants.permits(&agent, &Ask::path("/home/anna"), noon()));
    assert!(turn.granted().is_some());

    // What a person is shown of what they offered names three things, and what
    // they copied is not one of them.
    let strings = Strings::of(
        alo_saying::everything_this_machine_can_say().expect("alo OS's own words are collected"),
    );
    let shown: Vec<String> = turn
        .context()
        .shown(&strings)
        .iter()
        .map(|said| said.text().to_owned())
        .collect();
    assert_eq!(shown.len(), 3, "{shown:?}");
    for row in &shown {
        assert!(!row.contains(THE_PASSWORD), "{row}");
    }

    assert!(turn.ending(&mut grants));
    assert!(
        !asked.get(),
        "a turn asked the clipboard's owner for something"
    );

    // And the selection is still there afterwards, untouched by any of it: a
    // turn neither reads it nor spends it.
    let offered = clipboard.to_paste().expect("the selection is still there");
    assert!(offered.offers(&Kind::text()));
    assert!(!asked.get());
}

/// **Neither crate can name the other**, which is what stops tomorrow's turn
/// rather than today's.
///
/// `alo-context` does not depend on this crate at all; this crate names
/// `alo-context` only under `[dev-dependencies]`, where it is reachable from
/// the test above and from nothing that ships. A dependency in either direction
/// would be the first line of the background reader ADR 0001 §4 forbids, and it
/// would be added by somebody with a good reason — which is exactly why it is
/// worth failing a gate over.
#[test]
fn neither_the_clipboard_nor_a_context_can_name_the_other() {
    // A dependency is a line that names a crate and opens its table, rather
    // than any mention of the name: both of these manifests argue in prose
    // about the other crate, which is the argument being kept beside the thing
    // it is about and not something a check should be confused by.
    let depends_on = |manifest: &str, crate_named: &str| {
        manifest
            .lines()
            .any(|line| line.trim_start().starts_with(&format!("{crate_named} = ")))
    };

    assert!(
        !depends_on(&manifest("alo-context"), "alo-clipboard"),
        "alo-context depends on alo-clipboard: what a person copied is not what \
         an agent is offered (ADR 0001 §4)"
    );

    let ours = manifest("alo-clipboard");
    let (ships, tests) = ours
        .split_once("[dev-dependencies]")
        .expect("this crate has dev-dependencies");
    assert!(
        !depends_on(ships, "alo-context"),
        "alo-clipboard depends on alo-context outside its tests: a clipboard a \
         turn can reach is the background reader ADR 0001 §4 forbids"
    );
    assert!(
        depends_on(tests, "alo-context"),
        "the test above does not measure what it says it measures"
    );

    // And nothing that ships reaches the agent's side of the machine at all:
    // no daemon, no turn, no record, no grants, no questions.
    for elsewhere in [
        "alo-agentd",
        "alo-turn",
        "alo-record",
        "alo-capability",
        "alo-asking",
    ] {
        assert!(
            !depends_on(ships, elsewhere),
            "alo-clipboard depends on {elsewhere}, and copy and paste is a \
             person moving their own text between their own windows"
        );
    }
}
