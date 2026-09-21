//! The three doors a notification arrives by, and there is no fourth.
//!
//! **An application's notification arrives under a grant.** Not under a name it
//! wrote about itself, not under a socket it happened to reach: [`from_an_application`]
//! takes the `alo_portals::Allowed` that judging its request against the
//! person's grants produced, and reads who is asking off that. So there is no
//! way to build an application's notification without a grant over
//! `alo_capability::Facility::Notifications` having been current at the moment
//! the request was judged — and a revoked grant is refused at the next request,
//! because `alo_portals::judging` keeps nothing.
//!
//! An `Allowed` for some other portal is **not** a permission to notify, and is
//! refused here rather than being read as one. That is the refusal this file
//! exists for: `alo-portals` answers *this application may use the camera*, and
//! a door that accepted any allowed request would have turned every portal
//! grant on the machine into a grant to put text on the person's screen.
//!
//! **The agent's own notifications** come through [`from_the_agent`], which
//! takes an `alo_capability::Grantee` and refuses one that is an application:
//! ADR 0010's terracotta and the word *agent* are not borrowable. It takes no
//! grant, because an agent notifying the person is not an agent reaching the
//! machine — nothing leaves, nothing changes, and `alo-capability`'s verbs are
//! where an agent's reach is decided.
//!
//! **alo OS's own** come through [`from_alo_os`]: the machine telling the
//! person something about their own machine — an update ready, a disk nearly
//! full. They are drawn like anybody else's and say who they are from like
//! anybody else's.

use alo_capability::Grantee;
use alo_portals::{Allowed, Portal};

use alo_applications::Application;

use crate::notification::Notification;
use crate::refusing::NotSent;
use crate::sender::Sender;

/// A notification from an application whose request the person's grants
/// allowed.
///
/// `allowed` is what `alo_portals::Request::judged` answered, and the
/// application is read off it rather than taken as a separate argument, so the
/// sender and the grant can never be two different applications.
///
/// The application is named by the identifier the grant was made over and never
/// by whatever it calls itself — `alo_applications::Application` argues that at
/// length, and it is the same argument: two programs can call themselves the
/// same thing, and no two share an identifier. A shell that knows the name in
/// the person's own application list shows it beside the identifier.
///
/// # Errors
/// [`NotSent::NotANotification`] when what was allowed was some other portal
/// entirely, or when the identifier the grant names is not one a verb could
/// name; and the refusals in [`crate::Notification`] for a notification that
/// is not one — no title, nothing readable on something it offers, or more
/// things to do than a notification may carry.
pub fn from_an_application(
    allowed: &Allowed,
    title: &str,
    body: &str,
    offering: &[(&str, &str)],
) -> Result<Notification, NotSent> {
    let application = allowed.application().as_str().to_owned();
    if allowed.portal() != Portal::Notifications {
        return Err(NotSent::NotANotification { application });
    }
    let Ok(named) = Application::identified(&application) else {
        return Err(NotSent::NotANotification { application });
    };
    Notification::of(Sender::an_application(named), title, body, offering)
}

/// A notification the agent on this machine sent.
///
/// # Errors
/// [`NotSent::NotANotification`] for a grantee that is an application, which is
/// the one thing this door will not accept: an application arriving here has
/// asked nothing of the notification portal, and letting it through would hand
/// it the agent's colour and the agent's word (ADR 0010). Otherwise the
/// refusals in [`crate::Notification`].
pub fn from_the_agent(
    agent: &Grantee,
    title: &str,
    body: &str,
    offering: &[(&str, &str)],
) -> Result<Notification, NotSent> {
    let Some(sender) = Sender::the_agent(agent) else {
        return Err(NotSent::NotANotification {
            application: agent.as_str().to_owned(),
        });
    };
    Notification::of(sender, title, body, offering)
}

/// A notification alo OS sent about this machine.
///
/// # Errors
/// The refusals in [`crate::Notification`]. alo OS is held to its own rules
/// here: a notification of ours with no title is refused exactly as anybody
/// else's is, because a door that trusted the machine would be a door.
pub fn from_alo_os(
    title: &str,
    body: &str,
    offering: &[(&str, &str)],
) -> Result<Notification, NotSent> {
    Notification::of(Sender::alo_os_itself(), title, body, offering)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{allowed_to, the_agent};

    /// **An application that was allowed the notification portal may notify.**
    #[test]
    fn an_application_allowed_the_notification_portal_may_notify() {
        let allowed = allowed_to(Portal::Notifications);
        let notification = from_an_application(&allowed, "Anna Pärt", "Thursday?", &[]).unwrap();
        assert_eq!(
            notification
                .from()
                .application()
                .map(Application::identifier),
            Some("org.example.Mail")
        );
        assert!(!notification.is_the_agents());
    }

    /// **A permission to do something else is not a permission to notify.**
    /// The refusal this file exists for: every other portal grant on the
    /// machine would otherwise have been a grant to put text on the screen.
    #[test]
    fn a_permission_to_do_something_else_is_not_a_permission_to_notify() {
        for portal in Portal::EVERY {
            if portal == Portal::Notifications {
                continue;
            }
            let allowed = allowed_to(portal);
            assert_eq!(
                from_an_application(&allowed, "Anna Pärt", "Thursday?", &[]),
                Err(NotSent::NotANotification {
                    application: "org.example.Mail".to_owned(),
                }),
                "{portal:?} was read as a permission to notify"
            );
        }
    }

    /// **An application cannot come in at the agent's door**, however it is
    /// named: ADR 0010's colour and word are not borrowable.
    #[test]
    fn an_application_cannot_come_in_at_the_agents_door() {
        let application = alo_capability::Applicant::named("org.example.Mail").grantee();
        assert_eq!(
            from_the_agent(&application, "The invoices are filed", "", &[]),
            Err(NotSent::NotANotification {
                application: "org.example.Mail".to_owned(),
            })
        );

        let ours = from_the_agent(&the_agent(), "The invoices are filed", "", &[]).unwrap();
        assert!(ours.is_the_agents());
    }

    /// **alo OS is held to its own rules.** A notification of ours with
    /// nothing to read on it is refused exactly as anybody else's is.
    #[test]
    fn alo_os_is_held_to_the_same_rules_as_everybody_else() {
        assert!(from_alo_os("An update is ready", "", &[]).is_ok());
        assert_eq!(
            from_alo_os("   ", "", &[]),
            Err(NotSent::NoTitle {
                application: "alo OS".to_owned(),
            })
        );
    }
}
