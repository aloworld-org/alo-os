//! The only type that means *this may reach an application* — and the
//! application it may reach is one that is really here.
//!
//! [`alo_capability::Authorised`] is the end of the journey ADR 0001 §5
//! describes: validated, permitted, approved, and checked against the grants
//! once more at the moment it would run. A [`Reaching`] is an `Authorised` whose
//! applications have been found on this machine. It has one constructor, it
//! takes the authorisation **by value**, and it is not `Clone` — so whatever
//! drives the compositor holds one of these or it holds nothing.
//!
//! It is [`alo_files::Touching`](https://docs.rs/alo-files)'s shape, met on the
//! other half of the verb list, and the resemblance is the point: a guarantee
//! carried by a type is one that stays true when somebody who has not read this
//! file writes the next verb.
//!
//! # The two questions, in this order
//!
//! For every application a call names:
//!
//! 1. **do the grants permit it?** Exactly as [`alo_capability`] decided it. If
//!    not, that is the refusal, and **the list of what is installed is never
//!    consulted**;
//! 2. **is it here?** [`crate::Installed`], which is the only thing this crate
//!    knows that the deciding crate does not.
//!
//! **The order is the security property**, and it is the file half's order for
//! the same reason one level along. A refusal that answered *that is not
//! installed* about an application nobody granted would tell an agent what is
//! on somebody's machine — which applications a person uses is a fingerprint of
//! who they are, what they do for a living and who they work for, and a
//! capability model that hands it over in a refusal has a side channel in it.
//! Asked in this order, an ungranted application refuses identically whether it
//! is installed or not, and there is a test that asserts exactly that.
//!
//! Every application the call names goes through both — not only the ones the
//! verb declared its grant is over. A verb that forgot to require a grant over
//! one of its arguments is a mistake somebody will make one day, and it should
//! not be one that reaches a window.
//!
//! # What it does not do
//!
//! It opens nothing, focuses nothing and closes nothing: that is the acting
//! half, it is Wayland and D-Bus, and it is not this crate's. Nor can it close
//! the gap between an application being found and being reached — one
//! uninstalled in between is gone by the time the compositor is asked, and the
//! answer to that is the compositor's own *there is no such application*
//! rather than a second look at this list.

use std::collections::BTreeMap;

use alo_capability::{Ask, Authorised, Call, Grants, Refused, Value};
use alo_strings::Strings;

use crate::application::Application;
use crate::installed::Installed;
use crate::refusing::NotInstalled;

/// An application call that may be carried out, and the applications it names.
///
/// Deliberately not `Clone`, like the [`Authorised`] inside it: a thing that
/// means may-run and can be copied is a thing that can be run twice.
#[derive(Debug)]
pub struct Reaching {
    /// What may run, and the authority it runs under.
    authorised: Authorised,
    /// Each application this call names, by the argument that named it.
    applications: BTreeMap<String, Application>,
}

impl Reaching {
    /// Ask the grants about every application this call names, and then find
    /// each one on this machine.
    ///
    /// The moment is the authorisation's own ([`Authorised::at`]) rather than a
    /// fresh one: this is the last part of the same question the grants were
    /// asked when the call was authorised, and two moments would be two answers
    /// that could disagree.
    ///
    /// # Errors
    /// [`Refused`], carrying the call — the grants' own words when an
    /// application is not granted, and this crate's when it is not installed.
    pub fn of(
        authorised: Authorised,
        grants: &Grants,
        installed: &Installed,
        strings: &Strings,
    ) -> Result<Self, Refused> {
        let at = authorised.at();
        let under = authorised.under().clone();
        let mut applications = BTreeMap::new();
        for (argument, value) in authorised.call().values() {
            let Value::Application(identifier) = value else {
                continue;
            };
            // 1. Granted? Refused here, and nothing has been looked for.
            if let Err(why) = grants.permitting(&under, &Ask::Application(identifier.clone()), at) {
                return Err(Refused::not_granted(authorised.call().clone(), why));
            }
            // 2. And is it here at all?
            let Some(application) = installed.knows(identifier) else {
                return Err(Refused::worded_elsewhere(
                    authorised.call().clone(),
                    NotInstalled::wanting(identifier).said(strings),
                ));
            };
            applications.insert(argument.clone(), application.clone());
        }
        Ok(Self {
            authorised,
            applications,
        })
    }

    /// What may run, and the authority it runs under.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// What may run.
    #[must_use]
    pub fn call(&self) -> &Call {
        self.authorised.call()
    }

    /// The verb that may run.
    #[must_use]
    pub fn verb(&self) -> &str {
        self.authorised.verb()
    }

