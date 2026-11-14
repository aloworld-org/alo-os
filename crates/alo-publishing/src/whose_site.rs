//! **Whose site a publication goes to: the account `gh` is signed in as** —
//! asked of `gh` itself, never supplied by a model.
//!
//! On the walk of 2026-11-13 a person asked *publish the folder alo-test in my
//! home folder to my github pages*, the model supplied an account name of its
//! own invention, and the line rightly refused it as somebody else's site.
//! Only when the person typed their account's name did it publish. The person
//! had already said who they are, by signing in to `gh`; asking them again,
//! or letting a model guess, is the machine forgetting what it was told.
//!
//! So the verb a model is offered names **the folder only**
//! ([`crate::verbs`]), and the account comes from here, before the person is
//! asked — so the sentence they approve still names the address it goes to:
//!
//! | asked of the model | `gh` says | the person is asked about |
//! |---|---|---|
//! | nothing | `anna` | `anna.github.io` |
//! | `Anna` | `anna` | `anna.github.io` — the same account, as GitHub reads names |
//! | `bob` | `anna` | nothing: refused, [`NotPublished::SignedInAsAnother`] |
//! | anything | nobody | nothing: [`NotPublished::NotSignedIn`], which raises the sign-in sheet |
//!
//! A model that still sends `account`, as the verb asked until this change, is
//! answered as before: the contract moved additively.
//!
//! # What it runs
//!
//! One line, written here and never by a model: `gh config get -h github.com
//! user`, which reads the account `gh auth login` wrote into `gh`'s own
//! configuration and sends nothing; and only where that answers nothing, `gh
//! api user --jq .login`, which asks GitHub. The second reaches the network,
//! and the turn runs the line counted on the indicator as every command alo
//! runs is, so it is never silent. **And the publication's own line asks GitHub
//! again at the moment it acts** ([`crate::the_line`], step 73), so an account
//! read here that GitHub no longer agrees with publishes nothing.

use alo_capability::{
    ACommand, AroundACommand, Call, CallError, Given, ItDidNotWork, Value, WhatACommandDid,
};

use crate::account::Account;
use crate::not_published::NotPublished;
use crate::runs::{Runs, Through};
use crate::the_line::quoted;
use crate::verbs::{PUBLISH_TO_GITHUB_PAGES, the_question};
use crate::what_came_of_it::its_last_lines;
use crate::words;

/// The line that asks `gh` who is signed in to github.com, through `through`.
///
/// Ends with 71 where there is no `gh`, and 72 where neither `gh`'s own
/// configuration nor GitHub names anybody; otherwise prints the account.
///
/// # Errors
/// [`NotPublished::TheNameCannotBeCarried`], which a `gh` named on one line
/// cannot cause.
pub fn the_line_asking_who(through: &Through) -> Result<ACommand, NotPublished> {
    let gh = quoted(through.gh());
    let line = format!(
        "command -v {gh} >/dev/null 2>&1 || exit 71; \
         l=$({gh} config get -h github.com user 2>/dev/null); \
         [ -n \"$l\" ] || l=$({gh} api user --jq .login) || exit 72; \
         printf '%s\\n' \"$l\""
    );
    ACommand::of(&line).map_err(|_| NotPublished::TheNameCannotBeCarried)
}

/// **Who is signed in to GitHub**, asked of `gh` through `through`, run by
/// `runs` and started and watched by `around`.
///
/// # Errors
/// [`NotPublished::NoGh`], [`NotPublished::NotSignedIn`], and
/// [`NotPublished::GitHubDidNotAnswer`] for a network that is down or an
/// answer that is not an account's name.
pub fn who_is_signed_in(
    through: &Through,
    runs: &dyn Runs,
    around: &mut dyn AroundACommand,
) -> Result<Account, NotPublished> {
    let line = the_line_asking_who(through)?;
    let did = runs.run(&line, around).map_err(NotPublished::DidNotStart)?;
    who_it_said(&did)
}

