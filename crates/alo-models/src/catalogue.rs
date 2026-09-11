//! The catalogue: which models alo OS offers, and what may legally be done
//! with each of them.
//!
//! It is **data, not code** (`docs/decisions/0005-...` reasons the same way
//! about applications): adding a model is editing `data/catalogue.toml`, not
//! cutting a release. What lives here is the shape that data must have, and
//! the refusal to load data that does not have it.
//!
//! The licence is not decoration. `docs/features.md` promises "a curated
//! catalogue of open-weight models **with their licences stated**", and a
//! catalogue that quietly offers a model an organisation may not use
//! commercially has done something worse than omit it. So a model without an
//! explicit licence and an explicit commercial-use answer cannot be
//! represented — [`Model`] has no default for either, and a catalogue missing
//! one fails to load rather than loading with a blank.
//!
//! [`Driving`] is held to exactly that rule, one field further on, and for the
//! reason ADR 0007's *since it was accepted* section gives: everything else
//! here is about whether a model will **run**, and an entry that said nothing
//! about whether it can **work** would be recommending a model that produces
//! sentences and loses structure. Which model a machine actually gives the
//! agent is [`crate::choosing`]; this file is the shape the entry has to have.

use std::collections::BTreeSet;

use serde::Deserialize;

use crate::driving::Driving;

/// The catalogue shipped with the system.
///
/// Parsed at build time into the binary rather than read from disk at runtime:
/// the catalogue is part of what was signed and shipped, and a file an agent
/// could write would be a way to introduce a model nobody curated.
const BUILT_IN: &str = include_str!("../data/catalogue.toml");

/// Why a catalogue could not be read.
#[derive(Debug, thiserror::Error)]
pub enum CatalogueError {
    /// The TOML did not parse, or did not match the shape above.
    #[error("catalogue is not valid: {0}")]
    Malformed(#[from] toml::de::Error),
    /// Two entries claim the same `id`, so a request for it is ambiguous.
    #[error("duplicate model id: {0}")]
    DuplicateId(String),
    /// A field is present but says nothing usable.
    #[error("model {id}: {what}")]
    Invalid {
        /// The offending entry.
        id: String,
        /// What is wrong with it, in words a person can act on.
        what: &'static str,
    },
}

/// What an organisation is permitted to do with a model's weights.
///
/// Deliberately three values rather than a boolean. "Open weights" covers
/// licences that permit commercial use outright (Apache-2.0, MIT), licences
/// that permit it with conditions somebody must actually read, and licences
/// that forbid it — and flattening the middle into either neighbour is how a
/// customer ends up in breach without being told.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CommercialUse {
    /// Permitted, with no condition beyond attribution.
    Permitted,
    /// Permitted, but the licence attaches conditions a person must read.
    /// [`Licence::note`] says which.
    WithConditions,
    /// Not permitted. The model may still be offered — a person may have a
    /// good non-commercial reason — but it is never a default.
    Forbidden,
}

/// How a model behaves with no graphics card, on the machine most people
/// actually have.
///
/// [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md) makes the
/// CPU the default, which means the catalogue has to answer "will this run on
/// my laptop" and not only "will this run well on a card". Judged on a recent
/// eight-core business laptop, because a class that depends on nobody's machine
/// in particular is a class that tells nobody anything.
///
/// It matters more here than it would for a chatbot: an agent turn is several
/// model calls — the first ask, one after each read, one per handoff, one per
/// check — so per-call latency multiplies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OnCpu {
    /// Answers quickly enough that a person does not wait on it. This is what
    /// a default must be.
    Comfortable,
    /// Usable, and noticeably slower. Fine for one question; tiring across a
    /// turn that makes four model calls.
    Workable,
    /// Runs, but nobody should be offered it as a default without a card. Not
    /// hidden — somebody may have a good reason and the patience for it.
    Slow,
}

/// The licence a model's weights are published under.
#[derive(Debug, Clone, Deserialize)]
pub struct Licence {
    /// The licence's name as its publisher writes it.
    pub name: String,
    /// The SPDX identifier where one exists. Custom model licences frequently
    /// have none, and inventing one would be worse than admitting it.
    #[serde(default)]
    pub spdx: Option<String>,
    /// Whether an organisation may use this model commercially.
    pub commercial_use: CommercialUse,
    /// What a person needs to know before relying on it — required whenever
    /// commercial use carries conditions, because that is precisely the case
    /// where a bare licence name tells somebody nothing.
    #[serde(default)]
    pub note: Option<String>,
}

