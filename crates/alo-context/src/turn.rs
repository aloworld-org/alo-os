//! The one turn a context is good for, and the single grant it can make.
//!
//! # What a context grants, and what it does not
//!
//! **The document, and nothing else.** ADR 0001 §3 names two deliberate acts
//! that make a grant — a folder chosen in a picker, and the document offered at
//! invocation — and a context carries exactly one of them. The window in front
//! of somebody is something they were looking at and the selection is text they
//! had highlighted; neither is a decision to hand anything over, and neither
//! widens what an agent may reach by so much as a byte. There is a test in this
//! file that offers all three at once and asserts the list holds one grant.
//!
//! This is the part a reader is most likely to expect the other way round, and
//! getting it wrong would be the quiet kind of mistake: an agent that could
//! close whatever happened to be in front of a person, because they had once
//! pressed the key while it was.
//!
//! # A grant a context made is a grant like any other
//!
//! It goes into the machine's own [`Grants`], not into a list of this crate's.
//! ADR 0001 §3 says grants are enumerated, visible where the person can find
//! them, revocable in one action and expiring — and a grant kept somewhere else
//! would be authority that satisfies none of those while still deciding what an
//! agent may touch. So a person sees this one in the same list as the folder
//! they picked on Monday, and revokes it the same way.
//!
//! # It ends twice over, and both are on purpose
//!
//! **It expires.** The grant is made for the length of the turn, at the moment
//! of the invocation, so nothing has to remember to remove it: a daemon that
//! forgets a turn entirely still has an agent that can reach nothing when the
//! turn is over.
//!
//! **And it is revoked.** [`Turn::ending`] takes `self` and takes the grant
//! back out of the list, so a turn that finishes early does not leave the
//! document reachable for the rest of its allotted time. A turn cannot be ended
//! twice, and that is the compiler's job rather than a check — see the
//! `compile_fail` doctest below.

use std::time::{Duration, SystemTime};

use alo_capability::{Grant, GrantError, GrantId, Grantee, Grants};

use crate::context::Context;

/// One turn, and the authority the context offered at its invocation created.
///
/// Deliberately not `Clone`: a turn that can be copied is a turn that can be
/// ended twice, and the second ending would revoke a grant that the first had
/// already given back — on a list where the handle may since have been handed
/// to something else.
#[derive(Debug)]
pub struct Turn {
    /// What was offered at the invocation this turn began at.
    context: Context,
    /// The agent this turn is for.
    grantee: Grantee,
    /// The grant the offered document made, and the handle it went into the
    /// machine's list under. `None` when no document was offered.
    granted: Option<(GrantId, Grant)>,
    /// When the turn is over. The moment the grant expires, and the moment
    /// after which nothing offered here reaches anything.
    ends: SystemTime,
}

impl Turn {
    /// Begin a turn from what an invocation offered.
    ///
    /// The grant, if the person had a document open, goes into `grants` — the
    /// machine's own list, so that it is visible and revocable beside every
    /// other grant. It runs from the moment of the invocation for `lasting`,
    /// and the moment is the context's own rather than a fresh reading of a
    /// clock: two readings would be two answers that could disagree about when
    /// the turn ends.
    ///
    /// # Errors
    /// [`GrantError`], carried whole from `alo-capability` rather than reworded
    /// here — item 9f's rule, so the words a person reads about a grant that
    /// could not be made are the grants' own wherever it happened.
    ///
    /// # Examples
    ///
    /// The document a person had open is reachable for the turn, and what was
    /// merely in front of them is not:
    ///
    /// ```
    /// use alo_capability::{Ask, Grants};
    /// use alo_context::{Context, Document, Focused, Turn};
    /// use std::time::{Duration, SystemTime};
    ///
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let context = Context::at_invocation(now)
    ///     .and_document(Document::open("/home/anna/Invoices/march.pdf").unwrap())
    ///     .and_window(Focused::window("org.blender.Blender").unwrap());
    ///
    /// let mut grants = Grants::default();
    /// let turn = Turn::beginning(context, "@files", Duration::from_secs(300), &mut grants).unwrap();
    ///
    /// let agent = turn.grantee().clone();
    /// assert!(grants.permits(&agent, &Ask::path("/home/anna/Invoices/march.pdf"), now));
    /// assert!(!grants.permits(&agent, &Ask::application("org.blender.Blender"), now));
    ///
    /// // And when the turn ends the grant goes with it.
    /// assert!(turn.ending(&mut grants));
    /// assert!(!grants.permits(&agent, &Ask::path("/home/anna/Invoices/march.pdf"), now));
    /// ```
    ///
    /// One invocation is one turn. A context is taken by value, so a second
    /// turn from the same offer is not a program that compiles — which is ADR
    /// 0001 §4's *and only for that turn*, checked by the compiler:
    ///
    /// ```compile_fail
    /// use alo_capability::Grants;
    /// use alo_context::{Context, Turn};
    /// use std::time::{Duration, SystemTime};
    ///
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let context = Context::at_invocation(now);
    /// let mut grants = Grants::default();
    ///
    /// let first = Turn::beginning(context, "@files", Duration::from_secs(300), &mut grants).unwrap();
    /// let again = Turn::beginning(context, "@files", Duration::from_secs(300), &mut grants).unwrap();
    /// ```
    pub fn beginning(
        context: Context,
        grantee: &str,
        lasting: Duration,
        grants: &mut Grants,
    ) -> Result<Self, GrantError> {
        let named = Grantee::named(grantee);
        // Both of these are checked even when there is no document to grant, so
        // that a turn is the same thing whether or not somebody had a file
        // open. A turn belonging to nobody, or lasting no time, is not a turn
        // that happens to grant nothing — it is not a turn.
        if named.as_str().is_empty() {
            return Err(GrantError::Anonymous);
        }
        if lasting.is_zero() {
            return Err(GrantError::NoTime);
        }
        let ends = context.at().checked_add(lasting).ok_or(GrantError::NoEnd)?;
        let granted = match context.document() {
            None => None,
            Some(document) => {
                let grant = Grant::checked(grantee, document.reach(), context.at(), lasting)?;
                Some((grants.grant(grant.clone()), grant))
            }
        };
        Ok(Self {
            context,
            grantee: named,
            granted,
            ends,
        })
    }