/// What the line asking who said, read back.
///
/// # Errors
/// As [`who_is_signed_in`].
pub fn who_it_said(did: &WhatACommandDid) -> Result<Account, NotPublished> {
    let said = did.said();
    match ItDidNotWork::of(did.returned(), did.took_too_long()) {
        None => said
            .lines()
            .map(str::trim)
            .rfind(|line| !line.is_empty())
            .and_then(Account::named)
            .ok_or_else(|| NotPublished::GitHubDidNotAnswer {
                said: its_last_lines(said),
            }),
        Some(ItDidNotWork::EndedWith(71)) => Err(NotPublished::NoGh),
        Some(ItDidNotWork::EndedWith(72))
            if alo_capability::not_signed_in_to_github::not_signed_in_to_github(said) =>
        {
            Err(NotPublished::NotSignedIn)
        }
        // **Neither place named anybody, and GitHub said nothing a sign-in
        // would mend** — the configuration was empty and the network is down,
        // or GitHub answered with something else. Told as GitHub not
        // answering, with what it said, rather than as a sign-in that may not
        // be missing.
        Some(ItDidNotWork::EndedWith(72)) => Err(NotPublished::GitHubDidNotAnswer {
            said: its_last_lines(said),
        }),
        Some(ItDidNotWork::TookTooLong) => Err(NotPublished::TookTooLong),
        Some(ItDidNotWork::WasStopped) => Err(NotPublished::WasStopped),
        Some(ItDidNotWork::NotSignedInToGitHub) => Err(NotPublished::NotSignedIn),
        Some(ItDidNotWork::EndedWith(status)) => Err(NotPublished::EndedWith {
            status,
            said: its_last_lines(said),
        }),
    }
}

/// **What a model must give, for `verb`**: `given` as it arrived, except that
/// a publication's `account` is set aside — it is the machine's to name, and
/// [`the_call_naming`] compares it with `gh`'s.
#[must_use]
pub fn what_the_model_must_give<'g>(
    verb: &str,
    given: &[(&'g str, Given)],
) -> Vec<(&'g str, Given)> {
    given
        .iter()
        .filter(|(name, _)| verb.trim() != PUBLISH_TO_GITHUB_PAGES || name.trim() != words::ACCOUNT)
        .cloned()
        .collect()
}

/// The `account` a model gave alongside a publication, if it gave one.
#[must_use]
pub fn the_account_the_model_gave<'g>(given: &'g [(&str, Given)]) -> Option<&'g Given> {
    given
        .iter()
        .find(|(name, _)| name.trim() == words::ACCOUNT)
        .map(|(_, value)| value)
}

/// **The call a person is asked about**: `offered`, the publication a model
/// called with the folder alone, made into one naming the account `gh` said
/// is `signed_in` — so the sentence names the address.
///
/// `asked` is the account the model gave, if it gave one. It is never used as
/// the account: it is only compared with `signed_in`, and a different one is
/// refused, because the sentence would otherwise name a site that is not the
/// person's.
///
/// # Errors
/// What `signed_in` was, where it was an error; [`NotPublished::NotAnAccount`]
/// for an `asked` GitHub would not give; [`NotPublished::SignedInAsAnother`]
/// for an `asked` that is somebody else; and [`NotPublished::NotHere`] for an
/// `offered` that is not a publication of a folder.
pub fn the_call_naming(
    offered: &Call,
    asked: Option<&Given>,
    signed_in: Result<Account, NotPublished>,
) -> Result<Call, NotPublished> {
    let Some(Value::Path(folder)) = offered.value(words::FOLDER) else {
        return Err(NotPublished::NotHere(format!(
            "{} is not {PUBLISH_TO_GITHUB_PAGES} with a {}, so it names no account",
            offered.verb(),
            words::FOLDER
        )));
    };
    let signed_in = signed_in?;
    if let Some(asked) = asked {
        let text = match asked {
            Given::Text(text) => text.clone(),
            Given::Number(number) => number.to_string(),
        };
        let account =
            Account::named(text.trim()).ok_or_else(|| NotPublished::NotAnAccount(text.clone()))?;
        if account != signed_in {
            return Err(NotPublished::SignedInAsAnother {
                signed_in: signed_in.name().to_owned(),
                account: account.name().to_owned(),
            });
        }
    }
    let question = the_question().map_err(|why| NotPublished::NotHere(format!("{why:?}")))?;
    Call::of(
        &question,
        &[
            (words::FOLDER, Given::text(folder.display().to_string())),
            (words::ACCOUNT, Given::text(signed_in.name())),
        ],
    )
    .map_err(|why: CallError| NotPublished::NotHere(format!("{why:?}")))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "whose_site_tests.rs"]
mod tests;
