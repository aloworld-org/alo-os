//! What a shell is handed for a notification that is being shown.
//!
//! Nothing here draws. A [`Shown`] says what a notification is, who it is from
//! in the person's language, whether the agent's mark goes beside it and what
//! colour it is — and it says those in that order, which is
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)'s
//! order. Where a notification appears, how long it stays and what it looks
//! like are the shell's.
//!
//! # Mark, word — and only then colour
//!
//! ADR 0010 measured why: terracotta on cream is 2.87:1, under the 3.0:1 WCAG
//! 2.1 §1.4.11 asks of a shape that carries meaning. So a notification from the
//! agent carries **the agent's mark** ([`Shown::the_agents_mark`]) and **says
//! *the agent* in words** ([`Shown::sent_by`]) before its colour is looked at,
//! and a machine whose colours were all one colour would still tell a person
//! which notifications were the agent's.
//!
//! [`Shown::colour`] is terracotta exactly when the agent sent it, and the
//! colour alo OS writes everything else in for everything else. There is no
//! third case and no setting: an application is not the machine acting on the
//! person's behalf, and drawing one as though it were would spend the one
//! signal ADR 0010 reserved.
//!
//! # A shown notification cannot be built out of nothing
//!
//! [`Shown`] is made from a [`Notification`] by [`crate::deciding::arrives`]
//! and by nothing else — so what a shell draws is something that arrived under
//! a grant and was not held, and there is no second door.
//!
//! ```compile_fail
//! let made_up = alo_notifying::Shown::in_colour(alo_appearance::Token::Terracotta);
//! ```

use alo_appearance::Token;
use alo_strings::{Said, Strings};

use crate::notification::Notification;

/// A notification that is being shown, as a shell is handed it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Shown {
    /// The notification itself.
    notification: Notification,
}

impl Shown {
    /// This notification, shown.
    ///
    /// Not public: the only road to one is [`crate::deciding::arrives`],
    /// which has already asked whether the seat is locked and whether anything
    /// is holding notifications.
    pub(crate) const fn of(notification: Notification) -> Self {
        Self { notification }
    }

    /// The notification.
    #[must_use]
    pub const fn notification(&self) -> &Notification {
        &self.notification
    }

    /// The notification, taken out — for a shell that keeps it while it is on
    /// the screen.
    #[must_use]
    pub fn into_notification(self) -> Notification {
        self.notification
    }

    /// Whether the agent's own mark — ADR 0010's small dot — is drawn beside
    /// it.
    ///
    /// True exactly when it is terracotta, because ADR 0010 says the two
    /// arrive together or not at all.
    #[must_use]
    pub const fn the_agents_mark(&self) -> bool {
        self.notification.is_the_agents()
    }

    /// Who it is from, said in the language the person reads.
    ///
    /// The agent's says *the agent* in so many words, which is what carries
    /// the signal to somebody who cannot tell terracotta from anything else.
    #[must_use]
    pub fn sent_by(&self, strings: &Strings) -> Said {
        self.notification.sent_by(strings)
    }

    /// The colour it is drawn in: terracotta for the agent, and the colour
    /// everything else on the machine is written in for everything else.
    ///
    /// Reserved (ADR 0010), and carrying nothing the mark and the sentence do
    /// not already carry.
    #[must_use]
    pub const fn colour(&self) -> Token {
        if self.notification.is_the_agents() {
            Token::Terracotta
        } else {
            Token::Navy
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::arriving::{from_alo_os, from_the_agent};
    use crate::testing::{a_notification_titled, in_english, the_agent};

    /// **The agent's notification carries a mark, the word and the colour, in
    /// that order** (ADR 0010).
    #[test]
    fn the_agents_notification_carries_a_mark_a_word_and_a_colour() {
        let strings = in_english();
        let shown =
            Shown::of(from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap());
        assert!(shown.the_agents_mark());
        assert!(shown.sent_by(&strings).text().contains("the agent"));
        assert_eq!(shown.colour(), Token::Terracotta);
    }

    /// **Nobody else is terracotta**, and nobody else carries the agent's
    /// mark — not an application, and not alo OS itself.
    #[test]
    fn nobody_else_is_terracotta_and_nobody_else_carries_the_mark() {
        let strings = in_english();
        for notification in [
            a_notification_titled("Anna Pärt"),
            from_alo_os("An update is ready", "", &[]).unwrap(),
        ] {
            let shown = Shown::of(notification);
            assert!(!shown.the_agents_mark());
            assert_eq!(shown.colour(), Token::Navy);
            assert!(!shown.sent_by(&strings).text().contains("the agent"));
        }
    }

    /// **The mark and the colour agree, always.** ADR 0010 says they arrive
    /// together or not at all, so a change that made one of them conditional
    /// fails here.
    #[test]
    fn the_mark_and_the_colour_always_agree() {
        for notification in [
            a_notification_titled("Anna Pärt"),
            from_alo_os("An update is ready", "", &[]).unwrap(),
            from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap(),
        ] {
            let shown = Shown::of(notification);
            assert_eq!(shown.the_agents_mark(), shown.colour() == Token::Terracotta);
        }
    }
}
