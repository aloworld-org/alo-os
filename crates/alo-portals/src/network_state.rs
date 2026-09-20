//! The network's state, answered from what the network manager reported.
//!
//! `org.freedesktop.portal.NetworkMonitor` is how a sandboxed application asks
//! whether it is worth trying to send anything. This module is what it is
//! answered with, and who may be answered — without the bus, so both are
//! decided and tested on any machine. `crate::network_monitor_portal` speaks
//! it.
//!
//! # Two services number the same four states differently
//!
//! This is the whole reason this module is not three one-line casts.
//!
//! The portal specification's `connectivity` is **GLib's**
//! `GNetworkConnectivity`. The reading comes from **NetworkManager's**
//! `NM_CONNECTIVITY`. The two agree on *full*, and disagree in the middle:
//!
//! | The state | NetworkManager | GLib, which the portal answers |
//! |---|---|---|
//! | nothing worked out | `UNKNOWN` = 0 | *has no value for it* |
//! | reaching nothing past this machine | `NONE` = 1 | `LOCAL` = 1 |
//! | reaching this network and no further | `LIMITED` = **3** | `LIMITED` = **2** |
//! | a sign-in page in the way | `PORTAL` = **2** | `PORTAL` = **3** |
//! | reaching what was asked for | `FULL` = 4 | `FULL` = 4 |
//!
//! **2 and 3 are swapped between them.** A backend that passed the number
//! through would tell every application on the machine that a café's sign-in
//! page is a working connection, and that a working-but-local network is a
//! captive portal. Neither number is written here: `alo_networks::HowFar` is
//! words, this maps words to GLib's numbers, and
//! `tests::the_two_services_number_these_states_differently` is the mapping
//! held to both tables at once.
//!
//! # What GLib has no value for
//!
//! GLib's enum has no *unknown*. A network manager that has not worked out how
//! far the machine reaches — still starting up, or its connectivity check
//! switched off — is answered [`LOCAL`], the least it could be, and
//! `GetAvailable` is `false`. Answering `FULL` on no evidence would be this
//! machine telling an application it is online because nobody has said
//! otherwise.
//!
//! # Who is answered
//!
//! [`allowed`] judges the caller: a sandboxed application holding a grant of
//! the `network state` facility at this moment. The network is read only after
//! that, by [`read`], so a refused application is never answered from it.

use std::time::SystemTime;

use alo_networks::{HowFar, Metered, Reaching};

use crate::answered::{Outcome, Unanswered};
use crate::judging::Allowed;
use crate::portal::Portal;
use crate::request::Request;
use crate::the_machine::TheMachine;

/// `G_NETWORK_CONNECTIVITY_LOCAL`: the host is not configured to reach the
/// internet, and it is what this machine answers when nothing has been worked
/// out.
pub const LOCAL: u32 = 1;

/// `G_NETWORK_CONNECTIVITY_LIMITED`: the host is configured to reach the
/// internet but is not reaching it.
pub const LIMITED: u32 = 2;

/// `G_NETWORK_CONNECTIVITY_PORTAL`: something is standing in the way and
/// wants to be agreed with first.
pub const PORTAL: u32 = 3;

/// `G_NETWORK_CONNECTIVITY_FULL`: the host is reaching the internet.
pub const FULL: u32 = 4;

/// What the portal answers for `connectivity`, in GLib's numbering.
///
/// Never NetworkManager's number: see this module's table.
#[must_use]
pub const fn connectivity(how_far: HowFar) -> u32 {
    match how_far {
        HowFar::NotSaid | HowFar::Nowhere => LOCAL,
        HowFar::OnlyThisNetwork => LIMITED,
        HowFar::APageInTheWay => PORTAL,
        HowFar::AllOfIt => FULL,
    }
}

/// What the portal answers for `available`.
///
/// `alo_networks::HowFar::reaches_anything`, and nothing decided again here:
/// what *connected* means is that crate's one answer, so the portal and the
/// status area cannot come to differ about it.
#[must_use]
pub const fn available(how_far: HowFar) -> bool {
    how_far.reaches_anything()
}

/// What the portal answers for `metered`.
///
/// `alo_networks::Metered::should_hold_off`, which counts a guess that the
/// connection is metered, and nothing said at all, as metered. The
/// specification's own words for this key are *whether the connection is
/// metered*, and an application reads it to decide whether to spend somebody's
/// data. Answering `false` where the machine does not know would spend it.
#[must_use]
pub const fn metered(metered: Metered) -> bool {
    metered.should_hold_off()
}

