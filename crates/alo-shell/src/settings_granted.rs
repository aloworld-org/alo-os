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
//! # And one grant is made from here, by a person picking a folder
//!
//! **This section said *nothing is granted from here* until 2026-10-24, and
//! that sentence was true and was the fault.** Making a grant is
//! `alo-picking` — a person standing in a folder and choosing it — and nothing
//! on the machine built a picker, so the list this section draws could only
//! ever say *nothing granted*, every file verb was refused forever, and the
//! refusal a person met named an act the interface did not have.
//!
//! So the one row that makes a grant lives here, beside the list it appears
//! in: `crate::a_folder_is_picked_in_settings` holds the walking and the two
//! questions, and [`GrantedSection::act_on`] is where a pick becomes
//! `alo_changing::Changing::granted`. There is still no *revoke everything*,
//! and **nothing an agent asks reaches any of it** — a grant comes from a
//! person pressing a key in this window and from nowhere else.
//!
//! # And one application, by name (ADR 0143 Q10)
//!
//! A second row beside it gives alo one application: chosen from the ones this
//! machine has, then how long — `crate::an_application_is_picked_in_settings`
//! holds the two steps. **It is the same road**: the application is sealed by
//! `alo-picking` as a folder is, and `GrantedSection::give_it` carries either
//! to the disk with the one `alo_changing::Changing::granted` call, so a grant
//! is still made by a person pressing a key in this window and from nowhere
//! else. *Added 2026-11-15, the owner's decision: reuse the terminal settings'
//! picker.*
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
use alo_changing::{Changing, Door, Gone, Made, NotChanged, Row, SeenPairing};
use alo_granted::{Listing, Revoked};
use alo_nearby::{MachineId, MayAskIts, Pairings};
use alo_picking::{Applications, Chosen, Folders};
use alo_remembering::NotRemembered;
use alo_strings::{Filling, Strings};

use crate::a_folder_is_picked_in_settings::{AFolderBeingPicked, PickingRow, WhatPickingDid};
use crate::an_application_is_picked_in_settings::{
    AnApplicationBeingPicked, ApplicationRow, WhatChoosingDid,
};
use crate::how_long_a_grant_lasts::HowLong;

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
    /// What the person called the machines they paired with, as the daemon
    /// keeps it beside the pairings. None named when the file is not there
    /// or did not read, so a machine is then shown by its identity.
    names: alo_remembering::MachineNames,
    /// The machines whose pairing the daemon revoked while Settings was open,
    /// which the file read at opening still names — a revocation *until a
    /// restart* is not written down — and which are revoked all the same.
    unpaired: Vec<MachineId>,
    /// What the section says about the last revocation.
    told: Vec<String>,
    /// The person's home folder, where a picker opens.
    home: Option<PathBuf>,
    /// The agent a folder would be given to, by the name the grants use.
    agent: Option<String>,
    /// A picker, while one is open.
    ///
    /// **Its presence is the mode**, which is
    /// `crate::a_provider_is_added_in_settings`' rule: there is no flag beside
    /// it that could disagree with what is on the screen.
    picking: Option<AFolderBeingPicked>,
    /// The applications this machine has, as Settings was last told — what a
    /// person may give alo one of. Nothing until it is told.
    applications: Applications,
    /// The list of applications, while one is open — its presence the mode,
    /// as [`Self::picking`]'s is. **Never both at once**: opening one closes
    /// the other.
    choosing: Option<AnApplicationBeingPicked>,
    /// The grants made on the sheet alo asks for an application on for *This
    /// task*, whose rows say *until this task ends* rather than *until you
    /// sign out* (`crate::the_application_sheet`).
    for_a_task: Vec<alo_capability::GrantId>,
}

/// What giving alo a folder did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsGave {
    /// The list changed and should be drawn again.
    ItChanged,
    /// Nothing happened.
    Nothing,
    /// alo was given the folder, which is now in the list below.
    ItWasGiven,
    /// It was not given, and the section says why.
    ItWasNotGiven,
}

impl GrantedSection {
    /// The section at sign-in, and each file that did not read, for the
    /// host's log.
    pub(crate) fn at_sign_in(
        grants_at: &Path,
        pairings_at: &Path,
        home: Option<&Path>,
        agent: Option<&str>,
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
        // **The names a person gave the machines they paired with**, which the
        // daemon keeps beside the pairings. Until 2026-11-01 Settings never
        // read them, so a paired computer was shown by its 32-character
        // identity in *alo and AI* and in the list of what alo can reach.
        let names = match &pairings {
            Some(pairings) => match alo_remembering::machine_names_remembered(
                &crate::settings_places::the_names_beside(pairings_at),
                pairings,
                now,
            ) {
                Ok(names) => names,
                Err(NotRemembered::NotThere { .. }) => alo_remembering::MachineNames::none(),
                Err(why) => {
                    refused.push(why);
                    alo_remembering::MachineNames::none()
                }
            },
            None => alo_remembering::MachineNames::none(),
        };
        (
            Self {
                grants_at: grants_at.to_owned(),
                grants,
                pairings,
                names,
                unpaired: Vec::new(),
                told: Vec::new(),
                home: home.map(Path::to_owned),
                agent: agent.map(str::to_owned),
                picking: None,
                applications: Applications::default(),
                choosing: None,
                for_a_task: Vec::new(),
            },
            refused,
        )
    }