/// One model a person may run.
#[derive(Debug, Clone, Deserialize)]
pub struct Model {
    /// Stable identifier, as passed to the runtime. Never reused for a
    /// different model.
    pub id: String,
    /// What a person sees.
    pub name: String,
    /// Who publishes the weights.
    pub publisher: String,
    /// Parameter count in billions, as the publisher states it.
    pub parameters_b: f32,
    /// The quantisation these weights are in, which is why "it worked for me"
    /// is not a useful bug report without it.
    ///
    /// **Optional, and paired with [`artefact`](Model::artefact).** A
    /// quantisation is the name of a file somebody can fetch, and this field
    /// used to be a bare string that nothing anchored: `teuken-7b-instruct`
    /// said `Q4_K_M` while openGPT-X publishes no GGUF of Teuken at all, so the
    /// entry named an artefact that exists only as strangers' requantisations.
    /// An entry now either says which artefact it means or claims no
    /// quantisation, and [`Catalogue::parse`] refuses the half-statement.
    #[serde(default)]
    pub quantisation: Option<String>,
    /// **The artefact that quantisation names** — what a machine actually
    /// fetches, as the pinned model runtime (ADR 0006) spells it, or a
    /// publisher's own repository where the runtime's library has no entry.
    ///
    /// It is also what a measurement is run against: `ALO_DRIVING_MODEL` takes
    /// this value, and until it existed the mapping from a catalogue `id` to
    /// the name a runtime answers to lived in whoever last ran the
    /// measurement's head.
    ///
    /// [`None`] only for an entry that claims no quantisation either.
    #[serde(default)]
    pub artefact: Option<String>,
    /// Download size in bytes — what the disk actually loses.
    pub download_bytes: u64,
    /// The video memory this needs to run at a useful speed on a graphics card.
    /// A model above a machine's card is offered with its cost visible rather
    /// than hidden.
    pub min_vram_gb: f32,
    /// The system memory this needs to run on the CPU — the question that
    /// decides whether an ordinary laptop can use it at all (ADR 0007).
    pub min_ram_gb: f32,
    /// How it behaves with no graphics card.
    pub on_cpu: OnCpu,
    /// **Whether it can drive the verbs**, measured rather than claimed
    /// (ADR 0007). No serde default: an entry that does not state it fails to
    /// load, and [`Driving::NotMeasured`] is how an entry states that nobody
    /// has run the measurement yet.
    pub drives_verbs: Driving,
    /// The licence, which every entry must state.
    pub licence: Licence,
    /// Where the weights come from. We never redistribute them
    /// (ADR 0005's doctrine applied to models): the machine fetches from
    /// upstream, so the source is part of the record.
    pub upstream: String,
}

impl Model {
    /// Whether this model may be offered to an organisation as a default.
    ///
    /// Conservative on purpose: only an outright permission qualifies. A
    /// licence with conditions may well be usable, but deciding that on
    /// somebody's behalf is not ours to do.
    #[must_use]
    pub fn safe_default_for_business(&self) -> bool {
        self.licence.commercial_use == CommercialUse::Permitted
    }

    /// Whether this model may be given the agent at all.
    ///
    /// Only a model measured driving the verbs dependably
    /// ([`Driving::clears_the_bar`]). Whether it also *runs* on a particular
    /// machine, and whether its licence lets an organisation rely on it, are
    /// the other two questions — [`Catalogue::agent_for_cpu`] asks all three.
    #[must_use]
    pub fn can_be_the_agent(&self) -> bool {
        self.drives_verbs.clears_the_bar()
    }

    /// **The quantisation and the artefact it names, or neither.**
    ///
    /// The road to either half, so that nothing reads a quantisation without
    /// the file it refers to — the shape [`crate::Weights::lines`] uses for the
    /// same kind of reason. [`Catalogue::parse`] already refuses the
    /// half-statement, so this answers [`Some`] exactly when the entry claims a
    /// quantisation at all.
    #[must_use]
    pub fn quantised_at(&self) -> Option<(&str, &str)> {
        Some((self.quantisation.as_deref()?, self.artefact.as_deref()?))
    }
}

