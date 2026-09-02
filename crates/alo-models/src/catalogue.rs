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
    /// The video memory this needs to run at a useful speed. `docs/hardware.md`
    /// puts the certified floor at 24 GB, and a model above that on a machine
    /// below it is offered with its cost visible rather than hidden.
    pub min_vram_gb: f32,
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
upstream = "   "
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }
"#;
        assert!(matches!(
            Catalogue::parse(nowhere),
            Err(CatalogueError::Invalid { .. })
        ));
    }

    #[test]
    fn only_an_outright_permission_is_a_safe_default() {
        let c = Catalogue::built_in().unwrap();
        for m in c.safe_defaults_for_business() {
            assert_eq!(m.licence.commercial_use, CommercialUse::Permitted);
        }
    }
}
