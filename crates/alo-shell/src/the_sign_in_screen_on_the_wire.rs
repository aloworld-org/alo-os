//! **What crosses between the sign-in screen and the reader that speaks it**
//! ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md) §2).
//!
//! The compositor draws the screens before sign-in as root, with no
//! capabilities, and never touches the sign-in login's buses. A relay of alo's
//! own runs as that login (`alo-reading-the-sign-in-screen`) and is the only
//! thing that puts the screen on its accessibility bus. Between the two is one
//! Unix socket, and **this is everything that may cross it**: one line per
//! message, each way.
//!
//! - **To the reader**: the screen, whole, as `crate::the_screens_before_sign_in_as_read`
//!   built it; whether to read it aloud, and in which language; and that a
//!   person is signing in, so it steps aside.
//! - **From the reader**: what starting or stopping the voice came to, and that
//!   it has stepped aside.
//!
//! # The shape of a line
//!
//! Fields are separated by tabs; a node's own fields by commas, its attributes
//! by semicolons and each attribute's key from its value by `=`. Those five
//! characters, the newline, the carriage return and `%` itself are written as
//! `%` and two hex digits wherever they appear in a word. **Not JSON, and on
//! purpose**: the shipped shell is built with no serialiser at all
//! (`tests/settings_source.rs`), so that no setting can be written except by
//! the crate that owns it — and a screen sent over a socket is not worth
//! weakening that for.
//!
//! # Data, never code, and nothing typed
//!
//! Nothing here names a command, a path or a program. The tree is roles,
//! names, states and places — the same facts the accessibility bus carries to
//! anybody on it — and it was built from `crate::SignInShows`, which holds
//! whether a secret is typed and never the secret. A message that is not one
//! of these, or a tree that does not hang together, is refused whole
//! ([`NotUnderstood`]), so a half-read message is never acted on.
//!
//! # Bounded
//!
//! A line longer than [`THE_LONGEST_LINE`], a tree of more than
//! [`THE_MOST_NODES`] or a word longer than [`THE_LONGEST_WORDS`] is refused:
//! the largest screen alo draws — every region on the region sheet — is a few
//! hundred nodes of a few dozen bytes each.

use crate::TheScreenReaderAnswered;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_screens_before_sign_in_as_read::ScreenBeforeSignIn;

/// The longest line either side reads, in bytes.
pub const THE_LONGEST_LINE: usize = 4 * 1024 * 1024;

/// The most nodes a screen may have on the wire.
pub const THE_MOST_NODES: usize = 4096;

/// The longest name, description or attribute, in bytes, as it is read.
pub const THE_LONGEST_WORDS: usize = 4096;

/// What stands for *none* in a field that may hold nothing.
const NONE: &str = "-";

/// **What the sign-in screen tells its reader.**
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToTheReader {
    /// The screen now, as a reader is told it.
    TheScreen(ReadAloudTree),
    /// Read the screen aloud, in `language` where one is named, or stop.
    Read {
        /// Whether to read.
        on: bool,
        /// The language to read in, as a tag: `de`, `pt-PT`.
        language: Option<String>,
    },
    /// A person is signing in: stop reading, and say when that is done.
    SomebodyIsSigningIn,
}

/// **What the reader tells the sign-in screen.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FromTheReader {
    /// What starting or stopping the voice came to.
    Answered(TheScreenReaderAnswered),
    /// The voice has stopped for the person's own session.
    SteppedAside,
}

/// Why a line was not understood. None of these is acted on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotUnderstood {
    /// It is longer than [`THE_LONGEST_LINE`].
    #[error("the line is longer than either side reads")]
    TooLong,
    /// It is not one of the messages.
    #[error("the line is not a message the sign-in screen and its reader exchange")]
    NotAMessage,
    /// It is a screen that does not hang together, for this reason.
    #[error("the screen sent does not hang together: {0}")]
    NotAScreen(&'static str),
}

