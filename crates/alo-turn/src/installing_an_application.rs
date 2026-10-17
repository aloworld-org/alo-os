//! From *the person approved this installation* to *this application is here*.
//!
//! The second executor on this machine, and the shape of the first one
//! (`crate::carrying`) one verb family along. `alo-software` had every step of
//! an installation built — the places, the bound an organisation set, the two
//! halves either side of the indicator, the refusals in a person's own words —
//! and `alo_software::approved` had been waiting since it was written for the
//! one thing nothing did: hand it an authority an agent's call became.
//!
//! # The order, and why the indicator is in the middle of it
//!
//! 1. **Is this authority an installation at all?** `alo_software::approved`
//!    asks it, and refuses an authority from another verb, one that came from
//!    no approval, and one missing an argument. It is the only reader of the
//!    call here; nothing else in this file looks at an argument.
//! 2. **Which places may this machine install from?** `Enabled::read` asks the
//!    tool, under the bound an organisation set, which the machine holds.
//! 3. **May this one be used, and is the application already here?**
//!    `alo_software::installing`, which answers with the errand to show and has
//!    reached no network.
//! 4. **The line goes up on the indicator**, and only then
//! 5. **`alo_software::install`**, which asks step 3's questions again — a rule
//!    can change between one step and the next — and reaches the rented tool.
//! 6. **The line comes down, on every road out of here.**
//!
//! Law 1 is why 4 is where it is rather than anywhere later: an installation
//! fetches from somewhere else, and *nothing leaves silently* means the line is
//! up before the first byte and not after it. Steps 1 to 3 refuse without
//! anything leaving, which is `alo-software`'s own promise and is why a refusal
//! there leaves the indicator quiet.
//!
//! # There is no boundary around this, and that is not an omission
//!
//! A file verb runs inside a cgroup the kernel imposes, because what is bounded
//! is **our** work reaching the person's disk. An installation reaches the
//! rented tool, which `alo-software`'s own header holds at arm's length: what
//! is installed arrives **granted nothing** — not a folder, not the camera, not
//! a verb. The boundary ADR 0013 asks for is around a verb's work on the
//! person's files and is made out of resolved paths; there are no paths here to
//! make one out of.
//!
//! What this file does **not** do is widen anything. The install reaches one
//! program at one path, chosen by `alo-software` and not nameable from a call,
//! so law 2 holds exactly as it did: no argument of this verb reaches a command
//! line as a command.

use alo_capability::Authorised;
use alo_record::Entry;
use alo_software::{Enabled, Installed, Stopped};

use crate::machine::Machine;
use crate::refusing::NotDone;

