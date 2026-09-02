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
//! An [`Authorised`] carries all four answers ADR 0001 §7 asks of a record —
//! what ran, under whose authority, from which approval, and against which
//! grant — because this is the one moment all four are true at once. The
//! `alo-record` crate writes them down; it never works them out again.

use std::time::SystemTime;

use crate::approvals::ProposalId;
use crate::call::Call;
use crate::grant::Grantee;
use crate::grants::{GrantId, Grants};

/// Why a call may not run.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotAuthorised {
    /// The grants do not permit it, in the grants' own words — which say
    /// whether the grant expired or was never made.
    #[error("{0}")]
    NotGranted(String),
    /// A change offered where a read was expected.
    #[error(
        "{verb} changes something — propose it with the sentence describing it and let one person approve that, rather than running it in the turn"
    )]
    ChangeWaits {
        /// The verb that was offered.
        verb: String,
    },
}

/// A refusal, and the call it was a refusal of.
///
/// The call comes back because the record keeps refusals as carefully as it
/// keeps executions: *the agent tried and was stopped* is the sentence a
/// security review needs, and it needs to name what was tried.
///
/// The call is boxed so that carrying it costs the happy path nothing: every
/// authorisation returns this type in its `Err`, and a refusal is the rare one.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
#[error("{why}")]
pub struct Refused {
    /// What was refused.
    call: Box<Call>,
    /// Why it was.
    why: NotAuthorised,
}

impl Refused {
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
        assert!(refused.to_string().contains("has not been granted"));
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
        assert!(refused.to_string().contains("has expired"), "{refused}");
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
        assert!(refused.to_string().contains("approve"), "{refused}");
        assert_eq!(refused.call(), &call);
    }
}