/// Whether `application` may read the network's state at `at`, judged against
/// the grants as they are now.
///
/// # Errors
/// The [`Outcome`] the record keeps for a caller that may not, boxed as the
/// other portals' are: [`Unanswered::NotIdentified`] for a program with no
/// sandbox, [`Outcome::NotARequest`] for a sandbox naming something that is not
/// an identifier, [`Unanswered::GrantsUnread`], and [`Outcome::Refused`] with
/// the grants' own refusal.
pub fn allowed(
    application: Option<&str>,
    machine: &dyn TheMachine,
    at: SystemTime,
) -> Result<Allowed, Box<Outcome>> {
    let unanswered = |why| Box::new(Outcome::Unanswered(why));
    let application = application.ok_or_else(|| unanswered(Unanswered::NotIdentified))?;
    let request = Request::of(application, Portal::NetworkMonitor)
        .map_err(|not| Box::new(Outcome::NotARequest(not)))?;
    let grants = machine
        .grants()
        .ok_or_else(|| unanswered(Unanswered::GrantsUnread))?;
    request
        .judged(&grants, at)
        .map_err(|refused| Box::new(Outcome::Refused(refused)))
}

/// What the network manager reports at this moment.
///
/// # Errors
/// [`Unanswered::NetworkUnread`], boxed as [`allowed`]'s refusals are, when the
/// network manager cannot be asked — never a reachable reading in its place. A
/// machine whose network manager is not answering is not a machine that is
/// online, and an application told it was would spend a person's battery
/// retrying something that cannot work.
pub fn read(machine: &dyn TheMachine) -> Result<Reaching, Box<Outcome>> {
    machine
        .reaching()
        .ok_or_else(|| Box::new(Outcome::Unanswered(Unanswered::NetworkUnread)))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::refused::Refused;
    use crate::the_machine::{Applications, TimeOfDay};
    use alo_appearance::Appearance;
    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::time::Duration;

    const NEWSFLASH: &str = "io.gitlab.news_flash.NewsFlash";

    /// A machine whose parts a test decides, and which says whether the
    /// network was read.
    #[derive(Default)]
    struct AMachine {
        grants: Option<Grants>,
        reaching: Option<Reaching>,
        network_read: AtomicBool,
    }

    impl TheMachine for AMachine {
        fn grants(&self) -> Option<Grants> {
            self.grants.clone()
        }
        fn applications(&self) -> Option<Applications> {
            None
        }
        fn appearance(&self) -> Option<Appearance> {
            None
        }
        fn time_of_day(&self) -> Option<TimeOfDay> {
            None
        }
        fn reaching(&self) -> Option<Reaching> {
            self.network_read.store(true, Ordering::SeqCst);
            self.reaching
        }
    }

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn granted(reach: Reach) -> Grants {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked_for(
                &Applicant::named(NEWSFLASH).grantee(),
                reach,
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        grants
    }

    /// **The two services number these states differently, and the portal is
    /// answered in GLib's numbering** — both tables written out, so a reader
    /// can check the swap rather than trust it.
    ///
    /// NetworkManager: none 1, portal 2, limited 3, full 4.
    /// GLib, which the portal answers: local 1, limited 2, portal 3, full 4.
    #[test]
    fn the_two_services_number_these_states_differently() {
        // Read as the network manager numbers them...
        let nothing = HowFar::reported(0);
        let none = HowFar::reported(1);
        let nm_portal = HowFar::reported(2);
        let nm_limited = HowFar::reported(3);
        let full = HowFar::reported(4);

        // ...and answered as GLib numbers them.
        assert_eq!(connectivity(nothing), LOCAL);
        assert_eq!(connectivity(none), LOCAL);
        assert_eq!(connectivity(nm_portal), PORTAL, "2 in, 3 out");
        assert_eq!(connectivity(nm_limited), LIMITED, "3 in, 2 out");
        assert_eq!(connectivity(full), FULL);

        // The swap, stated as the bug it prevents: the number in is not the
        // number out for exactly these two.
        assert_ne!(connectivity(nm_portal), 2);
        assert_ne!(connectivity(nm_limited), 3);
    }

    /// **Every state this machine can read is answered as one of GLib's four**,
    /// and never as a number outside them.
    #[test]
    fn every_state_is_one_of_the_four_the_specification_has() {
        for how_far in HowFar::EVERY {
            let answered = connectivity(how_far);
            assert!(
                (LOCAL..=FULL).contains(&answered),
                "{how_far:?} answered {answered}"
            );
        }
    }

    /// **Nothing worked out is answered as the least it could be**, because
    /// GLib has no *unknown* and answering *full* on no evidence would be this
    /// machine telling an application it is online because nobody said it was
    /// not.
    #[test]
    fn nothing_worked_out_is_answered_as_the_least_it_could_be() {
        assert_eq!(connectivity(HowFar::NotSaid), LOCAL);
        assert!(!available(HowFar::NotSaid));
    }

    /// **A sign-in page in the way is available and is not full**, which is the
    /// pair an application needs to know to show one.
    #[test]
    fn a_sign_in_page_is_reachable_and_is_not_a_working_connection() {
        assert!(available(HowFar::APageInTheWay));
        assert_eq!(connectivity(HowFar::APageInTheWay), PORTAL);
        assert_ne!(connectivity(HowFar::APageInTheWay), FULL);
    }

    /// **What might be metered is answered metered**, because an application
    /// reads this key before spending somebody's data.
    #[test]
    fn what_might_be_metered_is_answered_metered() {
        assert!(metered(Metered::Metered));
        assert!(metered(Metered::ProbablyMetered));
        assert!(metered(Metered::NotSaid));
        assert!(!metered(Metered::Unmetered));
        assert!(!metered(Metered::ProbablyUnmetered));
    }

    /// **Only a sandboxed application holding the facility is allowed**, and
    /// none of the refusals reads the network.
    #[test]
    fn only_an_application_granted_the_network_state_is_allowed() {
        let machine = AMachine {
            grants: Some(granted(Reach::Facility(Facility::NetworkState))),
            reaching: Some(Reaching::reported(HowFar::AllOfIt, Metered::Unmetered)),
            ..AMachine::default()
        };
        let allowed = allowed(Some(NEWSFLASH), &machine, noon()).unwrap();
        assert_eq!(allowed.portal(), Portal::NetworkMonitor);

        assert_eq!(
            *super::allowed(None, &machine, noon()).unwrap_err(),
            Outcome::Unanswered(Unanswered::NotIdentified)
        );
        assert!(matches!(
            *super::allowed(Some("not an identifier"), &machine, noon()).unwrap_err(),
            Outcome::NotARequest(_)
        ));
        assert_eq!(
            *super::allowed(Some("org.example.Stranger"), &machine, noon()).unwrap_err(),
            Outcome::Refused(Refused::NothingGranted {
                application: Applicant::named("org.example.Stranger"),
                portal: Portal::NetworkMonitor,
            })
        );
        assert!(matches!(
            *super::allowed(Some(NEWSFLASH), &machine, noon() + Duration::from_secs(61))
                .unwrap_err(),
            Outcome::Refused(_)
        ));

        let camera_only = AMachine {
            grants: Some(granted(Reach::Facility(Facility::Camera))),
            ..AMachine::default()
        };
        assert!(matches!(
            *super::allowed(Some(NEWSFLASH), &camera_only, noon()).unwrap_err(),
            Outcome::Refused(Refused::NotAllowed { .. })
        ));
        let no_grants = AMachine::default();
        assert_eq!(
            *super::allowed(Some(NEWSFLASH), &no_grants, noon()).unwrap_err(),
            Outcome::Unanswered(Unanswered::GrantsUnread)
        );
        for machine in [&machine, &camera_only, &no_grants] {
            assert!(
                !machine.network_read.load(Ordering::SeqCst),
                "judging read the network"
            );
        }
    }

    /// **A network manager that cannot be asked is a refusal, never a
    /// reachable reading** — an application told it is online would retry
    /// something that cannot work, on a person's battery.
    #[test]
    fn a_network_that_cannot_be_read_is_refused_rather_than_guessed() {
        let unread = AMachine::default();
        assert_eq!(
            *read(&unread).unwrap_err(),
            Outcome::Unanswered(Unanswered::NetworkUnread)
        );
    }
}
