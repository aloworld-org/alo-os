//! Whose site a publication goes to, asked of `gh` and never of a model —
//! each row of the table in this module's header, and how `gh`'s answers are
//! read.

use super::*;
use crate::verbs::publishing_verbs;

/// What a model sends: the folder, and an account if it sends one.
fn sent(account: Option<&str>) -> Vec<(&'static str, Given)> {
    let mut given = vec![(words::FOLDER, Given::text("/home/anna/site"))];
    if let Some(account) = account {
        given.push((words::ACCOUNT, Given::text(account)));
    }
    given
}

/// The call the offered declaration makes of what a model sent, once the
/// account is set aside — what the turn has before it asks `gh`.
fn offered(given: &[(&str, Given)]) -> Call {
    publishing_verbs()
        .unwrap()
        .call(
            PUBLISH_TO_GITHUB_PAGES,
            &what_the_model_must_give(PUBLISH_TO_GITHUB_PAGES, given),
        )
        .unwrap()
}

/// `anna`, as `gh` would say she is signed in.
fn anna() -> Result<Account, NotPublished> {
    Ok(Account::named("anna").unwrap())
}

/// The account a call names.
fn named(call: &Call) -> Option<&str> {
    match call.value(words::ACCOUNT) {
        Some(Value::Name(name)) => Some(name.as_str()),
        _ => None,
    }
}

/// **No account given: the one `gh` is signed in as**, and the sentence the
/// person approves names its address.
#[test]
fn no_account_given_is_the_account_gh_is_signed_in_as() {
    let given = sent(None);
    let call =
        the_call_naming(&offered(&given), the_account_the_model_gave(&given), anna()).unwrap();
    assert_eq!(named(&call), Some("anna"));
    assert!(call.waits_for_approval());
    let strings = alo_strings::Strings::of(crate::words::publishing_words().unwrap());
    assert!(
        call.sentence(&strings).text().contains("anna.github.io"),
        "{}",
        call.sentence(&strings).text()
    );
}

/// **Given, and the same account**, as GitHub reads names — whatever case it
/// was written in.
#[test]
fn an_account_given_that_is_the_signed_in_one_is_published_to() {
    for asked in ["anna", "Anna", " ANNA "] {
        let given = sent(Some(asked));
        let call =
            the_call_naming(&offered(&given), the_account_the_model_gave(&given), anna()).unwrap();
        assert_eq!(named(&call), Some("anna"), "{asked:?}");
    }
}

/// **The refusal path: an account given that is somebody else is refused**,
/// naming both — because the sentence would name a site that is not theirs.
#[test]
fn an_account_given_that_is_somebody_else_is_refused() {
    let given = sent(Some("bob"));
    assert_eq!(
        the_call_naming(&offered(&given), the_account_the_model_gave(&given), anna()),
        Err(NotPublished::SignedInAsAnother {
            signed_in: "anna".to_owned(),
            account: "bob".to_owned(),
        })
    );
    let given = sent(Some("$(reboot)"));
    assert_eq!(
        the_call_naming(&offered(&given), the_account_the_model_gave(&given), anna()),
        Err(NotPublished::NotAnAccount("$(reboot)".to_owned()))
    );
}

/// **The refusal path: nobody signed in is refused as that**, whatever the
/// model gave — which is the refusal that raises the sign-in sheet.
#[test]
fn nobody_signed_in_is_refused_as_nobody_signed_in() {
    for given in [sent(None), sent(Some("anna"))] {
        assert_eq!(
            the_call_naming(
                &offered(&given),
                the_account_the_model_gave(&given),
                Err(NotPublished::NotSignedIn)
            ),
            Err(NotPublished::NotSignedIn)
        );
    }
}

/// **Only a publication's account is set aside**; every other verb's
/// arguments pass untouched.
#[test]
fn only_a_publications_account_is_set_aside() {
    let given = sent(Some("anna"));
    assert_eq!(
        what_the_model_must_give(PUBLISH_TO_GITHUB_PAGES, &given).len(),
        1
    );
    assert_eq!(what_the_model_must_give("rename_file", &given), given);
}

/// What the line asking who ended with, read back.
fn read(status: Option<i32>, said: &str) -> Result<Account, NotPublished> {
    who_it_said(&WhatACommandDid::of(status, said.to_owned(), false, false))
}

/// **What `gh` says is read back as an account, or as why there is none.**
#[test]
fn what_gh_says_is_read_as_an_account_or_why_not() {
    assert_eq!(
        read(Some(0), "SsebowaDisan\n"),
        Ok(Account::named("ssebowadisan").unwrap())
    );
    assert_eq!(read(Some(71), ""), Err(NotPublished::NoGh));
    assert_eq!(
        read(
            Some(72),
            "To get started with GitHub CLI, please run:  gh auth login\n"
        ),
        Err(NotPublished::NotSignedIn)
    );
    assert!(matches!(
        read(Some(72), "error connecting to api.github.com\n"),
        Err(NotPublished::GitHubDidNotAnswer { .. })
    ));
    assert!(matches!(
        read(Some(0), "not a name; reboot\n"),
        Err(NotPublished::GitHubDidNotAnswer { .. })
    ));
    assert_eq!(read(None, ""), Err(NotPublished::WasStopped));
}

/// **The line asks `gh`'s own configuration first and GitHub only where that
/// is empty**, and names nothing a model wrote.
#[test]
fn the_line_asks_the_configuration_first_and_github_only_after() {
    let line = the_line_asking_who(&Through::on_github_com()).unwrap();
    let text = line.text();
    let local = text.find("config get -h github.com user").unwrap();
    let network = text.find("api user --jq .login").unwrap();
    assert!(local < network, "{text}");
    assert!(!text.contains("auth login"), "{text}");
}
