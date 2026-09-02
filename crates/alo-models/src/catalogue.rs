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

use std::collections::BTreeSet;

use serde::Deserialize;

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
    pub quantisation: String,
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

    /// What to run on a machine with no graphics card: the largest model that
    /// still answers without making a person wait, that they may use
    /// commercially, and that fits in the memory they have.
    ///
    /// Comfortable before workable, then larger before smaller. A model that
    /// answers slowly is not a better default for being cleverer: an agent turn
    /// makes several calls, and the waiting multiplies.
    #[must_use]
    pub fn default_for_cpu(&self, ram_gb: f32) -> Option<&Model> {
        self.runnable_on_cpu(ram_gb)
            .into_iter()
            .filter(|m| m.safe_default_for_business())
            .max_by(|a, b| {
                let rank = |m: &Model| u8::from(m.on_cpu == OnCpu::Comfortable);
                rank(a)
                    .cmp(&rank(b))
                    .then(a.parameters_b.total_cmp(&b.parameters_b))
            })
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
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
upstream = "https://example.test/one"
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }

[[model]]
id = "same"
name = "Two"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
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
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
upstream = "https://example.test/vague"
licence = { name = "Custom Community Licence", commercial_use = "with-conditions" }
"#;
        assert!(matches!(
            Catalogue::parse(vague),
            Err(CatalogueError::Invalid { .. })
        ));
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
download_bytes = 1
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
upstream = "   "
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }
"#;
        assert!(matches!(
            Catalogue::parse(nowhere),
            Err(CatalogueError::Invalid { .. })
        ));
    }

    /// ADR 0007: the CPU is the default, so the catalogue must be able to
    /// answer "what runs on this laptop" and not only "what runs on a card".
    #[test]
    fn a_machine_with_no_graphics_card_is_offered_something() {
        let c = Catalogue::built_in().unwrap();
        // 16 GB is an ordinary business laptop, which is the machine this
        // product exists to reach.
        let chosen = c.default_for_cpu(16.0);
        assert!(
            chosen.is_some(),
            "a laptop with no card must have a default to run"
        );
        let chosen = chosen.unwrap();
        assert_eq!(chosen.on_cpu, OnCpu::Comfortable, "{}", chosen.id);
        assert!(chosen.safe_default_for_business(), "{}", chosen.id);
        assert!(chosen.min_ram_gb <= 16.0, "{}", chosen.id);
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

    /// A small machine gets a smaller model rather than nothing, and never one
    /// that will not fit in its memory.
    #[test]
    fn a_smaller_machine_is_offered_a_smaller_model() {
        let c = Catalogue::built_in().unwrap();
        let small = c.default_for_cpu(4.0);
        let large = c.default_for_cpu(32.0);
        assert!(small.is_some(), "even 4 GB must be offered something");
        for m in c.runnable_on_cpu(4.0) {
            assert!(m.min_ram_gb <= 4.0, "{} does not fit in 4 GB", m.id);
        }
        // More memory should not produce a *smaller* default.
        if let (Some(s), Some(l)) = (small, large) {
            assert!(l.parameters_b >= s.parameters_b, "{} vs {}", l.id, s.id);
        }
    }

    /// The commercial gate applies to the CPU default too: a machine with no
    /// card must not be quietly handed the one model an organisation may not
    /// use.
    #[test]
    fn the_cpu_default_is_still_licence_gated() {
        let c = Catalogue::built_in().unwrap();
        if let Some(m) = c.default_for_cpu(16.0) {
            assert_eq!(
                m.licence.commercial_use,
                CommercialUse::Permitted,
                "{}",
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
