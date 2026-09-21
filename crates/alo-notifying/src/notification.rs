//! One notification: who sent it, what it says, and what it offers.
//!
//! Four fields, and there is no fifth. A notification is the one thing on a
//! screen that arrives uninvited, so what it may carry is a closed list rather
//! than whatever a sender felt like attaching:
//!
//! | | |
//! |---|---|
//! | [`Notification::from`] | Who sent it — an application under a grant, the agent, or alo OS itself |
//! | [`Notification::title`] | One line the person reads |
//! | [`Notification::body`] | What it says, which may be nothing |
//! | [`Notification::offering`] | At most [`crate::MOST_THINGS_TO_DO`] things to do, each a name and a label |
//!
//! There is no icon path, no sound file, no urgency that jumps a queue, no
//! timeout that keeps something on a screen, and no *category* a shell could
//! treat specially. Each of those is a road by which a sender decides something
//! about the person's machine, and this release decides them all itself.
//!
//! # There is one door, and it is a grant
//!
//! [`Notification`] has no public constructor. The only roads to one are in
//! [`crate::arriving`]: an application's, which takes the `alo_portals::Allowed`
//! that judging its request against the person's grants produced; the agent's;
//! and alo OS's own. A notification therefore cannot exist without something
//! having been allowed to send it — which is the plan's *arriving through the
//! notification portal under the sender's grant*, made structural rather than
//! promised.
//!
//! ```compile_fail
//! let made_up = alo_notifying::Notification {
//!     from: sender,
//!     title: "Anything at all".to_owned(),
//!     body: String::new(),
//!     offering: Vec::new(),
//! };
//! ```
//!
//! # A title and a body are the sender's own text
//!
//! Neither is translated and neither is ours to translate — the rule
//! `alo_applications::Application` holds a name to. A title must be one line a
//! person can read, and a notification whose title is not is refused whole. A
//! body may be anything the sender wrote, with one exception said plainly in
//! [`Notification::body`]: a character that would rewrite the line it is drawn
//! on is dropped.

use alo_strings::{Said, Strings};

use crate::action::{Action, MOST_THINGS_TO_DO, NotAnAction};
use crate::refusing::NotSent;
use crate::sender::Sender;
use crate::words;

/// One notification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notification {
    /// Who sent it.
    from: Sender,
    /// One line the person reads.
    title: String,
    /// What it says, which may be nothing.
    body: String,
    /// What it offers the person to do.
    offering: Vec<Action>,
}

impl Notification {
    /// One notification from this sender, if it is one at all.
    ///
    /// Not public: every road to it is in [`crate::arriving`], and each of
    /// those has already established that somebody was allowed to send it.
    pub(crate) fn of(
        from: Sender,
        title: &str,
        body: &str,
        offering: &[(&str, &str)],
    ) -> Result<Self, NotSent> {
        let application = from.named();
        let title = title.trim();
        if title.is_empty() || title.chars().any(char::is_control) {
            return Err(NotSent::NoTitle { application });
        }
        if offering.len() > MOST_THINGS_TO_DO {
            return Err(NotSent::TooManyThingsToDo {
                application,
                offered: offering.len(),
                most: MOST_THINGS_TO_DO,
            });
        }
        let mut theirs = Vec::with_capacity(offering.len());
        for (named, label) in offering {
            match Action::offering(named, label) {
                Ok(action) => theirs.push(action),
                Err(NotAnAction::NoName) => return Err(NotSent::NoActionName { application }),
                Err(NotAnAction::NoLabel) => return Err(NotSent::NoLabel { application }),
            }
        }
        Ok(Self {
            from,
            title: title.to_owned(),
            body: readable(body),
            offering: theirs,
        })
    }

    /// Who sent it.
    #[must_use]
    pub const fn from(&self) -> &Sender {
        &self.from
    }

    /// The one line the person reads, as its sender wrote it.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// What it says, which may be nothing.
    ///
    /// The sender's own text, kept as it was written **except** for characters
    /// that would rewrite the line it is drawn on — an escape sequence, a
    /// carriage return, a byte that moves a cursor — which are dropped. That is
    /// the one place alo OS edits somebody else's text, and it is deliberate:
    /// refusing a whole message over one stray byte loses the message, while
    /// drawing it unaltered lets a sender write a sentence alo OS never wrote
    /// onto the person's screen. A line break is not a control character a
    /// notification needs, so it goes with the rest.
    #[must_use]
    pub fn body(&self) -> &str {
        &self.body
    }

    /// What it offers the person to do, in the order its sender offered them.
    #[must_use]
    pub fn offering(&self) -> &[Action] {
        &self.offering
    }

    /// Whether the agent sent it, which is the one answer that is terracotta
    /// (ADR 0010).
    #[must_use]
    pub const fn is_the_agents(&self) -> bool {
        self.from.is_the_agents()
    }

