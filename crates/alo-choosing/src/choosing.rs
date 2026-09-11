//! A person's change to their own settings, as one value.
//!
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! gives four things to the person sitting at the machine: which model answers
//! their questions, which weights they brought, which providers they added and
//! which language they read. Until this file **every one of them was a choice
//! nothing could carry out.** `crate::Settings` read the file and nothing in
//! this repository wrote it, so a settings surface had exactly one way to make
//! a choice happen, which was to compose the text of somebody's settings by
//! hand.
//!
//! This is the same gap `alo-changing` closed for the grants one file over, and
//! the shape is deliberately the same one.
//!
//! # What the doors take is what is already checked
//!
//! There is no door here that takes text. A choice arrives as
//! [`crate::Picked`], weights as `alo_models::Weights`, a provider as
//! `alo_models::Provider`, a language as `alo_strings::Language` — every one of
//! them a value whose own crate has already refused what it refuses. This file
//! adds the one rule that needs two of them at once, which is
//! [`crate::Settings::of`]'s: a choice into a list this person keeps must name
//! something on that list.
//!
//! So the two ways a settings file can be self-contradictory are refused **at
//! the moment somebody clicks**, naming what they named, rather than becoming a
//! file the same machine refuses whole the next time anybody asks it a
//! question.
//!
//! # A change is applied to a copy
//!
//! Every door builds the changed settings, writes **those** whole, and only
//! then replaces the settings this value holds. A refused change therefore
//! leaves all three places a choice lives exactly as they were — the file, this
//! value, and the settings a daemon will read next — which is what makes *a
//! write that fails leaves the file as it was* a shape rather than a promise
//! somebody keeps carefully.
//!
//! # Nothing is chosen for anybody, and nothing is pre-selected
//!
//! [`Choosing::at`] on a machine nobody has configured **writes nothing**. It
//! reads a file that is not there as a person who has not chosen and stops
//! there; the first byte is written by the first door somebody calls. ADR 0016
//! refuses a default nobody chose and
//! [ADR 0025](../../../docs/decisions/0025-the-default-is-what-a-machine-arrives-able-to-do.md)'s
//! reading turns on exactly that distinction, so a value that wrote *nothing
//! chosen* into somebody's file on being opened would be this crate taking the
//! one file ADR 0016 keeps for them.
//!
//! # There is no knock, and that is a finding rather than an omission
//!
//! `alo-changing` ends in `alo_protocol::FromAPerson::Granted`, because a
//! daemon holds the grants it read at start-up and would otherwise never learn.
//! **Settings are not held that way.** `alo-agentd` reads this file *once a
//! turn, at the first question of that turn* — `Questions::a_new_turn` forgets
//! and `Questions::what_answers` looks — so a change written here is in force
//! for the next question anybody asks, and a knock would be a message telling a
//! service to do something it already does.
//!
//! That was read off the daemon rather than assumed, and the daemon already
//! measures it: `a_new_turn_reads_the_file_the_person_has_just_written`. What
//! this crate is held to instead is that it *could not* knock — nothing here
//! names a daemon, a socket or the words one speaks, which
//! `tests/no_agents_door_reaches_these_settings.rs` reads off the manifest.
//!
//! # The organisation's file is not here
//!
//! `/etc/alo/agentd.toml` is the bound (ADR 0004, ADR 0016) and has an owner
//! who is not the person. Nothing in this file reaches it, nothing in this
//! crate has ever read it, and a door here that wrote it would be a person
//! editing the rule they are bounded by.

use std::path::{Path, PathBuf};

use alo_models::{Brought, Provider, Providers, Weights};
use alo_strings::Language;

use crate::chosen::Picked;
use crate::keeping::kept;
use crate::refusing::NotSet;
use crate::settings::{Settings, Unresolved};
use crate::setup::Setup;
use crate::unwritten::NotWritten;
use crate::writing::written;

/// A person's settings, and the changes they can make to them.
///
/// Made from a path and nothing else. What it holds afterwards is what the file
/// at that path says, and after every change it is what the file at that path
/// now says.
#[derive(Debug)]
pub struct Choosing {
    /// Where this person's settings are.
    at: PathBuf,
    /// What they say at this moment.
    settings: Settings,
}

