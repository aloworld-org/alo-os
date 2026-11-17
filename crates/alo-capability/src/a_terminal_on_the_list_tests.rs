//! **The person's terminal choice decides, on the list, at every door** — and
//! is never kept.
//!
//! Each permission is asserted beside its refusal: a refusal alone could be a
//! check that refuses everything, and a permission alone could be a check that
//! was deleted.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_strings::Word;

use super::*;
use crate::arg::{Arg, Given, Takes};
use crate::authorised::Authorised;
use crate::call::Call;
use crate::full_access::Permitted;
use crate::grant::Grant;
use crate::reach::{Ask, Reach};
use crate::test_calls::{files, granting, hour, noon};
use crate::verb::{Effect, Requires, Verb};
use crate::verbs::Verbs;

/// A terminal a fresh machine can install.
const PTYXIS: &str = "app.devsuite.Ptyxis";

/// The ask for the terminal.
fn the_terminal() -> Ask {
    Ask::application(PTYXIS)
}

/// A list holding a grant over the terminal for an hour, made while the person
/// had allowed it.
fn a_terminal_granted() -> Grants {
    let mut grants = granting(&["/home/anna/Invoices"]);
    grants.grant(
        Grant::checked_for_when(
            &files(),
            Reach::Application(PTYXIS.to_owned()),
            noon(),
            hour(),
            ATerminalForTheAgent::MayOpenOne,
        )
        .unwrap(),
    );
    grants
}

/// A read over an application's window, the shape `accessible.read_window`
/// has.
fn reading_the_window_of(application: &str) -> Call {
    let read = Verb::checked(
        "read_window",
        Word::saying("testing.terminal.purpose", "read a window"),
        Effect::Read,
        vec![Arg::taking(
            "application",
            Word::saying("testing.terminal.application", "the application"),
            Takes::Application,
        )],
        Requires::grants_over(["application"]),
        Word::saying("testing.terminal.sentence", "read {application}"),
    )
    .unwrap();
    let mut verbs = Verbs::default();
    verbs.declare(read).unwrap();
    verbs
        .call("read_window", &[("application", Given::text(application))])
        .unwrap()
}

/// **A list nobody told about the terminal allows none**, which is how every
/// list a service keeps arrives.
#[test]
fn a_list_arrives_allowing_no_terminal() {
    assert_eq!(
        Grants::default().the_terminal(),
        ATerminalForTheAgent::NotAtAll
    );
    assert_eq!(
        a_terminal_granted().the_terminal(),
        ATerminalForTheAgent::NotAtAll
    );
}

/// **The copy carries the choice and the list it came from does not.**
#[test]
fn the_copy_carries_the_choice_and_the_kept_list_does_not() {
    let kept = a_terminal_granted();
    let now = kept.with_the_terminal_as(ATerminalForTheAgent::MayOpenOne);
    assert_eq!(now.the_terminal(), ATerminalForTheAgent::MayOpenOne);
    assert_eq!(kept.the_terminal(), ATerminalForTheAgent::NotAtAll);
    assert_eq!(
        now.with_the_terminal_as(ATerminalForTheAgent::NotAtAll)
            .the_terminal(),
        ATerminalForTheAgent::NotAtAll,
        "a copy told not at all still allowed a terminal"
    );
}

/// **Never written down**: a list that carried the choice reads back without
/// it, so the grants file cannot become a second answer to the question.
#[test]
fn the_choice_is_never_written_into_the_grants() {
    let now = a_terminal_granted().with_the_terminal_as(ATerminalForTheAgent::MayOpenOne);
    let written = toml::to_string(&now).unwrap();
    assert!(
        !written.contains("may-open-one") && !written.contains("terminal"),
        "the choice was written into the grants: {written}"
    );
    let back: Grants = toml::from_str(&written).unwrap();
    assert_eq!(back.the_terminal(), ATerminalForTheAgent::NotAtAll);
}

/// **Under a grant**, as at *ask each time*: the grant over the terminal
/// permits only while the list says the person allows it.
#[test]
fn a_grant_over_a_terminal_permits_only_while_the_person_allows_it() {
    let kept = a_terminal_granted();
    assert!(
        kept.permitting(&files(), &the_terminal(), noon()).is_err(),
        "a grant over a terminal permitted on a list that does not allow one"
    );
    let allowed = kept.with_the_terminal_as(ATerminalForTheAgent::MayOpenOne);
    assert!(
        allowed
            .permitting(&files(), &the_terminal(), noon())
            .is_ok(),
        "the person allowed it and the grant over the terminal was refused"
    );
    assert!(
        matches!(
            allowed.permitted(&files(), &the_terminal(), noon()),
            Ok(Permitted::ByAGrant(_))
        ),
        "{:?}",
        allowed.permitted(&files(), &the_terminal(), noon())
    );
}

/// **At full access**, with no grant over the terminal: reached only where the
/// person allowed it.
#[test]
fn full_access_reaches_a_terminal_only_where_the_person_allows_it() {
    let at = Grants::default().at_full_access_for(&files());
    assert!(at.permitted(&files(), &the_terminal(), noon()).is_err());
    assert_eq!(
        at.with_the_terminal_as(ATerminalForTheAgent::MayOpenOne)
            .permitted(&files(), &the_terminal(), noon()),
        Ok(Permitted::ByFullAccess)
    );
}

/// **The door every read goes through follows the list**, so the read of a
/// terminal's window is authorised where the person allows it and refused
/// where they do not.
#[test]
fn a_read_of_a_terminals_window_follows_the_list() {
    let call = reading_the_window_of(PTYXIS);
    let kept = a_terminal_granted();
    assert!(
        Authorised::read(&call, &files(), &kept, noon()).is_err(),
        "a terminal's window was read on a list that does not allow one"
    );
    assert!(call.refusal(&kept, &files(), noon()).is_some());
    let allowed = kept.with_the_terminal_as(ATerminalForTheAgent::MayOpenOne);
    assert!(
        Authorised::read(&call, &files(), &allowed, noon()).is_ok(),
        "the person allowed it and the read was refused"
    );
    assert!(call.refusal(&allowed, &files(), noon()).is_none());
    assert!(call.permitting(&allowed, &files(), noon()).is_ok());
}

/// **Allowing a terminal widens nothing else**: a folder nobody granted is
/// still refused on the copy.
#[test]
fn allowing_a_terminal_reaches_nothing_else() {
    let allowed = a_terminal_granted().with_the_terminal_as(ATerminalForTheAgent::MayOpenOne);
    assert!(
        allowed
            .permitted(&files(), &Ask::path("/home/anna/Taxes/2024.pdf"), noon())
            .is_err()
    );
    assert!(
        Grants::default()
            .with_the_terminal_as(ATerminalForTheAgent::MayOpenOne)
            .permitted(&files(), &the_terminal(), noon())
            .is_err(),
        "allowing a terminal stood in for a grant over it"
    );
}
