//! A call that may run now — the only thing in this crate that means may-run.
//!
//! ADR 0001 §5 has two halves and this file is both of them, because they are
//! the same question asked of different calls: **a read answers inside the
//! turn, a change waits for one approval.** There are exactly two doors into an
//! [`Authorised`]:
//!
//! - [`Authorised::read`], for a call that changes nothing. A change offered
//!   here is refused rather than quietly run, so "changes wait" is a property of
//!   the type instead of a habit of whoever wrote the daemon;
//! - [`crate::Approved::redeem`], for a change, and the approval is spent
//!   getting through it.
//!
//! **Both doors ask the grants, and the redemption asks them again at the
//! moment of execution.** That is what makes a revoked grant take effect
//! immediately: a person who revokes a folder after approving something stops
//! it, because nothing decided anything ahead of time. Being permitted is
//! checked when the call is made, again when the change is proposed, and last
//! here — the answer that counts is the one given at the moment something would
//! happen.
//!
//! A refusal comes back as [`Refused`], which carries the call it refused. A
//! refusal is recorded (ADR 0001 §7), and a refusal that threw away what it
//! refused would leave the record saying only that something was stopped.
//!
//! **One question is asked from outside this crate**, and [`Refused::not_granted`]
//! and [`Refused::worded_elsewhere`] are how its answer comes back. Reach is
//! decided lexically here and nothing touches a disk ([`crate::path`]), so
//! whether the path a verb would *really* open is inside the grant can only be
//! asked by whatever resolves it — `alo-files` does, after this file has said
//! yes about the path as it was written. That refusal is the same fact as one
//! made here, so it comes back as the same type and reaches the record by the
//! same road.
//!
//! The two doors differ in one thing only: whether the words are this crate's.
//! A refusal the grants made is handed back as the value they made
//! ([`crate::NotGranted`]) and worded whenever somebody asks; a refusal about
//! something this crate cannot see — *this path really leads somewhere else* —
//! is worded by the crate that could see it, and arrives already said.
//!
//! An [`Authorised`] carries all four answers ADR 0001 §7 asks of a record —
//! what ran, under whose authority, from which approval, and against which
//! grant — because this is the one moment all four are true at once. The
//! `alo-record` crate writes them down; it never works them out again.

use std::time::SystemTime;

use alo_strings::{Filling, Said, Strings};

use crate::approvals::ProposalId;
use crate::call::Call;
use crate::grant::Grantee;
use crate::grants::{GrantId, Grants};
use crate::refusing::NotGranted;
use crate::words;

/// Why a call may not run.
///
/// **No `Display`**, like every other refusal a person reads: the road to words
/// is [`NotAuthorised::said`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAuthorised {
    /// The grants do not permit it — the refusal they made, which says whether
    /// the grant expired or was never made.
    NotGranted(NotGranted),
    /// The grants do not permit it, asked where this crate cannot ask, and
    /// worded there. See [`Refused::worded_elsewhere`].
    NotGrantedElsewhere(Said),
    /// A change offered where a read was expected.
    ChangeWaits {
        /// The verb that was offered.
        verb: String,
    },
}

impl NotAuthorised {
    /// What this says, in the language the person reads.
    ///
    /// A refusal that was already worded is handed back as it was said, because
    /// it was said by the only code that knew what it was about. The strings
    /// are a machine's rather than a caller's, so the two renderings are the
    /// same language in every case that exists.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NotGranted(why) => why.said(strings),
            Self::NotGrantedElsewhere(said) => said.clone(),
            Self::ChangeWaits { verb } => strings.say(
                &words::CHANGE_WAITS.key(),
                &Filling::of("verb", verb.clone()),
            ),
        }
    }
}

/// A refusal, and the call it was a refusal of.
///
/// The call comes back because the record keeps refusals as carefully as it
/// keeps executions: *the agent tried and was stopped* is the sentence a
/// security review needs, and it needs to name what was tried.
///
/// The call is boxed so that carrying it costs the happy path nothing: every
/// authorisation returns this type in its `Err`, and a refusal is the rare one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refused {
    /// What was refused.
    call: Box<Call>,
    /// Why it was.
    why: NotAuthorised,
}

impl Refused {
    /// A refusal by the grants, made where this crate could not ask the
    /// question itself.
    ///
    /// Reach is decided lexically here and touches no disk, so *is the path
    /// this verb would really open inside the grant?* is asked by whatever
    /// resolved it — see [`crate::path`] and the `alo-files` crate. The answer
    /// comes back as this type because it is the same fact as the last check in
    /// [`Authorised`] saying no: the grants were asked at the moment something
    /// would have run, and they refused.
    ///
    /// **It grants nothing**, which is why it can be public at all. The
    /// dangerous direction is a type that means may-run, and there is no
    /// constructor of one here; a refusal made in error stops something, which
    /// is the safe way to be wrong.
    #[must_use]
    pub fn not_granted(call: Call, why: NotGranted) -> Self {
        Self {
            call: Box::new(call),
            why: NotAuthorised::NotGranted(why),
        }
    }

