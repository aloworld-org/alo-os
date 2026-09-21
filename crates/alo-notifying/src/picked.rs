//! The person picked one of the things a notification offered, and it goes
//! back to whoever sent it.
//!
//! **There is one destination and it is the sender.** [`Picked`] holds who sent
//! the notification and the sender's own name for what was picked, and it holds
//! nothing else — no proposal, no verb, no grant, no approval and no road to
//! any of them. Whatever the shell does with one, it hands it to the program
//! that sent the notification and to nothing else on the machine.
//!
//! That is ADR 0001 kept where it is easiest to lose. A change to the machine
//! is proposed with a sentence and waits for one approval, and *what a person
//! approves is that sentence* — read on the approval surface, which they went
//! to in order to approve something. A notification is not that surface: it
//! arrives uninvited, beside whatever they were doing, and a person tapping a
//! button on one has not read a proposal. So there is no shape of [`Picked`]
//! that could answer one, which is why this file has two fields and no
//! constructor that takes anything from `alo-approving` — a crate this one does
//! not depend on at all.
//!
//! # And it cannot be something the notification never offered
//!
//! [`Picked::of`] takes the notification and refuses an action that is not one
//! of the ones it carries. A shell that had a name from somewhere else — an
//! older notification, a message from a program, a line it read off a socket —
//! cannot turn it into something the sender will be told the person chose.

use crate::action::Action;
use crate::notification::Notification;
use crate::sender::Sender;

/// One of the things a notification offered, picked by the person.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picked {
    /// Who sent the notification, and the only place this goes.
    to: Sender,
    /// The sender's own name for what was picked.
    named: String,
}

impl Picked {
    /// The person picked this action on this notification.
    ///
    /// Answers [`None`] for an action the notification does not offer, which is
    /// the only way to be handed one this crate never accepted.
    #[must_use]
    pub fn of(notification: &Notification, action: &Action) -> Option<Self> {
        if !notification.offering().contains(action) {
            return None;
        }
        Some(Self {
            to: notification.from().clone(),
            named: action.named().to_owned(),
        })
    }

    /// Who this goes to, which is whoever sent the notification.
    #[must_use]
    pub const fn to(&self) -> &Sender {
        &self.to
    }

    /// The sender's own name for what was picked, which is what the sender is
    /// handed and what nobody reads.
    #[must_use]
    pub fn named(&self) -> &str {
        &self.named
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::arriving::from_an_application;
    use crate::testing::allowed_to;
    use alo_applications::Application;
    use alo_portals::Portal;

    /// A notification offering two things, for the tests below.
    fn a_notification() -> Notification {
        from_an_application(
            &allowed_to(Portal::Notifications),
            "Anna Pärt",
            "Thursday?",
            &[("reply", "Reply"), ("mark-as-read", "Mark as read")],
        )
        .unwrap()
    }

    /// **What was picked goes to whoever sent it, by the name they know it
    /// by.**
    #[test]
    fn what_was_picked_goes_back_to_whoever_sent_it() {
        let notification = a_notification();
        let action = notification.offering().first().unwrap();
        let picked = Picked::of(&notification, action).unwrap();
        assert_eq!(picked.named(), "reply");
        assert_eq!(
            picked.to().application().map(Application::identifier),
            Some("org.example.Mail")
        );
    }

    /// **Something the notification never offered cannot be picked**, so a
    /// name from anywhere else cannot become something the sender is told the
    /// person chose.
    #[test]
    fn something_the_notification_never_offered_cannot_be_picked() {
        let notification = a_notification();
        let elsewhere = Action::offering("approve", "Approve").unwrap();
        assert_eq!(Picked::of(&notification, &elsewhere), None);

        let a_real_name_with_other_words =
            Action::offering("reply", "Approve: delete everything").unwrap();
        assert_eq!(
            Picked::of(&notification, &a_real_name_with_other_words),
            None
        );
    }
}
