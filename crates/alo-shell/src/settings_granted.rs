//! What has been granted to what: the machine's grants and the pairings a
//! person made, in one list, revoked the same way.
//!
//! `docs/features.md`, ★: *one list of what has been granted to what, revoked
//! the same way.* So this section holds **one list of `alo_changing::Row`** —
//! a grant's row is `alo_granted::Seen`, derived by `alo_granted::Listing`
//! from the grants the machine keeps; a pairing's row is
//! `alo_changing::SeenPairing`, made from the pairings the daemon wrote — and
//! it revokes any row with **the one call**, `alo_changing::Changing::revoked`,
//! answered with the one `alo_changing::Gone`. There is no second path for a
//! pairing here, and no way to revoke anything but a row a person saw.
//!
//! # Nothing is granted from here
//!
//! Making a grant is `alo-picking`: a person standing in a folder and choosing
//! it. This section lists and revokes. It has no *revoke everything*, and
//! nothing an agent asks reaches it.
//!
//! # Read once, and a list that did not read is not a list
//!
//! The grants are read through `alo_remembering::remembered` when Settings
//! opens and held as *the* list, which is what `alo_changing::Changing` has to
//! be handed. A grants file that is there and does not read is **not** drawn
//! as *nothing granted*, which would be a lie, and nothing is revoked from it;
//! its refusal goes to the host's log (`crate::SettingsOpened`). A pairing is
//! the daemon's to revoke, not the grants file's to write, so the pairings
//! stay revocable either way.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_capability::Grants;
use alo_changing::{Changing, Door, Gone, NotChanged, Row, SeenPairing};
use alo_granted::{Listing, Revoked};
use alo_nearby::{MachineId, MayAskIts, Pairings};
use alo_remembering::NotRemembered;
use alo_strings::Strings;

/// What revoking a row did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsRevoked {
    /// It is gone, as `alo-changing` answered.
    Gone(Gone),
    /// It was not revoked, and the section says why.
    NotRevoked,
    /// There is no row there to revoke.
    NoSuchRow,
}

/// The section, as it stands.
#[derive(Debug)]
pub(crate) struct GrantedSection {
    /// Where the machine keeps its grants.
    grants_at: PathBuf,
    /// The grants, when they read.
    grants: Option<Grants>,
    /// The pairings the daemon keeps, when they read.
    pairings: Option<Pairings>,
    /// The machines whose pairing the daemon revoked while Settings was open,
    /// which the file read at opening still names — a revocation *until a
    /// restart* is not written down — and which are revoked all the same.
    unpaired: Vec<MachineId>,
    /// What the section says about the last revocation.
    told: Vec<String>,
}

impl GrantedSection {
    /// The section at sign-in, and each file that did not read, for the
    /// host's log.
    pub(crate) fn at_sign_in(
        grants_at: &Path,
        pairings_at: &Path,
        now: SystemTime,
    ) -> (Self, Vec<NotRemembered>) {
        let mut refused = Vec::new();
        let grants = match alo_remembering::remembered(grants_at, now) {
            Ok(grants) => Some(grants),
            Err(NotRemembered::NotThere { .. }) => Some(Grants::default()),
            Err(why) => {
                refused.push(why);
                None
            }
        };
        let pairings = match alo_remembering::pairings_remembered(pairings_at, now) {
            Ok(pairings) => Some(pairings),
            Err(NotRemembered::NotThere { .. }) => Some(Pairings::none()),
            Err(why) => {
                refused.push(why);
                None
            }
        };
        (
            Self {
                grants_at: grants_at.to_owned(),
                grants,
                pairings,
                unpaired: Vec::new(),
                told: Vec::new(),
            },
            refused,
        )
    }

    /// The pairings in force: as read, less every one revoked since — none
    /// when they did not read.
    pub(crate) fn pairings(&self) -> Pairings {
        let mut in_force = Pairings::none();
        for pairing in self.pairings.iter().flat_map(Pairings::every) {
            if !self.unpaired.contains(pairing.with()) {
                in_force.keep(pairing.clone());
            }
        }
        in_force
    }

    /// The one list at `now`: every grant, in the order they were made, then
    /// every pairing.
    pub(crate) fn rows(&self, now: SystemTime) -> Vec<Row> {
        let mut rows: Vec<Row> = self
            .grants
            .as_ref()
            .map(|grants| {
                Listing::of(grants, now)
                    .rows()
                    .iter()
                    .cloned()
                    .map(Row::from)
                    .collect()
            })
            .unwrap_or_default();
        if let Some(pairings) = &self.pairings {
            rows.extend(
                pairings
                    .every()
                    .iter()
                    .filter(|pairing| {
                        pairings.paired_with(pairing.with(), now)
                            && !self.unpaired.contains(pairing.with())
                    })
                    .map(|pairing| Row::from(SeenPairing::of(pairing))),
            );
        }
        rows
    }

    /// What a pairing's row may do, as its pairing says.
    pub(crate) fn may(&self, row: &SeenPairing) -> Vec<MayAskIts> {
        self.pairings
            .as_ref()
            .and_then(|pairings| {
                pairings
                    .every()
                    .iter()
                    .find(|pairing| pairing.with() == row.machine())
            })
            .map(|pairing| pairing.may().to_vec())
            .unwrap_or_default()
    }

    /// Whether *nothing is granted* is true of the grants as read.
    pub(crate) fn nothing_granted(&self, now: SystemTime) -> Option<Listing> {
        self.grants
            .as_ref()
            .map(|grants| Listing::of(grants, now))
            .filter(Listing::is_nothing_granted)
    }

    /// What the section says about the last revocation.
    pub(crate) fn told(&self) -> &[String] {
        &self.told
    }

    /// The person revoked `row`, through `alo-changing`'s one call.
    pub(crate) fn revoked(
        &mut self,
        row: &Row,
        daemon: &dyn Door,
        now: SystemTime,
        strings: &Strings,
    ) -> SettingsRevoked {
        if !self.rows(now).contains(row) {
            return SettingsRevoked::NoSuchRow;
        }
        // A pairing's revocation writes nothing on this side, so a machine
        // whose grants did not read can still revoke one: the grants handed
        // over then are never written, because only a grant's row writes them
        // and no grant's row exists without the grants having read.
        let mut unread = Grants::default();
        let grants = match (&mut self.grants, row) {
            (Some(grants), _) => grants,
            (None, Row::Pairing(_)) => &mut unread,
            (None, Row::Grant(_)) => return SettingsRevoked::NoSuchRow,
        };
        let answered = Changing::of(grants, &self.grants_at, daemon).revoked(row, now);
        self.told = told(row, &answered, strings);
        match answered {
            Ok(gone) => {
                if let (Row::Pairing(seen), Gone::Revoked { .. }) = (row, &gone) {
                    self.unpaired.push(seen.machine().clone());
                }
                SettingsRevoked::Gone(gone)
            }
            Err(_) => SettingsRevoked::NotRevoked,
        }
    }
}

/// What the section says after a revocation: every sentence the crates have
/// for what became of it, in their words.
fn told(row: &Row, answered: &Result<Gone, NotChanged>, strings: &Strings) -> Vec<String> {
    let mut said = Vec::new();
    match answered {
        Ok(Gone::AlreadyGone) => said.push(Revoked::AlreadyGone.said(strings).into_text()),
        Ok(Gone::Revoked { stood }) => {
            if matches!(row, Row::Grant(_)) {
                said.push(Revoked::Now.said(strings).into_text());
            }
            said.extend(stood.explained(strings));
        }
        Err(refused) => said.push(refused.said(strings).into_text()),
    }
    said
}
