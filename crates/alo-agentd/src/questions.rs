//! What a question put during a turn is put to, looked for once and held.
//!
//! Everything between a person's choice and an answer already exists:
//! `alo_choosing::Settings` reads what they chose, `Chosen::asking` makes the
//! permission an organisation's rule leaves them,
//! `alo_models::found_on_this_machine` finds the runtime, and
//! `alo_turn::Turning::asking` takes the three. This file is what walks it, and
//! [`crate::doing`] is the one place that hands the result to a turn.
//!
//! # Found, never configured (ADR 0019)
//!
//! There is no address here, no key that could hold one, and no way to write
//! one. [`Questions`] is handed a catalogue and nothing else; where a runtime
//! is, is the adapter's own knowledge, and `found_on_this_machine` answers with
//! a type whose name no caller outside `alo-models` can write — so *point the
//! agent at a machine of my choosing* is refused by the compiler rather than by
//! nobody having added the field yet. A runtime somewhere else is a
//! **provider**, which is already modelled, already shown on the indicator and
//! already bounded.
//!
//! # Once a turn, and only for a turn that asks
//!
//! The two ends of *when* were a service that re-reads the person's file on
//! every question and a service that read it at start and never again. Neither
//! is right: the first pays for a file it almost never needs, and the second
//! means somebody who picks a model in Settings is told nothing answers
//! questions until the machine is restarted.
//!
//! So it is **once a turn, at the first question of that turn**:
//! [`Questions::a_new_turn`] forgets, [`Questions::what_answers`] looks and
//! holds. A turn that asks nothing costs nothing — no file is opened and no
//! runtime is probed, which is the *with no invocation, no context calls*
//! guarantee read one level in. And within a turn the answer cannot change
//! underneath the record: two questions in one turn go to the same place, so
//! the entries the record keeps are about one machine rather than two.
//!
//! # Why *nothing chosen* and *nothing running* are two answers
//!
//! A person who has picked nothing is told to pick something. A person who
//! picked a model, and whose runtime is not up, is told the runtime is not
//! reachable — `alo-models`' own sentence, because it is `alo-models`' fact.
//! Collapsing them would send somebody who *has* chosen back to a settings
//! panel that already agrees with them.
//!
//! # What is deliberately still `None`
//!
//! The bound is what an organisation permits (ADR 0016), and nothing on this
//! machine states one: `docs/contracts/machine-description.md` has no policy
//! key, and queue item 21o is where whether it gains one is decided. Until then
//! a machine no organisation manages passes `None` and is unaffected — and so
//! is every other machine today, because both lists a choice can name are *this
//! machine* and no `alo_models::SourcePolicy` refuses this machine answering on
//! itself. The argument is `Option<SourcePolicy>` rather than absent so that
//! the day a machine states one, this file is a caller's change.
//!
//! `alo_turn::Places` is the same shape of honesty: ADR 0008's *somewhere else
//! to offer* is a provider list this machine keeps nowhere, so the offer beside
//! a refusal is empty rather than invented.

use std::ffi::{OsStr, OsString};

use alo_choosing::{CONFIG_HOME, Chosen, HOME, NotSet};
use alo_models::{
    Catalogue, ModelRuntime, Provider, Secret, SecretRef, SourcePolicy, found_on_this_machine,
};
use alo_secrets::{NotStored, TheBus, TheKeyring};
use alo_turn::Places;

use crate::settings::of_a_session;

/// What an organisation permits on a machine none manages.
///
/// The same constant `alo-choosing` uses for the same reason: *no rule* and
/// *the rule that permits everywhere* are the same answer, and having one of
/// them stand for the other keeps the calling code from branching on it.
const UNMANAGED: SourcePolicy = SourcePolicy::Anywhere;

/// What this machine puts a question to, for the turn that is under way.
#[derive(Debug)]
pub struct Questions {
    /// `$XDG_CONFIG_HOME` as this process has it, or as a test named it.
    config_home: Option<OsString>,
    /// `$HOME`, the same.
    home: Option<OsString>,
    /// What an organisation permits, or `None` where none manages this machine.
    bound: TheBound,
    /// Every model this system offers, which is what a runtime may fetch.
    catalogue: Catalogue,
    /// What this turn's first question found, or `None` before there was one.
    looked: Option<Looked>,
    /// Whose keyring a provider's key is asked of.
    keyring: WhoseKeyring,
}

