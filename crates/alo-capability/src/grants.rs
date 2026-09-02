//! The grants held on this machine, and the questions asked of them.
//!
//! Two audiences ask this type things, and it owes them different answers.
//!
//! **The person** asks what is granted, to whom, and until when — so the list
//! is enumerated and readable ([`Grants::active_at`], [`Grants::held_by`]) and
//! one action takes a grant away ([`Grants::revoke`]).
//!
//! **The daemon** asks whether one agent may touch one thing, right now
//! ([`Grants::permitting`]), and gets back the grant that said yes or a
//! [`NotGranted`] saying why none of them did. [`Grants::permits`] and
//! [`Grants::refusal`] are the two halves of that one answer, so there is only
//! ever one search and nothing can be permitted by a grant the record cannot
//! name.
//!
//! **The refusal is a value and not a sentence.** Nothing here needs a
//! vocabulary to decide, and deciding must never depend on one having been
//! loaded — [`crate::refusing`] is where that is argued out, and where the
//! words are.
//!
//! Nothing here widens anything. Every query takes `&self`; asking about a path
//! a hundred times leaves the same grants that were there before, which is the
//! machine-checked half of "never widened by use". The other half is that there
//! is no method that turns a refusal into a grant: making one is a person's
//! act, and it happens in a file chooser rather than in this file.
//!
//! Expiry is not swept: a grant is expired when the time says so, whether or
//! not anything has run since. [`Grants::forget_expired`] exists to keep the
//! list short, and removing it would change nothing about what is permitted.

use std::time::SystemTime;

use serde::{Deserialize, Serialize};

use crate::grant::{Grant, Grantee};
use crate::reach::Ask;
use crate::refusing::NotGranted;

/// The handle a person revokes a grant by.
///
/// Unique for the life of the list. Ids are never reused, so a revoke that
/// arrives late — the settings panel was showing a list from a moment ago —
/// cannot land on a grant made since.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GrantId(u64);

impl GrantId {
    /// The number behind the handle, for showing and storing.
    #[must_use]
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// A grant as the list holds it: the grant, and the handle it is revoked by.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Held {
    /// What to pass to [`Grants::revoke`].
    pub id: GrantId,
    /// The grant itself.
    pub grant: Grant,
}

/// Every grant on this machine.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Grants {
    /// The next handle to hand out. Kept so that ids stay unique across a
    /// restart, since the list is written down and read back.
    #[serde(default)]
    next: u64,
    /// In the order they were made, which is the order a person will look for
    /// them in.
    #[serde(default)]
    held: Vec<Held>,
}

impl Grants {
    /// Add a grant, and return the handle it can be revoked by.
    ///
    /// If the same agent already holds the same reach, that entry is replaced
    /// rather than duplicated. Two identical rows would mean revoking the one
    /// a person can see and leaving the one they cannot — a list that lies
    /// about what is granted is worse than no list.
    pub fn grant(&mut self, grant: Grant) -> GrantId {
        self.held
            .retain(|held| !(held.grant.is_for(&grant.grantee) && held.grant.reach == grant.reach));
        let id = GrantId(self.next);
        self.next = self.next.saturating_add(1);
        self.held.push(Held { id, grant });
        id
    }

    /// Take a grant away. Says whether there was one.
    ///
    /// It takes effect on the next question asked, because there is no cache
    /// in front of this list and no decision made ahead of time.
    pub fn revoke(&mut self, id: GrantId) -> bool {
        let before = self.held.len();
        self.held.retain(|held| held.id != id);
        self.held.len() != before
    }

    /// Take away everything one agent holds, and say how many that was.
    ///
    /// "Stop this agent reaching anything" is one action, as ADR 0001 §3
    /// requires, rather than as many actions as the person happens to have
    /// granted.
    pub fn revoke_everything_for(&mut self, grantee: &Grantee) -> usize {
        let before = self.held.len();
        self.held.retain(|held| !held.grant.is_for(grantee));
        before.saturating_sub(self.held.len())
    }

    /// Drop the grants that have expired, and say how many went.
    ///
    /// Housekeeping only: an expired grant permits nothing whether or not this
    /// has been called.
    pub fn forget_expired(&mut self, now: SystemTime) -> usize {
        let before = self.held.len();
        self.held.retain(|held| held.grant.is_active_at(now));
        before.saturating_sub(self.held.len())
    }

