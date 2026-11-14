//! **A change, made into the call a person is asked about** — which for every
//! verb but one is the call a model made, and for a publication is that call
//! naming the GitHub account `gh` is signed in as.
//!
//! On the walk of 2026-11-13 a model asked to publish a folder *to my github
//! pages* supplied an account of its own invention, and the publication was
//! refused as somebody else's site; it published only once the person typed
//! their account's name. They had already said who they are, by signing in to
//! `gh`. So a model is offered the folder alone, and the account is asked of
//! `gh` here — before the person is asked, so the sentence they approve still
//! names the address (`alo_publishing::whose_site`).
//!
//! # Only on the roads that ask or act
//!
//! [`Turning::changing`] and [`Turning::proposing`] come through here. A
//! change refused before anybody is asked — at the sealed box — is written
//! down from the call the model made, through `Turning::calling`, and asks
//! `gh` nothing: a level that cannot run programs does not run one to find
//! out whose site it would not have published to.
//!
//! # What the asking sends
//!
//! The line reads `gh`'s own record of who signed in, which sends nothing, and
//! asks GitHub only where that record is empty. It runs counted on the
//! indicator as every command alo runs is (`crate::while_alos_command_runs`),
//! so what it sends is seen at the moment it leaves; a refusal keeps how many
//! bytes on its entry.

use std::time::SystemTime;

use alo_capability::{Call, Given, Grants};
use alo_publishing::NotPublished;
use alo_record::Entry;

use crate::refusing::NotDone;
use crate::turning::Turning;
use crate::while_alos_command_runs::WhileItRuns;

/// What the indicator names while the line asking who is signed in runs.
const WHAT_ASKS: &str = "gh";

impl Turning<'_, '_> {
    /// **The call a change becomes before anybody is asked about it.**
    ///
    /// A publication's is the one naming the account `gh` is signed in as,
    /// compared with any account the model gave; every other change's is the
    /// call `Turning::calling` makes.
    ///
    /// # Errors
    /// What `Turning::calling` answers, and — for a publication —
    /// [`NotDone::CouldNotPublish`] when nobody is signed in to GitHub (which
    /// raises the sign-in sheet), when the model named somebody else, or when
    /// `gh` could not be asked; each written down before it is answered.
    pub(crate) fn calling_a_change(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Call, NotDone> {
        // An application named by the name it shows is put in by its
        // identifier first (`crate::naming_an_application`).
        let offered = self.calling_by_name(verb, given, grants, now)?;
        if offered.verb() != alo_publishing::PUBLISH_TO_GITHUB_PAGES {
            return Ok(offered);
        }
        let (signed_in, sent) = match self.machine().publishing_and_counting() {
            Some((publishing, bounding, indicator)) => {
                let mut around =
                    WhileItRuns::begun(bounding.a_command_counted(), indicator, WHAT_ASKS, now);
                let who = alo_publishing::who_is_signed_in(
                    &publishing.through,
                    publishing.runs,
                    &mut around,
                );
                (who, around.ended())
            }
            None => (
                Err(NotPublished::NotHere(
                    "this machine was built without anything to publish with".into(),
                )),
                None,
            ),
        };
        match alo_publishing::the_call_naming(
            &offered,
            alo_publishing::the_account_the_model_gave(given),
            signed_in,
        ) {
            Ok(call) => Ok(call),
            Err(why) => {
                let strings = self.machine().strings();
                let entry = Entry::never_asked(
                    &offered,
                    self.grantee(),
                    why.said(strings).text(),
                    strings,
                    now,
                )
                .having_sent(sent);
                self.writing_down(entry)?;
                Err(NotDone::CouldNotPublish(why))
            }
        }
    }
}
