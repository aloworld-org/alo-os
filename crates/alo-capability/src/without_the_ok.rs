//! **A change carried out without the OK, because the person chose full
//! access** ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! The decision, as the owner accepted it: *"**At full access, alo can do
//! anything the person's own account can do**, without asking: read, create,
//! change and delete any file the account can, run any command, start, drive
//! and close any application."* And `CLAUDE.md`'s standing rule *Reads answer,
//! changes wait*, read to its end: *an approval is not a session at the default
//! setting, and a person may choose otherwise: ADR 0064's third level is the
//! agent runs code directly on the machine, without asking.*
//!
//! # Why this is a door of its own and not an approval the machine gives itself
//!
//! The obvious build is to propose the change and approve it on the person's
//! behalf. It would work, and it would write a lie into the record: *from
//! approval 7* about a change nobody approved. ADR 0001 §7 asks *from which
//! approval*, and at full access the true answer is **none, because the person
//! chose that nobody be asked** — so this authority carries no approval and
//! says why ([`Authorised::without_the_ok`]).
//!
//! # It opens only on a list that says full access
//!
//! The list is the one the service made for this moment
//! (`Grants::at_full_access_for`), and the service makes that only when the
//! person's own file says full access when the change is asked for. On any
//! other list — *ask me each time*, the sealed box, nothing chosen, a file
//! that would not read, another agent, an application — this refuses as the
//! read door refuses a change, [`NotAuthorised::ChangeWaits`]: the change waits
//! for one OK, as it always did. That is what keeps *one approval, exactly one
//! execution* true at every level that asks.
//!
//! And it asks the grants as every other door does, at this moment: full
//! access widens what a call may reach, never what one call is. A facility, a
//! terminal the person has not allowed, a machine whose person declined the
//! agent — none of those is reached here, because none is reached by full
//! access at all (`crate::full_access`).
//!
//! # A read is a read
//!
//! A read offered here is authorised as [`Authorised::read`] authorises it —
//! it needs no OK at any level, so there is nothing to carry out without one.

use std::time::SystemTime;

use crate::authorised::{Authorised, NotAuthorised, Refused};
use crate::call::Call;
use crate::grant::Grantee;
use crate::grants::Grants;

impl Authorised {
    /// Authorise a change **with no approval**, because `grants` is a list
    /// made at full access for `grantee` (ADR 0149).
    ///
    /// The call is borrowed, so a refusal leaves it with the caller to record.
    ///
    /// # Errors
    /// [`Refused`], carrying the call: [`NotAuthorised::ChangeWaits`] when
    /// `grants` does not give `grantee` full access — the change waits for the
    /// person's OK — and otherwise the grants' own refusal.
    pub fn at_full_access(
        call: &Call,
        grantee: &Grantee,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Self, Refused> {
        if !call.waits_for_approval() {
            return Self::read(call, grantee, grants, now);
        }
        if !grants.has_full_access(grantee) {
            return Err(Refused::changes_wait(call));
        }
        Self::granted(call.clone(), grantee.clone(), None, grants, now)
            .map(Self::carried_out_without_the_ok)
    }
}

impl Refused {
    /// The read door's refusal of a change, for a door that may not carry one
    /// out without the OK.
    pub(crate) fn changes_wait(call: &Call) -> Self {
        Self::of(
            call.clone(),
            NotAuthorised::ChangeWaits {
                verb: call.verb().to_owned(),
            },
        )
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "without_the_ok_tests.rs"]
mod tests;
