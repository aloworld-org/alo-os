//! What a person's side of the machine says when what is granted has changed,
//! and what a running service does about it.
//!
//! `alo-remembering` gave the machine somewhere to keep its grants and
//! `src/main.rs` reads that file once, before the socket exists. What a service
//! could not do until this file existed was hear about a grant made **while it
//! is running**: somebody picks a folder at eleven, the file on the disk says
//! so, and the service holding the turn goes on serving under the list it read
//! at sign-in. The gap was narrow and honest — the daemon is bound to the
//! person's session, so the grant applied at the next sign-in — but
//! `docs/features.md` promises *pick a folder*, and *pick a folder and sign out
//! again* is not that promise.
//!
//! # A knock, never a payload
//!
//! `alo_protocol::FromAPerson::Granted` carries **nothing**: no grant, no path,
//! no reach, no duration, and nothing naming which grant was revoked. The
//! person's side says only *what is granted has changed*, and the service
//! answers by reading its own file again, under the same rules about who may
//! have written it that `alo-remembering` applies at start-up.
//!
//! That is what keeps law 2 and ADR 0001 §5 true of this door. Nothing on the
//! wire carries a grant, so a request that arrived from anywhere else could
//! still not widen anything — the most it could cause is a file it cannot write
//! being read a second time. And the kernel already says which door a caller is
//! on, so the knock is the person's: an agent sending it is refused in words
//! and the refusal is written down ([`an_agent_knocked`]).
//!
//! # What is read is put in place of what was held, whole
//!
//! [`WhatIsGranted::read_again`] replaces the list rather than merging into it,
//! because that is what the file means: `alo-remembering` writes what is granted,
//! so a grant that is not in it is a grant that was revoked. Anything less than a
//! whole replacement would be a service where revoking took effect at the next
//! sign-in and granting took effect at once, which is the asymmetry a person
//! would least expect and would be least able to see.
//!
//! The one thing carried across is **the grant this turn's own invocation
//! made** — the document the person had open, ADR 0001 §4. It is not in the
//! file and never will be: it is the turn's, it lasts as long as the turn, and
//! `alo_turn::Turning::ending` gives it back by the handle it was made under. A
//! replacement that dropped it would leave a turn holding a handle to nothing
//! and a document reachable by nobody; one that re-made it under a fresh handle
//! would leave the *old* handle unrevokable, which is a grant outliving the turn
//! that made it. So it is carried over **under the handle it already has**, and
//! a file that has since handed that handle out to something else is refused
//! rather than merged — the grants stay as they were, and the person is told.
//!
//! Nothing offers a document at an invocation today (`crate::serving` begins
//! every turn with `alo_context::Context::at_invocation`, which offers nothing,
//! because there is no compositor here yet), so that refusal is unreachable on
//! this machine. It is written because the compositor lane is what makes it
//! reachable, and a grant leaking out of a turn is not a thing to discover then.
//!
//! # A file that has stopped being believable is not an empty file
//!
//! If the grants cannot be read, the service **keeps the ones it had** and says
//! so, with the refusal in the record. A machine that forgot what was granted
//! because somebody chmodded a file is a machine that went silent, and the
//! person would find out by discovering their agent can no longer read their
//! invoices — which is exactly what law 1 is about.
//!
//! A file that is simply **not there** is not that machine. It is the ordinary
//! first morning, or a person who has revoked the last thing they granted, and
//! it reads as an empty list here exactly as it does at start-up.
//!
//! # This file cannot write the grants either
//!
//! [`Remembering`] has one method and it reads. `alo-remembering`'s writer is
//! not named anywhere in this crate, so *nothing an agent can send over the
//! socket writes a byte of the grants file* survives this request being added:
//! what the socket can now reach is a road to reading it, and there is still no
//! road to writing it.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_capability::{GrantId, Grants, Held};
use alo_protocol::{FromAPerson, ToAnAgent};
use alo_remembering::NotRemembered;
use alo_strings::{Filling, Said, Strings};
use alo_turn::Turning;