impl ToTheReader {
    /// One line, ending in a newline, holding no other.
    #[must_use]
    pub fn written(&self) -> String {
        let line = match self {
            Self::TheScreen(tree) => the_tree_written(tree),
            Self::Read { on, language } => format!(
                "read\t{}\t{}",
                if *on { "on" } else { "off" },
                language.as_deref().map_or_else(|| NONE.to_owned(), escaped)
            ),
            Self::SomebodyIsSigningIn => "signing-in".to_owned(),
        };
        format!("{line}\n")
    }

    /// The message on `line`.
    ///
    /// # Errors
    /// [`NotUnderstood`], and nothing of the line is used.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        let fields = the_fields_of(line)?;
        match fields.as_slice() {
            ["signing-in"] => Ok(Self::SomebodyIsSigningIn),
            ["read", on, language] => {
                let on = match *on {
                    "on" => true,
                    "off" => false,
                    _ => return Err(NotUnderstood::NotAMessage),
                };
                let language = match *language {
                    NONE => None,
                    tag => {
                        let tag = unescaped(tag).ok_or(NotUnderstood::NotAMessage)?;
                        if !a_language_tag(&tag) {
                            return Err(NotUnderstood::NotAMessage);
                        }
                        Some(tag)
                    }
                };
                Ok(Self::Read { on, language })
            }
            ["screen", rest @ ..] => the_tree_read(rest).map(Self::TheScreen),
            _ => Err(NotUnderstood::NotAMessage),
        }
    }
}

impl FromTheReader {
    /// One line, ending in a newline, holding no other.
    #[must_use]
    pub fn written(&self) -> String {
        match self {
            Self::Answered(answered) => format!("answered\t{}\n", an_answers_word(*answered)),
            Self::SteppedAside => "stepped-aside\n".to_owned(),
        }
    }

    /// The message on `line`.
    ///
    /// # Errors
    /// [`NotUnderstood`], and nothing of the line is used.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        match the_fields_of(line)?.as_slice() {
            ["answered", word] => the_answer_named(word)
                .map(Self::Answered)
                .ok_or(NotUnderstood::NotAMessage),
            ["stepped-aside"] => Ok(Self::SteppedAside),
            _ => Err(NotUnderstood::NotAMessage),
        }
    }
}

/// The tab-separated fields of one line, its newline taken off.
fn the_fields_of(line: &str) -> Result<Vec<&str>, NotUnderstood> {
    if line.len() > THE_LONGEST_LINE {
        return Err(NotUnderstood::TooLong);
    }
    let line = line.strip_suffix('\n').unwrap_or(line);
    let line = line.strip_suffix('\r').unwrap_or(line);
    if line.contains('\n') {
        return Err(NotUnderstood::NotAMessage);
    }
    Ok(line.split('\t').collect())
}

/// The characters a word may not hold as they are.
const SPECIAL: [char; 8] = ['%', '\t', '\n', '\r', ',', ';', '=', '-'];

/// A word, with every special character written as `%` and two hex digits.
/// A word that is exactly [`NONE`] is escaped too, so it is never read as none.
fn escaped(word: &str) -> String {
    let mut out = String::with_capacity(word.len());
    for letter in word.chars() {
        if SPECIAL.contains(&letter) {
            out.push_str(&format!("%{:02X}", u32::from(letter)));
        } else {
            out.push(letter);
        }
    }
    out
}

/// A word as it was before [`escaped`], or [`None`] where it is not one.
fn unescaped(word: &str) -> Option<String> {
    let mut out = String::with_capacity(word.len());
    let mut letters = word.chars();
    while let Some(letter) = letters.next() {
        if letter != '%' {
            if SPECIAL.contains(&letter) {
                return None;
            }
            out.push(letter);
            continue;
        }
        let high = letters.next()?.to_digit(16)?;
        let low = letters.next()?.to_digit(16)?;
        let it = char::from_u32(high * 16 + low)?;
        if !SPECIAL.contains(&it) {
            return None;
        }
        out.push(it);
    }
    (out.len() <= THE_LONGEST_WORDS).then_some(out)
}