/// Who set the rule a question is bounded by, kept apart from what the rule
/// says.
///
/// **Three states rather than an `Option<SourcePolicy>`, and the reason is a
/// sentence a person reads.** ADR 0016 gives the bound to the organisation and
/// says a personal machine has *no policy at all — not empty, not permissive by
/// default, absent* — and therefore **nobody to name in a refusal**. An
/// `Option` collapses two different things into `Some`: a rule an administrator
/// wrote, and a rule the machine's owner chose for themselves. Attribution read
/// off that would tell a person an administrator restricted them when nobody
/// did.
///
/// So the origin is carried rather than inferred. Nothing supplies
/// [`Self::ThePersons`] today — there is no key for it and this does not add
/// one — and it exists because the distinction has to be **unrepresentable to
/// get wrong**, not because something writes it yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheBound {
    /// No policy at all, which is ADR 0016's *absent* and the common case.
    Nobodys,

    /// The person's own rule for their own machine. Bounds the same way and
    /// **names no administrator**, because there is none.
    ThePersons(SourcePolicy),

    /// An organisation's, from a file an administrator wrote. The only origin
    /// that earns the attribution.
    AnOrganisations(SourcePolicy),
}

impl TheBound {
    /// What the rule says, whoever set it.
    ///
    /// Absence answers [`SourcePolicy::Anywhere`], because ADR 0016's *absent*
    /// and *permits everything* decide identically — what they do not share is
    /// somebody to attribute a refusal to, which is [`Self::by_an_organisation`]
    /// and is asked separately for exactly that reason.
    #[must_use]
    pub const fn policy(&self) -> &SourcePolicy {
        match self {
            Self::Nobodys => &UNMANAGED,
            Self::ThePersons(policy) | Self::AnOrganisations(policy) => policy,
        }
    }

    /// Whether an **organisation** set it — never how strict it is.
    #[must_use]
    pub const fn by_an_organisation(&self) -> bool {
        matches!(self, Self::AnOrganisations(_))
    }
}

/// Whose keyring a provider's key is asked of.
///
/// Three states rather than an `Option`, because *there is no keyring* and *find
/// the one belonging to whoever this process is* are different instructions and
/// only one of them opens anything. The default is the first, so a test that
/// says nothing about credentials reaches no store at all — including the real
/// one on the machine it runs on, which a `None` meaning *the usual place*
/// would quietly have done.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum WhoseKeyring {
    /// Nobody's. Every lookup is [`NotStored::Unavailable`] and no bus is
    /// opened — which is also an honest machine: one whose image ships no
    /// Secret Service behaves exactly like this.
    #[default]
    Nobodys,

    /// The person this process runs as, found from the kernel's answer about
    /// its own uid. **This is production**, and the only constructor that
    /// selects it is [`Questions::of_this_process`].
    ThisProcess,

    /// One a caller named, which is a test with a keyring of its own.
    ///
    /// Naming a bus here is not a way to redirect the real lookup: production
    /// never passes this, and `alo_secrets::TheBus::of_this_process` reads a
    /// uid from the kernel rather than a name from anywhere.
    On(TheBus),
}

/// What looking found, kept for the rest of the turn.
#[derive(Debug)]
enum Looked {
    /// Nobody has chosen anything on this machine.
    Nothing,
    /// Somebody chose, and no runtime on this machine answered.
    NotRunning,
    /// The person's file is there and does not hold.
    NotSet(NotSet),
    /// The choice, and what will answer for it.
    OnThisMachine {
        /// What the person chose, as they wrote it.
        chosen: Chosen,
        /// The runtime that was found, which nothing here can point anywhere.
        runtime: Box<dyn ModelRuntime>,
    },
    /// A provider the person added, and the model they asked it for.
    ///
    /// Resolved out of their own list when the file was read, so what is held
    /// is the provider itself rather than the name of one — which is what makes
    /// *where does this answer come from* a question with an answer.
    FromAProvider {
        /// The provider, as their list has it.
        provider: Provider,
        /// What they asked it for, exactly as they wrote it.
        model: String,
    },
}

