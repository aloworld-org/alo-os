//! Everything holding this machine awake right now, and the machine's own hold
//! behind each.
//!
//! [`Holding`] is the closed list of [`crate::Keeper`]s as they stand, and it
//! has exactly three doors in — one per keeper — each of which asks for proof
//! of the right to be there:
//!
//! - [`Holding::your_setting`] takes the person's own value;
//! - [`Holding::an_application`] takes an `alo_portals::Allowed` **for the
//!   inhibit portal**, and asks the grants again before it holds anything;
//! - [`Holding::a_turn`] takes an `alo_turn::Turning` — a turn, which begins
//!   only at a person's invocation — and holds for that turn's own length.
//!
//! # An agent cannot keep the machine awake by asking
//!
//! There is no door here that takes an agent's name, a verb or a duration.
//! A turn's hold is the turn's length, read off the turn, and an agent has no
//! verb that reaches this crate at all
//! (`tests/an_agent_cannot_keep_this_machine_awake.rs`). So the one way an
//! agent's work keeps a laptop awake is by being work a person already asked
//! for, and only until it ends:
//!
//! ```compile_fail
//! # use alo_sleeping::Holding;
//! # fn holding<L: alo_sleeping::Logind>(holding: &mut Holding<L::Held>, logind: &mut L, strings: &alo_strings::Strings) {
//! let agent = alo_capability::Grantee::named("@files");
//! holding.a_turn(&agent, std::time::SystemTime::now(), logind, strings);
//! # }
//! ```
//!
//! That fails with **E0308, mismatched types**: a hold for a turn is made from
//! a turn.
//!
//! # A revoked grant lets go at once, and an ended turn is gone
//!
//! [`Holding::as_it_stands`] is asked at every moment sleep is decided, and it
//! asks the grants again for every application and the clock for every turn.
//! Whatever no longer stands is taken off the list **and its hold on the machine
//! dropped** in the same call, so the machine's own list and this one never
//! disagree for longer than it takes to ask.

use std::time::SystemTime;

use alo_capability::{Applicant, Grantee, Grants};
use alo_portals::{Allowed, Portal, Request};
use alo_strings::Strings;
use alo_turn::Turning;

use crate::keeper::Keeper;
use crate::logind::{Inhibit, Logind};
use crate::refusing::NotKeptAwake;

/// The handle an application's or a turn's hold is let go by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HoldId(u64);

/// One application holding the machine awake.
#[derive(Debug)]
struct ForAnApplication<H> {
    /// The handle it lets go by.
    id: HoldId,
    /// Which application.
    application: Applicant,
    /// The machine's own hold, dropped with this.
    _held: H,
}

/// One turn holding the machine awake.
#[derive(Debug)]
struct ForATurn<H> {
    /// The handle it lets go by.
    id: HoldId,
    /// Whose turn.
    agent: Grantee,
    /// When the turn ends, and the hold with it.
    until: SystemTime,
    /// The machine's own hold, dropped with this.
    _held: H,
}

/// Everything holding this machine awake.
///
/// `H` is the machine's own hold ([`Logind::Held`]); every entry here owns one,
/// so an entry that goes lets go of the machine too.
#[derive(Debug)]
pub struct Holding<H> {
    /// The next handle.
    next: u64,
    /// The person's own setting, when it is on.
    setting: Option<H>,
    /// Applications, in the order they asked.
    applications: Vec<ForAnApplication<H>>,
    /// Turns, in the order they began holding.
    turns: Vec<ForATurn<H>>,
}