impl Choosing {
    /// This person's settings, ready to be changed.
    ///
    /// **A file that is not there is not an error**, and nothing is written for
    /// it: it is a person who has not chosen, which is the ordinary state of a
    /// machine on its first morning and the state the first choice is made
    /// from. `crate::Settings::at` has that argument.
    ///
    /// **A file that is there and does not hold is refused**, and this is the
    /// one place where that matters more than it does on the way in: a value
    /// that read an unreadable file as *nothing chosen* would write a person's
    /// settings over the top of whatever they had typed, at the first thing
    /// they changed, and the typo they were about to fix would be gone.
    ///
    /// # Errors
    ///
    /// [`NotSet`], which is every way a file that **is** there is not settings.
    pub fn at(at: &Path) -> Result<Self, NotSet> {
        Ok(Self {
            at: at.to_owned(),
            settings: Settings::at(at)?,
        })
    }

    /// Where this person's settings are.
    #[must_use]
    pub fn where_it_is(&self) -> &Path {
        &self.at
    }

    /// What this person's settings say at this moment.
    ///
    /// After a change that was made, what the file now says. After a change
    /// that was refused, what it said before — unchanged byte for byte, because
    /// the refusal happened before anything was written.
    #[must_use]
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// **What answers this person's questions**, chosen or cleared.
    ///
    /// [`None`] is the person un-choosing, which is a thing they are allowed to
    /// do and is not the same as never having chosen — the file keeps whatever
    /// else is in it, and the machine goes back to saying nothing has been
    /// chosen to answer questions.
    ///
    /// # Errors
    ///
    /// [`NotWritten::NotBrought`] and [`NotWritten::NoSuchProvider`] when the
    /// choice names a list this person keeps and nothing on it answers to the
    /// name; [`NotWritten::NotExpressible`] and [`NotWritten::NotKept`] for the
    /// two ways the file itself does not happen. The settings are as they were
    /// in all four.
    pub fn answered_by(&mut self, picked: Option<Picked>) -> Result<(), NotWritten> {
        let changed = self.with(
            picked,
            self.settings.brought().clone(),
            self.settings.providers().clone(),
            self.settings.languages().to_vec(),
            self.settings.setup(),
        )?;
        self.apply(changed)
    }

    /// **Weights this person brought to the machine themselves**, added to
    /// their own list.
    ///
    /// The list is `alo_models::Brought` and so is the rule about it: the same
    /// name twice is refused, because *these answered it* could not then say
    /// which. That refusal is carried in `alo-models`' own words rather than
    /// reworded here.
    ///
    /// # Errors
    ///
    /// [`NotWritten::NotWeights`] when the list will not take them, and the
    /// three the other doors share.
    pub fn bringing(&mut self, weights: Weights) -> Result<(), NotWritten> {
        let mut brought = self.settings.brought().clone();
        brought.add(weights).map_err(|why| NotWritten::NotWeights {
            at: self.at.clone(),
            why,
        })?;
        let changed = self.with(
            self.settings.chosen().cloned(),
            brought,
            self.settings.providers().clone(),
            self.settings.languages().to_vec(),
            self.settings.setup(),
        )?;
        self.apply(changed)
    }

    /// **A provider this person added themselves**, added to their own list.
    ///
    /// What a provider is, where it may be and whether a key could travel in
    /// clear to it are all `alo_models::Provider::checked`'s, answered before
    /// one of these exists. What is answered here is the list's rule: two
    /// providers of one name would make *answered by Mistral* unable to say
    /// which.
    ///
    /// **Adding a provider is not choosing it.** It goes on the list and
    /// nothing else happens; what answers this person's questions is still
    /// whatever it was. That is ADR 0016 and ADR 0025's reading both — a
    /// machine that switched to the last thing somebody added would be choosing
    /// on their behalf, and a hosted provider is the one place that would cost
    /// them something no undo returns.
    ///
    /// # Errors
    ///
    /// [`NotWritten::NotAProvider`] when the list will not take it, and the
    /// three the other doors share.
    pub fn adding(&mut self, provider: Provider) -> Result<(), NotWritten> {
        let mut providers = self.settings.providers().clone();
        providers
            .add(provider)
            .map_err(|why| NotWritten::NotAProvider {
                at: self.at.clone(),
                why,
            })?;
        let changed = self.with(
            self.settings.chosen().cloned(),
            self.settings.brought().clone(),
            providers,
            self.settings.languages().to_vec(),
            self.settings.setup(),
        )?;
        self.apply(changed)
    }