/// Every model the system offers.
#[derive(Debug, Clone, Deserialize)]
pub struct Catalogue {
    /// The entries, in the order the catalogue lists them.
    #[serde(rename = "model")]
    pub models: Vec<Model>,
}

impl Catalogue {
    /// The catalogue built into this system image.
    ///
    /// # Errors
    /// [`CatalogueError`] if the built-in catalogue is malformed — which is a
    /// build-time mistake reaching runtime, and is why the test below exists.
    pub fn built_in() -> Result<Self, CatalogueError> {
        Self::parse(BUILT_IN)
    }

    /// Read a catalogue from TOML, refusing anything that does not hold.
    ///
    /// # Errors
    /// [`CatalogueError::Malformed`] if it is not valid TOML of this shape,
    /// [`CatalogueError::DuplicateId`] if two entries share an `id`, and
    /// [`CatalogueError::Invalid`] for an entry that parses but says nothing
    /// usable.
    pub fn parse(text: &str) -> Result<Self, CatalogueError> {
        let catalogue: Self = toml::from_str(text)?;

        let mut seen = BTreeSet::new();
        for model in &catalogue.models {
            if !seen.insert(model.id.as_str()) {
                return Err(CatalogueError::DuplicateId(model.id.clone()));
            }
            let invalid = |what| CatalogueError::Invalid {
                id: model.id.clone(),
                what,
            };
            if model.id.trim().is_empty() {
                return Err(invalid("an id that is blank cannot be asked for"));
            }
            if model.name.trim().is_empty() {
                return Err(invalid(
                    "a model with no name cannot be offered to a person",
                ));
            }
            if model.download_bytes == 0 {
                return Err(invalid("download size must say what the disk will lose"));
            }
            if model.min_vram_gb <= 0.0 {
                return Err(invalid("required video memory must be stated"));
            }
            let stated = |it: &Option<String>| {
                it.as_ref()
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty())
            };
            match (stated(&model.quantisation), stated(&model.artefact)) {
                (Some(_), Some(_)) | (None, None) => {}
                (Some(_), None) => {
                    return Err(invalid(
                        "a quantisation nobody can point at: name the artefact this entry means, \
                         or claim no quantisation",
                    ));
                }
                (None, Some(_)) => {
                    return Err(invalid(
                        "an artefact whose quantisation the entry does not say, which is the half \
                         of the pair a reader needs",
                    ));
                }
            }
            if model.licence.name.trim().is_empty() {
                return Err(invalid("every model states its licence"));
            }
            if model.licence.commercial_use == CommercialUse::WithConditions
                && model
                    .licence
                    .note
                    .as_ref()
                    .is_none_or(|n| n.trim().is_empty())
            {
                return Err(invalid(
                    "a licence with conditions must say what they are, or nobody can honour them",
                ));
            }
            if model.upstream.trim().is_empty() {
                return Err(invalid(
                    "weights are fetched, never redistributed, so say from where",
                ));
            }
        }
        Ok(catalogue)
    }

    /// One model by its id.
    #[must_use]
    pub fn get(&self, id: &str) -> Option<&Model> {
        self.models.iter().find(|m| m.id == id)
    }

    /// The models an organisation may use commercially without reading a
    /// licence first.
    #[must_use]
    pub fn safe_defaults_for_business(&self) -> Vec<&Model> {
        self.models
            .iter()
            .filter(|m| m.safe_default_for_business())
            .collect()
    }

    /// The models that will run well on a machine with this much video memory.
    #[must_use]
    pub fn runnable_with_vram(&self, vram_gb: f32) -> Vec<&Model> {
        self.models
            .iter()
            .filter(|m| m.min_vram_gb <= vram_gb)
            .collect()
    }

    /// The models a machine with no graphics card can run, given its system
    /// memory — the default question on most machines (ADR 0007).
    #[must_use]
    pub fn runnable_on_cpu(&self, ram_gb: f32) -> Vec<&Model> {
        self.models
            .iter()
            .filter(|m| m.min_ram_gb <= ram_gb && m.on_cpu != OnCpu::Slow)
            .collect()
    }

    /// The models this machine could give the agent, before the bar is applied:
    /// they run here, and they may be used without reading a licence first.
    ///
    /// The public answer to *which one* is
    /// [`agent_for_cpu`](Catalogue::agent_for_cpu), in [`crate::choosing`],
    /// which is where the third question — has it been measured driving the
    /// verbs — is asked and where a refusal that names the alternatives is
    /// made.
    #[must_use]
    pub fn to_choose_from_on_cpu(&self, ram_gb: f32) -> Vec<&Model> {
        self.runnable_on_cpu(ram_gb)
            .into_iter()
            .filter(|m| m.safe_default_for_business())
            .collect()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "a failing unwrap is a failing test")]