impl<H> Holding<H> {
    /// Nothing is holding the machine awake.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            next: 0,
            setting: None,
            applications: Vec::new(),
            turns: Vec::new(),
        }
    }

    /// The person's setting to keep the machine awake, turned on or off.
    ///
    /// # Errors
    /// [`NotKeptAwake::NotHeld`] when it is turned on and the machine would not
    /// take the hold — and then the setting is not holding anything, rather
    /// than holding in this list and not on the machine.
    pub fn your_setting<L: Logind<Held = H>>(
        &mut self,
        on: bool,
        logind: &mut L,
        strings: &Strings,
    ) -> Result<(), NotKeptAwake> {
        if !on {
            self.setting = None;
            return Ok(());
        }
        if self.setting.is_none() {
            let why = Keeper::YourSetting.said(strings);
            let held = logind
                .hold(Inhibit::Idle, &why)
                .map_err(NotKeptAwake::NotHeld)?;
            self.setting = Some(held);
        }
        Ok(())
    }

    /// An application that asked the inhibit portal, and was allowed.
    ///
    /// **The grants are asked again here**, at `now`, rather than trusted from
    /// whenever `allowed` was judged — a grant revoked between the two is
    /// refused.
    ///
    /// # Errors
    /// [`NotKeptAwake::NotAskingToStayAwake`] when `allowed` is for another
    /// portal; [`NotKeptAwake::NotAllowed`] when the grants no longer allow it;
    /// [`NotKeptAwake::NotHeld`] when the machine would not take the hold.
    pub fn an_application<L: Logind<Held = H>>(
        &mut self,
        allowed: &Allowed,
        grants: &Grants,
        now: SystemTime,
        logind: &mut L,
        strings: &Strings,
    ) -> Result<HoldId, NotKeptAwake> {
        let application = allowed.application().clone();
        if allowed.portal() != Portal::Inhibit {
            return Err(NotKeptAwake::NotAskingToStayAwake(application));
        }
        still_allowed(&application, grants, now)?;
        let why = Keeper::AnApplication(application.clone()).said(strings);
        let held = logind
            .hold(Inhibit::Idle, &why)
            .map_err(NotKeptAwake::NotHeld)?;
        let id = self.next_id();
        self.applications.push(ForAnApplication {
            id,
            application,
            _held: held,
        });
        Ok(id)
    }

    /// A turn that is running, held for its own length.
    ///
    /// # Errors
    /// [`NotKeptAwake::TheAgentIsNotWorking`] for a turn that has ended at
    /// `now` or has stopped; [`NotKeptAwake::NotHeld`] when the machine would
    /// not take the hold.
    pub fn a_turn<L: Logind<Held = H>>(
        &mut self,
        turning: &Turning<'_, '_>,
        now: SystemTime,
        logind: &mut L,
        strings: &Strings,
    ) -> Result<HoldId, NotKeptAwake> {
        let until = turning.ends();
        if turning.is_closed() || until <= now {
            return Err(NotKeptAwake::TheAgentIsNotWorking);
        }
        let agent = turning.grantee().clone();
        if let Some(already) = self
            .turns
            .iter()
            .find(|turn| turn.agent == agent && turn.until == until)
        {
            return Ok(already.id);
        }
        let why = Keeper::TheAgent(agent.clone()).said(strings);
        let held = logind
            .hold(Inhibit::Idle, &why)
            .map_err(NotKeptAwake::NotHeld)?;
        let id = self.next_id();
        self.turns.push(ForATurn {
            id,
            agent,
            until,
            _held: held,
        });
        Ok(id)
    }

    /// Let go of one hold: the application closed its request, or the turn
    /// ended early. Says whether there was one.
    pub fn lets_go(&mut self, id: HoldId) -> bool {
        let before = self.applications.len() + self.turns.len();
        self.applications.retain(|held| held.id != id);
        self.turns.retain(|held| held.id != id);
        before != self.applications.len() + self.turns.len()
    }

    /// Everything holding the machine awake at `now`, named.
    ///
    /// An application whose grant no longer allows it and a turn that has
    /// ended are taken off first, and their holds on the machine dropped.
    pub fn as_it_stands(&mut self, grants: &Grants, now: SystemTime) -> Vec<Keeper> {
        self.applications
            .retain(|held| still_allowed(&held.application, grants, now).is_ok());
        self.turns.retain(|held| held.until > now);

        let setting = self.setting.iter().map(|_| Keeper::YourSetting);
        let applications = self
            .applications
            .iter()
            .map(|held| Keeper::AnApplication(held.application.clone()));
        let turns = self
            .turns
            .iter()
            .map(|held| Keeper::TheAgent(held.agent.clone()));
        setting.chain(applications).chain(turns).collect()
    }

    /// A handle nothing has had.
    fn next_id(&mut self) -> HoldId {
        let id = HoldId(self.next);
        self.next = self.next.saturating_add(1);
        id
    }
}

