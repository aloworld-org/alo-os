//! **A screen reader hears the text alo will type, whole, with every hidden
//! character named** — on the approval card in the task window that is
//! asking.
//!
//! The card draws a tag, *U+200B*, where a character a person could miss
//! stands (`crate::the_text_alo_will_type`). A reader is told it in words, in
//! its place: *invisible character U+200B*, in the person's language
//! (`alo_capability::Unseen::said`). Nothing is cut and nothing is summarised:
//! what a reader hears is every line, which is what the card scrolls through.
//!
//! # Where it hangs
//!
//! Under the asking task's window — the node whose *Close* closes that task —
//! as a dialogue named by the sentence the person approves, holding how long
//! the text is, how much is marked, and the text. A task window that is not in
//! the tree, or a change with no text, leaves the tree as it was.

use alo_capability::{Piece, TheWholeText, WhyMarked};
use alo_strings::Strings;

use crate::access_action::Pressed;
use crate::access_nodes::{Node, ReadAloudTree};
use crate::access_roles;
use crate::the_tasks_a_person_opened::WhichTask;

/// What a reader hears of one text: every line, each hidden character named
/// in words where it stands.
#[must_use]
pub(crate) fn as_heard(text: &TheWholeText, strings: &Strings) -> String {
    let mut heard = String::new();
    for (at, line) in text.lines().iter().enumerate() {
        if at > 0 {
            heard.push('\n');
        }
        for piece in line.pieces() {
            match piece {
                Piece::Seen(words) => heard.push_str(words),
                Piece::Unseen(unseen) => {
                    if unseen.why() == WhyMarked::OtherAlphabet {
                        heard.push(unseen.character());
                    }
                    heard.push_str(" (");
                    heard.push_str(&unseen.said(strings).into_text());
                    heard.push_str(") ");
                }
            }
        }
    }
    heard
}

impl ReadAloudTree {
    /// The same tree, with the text the change waiting would type under the
    /// task window `which` is asking from.
    #[must_use]
    pub(crate) fn with_the_text_alo_will_type(
        mut self,
        strings: &Strings,
        which: WhichTask,
        sentence: &str,
        texts: &[TheWholeText],
    ) -> Self {
        if texts.is_empty() {
            return self;
        }
        let Some(window) = self
            .nodes()
            .iter()
            .find(|node| {
                node.press
                    == Some(Pressed::on_a_task_window(
                        which,
                        crate::window_edge::OnTheEdge::Close,
                    ))
            })
            .map(|node| node.parent)
        else {
            return self;
        };
        let read_only = || access_roles::words_of(alo_access::State::ReadOnly, None);
        let card = self.push(
            window,
            Node {
                role: access_roles::number_of(alo_access::Role::Dialogue),
                name: sentence.to_owned(),
                states: read_only(),
                ..Node::default()
            },
        );
        for text in texts {
            let mut said = vec![text.how_long(strings).into_text()];
            said.extend(
                text.what_is_marked(strings)
                    .map(alo_strings::Said::into_text),
            );
            for line in said {
                self.push(
                    card,
                    Node {
                        role: access_roles::number_of(alo_access::Role::Label),
                        name: line,
                        states: read_only(),
                        ..Node::default()
                    },
                );
            }
            self.push(
                card,
                Node {
                    role: access_roles::number_of(alo_access::Role::Label),
                    name: as_heard(text, strings),
                    states: read_only(),
                    ..Node::default()
                },
            );
        }
        self
    }
}

#[cfg(test)]
#[path = "the_text_alo_will_type_as_read_tests.rs"]
mod tests;