/// What a question in this turn goes to, or why it goes nowhere.
///
/// Borrowed from the [`Questions`] that found it: the runtime is the turn's for
/// as long as the turn lasts, and there is nowhere else to keep it.
#[derive(Debug)]
pub enum WhatAnswers<'a> {
    /// Nobody has chosen a model or a provider.
    Nothing,
    /// Somebody chose a model on this machine and no runtime answered.
    NotRunning,
    /// The person's settings are there and could not be read.
    NotSet(&'a NotSet),
    /// The question goes to a provider the person added.
    ///
    /// **This is the second of the three choices**, and alo's own service is
    /// the same one (ADR 0014). Nothing here decides whether it can be asked;
    /// `crate::doing` does, and refuses when the provider needs a credential
    /// this machine has nowhere to keep.
    FromAProvider {
        /// The provider, as the person's own list has it.
        provider: &'a Provider,
        /// What it is asked for.
        model: &'a str,
        /// What is permitted, and what else could be offered — which is
        /// nothing, honestly.
        places: Places<'a>,
    },
    /// The question is answered here, by this.
    OnThisMachine {
        /// What the person chose, which is also what the runtime is asked for.
        chosen: &'a Chosen,
        /// What answers it.
        runtime: &'a dyn ModelRuntime,
        /// What is permitted, and what else could be offered — which is
        /// nothing, honestly.
        places: Places<'a>,
    },
}

impl Questions {
    /// What the person this process runs as has chosen, once they ask.
    ///
    /// Nothing is read here: the environment is copied, and
    /// [`Questions::what_answers`] is what opens a file.
    #[must_use]
    pub fn of_this_process(catalogue: Catalogue, bound: TheBound) -> Self {
        Self::of_a_session(
            std::env::var_os(CONFIG_HOME),
            std::env::var_os(HOME),
            catalogue,
            bound,
            WhoseKeyring::ThisProcess,
        )
    }

    /// The same, for a session named rather than inherited.
    ///
    /// `keyring` is `None` for the person this process runs as, and `Some` for
    /// a bus a caller names — which is a test with a keyring of its own, and
    /// nothing else. **Naming one is not a way to redirect the real lookup**:
    /// production goes through [`Questions::of_this_process`], which passes
    /// `None`, and `alo_secrets::TheBus::of_this_process` reads a uid from the
    /// kernel rather than a name from anywhere.
    #[must_use]
    pub fn of_a_session(
        config_home: Option<OsString>,
        home: Option<OsString>,
        catalogue: Catalogue,
        bound: TheBound,
        keyring: WhoseKeyring,
    ) -> Self {
        Self {
            config_home,
            home,
            bound,
            catalogue,
            looked: None,
            keyring,
        }
    }

    /// Whether an **organisation** supplied the bound, rather than how strict
    /// the bound happens to be.
    ///
    /// [`TheBound`] carries the origin; this is that fact, asked where a
    /// refusal is worded. A person's own strict rule answers `false`, so a
    /// refusal tells them what the rule is without inventing an administrator.
    #[must_use]
    pub const fn by_an_organisation(&self) -> bool {
        self.bound.by_an_organisation()
    }

    /// Which keyring a key would be asked of, taken before a turn borrows this.
    ///
    /// `None` means *the person this process runs as*, which is production and
    /// is resolved from the kernel at the moment a key is wanted.
    #[must_use]
    pub fn whose_keyring(&self) -> WhoseKeyring {
        self.keyring.clone()
    }

    /// The key a provider's reference names, out of the keyring named.
    ///
    /// **The bus is found at the moment it is needed, never once at startup.**
    /// `/run/user/<uid>` does not exist before somebody signs in, so a daemon
    /// that resolved this when it started would go on telling a person who
    /// signed in afterwards that their machine has no keyring, for as long as
    /// it ran.
    ///
    /// An associated function rather than a method because the caller is
    /// already holding this turn's answer, which borrows the [`Questions`] it
    /// came out of — and a key is wanted in the middle of that.
    ///
    /// # Errors
    /// The four of [`NotStored`], unchanged and un-worded — the sentence for
    /// each is `crate::doing`'s, beside the refusal it becomes.
    pub fn a_key_from(keyring: &WhoseKeyring, named: &SecretRef) -> Result<Secret, NotStored> {
        let ours;
        let bus = match keyring {
            // Nothing is opened, and nothing is asked. A machine with no store
            // and a caller who named none are the same refusal.
            WhoseKeyring::Nobodys => return Err(NotStored::Unavailable),
            WhoseKeyring::On(named) => named,
            WhoseKeyring::ThisProcess => {
                ours = TheBus::of_this_process()?;
                &ours
            }
        };
        TheKeyring::opened(bus)?.look_up(named)
    }