/// **Only a language tag**: letters, with a region after a hyphen or an
/// underscore at most — so nothing but a language ever reaches the file the
/// reader's environment is made from.
fn a_language_tag(tag: &str) -> bool {
    let mut parts = tag.split(['-', '_']);
    let letters = |part: Option<&str>, from: usize, to: usize| {
        part.is_some_and(|part| {
            (from..=to).contains(&part.len()) && part.bytes().all(|b| b.is_ascii_alphabetic())
        })
    };
    let language = parts.next();
    let region = parts.next();
    letters(language, 2, 3) && (region.is_none() || letters(region, 2, 3)) && parts.next().is_none()
}

/// What an answer is called on the wire.
const fn an_answers_word(answered: TheScreenReaderAnswered) -> &'static str {
    match answered {
        TheScreenReaderAnswered::Started => "started",
        TheScreenReaderAnswered::Stopped => "stopped",
        TheScreenReaderAnswered::NotOnThisComputer => "not-on-this-computer",
        TheScreenReaderAnswered::DidNotStart => "did-not-start",
        TheScreenReaderAnswered::DidNotStop => "did-not-stop",
    }
}

/// The answer a word on the wire names.
fn the_answer_named(word: &str) -> Option<TheScreenReaderAnswered> {
    [
        TheScreenReaderAnswered::Started,
        TheScreenReaderAnswered::Stopped,
        TheScreenReaderAnswered::NotOnThisComputer,
        TheScreenReaderAnswered::DidNotStart,
        TheScreenReaderAnswered::DidNotStop,
    ]
    .into_iter()
    .find(|answered| an_answers_word(*answered) == word)
}

/// What a screen is called on the wire.
const fn a_screens_word(screen: ScreenBeforeSignIn) -> &'static str {
    match screen {
        ScreenBeforeSignIn::Overview => "overview",
        ScreenBeforeSignIn::Language => "language",
        ScreenBeforeSignIn::Region => "region",
        ScreenBeforeSignIn::Zone => "zone",
        ScreenBeforeSignIn::Keyboard => "keyboard",
        ScreenBeforeSignIn::Name => "name",
        ScreenBeforeSignIn::Pin => "pin",
        ScreenBeforeSignIn::Password => "password",
        ScreenBeforeSignIn::SignIn => "sign-in",
        ScreenBeforeSignIn::Sentence => "sentence",
    }
}

/// The screen a word on the wire names.
fn the_screen_named(word: &str) -> Option<ScreenBeforeSignIn> {
    [
        ScreenBeforeSignIn::Overview,
        ScreenBeforeSignIn::Language,
        ScreenBeforeSignIn::Region,
        ScreenBeforeSignIn::Zone,
        ScreenBeforeSignIn::Keyboard,
        ScreenBeforeSignIn::Name,
        ScreenBeforeSignIn::Pin,
        ScreenBeforeSignIn::Password,
        ScreenBeforeSignIn::SignIn,
        ScreenBeforeSignIn::Sentence,
    ]
    .into_iter()
    .find(|screen| a_screens_word(*screen) == word)
}

/// The tree, as a line: the screen in front (`-` for none), where it is and
/// where the keyboard is, then every node in order with what it hangs under.
/// Paths and children are not sent: both follow from the order, and a
/// receiver that took them from the wire could be told two different shapes
/// at once.
fn the_tree_written(tree: &ReadAloudTree) -> String {
    let in_front = tree.in_front().and_then(|sheet| match sheet.which {
        TheSheet::BeforeSignIn(screen) => Some(format!(
            "{}\t{}\t{}",
            a_screens_word(screen),
            sheet.at,
            sheet.keyboard
        )),
        TheSheet::Password(_)
        | TheSheet::OpenWith(_)
        | TheSheet::GitHub(_)
        | TheSheet::Settings => None,
    });
    let mut line = format!(
        "screen\t{}",
        in_front.unwrap_or_else(|| format!("{NONE}\t{NONE}\t{NONE}"))
    );
    for node in tree.nodes() {
        let attributes: Vec<String> = node
            .attributes
            .iter()
            .map(|(key, value)| format!("{}={}", escaped(key), escaped(value)))
            .collect();
        let [low, high] = node.states;
        line.push_str(&format!(
            "\t{},{},{},{},{},{},{},{}",
            node.role,
            low,
            high,
            u8::from(node.announced),
            node.parent,
            escaped(&node.name),
            escaped(&node.description),
            attributes.join(";")
        ));
    }
    line
}

