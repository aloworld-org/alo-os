//! A request judged against the grants a person made.
//!
//! [`Request::judged`] is a portal request evaluated the way `alo-capability`
//! evaluates a verb: against a grant naming the application and the reach,
//! refused outside it, never widened. It **decides nothing itself**. Every
//! answer is one of `alo_capability::Grants`' — [`Grants::allows_anything`]
//! first, then [`Grants::allowing`] for each thing the request is for — so
//! what a grant covers, when it has ended and whose it is are ruled on in one
//! crate, for agents and applications alike.
//!
//! # Against which list
//!
//! A machine's grants are `alo_capability::Agent::allowed`: the one list on a
//! machine with an agent, and the applications' grants alone on a machine
//! whose person declined it (ADR 0040, part 3). Handing this the list rather
//! than the machine is what lets a caller that holds only the list — the
//! daemon reads a `Grants` off the disk — judge with it.
//!
//! # Nothing is kept
//!
//! A judged request takes `&Grants`. It cannot add a grant, extend one or
//! remember an answer, so asking again after a revocation is answered by the
//! list as it now is: a revoked grant is refused at the next request.

use std::time::SystemTime;

use alo_capability::{Applicant, GrantId, Grants};

use crate::portal::Portal;
use crate::refused::Refused;
use crate::request::Request;

/// A request the grants allowed, and the grants that allowed it.
///
/// Carries every grant it was allowed by, in the order the request named what
/// it was for, because a record owes an answer to *against which grant* and
/// this is the one moment that answer exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Allowed {
    /// The application that asked.
    application: Applicant,
    /// The portal it asked.
    portal: Portal,
    /// The grant behind each thing it was for.
    against: Vec<GrantId>,
}

impl Allowed {
    /// The application that asked.
    #[must_use]
    pub const fn application(&self) -> &Applicant {
        &self.application
    }

    /// The portal it asked.
    #[must_use]
    pub const fn portal(&self) -> Portal {
        self.portal
    }

    /// The grant behind each thing the request was for — the handles a person
    /// revokes by.
    #[must_use]
    pub fn against(&self) -> &[GrantId] {
        &self.against
    }
}

impl Request {
    /// Whether the grants allow this request at this moment, and by which
    /// grants — or why not.
    ///
    /// # Errors
    ///
    /// [`Refused::NothingGranted`] for an application that holds no grant at
    /// all, decided before anything the request is for is looked at; and
    /// [`Refused::NotAllowed`], carrying `alo-capability`'s refusal, for the
    /// first thing it is for that no grant of its covers.
    pub fn judged(&self, grants: &Grants, now: SystemTime) -> Result<Allowed, Refused> {
        if !grants.allows_anything(self.application(), now) {
            return Err(Refused::NothingGranted {
                application: self.application().clone(),
                portal: self.portal(),
            });
        }
        let mut against = Vec::with_capacity(self.wanted().len());
        for ask in self.wanted() {
            let id = grants
                .allowing(self.application(), ask, now)
                .map_err(|why| Refused::NotAllowed {
                    portal: self.portal(),
                    why,
                })?;
            against.push(id);
        }
        Ok(Allowed {
            application: self.application().clone(),
            portal: self.portal(),
            against,
        })
    }
}
