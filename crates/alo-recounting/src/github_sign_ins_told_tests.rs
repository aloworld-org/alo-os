//! A GitHub sign-in, read back as a person reads it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_record::{AtGitHub, Entry, WhyNotSignedIn};

use super::{Outcome, Told};
use crate::testing::{in_english, noon};

/// Every kind of GitHub sign-in the record keeps, with the outcome it reads
/// as.
fn every_kind() -> Vec<(AtGitHub, Outcome)> {
    vec![
        (
            AtGitHub::signed_in("anna").unwrap(),
            Outcome::SignedInToGitHub,
        ),
        (
            AtGitHub::signed_out(Some("anna")).unwrap(),
            Outcome::SignedOutOfGitHub,
        ),
        (
            AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
            Outcome::GitHubSignInWithoutGh,
        ),
        (
            AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some("HTTP 502"))),
            Outcome::GitHubSignInFailed,
        ),
        (
            AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived),
            Outcome::GitHubSignInWithoutACode,
        ),
        (
            AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
            Outcome::GitHubSignInCancelled,
        ),
        (
            AtGitHub::did_not_sign_out(Some("not logged in")),
            Outcome::GitHubSignOutFailed,
        ),
    ]
}

/// **Each kind reads as its own clause, in the person's language, naming
/// nobody's authority** — and no two read alike, because *it did not finish
/// because gh is missing* and *you cancelled it* are different afternoons.
#[test]
fn each_kind_of_github_sign_in_reads_as_its_own_clause() {
    let strings = in_english();
    let mut seen: Vec<String> = Vec::new();
    for (what, outcome) in every_kind() {
        let told = Told::of(&Entry::at_github(what, noon()));
        assert_eq!(told.outcome(), outcome);
        assert_eq!(told.agent(), None, "{outcome:?}");
        assert_eq!(told.at(), noon());
        let said = told.outcome().said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("GitHub"), "{said}");
        assert!(!seen.contains(&said.text().to_owned()), "two say {said}");
        seen.push(said.text().to_owned());
    }
}

/// **The login is shown beneath the clause, and what gh printed is why** —
/// each where the entry has one, and nothing where it has none.
#[test]
fn the_login_and_what_gh_printed_are_read_back_beside_it() {
    let signed_in = Told::of(&Entry::at_github(
        AtGitHub::signed_in("anna").unwrap(),
        noon(),
    ));
    assert!(
        signed_in
            .github_login()
            .is_some_and(|login| login.is("anna"))
    );
    assert_eq!(signed_in.because(), None);
    assert_eq!(signed_in.sentence(), None);

    let signed_out_of_somebody = Told::of(&Entry::at_github(
        AtGitHub::signed_out(None).unwrap(),
        noon(),
    ));
    assert_eq!(signed_out_of_somebody.github_login(), None);

    let failed = Told::of(&Entry::at_github(
        AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some("HTTP 502"))),
        noon(),
    ));
    assert!(failed.because().is_some_and(|why| why.is("HTTP 502")));
    assert_eq!(failed.github_login(), None);

    let cancelled = Told::of(&Entry::at_github(
        AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
        noon(),
    ));
    assert_eq!(cancelled.because(), None);
}
