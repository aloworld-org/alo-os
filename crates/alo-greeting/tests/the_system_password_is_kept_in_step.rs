//! **The system's password is kept in step at every sign-in, and only then**
//! (ADR 0150, the owner's *"Keep both in step"*).
//!
//! The keeper is a fixture that remembers what it was handed, because *after a
//! password verified, with that person's number, once* is a statement about
//! how often something happened and for whom — and the roads where the right
//! answer is *nothing was handed over* matter as much as the road where it was.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::rc::Rc;
use std::cell::RefCell;

use alo_accounts::Accounts;
use alo_greeting::{Greeted, Greeting, Knocking, KeepingTheSystemPassword, NotAnswered};
use alo_sessiond::{Answered, Knock};

/// Hers, and it is right.
const HERS: &str = "1234";

/// A door that opens whatever it is asked for.
#[derive(Debug)]
struct ADoor;

impl Knocking for ADoor {
    fn knock(&self, _: Knock) -> Result<Answered, NotAnswered> {
        Ok(Answered::Opened)
    }
}

/// A keeper that remembers each number and password it was handed.
#[derive(Clone, Default)]
struct AKeeper(Rc<RefCell<Vec<(u32, String)>>>);

impl KeepingTheSystemPassword for AKeeper {
    fn keep(&self, person: u32, typed: &str) {
        self.0.borrow_mut().push((person, typed.to_owned()));
    }
}

/// A greeter whose machine has Ada as its person, number 1000.
fn a_greeter(keeper: &AKeeper) -> Greeting<ADoor> {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("ada", 1000, HERS).unwrap();
    Greeting::of(accounts, 1000, ADoor).keeping_the_system_password_with(Box::new(keeper.clone()))
}

/// **A sign-in that succeeds hands the very password to the keeper, once,
/// with the person's number.**
#[test]
fn a_sign_in_that_succeeds_keeps_the_password_in_step() {
    let keeper = AKeeper::default();
    let greeter = a_greeter(&keeper);
    assert!(matches!(greeter.signs_in("ada", HERS), Greeted::SignedIn(_)));
    assert_eq!(*keeper.0.borrow(), [(1000, HERS.to_owned())]);
}

/// **A wrong password, and a name with no account, hand nothing over**: the
/// system's list is never given a password alo did not just verify.
#[test]
fn a_refused_sign_in_keeps_nothing() {
    let keeper = AKeeper::default();
    let greeter = a_greeter(&keeper);
    assert!(matches!(greeter.signs_in("ada", "4321"), Greeted::Refused(_)));
    assert!(matches!(greeter.signs_in("grace", HERS), Greeted::Refused(_)));
    assert!(keeper.0.borrow().is_empty());
}

/// **An account whose number is not this machine's person keeps nothing**,
/// because the sign-in it would have kept in step is refused.
#[test]
fn somebody_who_is_not_this_machines_person_keeps_nothing() {
    let keeper = AKeeper::default();
    let mut accounts = Accounts::none().unwrap();
    accounts.created("ada", 1001, HERS).unwrap();
    let greeter = Greeting::of(accounts, 1000, ADoor)
        .keeping_the_system_password_with(Box::new(keeper.clone()));
    assert!(matches!(greeter.signs_in("ada", HERS), Greeted::Refused(_)));
    assert!(keeper.0.borrow().is_empty());
}

/// **A greeter given no keeper keeps nothing and still signs people in** —
/// every greeter in a test is one.
#[test]
fn a_greeter_with_no_keeper_still_signs_people_in() {
    let mut accounts = Accounts::none().unwrap();
    accounts.created("ada", 1000, HERS).unwrap();
    let greeter = Greeting::of(accounts, 1000, ADoor);
    assert!(matches!(greeter.signs_in("ada", HERS), Greeted::SignedIn(_)));
}
