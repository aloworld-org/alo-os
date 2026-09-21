//! One thing a notification offers the person: a name its sender knows it by,
//! and words for the person to read.
//!
//! **Two fields, and there is no third.** That is the whole of this file's
//! argument, and it is ADR 0001's: a change to the machine is proposed with a
//! sentence and waits for one approval on the approval surface, and *what a
//! person approves is that sentence*. A notification is not that surface. It
//! arrives uninvited, it is drawn wherever the shell draws notifications, and
//! anything on it that could answer a waiting proposal would be an approval
//! obtained somewhere the person did not go to approve anything.
//!
//! So an [`Action`] carries no proposal, no verb, no argument, no grant and no
//! answer. There is nothing in it but the sender's own name for it and the
//! words on it, and picking one produces a [`crate::Picked`] addressed back to
//! the sender, whose half of the argument is made in `src/picked.rs`. None of
//! those is a thing an approval could be hiding in:
//!
//! ```compile_fail
//! let answering = alo_notifying::Action::approving(some_proposal_id);
//! ```
//!
//! ```compile_fail
//! let running = alo_notifying::Action::calling("delete_file", &[("file", "/home/ada/notes")]);
//! ```
//!
//! `tests/a_notification_cannot_answer_an_approval.rs` is the same argument
//! made against a proposal that is really waiting, and against this crate's own
//! manifest and shipped source.
//!
//! # Why the sender's name is never shown
//!
//! [`Action::named`] is what the sender calls it — `mark-as-read`, `reply` —
//! and it exists so that a picked action can be handed back to the program that
//! sent the notification. Nobody reads it. What a person reads is
//! [`Action::label`], which is the sender's own text in the sender's own
//! language and is not ours to translate.

/// How many things one notification may offer a person.
///
/// Three. A notification is a small thing that arrives while somebody is doing
/// something else, and it is read in the corner of an eye; a program that wants
/// to put a menu in front of a person has a window for that. The number is here
/// rather than in the shell because it is a decision about what a notification
/// *is*, and a shell that drew four would be drawing something this crate would
/// never have accepted.
pub const MOST_THINGS_TO_DO: usize = 3;

/// Why what was offered is not something a notification can offer.
///
/// Carried out of here without words: what a person reads names the program
/// that sent it, and the program is known where the notification is judged
/// rather than here ([`crate::NotSent`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAnAction {
    /// The sender gave it no name of its own, so there would be nothing to
    /// hand back if somebody picked it.
    NoName,
    /// There is nothing on it for a person to read.
    NoLabel,
}

/// One thing a notification offers the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Action {
    /// What its sender calls it. Handed back to the sender when the person
    /// picks it, and shown to nobody.
    named: String,
    /// What the person reads on it, in the sender's own language.
    label: String,
}

impl Action {
    /// Something to offer: the sender's own name for it, and the words on it.
    ///
    /// # Errors
    /// [`NotAnAction::NoName`] when the sender named it nothing, and
    /// [`NotAnAction::NoLabel`] when there is nothing on it a person could
    /// read — which includes a label made only of characters that would
    /// rewrite the line it is drawn on.
    pub fn offering(named: &str, label: &str) -> Result<Self, NotAnAction> {
        let named = named.trim();
        if named.is_empty() || named.chars().any(char::is_control) {
            return Err(NotAnAction::NoName);
        }
        let label = label.trim();
        if label.is_empty() || label.chars().any(char::is_control) {
            return Err(NotAnAction::NoLabel);
        }
        Ok(Self {
            named: named.to_owned(),
            label: label.to_owned(),
        })
    }

    /// What its sender calls it, which is what is handed back when a person
    /// picks it and is shown to nobody.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }

    /// What the person reads on it.
    ///
    /// The sender's own text, in whatever language the sender wrote it in. It
    /// is not translated and it is not ours to translate: a `&str` rather than
    /// a `Said`, the rule `alo_applications::Application` holds a name to and
    /// `alo_files` holds a filename to.
    #[must_use]
    pub fn label(&self) -> &str {
        &self.label
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **An action is a name and a label**, trimmed, and nothing else.
    #[test]
    fn an_action_is_a_name_its_sender_knows_it_by_and_words_to_read() {
        let action = Action::offering("  mark-as-read ", " Mark as read ").unwrap();
        assert_eq!(action.named(), "mark-as-read");
        assert_eq!(action.label(), "Mark as read");
    }

    /// **Something with nothing to hand back, or nothing to read, is not
    /// something to offer.** A button with no words on it is a button a person
    /// is being asked to press blind.
    #[test]
    fn something_with_no_name_or_no_words_is_not_something_to_offer() {
        assert_eq!(Action::offering("", "Reply"), Err(NotAnAction::NoName));
        assert_eq!(Action::offering("   ", "Reply"), Err(NotAnAction::NoName));
        assert_eq!(
            Action::offering("re\nply", "Reply"),
            Err(NotAnAction::NoName)
        );
        assert_eq!(Action::offering("reply", ""), Err(NotAnAction::NoLabel));
        assert_eq!(Action::offering("reply", "  "), Err(NotAnAction::NoLabel));
    }

    /// **A label that would rewrite the line it is drawn on is refused**, not
    /// trimmed into something else: an escape sequence or a newline inside a
    /// button is how a notification draws a sentence alo OS never wrote.
    #[test]
    fn a_label_that_would_rewrite_the_line_is_refused() {
        for label in ["Reply\nApprove", "Reply\u{1b}[2K", "\u{7}"] {
            assert_eq!(
                Action::offering("reply", label),
                Err(NotAnAction::NoLabel),
                "{label:?}"
            );
        }
    }

    /// **A label is the sender's own text and is kept as it was written**, in
    /// whatever language and script that was.
    #[test]
    fn a_label_is_kept_in_the_language_its_sender_wrote_it_in() {
        for label in ["Als gelesen markieren", "Σήμανση ως αναγνωσμένο", "已读"]
        {
            let action = Action::offering("mark-as-read", label).unwrap();
            assert_eq!(action.label(), label);
        }
    }
}
