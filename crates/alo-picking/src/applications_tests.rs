//! Tests for [`super`]: a person choosing one application, by name, from the
//! ones this machine has, and the grant that choice becomes.
//!
//! **The refusal path is tested as carefully as the happy one**, because an
//! application grant is something an agent reaches: an application nobody
//! chose is still refused, and a grant that ends or is taken back stops at
//! that moment.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{Ask, Grantee, Grants};

use super::*;
use crate::testing::in_english;
use crate::{Granting, NotPicked};

/// The agent every test here grants to.
const HERS: &str = "@files";

/// The editor of the VSCodium demo.
const CODIUM: &str = "com.vscodium.codium";

/// The browser.
const FIREFOX: &str = "org.mozilla.firefox";

/// A terminal, which is a person's own (ADR 0043).
const PTYXIS: &str = "app.devsuite.Ptyxis";

/// A moment.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// An hour.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// What a machine with an editor, a browser and a terminal offers, in the
/// order its desktop entries were read.
fn this_machine() -> Applications {
    Applications::of([
        (FIREFOX.to_owned(), Some("Firefox".to_owned())),
        (PTYXIS.to_owned(), Some("Terminal".to_owned())),
        (CODIUM.to_owned(), Some("VSCodium".to_owned())),
    ])
}

/// Whether the agent may reach this application at this moment, asked the way
/// the daemon asks it of every application verb.
fn reaches(grants: &Grants, application: &str, at: SystemTime) -> bool {
    grants.permits(&Grantee::named(HERS), &Ask::application(application), at)
}

/// **Choosing VSCodium and saying an hour makes the grant the daemon honours**
/// — the one the application verbs ask — and nothing beside it: not the
/// browser listed next to it, and not any path.
#[test]
fn a_chosen_application_is_reachable_and_nothing_beside_it() {
    let mut grants = Grants::default();
    let chosen = this_machine().pick(CODIUM).unwrap();
    let id = Granting::to(HERS, hour())
        .of(&chosen, &mut grants, noon())
        .unwrap();
    assert!(id.is_some());
    assert_eq!(grants.len(), 1);

    assert!(reaches(&grants, CODIUM, noon()));
    assert!(
        !reaches(&grants, FIREFOX, noon()),
        "an application nobody chose is reachable"
    );
    assert!(
        !grants.permits(&Grantee::named(HERS), &Ask::path("/home/anna"), noon()),
        "a grant over an application reaches a folder"
    );
    // **Matched exactly at the daemon's question too**: a different case is a
    // different application.
    assert!(!reaches(&grants, "COM.VSCODIUM.CODIUM", noon()));
    // **And it is the agent's**: another agent is not given it.
    assert!(!grants.permits(&Grantee::named("@other"), &Ask::application(CODIUM), noon()));
}

/// **Nothing granted means every application is refused** — the state a
/// machine is in before a person chooses one, which is the state the demo was
/// stuck in.
#[test]
fn before_anything_is_chosen_every_application_is_refused() {
    let grants = Grants::default();
    for application in [CODIUM, FIREFOX, PTYXIS] {
        assert!(!reaches(&grants, application, noon()), "{application}");
    }
}

/// **An application that is not in the list cannot be chosen**, so there is
/// nothing to grant: a name typed or remembered from yesterday is refused in
/// words, and the grants are not touched.
#[test]
fn an_application_not_listed_is_refused_and_grants_nothing() {
    let refused = this_machine().pick("org.example.NotHere");
    assert_eq!(refused, Err(NotPicked::NotAnApplicationShown));
    // Matched exactly: a different case is a different application.
    assert_eq!(
        this_machine().pick("com.vscodium.Codium"),
        Err(NotPicked::NotAnApplicationShown)
    );
    let said = NotPicked::NotAnApplicationShown.said(&in_english());
    assert!(!said.is_a_bug(), "the refusal is not declared");
    assert!(said.into_text().contains("application"));
}

