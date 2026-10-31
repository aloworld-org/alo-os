//! Everything a person has said about their own machine, as one value.
//!
//! Three things, and
//! [ADR 0016](../../../docs/decisions/0016-the-organisation-bounds-and-the-person-chooses.md)
//! says why they are one store rather than three: which model answers their
//! questions, which weights they brought to this machine themselves and which
//! language they read have the same owner, the same lifetime, and are all read
//! before a turn can say anything. Inventing a second store for one owner is how
//! a settings system becomes six settings systems.
//!
//! # Only what somebody changed is here
//!
//! `alo-shortcuts` settled this in item 7 and `alo-appearance` followed it: the
//! difference is stored and the rest lives in the code, so a release can improve
//! what an untouched machine does. Here the untouched machine has **no model and
//! no language**, and that is not a value waiting to be improved — it is the
//! honest state of a machine nobody has configured, and the thing a person is
//! told when they ask it a question.
//!
//! There is deliberately no `Default`. [`Settings::untouched`] says the same
//! thing with a name on it, and a `Default` would put *nothing chosen* one
//! `..Default::default()` away from being the fallback for a file that failed
//! to read — which is exactly the silence this crate exists to prevent.
//!
//! # The weights somebody brought are here, and the choice cannot outrun them
//!
//! [ADR 0019](../../../docs/decisions/0019-a-runtime-is-found-not-configured.md)
//! puts `alo_models::Brought` in this file rather than in a store of its own:
//! the weights are the person's, they fetched them, the licence they accepted
//! is theirs, and ADR 0016 already put *which model answers* here. A second
//! store for one owner is how a settings system becomes six.
//!
//! What that costs is a rule, and it is carried by [`Settings::of`] rather than
//! by whoever remembers to check: **a choice naming the brought list must name
//! an entry on it.** Settings that could say *my questions are answered by
//! weights that are not on my list* would be a machine asking a runtime for a
//! name nobody could look up, and the sentence a person got back would be about
//! a runtime rather than about their own file. So the pair is refused where it
//! is made, both roads into it — the file, and a settings panel calling this
//! directly.

use std::path::Path;

use alo_models::{Brought, Providers, Weights};
use alo_strings::Language;

use crate::chosen::{Picked, Which};
use crate::refusing::NotSet;
use crate::setup::Setup;
use crate::written::read;

/// A choice naming weights this person's own list does not have.
///
/// Not a refusal with words of its own, for [`crate::NoModel`]'s reason:
/// whoever needs a sentence about it is reading a file, and
/// [`crate::NotSet::NotBrought`] is the one that can name the file it is in.
/// This says only that the two halves of a settings value have to agree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Unresolved {
    /// A choice named the brought list and nothing on it answers to the name.
    Weights(String),

    /// A choice named a provider and the person's own list has none of that
    /// name.
    Provider(String),
}

impl Unresolved {
    /// What the choice named, exactly as it was written.
    #[must_use]
    pub fn named(&self) -> &str {
        match self {
            Self::Weights(named) | Self::Provider(named) => named,
        }
    }
}

/// What this person has said about their machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// What they chose to answer their questions, where they have chosen.
    ///
    /// One of the three the owner named: a model on this machine, or a provider
    /// they added — which is also how alo's own service is chosen, because
    /// ADR 0014 makes it one more provider with no special case anywhere.
    chosen: Option<Picked>,
    /// The weights they brought to this machine themselves.
    brought: Brought,
    /// The providers they added themselves.
    ///
    /// **No credential is in here.** `alo_models::Provider` holds a
    /// `SecretRef`, which is a name in a keyring and not a key, and the settings
    /// file has no field a key could be typed into at all — see `written.rs`.
    providers: Providers,
    /// The languages they read, best first.
    languages: Vec<Language>,
    /// Whether they have been asked what their machine should do, and answered.
    ///
    /// **Not a second copy of the choice** — `crate::setup` has the argument.
    /// It says the question was put and answered; what was answered is
    /// [`Settings::chosen`], and [`None`] there beside
    /// [`Setup::Answered`] is ADR 0009's fourth choice rather than a person
    /// nobody has asked yet.
    setup: Setup,
    /// What they said about alo using something else when what they chose is
    /// not available.
    ///
    /// [`None`] is **not** `No`. See [`Settings::may_use_another`].
    may_use_another: Option<MayUseAnother>,
}

