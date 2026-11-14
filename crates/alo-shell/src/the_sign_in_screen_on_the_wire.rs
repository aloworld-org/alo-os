//! **What crosses between the sign-in screen and the reader that speaks it**
//! ([ADR 0155](../../../docs/decisions/0155-the-screens-before-sign-in-speak.md) §2).
//!
//! The compositor draws the screens before sign-in as root, with no
//! capabilities, and never touches the sign-in login's buses. A relay of alo's
//! own runs as that login (`alo-reading-the-sign-in-screen`) and is the only
//! thing that puts the screen on its accessibility bus. Between the two is one
//! Unix socket, and **this is everything that may cross it**: one line of JSON
//! per message, each way.
//!
//! - **To the reader**: the screen, whole, as `crate::the_screens_before_sign_in_as_read`
//!   built it; whether to read it aloud, and in which language; and that a
//!   person is signing in, so it steps aside.
//! - **From the reader**: what starting or stopping the voice came to, and that
//!   it has stepped aside.
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

use serde_json::{Map, Value, json};

use crate::TheScreenReaderAnswered;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_sheet_in_front::{ASheetInFront, TheSheet};
use crate::the_screens_before_sign_in_as_read::ScreenBeforeSignIn;

/// The longest line either side reads, in bytes.
pub const THE_LONGEST_LINE: usize = 4 * 1024 * 1024;

/// The most nodes a screen may have on the wire.
pub const THE_MOST_NODES: usize = 4096;

/// The longest name, description or attribute, in bytes.
pub const THE_LONGEST_WORDS: usize = 4096;

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
    /// It is not JSON.
    #[error("the line is not JSON")]
    NotJson,
    /// It is JSON and not one of the messages.
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
        let value = match self {
            Self::TheScreen(tree) => json!({ "screen": the_tree_written(tree) }),
            Self::Read { on, language } => json!({ "read": { "on": on, "language": language } }),
            Self::SomebodyIsSigningIn => json!({ "signing-in": true }),
        };
        format!("{value}\n")
    }

    /// The message on `line`.
    ///
    /// # Errors
    /// [`NotUnderstood`], and nothing of the line is used.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        let object = an_object(line)?;
        if let Some(screen) = object.get("screen") {
            return the_tree_read(screen).map(Self::TheScreen);
        }
        if let Some(read) = object.get("read") {
            let on = read
                .get("on")
                .and_then(Value::as_bool)
                .ok_or(NotUnderstood::NotAMessage)?;
            let language = match read.get("language") {
                None | Some(Value::Null) => None,
                Some(Value::String(tag)) if a_language_tag(tag) => Some(tag.clone()),
                Some(_) => return Err(NotUnderstood::NotAMessage),
            };
            return Ok(Self::Read { on, language });
        }
        if object.get("signing-in") == Some(&Value::Bool(true)) {
            return Ok(Self::SomebodyIsSigningIn);
        }
        Err(NotUnderstood::NotAMessage)
    }
}

impl FromTheReader {
    /// One line, ending in a newline, holding no other.
    #[must_use]
    pub fn written(&self) -> String {
        let value = match self {
            Self::Answered(answered) => json!({ "answered": an_answers_word(*answered) }),
            Self::SteppedAside => json!({ "stepped-aside": true }),
        };
        format!("{value}\n")
    }

    /// The message on `line`.
    ///
    /// # Errors
    /// [`NotUnderstood`], and nothing of the line is used.
    pub fn read(line: &str) -> Result<Self, NotUnderstood> {
        let object = an_object(line)?;
        if let Some(word) = object.get("answered").and_then(Value::as_str) {
            return the_answer_named(word)
                .map(Self::Answered)
                .ok_or(NotUnderstood::NotAMessage);
        }
        if object.get("stepped-aside") == Some(&Value::Bool(true)) {
            return Ok(Self::SteppedAside);
        }
        Err(NotUnderstood::NotAMessage)
    }
}