/// **A terminal is not offered**, and choosing one by name is refused before
/// a grant could be: whatever is typed into a terminal runs, so a grant over
/// one is the terminal's own choice (ADR 0111), not this list's.
#[test]
fn a_terminal_is_not_offered_and_cannot_be_chosen() {
    let applications = this_machine();
    let listed: Vec<&str> = applications
        .listed()
        .iter()
        .map(Listed::identifier)
        .collect();
    assert!(!listed.contains(&PTYXIS), "a terminal is offered: {listed:?}");
    assert_eq!(
        this_machine().pick(PTYXIS),
        Err(NotPicked::NotAnApplicationShown)
    );
}

/// **The list is ordered by the name a person reads**, with an application
/// that gave no name shown by its identifier; an empty identifier is not an
/// application, and one read twice is listed once — the first reading, which
/// is the person's own copy of an entry (XDG's order).
#[test]
fn the_list_is_by_name_once_each_and_never_empty_names() {
    let applications = Applications::of([
        (FIREFOX.to_owned(), Some("Firefox".to_owned())),
        (String::new(), Some("Nothing".to_owned())),
        ("   ".to_owned(), None),
        (CODIUM.to_owned(), Some("VSCodium".to_owned())),
        (CODIUM.to_owned(), Some("A second VSCodium".to_owned())),
        ("org.gnome.Calculator".to_owned(), None),
        ("org.gnome.Maps".to_owned(), Some(String::new())),
    ]);
    let shown: Vec<(&str, &str)> = applications
        .listed()
        .iter()
        .map(|it| (it.identifier(), it.called()))
        .collect();
    assert_eq!(
        shown,
        [
            (FIREFOX, "Firefox"),
            ("org.gnome.Calculator", "org.gnome.Calculator"),
            ("org.gnome.Maps", "org.gnome.Maps"),
            (CODIUM, "VSCodium"),
        ]
    );
    assert!(!applications.is_empty());
    assert!(Applications::of([]).is_empty());
}

/// **The pick carries the application that was chosen** — its identifier and
/// the name it was listed under — and nothing a caller could have edited.
#[test]
fn the_pick_is_the_application_that_was_chosen() {
    let chosen = this_machine().pick(CODIUM).unwrap();
    let picked = chosen.application().unwrap();
    assert_eq!(picked.identifier(), CODIUM);
    assert_eq!(picked.called(), "VSCodium");
    assert_eq!(chosen.folder(), None, "an application is not a folder");
    assert!(!chosen.is_nothing());
}

/// **The grant ends when it was always going to**: reachable for the hour it
/// was given, refused at the first moment after.
#[test]
fn an_expired_application_grant_is_refused_from_that_moment() {
    let mut grants = Grants::default();
    let chosen = this_machine().pick(CODIUM).unwrap();
    Granting::to(HERS, hour())
        .of(&chosen, &mut grants, noon())
        .unwrap();
    assert!(reaches(&grants, CODIUM, noon() + hour() - Duration::from_secs(1)));
    assert!(
        !reaches(&grants, CODIUM, noon() + hour()),
        "an expired grant still reaches"
    );
}

/// **A revoked grant is refused at once** — the next question, not the next
/// restart.
#[test]
fn a_revoked_application_grant_is_refused_at_once() {
    let mut grants = Grants::default();
    let chosen = this_machine().pick(CODIUM).unwrap();
    let id = Granting::to(HERS, hour())
        .of(&chosen, &mut grants, noon())
        .unwrap()
        .unwrap();
    assert!(reaches(&grants, CODIUM, noon()));
    assert!(grants.revoke(id));
    assert!(!reaches(&grants, CODIUM, noon()), "a revoked grant still reaches");
}

/// **Until I remove it lasts until it is removed**, and removing it ends it.
#[test]
fn an_application_given_until_removed_lasts_until_it_is_removed() {
    let mut grants = Grants::default();
    let chosen = this_machine().pick(CODIUM).unwrap();
    let id = Granting::to_until_removed(HERS)
        .of(&chosen, &mut grants, noon())
        .unwrap()
        .unwrap();
    let a_year = Duration::from_secs(365 * 24 * 60 * 60);
    assert!(reaches(&grants, CODIUM, noon() + a_year));
    assert!(grants.revoke(id));
    assert!(!reaches(&grants, CODIUM, noon() + a_year));
}