/// What a person said about alo using something else when their choice is not
/// available.
///
/// `docs/features.md`, starred: *Unless the person says otherwise, in
/// Settings.* Asked for by the owner on 2026-10-18.
///
/// **Two states, and the third is the absence of this type.** A person who has
/// never seen the setting has not refused anything, and
/// [`Settings::may_use_another`] answers [`None`] for them — which is why this
/// is not a `bool`. *Absent is stronger than defaulted*: a default can be
/// failed to override, and an absence cannot be overridden wrongly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MayUseAnother {
    /// They turned it on: another model or API may answer instead.
    ///
    /// **It does not switch off the sentence saying where the answer came
    /// from.** That sentence is beside every answer and is not a thing this
    /// choice reaches — `docs/features.md` says so in the promise itself, and
    /// `alo_asking::Answer` has no shape without a source.
    Yes,
    /// They turned it off: nothing else may answer.
    ///
    /// A question that cannot reach the place they chose is **refused**, which
    /// is a thing they asked for rather than something taken from them.
    No,
}

impl MayUseAnother {
    /// Both answers, in the order a person reads them in the list they pick
    /// from.
    ///
    /// `Yes` first because it is the one that keeps answers coming, and a
    /// person arriving at this row has already chosen a provider or a machine
    /// — so the question in front of them is *may alo cover for it*, and the
    /// affirmative is the one they came to consider.
    pub const BOTH: [Self; 2] = [Self::Yes, Self::No];

