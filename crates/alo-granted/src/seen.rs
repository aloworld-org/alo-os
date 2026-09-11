//! One grant, as a person sees it in their list.
//!
//! The plan's acceptance for this task is a sentence about provenance — *the
//! list a surface would show is derived from the machine's own kept grants and
//! from nothing else — no constructor from text, so nothing can show a grant
//! the machine does not hold* — so provenance is what this type is shaped
//! around rather than what its documentation promises.
//!
//! # There is one door, and it is the machine's own list
//!
//! A [`Seen`] is made inside [`crate::Listing::of`], from an
//! `alo_capability::Held` that was on the machine's [`Grants`] and active at
//! the moment the list was derived. There is no constructor here that takes a
//! path, an agent's name, a moment or anything else somebody assembled, no
//! public field, no `From` and no deserialiser. What that buys:
//!
//! - every `Held` was built by `alo_capability::Grant::checked` on its way
//!   onto the list — rooted, free of `..`, never the whole machine, and with
//!   an end — whether it was granted this session or read back off the disk by
//!   `alo-remembering`, whose one road in is `Grants::remembered`;
//! - so a row on this surface is a grant the daemon's own `permits` would
//!   honour, and there is no shape in which a grant the machine does not hold
//!   could be drawn.
//!
//! # Revoking a row is the machine's own revocation
//!
//! [`Seen::revoke`] calls [`Grants::revoke`] with the handle the row was
//! derived with — the same method a daemon's list is revoked by, taking effect
//! on the next question asked, because there is no cache in front of it. This
//! crate adds no second mechanism that could disagree with the first; what it
//! adds is the two answers a person needs ([`crate::Revoked`]).

use std::time::{Duration, SystemTime};

use alo_capability::{Grant, GrantId, Grants, Held, Reach};
use alo_strings::{Filling, Said, Strings};

use crate::revoking::Revoked;
use crate::words;

/// One grant a person can see: who may reach what, since when, and for how
/// much longer.
///
/// ```
/// use alo_capability::{Grant, Grants, Reach};
/// use alo_granted::Listing;
/// use std::path::PathBuf;
/// use std::time::{Duration, SystemTime};
///
/// let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
/// let hour = Duration::from_secs(60 * 60);
/// let mut grants = Grants::default();
/// grants.grant(Grant::checked(
///     "@files",
///     Reach::Folder(PathBuf::from("/home/anna/Invoices")),
///     noon,
///     hour,
/// )?);
///
/// let listing = Listing::of(&grants, noon);
/// let row = listing.rows().first().expect("one grant, one row");
/// assert_eq!(row.to(), "@files");
/// assert_eq!(row.expires_in(), hour);
/// # Ok::<(), alo_capability::GrantError>(())
/// ```
///
/// A row cannot be made from text somebody assembled, and this is what says
/// so — there is no such constructor to call:
///
/// ```compile_fail
/// let row = alo_granted::Seen::of_text("@files", "/home/anna/Invoices");
/// ```
///
/// nor can one be assembled from its parts, because it has none that are
/// reachable:
///
/// ```compile_fail
/// let row = alo_granted::Seen {
///     id: alo_capability::GrantId::numbered(0),
///     to: String::from("@files"),
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seen {
    /// The handle the machine holds this grant under — what [`Seen::revoke`]
    /// hands to [`Grants::revoke`], so the row and the revocation cannot be
    /// about two different grants.
    id: GrantId,
    /// The agent's name, as the system knows it.
    to: String,
    /// What the grant covers, worded by `alo-capability` when it is shown.
    over: Reach,
    /// When the person made it.
    granted_at: SystemTime,
    /// How much of it was left at the moment the list was derived. Never
    /// zero: an expired grant never becomes a row at all.
    left: Duration,
}

impl Seen {
    /// The row this held grant reads as, at this moment — or [`None`] once it
    /// has expired, which is how *an expired grant is never shown as live*
    /// stays a property of the type rather than a habit of its callers.
    ///
    /// Crate-private: the one caller is [`crate::Listing::of`], and the one
    /// road to a `Held` outside a test is the machine's own [`Grants`].
    pub(crate) fn of(held: &Held, now: SystemTime) -> Option<Self> {
        let grant: &Grant = &held.grant;
        grant.expires_in(now).map(|left| Self {
            id: held.id,
            to: grant.grantee.as_str().to_owned(),
            over: grant.reach.clone(),
            granted_at: grant.granted_at,
            left,
        })
    }

    /// The handle the machine holds this grant under — what a list shows so a
    /// late revocation cannot land on a grant made since (handles are never
    /// reused).
    #[must_use]
    pub const fn id(&self) -> GrantId {
        self.id
    }

    /// Which agent may reach it, by the name the system knows it by.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// What it covers.
    #[must_use]
    pub const fn over(&self) -> &Reach {
        &self.over
    }

    /// When the person made it — shown beside the row, so they can see what
    /// they did and when.
    #[must_use]
    pub const fn granted_at(&self) -> SystemTime {
        self.granted_at
    }

