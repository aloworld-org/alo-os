//! What a person missed, kept until they dismiss it — and kept **here**.
//!
//! The plan asks for *the notifications a person missed kept in a list on the
//! machine until they dismiss them, never synced*. This is that list, and where
//! it lives is the decision this file is about.
//!
//! # It is in this session's memory, and it reaches no disk
//!
//! There is no `serde` on anything in this file, no path, and no door that
//! writes one anywhere. `notifying.toml` holds two settings and cannot hold a
//! notification ([`crate::keeping`]); nothing else in this crate opens
//! anything at all.
//!
//! *Never synced* is a promise that is easy to keep badly — a setting that is
//! off by default, a server nobody configured — and the way it is kept here is
//! that **there is nothing to sync**. A message's first line, the name of whoever
//! sent it and the subject of a calendar entry are the most private text on a
//! machine and the least useful to keep: a person reads what they missed and
//! dismisses it. So the list ends when the session ends, and a machine that is
//! signed out of holds nothing about who was trying to reach the person who
//! owns it.
//!
//! That is a deliberate narrowing of *until they dismiss them*: dismissing is
//! how the list empties while a person is signed in, and signing out empties it
//! too. The report for this task says so in as many words rather than leaving
//! somebody to discover it.
//!
//! # What goes on it
//!
//! Everything that was **held** rather than shown: what do-not-disturb held
//! ([`crate::deciding::arrives`]), and everything the lock screen held, handed
//! over by task 1 at the unlock ([`Missed::at_the_unlock`]). A notification
//! that was shown is not on it, because a person did not miss it.

use alo_strings::{Filling, Said, Strings};

use crate::notification::Notification;
use crate::words;

/// One waiting notification's handle, which is what a person dismisses by.
///
/// Assigned here and never by a sender: two applications that both called
/// their notification `1` would otherwise be able to dismiss each other's.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NotificationId(u64);

impl NotificationId {
    /// The number, for whoever has to write one into a surface's own bookkeeping.
    #[must_use]
    pub const fn as_number(self) -> u64 {
        self.0
    }
}

/// One notification waiting for the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Waiting {
    /// What it is dismissed by.
    id: NotificationId,
    /// The notification itself.
    notification: Notification,
}

impl Waiting {
    /// What it is dismissed by.
    #[must_use]
    pub const fn id(&self) -> NotificationId {
        self.id
    }

    /// The notification itself.
    #[must_use]
    pub const fn notification(&self) -> &Notification {
        &self.notification
    }
}

/// Everything the person missed, oldest first.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Missed {
    /// What is waiting, in the order it arrived.
    waiting: Vec<Waiting>,
    /// The next handle to give out. Never reused inside one session, so a
    /// dismissal cannot land on a notification that took a dismissed one's
    /// place.
    next: u64,
}

impl Missed {
    /// Nothing missed yet.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// Keep this one for the person, and answer with what it is dismissed by.
    pub fn keeps(&mut self, notification: Notification) -> NotificationId {
        let id = NotificationId(self.next);
        self.next = self.next.saturating_add(1);
        self.waiting.push(Waiting { id, notification });
        id
    }

    /// Everything the lock screen held, handed over at the unlock.
    ///
    /// `held` is `alo_locking::Unlocking::Unlocked`'s own list, oldest first,
    /// which task 1 hands to the person who has just proved who they are. It
    /// goes on this list in that order and nothing here reorders it.
    pub fn at_the_unlock(&mut self, held: Vec<Notification>) -> Vec<NotificationId> {
        held.into_iter()
            .map(|notification| self.keeps(notification))
            .collect()
    }

    /// Everything waiting, oldest first.
    #[must_use]
    pub fn waiting(&self) -> &[Waiting] {
        &self.waiting
    }