    /// The string this crate declares for this answer.
    #[must_use]
    pub const fn word(self) -> alo_strings::Word {
        match self {
            Self::Yes => crate::words::ANOTHER_MAY_BE_USED,
            Self::No => crate::words::NOTHING_ELSE_MAY_BE_USED,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given this
    /// crate's words through [`crate::declare_into`] answers with the key,
    /// marked.
    #[must_use]
    pub fn said(self, strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(&self.word().key(), &alo_strings::Filling::nothing())
    }
}

impl Settings {
    /// Settings holding what somebody chose, brought and reads.
    ///
    /// # Errors
    /// [`Unresolved`] when the choice is a reference into a list this person
    /// keeps and that list has nothing of the name — weights they brought, or a
    /// provider they added. This file's header says why that is a refusal here
    /// rather than a `None` somewhere later.
    pub fn of(
        chosen: Option<Picked>,
        brought: Brought,
        providers: Providers,
        languages: Vec<Language>,
        setup: Setup,
    ) -> Result<Self, Unresolved> {
        match &chosen {
            Some(Picked::OnThisMachine(local))
                if local.which() == Which::Brought && brought.get(local.model()).is_none() =>
            {
                return Err(Unresolved::Weights(local.model().to_owned()));
            }
            // The same rule one list to the right: a choice is a reference into
            // a list the person keeps, and a reference that resolves to nothing
            // is a file whose two halves disagree.
            Some(Picked::FromAProvider { provider, .. }) if providers.get(provider).is_none() => {
                return Err(Unresolved::Provider(provider.clone()));
            }
            Some(_) | None => {}
        }
        Ok(Self {
            // **Nothing here decides it.** `crate::written` reads it from the
            // file and `and_may_use_another` puts it on; a person who has not
            // been asked arrives as `None` and stays that way.
            may_use_another: None,
            chosen,
            brought,
            providers,
            languages,
            setup,
        })
    }

    /// A machine nobody has configured.
    ///
    /// What a person who has never opened a settings panel has, and what a
    /// missing file answers with. Nothing answers questions here, nothing was
    /// brought here, and the machine says so.
    #[must_use]
    pub fn untouched() -> Self {
        Self {
            may_use_another: None,
            chosen: None,
            brought: Brought::default(),
            providers: Providers::default(),
            languages: Vec::new(),
            setup: Setup::NotAnswered,
        }
    }

    /// Whether this person has answered setup.
    ///
    /// [`Setup::NotAnswered`] on a machine nobody has configured, and on every
    /// machine configured before this key existed. It is deliberately separate
    /// from [`Settings::chosen`]: a person who answered *not at all* has chosen
    /// nothing and has finished setup, and a machine that read those two states
    /// as one would put the question to somebody who has already declined it.
    #[must_use]
    pub const fn setup(&self) -> Setup {
        self.setup
    }

    /// The providers this person added themselves.
    ///
    /// Empty on a machine where nobody has added one, which is not the same as
    /// a machine that cannot have any.
    #[must_use]
    pub const fn providers(&self) -> &Providers {
        &self.providers
    }

    /// What answers this person's questions, where they have chosen.
    ///
    /// **Not `Option<&Chosen>`.** It was, and a provider choice would have read
    /// as *nobody has chosen* — which is not a silent switch to somewhere else,
    /// and is still a machine telling somebody they have chosen nothing when
    /// they have. Every caller now has to say what it does about a provider,
    /// and the compiler is what asks them.
    #[must_use]
    pub fn chosen(&self) -> Option<&Picked> {
        self.chosen.as_ref()
    }

    /// What they said about alo using something else when their choice is not
    /// available.
    ///
    /// **Three answers and not two.** [`None`] is a person who has not been
    /// asked, [`MayUseAnother::No`] is one who said no, and reading them as the
    /// same thing would turn *nobody has decided* into *they refused* — which
    /// would refuse answers somebody gets today, and law 5 calls that taking a
    /// choice away.
    ///
    /// What each means where it is acted on is `alo-ask`'s, and
    /// `docs/features.md` carries the promise.
    #[must_use]
    pub const fn may_use_another(&self) -> Option<MayUseAnother> {
        self.may_use_another
    }

    /// The same settings, with what they said about using something else.
    ///
    /// A builder rather than a sixth argument to [`Settings::of`], which
    /// fifteen call sites would otherwise all have to pass [`None`] to — and a
    /// sixth argument nobody has an opinion about is a sixth chance to pass the
    /// wrong one.
    #[must_use]
    pub const fn and_may_use_another(mut self, said: Option<MayUseAnother>) -> Self {
        self.may_use_another = said;
        self
    }

    /// The weights this person brought to this machine themselves.
    ///
    /// Empty is a machine nobody has pointed at weights of their own, which is
    /// most of them. It is a list rather than a question because what is on it
    /// is read by a settings panel offering somebody a choice, and
    /// `alo_models::Brought::for_the_agent` is what that panel offers from.
    #[must_use]
    pub fn brought(&self) -> &Brought {
        &self.brought
    }

    /// The weights this person's choice names, where it names their own list.
    ///
    /// [`None`] for a person who chose nothing and for one who chose from the
    /// catalogue — **and never for one who chose weights**, because
    /// [`Settings::of`] refuses a choice its own list cannot answer. That is
    /// the guarantee this method exists to hand on: whoever holds it does not
    /// have to decide what to do about a name nobody can look up.
    ///
    /// What it is for is the entry rather than the name: the grade a
    /// measurement earned and what the weights cost on this machine are on it,
    /// and both are read beside the name a runtime is asked for.
    #[must_use]
    pub fn weights(&self) -> Option<&Weights> {
        let chosen = self.chosen.as_ref()?.on_this_machine()?;
        if chosen.which() != Which::Brought {
            return None;
        }
        self.brought.get(chosen.model())
    }

    /// The provider this person chose, where that is what they chose.
    ///
    /// Resolved against their own list, so what comes back is the provider
    /// itself — its address, the region whoever added it **stated**, and the
    /// keyring name its credential lives under. [`Settings::of`] has already
    /// refused a choice that names a provider the list does not have, so a
    /// [`Picked::FromAProvider`] here always resolves.
    #[must_use]
    pub fn provider(&self) -> Option<&alo_models::Provider> {
        self.providers.get(self.chosen.as_ref()?.provider()?)
    }

    /// The languages this person reads, best first.
    ///
    /// Empty is a person who has not said, which is not the same as a person
    /// who reads English: `alo-strings` holds English as the **source** rather
    /// than as a default, so a machine with no language named shows the source
    /// and every string on it answers `Said::is_translated` with `false`.
    #[must_use]
    pub fn languages(&self) -> &[Language] {
        &self.languages
    }

    /// The settings in the file at this path.
    ///
    /// **A file that is not there is a person who has not chosen**, and comes
    /// back as [`untouched`](Self::untouched) rather than as a refusal. That is
    /// deliberately the opposite of `alo_keeping::Reading`, where a missing
    /// record is refused: a record is evidence alo OS wrote and its absence is
    /// something to answer for, while a settings file is one somebody may
    /// simply never have made.
    ///
    /// **Nothing about who owns the file is asked here.** It is under the
    /// person's own home directory and is read by a service running as them
    /// (ADR 0001 §2), so the question `alo-agentd`'s `trusting` asks about the
    /// organisation's description — *could somebody else have written this* —
    /// has a different answer and a different owner. Where that is checked at
    /// all, it is checked by whoever knows what a login is, which is not this
    /// crate.
    ///
    /// # Errors
    ///
    /// [`NotSet`], which is every way a file that **is** there is not settings.
    /// Nothing in it is honoured in any of them.
    pub fn at(path: &Path) -> Result<Self, NotSet> {
        match std::fs::read_to_string(path) {
            Ok(said) => read(&said, path),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Ok(Self::untouched()),
            Err(why) => Err(NotSet::NotRead {
                at: path.to_owned(),
                why: why.to_string(),
            }),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::chosen::Chosen;
    use alo_models::Driving;

    /// One set of weights on somebody's own list, measured well enough to be
    /// given a turn.
    fn theirs(id: &str) -> Brought {
        let mut brought = Brought::default();
        brought
            .add(Weights::checked(id, 4_000_000_000).unwrap().measured(
                Driving::Reliably,
                alo_models::MeasuredOn {
                    machine: "a test machine, 16 GB".to_owned(),
                    date: "2026-09-14".to_owned(),
                    runtime: "Ollama 0.34.0".to_owned(),
                    drove: None,
                    of: None,
                    loaded_bytes: None,
                    on_the_gpu_bytes: None,
                    held_to: None,
                    instructions: None,
                },
            ))
            .unwrap();
        brought
    }

    /// **A machine nobody has configured has chosen nothing and said nothing**,
    /// which is the state most machines are in the first time somebody asks
    /// their agent a question.
    #[test]
    fn a_machine_nobody_has_configured_has_chosen_nothing() {
        let settings = Settings::untouched();
        assert!(settings.chosen().is_none());
        assert!(settings.languages().is_empty());
        assert!(settings.brought().weights.is_empty());
        assert!(settings.weights().is_none());
        assert_eq!(settings.setup(), Setup::NotAnswered);
    }

    /// **A person who declined and a person nobody has asked are two different
    /// machines**, and the difference is the only thing `setup` records: both
    /// have chosen nothing, and only one of them is finished.
    #[test]
    fn a_declined_setup_is_not_a_machine_nobody_has_asked() {
        let declined = Settings::of(
            None,
            Brought::default(),
            Providers::default(),
            Vec::new(),
            Setup::Answered,
        )
        .unwrap();

        assert!(declined.chosen().is_none());
        assert!(declined.setup().is_answered());
        assert_ne!(declined, Settings::untouched());
    }

    /// What went in comes back out, which is what the daemon is handed.
    #[test]
    fn settings_say_back_what_somebody_chose() {
        let settings = Settings::of(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )),
            theirs("my-finetune"),
            Providers::default(),
            vec![Language::written("pt-BR").unwrap()],
            Setup::NotAnswered,
        )
        .unwrap();
        assert_eq!(settings.chosen().unwrap().model(), "my-finetune");
        assert_eq!(
            settings
                .chosen()
                .unwrap()
                .on_this_machine()
                .unwrap()
                .which(),
            Which::Brought
        );
        assert_eq!(
            settings
                .languages()
                .iter()
                .map(Language::tag)
                .collect::<Vec<_>>(),
            ["pt-BR"]
        );
    }

    /// **A choice of brought weights resolves to the entry itself**, which is
    /// what carries the measurement and the cost — the name alone would be a
    /// second lookup somebody could get wrong.
    #[test]
    fn a_choice_of_brought_weights_comes_back_as_the_weights() {
        let settings = Settings::of(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )),
            theirs("my-finetune"),
            Providers::default(),
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap();
        let weights = settings.weights().unwrap();
        assert_eq!(weights.id, "my-finetune");
        assert!(weights.can_be_the_agent());
    }

    /// **A settings value cannot say its questions are answered by weights that
    /// are not on its list.** The refusal names what was asked for, so whoever
    /// is holding the file can find the line.
    #[test]
    fn a_choice_naming_weights_nobody_brought_is_not_settings() {
        let refused = Settings::of(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Brought, "my-finetune").unwrap(),
            )),
            theirs("something-else"),
            Providers::default(),
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap_err();
        assert_eq!(refused.named(), "my-finetune");

