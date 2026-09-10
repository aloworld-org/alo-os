//! The pick, become a grant the daemon honours.
//!
//! This is the join the plan's task is named for: `alo-capability` has grants
//! and `alo-agentd` enforces them, and until this file nothing on the machine
//! could make one. It is four lines of decision and a great deal of care about
//! what it must **not** be able to do.
//!
//! # What it cannot do, by construction
//!
//! **It cannot grant anything from a pick that did not happen.**
//! [`Granting::of`] takes a [`Chosen`], whose only other case is
//! [`Chosen::Nothing`], and the folder inside the other case is a
//! [`crate::Picked`] — sealed, with no constructor outside this crate. So
//! *picking nothing grants nothing* is not a branch somebody remembered to
//! write; there is no folder in that value to grant.
//!
//! **It cannot widen what was picked.** The reach handed to
//! `alo_capability::Grant::checked` is [`crate::Picked::reach`], which is the
//! folder itself. Nothing here looks at a parent, and nothing here has a path
//! of its own.
//!
//! **It cannot make a grant that does not end.** The duration is this type's
//! and is required to make one at all, so a caller cannot omit it and get a
//! default that outlives the reason the folder was picked. `Grant::checked`
//! refuses a zero one, in its own words.
//!
//! # It does not read the clock
//!
//! `now` is passed in, as everywhere else in this repository: what a person
//! was shown and what the grants say must not be able to disagree about when
//! something was true.

use std::time::{Duration, SystemTime};

use alo_capability::{Grant, GrantError, GrantId, Grants};

use crate::picked::Chosen;

/// Who a pick grants to, and for how long.
///
/// Held by whatever opens the picker: the agent a person is granting to is a
/// fact about the surface they opened it from, not something to be decided
/// once a folder is in hand. Two arguments together rather than two arguments
/// at the end of [`Granting::of`], because they are the same decision — *this
/// agent, this long* — and a call site that could pass one without the other
/// is a call site that can pass yesterday's agent with today's duration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Granting {
    /// The agent, by the name the system knows it by.
    grantee: String,
    /// How long a grant made from a pick lasts.
    lasting: Duration,
}

impl Granting {
    /// Grants to this agent, lasting this long.
    #[must_use]
    pub fn to(grantee: &str, lasting: Duration) -> Self {
        Self {
            grantee: grantee.to_owned(),
            lasting,
        }
    }

    /// The agent a pick would be granted to.
    #[must_use]
    pub fn grantee(&self) -> &str {
        &self.grantee
    }

    /// How long a grant made from a pick would last.
    #[must_use]
    pub fn lasting(&self) -> Duration {
        self.lasting
    }

