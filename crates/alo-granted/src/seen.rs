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
//! - every `Held` was built by `alo_capability::Grant::checked_for` on its way
//!   onto the list — rooted, free of `..`, never the whole machine, and with
//!   an end — whether it was granted this session or read back off the disk by
//!   `alo-remembering`, whose one road in is `Grants::remembered`;
//! - so a row on this surface is a grant the daemon's own `permits` would
//!   honour, and there is no shape in which a grant the machine does not hold
//!   could be drawn.
//!
//! # An application's row is an agent's row
//!
//! [ADR 0040](../../../docs/decisions/0040-what-an-applications-grant-is-over.md)
//! put applications' grants on the one `alo_capability::Grants` an agent's are
//! on, and ADR 0005's promise is *one list — agents and applications in the
//! same place, revoked the same way*. So a [`Seen`] **does not say which kind
//! it is**: it keeps the grantee's name and nothing else about the grantee, it
//! is worded by the one clause [`crate::words::ONE_GRANT`], and it is revoked
//! by the one [`Seen::revoke`]. A surface that wants to sort the list into
//! agents and applications has nothing here to sort by, which is the point: a
//! person reads who has been granted what, and the name says who.
//!
//! # Revoking a row is the machine's own revocation
//!
//! [`Seen::revoke`] calls [`Grants::revoke`] with the handle the row was
//! derived with — the same method a daemon's list is revoked by, taking effect
//! on the next question asked, because there is no cache in front of it. This
//! crate adds no second mechanism that could disagree with the first; what it
//! adds is the two answers a person needs ([`crate::Revoked`]).
//!
//! A portal request is judged against that same list
//! (`alo_portals::Request::judged`), with nothing remembered between one
//! request and the next, so an application's revoked grant is refused at its
//! next request exactly as an agent's is refused at its next verb.
//!
//! On a machine whose person declined the agent (ADR 0009) there is no
//! `&mut Grants` to hand over — only applications' grants are held, and they
//! are revoked through `alo_capability::Agent::revoke_allowed`.
//! [`Seen::revoke_on`] is that one action taken on the machine value rather
//! than on its list, and answers the same two ways.

use std::time::{Duration, SystemTime};

use alo_capability::{Agent, Grant, GrantId, Grants, Held, Reach};
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
    /// The name of whoever holds the grant, as the system knows it — an
    /// agent's or an application's. Deliberately not the `Grantee`: a row
    /// does not carry which kind it is, so no surface can show the two kinds
    /// differently.
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

    /// Who holds it — an agent or an application — by the name the system
    /// knows it by. The only thing about a row that says whose it is.
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

    /// The row, in the language the person reads: who has been granted what,
    /// with the *what* worded by the crate that decides what a grant covers.
    /// One clause for an agent's row and an application's.
    ///
    /// The times are deliberately not in it — they are shown beside the row
    /// by whoever displays it, from [`Seen::granted_at`] and
    /// [`Seen::expires_in`].
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of(words::WHO, self.to.clone())
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
        answered(grants.revoke(self.id))
    }

    /// Take this grant away, on the machine value rather than its list.
    ///
    /// On a machine with an agent this is [`Seen::revoke`] on its one list.
    /// On a machine whose person declined the agent it is
    /// [`Agent::revoke_allowed`], because such a machine holds applications'
    /// grants alone and hands out no `&mut Grants` (ADR 0009, ADR 0040) — and
    /// a row naming anything else there lands on nothing and changes nothing.
    pub fn revoke_on(&self, machine: &mut Agent) -> Revoked {
        match machine.grants_mut() {
            Some(grants) => self.revoke(grants),
            None => answered(machine.revoke_allowed(self.id)),
        }
    }
}

/// Which of the two answers a revocation that did or did not remove a grant is.
const fn answered(removed: bool) -> Revoked {
    if removed {
        Revoked::Now
    } else {
        Revoked::AlreadyGone
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
    use crate::testing::{
        camera_for_cheese, grants_of_a_folder_and_a_camera, grants_of_one_folder, hour, in_english,
        noon, translated,
    };
    use alo_capability::{Applicant, Ask, Facility, Grantee};

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
            "@files has been granted /home/anna/Invoices and everything in it"
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
        let strings = translated(&[(words::ONE_GRANT, "{who} wurde {what} gewährt")]);
        let said = the_row(&grants_of_one_folder()).said(&strings);
        assert!(said.text().starts_with("@files wurde"), "{said}");
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

    /// **An application's row says itself in the agent's clause**, with the
    /// facility worded by `alo-capability`.
    #[test]
    fn an_applications_row_is_worded_by_the_same_clause() {
        let grants = grants_of_a_folder_and_a_camera();
        let listing = Listing::of(&grants, noon());
        assert_eq!(listing.rows().len(), 2, "two grants, two rows");
        let files = listing.rows().first().unwrap();
        let cheese = listing.rows().get(1).unwrap();
        assert_eq!(cheese.to(), "org.gnome.Cheese");
        assert_eq!(cheese.over(), &Reach::Facility(Facility::Camera));
        let strings = in_english();
        assert_eq!(
            cheese.said(&strings).text(),
            "org.gnome.Cheese has been granted the camera"
        );
        assert!(
            files
                .said(&strings)
                .text()
                .starts_with("@files has been granted")
        );
    }

    /// **On a machine with an agent, revoking on the machine is revoking on
    /// its list** — for an application's row and an agent's alike.
    #[test]
    fn revoking_on_a_machine_with_an_agent_is_revoking_its_list() {
        let mut machine = Agent::present();
        let grants = machine.grants_mut().unwrap();
        *grants = grants_of_a_folder_and_a_camera();
        let listing = Listing::of(machine.allowed(), noon());
        for row in listing.rows() {
            assert_eq!(row.revoke_on(&mut machine), Revoked::Now, "{row:?}");
            assert_eq!(row.revoke_on(&mut machine), Revoked::AlreadyGone, "{row:?}");
        }
        assert!(machine.allowed().is_empty());
    }

    /// **On a declined machine an application's row is still revoked, and an
    /// agent's row from before declining lands on nothing and changes
    /// nothing.**
    #[test]
    fn revoking_on_a_declined_machine_reaches_only_what_it_holds() {
        let mut machine = Agent::present();
        let before_declining = {
            let grants = machine.grants_mut().unwrap();
            *grants = grants_of_one_folder();
            Listing::of(grants, noon())
        };
        let cheese = machine.allow(camera_for_cheese()).unwrap();
        let _ended = machine.declining(noon());

        let stale_agents_row = before_declining.rows().first().unwrap();
        let kept = serde_json::to_string(machine.allowed()).unwrap();
        assert_eq!(
            stale_agents_row.revoke_on(&mut machine),
            Revoked::AlreadyGone
        );
        assert_eq!(serde_json::to_string(machine.allowed()).unwrap(), kept);

        let listing = Listing::of(machine.allowed(), noon());
        let row = listing.rows().first().unwrap();
        assert_eq!(row.id(), cheese);
        let camera = Ask::facility(Facility::Camera);
        let application = Applicant::named("org.gnome.Cheese");
        assert!(machine.allowing(&application, &camera, noon()).is_ok());
        assert_eq!(row.revoke_on(&mut machine), Revoked::Now);
        assert!(machine.allowing(&application, &camera, noon()).is_err());
        assert!(Listing::of(machine.allowed(), noon()).is_nothing_granted());
    }
}
