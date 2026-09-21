//! An agent's own notifications are marked as the agent's, with its mark and
//! its word.
//!
//! [ADR 0010](../../../docs/decisions/0010-terracotta-is-reserved-and-never-alone.md)
//! reserved terracotta for the agent and measured why it can never be alone:
//! terracotta on cream is 2.87:1, under the 3.0:1 WCAG 2.1 §1.4.11 asks of a
//! shape that carries meaning and well under the 4.5:1 EN 301 549 asks of text.
//! So the agent arrives with a **mark** and a **word** beside its colour, and a
//! machine whose colours were all one colour still tells a person which
//! notifications are the machine speaking on their behalf.
//!
//! The other half of the rule is that nobody else may wear it. An application
//! that could send a notification drawn in terracotta saying *the agent* would
//! have taken the one signal alo OS reserved — so `alo_notifying::Sender`'s
//! agent constructor reads `alo_capability::Grantee::is_an_application` and
//! refuses, and there is no other road to an agent's notification.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

mod common;

use alo_appearance::Token;
use alo_capability::{Applicant, Grantee};
use alo_notifying::{Missed, NotSent, Quiet, Sender, arriving, deciding};

use common::{a_notification_titled, an_open_seat, the_agent, the_machines_words};

/// **The agent's notification carries its mark, says *the agent* in words, and
/// only then is terracotta** — ADR 0010's order.
#[test]
fn the_agents_notification_carries_a_mark_a_word_and_a_colour() {
    let strings = the_machines_words();
    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    let became = deciding::arrives(
        arriving::from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap(),
        &mut seat,
        Quiet::No,
        &mut missed,
    );
    let shown = became.shown().unwrap();
    assert!(shown.the_agents_mark());
    assert!(shown.sent_by(&strings).text().contains("the agent"));
    assert_eq!(shown.colour(), Token::Terracotta);
}

/// **Nobody else is terracotta and nobody else carries the mark** — not an
/// application, and not alo OS itself, which has its own clause and none of the
/// agent's.
#[test]
fn nobody_else_is_terracotta_and_nobody_else_carries_the_mark() {
    let strings = the_machines_words();
    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    for notification in [
        a_notification_titled("Anna Pärt"),
        arriving::from_alo_os("An update is ready", "", &[]).unwrap(),
    ] {
        let became = deciding::arrives(notification, &mut seat, Quiet::No, &mut missed);
        let shown = became.shown().unwrap();
        assert!(!shown.the_agents_mark());
        assert_eq!(shown.colour(), Token::Navy);
        assert!(!shown.sent_by(&strings).text().contains("the agent"));
    }
}

/// **An application cannot come in at the agent's door**, whatever it is
/// called — including an application whose identifier is the agent's own name.
#[test]
fn an_application_cannot_come_in_at_the_agents_door() {
    for named in ["org.example.Mail", "@alo", "alo"] {
        let application = Applicant::named(named).grantee();
        assert!(application.is_an_application());
        assert_eq!(Sender::the_agent(&application), None, "{named}");
        assert_eq!(
            arriving::from_the_agent(&application, "The invoices are filed", "", &[]),
            Err(NotSent::NotANotification {
                application: named.to_owned(),
            }),
            "{named}"
        );
    }
}

/// **The word survives whatever the agent is called.** A person who cannot
/// tell terracotta from anything else reads *the agent*, and that cannot come
/// from a name somebody chose.
#[test]
fn the_word_survives_whatever_the_agent_is_called() {
    let strings = the_machines_words();
    for named in ["@alo", "@files", "@mail"] {
        let notification =
            arriving::from_the_agent(&Grantee::named(named), "Done", "", &[]).unwrap();
        let said = notification.sent_by(&strings);
        assert!(said.text().contains(named), "{said}");
        assert!(said.text().contains("the agent"), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
    }
}

/// **The agent gets no exemption from being held.** Its notification waits
/// behind do-not-disturb exactly as anybody else's does, because *the machine
/// is acting on your behalf* is not *interrupt the meeting*.
#[test]
fn the_agent_gets_no_exemption_from_being_held() {
    let mut seat = an_open_seat();
    let mut missed = Missed::nothing();
    let became = deciding::arrives(
        arriving::from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap(),
        &mut seat,
        Quiet::Yes(alo_notifying::Because::TheScreenIsShared),
        &mut missed,
    );
    assert!(!became.is_shown());
    assert_eq!(missed.how_many(), 1);
}
