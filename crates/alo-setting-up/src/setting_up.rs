//! Setup as a value: the four in front of somebody, what they selected, and the
//! one answer it gets.
//!
//! # Nothing is selected, and pressing on without choosing does not choose
//!
//! [`SettingUp::at`] opens a person's settings and selects **nothing**.
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)
//! rejected a pre-selected control as *persuasion by geometry*, and this is
//! where that stops being a sentence in a document: there is no constructor
//! here that takes a selection, `selected` starts as [`None`], and answering
//! with nothing selected is a refusal rather than a quiet default.
//!
//! So the one thing this type may not do, it **cannot** do. A later change that
//! pre-selected the local choice — however reasonable — would have to add a
//! line to this file, where a reviewer reading ADR 0025 would find it.
//!
//! # Opening setup writes nothing
//!
//! Not a file, not a directory, not a `format` line.
//! `alo_choosing::Choosing::at` has that argument and it is inherited whole:
//! the first byte of a person's settings is written by the first answer they
//! give. A machine that had been switched on and never answered has a home
//! directory with nothing of ours in it.
//!
//! # It is asked once
//!
//! `alo_choosing::Settings::setup` is what makes that possible: a machine whose
//! owner declined has chosen nothing **and** has answered, which is a different
//! file from a machine nobody asked. Answering a second time is refused, and
//! everything setup decides is changeable afterwards through the doors
//! `alo-choosing` already has — which is ADR 0009's *turning it on later is a
//! setting, not a reinstall*, and its *no nagging* in the same sentence.
//!
//! # There are no pixels here
//!
//! No size, no place, no colour, no buttons, no order on the screen — only the
//! order of the list, which is ADR 0025's and is fixed in `crate::offered`.
//! What a person sees is the compositor's, exactly as `alo-approving` leaves
//! the question's appearance to whatever owns the screen. What is fixed here is
//! what a shell must not be free to decide differently: what the four are, what
//! each one says, that none is chosen, and what happens when one is answered.

use std::path::Path;

use alo_choosing::{Choosing, NotSet, Settings};

use crate::agent::TheAgent;
use crate::answering::Answer;
use crate::offered::{Offered, THE_FOUR};
use crate::refusing::NotSetUp;

/// Setup, as the thing a person is in the middle of.
#[derive(Debug)]
pub struct SettingUp {
    /// The person's own settings, and the only door that writes them.
    choosing: Choosing,
    /// Which of the four is selected, if any.
    ///
    /// [`None`] until the person selects one, and [`None`] again if they
    /// unselect. There is no way to build this value with one already in it.
    selected: Option<Offered>,
}

impl SettingUp {
    /// Setup, for the person whose settings are at this path.
    ///
    /// **Writes nothing.** A file that is not there is a person who has not
    /// chosen, which is the ordinary state of a machine on its first morning
    /// and is exactly the machine setup is for.
    ///
    /// **Nothing is selected**, and that is ADR 0025 rather than an initial
    /// value somebody may improve later.
    ///
    /// # Errors
    ///
    /// [`NotSet`], which is every way a settings file that **is** there is not
    /// settings. Setup refuses to run over a file it cannot read rather than
    /// treating it as a person who has chosen nothing — a flow that read a typo
    /// as *nothing chosen* and then wrote would take away the keystroke that
    /// was about to fix it.
    pub fn at(at: &Path) -> Result<Self, NotSet> {
        Ok(Self {
            choosing: Choosing::at(at)?,
            selected: None,
        })
    }

    /// The four configurations, in the order setup lists them.
    ///
    /// The local one first, and nothing about the order is weight.
    #[must_use]
    pub const fn offers(&self) -> [Offered; 4] {
        THE_FOUR
    }

    /// Which of the four is selected, if any.
    ///
    /// [`None`] until the person picks one.
    #[must_use]
    pub const fn selected(&self) -> Option<Offered> {
        self.selected
    }

    /// The person selects one of the four.
    ///
    /// Selecting is not answering and writes nothing: somebody reading the four
    /// may land on each of them in turn, and a machine that wrote to their
    /// settings as they did would be recording a decision they have not taken.
    pub const fn select(&mut self, offered: Offered) {
        self.selected = Some(offered);
    }

    /// The person selects nothing — back to the state setup opens in.
    ///
    /// Here because *unselect* has to be possible: a flow where the first click
    /// is irreversible is one where the first click is the answer.
    pub const fn select_nothing(&mut self) {
        self.selected = None;
    }