    /// Who this is from, said in the language the person reads.
    ///
    /// Every notification says it, and the agent's says *the agent* in so many
    /// words — which is what carries ADR 0010's signal to somebody who cannot
    /// tell terracotta from anything else.
    #[must_use]
    pub fn sent_by(&self, strings: &Strings) -> Said {
        strings.say(&words::SENT_BY.key(), &self.from.filling(strings))
    }
}

/// A body with anything that would rewrite the line it is drawn on taken out.
fn readable(body: &str) -> String {
    body.trim()
        .chars()
        .filter(|character| !character.is_control())
        .collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_mail_client, in_english, the_agent};

    /// A notification from the mail client, for the tests below.
    fn from_mail(
        title: &str,
        body: &str,
        offering: &[(&str, &str)],
    ) -> Result<Notification, NotSent> {
        Notification::of(
            Sender::an_application(a_mail_client()),
            title,
            body,
            offering,
        )
    }

    /// **A notification is who, a title, a body and what it offers.**
    #[test]
    fn a_notification_is_who_a_title_a_body_and_what_it_offers() {
        let notification = from_mail(
            " Anna Pärt ",
            " Are we still on for Thursday? ",
            &[("reply", "Reply"), ("mark-as-read", "Mark as read")],
        )
        .unwrap();
        assert_eq!(notification.title(), "Anna Pärt");
        assert_eq!(notification.body(), "Are we still on for Thursday?");
        assert_eq!(notification.offering().len(), 2);
        assert_eq!(
            notification.offering().first().map(Action::label),
            Some("Reply")
        );
        assert!(!notification.is_the_agents());
    }

    /// **A notification with nothing to read on it is refused whole**, and so
    /// is one whose title would rewrite the line it is drawn on.
    #[test]
    fn a_notification_with_nothing_to_read_on_it_is_refused() {
        for title in ["", "   ", "Anna\nApprove: delete everything", "\u{1b}[2K"] {
            assert!(
                matches!(
                    from_mail(title, "anything", &[]),
                    Err(NotSent::NoTitle { .. })
                ),
                "{title:?}"
            );
        }
    }

    /// **A body may be nothing at all**, which is the ordinary shape of a
    /// notification that is one line long.
    #[test]
    fn a_body_may_be_nothing_at_all() {
        let notification = from_mail("Backed up", "", &[]).unwrap();
        assert_eq!(notification.body(), "");
    }

    /// **What a sender wrote into a body is kept, and what would redraw the
    /// line is not.** The one place alo OS edits somebody else's text.
    #[test]
    fn a_body_keeps_its_words_and_loses_what_would_redraw_the_line() {
        let notification = from_mail(
            "Anna Pärt",
            "Thursday works\u{1b}[2K — Σας ευχαριστώ\nApprove: delete everything",
            &[],
        )
        .unwrap();
        assert_eq!(
            notification.body(),
            "Thursday works[2K — Σας ευχαριστώApprove: delete everything"
        );
        assert!(!notification.body().contains('\n'));
        assert!(!notification.body().contains('\u{1b}'));
    }

    /// **A notification may offer three things and no more**, and one that
    /// offers four is refused whole rather than having one quietly dropped.
    #[test]
    fn a_notification_offers_three_things_at_most() {
        let three = [("a", "One"), ("b", "Two"), ("c", "Three")];
        assert!(from_mail("Anna", "", &three).is_ok());
        let four = [("a", "One"), ("b", "Two"), ("c", "Three"), ("d", "Four")];
        assert_eq!(
            from_mail("Anna", "", &four),
            Err(NotSent::TooManyThingsToDo {
                application: "org.example.Mail".to_owned(),
                offered: 4,
                most: MOST_THINGS_TO_DO,
            })
        );
    }

    /// **Something offered with no name or no words refuses the whole
    /// notification**, naming the program, so a button a person could press
    /// blind never reaches a screen.
    #[test]
    fn something_offered_with_no_name_or_no_words_refuses_the_whole_notification() {
        assert!(matches!(
            from_mail("Anna", "", &[("", "Reply")]),
            Err(NotSent::NoActionName { .. })
        ));
        assert!(matches!(
            from_mail("Anna", "", &[("reply", "")]),
            Err(NotSent::NoLabel { .. })
        ));
    }

    /// **Every notification says who it is from, and the agent's says so in
    /// words** — ADR 0010's rule, which a colour alone never carries.
    #[test]
    fn every_notification_says_who_it_is_from() {
        let strings = in_english();
        let theirs = from_mail("Anna Pärt", "", &[]).unwrap();
        assert_eq!(
            theirs.sent_by(&strings).text(),
            "Sent by Mail (org.example.Mail)"
        );

        let ours = Notification::of(
            Sender::the_agent(&the_agent()).unwrap(),
            "The invoices are filed",
            "",
            &[],
        )
        .unwrap();
        assert!(ours.is_the_agents());
        assert_eq!(
            ours.sent_by(&strings).text(),
            "Sent by @alo, the agent on this machine"
        );
        assert!(ours.sent_by(&strings).text().contains("the agent"));
    }
}