/// Whether this application may hold the inhibit portal at `now`, asked of the
/// grants the way `alo-portals` asks them.
fn still_allowed(
    application: &Applicant,
    grants: &Grants,
    now: SystemTime,
) -> Result<(), NotKeptAwake> {
    let request = Request::of(application.as_str(), Portal::Inhibit)
        .map_err(|_| NotKeptAwake::NotAskingToStayAwake(application.clone()))?;
    request
        .judged(grants, now)
        .map(|_| ())
        .map_err(NotKeptAwake::NotAllowed)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_capability::{Facility, Grant, Reach};

    use super::*;
    use crate::testing::{
        Holds, TheMachine, allowed_to_stay_awake, hour, in_english, noon, on_a_machine,
    };

    /// **An application allowed the inhibit portal keeps the machine awake,
    /// named, and a revoked grant lets go at the next moment anything asks** —
    /// the machine's own hold with it.
    #[test]
    fn a_revoked_grant_lets_go_of_the_machine_at_once() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let (mut grants, id) = allowed_to_stay_awake("org.libreoffice.Impress");
        let allowed = Request::of("org.libreoffice.Impress", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();

        let mut holding = Holding::none();
        holding
            .an_application(&allowed, &grants, noon(), &mut logind, &strings)
            .unwrap();
        assert_eq!(
            holding.as_it_stands(&grants, noon()),
            [Keeper::AnApplication(Applicant::named(
                "org.libreoffice.Impress"
            ))]
        );
        assert_eq!(holds.alive(), 1);

        assert!(grants.revoke(id));
        assert!(holding.as_it_stands(&grants, noon()).is_empty());
        assert_eq!(holds.alive(), 0, "the machine's own hold went with it");
    }

    /// **A grant revoked after the request was judged is refused when the hold
    /// is taken**, and nothing is held on the machine.
    #[test]
    fn a_grant_revoked_after_judging_is_refused_at_the_hold() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let (mut grants, id) = allowed_to_stay_awake("org.libreoffice.Impress");
        let allowed = Request::of("org.libreoffice.Impress", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();
        grants.revoke(id);

        let mut holding = Holding::none();
        assert!(matches!(
            holding.an_application(&allowed, &grants, noon(), &mut logind, &strings),
            Err(NotKeptAwake::NotAllowed(_))
        ));
        assert_eq!(holds.alive(), 0);
    }

    /// **An expired grant is gone**: the hold stands until the grant's hour is
    /// up and not a moment after.
    #[test]
    fn an_expired_grant_is_gone() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let (grants, _) = allowed_to_stay_awake("org.libreoffice.Impress");
        let allowed = Request::of("org.libreoffice.Impress", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();
        let mut holding = Holding::none();
        holding
            .an_application(&allowed, &grants, noon(), &mut logind, &strings)
            .unwrap();

        let later = noon() + hour() / 2;
        assert_eq!(holding.as_it_stands(&grants, later).len(), 1);
        assert!(holding.as_it_stands(&grants, noon() + hour()).is_empty());
        assert_eq!(holds.alive(), 0);
    }

    /// **An application allowed the camera is not allowed to keep the machine
    /// awake with that permission**, and one granted nothing never gets as far
    /// as a request that could be allowed.
    #[test]
    fn another_portals_permission_does_not_keep_the_machine_awake() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked_for(
                &Applicant::named("org.gnome.Cheese").grantee(),
                Reach::Facility(Facility::Camera),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let camera = Request::of("org.gnome.Cheese", Portal::Camera)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();

        let mut holding = Holding::none();
        assert_eq!(
            holding.an_application(&camera, &grants, noon(), &mut logind, &strings),
            Err(NotKeptAwake::NotAskingToStayAwake(Applicant::named(
                "org.gnome.Cheese"
            )))
        );
        assert!(
            Request::of("org.gnome.Cheese", Portal::Inhibit)
                .unwrap()
                .judged(&grants, noon())
                .is_err()
        );
        assert!(holding.as_it_stands(&grants, noon()).is_empty());
        assert_eq!(holds.alive(), 0);
    }

    /// **An agent granted sleep itself is not an application**, so a grant made
    /// out to an agent never lets anything hold the inhibit portal in its name.
    #[test]
    fn a_grant_to_an_agent_is_not_an_applications() {
        let mut grants = Grants::default();
        if let Ok(grant) = Grant::checked_for(
            &Grantee::named("@files"),
            Reach::Facility(Facility::Sleep),
            noon(),
            hour(),
        ) {
            grants.grant(grant);
        }
        let asked = Request::of("@files", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon());
        assert!(matches!(
            asked,
            Err(alo_portals::Refused::NothingGranted { .. })
        ));
    }

    /// **A turn holds the machine awake for its own length and no longer**, and
    /// a turn that is over cannot begin to.
    #[test]
    fn a_turn_holds_for_its_own_length() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let mut holding = Holding::none();
        let grants = Grants::default();
        on_a_machine(|turning| {
            let id = holding
                .a_turn(turning, noon(), &mut logind, &strings)
                .unwrap();
            let again = holding
                .a_turn(turning, noon(), &mut logind, &strings)
                .unwrap();
            assert_eq!(id, again, "one turn, one hold");
            assert_eq!(holds.alive(), 1);
            assert!(matches!(
                holding.as_it_stands(&grants, noon()).as_slice(),
                [Keeper::TheAgent(_)]
            ));
            assert!(
                holding.as_it_stands(&grants, turning.ends()).is_empty(),
                "gone the moment the turn ends"
            );
            assert_eq!(holds.alive(), 0);

            assert_eq!(
                holding.a_turn(turning, turning.ends(), &mut logind, &strings),
                Err(NotKeptAwake::TheAgentIsNotWorking)
            );
            assert_eq!(holds.alive(), 0);
        });
    }

    /// **A hold the machine would not take is not on the list either**, for
    /// every one of the three.
    #[test]
    fn a_hold_the_machine_refused_is_not_held() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::refusing(&holds);
        let (grants, _) = allowed_to_stay_awake("org.libreoffice.Impress");
        let allowed = Request::of("org.libreoffice.Impress", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();
        let mut holding = Holding::none();

        assert!(matches!(
            holding.your_setting(true, &mut logind, &strings),
            Err(NotKeptAwake::NotHeld(_))
        ));
        assert!(matches!(
            holding.an_application(&allowed, &grants, noon(), &mut logind, &strings),
            Err(NotKeptAwake::NotHeld(_))
        ));
        on_a_machine(|turning| {
            assert!(matches!(
                holding.a_turn(turning, noon(), &mut logind, &strings),
                Err(NotKeptAwake::NotHeld(_))
            ));
        });
        assert!(holding.as_it_stands(&grants, noon()).is_empty());
    }

    /// The person's setting holds once however often it is turned on, and lets
    /// go when it is turned off; an application lets go by its handle.
    #[test]
    fn a_setting_and_an_application_let_go_when_asked() {
        let strings = in_english();
        let holds = Holds::default();
        let mut logind = TheMachine::holding(&holds);
        let (grants, _) = allowed_to_stay_awake("org.libreoffice.Impress");
        let allowed = Request::of("org.libreoffice.Impress", Portal::Inhibit)
            .unwrap()
            .judged(&grants, noon())
            .unwrap();
        let mut holding = Holding::none();

        holding.your_setting(true, &mut logind, &strings).unwrap();
        holding.your_setting(true, &mut logind, &strings).unwrap();
        let id = holding
            .an_application(&allowed, &grants, noon(), &mut logind, &strings)
            .unwrap();
        assert_eq!(holds.alive(), 2);
        assert_eq!(
            holding.as_it_stands(&grants, noon()).first(),
            Some(&Keeper::YourSetting)
        );

        assert!(holding.lets_go(id));
        assert!(!holding.lets_go(id));
        holding.your_setting(false, &mut logind, &strings).unwrap();
        assert!(holding.as_it_stands(&grants, noon()).is_empty());
        assert_eq!(holds.alive(), 0);
    }
}
