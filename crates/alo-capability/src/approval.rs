//! One approval: what a person agreed to, and the one execution it buys.
//!
//! ADR 0001 §5: *one approval, one action — an approval is never a session, and
//! no approval grants anything beyond the action named in it.* An [`Approved`]
//! is that sentence as a value, and it is shaped so that the rule does not
//! depend on anybody remembering it:
//!
//! - **it holds the call it was given**, so the arguments that run are the ones
//!   the person's sentence described. Nothing hands an executor a verb name and
//!   a fresh set of arguments alongside an approval for something else;
//! - **it does not lend out those arguments.** There is no accessor for them
//!   here; they come out of [`Approved::redeem`] inside an
//!   [`crate::Authorised`], which is the only type that means may-run;
//! - **it is spent by redeeming it.** `redeem` takes `self`, so a second
//!   execution is not something the compiler will assemble. The list refuses
//!   the other half — a proposal answered twice is [`crate::AnswerError`];
//! - **it names its own authority.** Redeeming takes no agent, so an approval
//!   given to one agent cannot be redeemed under the grants of another.
//!
//! There is nothing here about duration, remembering, or allowing something
//! again. Durable permission is a grant, made deliberately and revocable in one
//! action; an approval is one execution and then it is gone.

use std::time::SystemTime;

use crate::approvals::ProposalId;
use crate::authorised::{Authorised, Refused};
use crate::call::Call;
use crate::grant::Grantee;
use crate::grants::Grants;
use crate::proposal::Proposal;

/// A change one person approved, worth exactly one execution of it.
///
/// Deliberately not `Clone` and deliberately not `Deserialize`: either would be
/// a way to hold two of something that means once.
#[derive(Debug)]
pub struct Approved {
    /// The proposal this answers — what the record means by *which approval*.
    id: ProposalId,
    /// What was approved, with the sentence that was read.
    call: Call,
    /// Whose authority it was approved for.
    grantee: Grantee,
    /// When the person answered.
    approved_at: SystemTime,
}

impl Approved {
    /// Answer a proposal. Only [`crate::Approvals::approve`] can, because only
    /// the list can take a proposal off itself while answering it.
    pub(crate) fn of(id: ProposalId, proposal: Proposal, at: SystemTime) -> Self {
        let (call, grantee) = proposal.into_parts();
        Self {
            id,
            call,
            grantee,
            approved_at: at,
        }
    }

    /// Which proposal this answers.
    #[must_use]
    pub fn id(&self) -> ProposalId {
        self.id
    }

    /// The verb that was approved.
    #[must_use]
    pub fn verb(&self) -> &str {
        self.call.verb()
    }

    /// The sentence the person approved, word for word.
    #[must_use]
    pub fn sentence(&self) -> &str {
        self.call.sentence()
    }

    /// Whose authority this was approved for.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }

    /// When the person answered.
    #[must_use]
    pub fn approved_at(&self) -> SystemTime {
        self.approved_at
    }