        // And an empty list is the same answer, which is the shape it really
        // arrives in: somebody wrote the choice and never wrote the list.
        assert!(
            Settings::of(
                Some(Picked::OnThisMachine(
                    Chosen::of(Which::Brought, "my-finetune").unwrap()
                )),
                Brought::default(),
                Providers::default(),
                Vec::new(),
                Setup::NotAnswered,
            )
            .is_err()
        );
    }

    /// **The catalogue is not cross-checked here, and that is not an
    /// oversight.** The catalogue ships with the release rather than living in
    /// this file, and `alo_models::ModelRuntime::answers` has never been gated
    /// by it — a model already on somebody's own disk is theirs. So a
    /// catalogued choice is carried as written, and the one list this file can
    /// contradict itself about is the one this file holds.
    #[test]
    fn a_catalogued_choice_is_carried_as_written_and_names_no_weights() {
        let settings = Settings::of(
            Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "a-model-nobody-here-lists").unwrap(),
            )),
            theirs("my-finetune"),
            Providers::default(),
            Vec::new(),
            Setup::NotAnswered,
        )
        .unwrap();
        assert_eq!(
            settings.chosen().unwrap().model(),
            "a-model-nobody-here-lists"
        );
        assert!(settings.weights().is_none());
        // The list is still theirs and is still here, which is what a settings
        // panel offering them a different choice reads.
        assert_eq!(settings.brought().weights.len(), 1);
    }

    /// **A file that is not there is not a refusal.** The path here cannot
    /// exist, and what comes back is a person who has not chosen.
    #[test]
    fn settings_nobody_has_written_read_as_a_person_who_has_not_chosen() {
        let nowhere = std::env::temp_dir()
            .join("alo-choosing-nothing-here")
            .join("settings.toml");
        assert_eq!(Settings::at(&nowhere).unwrap(), Settings::untouched());
    }
}

