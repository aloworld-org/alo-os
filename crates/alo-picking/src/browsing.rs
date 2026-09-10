//! Where a person is standing while they pick, and everything they may do
//! from there.
//!
//! [`Picker`] is the whole of native folder selection as a decision: a folder
//! it stands in, the folders it shows, three moves — in, up, and pick — and a
//! refusal in words for every way each of them can fail. It draws nothing.
//! What the picker *looks like* is the compositor's, exactly as
//! `alo_overlay::Summoning` leaves the overlay's appearance to whatever owns
//! the screen; what is here is what a shell must not be free to decide
//! differently.
//!
//! # It walks; it does not jump
//!
//! A picker is opened at one folder and reaches every other one by opening
//! something it is already showing. There is no method that takes a path, and
//! that is the rule the rest of the crate rests on: a person cannot be shown a
//! folder they did not navigate to, so a shell cannot arrange — deliberately
//! or by a bug — for a person to grant something they never opened. The one
//! path this crate accepts from outside is where the picker starts, and it is
//! checked against the same two rules `alo_capability::path` compares a grant
//! by.
//!
//! # Standing at the top of the disk is allowed; picking it is not
//!
//! Going up from `/home` reaches `/`, because a person navigating upwards
//! should not hit an invisible floor and be unable to reach a sibling folder.
//! Picking there is refused ([`crate::NotPicked::TheWholeMachine`]), which is
//! ADR 0001 §3 — *there is no grant to `/`* — met at the surface, in a
//! sentence, rather than three layers down as a `GrantError` a shell might
//! render as *something went wrong*. `alo-capability` refuses it as well, and
//! that duplication is deliberate: this one is what a person reads, and that
//! one is what makes it true.
//!
//! # A move that fails leaves the person where they were
//!
//! Every move here is *look first, then move*. A folder that went away between
//! being listed and being opened refuses the move and changes nothing, so a
//! picker never ends up standing somewhere it cannot show — which would be a
//! surface with no rows, no explanation and no way back.

use std::path::{Path, PathBuf};

use alo_capability::path::{is_a_root, is_usable};
use alo_strings::{Filling, Said, Strings};

use crate::folders::{Folders, Inside};
use crate::picked::{Chosen, Picked};
use crate::refusing::NotPicked;
use crate::words;

/// A person picking a folder.
///
/// One per open picker. It holds where it is standing and what is in that
/// folder, and it is handed the [`Folders`] to ask at the moment of each move
/// rather than holding one — the same shape as `alo_overlay::Summoning` taking
/// the compositor at the moment of the press, and for the same reason: what
/// the machine will show is a fact that can change while a person is looking
/// at it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Picker {
    /// The folder it is standing in. Always rooted and free of `..`, because
    /// it began that way and only ever gained one name at a time.
    at: PathBuf,
    /// What is in that folder, as of the last time it was asked.
    showing: Inside,
}

impl Picker {
    /// Open a picker at this folder.
    ///
    /// `at` is where the shell starts a person — their home folder, in the
    /// ordinary case. It is checked before anything is opened: a path that is
    /// not written in full, or that steps upwards, cannot be compared honestly
    /// against a grant, and normalising it here would mean this crate and the
    /// kernel disagreeing about what a path means.
    ///
    /// # Errors
    /// [`NotPicked`] — the two refusals about the path itself, or whatever the
    /// machine said about the folder.
    pub fn standing_in(at: &Path, folders: &dyn Folders) -> Result<Self, NotPicked> {
        if !at.has_root() {
            return Err(NotPicked::NotAFullPath);
        }
        if !is_usable(at) {
            return Err(NotPicked::CouldLeadElsewhere);
        }
        let showing = folders.inside(at)?;
        Ok(Self {
            at: at.to_path_buf(),
            showing,
        })
    }

    /// The folder it is standing in.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// The folders in it, sorted, as they are shown.
    #[must_use]
    pub fn showing(&self) -> &[String] {
        self.showing.names()
    }

    /// How many things in this folder could not be named or read.
    #[must_use]
    pub fn could_not_be_named(&self) -> usize {
        self.showing.could_not_be_named()
    }

