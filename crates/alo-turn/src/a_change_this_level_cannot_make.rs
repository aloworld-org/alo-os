//! **A change the person's level of running code cannot make**, refused before
//! anybody is asked — and written down, as every refusal is.
//!
//! Publishing a folder runs `git` and `gh` as the person, and the sealed box
//! (ADR 0064's default) cannot run programs on this computer yet. So at the
//! sealed box a publication is not put to the person at all: asking them to
//! approve something the machine will then not do would be an approval that
//! does nothing, which `crate::machine`'s header calls the worst outcome
//! available. The level is the daemon's to read — this crate reads no file —
//! so the daemon decides and words it, and this is the door it records through.

use std::time::SystemTime;

use alo_capability::{AnswerError, Given, ProposalId, Refused};
use alo_record::Entry;
use alo_strings::Said;

use crate::became::Became;
use crate::refusing::NotDone;
use crate::turning::Turning;

impl Turning<'_, '_> {
    /// Refuse `verb` with `given` in `said`, because the person's level cannot
    /// make it; write it down as never asked, and answer
    /// [`NotDone::NotAtThisLevel`].
    ///
    /// A call that does not form is turned away in its own words instead, and a
    /// refusal that could not be written down is answered as that.
    pub fn a_change_this_level_cannot_make(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        said: &Said,
        now: SystemTime,
    ) -> NotDone {
        if let Err(closed) = self.still_open() {
            return closed;
        }
        let call = match self.calling(verb, given, now) {
            Ok(call) => call,
            Err(turned_away) => return turned_away,
        };
        let strings = self.machine().strings();
        let entry = Entry::never_asked(&call, self.grantee(), said.text(), strings, now);
        if let Err(not_recorded) = self.writing_down(entry) {
            return not_recorded;
        }
        NotDone::NotAtThisLevel(said.clone())
    }

    /// **The person said yes to a change their level no longer lets run** —
    /// they moved to the sealed box while the question stood — so the approval
    /// is spent, nothing runs, and it is written down as stopped at the moment
    /// it would have run, as a revoked grant is (`CLAUDE.md`: *a revoked grant
    /// takes effect immediately*).
    pub fn a_waiting_change_this_level_cannot_make(
        &mut self,
        id: ProposalId,
        said: &Said,
        now: SystemTime,
    ) -> NotDone {
        if let Err(closed) = self.still_open() {
            return closed;
        }
        let Some(call) = self
            .proposed(id)
            .map(|waiting| waiting.proposal.call().clone())
        else {
            return NotDone::NotAnswered(AnswerError::NothingWaiting {
                number: id.as_u64(),
            });
        };
        if let Err(not_waiting) = self.approvals_mut().approve(id, now) {
            return NotDone::NotAnswered(not_waiting);
        }
        self.remembering(id, Became::Approved);
        let refused = Refused::worded_elsewhere(call, said.clone());
        match self.stopped_at_the_moment::<()>(refused, now) {
            Err(why) => why,
            Ok(()) => NotDone::NotAtThisLevel(said.clone()),
        }
    }
}