    /// How many are waiting.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.waiting.len()
    }

    /// Whether nothing is waiting.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.waiting.is_empty()
    }

    /// The person dismissed this one. Says whether there was one to dismiss.
    pub fn dismiss(&mut self, id: NotificationId) -> bool {
        let before = self.waiting.len();
        self.waiting.retain(|waiting| waiting.id != id);
        self.waiting.len() != before
    }

    /// The person dismissed everything. Says how many there were.
    pub fn dismiss_everything(&mut self) -> usize {
        let how_many = self.waiting.len();
        self.waiting.clear();
        how_many
    }

    /// What the list reads as, in the language the person reads.
    ///
    /// One line per waiting notification saying who it is from, or the single
    /// sentence for an empty list. **Never empty**: a list that said nothing
    /// would be indistinguishable from one that was not there.
    #[must_use]
    pub fn reads_as(&self, strings: &Strings) -> Vec<Said> {
        if self.waiting.is_empty() {
            return vec![strings.say(&words::NOTHING_WAITING.key(), &Filling::nothing())];
        }
        self.waiting
            .iter()
            .map(|waiting| waiting.notification.sent_by(strings))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{a_notification_titled, in_english};

    /// **What was missed waits in the order it arrived, and is dismissed one
    /// at a time.**
    #[test]
    fn what_was_missed_waits_until_it_is_dismissed() {
        let mut missed = Missed::nothing();
        assert!(missed.is_empty());

        let first = missed.keeps(a_notification_titled("Anna Pärt"));
        let second = missed.keeps(a_notification_titled("Bertrand"));
        assert_eq!(missed.how_many(), 2);
        assert_eq!(
            missed.waiting().first().map(Waiting::id),
            Some(first),
            "the oldest is first"
        );

        assert!(missed.dismiss(first));
        assert!(!missed.dismiss(first), "dismissing twice dismisses once");
        assert_eq!(missed.how_many(), 1);
        assert_eq!(missed.waiting().first().map(Waiting::id), Some(second));

        assert_eq!(missed.dismiss_everything(), 1);
        assert!(missed.is_empty());
    }

    /// **A handle is never reused inside a session**, so a dismissal cannot
    /// land on whatever took a dismissed notification's place.
    #[test]
    fn a_handle_is_never_reused_inside_a_session() {
        let mut missed = Missed::nothing();
        let first = missed.keeps(a_notification_titled("Anna Pärt"));
        assert!(missed.dismiss(first));
        let second = missed.keeps(a_notification_titled("Bertrand"));
        assert_ne!(first, second);
        assert!(!missed.dismiss(first));
        assert_eq!(missed.how_many(), 1);
    }

    /// **Everything the lock screen held goes on in the order it arrived.**
    /// Task 1 hands the list over at the unlock; nothing here reorders it and
    /// nothing here drops one.
    #[test]
    fn everything_the_lock_screen_held_goes_on_in_order() {
        let mut missed = Missed::nothing();
        let ids = missed.at_the_unlock(vec![
            a_notification_titled("First"),
            a_notification_titled("Second"),
            a_notification_titled("Third"),
        ]);
        assert_eq!(ids.len(), 3);
        let titles: Vec<&str> = missed
            .waiting()
            .iter()
            .map(|waiting| waiting.notification().title())
            .collect();
        assert_eq!(titles, ["First", "Second", "Third"]);
    }

    /// **An empty list says so rather than saying nothing.**
    #[test]
    fn an_empty_list_says_so_rather_than_saying_nothing() {
        let strings = in_english();
        let empty = Missed::nothing().reads_as(&strings);
        assert_eq!(empty.len(), 1);
        assert_eq!(empty.first().map(Said::text), Some("Nothing is waiting"));

        let mut missed = Missed::nothing();
        missed.keeps(a_notification_titled("Anna Pärt"));
        let read = missed.reads_as(&strings);
        assert_eq!(read.len(), 1);
        // By the identifier the grant was made over, and not by a name: what
        // arrives through the portal is an `alo_portals::Allowed`, which
        // carries the identifier and nothing an application wrote about
        // itself. A shell that knows the person's own application list draws
        // the name beside it; this crate never invents one.
        assert_eq!(
            read.first().map(Said::text),
            Some("Sent by org.example.Mail")
        );
    }
}