/// Where a question put on this machine is answered, from the choice recorded
/// in this settings file.
///
/// **The one reading a session needs to make a task of a request.** `alo-shell`
/// may not make it — its own source guard holds every shipped file in that
/// crate to never naming a settings file — so the binary that owns the folder
/// asks this, and the arithmetic lives here where it can be tested rather than
/// in a binary that has no tests at all.
///
/// `at` is the **settings file**, which is `crate::where_it_is`' answer and not
/// `crate::where_the_folder_is`'. The two differ by one path segment and a
/// caller that passes the folder gets [`None`] from a file that will not read —
/// *no task, no window, and nothing saying why.* That mistake was made once on
/// the way to writing this, and giving it a tested home is how it stops being
/// available to make.
///
/// # Three ways this answers nothing, and all three are the right answer
///
/// **No file, or one that does not read.** A session with no choice recorded
/// cannot answer a question.
///
/// **Nothing chosen.** Somebody who has not picked what answers their
/// questions has not been asked to yet.
///
/// **A provider this file names no entry for.** [`Picked::source`] answers
/// `None` and this passes it through **unchanged** — never *this machine
/// instead*. Answering locally a question somebody pointed at a provider is the
/// silent switch between local and remote processing that law 1 forbids, and it
/// would be invisible: everything a person could see would look right.
#[must_use]
pub fn what_answers_here(at: &std::path::Path) -> Option<alo_models::InferenceSource> {
    let settings = Settings::at(at).ok()?;
    let picked = settings.chosen()?;
    // **The one provider a settings file can name, in a list of its own.**
    // `Picked::source` takes a list because a choice and a list can be checked
    // against each other; a file holds at most one, and a choice naming any
    // other provider correctly answers nothing.
    let mut providers = alo_models::Providers::default();
    if let Some(provider) = settings.provider() {
        providers.add(provider.clone()).ok()?;
    }
    picked.source(&providers)
}

