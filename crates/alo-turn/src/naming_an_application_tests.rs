//! An application named by its name is put into a call as the identifier of the
//! one installed application that answers to it — only where the grants permit
//! that one, only in an argument that takes an application, and never where the
//! name means several.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_adapters::fallback_verbs::{self, NAME, TEXT, TYPE_TEXT};
use alo_applications::Installed;
use alo_applications::application::Application;
use alo_applications::verbs::{APPLICATION, OPEN_APPLICATION};
use alo_capability::{Given, Grant, Grantee, Grants, Reach, Verbs};

use super::as_this_machine_names_them;

const EDITOR: &str = "com.vscodium.codium";

fn verbs() -> Verbs {
    crate::the_verbs_a_whole_machine_offers().unwrap()
}

fn installed() -> Installed {
    Installed::holding([
        Application::called(EDITOR, "VSCodium").unwrap(),
        Application::called("org.gnome.TextEditor", "Text Editor").unwrap(),
        Application::called("org.kde.kwrite", "Text Editor").unwrap(),
    ])
}

fn agent() -> Grantee {
    Grantee::named("alo")
}

fn at_full_access() -> Grants {
    Grants::default().at_full_access_for(&agent())
}

fn named(
    grants: &Grants,
    verb: &str,
    given: &[(&'static str, Given)],
) -> Vec<(&'static str, Given)> {
    as_this_machine_names_them(
        &verbs(),
        &installed(),
        grants,
        &agent(),
        verb,
        given,
        SystemTime::now(),
    )
}

/// **The name a person reads is the application the machine has**: VSCodium,
/// in any case, is `com.vscodium.codium`.
#[test]
fn a_name_one_installed_application_answers_to_is_its_identifier() {
    for said in ["VSCodium", "vscodium", "codium", EDITOR] {
        assert_eq!(
            named(
                &at_full_access(),
                OPEN_APPLICATION,
                &[(APPLICATION, Given::text(said))]
            ),
            [(APPLICATION, Given::text(EDITOR))],
            "{said}"
        );
    }
}

/// **Not permitted, not rewritten**: with no grant and no full access the call
/// keeps what was sent, so it is refused in those words.
#[test]
fn a_name_nothing_permits_is_left_as_it_was_sent() {
    assert_eq!(
        named(
            &Grants::default(),
            OPEN_APPLICATION,
            &[(APPLICATION, Given::text("VSCodium"))]
        ),
        [(APPLICATION, Given::text("VSCodium"))]
    );
}

/// **Two applications answering to one name make it name neither.**
#[test]
fn a_name_several_answer_to_is_left_as_it_was_sent() {
    assert_eq!(
        named(
            &at_full_access(),
            OPEN_APPLICATION,
            &[(APPLICATION, Given::text("Text Editor"))]
        ),
        [(APPLICATION, Given::text("Text Editor"))]
    );
}

/// **Only an argument that takes an application**: the text typed and the
/// field's name are left alone even where they are an application's name.
#[test]
fn an_argument_that_is_not_an_application_is_left_as_it_was_sent() {
    let given = [
        (fallback_verbs::APPLICATION, Given::text("VSCodium")),
        (NAME, Given::text("VSCodium")),
        (TEXT, Given::text("VSCodium")),
    ];
    assert_eq!(
        named(&at_full_access(), TYPE_TEXT, &given),
        [
            (fallback_verbs::APPLICATION, Given::text(EDITOR)),
            (NAME, Given::text("VSCodium")),
            (TEXT, Given::text("VSCodium")),
        ]
    );
}

/// **A verb the machine does not offer changes nothing.**
#[test]
fn a_verb_nobody_declared_is_left_as_it_was_sent() {
    assert_eq!(
        named(
            &at_full_access(),
            "no_such_verb",
            &[(APPLICATION, Given::text("VSCodium"))]
        ),
        [(APPLICATION, Given::text("VSCodium"))]
    );
}

/// **A name nothing installed answers to is left**, for the machine to refuse
/// as not installed.
#[test]
fn a_name_nothing_installed_answers_to_is_left_as_it_was_sent() {
    assert_eq!(
        named(
            &at_full_access(),
            OPEN_APPLICATION,
            &[(APPLICATION, Given::text("GIMP"))]
        ),
        [(APPLICATION, Given::text("GIMP"))]
    );
}

const FIREFOX: &str = "org.mozilla.firefox";

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_800_000_000)
}

/// The grants with one application granted to the agent, from noon for an
/// hour.
fn granting(identifier: &str) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "alo",
            Reach::Application(identifier.to_owned()),
            noon(),
            Duration::from_secs(3600),
        )
        .unwrap(),
    );
    grants
}

fn named_at(grants: &Grants, now: SystemTime, said: &str) -> Vec<(&'static str, Given)> {
    let installed = Installed::holding([
        Application::called(EDITOR, "VSCodium").unwrap(),
        Application::called(FIREFOX, "Firefox").unwrap(),
        Application::called("org.gnome.TextEditor", "Text Editor").unwrap(),
        Application::called("org.kde.kwrite", "Text Editor").unwrap(),
    ]);
    let given = [(APPLICATION, Given::text(said))];
    as_this_machine_names_them(
        &verbs(),
        &installed,
        grants,
        &agent(),
        OPEN_APPLICATION,
        &given,
        now,
    )
}

/// **A grant names the one it grants**: VSCodium granted, its name is its
/// identifier, and Firefox's name, not granted, stays as it was sent.
#[test]
fn a_grant_for_one_application_names_that_one_and_no_other() {
    let grants = granting(EDITOR);
    let a_minute_in = noon() + Duration::from_secs(60);
    assert_eq!(
        named_at(&grants, a_minute_in, "VSCodium"),
        [(APPLICATION, Given::text(EDITOR))]
    );
    assert_eq!(
        named_at(&grants, a_minute_in, "Firefox"),
        [(APPLICATION, Given::text("Firefox"))]
    );
}

/// **An expired grant permits nothing**, so the name stays.
#[test]
fn a_name_whose_grant_has_expired_is_left_as_it_was_sent() {
    let two_hours_on = noon() + Duration::from_secs(7200);
    assert_eq!(
        named_at(&granting(EDITOR), two_hours_on, "VSCodium"),
        [(APPLICATION, Given::text("VSCodium"))]
    );
}

/// **A shared name names neither, whatever is granted**: only one of the two
/// Text Editors is granted, and the name is still not that one.
#[test]
fn a_shared_name_names_neither_even_where_only_one_is_granted() {
    let a_minute_in = noon() + Duration::from_secs(60);
    assert_eq!(
        named_at(
            &granting("org.gnome.TextEditor"),
            a_minute_in,
            "Text Editor"
        ),
        [(APPLICATION, Given::text("Text Editor"))]
    );
}

/// **Full access is the agent's it was given to**: another agent's full
/// access names nothing for this one.
#[test]
fn another_agents_full_access_names_nothing() {
    let theirs = Grants::default().at_full_access_for(&Grantee::named("someone-else"));
    assert_eq!(
        named_at(&theirs, noon(), "VSCodium"),
        [(APPLICATION, Given::text("VSCodium"))]
    );
}