    /// Spend the approval: ask the grants once more, and authorise what they
    /// still permit.
    ///
    /// The grants are asked here, at the moment of execution, and not when the
    /// person answered — a grant revoked or expired in between stops this, which
    /// is what *takes effect immediately* means.
    ///
    /// Taking `self` is the other half of *one approval, one execution*: an
    /// approval that has been redeemed no longer exists to redeem again.
    ///
    /// ```
    /// use alo_capability::*;
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let hour = Duration::from_secs(60 * 60);
    /// let verb = Verb::checked(
    ///     "archive_file",
    ///     "move a file into the archive",
    ///     Effect::Change,
    ///     vec![Arg::taking("file", "the file to archive", Takes::Path)],
    ///     Requires::grants_over(["file"]),
    ///     "archive {file}",
    /// )?;
    /// let call = Call::of(&verb, &[("file", Given::text("/home/anna/Invoices/march.pdf"))])?;
    /// let files = Grantee::named("@files");
    /// let mut grants = Grants::default();
    /// grants.grant(Grant::checked(
    ///     "@files",
    ///     Reach::Folder("/home/anna/Invoices".into()),
    ///     now,
    ///     hour,
    /// )?);
    ///
    /// let mut approvals = Approvals::default();
    /// let id = approvals.propose(Proposal::checked(&call, &files, &grants, now, hour)?);
    /// let approved = approvals.approve(id, now)?;
    /// let running = approved.redeem(&grants, now)?;
    /// assert_eq!(running.sentence(), "archive /home/anna/Invoices/march.pdf");
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// The same thing, redeemed twice, is not a program. This is the guarantee
    /// *one approval causes exactly one execution*, checked by the compiler:
    ///
    /// ```compile_fail
    /// use alo_capability::*;
    /// use std::time::{Duration, SystemTime};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let hour = Duration::from_secs(60 * 60);
    /// let verb = Verb::checked(
    ///     "archive_file",
    ///     "move a file into the archive",
    ///     Effect::Change,
    ///     vec![Arg::taking("file", "the file to archive", Takes::Path)],
    ///     Requires::grants_over(["file"]),
    ///     "archive {file}",
    /// )?;
    /// let call = Call::of(&verb, &[("file", Given::text("/home/anna/Invoices/march.pdf"))])?;
    /// let files = Grantee::named("@files");
    /// let mut grants = Grants::default();
    /// grants.grant(Grant::checked(
    ///     "@files",
    ///     Reach::Folder("/home/anna/Invoices".into()),
    ///     now,
    ///     hour,
    /// )?);
    ///
    /// let mut approvals = Approvals::default();
    /// let id = approvals.propose(Proposal::checked(&call, &files, &grants, now, hour)?);
    /// let approved = approvals.approve(id, now)?;
    /// let once = approved.redeem(&grants, now)?;
    /// let twice = approved.redeem(&grants, now)?; // the approval was spent above
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    /// [`Refused`], carrying the call, when the grants no longer permit it.
    pub fn redeem(self, grants: &Grants, now: SystemTime) -> Result<Authorised, Refused> {
        Authorised::granted(self.call, self.grantee, Some(self.id), grants, now)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::approvals::Approvals;
    use crate::authorised::NotAuthorised;
    use crate::grant::Grant;
    use crate::grants::Grants;
    use crate::reach::Reach;
    use crate::test_calls::{archiving_march, files, granting_both, hour, noon};

    /// Propose the archiving change and approve it, which is where every test
    /// below starts.
    fn approved(grants: &Grants) -> (Approvals, Approved) {
        let mut approvals = Approvals::default();
        let id = approvals.propose(
            Proposal::checked(&archiving_march(), &files(), grants, noon(), hour()).unwrap(),
        );
        let approved = approvals.approve(id, noon()).unwrap();
        (approvals, approved)
    }

    /// An approval says what was agreed to, in the words that were on the
    /// screen, and under whose authority it was agreed.
    #[test]
    fn an_approval_carries_what_was_approved() {
        let grants = granting_both();
        let (_, approved) = approved(&grants);
        assert_eq!(approved.verb(), "move_file");
        assert_eq!(
            approved.sentence(),
            "move /home/anna/Invoices/march.pdf into /home/anna/Archive"
        );
        assert_eq!(approved.grantee(), &files());
        assert_eq!(approved.approved_at(), noon());
        assert_eq!(approved.id().as_u64(), 0);
    }

    /// What runs is what was approved — exactly those arguments, because they
    /// travel inside the approval rather than beside it.
    #[test]
    fn redeeming_runs_exactly_what_was_approved() {
        let grants = granting_both();
        let (_, approved) = approved(&grants);
        let id = approved.id();
        let running = approved.redeem(&grants, noon()).unwrap();
        assert_eq!(running.call(), &archiving_march());
        assert_eq!(running.from_approval(), Some(id));
        assert_eq!(running.under(), &files());
        assert_eq!(
            running.sentence(),
            "move /home/anna/Invoices/march.pdf into /home/anna/Archive"
        );
    }

    /// A grant revoked after the approval and before the execution stops it.
    /// Nothing was decided ahead of time, so there is nothing left holding the
    /// old answer.
    #[test]
    fn a_grant_revoked_after_the_approval_stops_it() {
        let mut grants = granting_both();
        let (_, approved) = approved(&grants);
        assert_eq!(grants.revoke_everything_for(&files()), 2);
        let refused = approved.redeem(&grants, noon()).unwrap_err();
        assert!(matches!(refused.why(), NotAuthorised::NotGranted(_)));
        assert!(refused.to_string().contains("has not been granted"));
        assert_eq!(refused.call(), &archiving_march());
    }

    /// A grant that ran out between the approval and the execution stops it
    /// too, and the refusal says which of the two it was.
    #[test]
    fn a_grant_that_expired_after_the_approval_stops_it() {
        let grants = granting_both();
        let (_, approved) = approved(&grants);
        let refused = approved.redeem(&grants, noon() + hour()).unwrap_err();
        assert!(refused.to_string().contains("has expired"), "{refused}");
    }

    /// An approval is redeemed under the authority it was given, and there is
    /// no argument for naming another. Another agent's grants do not answer for
    /// this one.
    #[test]
    fn an_approval_is_redeemed_under_its_own_authority() {
        let grants = granting_both();
        let (_, approved) = approved(&grants);

        let mut someone_else = Grants::default();
        for folder in ["/home/anna/Invoices", "/home/anna/Archive"] {
            someone_else.grant(
                Grant::checked("@mail", Reach::Folder(folder.into()), noon(), hour()).unwrap(),
            );
        }
        let refused = approved.redeem(&someone_else, noon()).unwrap_err();
        assert!(refused.to_string().contains("@files"), "{refused}");
        assert!(refused.to_string().contains("has not been granted"));
    }
}