    /// Whether there is anything above where it is standing.
    #[must_use]
    pub fn is_at_the_top(&self) -> bool {
        is_a_root(&self.at) || self.at.parent().is_none()
    }

    /// Open one of the folders being shown.
    ///
    /// The name must be one of [`Picker::showing`], matched exactly. That is
    /// the whole of how this crate stays inside the tree it walked: a name
    /// that is not on the list is refused before any path is built from it, so
    /// `..`, `/etc` and a folder that was never listed are all the same
    /// refusal — *there is no folder of that name here*.
    ///
    /// # Errors
    /// [`NotPicked::NotShownHere`] for a name that is not being shown, or
    /// whatever the machine said about the folder — and in every case the
    /// picker is left exactly where it was.
    pub fn go_into(&mut self, named: &str, folders: &dyn Folders) -> Result<(), NotPicked> {
        if !self.showing.holds(named) {
            return Err(NotPicked::NotShownHere);
        }
        let into = self.at.join(named);
        let showing = folders.inside(&into)?;
        self.at = into;
        self.showing = showing;
        Ok(())
    }

    /// Go up to the folder above.
    ///
    /// # Errors
    /// [`NotPicked::NothingAbove`] at the top of the disk, or whatever the
    /// machine said about the folder above — and in either case the picker is
    /// left exactly where it was.
    pub fn go_up(&mut self, folders: &dyn Folders) -> Result<(), NotPicked> {
        if self.is_at_the_top() {
            return Err(NotPicked::NothingAbove);
        }
        let Some(above) = self.at.parent() else {
            return Err(NotPicked::NothingAbove);
        };
        let above = above.to_path_buf();
        let showing = folders.inside(&above)?;
        self.at = above;
        self.showing = showing;
        Ok(())
    }

    /// Pick the folder it is standing in.
    ///
    /// This is the act ADR 0001 §3 means by *a folder chosen in a picker*, and
    /// what comes out of it is the only thing in alo OS a grant can be made
    /// from.
    ///
    /// # Errors
    /// [`NotPicked::TheWholeMachine`] at the top of the disk. Nothing else:
    /// what is standing here has been listed, and picking asks the machine
    /// nothing further.
    pub fn pick(&self) -> Result<Chosen, NotPicked> {
        if is_a_root(&self.at) {
            return Err(NotPicked::TheWholeMachine);
        }
        Ok(Chosen::Folder(Picked::folder_chosen(self.at.clone())))
    }

    /// Close the picker without picking anything.
    ///
    /// A method rather than a bare [`Chosen::Nothing`] anybody writes, so that
    /// *the person closed it* is something the picker said and can be read as
    /// what it is.
    #[must_use]
    pub fn closed_without_picking(&self) -> Chosen {
        Chosen::Nothing
    }

    /// The heading over the picker, in the language the person reads.
    #[must_use]
    pub fn ask(strings: &Strings) -> Said {
        strings.say(&words::ASK.key(), &Filling::nothing())
    }