    /// **The languages this person reads**, best first.
    ///
    /// The whole list at once rather than one language added to it, because the
    /// order *is* the setting: `alo-strings` reads it as a preference and a
    /// door that appended would make *which is my second language* a function of
    /// the order somebody happened to click. An empty list is a person who has
    /// not said, which is not a person who reads English — `alo-strings` holds
    /// English as the source rather than as a default.
    ///
    /// # Errors
    ///
    /// The three every door shares: the two halves of the settings disagreeing,
    /// and the two ways the file does not happen.
    pub fn reading(&mut self, languages: Vec<Language>) -> Result<(), NotWritten> {
        let changed = self.with(
            self.settings.chosen().cloned(),
            self.settings.brought().clone(),
            self.settings.providers().clone(),
            languages,
            self.settings.setup(),
        )?;
        self.apply(changed)
    }

    /// **The answer this person gave at setup**, written once and whole.
    ///
    /// [`Some`] is one of the sources setup offered; [`None`] is ADR 0009's
    /// fourth choice — *no model, no provider, no agent* — which is an answer
    /// rather than a skip, and the file says so afterwards.
    ///
    /// It is a door of its own rather than [`Choosing::answered_by`] with a
    /// flag, because the two say different things about the machine.
    /// `answered_by` is a person changing their mind in Settings and leaves
    /// `crate::Settings::setup` exactly as it was; this one says the question
    /// was put to them and answered. A machine that could not tell those apart
    /// would either put setup in front of somebody who declined it, or count a
    /// person who cleared a field in Settings as having been asked.
    ///
    /// **The provider case takes two writes and that is deliberate.** A choice
    /// naming a provider is a reference into the person's own list, so the
    /// provider is added through [`Choosing::adding`] first and chosen here
    /// second. Each write is whole or not at all; what a refusal here leaves
    /// behind is a provider on their list and an unanswered setup, which is
    /// true of the machine and is the state they can act on. The alternative —
    /// one door taking a provider and a choice together — would put the list's
    /// own rule and setup's in one call, and a person who mistyped an address
    /// would be told their setup failed rather than that their provider did.
    ///
    /// # Errors
    ///
    /// [`NotWritten::NotBrought`] and [`NotWritten::NoSuchProvider`] when the
    /// answer names a list this person keeps and nothing on it answers to the
    /// name, and the two ways the file itself does not happen. **Setup is not
    /// recorded as answered in any of them** — the settings are exactly as they
    /// were, and the question is still waiting.
    pub fn setting_up(&mut self, answered: Option<Picked>) -> Result<(), NotWritten> {
        let changed = self.with(
            answered,
            self.settings.brought().clone(),
            self.settings.providers().clone(),
            self.settings.languages().to_vec(),
            Setup::Answered,
        )?;
        self.apply(changed)
    }

    /// The settings these parts make, or the reason they are not settings.
    ///
    /// [`Settings::of`] is the one rule that needs two halves of a file at
    /// once, and it is asked here rather than trusted to have been asked by
    /// whoever built the parts.
    fn with(
        &self,
        chosen: Option<Picked>,
        brought: Brought,
        providers: Providers,
        languages: Vec<Language>,
        setup: Setup,
    ) -> Result<Settings, NotWritten> {
        Settings::of(chosen, brought, providers, languages, setup).map_err(|why| match why {
            Unresolved::Weights(model) => NotWritten::NotBrought {
                at: self.at.clone(),
                model,
            },
            Unresolved::Provider(provider) => NotWritten::NoSuchProvider {
                at: self.at.clone(),
                provider,
            },
        })
    }