    /// Make the grant this pick means, and add it to the machine's grants.
    ///
    /// Answers with the handle the person revokes it by — the same handle a
    /// list of grants shows — or [`None`] when the picker was closed without
    /// picking, in which case **`grants` is not touched at all**.
    ///
    /// # Errors
    /// `alo_capability::GrantError`, in that crate's own words. It cannot
    /// arrive from the folder — a picked folder is rooted, free of `..` and
    /// never the top of the disk, because [`crate::Picker`] refused all three
    /// before there was anything to grant — so what is left is this type's own
    /// two: an agent with no name, and a duration of zero or of no end.
    pub fn of(
        &self,
        chosen: &Chosen,
        grants: &mut Grants,
        now: SystemTime,
    ) -> Result<Option<GrantId>, GrantError> {
        let Chosen::Folder(picked) = chosen else {
            return Ok(None);
        };
        let grant = Grant::checked(&self.grantee, picked.reach(), now, self.lasting)?;
        Ok(Some(grants.grant(grant)))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{WrittenDown, in_english};
    use crate::{NotPicked, Picker};
    use alo_capability::{Ask, Grantee};
    use std::path::Path;

    /// The agent every test here grants to.
    const HERS: &str = "@files";

    /// A moment, and an hour after it.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// How long a grant made in these tests lasts.
    fn hour() -> Duration {
        Duration::from_secs(60 * 60)
    }

    /// The ordinary grant these tests make.
    fn for_an_hour() -> Granting {
        Granting::to(HERS, hour())
    }

    /// A person who walked into her invoices folder and picked it.
    fn a_folder_picked(machine: &WrittenDown) -> Chosen {
        let mut picker = Picker::standing_in(Path::new("/home/anna"), machine).unwrap();
        picker.go_into("Invoices", machine).unwrap();
        picker.pick().unwrap()
    }

    /// **A pick becomes a grant the daemon's own question honours.** Not a
    /// grant this crate believes in — the one `alo_capability::Grants::permits`
    /// answers with, which is the question `alo-agentd` asks of every verb.
    #[test]
    fn a_pick_becomes_a_grant_that_permits_what_was_picked() {
        let machine = WrittenDown::a_home_folder();
        let mut grants = Grants::default();
        assert!(grants.is_empty());

        let id = for_an_hour()
            .of(&a_folder_picked(&machine), &mut grants, noon())
            .unwrap();
        assert!(id.is_some());
        assert_eq!(grants.len(), 1);
        assert!(grants.permits(
            &Grantee::named(HERS),
            &Ask::path("/home/anna/Invoices/march.pdf"),
            noon()
        ));
    }

    /// **Picking nothing grants nothing**, and leaves the list exactly as it
    /// was — not an empty grant, not a grant over where the picker happened to
    /// be standing.
    #[test]
    fn picking_nothing_grants_nothing_at_all() {
        let mut grants = Grants::default();
        let nothing = for_an_hour()
            .of(&Chosen::Nothing, &mut grants, noon())
            .unwrap();
        assert_eq!(nothing, None);
        assert!(grants.is_empty());
        assert!(!grants.permits(&Grantee::named(HERS), &Ask::path("/home/anna"), noon()));
    }

    /// **The grant ends**, because this type cannot make one that does not:
    /// the hour it was given is the hour it lasts, and the folder is reachable
    /// on neither side of it.
    #[test]
    fn the_grant_a_pick_makes_ends_when_it_was_always_going_to() {
        let machine = WrittenDown::a_home_folder();
        let mut grants = Grants::default();
        for_an_hour()
            .of(&a_folder_picked(&machine), &mut grants, noon())
            .unwrap();

        let ask = Ask::path("/home/anna/Invoices/march.pdf");
        let hers = Grantee::named(HERS);
        assert!(grants.permits(&hers, &ask, noon()));
        assert!(!grants.permits(&hers, &ask, noon() + hour()));
    }

    /// **A grant that would never end is refused**, in `alo-capability`'s own
    /// words rather than in this crate's — the words a person reads about a
    /// grant belong to the crate that decides what a grant is.
    #[test]
    fn a_grant_with_no_time_or_no_end_is_refused_in_the_grants_own_words() {
        let machine = WrittenDown::a_home_folder();
        let chosen = a_folder_picked(&machine);
        let mut grants = Grants::default();

        let none = Granting::to(HERS, Duration::ZERO).of(&chosen, &mut grants, noon());
        assert_eq!(none, Err(GrantError::NoTime));
        let forever = Granting::to(HERS, Duration::MAX).of(&chosen, &mut grants, noon());
        assert_eq!(forever, Err(GrantError::NoEnd));
        assert!(grants.is_empty(), "a refused grant was added anyway");

        let said = GrantError::NoTime.said(&in_english());
        assert!(!said.is_a_bug(), "the grants' own refusal is not declared");
    }

    /// **A grant to nobody is refused**, and nothing is added to the list.
    #[test]
    fn a_grant_to_no_agent_is_refused() {
        let machine = WrittenDown::a_home_folder();
        let mut grants = Grants::default();
        let refused =
            Granting::to("   ", hour()).of(&a_folder_picked(&machine), &mut grants, noon());
        assert_eq!(refused, Err(GrantError::Anonymous));
        assert!(grants.is_empty());
    }

    /// **The whole machine never reaches this file.** The picker refuses it,
    /// so there is no [`Chosen`] to hand over — which is why the refusal below
    /// is a `NotPicked` and not a `GrantError`.
    #[test]
    fn the_whole_machine_is_refused_before_anything_can_be_granted() {
        let machine = WrittenDown::a_home_folder();
        let picker = Picker::standing_in(Path::new("/"), &machine).unwrap();
        assert_eq!(picker.pick(), Err(NotPicked::TheWholeMachine));
    }

    /// What the granting is for reads back, so a surface can show the person
    /// who they are granting to and for how long before they pick.
    #[test]
    fn a_granting_says_who_it_is_for_and_how_long_it_lasts() {
        let granting = for_an_hour();
        assert_eq!(granting.grantee(), HERS);
        assert_eq!(granting.lasting(), hour());
    }

    /// Two picks of one folder are one grant rather than two rows, which is
    /// `alo_capability::Grants::grant`'s rule seen from the picker: a person
    /// who picks the same folder twice has one thing to revoke.
    #[test]
    fn picking_the_same_folder_twice_leaves_one_grant() {
        let machine = WrittenDown::a_home_folder();
        let mut grants = Grants::default();
        let granting = for_an_hour();
        granting
            .of(&a_folder_picked(&machine), &mut grants, noon())
            .unwrap();
        granting
            .of(&a_folder_picked(&machine), &mut grants, noon())
            .unwrap();
        assert_eq!(grants.len(), 1);
    }
}