    /// Whether setup has already been answered on this machine.
    ///
    /// True of a machine whose owner chose a source **and** of one whose owner
    /// declined. A surface asks this before it draws anything: showing setup to
    /// somebody who declined it is ADR 0009's nagging, arriving by the one road
    /// that is guaranteed to meet everybody who declined.
    #[must_use]
    pub fn is_answered(&self) -> bool {
        self.choosing.settings().setup().is_answered()
    }

    /// What this person's settings say at this moment.
    #[must_use]
    pub const fn settings(&self) -> &Settings {
        self.choosing.settings()
    }

    /// Where this person's settings are.
    #[must_use]
    pub fn where_it_is(&self) -> &Path {
        self.choosing.where_it_is()
    }

    /// Whether this machine has an agent at all.
    ///
    /// [`TheAgent::Absent`] after a person declines, which is ADR 0009's *the
    /// agent's surfaces disappear rather than nag*.
    #[must_use]
    pub fn the_agent(&self) -> TheAgent {
        TheAgent::of(self.choosing.settings())
    }

    /// **The person answers setup**, and their answer is written into their own
    /// settings.
    ///
    /// One answer, and it must be the choice that is selected. Everything this
    /// refuses, it refuses **before** anything is written, so a refused answer
    /// leaves the settings byte for byte as they were and leaves setup waiting.
    ///
    /// # Errors
    ///
    /// [`NotSetUp::AlreadyAnswered`] when setup is over — it is asked once.
    /// [`NotSetUp::NothingSelected`] when nothing is selected, which is the
    /// ordinary way to press on without choosing and is not an accusation.
    /// [`NotSetUp::AnotherChoice`] when the answer is not the selected choice.
    /// [`NotSetUp::NoPairedMachine`] for a machine on this network, which this
    /// machine keeps no list of. [`NotSetUp::NothingToAskFor`] for a provider
    /// with no model beside it. [`NotSetUp::NotWritten`] for everything the
    /// person's own settings refuse, in `alo-choosing`'s own words.
    pub fn answer(&mut self, answered: &Answer) -> Result<(), NotSetUp> {
        if self.is_answered() {
            return Err(NotSetUp::AlreadyAnswered);
        }
        let Some(selected) = self.selected else {
            return Err(NotSetUp::NothingSelected);
        };
        if selected != answered.answers() {
            return Err(NotSetUp::AnotherChoice {
                selected,
                answered: answered.answers(),
            });
        }
        if matches!(answered, Answer::OnAMachineOnThisNetwork) {
            return Err(NotSetUp::NoPairedMachine);
        }
        let chosen = answered.chosen().map_err(|_| NotSetUp::NothingToAskFor)?;

        // A provider is added to the person's own list first, because the
        // choice is a reference into that list. Two writes, each whole or not
        // at all: `alo_choosing::Choosing::setting_up` says why this is not one
        // door taking both.
        if let Answer::FromAProvider { provider, .. } = answered {
            self.choosing
                .adding(provider.clone())
                .map_err(NotSetUp::NotWritten)?;
        }

        self.choosing
            .setting_up(chosen)
            .map_err(NotSetUp::NotWritten)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, a_provider};
    use alo_choosing::{Chosen, Which};
    use std::path::PathBuf;

    /// A settings file under a folder these tests own, with nothing in it yet.
    fn a_machine_on_its_first_morning(what: &str) -> PathBuf {
        a_folder_of_our_own(what)
            .join(alo_choosing::THE_FOLDER)
            .join(alo_choosing::THE_SETTINGS)
    }

    /// A local answer, built the way a setup surface builds one.
    fn on_this_machine() -> Answer {
        Answer::OnThisMachine(Chosen::of(Which::Catalogue, "mistral-small").unwrap())
    }

    /// **Setup opens with the four listed, the local one first, and nothing
    /// selected** — and it writes nothing while it does. This is ADR 0025's
    /// whole reading in one test.
    #[test]
    fn setup_opens_with_nothing_selected_and_writes_nothing() {
        let at = a_machine_on_its_first_morning("nothing-selected");

        let setting_up = SettingUp::at(&at).unwrap();

        assert_eq!(setting_up.offers(), THE_FOUR);
        assert_eq!(setting_up.offers()[0], Offered::OnThisMachine);
        assert_eq!(setting_up.selected(), None);
        assert!(!setting_up.is_answered());
        assert!(!at.exists(), "opening setup wrote a file");
        assert!(!at.parent().unwrap().exists(), "a folder was made for it");
    }