    /// The same, for a refusal whose words are not this crate's to write.
    ///
    /// *This path is granted where it was written and really leads somewhere
    /// nobody granted* is a true sentence about the grants that only the crate
    /// holding the resolved path can say. It arrives already said, in the
    /// language the person reads, and travels into the record as it was said —
    /// so what somebody was told and what is written down are one rendering.
    #[must_use]
    pub fn worded_elsewhere(call: Call, why: Said) -> Self {
        Self {
            call: Box::new(call),
            why: NotAuthorised::NotGrantedElsewhere(why),
        }
    }

    /// What was refused, for the record.
    #[must_use]
    pub fn call(&self) -> &Call {
        &self.call
    }

    /// Why it was refused.
    #[must_use]
    pub fn why(&self) -> &NotAuthorised {
        &self.why
    }

    /// Why it was refused, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        self.why.said(strings)
    }
}

/// A call that may run, and the authority it runs under.
///
/// Deliberately not `Clone`: an authority that can be copied is an authority
/// that can be used twice, and one approval means one execution. An executor
/// takes this by value, runs it, and records what it was.
#[derive(Debug)]
pub struct Authorised {
    /// What runs: validated, with the sentence a person saw.
    call: Call,
    /// Whose grants permitted it.
    under: Grantee,
    /// Which approval it came from — `None` for a read, which needs none.
    from: Option<ProposalId>,
    /// The grants that permitted it, one for each thing it touches.
    against: Vec<GrantId>,
    /// The moment the grants were asked, which is the moment it may run.
    at: SystemTime,
}