use crate::refusing::NotReadAgain;
use crate::words::{AN_AGENT_CANNOT_SAY_WHAT_IS_GRANTED, WHAT_IS_GRANTED_WAS_NOT_READ_AGAIN};

/// Somewhere this machine's grants can be read from again.
///
/// One method, and it reads. There is deliberately nothing here that writes:
/// see this module's header.
///
/// A trait for `alo_files::Resolving`'s reason — the service has to be testable
/// against a file that has become unbelievable, and against one that will not
/// read at all, without a test having to arrange either on the disk it is
/// running on. [`ThePersonsFile`] is the only implementation that ships.
pub trait Remembering: std::fmt::Debug {
    /// What is granted at this moment, believed and read.
    ///
    /// # Errors
    ///
    /// [`NotRemembered`], carried whole from `alo-remembering` rather than
    /// reworded: whoever reads it is whoever is standing the machine up, and
    /// that crate already names the file and what was wrong with it.
    fn read_again(&self, now: SystemTime) -> Result<Grants, NotRemembered>;
}

/// The file this machine really keeps its grants in.
///
/// Holds a path and nothing else, and the path is `src/main.rs`'s — the one
/// place in this service that names `alo_remembering::THE_GRANTS`.
#[derive(Debug, Clone)]
pub struct ThePersonsFile {
    /// Where the grants are.
    at: PathBuf,
}

impl ThePersonsFile {
    /// The grants at this path.
    #[must_use]
    pub fn at(at: &Path) -> Self {
        Self { at: at.to_owned() }
    }

    /// Where the grants are.
    ///
    /// For a service log and for a test that has to make the file unbelievable.
    /// Reading it does not open anything, and there is nothing here that writes.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.at
    }
}

impl Remembering for ThePersonsFile {
    /// `alo-remembering`'s own reading, with its three rules about who may have
    /// written the file and its dropping of what has expired.
    fn read_again(&self, now: SystemTime) -> Result<Grants, NotRemembered> {
        alo_remembering::remembered(&self.at, now)
    }
}

/// Whether this line is the person saying that what is granted has changed.
///
/// Asked of the door the request belongs to, rather than by looking at the text:
/// `alo-protocol` is what decides what a message is, and a second reader here
/// would be a second place that decision could be got wrong.
///
/// The one caller is [`an_agent_knocked`]. On the person's own door the request
/// arrives as a `FromAPerson` already, and nothing needs to ask.
#[must_use]
pub fn is_the_knock(line: &str) -> bool {
    matches!(FromAPerson::read(line), Ok(FromAPerson::Granted))
}

/// An agent sent the knock: refuse it in words, and write the refusal down.
///
/// [`None`] when the line was not the knock, which is every other way a message
/// can be one an agent may not send — those are `alo-protocol`'s refusals and
/// are answered without an entry, because a malformed message is noise and this
/// is an agent reaching for the person's own list.
///
/// Nothing is read. The refusal happens before the file is opened, so an agent
/// cannot even cause the reading it is not allowed to ask for.
///
/// **A refusal that could not be written down is not answered as a refusal**,
/// for `crate::doing::refused_by_a_rule`'s reason: somebody told only that their
/// agent was turned away would believe the machine had behaved correctly, when
/// it had also failed to keep the evidence that it had.
#[must_use]
pub fn an_agent_knocked(
    line: &str,
    turning: &mut Turning<'_, '_>,
    strings: &Strings,
    now: SystemTime,
) -> Option<ToAnAgent> {
    if !is_the_knock(line) {
        return None;
    }
    let said = strings.say(
        &AN_AGENT_CANNOT_SAY_WHAT_IS_GRANTED.key(),
        &Filling::nothing(),
    );
    Some(match turning.the_grants_were_not_read_again(&said, now) {
        Ok(()) => ToAnAgent::refused(&said),
        Err(why) => ToAnAgent::refused(&why.said(strings)),
    })
}