/// The JSON object on `line`, or why there is none.
fn an_object(line: &str) -> Result<Map<String, Value>, NotUnderstood> {
    if line.len() > THE_LONGEST_LINE {
        return Err(NotUnderstood::TooLong);
    }
    match serde_json::from_str::<Value>(line.trim_end()) {
        Ok(Value::Object(object)) => Ok(object),
        Ok(_) => Err(NotUnderstood::NotAMessage),
        Err(_) => Err(NotUnderstood::NotJson),
    }
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

/// The tree, as JSON: every node in order with what it hangs under, and the
/// screen in front. Paths and children are not sent: both follow from the
/// order, and a receiver that took them from the wire could be told two
/// different shapes at once.
fn the_tree_written(tree: &ReadAloudTree) -> Value {
    let nodes: Vec<Value> = tree
        .nodes()
        .iter()
        .map(|node| {
            json!({
                "role": node.role,
                "name": node.name,
                "description": node.description,
                "states": node.states,
                "announced": node.announced,
                "parent": node.parent,
                "attributes": node.attributes,
            })
        })
        .collect();
    let in_front = tree.in_front().and_then(|sheet| match sheet.which {
        TheSheet::BeforeSignIn(screen) => Some(json!({
            "screen": a_screens_word(screen),
            "at": sheet.at,
            "keyboard": sheet.keyboard,
        })),
        TheSheet::Password(_)
        | TheSheet::OpenWith(_)
        | TheSheet::GitHub(_)
        | TheSheet::Settings => None,
    });
    json!({ "nodes": nodes, "in-front": in_front })
}

/// A tree read off the wire, or why it is not one.
fn the_tree_read(value: &Value) -> Result<ReadAloudTree, NotUnderstood> {
    let nodes = value
        .get("nodes")
        .and_then(Value::as_array)
        .ok_or(NotUnderstood::NotAScreen("it has no nodes"))?;
    if nodes.is_empty() {
        return Err(NotUnderstood::NotAScreen("it has no nodes"));
    }
    if nodes.len() > THE_MOST_NODES {
        return Err(NotUnderstood::NotAScreen(
            "it has more nodes than any screen",
        ));
    }
    let mut tree = ReadAloudTree::with_nothing_on_it();
    for (at, node) in nodes.iter().enumerate() {
        let read = a_node_read(node)?;
        if at == 0 {
            if let Some(root) = tree.node_mut(0) {
                *root = Node {
                    path: root.path.clone(),
                    ..read
                };
            }
            continue;
        }
        if read.parent >= at {
            return Err(NotUnderstood::NotAScreen(
                "a node hangs under one that comes after it",
            ));
        }
        tree.push(read.parent, read);
    }
    match value.get("in-front") {
        None | Some(Value::Null) => {}
        Some(in_front) => {
            let screen = in_front
                .get("screen")
                .and_then(Value::as_str)
                .and_then(the_screen_named)
                .ok_or(NotUnderstood::NotAScreen("the screen in front has no name"))?;
            let place = |key: &str| {
                in_front
                    .get(key)
                    .and_then(Value::as_u64)
                    .and_then(|it| usize::try_from(it).ok())
                    .filter(|it| *it < nodes.len())
                    .ok_or(NotUnderstood::NotAScreen(
                        "the screen in front is not one of its nodes",
                    ))
            };
            tree.put_in_front(ASheetInFront {
                which: TheSheet::BeforeSignIn(screen),
                at: place("at")?,
                keyboard: place("keyboard")?,
            });
        }
    }
    Ok(tree)
}

/// One node read off the wire, with nothing under it yet.
fn a_node_read(value: &Value) -> Result<Node, NotUnderstood> {
    let number = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_u64)
            .ok_or(NotUnderstood::NotAScreen("a node is missing a number"))
    };
    let words = |key: &str| {
        value
            .get(key)
            .and_then(Value::as_str)
            .filter(|it| it.len() <= THE_LONGEST_WORDS)
            .map(str::to_owned)
            .ok_or(NotUnderstood::NotAScreen(
                "a node's words are missing or too long",
            ))
    };
    let a_word = |it: u64| u32::try_from(it).ok();
    let role = a_word(number("role")?).ok_or(NotUnderstood::NotAScreen("a role is too large"))?;
    let states = match value
        .get("states")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
    {
        Some([low, high]) => [
            low.as_u64().and_then(a_word),
            high.as_u64().and_then(a_word),
        ],
        _ => [None, None],
    };
    let [Some(low), Some(high)] = states else {
        return Err(NotUnderstood::NotAScreen(
            "a node's states are not two words",
        ));
    };
    let parent = usize::try_from(number("parent")?)
        .map_err(|_| NotUnderstood::NotAScreen("a parent is too large"))?;
    let announced =
        value
            .get("announced")
            .and_then(Value::as_bool)
            .ok_or(NotUnderstood::NotAScreen(
                "a node does not say whether it is announced",
            ))?;
    let mut attributes = Vec::new();
    for pair in value
        .get("attributes")
        .and_then(Value::as_array)
        .ok_or(NotUnderstood::NotAScreen("a node has no attributes"))?
    {
        let (Some(key), Some(said)) = (
            pair.get(0).and_then(Value::as_str),
            pair.get(1).and_then(Value::as_str),
        ) else {
            return Err(NotUnderstood::NotAScreen(
                "an attribute is not a key and a value",
            ));
        };
        if key.len() > THE_LONGEST_WORDS || said.len() > THE_LONGEST_WORDS {
            return Err(NotUnderstood::NotAScreen("an attribute is too long"));
        }
        attributes.push((key.to_owned(), said.to_owned()));
    }
    Ok(Node {
        role,
        name: words("name")?,
        description: words("description")?,
        states: [low, high],
        announced,
        parent,
        attributes,
        ..Node::default()
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "the_sign_in_screen_on_the_wire_tests.rs"]
mod tests;