/// A tree read off the wire — the fields after `screen` — or why it is not one.
fn the_tree_read(fields: &[&str]) -> Result<ReadAloudTree, NotUnderstood> {
    let &[front, at, keyboard, ref nodes @ ..] = fields else {
        return Err(NotUnderstood::NotAScreen(
            "it says nothing of what is in front",
        ));
    };
    if nodes.is_empty() {
        return Err(NotUnderstood::NotAScreen("it has no nodes"));
    }
    if nodes.len() > THE_MOST_NODES {
        return Err(NotUnderstood::NotAScreen(
            "it has more nodes than any screen",
        ));
    }
    let mut tree = ReadAloudTree::with_nothing_on_it();
    for (place, node) in nodes.iter().enumerate() {
        let read = a_node_read(node)?;
        if place == 0 {
            if let Some(root) = tree.node_mut(0) {
                *root = Node {
                    path: root.path.clone(),
                    ..read
                };
            }
            continue;
        }
        if read.parent >= place {
            return Err(NotUnderstood::NotAScreen(
                "a node hangs under one that comes after it",
            ));
        }
        tree.push(read.parent, read);
    }
    if front != NONE {
        let screen = the_screen_named(front)
            .ok_or(NotUnderstood::NotAScreen("the screen in front has no name"))?;
        let place = |field: &str| {
            field
                .parse::<usize>()
                .ok()
                .filter(|it| *it < nodes.len())
                .ok_or(NotUnderstood::NotAScreen(
                    "the screen in front is not one of its nodes",
                ))
        };
        tree.put_in_front(ASheetInFront {
            which: TheSheet::BeforeSignIn(screen),
            at: place(at)?,
            keyboard: place(keyboard)?,
        });
    }
    Ok(tree)
}

/// One node read off the wire, with nothing under it yet.
fn a_node_read(field: &str) -> Result<Node, NotUnderstood> {
    let parts: Vec<&str> = field.split(',').collect();
    let &[
        role,
        low,
        high,
        announced,
        parent,
        name,
        description,
        attributes,
    ] = parts.as_slice()
    else {
        return Err(NotUnderstood::NotAScreen("a node is not eight fields"));
    };
    let a_word = |it: &str| {
        it.parse::<u32>()
            .map_err(|_| NotUnderstood::NotAScreen("a node's number is not one"))
    };
    let words = |it: &str| {
        unescaped(it).ok_or(NotUnderstood::NotAScreen(
            "a node's words are not written as words, or are too long",
        ))
    };
    let announced = match announced {
        "0" => false,
        "1" => true,
        _ => {
            return Err(NotUnderstood::NotAScreen(
                "a node does not say whether it is announced",
            ));
        }
    };
    let parent = parent
        .parse::<usize>()
        .map_err(|_| NotUnderstood::NotAScreen("a parent is not a place"))?;
    let mut read_attributes = Vec::new();
    if !attributes.is_empty() {
        for pair in attributes.split(';') {
            let Some((key, value)) = pair.split_once('=') else {
                return Err(NotUnderstood::NotAScreen(
                    "an attribute is not a key and a value",
                ));
            };
            read_attributes.push((words(key)?, words(value)?));
        }
    }
    Ok(Node {
        role: a_word(role)?,
        name: words(name)?,
        description: words(description)?,
        states: [a_word(low)?, a_word(high)?],
        announced,
        parent,
        attributes: read_attributes,
        ..Node::default()
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_sign_in_screen_on_the_wire_tests.rs"]
mod tests;