/// What is granted on this machine, and the one place it is read from.
///
/// The two travel together everywhere below `src/main.rs`, and they are one
/// value rather than two arguments because they are one fact: the list a turn is
/// answered against, and the file it came out of. Two arguments would be a
/// service that could be handed the list from one machine and the file from
/// another, and would be one more thing every reader of `crate::serving` has to
/// rule out.
///
/// **There is nothing here that writes.** [`Remembering`] has one method and it
/// reads, so what the socket can reach is a road to reading the grants file and
/// there is no road at all to writing it.
#[derive(Debug)]
pub struct WhatIsGranted<'a> {
    /// The list this service is serving under.
    holding: &'a mut Grants,
    /// Where it is read from again when the person says it has changed.
    file: &'a dyn Remembering,
}

impl<'a> WhatIsGranted<'a> {
    /// The list a service is holding, and the file it came from.
    pub fn of(holding: &'a mut Grants, file: &'a dyn Remembering) -> Self {
        Self { holding, file }
    }

    /// What is granted at this moment, for asking.
    #[must_use]
    pub fn holding(&self) -> &Grants {
        self.holding
    }

    /// The same, for the two things that really move it: a turn beginning and a
    /// turn ending, both of which lend and take back a grant of their own.
    pub fn holding_mut(&mut self) -> &mut Grants {
        self.holding
    }

    /// Read what is granted again, and serve under it from here.
    ///
    /// The list is replaced whole when the reading succeeds. `lent` is the
    /// handle this turn's own invocation made, when there is a turn and it made
    /// one. Answers how many grants are in force afterwards, which is what the
    /// person's shell is told.
    ///
    /// The clock is the caller's, as everywhere else in this workspace: the
    /// service reads one moment a round, so what the person is told and what the
    /// grants permit in that same round cannot disagree about when it was.
    ///
    /// # Errors
    ///
    /// [`NotReadAgain`], in English, for whoever is standing the machine up.
    /// **The grants are untouched in every one of them** — a refusal here leaves
    /// the service serving under exactly the list it had, which is the whole
    /// point of the refusal existing. What the person is told is one sentence,
    /// [`what_to_say`], because there is one thing for them to do about all of
    /// them.
    pub fn read_again(
        &mut self,
        lent: Option<GrantId>,
        now: SystemTime,
    ) -> Result<u64, NotReadAgain> {
        let fresh = match self.file.read_again(now) {
            Ok(fresh) => fresh,
            // Nothing granted is not a broken machine, here or at start-up.
            Err(NotRemembered::NotThere { .. }) => Grants::default(),
            Err(why) => return Err(NotReadAgain::NotBelievable(why)),
        };
        let fresh = with_what_this_turn_was_lent(fresh, self.holding, lent, now)?;
        let how_many = u64::try_from(fresh.active_at(now).count()).unwrap_or(u64::MAX);
        *self.holding = fresh;
        Ok(how_many)
    }
}

/// The list from the file, with this turn's own grant put back under the handle
/// it already has.
///
/// See this module's header for why it is carried across at all and why it
/// keeps its handle. `next` is moved past it as well, so a grant made after this
/// one cannot be handed a handle that is already in use.
fn with_what_this_turn_was_lent(
    fresh: Grants,
    holding: &Grants,
    lent: Option<GrantId>,
    now: SystemTime,
) -> Result<Grants, NotReadAgain> {
    let Some(lent) = lent else {
        return Ok(fresh);
    };
    let Some(held) = holding.active_at(now).find(|held| held.id == lent).cloned() else {
        // The turn holds a handle to a grant that has expired or was revoked
        // out from under it. Nothing to carry, and nothing wrong.
        return Ok(fresh);
    };
    let mut every: Vec<Held> = fresh.active_at(now).cloned().collect();
    let next = fresh
        .next_handle()
        .as_u64()
        .max(held.id.as_u64().saturating_add(1));
    every.push(held);
    Grants::remembered(every, next).map_err(|why| NotReadAgain::ATurnsOwnGrant(why.to_string()))
}

