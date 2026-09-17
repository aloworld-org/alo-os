//! The machine, the person, the grants and the turn this crate's own tests are
//! written against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The machine counts its holds
//!
//! [`TheMachine`] is a [`Logind`] whose holds are [`Hold`]s, each of which
//! counts itself alive in a shared [`Holds`] and uncounts itself when dropped.
//! So *a revoked grant let go of the machine* is a number going to zero, not a
//! flag somebody set.
//!
//! # The session is a real sign-in, and the turn a real turn
//!
//! [`anna`] is what `alo-accounts` hands back when a password verifies, the
//! only way an `alo_accounts::Session` exists. [`a_turn_on_a_machine`] begins a
//! real `alo_turn::Turning` from an invocation, on a machine with a record, so
//! *what became of the turn was written down* is an entry in that record.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

use alo_accounts::{Accounts, Session};
use alo_capability::{Applicant, Facility, Grant, GrantId, Grants, Reach};
use alo_context::Context;
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Reaching};
use alo_record::Record;
use alo_strings::{Said, Strings, Vocabulary};
use alo_turn::{Doing, Done, Machine, NoBoundary, Turning};

use crate::logind::{Inhibit, LockedFirst, Logind, NotHeld, NotSlept};

/// The number this machine's person runs as.
const PERSON: u32 = 1000;

/// Anna's password, and it is right.
pub(crate) const ANNAS: &str = "correct horse battery staple";

/// The moment every test here is written against.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long a grant and a turn stand.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// This crate's words, and the words of every crate whose refusals and
/// machine these tests reach.
pub(crate) fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::declare_into(&mut vocabulary).unwrap();
    alo_portals::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The machine's accounts: Anna, at this machine's number.
pub(crate) fn the_accounts() -> Accounts {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("anna", PERSON, ANNAS).unwrap();
    accounts
}

/// Anna's session, as a sign-in really opens one. Made once: hashing a
/// password is deliberately slow.
pub(crate) fn anna() -> Session {
    static ANNA: OnceLock<Session> = OnceLock::new();
    ANNA.get_or_init(|| {
        let signed_in = the_accounts().signs_in("anna", ANNAS).unwrap();
        Session::opened(signed_in, PERSON).unwrap()
    })
    .clone()
}

/// A grant allowing this application to keep the machine awake for an hour
/// from noon, and its handle.
pub(crate) fn allowed_to_stay_awake(application: &str) -> (Grants, GrantId) {
    let mut grants = Grants::default();
    let id = grants.grant(
        Grant::checked_for(
            &Applicant::named(application).grantee(),
            Reach::Facility(Facility::Sleep),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    (grants, id)
}

/// What the machine was asked, shared between it and every hold it handed out.
#[derive(Debug, Default)]
struct Asked {
    /// How many holds are alive.
    alive: usize,
    /// Every hold taken, in order.
    taken: Vec<Inhibit>,
    /// How many times the machine went to sleep.
    slept: usize,
}

/// A window onto what the machine was asked.
#[derive(Debug, Clone, Default)]
pub(crate) struct Holds(Rc<RefCell<Asked>>);

impl Holds {
    /// How many holds are alive now.
    pub(crate) fn alive(&self) -> usize {
        self.0.borrow().alive
    }

    /// Every hold taken, in order.
    pub(crate) fn taken(&self) -> Vec<Inhibit> {
        self.0.borrow().taken.clone()
    }

    /// How many times the machine went to sleep.
    pub(crate) fn slept(&self) -> usize {
        self.0.borrow().slept
    }
}

/// One hold, counted alive until it is dropped.
#[derive(Debug)]
pub(crate) struct Hold(Holds);

impl Drop for Hold {
    fn drop(&mut self) {
        let mut asked = (self.0).0.borrow_mut();
        asked.alive = asked.alive.saturating_sub(1);
    }
}

/// A machine that takes every hold and sleeps when asked — or refuses both.
pub(crate) struct TheMachine {
    /// What it was asked.
    holds: Holds,
    /// Whether it refuses.
    refuses: bool,
}

impl TheMachine {
    /// A machine that holds and sleeps.
    pub(crate) fn holding(holds: &Holds) -> Self {
        Self {
            holds: holds.clone(),
            refuses: false,
        }
    }

    /// A machine that will neither hold nor sleep.
    pub(crate) fn refusing(holds: &Holds) -> Self {
        Self {
            holds: holds.clone(),
            refuses: true,
        }
    }
}

impl Logind for TheMachine {
    type Held = Hold;

    fn hold(&mut self, inhibit: Inhibit, why: &Said) -> Result<Hold, NotHeld> {
        assert!(!why.is_a_bug(), "a hold described by a key: {why}");
        if self.refuses {
            return Err(NotHeld {
                machine: "org.freedesktop.DBus.Error.AccessDenied".to_owned(),
            });
        }
        let mut asked = self.holds.0.borrow_mut();
        asked.alive = asked.alive.saturating_add(1);
        asked.taken.push(inhibit);
        Ok(Hold(self.holds.clone()))
    }

    fn sleep(&mut self, _locked_first: LockedFirst) -> Result<(), NotSlept> {
        if self.refuses {
            return Err(NotSlept {
                machine: "Sleep verb not supported".to_owned(),
            });
        }
        let mut asked = self.holds.0.borrow_mut();
        asked.slept = asked.slept.saturating_add(1);
        Ok(())
    }
}

/// Begin a real turn from an invocation at noon, lasting an hour, hand it to
/// `body` by value with the machine's grants, and hand back the record it
/// wrote to.
pub(crate) fn a_turn_on_a_machine(body: impl FnOnce(Turning<'_, '_>, &mut Grants)) -> Record {
    let strings = in_english();
    let mut grants = Grants::default();
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    {
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let turning = Turning::beginning(
            Context::at_invocation(noon()),
            "@files",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        body(turning, &mut grants);
    }
    record
}

/// [`a_turn_on_a_machine`], for a test that only looks at the turn.
pub(crate) fn on_a_machine(body: impl FnOnce(&mut Turning<'_, '_>)) {
    let _record = a_turn_on_a_machine(|mut turning, grants| {
        body(&mut turning);
        let _ended = turning.ending(grants);
    });
}

/// A machine with nothing in front of a turn, which is not a machine alo OS
/// ships — `alo_turn::bounding` says why no library here has one.
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(&mut self, _reaching: &Reaching, doing: Doing<'_>) -> Result<Done, NoBoundary> {
        Ok(doing.done())
    }

    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), NoBoundary> {
        doing();
        Ok(())
    }
}