/// Carry out an approved installation, and say what happened.
///
/// Answers with the entry on every road, including every refusal — the same
/// promise `crate::carrying` makes, and for the same reason: a turn that
/// answered without one would be a thing that happened with no evidence of it.
///
/// `now` is passed rather than read because this crate reads no clock, and the
/// indicator wants the moment the line went up.
pub(crate) fn installing_an_application(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    now: std::time::SystemTime,
) -> (Entry, Result<Installed, NotDone>) {
    // Answers for as long as the words live rather than for as long as this
    // borrow does, which is what lets the machine be borrowed mutably below
    // and still be asked for a sentence afterwards.
    let strings = machine.strings();

    // Step 1. The only reader of the call in this file.
    let wanted = match alo_software::approved(&authorised) {
        Ok(wanted) => wanted,
        Err(not_an_installation) => {
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::CouldNotInstallHere(not_an_installation.into())),
            );
        }
    };

    // The tool, the organisation's rule and the indicator, handed over at once
    // for `Machine::bounding_and_indicator`'s reason: an installation needs all
    // three and asking one at a time would be three borrows of one machine.
    let Some((tool, bound, indicator)) = machine.installing_and_showing() else {
        // Unreachable from outside: the constructor that offers this verb is
        // the one that takes the tool. Said rather than unwrapped, because the
        // whole point of a `Result` here is that a library inside the daemon
        // does not take the daemon down to report its own mistake.
        return (
            Entry::ran(&authorised, strings),
            Err(NotDone::CouldNotInstallHere(
                crate::went_wrong_installing::WentWrongInstalling::NothingToInstallWith,
            )),
        );
    };

    // Steps 2 and 3. Both refuse with nothing having left the machine.
    let enabled = match Enabled::read(tool, bound) {
        Ok(enabled) => enabled,
        Err(refused) => return (Entry::ran(&authorised, strings), refused_here(refused)),
    };
    let errand = match alo_software::installing(&enabled, &wanted, tool) {
        Ok(errand) => errand,
        Err(refused) => return (Entry::ran(&authorised, strings), refused_here(refused)),
    };

    // Steps 4, 5 and 6. From here the machine is reaching somewhere else and
    // the person can see that it is.
    let underway = indicator.beginning_on_its_own(errand, now);
    let done = alo_software::install(&underway, &enabled, wanted, tool);
    // **The line comes down whatever happened.** A refused install that left it
    // lit would be law 1's surface saying something is leaving when nothing is,
    // for the rest of the session.
    indicator.ended_on_its_own(underway);

    let outcome = match done {
        Ok(installed) => Ok(installed),
        Err(Stopped::Refused(refused)) => refused_here(refused),
        Err(Stopped::NotShown(not_shown)) => Err(NotDone::CouldNotInstallHere(not_shown.into())),
    };
    (Entry::ran(&authorised, strings), outcome)
}

/// `alo-software` said no, in its own words.
///
/// Its own function because three roads reach it and they are the same value.
/// The refusal is **not reworded here**: a place an organisation does not
/// permit is a sentence that crate wrote and that names the organisation, and
/// a second rendering of it is a second thing to keep true.
fn refused_here(refused: alo_software::NotDone) -> Result<Installed, NotDone> {
    Err(NotDone::CouldNotInstall(refused))
}