    /// Whether a person can give alo a folder from here at all.
    ///
    /// Three things have to be true, and each absence is a different machine
    /// rather than a failure: the session has to have said where home is, the
    /// machine has to have said what its agent is called — a grant under any
    /// other name is one the daemon will never match — and the grants file has
    /// to have read, because a grant is added to the list that was read and a
    /// list that did not read would be written over with one grant in it.
    ///
    /// Where it is false the row is **absent** rather than drawn and refusing,
    /// which is `crate::settings_window`'s own rule: *a setting the machine
    /// has no road to change from a window yet is absent rather than greyed
    /// out*.
    pub(crate) const fn a_folder_can_be_given(&self) -> bool {
        self.home.is_some() && self.agent.is_some() && self.grants.is_some()
    }

    /// The picker, while one is open.
    pub(crate) const fn picking(&self) -> Option<&AFolderBeingPicked> {
        self.picking.as_ref()
    }

    /// **The grants were changed outside this window**, by the sheet alo asks
    /// for an application on: read them again, so the list shows what the
    /// file holds and a grant made here next is added to it rather than
    /// written over it. A file that will not read is no list, as at opening.
    pub(crate) fn the_grants_are_read_again(&mut self, now: SystemTime) {
        self.grants = match alo_remembering::remembered(&self.grants_at, now) {
            Ok(grants) => Some(grants),
            Err(NotRemembered::NotThere { .. }) => Some(Grants::default()),
            Err(_) => None,
        };
    }

    /// These grants are a task's, and end with it.
    pub(crate) fn these_are_a_tasks(&mut self, for_a_task: Vec<alo_capability::GrantId>) {
        self.for_a_task = for_a_task;
    }

    /// Whether this grant is a task's.
    pub(crate) fn is_for_a_task(&self, id: alo_capability::GrantId) -> bool {
        self.for_a_task.contains(&id)
    }

    /// The list of applications, while one is open.
    pub(crate) const fn choosing(&self) -> Option<&AnApplicationBeingPicked> {
        self.choosing.as_ref()
    }

    /// Whether either chooser is open, which is what Escape closes.
    pub(crate) const fn is_choosing_something(&self) -> bool {
        self.picking.is_some() || self.choosing.is_some()
    }

    /// The applications this machine has, as the seat last read them.
    ///
    /// **Each by the name its packager gave**, and by its identifier where it
    /// gave none or the name is one of this project's own words, which needs
    /// the vocabulary and is not a third party's to give.
    pub(crate) fn the_applications_are(&mut self, installed: &alo_applications::Installed) {
        self.applications = Applications::of(installed.all().map(|application| {
            let called = match application.name() {
                Some(alo_applications::Called::ByWhoeverPackagedIt(called)) => Some(called.clone()),
                Some(alo_applications::Called::InOurOwnWords(_)) | None => None,
            };
            (application.identifier().to_owned(), called)
        }));
    }

    /// Whether a person can give alo an application from here.
    ///
    /// [`Self::a_folder_can_be_given`]'s three conditions less the home
    /// folder, which an application does not need — and one more: there has
    /// to be an application to give. **Absent rather than empty**, for the
    /// same rule: a row that opens a list with nothing in it teaches somebody
    /// the list lies.
    pub(crate) fn an_application_can_be_given(&self) -> bool {
        self.agent.is_some() && self.grants.is_some() && !self.applications.is_empty()
    }

    /// A person asked to give alo an application: open the list.
    pub(crate) fn give_alo_an_application(&mut self) -> SettingsGave {
        if !self.an_application_can_be_given() {
            return SettingsGave::Nothing;
        }
        self.picking = None;
        self.choosing = Some(AnApplicationBeingPicked::among(self.applications.clone()));
        self.told.clear();
        SettingsGave::ItChanged
    }

    /// The person pressed Enter on one of the list's rows.
    pub(crate) fn act_on_an_application(
        &mut self,
        row: &ApplicationRow,
        daemon: &dyn Door,
        now: SystemTime,
        strings: &Strings,
    ) -> SettingsGave {
        let Some(choosing) = &mut self.choosing else {
            return SettingsGave::Nothing;
        };
        match choosing.act(row) {
            WhatChoosingDid::ItChanged => SettingsGave::ItChanged,
            WhatChoosingDid::Nothing => SettingsGave::Nothing,
            WhatChoosingDid::GiveItThisApplication { chosen, how_long } => {
                self.give_it(&chosen, how_long, daemon, now, strings)
            }
        }
    }