    /// **Selecting is not answering.** A person may land on each of the four in
    /// turn, and nothing is written until they answer.
    #[test]
    fn selecting_one_of_the_four_writes_nothing() {
        let at = a_machine_on_its_first_morning("selecting");
        let mut setting_up = SettingUp::at(&at).unwrap();

        for offered in THE_FOUR {
            setting_up.select(offered);
            assert_eq!(setting_up.selected(), Some(offered));
            assert!(!at.exists(), "selecting {offered:?} wrote a file");
        }

        setting_up.select_nothing();
        assert_eq!(setting_up.selected(), None);
    }

    /// **A local choice answered reaches the person's own file**, as both the
    /// choice and an answered setup.
    #[test]
    fn a_local_choice_answered_lands_in_the_persons_own_file() {
        let at = a_machine_on_its_first_morning("local");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::OnThisMachine);

        setting_up.answer(&on_this_machine()).unwrap();

        let read = Settings::at(&at).unwrap();
        assert_eq!(read.chosen().unwrap().model(), "mistral-small");
        assert!(read.setup().is_answered());
        assert!(setting_up.is_answered());
        assert_eq!(setting_up.the_agent(), TheAgent::Present);
    }

    /// **A provider answered is added and chosen in one act**, and reads back
    /// as both — which is the only order that can work, since the choice is a
    /// reference into the person's own list.
    #[test]
    fn a_provider_answered_is_added_to_the_list_and_then_chosen() {
        let at = a_machine_on_its_first_morning("provider");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::FromAProvider);

        setting_up
            .answer(&Answer::FromAProvider {
                provider: a_provider("Mistral"),
                model: "mistral-small-latest".to_owned(),
            })
            .unwrap();

        let read = Settings::at(&at).unwrap();
        assert_eq!(read.chosen().unwrap().provider(), Some("Mistral"));
        assert_eq!(read.provider().unwrap().endpoint, "https://api.mistral.ai");
        assert!(read.setup().is_answered());
    }

    /// **Declining is a finished setup, not a skipped one**, and the machine it
    /// leaves has no agent surfaces rather than greyed-out ones.
    #[test]
    fn declining_finishes_setup_and_leaves_a_machine_with_no_agent() {
        let at = a_machine_on_its_first_morning("declined");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::NotAtAll);

        setting_up.answer(&Answer::NotAtAll).unwrap();

        let read = Settings::at(&at).unwrap();
        assert!(read.chosen().is_none());
        assert!(read.setup().is_answered());
        assert_ne!(read, Settings::untouched());
        assert_eq!(setting_up.the_agent(), TheAgent::Absent);
    }

    /// **Answering with nothing selected chooses nothing and writes nothing.**
    /// Pressing on without choosing is the ordinary way to arrive here — ADR
    /// 0025's *pressing continue without choosing does not choose* — so it is a
    /// refusal in words rather than a default quietly taken.
    #[test]
    fn answering_with_nothing_selected_is_refused_and_nothing_is_written() {
        let at = a_machine_on_its_first_morning("none-selected");
        let mut setting_up = SettingUp::at(&at).unwrap();

        let refused = setting_up.answer(&on_this_machine()).unwrap_err();

        assert!(matches!(refused, NotSetUp::NothingSelected), "{refused:?}");
        assert!(!at.exists(), "a refused answer wrote a file");
        assert!(!setting_up.is_answered());
    }

    /// **And so is an answer that is not the selected choice.** Resolving it
    /// would be alo OS deciding which of the two the person meant.
    #[test]
    fn an_answer_that_is_not_the_selected_choice_is_refused() {
        let at = a_machine_on_its_first_morning("another-choice");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::NotAtAll);

        let refused = setting_up.answer(&on_this_machine()).unwrap_err();

        assert!(
            matches!(
                refused,
                NotSetUp::AnotherChoice {
                    selected: Offered::NotAtAll,
                    answered: Offered::OnThisMachine,
                }
            ),
            "{refused:?}"
        );
        assert!(!at.exists());
        assert!(!setting_up.is_answered());
    }

    /// **A machine on this network is offered and cannot yet be answered**, and
    /// the refusal is a true sentence about the machine: nothing here pairs
    /// with anything, and ADR 0003 says a pairing is deliberate on both sides.
    #[test]
    fn a_machine_on_this_network_is_refused_because_none_is_paired() {
        let at = a_machine_on_its_first_morning("no-pairing");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::OnAMachineOnThisNetwork);

        let refused = setting_up
            .answer(&Answer::OnAMachineOnThisNetwork)
            .unwrap_err();

        assert!(matches!(refused, NotSetUp::NoPairedMachine), "{refused:?}");
        assert!(!at.exists(), "a refused answer wrote a file");
        assert!(!setting_up.is_answered());
    }

    /// **A provider with no model to ask for is refused, and nothing is
    /// added.** The provider is written first when an answer is good, so this
    /// is the refusal that would otherwise leave half an answer on the disk.
    #[test]
    fn a_provider_with_no_model_is_refused_before_the_provider_is_added() {
        let at = a_machine_on_its_first_morning("no-model");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::FromAProvider);

        let refused = setting_up
            .answer(&Answer::FromAProvider {
                provider: a_provider("Mistral"),
                model: String::new(),
            })
            .unwrap_err();

        assert!(matches!(refused, NotSetUp::NothingToAskFor), "{refused:?}");
        assert!(!at.exists(), "a refused answer added a provider");
        assert!(!setting_up.is_answered());
    }

    /// **Setup is asked once.** A second answer is refused and the first one
    /// stands — a flow that took the second would let a stale screen overwrite
    /// what somebody chose.
    #[test]
    fn setup_answered_twice_keeps_the_first_answer() {
        let at = a_machine_on_its_first_morning("twice");
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::NotAtAll);
        setting_up.answer(&Answer::NotAtAll).unwrap();

        setting_up.select(Offered::OnThisMachine);
        let refused = setting_up.answer(&on_this_machine()).unwrap_err();

        assert!(matches!(refused, NotSetUp::AlreadyAnswered), "{refused:?}");
        assert!(Settings::at(&at).unwrap().chosen().is_none());
    }

    /// **And a machine that was answered before this value existed is already
    /// answered.** Setup opened over settings that say so refuses the first
    /// answer as well as the second, which is what stops a second sign-in from
    /// asking somebody again.
    #[test]
    fn setup_opened_over_an_answered_machine_is_already_answered() {
        let at = a_machine_on_its_first_morning("already");
        let mut first = SettingUp::at(&at).unwrap();
        first.select(Offered::NotAtAll);
        first.answer(&Answer::NotAtAll).unwrap();

        let mut again = SettingUp::at(&at).unwrap();
        assert!(again.is_answered());
        again.select(Offered::OnThisMachine);
        assert!(matches!(
            again.answer(&on_this_machine()).unwrap_err(),
            NotSetUp::AlreadyAnswered
        ));
    }

    /// **Settings that are there and do not hold are never written over**, and
    /// setup will not run on top of them: a flow that read a typo as *nothing
    /// chosen* and then wrote would take away the keystroke that was about to
    /// fix it.
    #[test]
    fn setup_over_settings_that_do_not_hold_is_refused_rather_than_started() {
        let folder = a_folder_of_our_own("broken");
        let at = folder.join(alo_choosing::THE_SETTINGS);
        std::fs::write(&at, "format = 1\n[answers\n").unwrap();

        let refused = SettingUp::at(&at).unwrap_err();

        assert!(
            matches!(refused, NotSet::NotUnderstood { .. }),
            "{refused:?}"
        );
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\n[answers\n"
        );
    }

    /// **An answer the disk will not take leaves setup waiting**, and says so
    /// in the file's own words rather than in a second sentence of this
    /// crate's.
    #[test]
    fn an_answer_the_disk_will_not_take_leaves_setup_waiting() {
        let folder = a_folder_of_our_own("in-the-way");
        let in_the_way = folder.join(alo_choosing::THE_FOLDER);
        let at = in_the_way.join(alo_choosing::THE_SETTINGS);

        // Nothing is there yet, so this is an ordinary first morning.
        let mut setting_up = SettingUp::at(&at).unwrap();
        setting_up.select(Offered::NotAtAll);

        // And now the folder cannot be made, because a file is where it goes.
        std::fs::write(&in_the_way, "not a folder").unwrap();
        let refused = setting_up.answer(&Answer::NotAtAll).unwrap_err();

        assert!(matches!(refused, NotSetUp::NotWritten(_)), "{refused:?}");
        assert!(!setting_up.is_answered());
        assert_eq!(
            std::fs::read_to_string(&in_the_way).unwrap(),
            "not a folder",
            "the write reached something it should not have"
        );
    }

    /// **Where the settings are is the path setup was pointed at**, which is
    /// what the one refusal about a file names.
    #[test]
    fn the_settings_are_where_this_value_was_pointed() {
        let at = a_machine_on_its_first_morning("where");
        assert_eq!(SettingUp::at(&at).unwrap().where_it_is(), at);
    }
}
