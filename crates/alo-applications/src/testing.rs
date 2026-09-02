//! The grants, the machine and the strings this crate's tests are written
//! against.
//!
//! Every file here has the same three things to build before it can say
//! anything — an agent with a grant, a machine with something installed on it,
//! and a vocabulary — and building them from one fixture is what stops five
//! files inventing five machines that resemble each other. The vocabulary is
//! [`crate::application_words`] beside `alo-capability`'s, which is the
//! arrangement a shell has: one vocabulary, one area per crate.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::time::{Duration, SystemTime};

use alo_capability::{
    Approvals, Authorised, Call, Given, Grant, Grantee, Grants, Proposal, Reach, Refused,
};
use alo_strings::{Language, Strings, Translation, Vocabulary, Word};

use crate::application::Application;
use crate::installed::Installed;
use crate::verbs::application_verbs;
use crate::words::application_words;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants and the questions in these tests last.
pub(crate) fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// The agent the tests grant things to.
pub(crate) fn agent() -> Grantee {
    Grantee::named("@applications")
}

/// Grants to that agent over these applications, made at noon, lasting an hour.
pub(crate) fn granting(applications: &[&str]) -> Grants {
    let mut grants = Grants::default();
    for application in applications {
        grants.grant(
            Grant::checked(
                "@applications",
                Reach::Application((*application).to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// One application most of these tests are about.
pub(crate) fn blender() -> Application {
    Application::called("org.blender.Blender", "Blender").unwrap()
}

/// A machine with these applications on it.
pub(crate) fn installing(applications: &[Application]) -> Installed {
    Installed::holding(applications.iter().cloned())
}

/// A call to start an application.
pub(crate) fn opening(application: &str) -> Call {
    application_verbs()
        .unwrap()
        .call(
            "open_application",
            &[("application", Given::text(application))],
        )
        .unwrap()
}

/// The whole of ADR 0001 §5 for a change: proposed, approved once, and redeemed
/// at the moment it would run.
///
/// Every verb in this crate is a change, so there is no read door to take and
/// no shorter way to an [`Authorised`] than this one.
pub(crate) fn approving(call: &Call, grants: &Grants) -> Authorised {
    let mut approvals = Approvals::default();
    let id = approvals.propose(Proposal::checked(call, &agent(), grants, noon(), hour()).unwrap());
    approvals
        .approve(id, noon())
        .unwrap()
        .redeem(grants, noon())
        .unwrap()
}

/// This crate's words beside the capability model's, with nothing translated:
/// what a machine that has no translations shows.
///
/// Both lists, because a refusal met here can have been worded by this crate —
/// *nothing installed on this machine is that* — or by the grants, which is
/// where *has not been granted* comes from.
pub(crate) fn in_english() -> Strings {
    Strings::of(everything())
}

/// The same, with these words translated into German and German preferred.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = everything();
    let mut into_german = Translation::into_language(german());
    for (word, says) in words {
        into_german = into_german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(into_german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// What a refusal says on a machine with no translations.
pub(crate) fn refusal(refused: &Refused) -> String {
    refused.said(&in_english()).into_text()
}

/// German, as `alo-strings` names a language.
pub(crate) fn german() -> Language {
    Language::written("de").unwrap()
}

/// This crate's vocabulary and the capability model's, in one.
fn everything() -> Vocabulary {
    let mut vocabulary = application_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    vocabulary
}