    /// How much of it was left at the moment the list was derived. Never
    /// zero.
    ///
    /// A duration rather than a formatted time, for `alo-capability`'s own
    /// reason: formatting an expiry here would hardcode a calendar as well as
    /// a language, and neither decision belongs to this crate.
    #[must_use]
    pub const fn expires_in(&self) -> Duration {
        self.left
    }

    /// The row, in the language the person reads: who may reach what, with
    /// the *what* worded by the crate that decides what a grant covers.
    ///
    /// The times are deliberately not in it — they are shown beside the row
    /// by whoever displays it, from [`Seen::granted_at`] and
    /// [`Seen::expires_in`].
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of(words::AGENT, self.to.clone())
            .and_said(words::WHAT, &self.over.said(strings));
        strings.say(&words::ONE_GRANT.key(), &filling)
    }

    /// Take this grant away, through the machine's own list.
    ///
    /// This is [`Grants::revoke`] and nothing beside it — the same revocation
    /// the daemon's `permits` answers to, taking effect on the next question
    /// asked. The answer says which of the two things happened, because a row
    /// can outlive the grant it shows: [`Revoked::Now`] when this act removed
    /// it, and [`Revoked::AlreadyGone`] when the list had moved on — in which
    /// case **`grants` is exactly as it was**.
    pub fn revoke(&self, grants: &mut Grants) -> Revoked {
        if grants.revoke(self.id) {
            Revoked::Now
        } else {
            Revoked::AlreadyGone
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
    use crate::Listing;
    use crate::testing::{grants_of_one_folder, hour, in_english, noon, translated};
    use alo_capability::{Ask, Grantee};

    /// The one row these tests read.
    fn the_row(grants: &Grants) -> Seen {
        Listing::of(grants, noon()).rows().first().unwrap().clone()
    }

    /// **A row says who, what, since when and for how much longer** — the four
    /// things ADR 0001 §3 makes a grant out of, read back off the machine's
    /// own list.
    #[test]
    fn a_row_says_who_what_since_when_and_for_how_long() {
        let grants = grants_of_one_folder();
        let row = the_row(&grants);
        assert_eq!(row.to(), "@files");
        assert_eq!(
            row.over(),
            &Reach::Folder(std::path::PathBuf::from("/home/anna/Invoices"))
        );
        assert_eq!(row.granted_at(), noon());
        assert_eq!(row.expires_in(), hour());

        let said = row.said(&in_english());
        assert!(!said.is_a_bug(), "the row is not declared");
        assert_eq!(
            said.text(),
            "@files can reach /home/anna/Invoices and everything in it"
        );
    }

    /// **Revoking a row takes effect on the daemon's own `permits`
    /// immediately** — the next question, not the next sign-in — because it
    /// is the same revocation, not a second mechanism.
    #[test]
    fn revoking_a_row_stops_the_daemons_own_permits_at_once() {
        let mut grants = grants_of_one_folder();
        let row = the_row(&grants);
        let ask = Ask::path("/home/anna/Invoices/march.pdf");
        assert!(grants.permits(&Grantee::named("@files"), &ask, noon()));

        assert_eq!(row.revoke(&mut grants), Revoked::Now);
        assert!(!grants.permits(&Grantee::named("@files"), &ask, noon()));
    }

    /// **A stale row revokes nothing and changes nothing.** The refusal is
    /// tested as carefully as the answer: the grants after a refused
    /// revocation are byte for byte the grants before it.
    #[test]
    fn a_stale_row_changes_nothing_at_all() {
        let mut grants = grants_of_one_folder();
        let row = the_row(&grants);
        assert_eq!(row.revoke(&mut grants), Revoked::Now);

        let before = serde_json::to_string(&grants).unwrap();
        assert_eq!(row.revoke(&mut grants), Revoked::AlreadyGone);
        assert_eq!(serde_json::to_string(&grants).unwrap(), before);
    }

    /// **The words around the row are translated and the names in it are
    /// not.** A person reading their list in German reads German about an
    /// agent that is still `@files` and a path that is still theirs.
    #[test]
    fn the_row_reads_in_the_language_the_person_reads() {
        let strings = translated(&[(words::ONE_GRANT, "{agent} erreicht {what}")]);
        let said = the_row(&grants_of_one_folder()).said(&strings);
        assert!(said.text().starts_with("@files erreicht"), "{said}");
        assert!(said.text().contains("/home/anna/Invoices"), "{said}");
    }

    /// **A machine that never declared these words is told so** rather than
    /// shown a blank row — the refusal path of externalisation itself.
    #[test]
    fn a_machine_that_never_declared_these_words_says_it_is_a_bug() {
        let nothing = Strings::of(alo_strings::Vocabulary::empty());
        let said = the_row(&grants_of_one_folder()).said(&nothing);
        assert!(said.is_a_bug());
        assert!(said.text().contains("granted.one-grant"));
    }
}