#[cfg(test)]
// Counted rather than guessed: thirteen `unwrap` and five `expect`, and no
// `panic!` — so `clippy::panic` is not listed. An unfulfilled expectation is
// an error in this workspace, which is what catches a list copied from
// somewhere else.
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::RefCell;

    use alo_applications::Application;
    use alo_capability::{Given, Grants};
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_record::Record;
    use alo_software::{
        Bound, Configured, Failed, INSTALL_APPLICATION, SetBy, SourceName, TheRentedTool, Tool,
    };

    use crate::Turning;
    use crate::machine::Machine;
    use crate::refusing::NotDone;
    use crate::testing::{hour, in_english, noon, offering};
    use crate::what_a_verb_did::WhatAVerbDid;

    /// A machine with two places set up, which remembers what it installed.
    ///
    /// A list in memory rather than the rented tool, because what these tests
    /// ask is **whether anything reached it** and under what conditions — the
    /// tool itself is `alo-software`'s to test and is tested there.
    #[derive(Default)]
    struct TwoPlaces {
        /// Every application it was asked to install, in order.
        installed: RefCell<Vec<String>>,
    }

    impl Tool for TwoPlaces {
        fn sources(&self) -> Result<Vec<Configured>, Failed> {
            Ok(vec![
                Configured {
                    name: "flathub".to_owned(),
                    address: "https://dl.flathub.org/repo/".to_owned(),
                    checks_signatures: true,
                    switched_off: false,
                },
                Configured {
                    name: "acme-apps".to_owned(),
                    address: "https://apps.acme.example/repo".to_owned(),
                    checks_signatures: true,
                    switched_off: false,
                },
            ])
        }

        fn installed(&self) -> Result<Vec<String>, Failed> {
            Ok(self.installed.borrow().clone())
        }

        fn open(&self) -> Result<Vec<String>, Failed> {
            Ok(Vec::new())
        }

        fn install(&self, _: &SourceName, application: &Application) -> Result<(), Failed> {
            self.installed
                .borrow_mut()
                .push(application.identifier().to_owned());
            Ok(())
        }

        fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
            Ok(Vec::new())
        }

        fn update(&self, _: &Application) -> Result<(), Failed> {
            Ok(())
        }

        fn remove(&self, _: &Application) -> Result<(), Failed> {
            Ok(())
        }
    }

    /// What installing the text editor from Flathub is asked for with.
    fn installing_the_text_editor() -> Vec<(&'static str, Given)> {
        vec![
            ("application", Given::text("org.gnome.TextEditor")),
            ("source", Given::text("flathub")),
        ]
    }

    /// A machine that can install, and the turn on it.
    ///
    /// Written as a closure for the reason `turning.rs`'s own arrangement is:
    /// a [`Turning`] borrows the machine, the machine borrows the tool, the
    /// record and the indicator, so every one of them has to live in the
    /// caller's frame.
    fn on_a_machine_that_can_install<T>(
        bound: Bound,
        doing: impl FnOnce(&mut Turning<'_, '_>, &mut Grants, &TwoPlaces) -> T,
    ) -> T {
        let strings = in_english();
        let tool = TwoPlaces::default();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let folder = crate::testing::a_folder_of_our_own("installing");
        let there = folder.join("a-note.txt");
        std::fs::write(&there, "anything").unwrap();
        let mut machine = Machine::carrying_out_file_and_software_verbs(
            &strings,
            &OnThisMachine,
            &tool,
            bound,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        // No grant for anything. `install_application` declares
        // `Requires::nothing_because` — the application is not on the machine
        // yet and arrives granted nothing — so an empty `Grants` is the
        // arrangement this verb is really used in, and a test that granted
        // something would be testing a machine nobody has.
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            offering(&there),
            "@software",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        doing(&mut turning, &mut grants, &tool)
    }

    /// **The whole road, in one test**: a machine that can install offers the
    /// verb, a person approves the sentence, the tool is reached once, and what
    /// comes back says what was installed and that it is sandboxed.
    #[test]
    fn an_approved_installation_reaches_the_tool_and_answers_what_arrived() {
        on_a_machine_that_can_install(Bound::Nobodys, |turning, grants, tool| {
            let id = turning
                .proposing(
                    INSTALL_APPLICATION,
                    &installing_the_text_editor(),
                    grants,
                    hour(),
                    noon(),
                )
                .expect("an installation is a change, and a change waits for a person");
            assert!(
                tool.installed.borrow().is_empty(),
                "it was installed before anybody approved it"
            );

            let did = turning
                .approving(id, grants, noon())
                .expect("the person approved it");
            let installed = did
                .installed()
                .expect("an installation answers as an installation");
            assert_eq!(installed.application().identifier(), "org.gnome.TextEditor");
            assert_eq!(
                *tool.installed.borrow(),
                ["org.gnome.TextEditor"],
                "the tool was reached once, with what was approved"
            );
            assert!(
                did.to_files().is_none(),
                "an installation answered as a file verb"
            );
            // **That the line was up while it happened is enforced by the
            // types, not by this test, and that is the stronger of the two.**
            // `alo_software::install` demands an `alo_egress::Underway`, and
            // the only maker of one is `Indicator::beginning_on_its_own`;
            // `ended_on_its_own` consumes it, so it cannot be lowered and then
            // reused. There is no arrangement of this code that installs
            // anything without the indicator having shown it. A test asserting
            // *not quiet* mid-install would need the tool to see the indicator,
            // which it deliberately cannot — `installing_under_the_description`
            // in `alo-agentd` asserts it on the raw road, where both are in one
            // frame.
        });
    }

    /// **And the indicator is quiet again afterwards.** The other half of law
    /// 1's surface: a line left up says something is leaving when nothing is,
    /// for the rest of the session, and a refused installation is the road
    /// where that is easiest to get wrong.
    #[test]
    fn the_indicator_is_quiet_after_an_installation_and_after_a_refused_one() {
        on_a_machine_that_can_install(Bound::Nobodys, |turning, grants, _| {
            let id = turning
                .proposing(
                    INSTALL_APPLICATION,
                    &installing_the_text_editor(),
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();
            turning.approving(id, grants, noon()).unwrap();
            assert!(
                turning.showing().is_quiet(),
                "an installation that worked left the indicator lit"
            );
        });
        // The same application again, on a machine that already has it:
        // `alo-software` refuses it as already here, before anything leaves.
        on_a_machine_that_can_install(Bound::Nobodys, |turning, grants, tool| {
            tool.installed
                .borrow_mut()
                .push("org.gnome.TextEditor".to_owned());
            let id = turning
                .proposing(
                    INSTALL_APPLICATION,
                    &installing_the_text_editor(),
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();
            let refused = turning.approving(id, grants, noon()).unwrap_err();
            assert!(
                matches!(refused, NotDone::CouldNotInstall(_)),
                "installing what is already here was not refused: {refused:?}"
            );
            assert!(
                turning.showing().is_quiet(),
                "a refused installation left the indicator lit"
            );
        });
    }

    /// **One approval is one execution.** The second answer under the same
    /// number is refused, and nothing reaches the tool a second time — which is
    /// one of the capability guarantees `CLAUDE.md` asks to be a test rather
    /// than prose, now that there is a second thing an approval can run.
    #[test]
    fn one_approval_installs_once_and_the_same_number_again_installs_nothing() {
        on_a_machine_that_can_install(Bound::Nobodys, |turning, grants, tool| {
            let id = turning
                .proposing(
                    INSTALL_APPLICATION,
                    &installing_the_text_editor(),
                    grants,
                    hour(),
                    noon(),
                )
                .unwrap();
            turning.approving(id, grants, noon()).unwrap();
            let again = turning.approving(id, grants, noon()).unwrap_err();
            assert!(
                matches!(again, NotDone::NotAnswered(_)),
                "the same approval answered twice: {again:?}"
            );
            assert_eq!(
                tool.installed.borrow().len(),
                1,
                "one approval installed it twice"
            );
        });
    }

    /// **A place the organisation does not permit is refused, in its words, and
    /// nothing leaves.** The refusal is `alo-software`'s own — this crate does
    /// not reword it — and the indicator is quiet, because the three questions
    /// that can refuse are all asked before the line goes up.
    #[test]
    fn a_place_the_organisation_does_not_permit_is_refused_and_nothing_is_installed() {
        on_a_machine_that_can_install(
            Bound::only(
                [SourceName::checked("acme-apps").expect("a place name of one word")],
                SetBy::AnAdministrator,
            ),
            |turning, grants, tool| {
                let id = turning
                    .proposing(
                        INSTALL_APPLICATION,
                        &installing_the_text_editor(),
                        grants,
                        hour(),
                        noon(),
                    )
                    .expect("the bound is asked at execution, not when it is proposed");
                let refused = turning.approving(id, grants, noon()).unwrap_err();
                assert!(
                    matches!(
                        &refused,
                        NotDone::CouldNotInstall(alo_software::NotDone::OutsideTheBound { .. })
                    ),
                    "a place outside the bound was not refused as one: {refused:?}"
                );
                let said = refused.said(turning.strings());
                assert!(!said.is_a_bug(), "{said}");
                assert!(
                    said.text().contains("flathub"),
                    "the refusal does not name the place: {said}"
                );
                assert!(
                    tool.installed.borrow().is_empty(),
                    "a refused installation reached the tool"
                );
                assert!(
                    refused.was_refused(),
                    "a place an organisation forbids is the capability model saying no"
                );
            },
        );
    }

    /// **A machine with no tool does not offer the verb at all.** The two lists
    /// are one list, which is `crate::machine`'s rule: asking for an
    /// installation on a machine built for the six is turned away where every
    /// other unknown verb is, and never reaches an executor.
    #[test]
    fn a_machine_built_for_the_six_offers_no_installation_and_runs_none() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let folder = crate::testing::a_folder_of_our_own("no-installing");
        let there = folder.join("a-note.txt");
        std::fs::write(&there, "anything").unwrap();
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert!(
            machine.verbs().of(INSTALL_APPLICATION).is_none(),
            "a machine with nothing to install with offered an installation"
        );
        let mut grants = Grants::default();
        let mut turning = Turning::beginning(
            offering(&there),
            "@software",
            hour(),
            &mut grants,
            &mut machine,
        )
        .unwrap();
        let turned_away = turning
            .proposing(
                INSTALL_APPLICATION,
                &installing_the_text_editor(),
                &grants,
                hour(),
                noon(),
            )
            .unwrap_err();
        assert!(
            matches!(turned_away, NotDone::TurnedAway(_)),
            "an unoffered verb was not turned away: {turned_away:?}"
        );
    }

    /// **A machine that can install offers seven verbs, and the seventh is the
    /// installation.** The count is asserted rather than the name alone,
    /// because a family declared twice is the one thing a second executor made
    /// possible and it would otherwise show up as a passing test.
    #[test]
    fn a_machine_that_can_install_offers_the_six_and_one_more() {
        let strings = in_english();
        let tool = TwoPlaces::default();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let machine = Machine::carrying_out_file_and_software_verbs(
            &strings,
            &OnThisMachine,
            &tool,
            Bound::Nobodys,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert_eq!(machine.verbs().all().count(), 8);
        assert!(machine.verbs().of(INSTALL_APPLICATION).is_some());
        // **And the read beside it**, which arrived on 2026-10-17. It needs no
        // tool at all — `alo_turn::looking_at_the_catalogue` reads a string
        // compiled into this binary — and is declared with the installation
        // only because `alo_software::verbs::declare_into` is one function. A
        // surface that wants the read without a tool is the change that splits
        // it, and until something does, a machine with no tool offers neither.
        assert!(
            machine
                .verbs()
                .of(alo_software::WHAT_CAN_BE_INSTALLED)
                .is_some()
        );
        // Named rather than counted, because a count of seven would also pass
        // if the installation had replaced one of the six.
        for file_verb in [
            "list_folder",
            "read_file",
            "find_in_folder",
            "rename_file",
            "move_file",
            "archive_folder",
        ] {
            assert!(
                machine.verbs().of(file_verb).is_some(),
                "{file_verb} went missing when the seventh arrived"
            );
        }
    }

    /// The rented tool is what the daemon really hands over, and this asserts
    /// the type it is built from satisfies the trait the machine wants — so a
    /// change to `Tool` cannot leave `alo-agentd` as the only place that finds
    /// out.
    #[test]
    fn the_rented_tool_is_what_a_real_machine_installs_with() {
        let strings = in_english();
        let tool = TheRentedTool::on_this_machine();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = crate::testing::NothingIsBounded;
        let machine = Machine::carrying_out_file_and_software_verbs(
            &strings,
            &OnThisMachine,
            &tool,
            Bound::Nobodys,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        assert!(machine.verbs().of(INSTALL_APPLICATION).is_some());
    }

    /// Nothing in this module is reached by a file verb, and this is the test
    /// that says so: the dispatch is on the verb's name, so a rename on a
    /// machine that can also install still answers as a file verb.
    #[test]
    fn a_file_verb_on_a_machine_that_can_install_still_answers_as_a_file_verb() {
        on_a_machine_that_can_install(Bound::Nobodys, |turning, grants, tool| {
            let read = turning.reading("read", &[], grants, noon());
            assert!(
                read.is_err(),
                "a read with no file named should not have formed"
            );
            assert!(
                tool.installed.borrow().is_empty(),
                "a file verb reached the installing tool"
            );
            // And the union answers which kind it is, so a caller that only
            // ever asks file verbs is not made to match an arm it cannot take.
            let as_a_file_verb = WhatAVerbDid::from(alo_files::Answer::Read(String::new()));
            assert!(as_a_file_verb.to_files().is_some());
            assert!(as_a_file_verb.installed().is_none());
        });
    }
}