    /// Forget what the last turn found.
    ///
    /// Called where a turn begins rather than where one ends, so that a service
    /// which stopped mid-turn cannot leave a runtime behind for the next one.
    pub fn a_new_turn(&mut self) {
        self.looked = None;
    }

    /// What a question goes to, looked for on the first call of each turn.
    pub fn what_answers(&mut self) -> WhatAnswers<'_> {
        if self.looked.is_none() {
            self.looked = Some(look(
                self.config_home.as_deref(),
                self.home.as_deref(),
                &self.catalogue,
            ));
        }
        let bound = self.bound.policy();
        match &self.looked {
            Some(Looked::OnThisMachine { chosen, runtime }) => WhatAnswers::OnThisMachine {
                chosen,
                runtime: runtime.as_ref(),
                places: Places::under(bound),
            },
            Some(Looked::FromAProvider { provider, model }) => WhatAnswers::FromAProvider {
                provider,
                model,
                places: Places::under(bound),
            },
            Some(Looked::NotSet(why)) => WhatAnswers::NotSet(why),
            Some(Looked::NotRunning) => WhatAnswers::NotRunning,
            // Filled two lines above, so `None` is unreachable rather than
            // meaningful — and it is answered rather than assumed away, which
            // is the rule `crate::serving` states for its own impossible arm.
            Some(Looked::Nothing) | None => WhatAnswers::Nothing,
        }
    }

    /// A machine already holding what a turn would have looked for.
    ///
    /// **A test seam, and it is `cfg(test)` on purpose.** ADR 0019's *no
    /// override key, no environment variable, no advanced address field* is
    /// held here by there being no way to hand this crate a runtime; a
    /// constructor that took one would be that field, reached through a
    /// different door. What a test needs is a runtime that does not require
    /// one to be installed, and what nothing else needs is this.
    #[cfg(test)]
    pub(crate) fn already_found(
        chosen: Chosen,
        runtime: Box<dyn ModelRuntime>,
        bound: TheBound,
    ) -> Self {
        Self {
            config_home: None,
            home: None,
            bound,
            catalogue: Catalogue { models: Vec::new() },
            looked: Some(Looked::OnThisMachine { chosen, runtime }),
            keyring: WhoseKeyring::Nobodys,
        }
    }
}