/// **The name of the model the person chose, as they wrote it**, from the
/// settings file at `at` — for a task window's model chip, which shows *what
/// the person configured; nothing is branded* (the owner, 2026-10-31).
///
/// [`None`] where no file reads, nothing is chosen, or a machine down the
/// corridor is (`Picked::the_name_the_person_gave`). Read from the same file
/// [`what_answers_here`] reads, so the name and the place a question goes are
/// one person's one choice.
#[must_use]
pub fn the_model_named_here(at: &std::path::Path) -> Option<String> {
    let settings = Settings::at(at).ok()?;
    settings
        .chosen()?
        .the_name_the_person_gave()
        .map(str::to_owned)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on a settings file that will not write is the failure being \
              reported"
)]
mod what_answers_here_tests {
    use std::path::PathBuf;

    use alo_models::InferenceSource;

    use super::{the_model_named_here, what_answers_here};
    use crate::{Choosing, Chosen, Picked, Which};

    /// **The model's name is the one the person wrote**, read from the file
    /// the place a question goes is read from; nothing chosen names nothing.
    #[test]
    fn the_model_named_is_the_one_the_person_chose() {
        let at = a_settings_file("named");
        assert_eq!(the_model_named_here(&at), None, "an unset machine named a model");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();
        assert_eq!(the_model_named_here(&at).as_deref(), Some("mistral-small"));
    }

    /// **A provider's model is named as the person wrote it, and a machine
    /// down the corridor names none** — its own person chooses its model.
    #[test]
    fn a_provider_names_its_model_and_a_paired_machine_names_none() {
        let provider = Picked::from_a_provider("My provider", "gpt-4o").unwrap();
        assert_eq!(provider.the_name_the_person_gave(), Some("gpt-4o"));
        let local = Picked::OnThisMachine(Chosen::of(Which::Brought, "my-finetune").unwrap());
        assert_eq!(local.the_name_the_person_gave(), Some("my-finetune"));
    }

    /// A settings file of our own, in a folder nobody else's test uses.
    fn a_settings_file(what: &str) -> PathBuf {
        let at = std::env::temp_dir()
            .join(format!("alo-what-answers-{what}-{}", std::process::id()))
            .join(crate::THE_FOLDER)
            .join(crate::THE_SETTINGS);
        drop(std::fs::remove_dir_all(at.parent().unwrap()));
        at
    }

    /// **A chosen local model answers on this machine.**
    #[test]
    fn a_chosen_local_model_answers_here() {
        let at = a_settings_file("local");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();

        assert_eq!(
            what_answers_here(&at),
            Some(InferenceSource::ThisMachine),
            "a recorded local choice did not answer"
        );
    }

    /// **A machine nobody has configured answers nothing**, rather than
    /// answering *this machine* because that is the cheap guess.
    #[test]
    fn a_machine_nobody_has_configured_answers_nothing() {
        let at = a_settings_file("untouched");

        assert_eq!(what_answers_here(&at), None);
        assert!(!at.exists(), "asking wrote a file");
    }

    /// **The folder is not the file**, and passing the folder answers nothing.
    ///
    /// The mistake this function exists to make untakeable: `where_it_is` gives
    /// the file and `where_the_folder_is` gives the folder, they differ by one
    /// segment, and a caller that passes the folder would get no task, no
    /// window, and nothing saying why.
    #[test]
    fn the_folder_is_not_the_file_and_answers_nothing() {
        let at = a_settings_file("folder-not-file");
        let mut choosing = Choosing::at(&at).unwrap();
        choosing
            .answered_by(Some(Picked::OnThisMachine(
                Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            )))
            .unwrap();
        let folder = at.parent().unwrap();

        assert!(
            what_answers_here(&at).is_some(),
            "the file itself must answer, or this test is measuring nothing"
        );
        assert_eq!(
            what_answers_here(folder),
            None,
            "the folder answered as though it were the file"
        );
    }
}