    /// Write these settings, and hold them only once they are on the disk.
    ///
    /// The order is the whole of it and it is held by shape: the value this
    /// crate hands back is assigned on the far side of two `?`, so there is no
    /// arrangement of these three lines in which a caller is told a change was
    /// made and the file does not say so.
    fn apply(&mut self, changed: Settings) -> Result<(), NotWritten> {
        let text = written(&changed, &self.at)?;
        kept(&self.at, &text).map_err(|why| NotWritten::NotKept {
            at: self.at.clone(),
            why: why.to_string(),
        })?;
        self.settings = changed;
        Ok(())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::chosen::{Chosen, Which};
    use crate::testing::a_folder_of_our_own;
    use alo_models::{Driving, Region};

    /// A settings file under a folder these tests own, with nothing in it yet.
    fn a_machine_nobody_has_configured(what: &str) -> PathBuf {
        a_folder_of_our_own(what)
            .join(crate::THE_FOLDER)
            .join(crate::THE_SETTINGS)
    }

    /// One set of weights somebody brought.
    fn weights(id: &str) -> Weights {
        Weights::checked(id, 4_700_000_000)
            .unwrap()
            .measured(Driving::Reliably)
    }

    /// One provider, built the way a settings surface builds one.
    fn a_provider(name: &str) -> Provider {
        Provider::checked(
            name,
            "https://api.mistral.ai",
            Region::Declared("the EU".to_owned()),
            Some(crate::written::a_key_for(name)),
        )
        .unwrap()
    }

    /// **A machine nobody has configured is opened without writing anything.**
    /// Nothing is chosen for anybody, and a file appearing merely because
    /// something was opened would be this crate writing in the one file
    /// ADR 0016 keeps for the person.
    #[test]
    fn opening_settings_nobody_has_written_writes_nothing() {
        let at = a_machine_nobody_has_configured("untouched");

        let choosing = Choosing::at(&at).unwrap();

        assert_eq!(*choosing.settings(), Settings::untouched());
        assert!(!at.exists(), "a file was written by opening one");
        assert!(!at.parent().unwrap().exists(), "a folder was made for it");
    }

    /// **The first choice on a machine is an ordinary morning**: the file is
    /// not there, the folder is not there, and what somebody picks reads back
    /// through the door a daemon reads it through.
    #[test]
    fn a_first_choice_lands_in_a_file_that_was_not_there() {
        let at = a_machine_nobody_has_configured("first");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();

        let read = Settings::at(&at).unwrap();
        assert_eq!(read.chosen().unwrap().model(), "mistral-small");
        assert_eq!(
            read.chosen().unwrap().on_this_machine().unwrap().which(),
            Which::Catalogue
        );
        // And the value holds what the disk holds.
        assert_eq!(*choosing.settings(), read);
    }

    /// **A provider added and then chosen reads back as both**, which is the
    /// order a person does it in and the only order that can work: the choice
    /// is a reference into their own list.
    #[test]
    fn a_provider_added_and_then_chosen_reads_back_as_both() {
        let at = a_machine_nobody_has_configured("provider");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing.adding(a_provider("Mistral")).unwrap();
        // Adding is not choosing: what answers questions has not moved.
        assert!(choosing.settings().chosen().is_none());
        assert_eq!(Settings::at(&at).unwrap().providers().configured.len(), 1);

        choosing
            .answered_by(Some(
                Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap(),
            ))
            .unwrap();

        let read = Settings::at(&at).unwrap();
        assert_eq!(read.chosen().unwrap().provider(), Some("Mistral"));
        assert_eq!(read.provider().unwrap().endpoint, "https://api.mistral.ai");
        assert_eq!(
            read.provider().unwrap().region,
            Region::Declared("the EU".to_owned())
        );
    }

    /// **A language picked reads back, in the order it was picked in.**
    #[test]
    fn a_language_picked_reads_back_best_first() {
        let at = a_machine_nobody_has_configured("language");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing
            .reading(vec![
                Language::written("de").unwrap(),
                Language::written("en").unwrap(),
            ])
            .unwrap();

        assert_eq!(
            Settings::at(&at)
                .unwrap()
                .languages()
                .iter()
                .map(Language::tag)
                .collect::<Vec<_>>(),
            ["de", "en"]
        );
    }

    /// **Weights brought and then chosen reads back as the weights
    /// themselves**, with the measurement they earned — which is the thing
    /// nobody would want re-run at every boot.
    #[test]
    fn weights_brought_and_then_chosen_read_back_as_the_weights() {
        let at = a_machine_nobody_has_configured("brought");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing.bringing(weights("my-finetune")).unwrap();
        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )))
            .unwrap();

        let read = Settings::at(&at).unwrap();
        assert_eq!(read.weights().unwrap().bytes_on_disk, 4_700_000_000);
        assert_eq!(read.weights().unwrap().drives_verbs, Driving::Reliably);
    }

    /// **Un-choosing is a thing a person may do**, and it keeps the rest of
    /// their file: the list they brought is still theirs.
    #[test]
    fn un_choosing_leaves_the_rest_of_the_file_alone() {
        let at = a_machine_nobody_has_configured("un-choose");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing.bringing(weights("my-finetune")).unwrap();
        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )))
            .unwrap();

        choosing.answered_by(None).unwrap();

        let read = Settings::at(&at).unwrap();
        assert!(read.chosen().is_none());
        assert_eq!(read.brought().weights.len(), 1);
    }

    /// **A choice naming weights nobody brought is refused, and nothing is
    /// written** — the same disagreement the reader refuses whole, caught
    /// before it can become a file the machine would afterwards refuse.
    #[test]
    fn choosing_weights_this_machine_has_no_record_of_is_refused() {
        let at = a_machine_nobody_has_configured("no-weights");
        let mut choosing = Choosing::at(&at).unwrap();

        let refused = choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )))
            .unwrap_err();

        assert!(
            matches!(&refused, NotWritten::NotBrought { model, .. } if model == "my-finetune"),
            "{refused:?}"
        );
        assert!(!at.exists(), "a refused change wrote a file");
        assert!(choosing.settings().chosen().is_none());
    }

    /// **And so is a provider chosen before it was added.**
    #[test]
    fn choosing_a_provider_nobody_added_is_refused() {
        let at = a_machine_nobody_has_configured("no-provider");
        let mut choosing = Choosing::at(&at).unwrap();

        let refused = choosing
            .answered_by(Some(
                Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap(),
            ))
            .unwrap_err();

        assert!(
            matches!(&refused, NotWritten::NoSuchProvider { provider, .. } if provider == "Mistral"),
            "{refused:?}"
        );
        assert!(!at.exists());
    }

    /// **The same weights twice is refused in `alo-models`' own words**, and
    /// the file keeps the one entry it had.
    #[test]
    fn the_same_weights_twice_is_refused_and_the_file_keeps_what_it_had() {
        let at = a_machine_nobody_has_configured("twice");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing.bringing(weights("my-finetune")).unwrap();
        let before = std::fs::read_to_string(&at).unwrap();

        let refused = choosing.bringing(weights("my-finetune")).unwrap_err();

        assert!(
            matches!(refused, NotWritten::NotWeights { .. }),
            "{refused:?}"
        );
        assert_eq!(std::fs::read_to_string(&at).unwrap(), before);
        assert_eq!(choosing.settings().brought().weights.len(), 1);
    }

    /// **The same provider name twice is refused**, for the reason an answer
    /// has to be able to say which one produced it.
    #[test]
    fn the_same_provider_name_twice_is_refused_and_the_file_keeps_what_it_had() {
        let at = a_machine_nobody_has_configured("added-twice");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing.adding(a_provider("Mistral")).unwrap();
        let before = std::fs::read_to_string(&at).unwrap();

        let refused = choosing.adding(a_provider("Mistral")).unwrap_err();

        assert!(
            matches!(refused, NotWritten::NotAProvider { .. }),
            "{refused:?}"
        );
        assert_eq!(std::fs::read_to_string(&at).unwrap(), before);
        assert_eq!(choosing.settings().providers().configured.len(), 1);
    }

    /// **A write that cannot happen leaves the file, this value and the next
    /// sign-in exactly as they were**, and says so in words. The folder the
    /// file would go in is a file, so nothing can be made under it.
    ///
    /// **The blockage is put there after the value is made, and that ordering
    /// is the test rather than a detail of it.** Written the other way round —
    /// the file in the way before `Choosing::at` — this asserted something that
    /// is not true on the system alo OS runs on: reading through a path whose
    /// parent is a file answers `NotADirectory` on Linux, never `NotFound`, so
    /// `Settings::at` refuses it as a machine in a state nobody can read rather
    /// than reporting a person who has not chosen. That refusal is correct and
    /// deliberate, and it is `a_file_that_does_not_hold_is_refused_rather_than_replaced`'s
    /// subject; what *this* test is about is a **write** that fails, which
    /// needs a value that was made before anything was in the way.
    #[test]
    fn a_write_that_fails_changes_nothing_anywhere_and_is_told_in_words() {
        let folder = a_folder_of_our_own("in-the-way");
        let in_the_way = folder.join(crate::THE_FOLDER);
        let at = in_the_way.join(crate::THE_SETTINGS);

        // Nothing is there yet, so this is an ordinary first morning: a person
        // who has not chosen anything.
        let mut choosing = Choosing::at(&at).unwrap();
        assert!(choosing.settings().languages().is_empty());

        // And now the folder cannot be made, because a file is where it goes.
        std::fs::write(&in_the_way, "not a folder").unwrap();
        let refused = choosing
            .reading(vec![Language::written("de").unwrap()])
            .unwrap_err();

        assert!(matches!(refused, NotWritten::NotKept { .. }), "{refused:?}");
        assert_eq!(refused.at(), at);
        assert_eq!(
            std::fs::read_to_string(&in_the_way).unwrap(),
            "not a folder",
            "the write reached something it should not have"
        );
        assert!(choosing.settings().languages().is_empty());

        let said = refused.said(&crate::testing::in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text()
                .contains("nothing in your settings has been changed"),
            "{said}"
        );
    }

    /// **Settings that are there and do not hold are never written over.** A
    /// value that read them as *nothing chosen* would replace whatever somebody
    /// had typed at the first thing they changed, and the typo they were about
    /// to fix would be gone with it.
    #[test]
    fn a_file_that_does_not_hold_is_refused_rather_than_replaced() {
        let folder = a_folder_of_our_own("broken");
        let at = folder.join(crate::THE_SETTINGS);
        std::fs::write(&at, "format = 1\n[answers\n").unwrap();

        let refused = Choosing::at(&at).unwrap_err();

        assert!(
            matches!(refused, NotSet::NotUnderstood { .. }),
            "{refused:?}"
        );
        assert_eq!(
            std::fs::read_to_string(&at).unwrap(),
            "format = 1\n[answers\n"
        );
    }

    /// **A change made through this value is in force for the next question**,
    /// because the door a daemon reads settings through is the door this test
    /// reads them through — `alo_choosing::Settings::at`, which
    /// `alo-agentd`'s `of_a_session` calls once a turn. There is nothing to
    /// knock on, which this crate's manifest test is the other half of.
    #[test]
    fn a_change_is_in_force_the_next_time_anything_reads_the_file() {
        let at = a_machine_nobody_has_configured("in-force");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();
        assert_eq!(
            Settings::at(&at).unwrap().chosen().unwrap().model(),
            "mistral-small"
        );

        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "something-else").unwrap(),
            )))
            .unwrap();
        assert_eq!(
            Settings::at(&at).unwrap().chosen().unwrap().model(),
            "something-else"
        );
    }

    /// **A change replaces the file rather than adding to it**, so a provider
    /// removed from a person's settings is not left behind in them. Nothing in
    /// this crate removes one yet; what this measures is the shape that makes
    /// removal safe when a surface has one, which is that the file is written
    /// whole from the value.
    #[test]
    fn the_file_is_written_whole_rather_than_added_to() {
        let at = a_machine_nobody_has_configured("whole");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing
            .reading(vec![
                Language::written("de").unwrap(),
                Language::written("en").unwrap(),
            ])
            .unwrap();

        choosing
            .reading(vec![Language::written("fr").unwrap()])
            .unwrap();

        let text = std::fs::read_to_string(&at).unwrap();
        assert!(!text.contains("de"), "{text}");
        assert_eq!(
            Settings::at(&at)
                .unwrap()
                .languages()
                .iter()
                .map(Language::tag)
                .collect::<Vec<_>>(),
            ["fr"]
        );
    }

    /// **An answered setup and a machine nobody has asked are two different
    /// files**, and the difference survives the disk: a person who declined has
    /// chosen nothing and is finished, which is the state ADR 0009's fourth
    /// choice needs in order to exist at all.
    #[test]
    fn declining_at_setup_is_a_finished_setup_rather_than_a_skipped_one() {
        let at = a_machine_nobody_has_configured("declined");
        let mut choosing = Choosing::at(&at).unwrap();
        assert_eq!(choosing.settings().setup(), Setup::NotAnswered);

        choosing.setting_up(None).unwrap();

        let read = Settings::at(&at).unwrap();
        assert!(read.chosen().is_none());
        assert!(read.setup().is_answered());
        assert_ne!(read, Settings::untouched());
        assert_eq!(*choosing.settings(), read);
    }

    /// **A source chosen at setup is both the choice and the answer**, read
    /// back through the door a daemon reads settings through.
    #[test]
    fn a_source_chosen_at_setup_is_written_as_the_choice_and_as_an_answer() {
        let at = a_machine_nobody_has_configured("chose-at-setup");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing
            .setting_up(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();

        let read = Settings::at(&at).unwrap();
        assert_eq!(read.chosen().unwrap().model(), "mistral-small");
        assert!(read.setup().is_answered());
    }

    /// **Changing your mind in Settings afterwards is not being asked again**,
    /// and un-choosing does not un-answer: the person was asked, they answered,
    /// and a machine that forgot would show them setup a second time.
    #[test]
    fn a_later_change_in_settings_leaves_the_answer_where_it_was() {
        let at = a_machine_nobody_has_configured("still-answered");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing
            .setting_up(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();

        choosing.answered_by(None).unwrap();

        let read = Settings::at(&at).unwrap();
        assert!(read.chosen().is_none());
        assert!(read.setup().is_answered());
    }

    /// **And a change made in Settings by somebody nobody ever asked does not
    /// count as an answer.** The other direction of the same rule: a person who
    /// typed their own settings file has configured their machine and has still
    /// not been through setup.
    #[test]
    fn choosing_in_settings_is_not_an_answer_to_a_question_nobody_asked() {
        let at = a_machine_nobody_has_configured("never-asked");
        let mut choosing = Choosing::at(&at).unwrap();

        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();

        assert_eq!(Settings::at(&at).unwrap().setup(), Setup::NotAnswered);
    }

    /// **An answer that is refused does not record a setup**, so the question
    /// is still waiting rather than answered by a write that never happened.
    #[test]
    fn an_answer_that_is_refused_leaves_setup_unanswered() {
        let at = a_machine_nobody_has_configured("refused-answer");
        let mut choosing = Choosing::at(&at).unwrap();

        let refused = choosing
            .setting_up(Some(
                Picked::from_a_provider("Mistral", "mistral-small-latest").unwrap(),
            ))
            .unwrap_err();

        assert!(
            matches!(&refused, NotWritten::NoSuchProvider { provider, .. } if provider == "Mistral"),
            "{refused:?}"
        );
        assert!(!at.exists(), "a refused answer wrote a file");
        assert_eq!(choosing.settings().setup(), Setup::NotAnswered);
    }

    /// **Where the settings are is the path it was made with**, which is what
    /// every refusal names.
    #[test]
    fn the_settings_are_where_this_value_was_pointed() {
        let at = a_machine_nobody_has_configured("where");
        assert_eq!(Choosing::at(&at).unwrap().where_it_is(), at);
    }
}