    /// What was offered at the invocation this turn began at.
    #[must_use]
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// The agent this turn is for.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        &self.grantee
    }

    /// The handle the grant this context made went into the machine's list
    /// under, and nothing when no document was offered.
    ///
    /// A person revokes it with this, exactly as they revoke a folder they
    /// picked.
    #[must_use]
    pub fn granted(&self) -> Option<GrantId> {
        self.granted.as_ref().map(|(id, _)| *id)
    }

    /// When the turn is over.
    #[must_use]
    pub fn ends(&self) -> SystemTime {
        self.ends
    }

    /// End the turn, taking the grant it made back out of the list.
    ///
    /// Answers whether a grant was taken away — `false` when there was no
    /// document to grant, and when the person has already revoked it
    /// themselves.
    ///
    /// **It revokes only the grant this turn made.** The handle is checked
    /// against the grant it was given for before anything is taken away, so
    /// ending a turn against a list it did not begin on removes nothing:
    /// handles are unique to one list, and revoking by a number alone would
    /// mean a turn on one machine's list could take away an unrelated grant on
    /// another's.
    ///
    /// # Examples
    ///
    /// A turn cannot be ended twice, because ending it consumes it:
    ///
    /// ```compile_fail
    /// use alo_capability::Grants;
    /// use alo_context::{Context, Turn};
    /// use std::time::{Duration, SystemTime};
    ///
    /// let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    /// let mut grants = Grants::default();
    /// let turn = Turn::beginning(
    ///     Context::at_invocation(now),
    ///     "@files",
    ///     Duration::from_secs(300),
    ///     &mut grants,
    /// )
    /// .unwrap();
    ///
    /// turn.ending(&mut grants);
    /// turn.ending(&mut grants);
    /// ```
    #[must_use]
    pub fn ending(self, grants: &mut Grants) -> bool {
        let Some((id, grant)) = self.granted else {
            return false;
        };
        let ours = grants
            .active_at(self.context.at())
            .any(|held| held.id == id && held.grant == grant);
        ours && grants.revoke(id)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{everything_offered, hour, march, noon};
    use crate::{Context, Document, Focused, Selection};
    use alo_capability::Ask;

    /// **The document is the only thing a context grants.** Everything is
    /// offered here — a window, a selection and a document — and the list ends
    /// up with one grant in it.
    #[test]
    fn the_document_offered_is_the_only_thing_that_grants_anything() {
        let mut grants = Grants::default();
        let turn = Turn::beginning(everything_offered(), "@files", hour(), &mut grants).unwrap();

        assert_eq!(grants.len(), 1);
        assert!(turn.granted().is_some());
        let agent = turn.grantee().clone();
        assert!(grants.permits(&agent, &Ask::path(march()), noon()));
        assert!(!grants.permits(&agent, &Ask::application("org.blender.Blender"), noon()));
    }

    /// **A window in front of somebody is not a grant over the application.**
    /// The mistake this crate exists to not make: an agent that could close
    /// whatever a person happened to be looking at when they pressed the key.
    #[test]
    fn the_window_in_front_of_somebody_grants_nothing() {
        let mut grants = Grants::default();
        let context = Context::at_invocation(noon())
            .and_window(Focused::titled("org.blender.Blender", "untitled.blend").unwrap());
        let turn = Turn::beginning(context, "@files", hour(), &mut grants).unwrap();

        assert!(turn.granted().is_none());
        assert!(grants.is_empty());
        assert!(!grants.permits(
            turn.grantee(),
            &Ask::application("org.blender.Blender"),
            noon()
        ));
        assert!(!turn.ending(&mut grants));
    }

    /// **A selection is text, whatever it says.** One that reads like a path is
    /// still text, and offering it reaches nothing — which is the shape of the
    /// attack this rule stops, since a selection is the part of a context most
    /// likely to have been written by somebody else.
    #[test]
    fn a_selection_that_reads_like_a_path_grants_nothing() {
        let mut grants = Grants::default();
        let context = Context::at_invocation(noon())
            .and_selection(Selection::of("/etc/shadow\n/home/anna/.ssh/id_ed25519").unwrap());
        let turn = Turn::beginning(context, "@files", hour(), &mut grants).unwrap();

        assert!(grants.is_empty());
        assert!(!grants.permits(turn.grantee(), &Ask::path("/etc/shadow"), noon()));
    }

    /// **A grant a context made expires with the turn**, whether or not
    /// anything remembers to end it. The moment it runs from is the
    /// invocation's, so this is arithmetic rather than a wait.
    #[test]
    fn the_grant_a_context_made_dies_when_the_turn_does() {
        let mut grants = Grants::default();
        let turn = Turn::beginning(everything_offered(), "@files", hour(), &mut grants).unwrap();
        assert_eq!(turn.ends(), noon() + hour());

        let agent = turn.grantee().clone();
        assert!(grants.permits(&agent, &Ask::path(march()), noon()));
        assert!(grants.permits(
            &agent,
            &Ask::path(march()),
            noon() + hour() - Duration::from_secs(1)
        ));
        assert!(!grants.permits(&agent, &Ask::path(march()), turn.ends()));
    }

    /// **And it is taken out of the list when the turn ends**, so a turn that
    /// finishes in ten seconds does not leave the document reachable for the
    /// rest of the hour.
    #[test]
    fn ending_a_turn_takes_the_grant_back_out() {
        let mut grants = Grants::default();
        let turn = Turn::beginning(everything_offered(), "@files", hour(), &mut grants).unwrap();
        let agent = turn.grantee().clone();

        assert!(turn.ending(&mut grants));
        assert!(grants.is_empty());
        assert!(!grants.permits(&agent, &Ask::path(march()), noon()));
    }

    /// **A grant a context made is a grant like any other**: in the list the
    /// person reads, held by the agent it was made for, and revocable in one
    /// action by the handle the turn hands over.
    #[test]
    fn a_person_sees_it_beside_the_folder_they_picked_and_revokes_it_the_same_way() {
        let mut grants = Grants::default();
        let turn = Turn::beginning(everything_offered(), "@files", hour(), &mut grants).unwrap();
        let agent = turn.grantee().clone();

        assert_eq!(grants.active_at(noon()).count(), 1);
        assert_eq!(grants.held_by(&agent, noon()).count(), 1);

        let id = turn.granted().unwrap();
        assert!(grants.revoke(id));
        assert!(!grants.permits(&agent, &Ask::path(march()), noon()));
        // And ending the turn afterwards takes nothing away that is not there.
        assert!(!turn.ending(&mut grants));
    }

    /// **A turn ends only its own grant.** Handles are unique to one list, so
    /// ending a turn against a list it did not begin on must remove nothing —
    /// otherwise a number alone could take away a grant somebody else made.
    #[test]
    fn ending_a_turn_against_another_list_takes_nothing_away_from_it() {
        let mut ours = Grants::default();
        let turn = Turn::beginning(everything_offered(), "@files", hour(), &mut ours).unwrap();

        let mut somebody_elses = Grants::default();
        let theirs = somebody_elses.grant(
            Grant::checked(
                "@mail",
                alo_capability::Reach::Folder("/home/bo/Mail".into()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        assert_eq!(theirs, turn.granted().unwrap());

        assert!(!turn.ending(&mut somebody_elses));
        assert_eq!(somebody_elses.len(), 1);
    }

    /// A turn belonging to nobody, or lasting no time, is not a turn — and it
    /// is refused whether or not there was a document to grant, so that a turn
    /// is the same thing either way.
    #[test]
    fn a_turn_has_an_agent_and_an_end() {
        let mut grants = Grants::default();
        for empty_handed in [true, false] {
            let context = || {
                if empty_handed {
                    Context::at_invocation(noon())
                } else {
                    Context::at_invocation(noon()).and_document(Document::open(march()).unwrap())
                }
            };
            assert_eq!(
                Turn::beginning(context(), "  ", hour(), &mut grants).unwrap_err(),
                GrantError::Anonymous
            );
            assert_eq!(
                Turn::beginning(context(), "@files", Duration::ZERO, &mut grants).unwrap_err(),
                GrantError::NoTime
            );
            assert_eq!(
                Turn::beginning(context(), "@files", Duration::MAX, &mut grants).unwrap_err(),
                GrantError::NoEnd
            );
        }
        assert!(grants.is_empty(), "a refused turn granted something");
    }

    /// What was offered is still readable through the turn, because the shell
    /// shows it while the agent is working rather than only at the moment the
    /// key was pressed.
    #[test]
    fn a_turn_still_knows_what_was_offered_at_its_invocation() {
        let mut grants = Grants::default();
        let turn = Turn::beginning(everything_offered(), "@files", hour(), &mut grants).unwrap();
        assert_eq!(turn.context().at(), noon());
        assert_eq!(turn.context().document().map(Document::path), Some(march()));
        assert!(!turn.context().is_empty());
    }
}