    /// A person asked to give alo a folder: open a picker in their home.
    ///
    /// A picker that will not open says so and leaves the section as it was —
    /// a home folder that is not there, or one this machine will not read, is
    /// `alo-picking`'s refusal and is shown in its words.
    pub(crate) fn give_alo_a_folder(
        &mut self,
        folders: &dyn Folders,
        strings: &Strings,
    ) -> SettingsGave {
        let Some(home) = self.home.clone() else {
            return SettingsGave::Nothing;
        };
        match AFolderBeingPicked::standing_in(&home, folders) {
            Ok(picking) => {
                self.choosing = None;
                self.picking = Some(picking);
                self.told.clear();
                SettingsGave::ItChanged
            }
            Err(why) => {
                self.told = vec![why.said(strings).into_text()];
                SettingsGave::ItWasNotGiven
            }
        }
    }

    /// Stop picking without giving anything away — a folder or an
    /// application, whichever was open.
    pub(crate) fn that_folder_is_not_being_given(&mut self) {
        self.picking = None;
        self.choosing = None;
    }

    /// The person pressed Enter on one of the picker's rows.
    ///
    /// Every move is the picker's; the only thing that happens here is the
    /// grant, at the moment they say how long.
    pub(crate) fn act_on(
        &mut self,
        row: &PickingRow,
        folders: &dyn Folders,
        daemon: &dyn Door,
        now: SystemTime,
        strings: &Strings,
    ) -> SettingsGave {
        let Some(picking) = &mut self.picking else {
            return SettingsGave::Nothing;
        };
        match picking.act(row, folders, strings) {
            WhatPickingDid::ItChanged => SettingsGave::ItChanged,
            WhatPickingDid::Nothing => SettingsGave::Nothing,
            WhatPickingDid::GiveItThisFolder { chosen, how_long } => {
                self.give_it(&chosen, how_long, daemon, now, strings)
            }
        }
    }

    /// Make the grant the person just asked for — a folder or an application
    /// — and close whichever chooser it came from.
    ///
    /// **The picker closes whichever way this goes.** A person who has
    /// answered both questions has finished asking; leaving them inside a
    /// folder chooser with a sentence under it would make them wonder whether
    /// to answer again.
    fn give_it(
        &mut self,
        chosen: &Chosen,
        how_long: HowLong,
        daemon: &dyn Door,
        now: SystemTime,
        strings: &Strings,
    ) -> SettingsGave {
        self.picking = None;
        self.choosing = None;
        let (Some(grants), Some(agent)) = (&mut self.grants, self.agent.clone()) else {
            // Unreachable: `a_folder_can_be_given` is what put the row there.
            // An arm rather than an unwrap, because a window that panicked on
            // a key press is a machine nobody can use.
            return SettingsGave::Nothing;
        };
        // **The length decides which kind of grant it is**, in
        // `HowLong::granting_to` and nowhere else: this call site held a
        // duration until 2026-10-25, and a duration has no value that means
        // *until the person removes it*.
        let granting = how_long.granting_to(&agent);
        match Changing::of(grants, &self.grants_at, daemon).granted(&granting, chosen, now) {
            Ok(Made::Granted { .. }) => {
                let folder = chosen.folder().map(|folder| {
                    strings.say(
                        &crate::the_pickers_words::IT_WAS_GIVEN.key(),
                        &Filling::of(
                            crate::the_pickers_words::THE_FOLDER,
                            folder.display().to_string(),
                        ),
                    )
                });
                let application = chosen.application().map(|application| {
                    strings.say(
                        &crate::the_pickers_words::IT_WAS_GIVEN_AN_APPLICATION.key(),
                        &Filling::of(
                            crate::the_pickers_words::THE_APPLICATION,
                            application.called().to_owned(),
                        ),
                    )
                });
                self.told = folder
                    .into_iter()
                    .chain(application)
                    .map(alo_strings::Said::into_text)
                    .collect();
                SettingsGave::ItWasGiven
            }
            // The picker was closed without picking, which cannot arrive here
            // — both choosers hand over only a folder or an application.
            Ok(Made::Nothing) => SettingsGave::Nothing,
            Err(why) => {
                self.told = vec![why.said(strings).into_text()];
                SettingsGave::ItWasNotGiven
            }
        }
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
                    .map(|pairing| Row::from(SeenPairing::named(pairing, &self.names))),
            );
        }
        rows
    }

    /// **What a paired machine is shown as**: the name the person gave it
    /// when they did, and its identity when they did not — which is how
    /// pairing itself names a machine (`alo_nearby::Origin`).
    pub(crate) fn what_a_machine_is_called(&self, identity: &str) -> String {
        MachineId::read(identity)
            .ok()
            .and_then(|machine| {
                self.names
                    .called(&machine)
                    .map(|name| name.as_str().to_owned())
            })
            .unwrap_or_else(|| identity.to_owned())
    }

    /// **What an application is shown as** in the list of what alo was given:
    /// the name it was listed under, and its identifier where it is not
    /// listed or gave none.
    pub(crate) fn what_an_application_is_called(&self, identifier: &str) -> String {
        self.applications
            .listed()
            .iter()
            .find(|listed| listed.identifier() == identifier)
            .map_or_else(
                || identifier.to_owned(),
                |listed| listed.called().to_owned(),
            )
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