    /// What to say about a folder holding more than can be shown, or [`None`]
    /// when everything in it is on the list.
    ///
    /// [`None`] rather than an empty sentence, so a surface cannot draw a
    /// blank line where a warning belongs — and so a folder that fits says
    /// nothing at all rather than saying *and no more*.
    #[must_use]
    pub fn more_than_shown(&self, strings: &Strings) -> Option<Said> {
        self.showing
            .there_are_more()
            .then(|| strings.say(&words::MORE_THAN_SHOWN.key(), &Filling::nothing()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::folders::{MOST_SHOWN, NotShown};
    use crate::testing::{WrittenDown, in_english};

    /// A picker opened in the home folder of the tree these tests walk.
    fn in_her_home_folder(machine: &WrittenDown) -> Picker {
        Picker::standing_in(Path::new("/home/anna"), machine).unwrap()
    }

    /// The ordinary case: a picker opens where it was told to and shows what
    /// is there, sorted.
    #[test]
    fn a_picker_opens_where_it_was_told_and_shows_what_is_there() {
        let machine = WrittenDown::a_home_folder();
        let picker = in_her_home_folder(&machine);
        assert_eq!(picker.at(), Path::new("/home/anna"));
        assert_eq!(picker.showing(), ["Invoices", "Photos", "Work"]);
        assert!(!picker.is_at_the_top());
        assert!(picker.more_than_shown(&in_english()).is_none());
    }

    /// **A picker is never opened at a path that cannot be compared against a
    /// grant.** Both refusals happen before anything on the machine is opened,
    /// which is why the fixture here has nothing in it at all.
    #[test]
    fn a_picker_is_not_opened_at_a_path_a_grant_could_not_be_made_from() {
        let nothing = WrittenDown::nothing();
        assert_eq!(
            Picker::standing_in(Path::new("Invoices"), &nothing),
            Err(NotPicked::NotAFullPath)
        );
        assert_eq!(
            Picker::standing_in(Path::new("/home/anna/../root"), &nothing),
            Err(NotPicked::CouldLeadElsewhere)
        );
    }

    /// **A picker opened at a folder that is not there is refused in words**,
    /// rather than opening onto an empty list that reads as an empty folder.
    #[test]
    fn a_picker_opened_at_nothing_is_refused() {
        let machine = WrittenDown::a_home_folder();
        assert_eq!(
            Picker::standing_in(Path::new("/home/bruno"), &machine),
            Err(NotPicked::NothingThere)
        );
    }

    /// And every other thing the machine can say about a folder reaches its
    /// own refusal at the moment the picker is opened.
    #[test]
    fn a_picker_opened_at_something_the_machine_refuses_says_which() {
        let machine = WrittenDown::a_home_folder()
            .refusing("/home/anna/Photos", NotShown::WouldNotBeRead)
            .refusing("/home/anna/Work", NotShown::NotAFolder);
        assert_eq!(
            Picker::standing_in(Path::new("/home/anna/Photos"), &machine),
            Err(NotPicked::WouldNotBeRead)
        );
        assert_eq!(
            Picker::standing_in(Path::new("/home/anna/Work"), &machine),
            Err(NotPicked::NotAFolder)
        );
    }

    /// Opening a folder that is shown moves the picker into it, and what it
    /// shows is that folder's.
    #[test]
    fn opening_a_folder_that_is_shown_moves_into_it() {
        let machine = WrittenDown::a_home_folder();
        let mut picker = in_her_home_folder(&machine);
        picker.go_into("Invoices", &machine).unwrap();
        assert_eq!(picker.at(), Path::new("/home/anna/Invoices"));
        assert_eq!(picker.showing(), ["2026"]);
    }

    /// **A name that is not being shown opens nothing**, and this is the
    /// refusal that keeps the picker inside the tree it walked: `..` and a
    /// full path are refused for exactly the same reason as a folder that was
    /// never listed, and the picker has not moved.
    #[test]
    fn a_name_that_is_not_shown_opens_nothing_and_moves_nothing() {
        let machine = WrittenDown::a_home_folder();
        let mut picker = in_her_home_folder(&machine);
        for named in ["..", "/etc", "etc", "invoices", "Invoices/2026", ""] {
            assert_eq!(
                picker.go_into(named, &machine),
                Err(NotPicked::NotShownHere),
                "`{named}` was opened"
            );
            assert_eq!(picker.at(), Path::new("/home/anna"));
        }
    }

    /// **A folder that went away between being listed and being opened leaves
    /// the person where they were**, with a sentence rather than an empty
    /// surface they cannot get out of.
    #[test]
    fn a_folder_that_went_away_leaves_the_picker_where_it_was() {
        let mut machine = WrittenDown::a_home_folder();
        let mut picker = in_her_home_folder(&machine);
        machine.took_away("/home/anna/Invoices");

        assert_eq!(
            picker.go_into("Invoices", &machine),
            Err(NotPicked::NothingThere)
        );
        assert_eq!(picker.at(), Path::new("/home/anna"));
        assert_eq!(picker.showing(), ["Invoices", "Photos", "Work"]);
    }

    /// Going up reaches the folder above, and the top of the disk is reachable
    /// — a person navigating upwards should not meet an invisible floor.
    #[test]
    fn going_up_reaches_the_folder_above_and_then_the_top() {
        let machine = WrittenDown::a_home_folder();
        let mut picker = in_her_home_folder(&machine);
        picker.go_up(&machine).unwrap();
        assert_eq!(picker.at(), Path::new("/home"));
        picker.go_up(&machine).unwrap();
        assert_eq!(picker.at(), Path::new("/"));
        assert!(picker.is_at_the_top());
    }

    /// **There is nothing above the top**, and asking is refused in words
    /// rather than silently doing nothing.
    #[test]
    fn there_is_nothing_above_the_top_of_the_disk() {
        let machine = WrittenDown::a_home_folder();
        let mut picker = Picker::standing_in(Path::new("/"), &machine).unwrap();
        assert_eq!(picker.go_up(&machine), Err(NotPicked::NothingAbove));
        assert_eq!(picker.at(), Path::new("/"));
    }

    /// **A folder above that the machine will not open leaves the picker where
    /// it was**, the same as a move downwards.
    #[test]
    fn going_up_into_something_the_machine_refuses_moves_nothing() {
        let machine = WrittenDown::a_home_folder().refusing("/home", NotShown::WouldNotBeRead);
        let mut picker = in_her_home_folder(&machine);
        assert_eq!(picker.go_up(&machine), Err(NotPicked::WouldNotBeRead));
        assert_eq!(picker.at(), Path::new("/home/anna"));
        assert_eq!(picker.showing(), ["Invoices", "Photos", "Work"]);
    }

    /// **A pick is over the folder standing in, and never over its parent.**
    /// The picker walked down two folders; what comes out names the second.
    #[test]
    fn a_pick_is_over_the_folder_standing_in() {
        let machine = WrittenDown::a_home_folder();
        let mut picker = in_her_home_folder(&machine);
        picker.go_into("Invoices", &machine).unwrap();
        picker.go_into("2026", &machine).unwrap();

        let chosen = picker.pick().unwrap();
        assert_eq!(chosen.folder(), Some(Path::new("/home/anna/Invoices/2026")));
    }

    /// **The whole machine cannot be picked**, which is ADR 0001 §3 met where
    /// a person can read it.
    #[test]
    fn the_whole_machine_cannot_be_picked() {
        let machine = WrittenDown::a_home_folder();
        let picker = Picker::standing_in(Path::new("/"), &machine).unwrap();
        assert_eq!(picker.pick(), Err(NotPicked::TheWholeMachine));
        assert!(
            !NotPicked::TheWholeMachine
                .said(&in_english())
                .text()
                .is_empty()
        );
    }

    /// **Closing without picking is nothing**, said by the picker rather than
    /// assumed by whatever was holding it.
    #[test]
    fn closing_without_picking_chooses_nothing() {
        let machine = WrittenDown::a_home_folder();
        let picker = in_her_home_folder(&machine);
        let chosen = picker.closed_without_picking();
        assert!(chosen.is_nothing());
        assert_eq!(chosen.folder(), None);
    }

    /// A folder with more in it than can be shown says so, in the person's own
    /// language, and a folder that fits says nothing at all.
    #[test]
    fn a_folder_with_more_in_it_than_can_be_shown_says_so() {
        let many: Vec<String> = (0..=MOST_SHOWN)
            .map(|which| format!("folder-{which:05}"))
            .collect();
        let names: Vec<&str> = many.iter().map(String::as_str).collect();
        let machine = WrittenDown::nothing().holding("/home/anna", &names);
        let picker = in_her_home_folder(&machine);

        assert_eq!(picker.showing().len(), MOST_SHOWN);
        let said = picker.more_than_shown(&in_english()).unwrap();
        assert!(!said.is_a_bug());
        assert!(said.text().contains("more folders"), "{said}");
    }

    /// The heading is a sentence a person can read, from the machine's own
    /// vocabulary.
    #[test]
    fn the_picker_says_what_it_is_for() {
        let said = Picker::ask(&in_english());
        assert!(!said.is_a_bug());
        assert_eq!(said.text(), words::ASK.says());
    }

    /// What could not be named is carried through to whatever draws the
    /// picker, rather than stopping at the port that counted it.
    #[test]
    fn what_could_not_be_named_reaches_the_surface() {
        let machine = WrittenDown::nothing().holding("/home/anna", &["Invoices", ".."]);
        let picker = in_her_home_folder(&machine);
        assert_eq!(picker.showing(), ["Invoices"]);
        assert_eq!(picker.could_not_be_named(), 1);
    }
}