/// What the person is told when their grants were not read again.
///
/// **One sentence for every way it can fail**, because there is one thing for
/// them to do about all of them: nothing was widened, nothing was forgotten, and
/// what is in front of them is a file on their own machine that somebody has to
/// look at. Which failure it was is `NotReadAgain`'s English, for whoever does.
#[must_use]
pub fn what_to_say(strings: &Strings) -> Said {
    strings.say(
        &WHAT_IS_GRANTED_WAS_NOT_READ_AGAIN.key(),
        &Filling::nothing(),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_directory_of_our_own, a_folder_with_an_invoice, granting, noon};
    use alo_capability::{Ask, Grant, Grantee, Reach};
    use std::time::Duration;

    /// A grants file of this test's own, with this list in it.
    fn a_file_holding(what: &str, grants: &Grants) -> ThePersonsFile {
        let at = a_directory_of_our_own(what).join("grants.toml");
        alo_remembering::kept(&at, grants, noon()).unwrap();
        ThePersonsFile::at(&at)
    }

    /// Whether `@files` may reach this path at this moment.
    fn may_reach(grants: &Grants, at: &Path, now: SystemTime) -> bool {
        grants.permits(&Grantee::named("@files"), &Ask::path(at), now)
    }

    /// Read this file again into this list, as a running service does.
    fn read_again(
        file: &dyn Remembering,
        holding: &mut Grants,
        lent: Option<GrantId>,
        now: SystemTime,
    ) -> Result<u64, NotReadAgain> {
        WhatIsGranted::of(holding, file).read_again(lent, now)
    }

    /// **A grant made while the service is running is honoured after the
    /// knock**, and the service was refusing that same folder a moment before.
    #[test]
    fn a_grant_made_now_is_honoured_now() {
        let (folder, invoice) = a_folder_with_an_invoice("granted-now");
        let file = a_file_holding("granted-now-file", &granting(&folder, noon()));

        let mut holding = Grants::default();
        assert!(
            !may_reach(&holding, &invoice, noon()),
            "the service began with something granted"
        );

        assert_eq!(read_again(&file, &mut holding, None, noon()).unwrap(), 1);
        assert!(
            may_reach(&holding, &invoice, noon()),
            "the grant the person made did not reach the running service"
        );
    }

    /// **A revocation takes effect on the next question asked.** The file is
    /// what is granted, so a grant that is no longer in it is one that was
    /// revoked — and the answer is the grants saying no rather than an older
    /// list saying yes.
    #[test]
    fn a_revoked_grant_is_gone_after_the_knock() {
        let (folder, invoice) = a_folder_with_an_invoice("revoked-now");
        let mut holding = granting(&folder, noon());
        let file = a_file_holding("revoked-now-file", &Grants::default());

        assert!(may_reach(&holding, &invoice, noon()));
        assert_eq!(read_again(&file, &mut holding, None, noon()).unwrap(), 0);
        assert!(
            !may_reach(&holding, &invoice, noon()),
            "a revoked grant came back"
        );
    }

    /// **A file that is not there is an empty list rather than a refusal**, as
    /// it is at start-up: nobody has granted anything, or the last grant was
    /// revoked, and neither is a broken machine.
    #[test]
    fn a_machine_with_no_grants_file_reads_as_nothing_granted() {
        let (folder, invoice) = a_folder_with_an_invoice("no-file");
        let mut holding = granting(&folder, noon());
        let nowhere = ThePersonsFile::at(&a_directory_of_our_own("no-file-at-all").join("g.toml"));

        assert_eq!(read_again(&nowhere, &mut holding, None, noon()).unwrap(), 0);
        assert!(!may_reach(&holding, &invoice, noon()));
    }

    /// **A file that has stopped being believable leaves the grants where they
    /// were.** The refusal is the whole point: a machine that emptied its list
    /// because somebody chmodded a file is a machine that went silent.
    #[test]
    fn a_file_that_is_not_believable_leaves_the_grants_alone() {
        use std::os::unix::fs::PermissionsExt as _;

        let (folder, invoice) = a_folder_with_an_invoice("unbelievable");
        let mut holding = granting(&folder, noon());
        let file = a_file_holding("unbelievable-file", &granting(&folder, noon()));
        std::fs::set_permissions(file.path(), std::fs::Permissions::from_mode(0o666)).unwrap();

        let refused = read_again(&file, &mut holding, None, noon()).unwrap_err();
        assert!(
            matches!(refused, NotReadAgain::NotBelievable(_)),
            "{refused:?}"
        );
        assert!(
            may_reach(&holding, &invoice, noon()),
            "the grants were emptied by a file nobody could believe"
        );
    }

    /// **The grant a turn's own invocation made survives the replacement, under
    /// the handle it already has**, so the turn can still give it back.
    #[test]
    fn the_grant_this_turns_invocation_made_is_carried_across() {
        let (folder, invoice) = a_folder_with_an_invoice("lent");
        let document = folder.join("open.pdf");
        std::fs::write(&document, "what the person had open").unwrap();

        let file = a_file_holding("lent-file", &granting(&folder, noon()));
        // The list the service is holding is the one it read at sign-in, which
        // is what makes the turn's own handle come after the file's.
        let mut holding = file.read_again(noon()).unwrap();
        let lent = holding.grant(
            Grant::checked(
                "@files",
                Reach::File(document.clone()),
                noon(),
                Duration::from_secs(600),
            )
            .unwrap(),
        );

        assert_eq!(
            read_again(&file, &mut holding, Some(lent), noon()).unwrap(),
            2
        );
        assert!(
            may_reach(&holding, &document, noon()),
            "the turn's own grant was dropped by the replacement"
        );
        assert!(may_reach(&holding, &invoice, noon()));
        assert!(
            holding.revoke(lent),
            "the turn's grant is no longer under the handle the turn will end it by"
        );
    }

    /// **A list that would collide with the turn's own handle is refused, and
    /// the grants stay as they were.** Unreachable on this machine — nothing
    /// offers a document at an invocation yet — and written because the
    /// alternative is a grant that outlives the turn that made it.
    #[test]
    fn a_list_that_reuses_the_turns_handle_is_refused() {
        let (folder, invoice) = a_folder_with_an_invoice("collision");
        // The file's own handles start at zero, and so does this list — so the
        // grant the turn is holding is under handle zero as well.
        let mut holding = Grants::default();
        let lent = holding.grant(
            Grant::checked(
                "@files",
                Reach::File(folder.join("open.pdf")),
                noon(),
                Duration::from_secs(600),
            )
            .unwrap(),
        );
        let file = a_file_holding("collision-file", &granting(&folder, noon()));

        let refused = read_again(&file, &mut holding, Some(lent), noon()).unwrap_err();
        assert!(
            matches!(refused, NotReadAgain::ATurnsOwnGrant(_)),
            "{refused:?}"
        );
        assert!(
            !may_reach(&holding, &invoice, noon()),
            "the grants were replaced by a list that was refused"
        );
        assert!(holding.revoke(lent), "the turn's own grant was taken away");
    }

    /// The knock is recognised by asking the door it belongs to, and nothing
    /// else on either door is mistaken for it.
    #[test]
    fn only_the_knock_is_the_knock() {
        assert!(is_the_knock(r#"{"format":1,"asks":{"granted":{}}}"#));
        for other in [
            r#"{"format":1,"asks":{"waiting":{}}}"#,
            r#"{"format":1,"asks":{"approve":{"number":7}}}"#,
            r#"{"format":1,"asks":{"granted":{"folder":"/"}}}"#,
            r#"{"format":1,"asks":{"read":{"verb":"list_folder","given":[]}}}"#,
            "not a message at all",
        ] {
            assert!(!is_the_knock(other), "{other}");
        }
    }
}