impl Authorised {
    /// Authorise a read, which answers inside the turn.
    ///
    /// A read still needs its grant: running inside the turn is about approval,
    /// never about reach. The call is borrowed rather than taken, so a refusal
    /// leaves it with the caller to record.
    ///
    /// # Errors
    /// [`Refused`], carrying the call — [`NotAuthorised::ChangeWaits`] if a
    /// change was offered here, otherwise the grants' own refusal.
    pub fn read(
        call: &Call,
        grantee: &Grantee,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Self, Refused> {
        if call.waits_for_approval() {
            return Err(Refused {
                call: Box::new(call.clone()),
                why: NotAuthorised::ChangeWaits {
                    verb: call.verb().to_owned(),
                },
            });
        }
        Self::granted(call.clone(), grantee.clone(), None, grants, now)
    }

    /// Ask the grants, at the moment of execution, and authorise what they
    /// permit.
    ///
    /// Crate-private because it is the last check before something runs, and
    /// the two doors above are how anything reaches it.
    pub(crate) fn granted(
        call: Call,
        under: Grantee,
        from: Option<ProposalId>,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Self, Refused> {
        match call.permitting(grants, &under, now) {
            Ok(against) => Ok(Self {
                call,
                under,
                from,
                against,
                at: now,
            }),
            Err(why) => Err(Refused {
                call: Box::new(call),
                why: NotAuthorised::NotGranted(why),
            }),
        }
    }

    /// What runs.
    #[must_use]
    pub fn call(&self) -> &Call {
        &self.call
    }

    /// The verb that runs.
    #[must_use]
    pub fn verb(&self) -> &str {
        self.call.verb()
    }

    /// What this was, in the words a person read — or would have read, for a
    /// read that nobody was asked about.
    #[must_use]
    pub fn sentence(&self) -> &str {
        self.call.sentence()
    }

    /// Whose authority it runs under.
    #[must_use]
    pub fn under(&self) -> &Grantee {
        &self.under
    }

    /// Which approval it came from, or `None` for a read.
    ///
    /// This is the *from which approval* the record owes an answer to
    /// (ADR 0001 §7), and `None` is an answer: no approval, because none was
    /// needed.
    #[must_use]
    pub fn from_approval(&self) -> Option<ProposalId> {
        self.from
    }

    /// Which grants permitted it — one for each thing it touches, in the order
    /// the verb named them.
    ///
    /// This is the last of the four answers ADR 0001 §7 asks of a record, and
    /// it is carried here because here is where it was true. A record that
    /// asked the grants again afterwards would be reporting a second search,
    /// against a list a person may have changed since.
    ///
    /// Empty for a verb that requires no grant, which is the honest answer to
    /// *against which grant*: none, for the reason its author wrote down.
    #[must_use]
    pub fn against(&self) -> &[GrantId] {
        &self.against
    }

    /// The moment the grants were last asked.
    #[must_use]
    pub fn at(&self) -> SystemTime {
        self.at
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::test_calls::{
        archiving_march, files, granting, granting_both, hour, listing_invoices, noon,
    };
    use crate::testing::in_english;
    use alo_strings::Key;

    /// A read answers inside the turn: no proposal, no approval, no wait.
    #[test]
    fn a_read_runs_inside_the_turn() {
        let call = listing_invoices();
        let authorised =
            Authorised::read(&call, &files(), &granting(&["/home/anna/Invoices"]), noon()).unwrap();
        assert_eq!(authorised.verb(), "list_folder");
        assert_eq!(authorised.sentence(), "list what is in /home/anna/Invoices");
        assert_eq!(authorised.call(), &call);
        assert_eq!(authorised.under(), &files());
        assert_eq!(authorised.at(), noon());
        assert!(authorised.from_approval().is_none());
    }

    /// What ran carries all four of ADR 0001 §7's answers, and the fourth —
    /// *against which grant* — is the grant a person can find and revoke.
    #[test]
    fn what_may_run_names_the_grant_that_permitted_it() {
        let grants = granting(&["/home/anna/Invoices"]);
        let held: Vec<_> = grants.active_at(noon()).map(|held| held.id).collect();
        let authorised = Authorised::read(&listing_invoices(), &files(), &grants, noon()).unwrap();
        assert_eq!(authorised.against(), held);

        // Revoking that grant is what stops the same read a moment later, so
        // the grant the record names is the one that was actually load-bearing.
        let mut grants = grants;
        assert_eq!(grants.revoke_everything_for(&files()), 1);
        assert!(Authorised::read(&listing_invoices(), &files(), &grants, noon()).is_err());
    }

    /// Running inside the turn is about approval, never about reach. A read of
    /// a folder nobody granted is refused like anything else.
    #[test]
    fn a_read_still_needs_its_grant() {
        let call = listing_invoices();
        let refused = Authorised::read(&call, &files(), &granting(&["/home/anna/Taxes"]), noon())
            .unwrap_err();
        assert!(matches!(refused.why(), NotAuthorised::NotGranted(_)));
        assert!(
            refused
                .said(&in_english())
                .text()
                .contains("has not been granted")
        );
        // And the refusal knows what it refused, because it will be recorded.
        assert_eq!(refused.call(), &call);
    }

    /// An expired grant permits nothing, including a read.
    #[test]
    fn a_read_stops_when_the_grant_does() {
        let refused = Authorised::read(
            &listing_invoices(),
            &files(),
            &granting(&["/home/anna/Invoices"]),
            noon() + hour(),
        )
        .unwrap_err();
        let said = refused.said(&in_english());
        assert!(said.text().contains("has expired"), "{said}");
    }

    /// A refusal made outside this crate is the same fact and the same type, so
    /// the one question this crate cannot answer — whether the *real* path is
    /// inside the grant — reaches the record by the road every other refusal
    /// takes.
    ///
    /// Both doors are here: the grants' own refusal handed back as the value
    /// they made, and one whose words belong to the crate that could ask.
    #[test]
    fn a_refusal_made_where_the_disk_is_can_be_brought_back_here() {
        let call = listing_invoices();
        let strings = in_english();

        let theirs = Refused::worded_elsewhere(
            call.clone(),
            strings.say(
                &Key::named("files.refused.really-leads-elsewhere").unwrap(),
                &Filling::nothing(),
            ),
        );
        assert_eq!(theirs.call(), &call);
        assert!(matches!(
            theirs.why(),
            NotAuthorised::NotGrantedElsewhere(_)
        ));
        // A key this crate does not declare says so rather than pretending, and
        // the words still travel: whoever declared it is the crate that asked.
        assert!(theirs.said(&strings).is_a_bug());

        let ours = Refused::not_granted(
            call.clone(),
            NotGranted::Never {
                agent: "@files".to_owned(),
                wanted: crate::reach::Ask::path("/etc/shadow"),
            },
        );
        assert!(matches!(ours.why(), NotAuthorised::NotGranted(_)));
        assert!(ours.said(&strings).text().contains("/etc/shadow"));
    }

    /// **Changes wait.** A change offered at the read door is refused there,
    /// rather than being run because the code path happened to be reachable.
    #[test]
    fn a_change_cannot_take_the_read_door() {
        let call = archiving_march();
        let refused = Authorised::read(&call, &files(), &granting_both(), noon()).unwrap_err();
        assert_eq!(
            refused.why(),
            &NotAuthorised::ChangeWaits {
                verb: "move_file".to_owned()
            }
        );
        let said = refused.said(&in_english());
        assert!(said.text().contains("approve"), "{said}");
        assert_eq!(refused.call(), &call);
    }
}