/// Read the person's file, and find what would answer what it names.
///
/// The catalogue is cloned into the runtime because that is what a runtime is
/// built with — it is the list a fetch is gated against, and a question is
/// never gated against it: weights somebody brought themselves are not in any
/// catalogue and are still theirs to ask (item 25).
fn look(config_home: Option<&OsStr>, home: Option<&OsStr>, catalogue: &Catalogue) -> Looked {
    let settings = match of_a_session(config_home, home) {
        Ok(settings) => settings,
        Err(why) => return Looked::NotSet(why),
    };
    let Some(chosen) = settings.chosen() else {
        return Looked::Nothing;
    };
    // A provider is looked up in the person's own list and not on this machine.
    // Falling through to `found_on_this_machine` here would be the one thing
    // nothing may do: a question the person addressed elsewhere answered by
    // whatever happens to be running locally.
    if let Some(provider) = settings.provider() {
        return Looked::FromAProvider {
            provider: provider.clone(),
            model: chosen.model().to_owned(),
        };
    }
    let Some(local) = chosen.on_this_machine() else {
        // A provider was chosen and the list did not resolve it. `Settings`
        // refuses such a file, so this is unreachable through `Settings::at`
        // and is a refusal rather than an assumption.
        return Looked::Nothing;
    };
    match found_on_this_machine(catalogue.clone()) {
        Some(runtime) => Looked::OnThisMachine {
            chosen: local.clone(),
            runtime: Box::new(runtime),
        },
        None => Looked::NotRunning,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_directory_of_our_own, a_runtime_saying};
    use alo_choosing::Which;
    use alo_models::InferenceSource;

    /// A machine whose person has written this, and nothing else.
    fn a_machine_whose_person_wrote(what: &str, said: &str) -> Questions {
        let config = a_directory_of_our_own(what);
        let folder = config.join(alo_choosing::THE_FOLDER);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join(alo_choosing::THE_SETTINGS), said).unwrap();
        Questions::of_a_session(
            Some(config.into_os_string()),
            None,
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        )
    }

    /// **A machine nobody has configured answers no questions**, and reaching
    /// that answer opened no file and probed no runtime — there was nothing
    /// named to open.
    #[test]
    fn a_person_who_has_chosen_nothing_has_nothing_answering() {
        let mut questions = Questions::of_a_session(
            None,
            None,
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        );

        assert!(matches!(questions.what_answers(), WhatAnswers::Nothing));
    }

    /// **The second of the three choices reaches the daemon**, resolved out of
    /// the person's own list rather than looked for on this machine.
    ///
    /// What comes back is the provider itself — its address and the region
    /// whoever added it stated — because *where did this answer come from* is a
    /// question `crate::doing` has to be able to answer, and a name alone
    /// cannot.
    #[test]
    fn a_provider_the_person_chose_is_what_answers() {
        let mut questions = a_machine_whose_person_wrote(
            "a-provider",
            "format = 2

[answers]
provider = { name = \"Mistral\", model = \"mistral-small-latest\" }

             [[provider]]
name = \"Mistral\"
endpoint = \"https://api.mistral.ai\"
region = \"the EU\"
",
        );

        let WhatAnswers::FromAProvider {
            provider, model, ..
        } = questions.what_answers()
        else {
            unreachable!("a provider the person chose did not reach the daemon")
        };
        assert_eq!(provider.name, "Mistral");
        assert_eq!(provider.endpoint, "https://api.mistral.ai");
        assert_eq!(model, "mistral-small-latest");
        // The credential is referred to and not held: what the daemon has is a
        // name in a keyring, and `crate::doing` refuses rather than sending
        // without it, because there is no store behind that name yet.
        assert_eq!(
            provider.key.as_ref().map(alo_models::SecretRef::as_str),
            Some("provider/Mistral")
        );
    }

    /// **A provider choice never becomes a local answer**, which is the
    /// no-fallback rule at the layer where it would be easiest to break.
    ///
    /// `look` used to end in *find a runtime on this machine*, and a provider
    /// choice that reached it would have been answered by whatever was running
    /// here — a question addressed to a company answered by a model on the
    /// person's own disk, with the answer saying so and nobody having chosen
    /// it. The provider is looked up first and returns before that.
    #[test]
    fn a_provider_choice_does_not_fall_back_to_this_machine() {
        let mut questions = a_machine_whose_person_wrote(
            "no-local-fallback",
            "format = 2

[answers]
provider = { name = \"Mistral\", model = \"m\" }

             [[provider]]
name = \"Mistral\"
endpoint = \"https://api.mistral.ai\"
",
        );

        assert!(
            !matches!(questions.what_answers(), WhatAnswers::OnThisMachine { .. }),
            "a question addressed to a provider was answered on this machine"
        );
        assert!(
            !matches!(questions.what_answers(), WhatAnswers::Nothing),
            "a provider choice read as nobody having chosen anything"
        );
    }

    /// **A choice with no runtime behind it is not *nothing chosen*.** The
    /// person picked a model and this machine has nothing running that can run
    /// it, which is a different sentence and a different thing to go and fix.
    #[test]
    fn a_choice_with_no_runtime_behind_it_says_so() {
        let mut questions = a_machine_whose_person_wrote(
            "nothing-running",
            "format = 1\n\n[answers]\ncatalogue = \"mistral-small\"\n",
        );

        // On a machine where a runtime really is up this is the other answer,
        // and both are correct — what is being tested is that the two are told
        // apart rather than which one this developer's box gives.
        assert!(matches!(
            questions.what_answers(),
            WhatAnswers::NotRunning | WhatAnswers::OnThisMachine { .. }
        ));
    }

    /// **A settings file that does not hold is refused in words**, and the
    /// refusal names the file rather than reporting an empty choice.
    #[test]
    fn a_settings_file_that_does_not_hold_comes_back_as_the_refusal() {
        let mut questions = a_machine_whose_person_wrote("bad-file", "format = 1\n[answers\n");

        let WhatAnswers::NotSet(why) = questions.what_answers() else {
            unreachable!("a file that is not TOML was read as settings")
        };
        assert!(matches!(why, NotSet::NotUnderstood { .. }), "{why:?}");
    }

    /// **What was chosen is what the runtime is asked for**, exactly as the
    /// person wrote it, and the question is answered on this machine.
    #[test]
    fn what_was_chosen_is_carried_through_to_what_answers() {
        let mut questions = Questions::already_found(
            Chosen::of(Which::Brought, "my-finetune").unwrap(),
            a_runtime_saying(Ok("four".to_owned())),
            TheBound::Nobodys,
        );

        let WhatAnswers::OnThisMachine { chosen, places, .. } = questions.what_answers() else {
            unreachable!("a machine holding a runtime answered that it holds none")
        };
        assert_eq!(chosen.model(), "my-finetune");
        assert_eq!(chosen.source(), InferenceSource::ThisMachine);
        assert_eq!(places.policy(), &SourcePolicy::Anywhere);
        assert!(
            places.everywhere_else().is_empty(),
            "somewhere else was offered, and this machine has nowhere else"
        );
    }

    /// **A bound an organisation set is the one that is carried**, rather than
    /// the unmanaged default — and it reaches the offer beside a refusal too,
    /// which is where a person reads who set the rule.
    #[test]
    fn an_organisations_rule_is_what_a_question_is_asked_under() {
        let mut questions = Questions::already_found(
            Chosen::of(Which::Catalogue, "mistral-small").unwrap(),
            a_runtime_saying(Ok("four".to_owned())),
            TheBound::AnOrganisations(SourcePolicy::ThisMachineOnly),
        );

        let WhatAnswers::OnThisMachine { chosen, places, .. } = questions.what_answers() else {
            unreachable!("a machine holding a runtime answered that it holds none")
        };
        assert_eq!(places.policy(), &SourcePolicy::ThisMachineOnly);
        assert!(
            chosen.asking(Some(places.policy())).is_ok(),
            "an organisation's rule refused this machine answering on itself"
        );
    }

    /// **Looking happens once a turn.** A second question in the same turn is
    /// answered from what the first found, which is what keeps two questions in
    /// one turn from being about two different machines.
    #[test]
    fn a_second_question_in_one_turn_does_not_look_again() {
        let mut questions = a_machine_whose_person_wrote("looked-once", "format = 1\n[answers\n");

        assert!(matches!(questions.what_answers(), WhatAnswers::NotSet(_)));

        // The file is repaired underneath, and the turn does not notice: what
        // it decided at its first question is what it is still holding.
        let at = match &questions.looked {
            Some(Looked::NotSet(why)) => why.at().to_path_buf(),
            other => unreachable!("the refusal stopped being a refusal: {other:?}"),
        };
        std::fs::write(&at, "format = 1\n").unwrap();

        assert!(matches!(questions.what_answers(), WhatAnswers::NotSet(_)));
    }

    /// **And a new turn looks again**, which is what a person who has just
    /// chosen a model in Settings depends on.
    #[test]
    fn a_new_turn_reads_the_file_the_person_has_just_written() {
        let mut questions = a_machine_whose_person_wrote("read-again", "format = 1\n[answers\n");
        let WhatAnswers::NotSet(why) = questions.what_answers() else {
            unreachable!("a file that is not TOML was read as settings")
        };
        let at = why.at().to_path_buf();

        std::fs::write(&at, "format = 1\n").unwrap();
        questions.a_new_turn();

        assert!(
            matches!(questions.what_answers(), WhatAnswers::Nothing),
            "the next turn was still holding the last turn's refusal"
        );
    }

    /// **Making one reads nothing and probes nothing.** The daemon builds this
    /// while it is starting, before anybody has invoked anything, and the
    /// guarantee is that a machine nobody has asked makes no calls at all — so
    /// what a settings file says and whether a runtime is up are both still
    /// unasked here.
    #[test]
    fn nothing_is_read_or_probed_while_a_machine_is_being_stood_up() {
        let questions = Questions::of_a_session(
            Some(OsString::from("/home/ada/.config")),
            Some(OsString::from("/home/ada")),
            Catalogue::built_in().unwrap(),
            TheBound::Nobodys,
            WhoseKeyring::Nobodys,
        );

        assert!(
            questions.looked.is_none(),
            "a machine that had been asked nothing had already gone looking"
        );
    }
}