    /// What is granted at this moment, in the order it was granted.
    pub fn active_at(&self, now: SystemTime) -> impl Iterator<Item = &Held> {
        self.held
            .iter()
            .filter(move |held| held.grant.is_active_at(now))
    }

    /// What one agent holds at this moment.
    pub fn held_by<'a>(
        &'a self,
        grantee: &'a Grantee,
        now: SystemTime,
    ) -> impl Iterator<Item = &'a Held> {
        self.active_at(now)
            .filter(move |held| held.grant.is_for(grantee))
    }

    /// How many grants are on the list, expired ones included.
    ///
    /// The number a settings panel shows next to "forget the expired ones",
    /// not an answer about what is permitted.
    #[must_use]
    pub fn len(&self) -> usize {
        self.held.len()
    }

    /// Whether nothing has been granted at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.held.is_empty()
    }

    /// Which grant permits this agent to touch this thing, at this moment —
    /// or why none of them does.
    ///
    /// The whole crate exists for this method. Everything it does not permit
    /// is refused: there is no default, no fallback and no path that is
    /// reachable because nobody thought to forbid it.
    ///
    /// It answers with the grant rather than with `true` because a record owes
    /// an answer to *against which grant* (ADR 0001 §7), and this search is the
    /// only moment that answer exists. Deriving it again afterwards would be a
    /// second search, made against a list that may have moved on, and two
    /// searches that can disagree are worse than none.
    ///
    /// # Errors
    /// [`NotGranted`], carrying what was asked for and the grant that ran out
    /// where there was one — see [`Grants::refusal`] for what it says and why.
    pub fn permitting(
        &self,
        grantee: &Grantee,
        ask: &Ask,
        now: SystemTime,
    ) -> Result<GrantId, NotGranted> {
        self.held
            .iter()
            .find(|held| held.grant.is_for(grantee) && held.grant.permits(ask, now))
            .map(|held| held.id)
            .ok_or_else(|| self.why_not(grantee, ask))
    }

    /// Whether this agent may touch this thing, at this moment.
    #[must_use]
    pub fn permits(&self, grantee: &Grantee, ask: &Ask, now: SystemTime) -> bool {
        self.permitting(grantee, ask, now).is_ok()
    }

    /// Why an agent may not touch something — `None` when it may.
    ///
    /// This is read by a person after the fact, in the record of a refusal, so
    /// it says which of the two reasons it was. "It expired" and "you never
    /// granted it" call for different things from the reader, and a message
    /// that covered both would tell them to check something they already know.
    #[must_use]
    pub fn refusal(&self, grantee: &Grantee, ask: &Ask, now: SystemTime) -> Option<NotGranted> {
        self.permitting(grantee, ask, now).err()
    }

    /// The refusal, for an ask no grant permitted.
    ///
    /// Private because it is only ever true alongside a failed search: called
    /// on its own it would say no about something that is in fact granted. It
    /// looks for an expired grant that *would* have covered this, because that
    /// is the difference between the two things a person can do about it.
    fn why_not(&self, grantee: &Grantee, ask: &Ask) -> NotGranted {
        let agent = grantee.as_str().to_owned();
        let lapsed = self
            .held
            .iter()
            .find(|held| held.grant.is_for(grantee) && held.grant.reach.covers(ask));
        match lapsed {
            Some(held) => NotGranted::Lapsed {
                agent,
                reach: held.grant.reach.clone(),
                wanted: ask.clone(),
            },
            None => NotGranted::Never {
                agent,
                wanted: ask.clone(),
            },
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::grant::GrantError;
    use crate::reach::Reach;
    use std::path::PathBuf;
    use std::time::Duration;

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    fn files() -> Grantee {
        Grantee::named("@files")
    }

    fn invoices() -> Reach {
        Reach::Folder(PathBuf::from("/home/anna/Invoices"))
    }

    fn march() -> Ask {
        Ask::path("/home/anna/Invoices/march.pdf")
    }

    fn granted_for(lasting: Duration) -> Result<Grant, GrantError> {
        Grant::checked("@files", invoices(), noon(), lasting)
    }

    fn one_grant() -> Grants {
        let mut grants = Grants::default();
        grants.grant(granted_for(hour()).unwrap());
        grants
    }

    /// A path outside every grant is refused. This is the whole product in one
    /// assertion: the agent asked, and the answer was no.
    #[test]
    fn a_path_outside_a_grant_is_refused() {
        let grants = one_grant();
        assert!(grants.permits(&files(), &march(), noon()));
        assert!(!grants.permits(&files(), &Ask::path("/home/anna/Taxes/2024.pdf"), noon()));
        assert!(!grants.permits(&files(), &Ask::path("/home/anna"), noon()));
        assert!(!grants.permits(&files(), &Ask::path("/etc/shadow"), noon()));
    }

    /// The record has to be able to say *which* grant permitted something, so
    /// the answer is the grant rather than a yes — and it is the grant a person
    /// can find in their list and revoke.
    #[test]
    fn what_permits_something_is_answered_by_name() {
        let mut grants = one_grant();
        let invoices = grants.active_at(noon()).next().unwrap().id;
        let taxes = grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Taxes")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        assert_eq!(grants.permitting(&files(), &march(), noon()), Ok(invoices));
        assert_eq!(
            grants.permitting(&files(), &Ask::path("/home/anna/Taxes/2024.pdf"), noon()),
            Ok(taxes)
        );

        // And a refusal is the same answer's other half, so nothing can be
        // permitted by a grant that cannot be named.
        assert!(
            grants
                .permitting(&files(), &Ask::path("/etc/shadow"), noon())
                .is_err()
        );
        assert!(grants.revoke(invoices));
        assert!(matches!(
            grants.permitting(&files(), &march(), noon()),
            Err(NotGranted::Never { .. })
        ));
    }

    /// A grant is for one agent. Another agent's grant is not a fallback.
    #[test]
    fn one_agents_grant_is_not_another_agents() {
        let grants = one_grant();
        assert!(!grants.permits(&Grantee::named("@mail"), &march(), noon()));
        assert_eq!(grants.held_by(&Grantee::named("@mail"), noon()).count(), 0);
    }

    /// A revoked grant stops immediately — on the next question, not at the
    /// next sign-in.
    #[test]
    fn a_revoked_grant_stops_at_once() {
        let mut grants = one_grant();
        let id = grants.active_at(noon()).next().unwrap().id;
        assert!(grants.permits(&files(), &march(), noon()));
        assert!(grants.revoke(id));
        assert!(!grants.permits(&files(), &march(), noon()));
        // And revoking it again changes nothing, rather than reporting success
        // for something it did not do.
        assert!(!grants.revoke(id));
    }

    /// Everything one agent holds goes in one action.
    #[test]
    fn everything_an_agent_holds_can_go_at_once() {
        let mut grants = one_grant();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::File(PathBuf::from("/home/anna/Taxes/2024.pdf")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        grants.grant(Grant::checked("@mail", invoices(), noon(), hour()).unwrap());
        assert_eq!(grants.revoke_everything_for(&files()), 2);
        assert!(!grants.permits(&files(), &march(), noon()));
        assert!(grants.permits(&Grantee::named("@mail"), &march(), noon()));
    }

    /// An expired grant is gone whether or not anything has swept it up.
    #[test]
    fn an_expired_grant_permits_nothing_and_is_not_listed() {
        let mut grants = one_grant();
        let later = noon() + hour();
        assert!(!grants.permits(&files(), &march(), later));
        assert_eq!(grants.active_at(later).count(), 0);
        assert_eq!(grants.held_by(&files(), later).count(), 0);
        // Still on the list until it is swept, and still permitting nothing.
        assert_eq!(grants.len(), 1);
        assert_eq!(grants.forget_expired(later), 1);
        assert!(grants.is_empty());
        assert!(!grants.permits(&files(), &march(), later));
    }

    /// Asking is not granting. A hundred refused questions leave the list
    /// exactly as it was — reach is never widened by use.
    #[test]
    fn asking_about_a_path_never_grants_it() {
        let grants = one_grant();
        let before = serde_json::to_string(&grants).unwrap();
        for path in [
            "/home/anna/Taxes/2024.pdf",
            "/etc/shadow",
            "/home/anna",
            "/",
        ] {
            assert!(
                !grants.permits(&files(), &Ask::path(path), noon()),
                "{path}"
            );
        }
        assert_eq!(serde_json::to_string(&grants).unwrap(), before);
        assert_eq!(grants.active_at(noon()).count(), 1);
    }

    /// Granting the same folder twice leaves one row, so revoking the one a
    /// person can see is not shadowed by one they cannot.
    #[test]
    fn granting_the_same_reach_twice_leaves_one_grant() {
        let mut grants = one_grant();
        let again = grants.grant(granted_for(hour() * 2).unwrap());
        assert_eq!(grants.len(), 1);
        assert!(grants.revoke(again));
        assert!(!grants.permits(&files(), &march(), noon()));
    }

    /// A handle is never reused, so a revoke from a stale list cannot land on
    /// a grant made since.
    #[test]
    fn a_handle_is_never_reused() {
        let mut grants = one_grant();
        let first = grants.active_at(noon()).next().unwrap().id;
        assert!(grants.revoke(first));
        let second = grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Taxes")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        assert_ne!(first, second);
        assert!(!grants.revoke(first));
        assert!(grants.permits(&files(), &Ask::path("/home/anna/Taxes/2024.pdf"), noon()));
    }

    /// A refusal says which of the two reasons it was: an expiry is something
    /// a person fixes by granting again, and a path they never granted is not.
    ///
    /// It says it as a *value* — the words are asked for afterwards, from the
    /// strings the person reads, and the value carries everything those words
    /// need.
    #[test]
    fn a_refusal_says_whether_it_expired_or_was_never_granted() {
        let grants = one_grant();
        assert!(grants.refusal(&files(), &march(), noon()).is_none());

        let expired = grants.refusal(&files(), &march(), noon() + hour()).unwrap();
        assert_eq!(
            expired,
            NotGranted::Lapsed {
                agent: "@files".to_owned(),
                reach: invoices(),
                wanted: march(),
            }
        );

        let taxes = Ask::path("/home/anna/Taxes/2024.pdf");
        let never = grants.refusal(&files(), &taxes, noon()).unwrap();
        assert_eq!(
            never,
            NotGranted::Never {
                agent: "@files".to_owned(),
                wanted: taxes,
            }
        );

        let words = never.said(&crate::testing::in_english());
        assert!(words.text().contains("has not been granted"), "{words}");
        assert!(words.text().contains("never by asking for one"), "{words}");
    }

    /// The list a person reads: what is granted, to whom, and until when.
    #[test]
    fn the_list_says_what_is_granted_to_whom_and_until_when() {
        let mut grants = one_grant();
        grants.grant(
            Grant::checked(
                "@blender",
                Reach::Application("org.blender.Blender".to_owned()),
                noon(),
                hour() * 2,
            )
            .unwrap(),
        );
        let listed: Vec<_> = grants.active_at(noon()).collect();
        assert_eq!(listed.len(), 2);
        let first = listed.first().unwrap();
        assert_eq!(first.grant.grantee.as_str(), "@files");
        assert!(
            first
                .grant
                .reach
                .shown(&crate::testing::in_english())
                .contains("everything in it")
        );
        assert_eq!(first.grant.expires_in(noon()), Some(hour()));

        let blender = Grantee::named("@blender");
        assert_eq!(grants.held_by(&blender, noon()).count(), 1);
        assert!(grants.permits(&blender, &Ask::application("org.blender.Blender"), noon()));
    }

    /// The list is written down and read back — a grant made on Monday is on
    /// the list on Tuesday, expiring when it always was.
    #[test]
    fn the_list_survives_being_written_down_and_read_back() {
        let grants = one_grant();
        let written = serde_json::to_string(&grants).unwrap();
        let read: Grants = serde_json::from_str(&written).unwrap();
        assert!(read.permits(&files(), &march(), noon()));
        assert!(!read.permits(&files(), &march(), noon() + hour()));

        // And handles keep going from where they were, rather than starting
        // again and colliding with a grant that is still on the list.
        let mut read = read;
        let existing = read.active_at(noon()).next().unwrap().id;
        let next = read.grant(
            Grant::checked(
                "@files",
                Reach::Folder(PathBuf::from("/home/anna/Taxes")),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        assert_ne!(existing, next);
        assert_eq!(read.active_at(noon()).count(), 2);
    }
}