mod tests {
    use super::*;

    /// The catalogue we ship must load. It is compiled into the binary, so a
    /// malformed one is a mistake that would otherwise reach a customer's
    /// machine and fail there instead of here.
    #[test]
    fn the_built_in_catalogue_loads() {
        let c = Catalogue::built_in().unwrap();
        assert!(!c.models.is_empty(), "an empty catalogue offers nothing");
    }

    /// `docs/features.md` promises licences are stated. This is that promise as
    /// a test rather than as prose — the distinction CLAUDE.md's gate is about.
    #[test]
    fn every_model_states_a_licence_and_answers_the_commercial_question() {
        for m in Catalogue::built_in().unwrap().models {
            assert!(!m.licence.name.trim().is_empty(), "{} has no licence", m.id);
            if m.licence.commercial_use == CommercialUse::WithConditions {
                assert!(
                    m.licence.note.is_some(),
                    "{} has licence conditions nobody can read",
                    m.id
                );
            }
        }
    }

    /// The certified floor in `docs/hardware.md` is 24 GB, and a catalogue
    /// where nothing runs on a certified machine would be a catalogue for a
    /// machine we do not sell.
    #[test]
    fn something_runs_on_the_certified_machine() {
        let c = Catalogue::built_in().unwrap();
        assert!(
            !c.runnable_with_vram(24.0).is_empty(),
            "nothing in the catalogue runs on 24 GB of VRAM"
        );
    }