    /// The application this argument named, as this machine has it.
    ///
    /// `None` for an argument that is not an application, and for one this verb
    /// does not take. Whatever drives the compositor drives **this**, and never
    /// the identifier the call arrived with — that identifier was checked, and
    /// this is the entry the check was about.
    #[must_use]
    pub fn application(&self, argument: &str) -> Option<&Application> {
        self.applications.get(argument)
    }

    /// Every application this call may reach, by the argument that named it.
    pub fn all(&self) -> impl Iterator<Item = (&str, &Application)> {
        self.applications
            .iter()
            .map(|(argument, application)| (argument.as_str(), application))
    }

    /// Give back what ran, so it can be recorded.
    ///
    /// `alo-record` writes an entry from an [`Authorised`], and this is how the
    /// executor hands one over once the work is done. It consumes the token,
    /// because a thing that has run is not a thing that may run.
    #[must_use]
    pub fn into_authorised(self) -> Authorised {
        self.authorised
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{
        agent, approving, blender, granting, hour, in_english, installing, noon, opening, refusal,
    };
    use alo_capability::{
        Arg, Effect, Given, Grant, NotAuthorised, Proposal, ProposalError, Reach, Requires, Takes,
        Verb, Verbs,
    };
    use alo_strings::Word;

    /// The ordinary way through: granted, installed, approved, and what comes
    /// back is the entry this machine holds rather than the text that arrived.
    #[test]
    fn what_may_be_reached_is_the_application_this_machine_has() {
        let call = opening("org.blender.Blender");
        let grants = granting(&["org.blender.Blender"]);
        let authorised = approving(&call, &grants);

        let reaching = Reaching::of(
            authorised,
            &grants,
            &installing(&[blender()]),
            &in_english(),
        )
        .unwrap();
        assert_eq!(reaching.verb(), "open_application");
        assert_eq!(
            reaching.application("application").map(Application::name),
            Some(Some("Blender"))
        );
        assert_eq!(reaching.all().count(), 1);
        assert!(reaching.application("folder").is_none());
        assert_eq!(reaching.call(), &call);
        assert!(reaching.into_authorised().from_approval().is_some());
    }

    /// **The reason the order is what it is.** An agent whose grant has gone is
    /// told the same thing whether or not the application is on this machine —
    /// so a refusal cannot be used to enumerate what somebody has installed.
    ///
    /// A revoked grant is how a call reaches here ungranted at all: the grants
    /// are asked when the call is made, when it is proposed and again at the
    /// moment it would run, so anything else has already been refused earlier.
    #[test]
    fn an_agent_learns_nothing_about_this_machine_from_a_refusal() {
        let call = opening("org.blender.Blender");
        let strings = in_english();

        let refusing = |installed: Installed| {
            let mut grants = granting(&["org.blender.Blender"]);
            let authorised = approving(&call, &grants);
            grants.revoke_everything_for(&agent());
            Reaching::of(authorised, &grants, &installed, &strings).unwrap_err()
        };

        let here = refusing(installing(&[blender()]));
        let not_here = refusing(Installed::nothing());

        assert_eq!(
            refusal(&here),
            refusal(&not_here),
            "the refusal says whether the application is installed"
        );
        assert!(
            refusal(&here).contains("has not been granted"),
            "{}",
            refusal(&here)
        );
        assert!(!refusal(&here).contains("installed on this machine"));
        // And it is the grants' own refusal, so it reaches the record by the
        // road every other refusal takes.
        assert!(matches!(here.why(), NotAuthorised::NotGranted(_)));
        assert_eq!(here.call(), &call);
    }

    /// An application that was granted and is not here is refused rather than
    /// silently doing nothing — there is nothing to act on, so there is nothing
    /// to permit. It is this crate's words, carried into the record as said.
    #[test]
    fn an_application_that_is_not_installed_is_refused_in_this_crates_words() {
        let call = opening("org.blender.Blender");
        let grants = granting(&["org.blender.Blender"]);
        let refused = Reaching::of(
            approving(&call, &grants),
            &grants,
            &Installed::nothing(),
            &in_english(),
        )
        .unwrap_err();
        assert!(
            refusal(&refused).contains("nothing installed on this machine"),
            "{}",
            refusal(&refused)
        );
        assert!(refusal(&refused).contains("org.blender.Blender"));
        assert!(matches!(
            refused.why(),
            NotAuthorised::NotGrantedElsewhere(_)
        ));
        assert_eq!(refused.call(), &call);
    }

    /// The grants are asked again here, so a grant taken away between the
    /// approval and the window being reached still stops it.
    #[test]
    fn a_grant_taken_away_after_the_approval_still_stops_it() {
        let call = opening("org.blender.Blender");
        let mut grants = granting(&["org.blender.Blender"]);
        let authorised = approving(&call, &grants);
        assert_eq!(grants.revoke_everything_for(&agent()), 1);

        let refused = Reaching::of(
            authorised,
            &grants,
            &installing(&[blender()]),
            &in_english(),
        )
        .unwrap_err();
        assert!(
            refusal(&refused).contains("has not been granted"),
            "{}",
            refusal(&refused)
        );
    }

    /// An expired grant permits nothing here either, and the moment asked about
    /// is the authorisation's own rather than a second reading of a clock.
    #[test]
    fn the_moment_asked_about_is_the_one_the_call_was_authorised_at() {
        let call = opening("org.blender.Blender");
        let grants = granting(&["org.blender.Blender"]);
        let authorised = approving(&call, &grants);
        assert_eq!(authorised.at(), noon());

        let reaching = Reaching::of(
            authorised,
            &grants,
            &installing(&[blender()]),
            &in_english(),
        )
        .unwrap();
        assert_eq!(reaching.authorised().at(), noon());
        assert_eq!(reaching.authorised().against().len(), 1);

        // An hour later the grant has run out, so nothing gets as far as being
        // proposed and there is no authorisation to reach anything with.
        assert!(matches!(
            Proposal::checked(&call, &agent(), &grants, noon() + hour(), hour()),
            Err(ProposalError::NotGranted(_))
        ));
    }

    /// **Every application a call names is asked about**, not only the ones the
    /// verb declared its grant is over. The verb here requires a grant over one
    /// of its two, which the contract permits and the four do not do.
    #[test]
    fn an_application_the_verb_forgot_to_require_a_grant_over_is_still_asked_about() {
        let forgetful = Verb::checked(
            "swap_applications",
            Word::saying(
                "testing.swap.purpose",
                "put one application where another is",
            ),
            Effect::Change,
            vec![
                Arg::taking(
                    "application",
                    Word::saying("testing.swap.application", "the one to bring forward"),
                    Takes::Application,
                ),
                Arg::taking(
                    "behind",
                    Word::saying("testing.swap.behind", "the one to put behind it"),
                    Takes::Application,
                ),
            ],
            Requires::grants_over(["application"]),
            Word::saying(
                "testing.swap.sentence",
                "put {application} in front of {behind}",
            ),
        )
        .unwrap();
        let mut verbs = Verbs::default();
        verbs.declare(forgetful).unwrap();
        let call = verbs
            .call(
                "swap_applications",
                &[
                    ("application", Given::text("org.blender.Blender")),
                    ("behind", Given::text("com.example.Payroll")),
                ],
            )
            .unwrap();

        // The deciding crate permits it: the verb only asked about one of them.
        let grants = granting(&["org.blender.Blender"]);
        let authorised = approving(&call, &grants);

        let refused = Reaching::of(
            authorised,
            &grants,
            &installing(&[
                blender(),
                Application::called("com.example.Payroll", "Payroll").unwrap(),
            ]),
            &in_english(),
        )
        .unwrap_err();
        assert!(
            refusal(&refused).contains("com.example.Payroll"),
            "{}",
            refusal(&refused)
        );
        assert!(refusal(&refused).contains("has not been granted"));
    }

    /// A verb that names no application at all reaches nothing, and says so —
    /// an empty answer rather than a refusal, because reaching nothing is a
    /// perfectly good thing to be allowed to do.
    #[test]
    fn a_call_that_names_no_application_reaches_nothing() {
        let verb = Verb::checked(
            "list_displays",
            Word::saying(
                "testing.displays.purpose",
                "list the displays on this machine",
            ),
            Effect::Read,
            vec![],
            Requires::nothing_because(
                "a display is not a path, a file or an application, and naming one reaches nothing",
            ),
            Word::saying("testing.displays.sentence", "list the displays"),
        )
        .unwrap();
        let mut verbs = Verbs::default();
        verbs.declare(verb).unwrap();
        let call = verbs.call("list_displays", &[]).unwrap();
        let grants = Grants::default();
        let authorised = Authorised::read(&call, &agent(), &grants, noon()).unwrap();

        let reaching =
            Reaching::of(authorised, &grants, &Installed::nothing(), &in_english()).unwrap();
        assert_eq!(reaching.all().count(), 0);
        assert!(reaching.application("application").is_none());
    }

    /// A grant over a folder is not a grant over an application that happens to
    /// live in it — `alo_capability::Reach`'s rule met from this side. It never
    /// reaches this file, because a call nothing permits is refused before it
    /// can be put to anybody.
    #[test]
    fn a_grant_over_a_folder_reaches_no_application() {
        let call = opening("org.blender.Blender");
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "@applications",
                Reach::Folder(std::path::PathBuf::from("/usr/share/applications")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let refused = Proposal::checked(&call, &agent(), &grants, noon(), hour()).unwrap_err();
        let said = refused.said(&in_english());
        assert!(said.text().contains("has not been granted"), "{said}");
        assert!(said.text().contains("org.blender.Blender"), "{said}");
    }
}