    #[test]
    fn a_duplicate_id_is_refused_because_asking_for_it_would_be_ambiguous() {
        let two = r#"
[[model]]
id = "same"
name = "One"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
upstream = "https://example.test/one"
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }

[[model]]
id = "same"
name = "Two"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
upstream = "https://example.test/two"
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }
"#;
        assert!(matches!(
            Catalogue::parse(two),
            Err(CatalogueError::DuplicateId(id)) if id == "same"
        ));
    }

    /// The case the type system cannot catch: a licence that says "conditions
    /// apply" and never says which. Loading that would put a customer in
    /// breach while showing them a tidy licence name.
    #[test]
    fn conditions_without_saying_what_they_are_is_refused() {
        let vague = r#"
[[model]]
id = "vague"
name = "Vague"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
upstream = "https://example.test/vague"
licence = { name = "Custom Community Licence", commercial_use = "with-conditions" }
"#;
        assert!(matches!(
            Catalogue::parse(vague),
            Err(CatalogueError::Invalid { .. })
        ));
    }

    /// **A quantisation nobody can point at is refused**, which is rule 4 and
    /// the bug that wrote it: `teuken-7b-instruct` claimed `Q4_K_M` of a model
    /// whose publisher ships no GGUF, so the catalogue named a file it had
    /// never chosen. Both halves of the pair are refused alone — an artefact
    /// with no quantisation beside it is the same claim missing its other half,
    /// and a reader of one without the other learns something untrue.
    ///
    /// Whitespace counts as absent, for [`Model::name`]'s reason one field
    /// over: a field that is there and says nothing is not a statement.
    #[test]
    fn a_quantisation_with_no_artefact_and_an_artefact_with_no_quantisation_are_both_refused() {
        let entry = |lines: &str| {
            format!(
                r#"
[[model]]
id = "pointed"
name = "Pointed"
publisher = "p"
parameters_b = 1.7
{lines}
download_bytes = 1
min_vram_gb = 2.0
min_ram_gb = 3.0
on_cpu = "comfortable"
drives_verbs = "not-measured"
upstream = "https://example.test/pointed"
licence = {{ name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }}
"#
            )
        };

        // Both halves, and neither half: the two shapes an entry may take.
        for stated in [
            "quantisation = \"Q4_K_M\"\nartefact = \"runtime:pointed-q4_K_M\"",
            "",
        ] {
            let catalogue = Catalogue::parse(&entry(stated)).unwrap();
            let model = catalogue.models.first().unwrap();
            assert_eq!(model.quantised_at().is_some(), !stated.is_empty());
        }

        for half in [
            "quantisation = \"Q4_K_M\"",
            "artefact = \"runtime:pointed-q4_K_M\"",
            "quantisation = \"Q4_K_M\"\nartefact = \"   \"",
            "quantisation = \"  \"\nartefact = \"runtime:pointed-q4_K_M\"",
        ] {
            let refused = Catalogue::parse(&entry(half)).unwrap_err();
            assert!(
                matches!(&refused, CatalogueError::Invalid { id, .. } if id == "pointed"),
                "{half} was accepted: {refused}"
            );
        }
    }

    /// **Every entry that claims a quantisation says which artefact it is**,
    /// asked of the catalogue we ship rather than of a fixture — the shape
    /// [`Catalogue::parse`] guarantees, checked where a curator would break it.
    #[test]
    fn the_catalogue_we_ship_points_at_every_quantisation_it_claims() {
        for m in Catalogue::built_in().unwrap().models {
            assert_eq!(
                m.quantised_at().is_some(),
                m.quantisation.is_some(),
                "{} states a quantisation with no artefact behind it",
                m.id
            );
        }
    }

    #[test]
    fn a_model_with_no_upstream_is_refused_because_we_never_redistribute_weights() {
        let nowhere = r#"
[[model]]
id = "nowhere"
name = "Nowhere"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
upstream = "   "
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }
"#;
        assert!(matches!(
            Catalogue::parse(nowhere),
            Err(CatalogueError::Invalid { .. })
        ));
    }

    /// ADR 0007: a machine with no graphics card is a machine this catalogue
    /// has to be able to answer for, so there is something for it to **run**.
    ///
    /// Whether any of it may be given the **agent** is a second question, and
    /// it is `crate::choosing`'s — a model that runs and cannot produce a verb
    /// call is exactly what this catalogue used to recommend.
    #[test]
    fn a_machine_with_no_graphics_card_has_something_it_can_run() {
        let c = Catalogue::built_in().unwrap();
        // 16 GB is an ordinary business laptop, which is the machine this
        // product exists to reach.
        let runs = c.runnable_on_cpu(16.0);
        assert!(!runs.is_empty(), "a laptop with no card must have a model");
        for m in &runs {
            assert!(m.min_ram_gb <= 16.0, "{}", m.id);
            assert_ne!(m.on_cpu, OnCpu::Slow, "{}", m.id);
        }
        assert!(
            !c.to_choose_from_on_cpu(16.0).is_empty(),
            "and one of them must be usable without reading a licence first"
        );
    }

    /// A model nobody should wait on is not offered as a CPU default, however
    /// capable it is. An agent turn is several calls and the waiting multiplies.
    #[test]
    fn a_slow_model_is_never_a_cpu_default() {
        let c = Catalogue::built_in().unwrap();
        for m in c.runnable_on_cpu(64.0) {
            assert_ne!(m.on_cpu, OnCpu::Slow, "{} was offered for CPU use", m.id);
        }
    }

    /// A small machine has something smaller to run rather than nothing, and
    /// never one that will not fit in its memory.
    #[test]
    fn a_smaller_machine_has_a_smaller_model() {
        let c = Catalogue::built_in().unwrap();
        assert!(
            !c.to_choose_from_on_cpu(4.0).is_empty(),
            "even 4 GB must have something"
        );
        for m in c.runnable_on_cpu(4.0) {
            assert!(m.min_ram_gb <= 4.0, "{} does not fit in 4 GB", m.id);
        }
        // More memory never removes a choice a smaller machine had.
        assert!(c.to_choose_from_on_cpu(32.0).len() >= c.to_choose_from_on_cpu(4.0).len());
    }

    /// The commercial gate applies before the agent question is even asked: a
    /// machine with no card must not be quietly handed the one model an
    /// organisation may not use.
    #[test]
    fn what_there_is_to_choose_from_is_licence_gated() {
        let c = Catalogue::built_in().unwrap();
        for m in c.to_choose_from_on_cpu(16.0) {
            assert_eq!(
                m.licence.commercial_use,
                CommercialUse::Permitted,
                "{}",
                m.id
            );
        }
    }

    /// **Every entry states whether it can drive the verbs, and an entry that
    /// does not fails to load.** ADR 0007's *measured rather than claimed*, as
    /// a refusal rather than as a convention — the shape `licence` has had
    /// since this file was written.
    #[test]
    fn an_entry_that_does_not_say_whether_it_drives_the_verbs_is_refused() {
        let silent = r#"
[[model]]
id = "silent"
name = "Silent"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
upstream = "https://example.test/silent"
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }
"#;
        assert!(matches!(
            Catalogue::parse(silent),
            Err(CatalogueError::Malformed(_))
        ));
        assert!(
            Catalogue::parse(&silent.replace(
                "on_cpu = \"workable\"",
                "on_cpu = \"workable\"\ndrives_verbs = \"invented\""
            ))
            .is_err(),
            "a grade nobody defined is not a grade"
        );
        assert!(
            Catalogue::built_in()
                .unwrap()
                .models
                .iter()
                .all(|m| !m.can_be_the_agent() || m.drives_verbs.has_been_measured())
        );
    }

    /// **Nothing in the catalogue we ship claims a measurement nobody ran.**
    ///
    /// It was every entry until 2026-09-04, when `phi-3-mini-instruct` was put
    /// through `alo-driving` against a real runtime and graded `rarely`, and
    /// the four entries small enough for the same box followed it that day. So
    /// the test is a list rather than a blanket, and the list is the thing to
    /// maintain: adding a name to it means having run the measurement, and
    /// `crates/alo-driving/tests/against_a_model_on_this_machine.rs` is how.
    ///
    /// Written this way round on purpose. A test that only checked *some* entry
    /// is measured would pass while seven unmeasured ones quietly grew grades
    /// somebody guessed from a parameter count, which is the one thing
    /// `drives_verbs` exists to stop.
    ///
    /// It caught exactly that once: four grades arrived in `catalogue.toml`
    /// with nothing but a comment in the same file to support them, and this
    /// test held them out of a release until the run was made again and the
    /// grades were the loop's own. A name here is a measurement somebody ran,
    /// not a measurement somebody read about.
    ///
    /// **The list stuck at five for a reason, and then grew for a better
    /// one.** A 7B entry was fetched and put to `alo-driving` on 2026-09-11 and
    /// the box could not carry it: the model loads more slowly than
    /// `alo_models` waits for an answer, and the guest went down under it. So
    /// that run produced no grade and no name was added, which is what this
    /// test is for. The two names after it are the opposite case —
    /// `qwen3-1.7b` and `granite-3.2-2b-instruct`, added the same day, chosen
    /// because their publishers train them for tool calls and constrained
    /// output, and measured here because they fit. Both earned `Rarely`.
    /// `crates/alo-models/tests/the_grade_the_weights_wait_on.rs` and
    /// `crates/alo-models/tests/candidates_the_box_can_hold.rs` hold both
    /// findings to the catalogue, and `docs/quirks.md` has the numbers.
    #[test]
    fn the_catalogue_we_ship_claims_no_measurement_it_did_not_make() {
        /// Every entry anybody has run `alo-driving` against, and the grade it
        /// earned.
        const MEASURED: [(&str, Driving); 7] = [
            ("phi-3-mini-instruct", Driving::Rarely),
            ("llama-3.2-3b-instruct", Driving::Rarely),
            ("qwen2.5-3b-instruct", Driving::Rarely),
            ("gemma-2-2b-instruct", Driving::Rarely),
            ("smollm2-1.7b-instruct", Driving::Rarely),
            ("qwen3-1.7b", Driving::Rarely),
            ("granite-3.2-2b-instruct", Driving::Rarely),
        ];
        for m in Catalogue::built_in().unwrap().models {
            let ran = MEASURED.iter().find(|(id, _)| *id == m.id);
            assert_eq!(
                m.drives_verbs,
                ran.map_or(Driving::NotMeasured, |(_, grade)| *grade),
                "{} claims a grade; was it measured?",
                m.id
            );
        }
    }

    #[test]
    fn only_an_outright_permission_is_a_safe_default() {
        let c = Catalogue::built_in().unwrap();
        for m in c.safe_defaults_for_business() {
            assert_eq!(m.licence.commercial_use, CommercialUse::Permitted);
        }
    }
}
