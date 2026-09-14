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

use crate::{
    also_at::AlsoAt, also_under::AlsoUnder, chat_template::ChatTemplate, costing::GIGABYTE,
    driving::Driving, measured_on::MeasuredOn, requantised::Requantised, unmeasured::Unmeasured,
};

/// Bytes per parameter at which a stated size stops being a quantised
/// artefact's and becomes a full-precision release's.
///
/// Rule 5 of `data/catalogue.toml`, as arithmetic. A four-bit artefact costs
/// between half a byte and a little over three-quarters of one per parameter —
/// every quantised entry in the catalogue we ship is between 0.56 and 0.80 —
/// and a `bfloat16` release costs two. Nothing anybody publishes sits at 1.5,
/// so the line separates the two claims without being near either.
///
/// It exists because a size and a quantisation are one claim and the size half
/// had nothing checking it: `teuken-7b-instruct` stated 4.6 GB after it had
/// stopped claiming any quantisation at all, which is a four-bit figure for an
/// artefact the entry no longer names.
const THE_PRECISION_LINE: f64 = 1.5;

/// Below this many bytes per parameter, a size is not any artefact's.
///
/// Two-bit quantisations are the smallest anybody serves, at roughly a third of
/// a byte. A figure under that is a placeholder somebody left in rather than a
/// download anybody will make.
const NO_ARTEFACT_IS_THIS_SMALL: f64 = 0.35;

/// Above this many bytes per parameter, a size is larger than the weights can
/// be: `float32` is four bytes per parameter and nothing is published above it.
const NOTHING_IS_THIS_LARGE: f64 = 4.5;

/// The catalogue shipped with the system.
///
/// Parsed at build time into the binary rather than read from disk at runtime:
/// the catalogue is part of what was signed and shipped, and a file an agent
/// could write would be a way to introduce a model nobody curated.
const BUILT_IN: &str = include_str!("../data/catalogue.toml");

/// Why a catalogue grade that does not name its instructions is refused
/// ([ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)).
const THE_INSTRUCTIONS_UNNAMED: &str = "a grade that does not name the instructions it was earned \
     under: give their SHA-256 as `instructions`, because two grades under different instructions \
     are two measurements";

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
///
/// **A field this shape does not know is refused rather than ignored.** The
/// reason is [`requantised`](Model::requantised): a provenance block spelled
/// `[model.requantized]` would be dropped in silence and the entry would load
/// looking like a first-party artefact, which is the exact confusion ADR 0026
/// exists to prevent. A misspelling anywhere else in an entry has the same
/// shape — a claim a curator made and nothing read.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
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
    /// **Whose artefact that is, when it is not the publisher's own** — rule 6
    /// of `data/catalogue.toml` and
    /// [ADR 0026](../../../docs/decisions/0026-whose-requantisation-this-catalogue-vouches-for.md).
    ///
    /// [`None`] on every entry whose publisher publishes the file it names, and
    /// on every entry that names no file at all. [`Some`] is a deliberate act
    /// with a name, a pin and a note attached, and [`Catalogue::parse`] refuses
    /// it with any of the three missing — see [`crate::Requantised`].
    #[serde(default)]
    pub requantised: Option<Requantised>,
    /// **The chat template these weights are asked through**, where the file
    /// [`artefact`](Model::artefact) names does not carry one — the publisher's
    /// own, with the address they published it at
    /// ([`crate::ChatTemplate`]). Applied when the model is fetched, and
    /// [`None`] on every entry whose file carries its own.
    #[serde(default)]
    pub chat_template: Option<ChatTemplate>,
    /// Download size in bytes — what the disk actually loses, **for the
    /// artefact this entry names**.
    ///
    /// The second half of that sentence is rule 5 of `data/catalogue.toml`, and
    /// it is checked rather than trusted: [`Catalogue::parse`] divides this by
    /// [`parameters_b`](Model::parameters_b) and refuses a figure that belongs
    /// to a precision the entry does not claim. An entry that names a quantised
    /// [`artefact`](Model::artefact) states that artefact's size; an entry that
    /// claims no quantisation states its publisher's own release, which is the
    /// only thing left that a reader can go and check.
    pub download_bytes: u64,
    /// The video memory this needs to run at a useful speed on a graphics card.
    /// A model above a machine's card is offered with its cost visible rather
    /// than hidden.
    ///
    /// Never below [`download_bytes`](Model::download_bytes): a card that
    /// cannot hold the weights cannot run them at any speed, and
    /// [`Catalogue::parse`] refuses the pair.
    pub min_vram_gb: f32,
    /// The system memory this needs to run on the CPU — the question that
    /// decides whether an ordinary laptop can use it at all (ADR 0007).
    ///
    /// Never below [`download_bytes`](Model::download_bytes), for
    /// [`min_vram_gb`](Model::min_vram_gb)'s reason. This is the check that
    /// would have caught `teuken-7b-instruct` stating ten gigabytes beside
    /// weights that are fifteen.
    pub min_ram_gb: f32,
    /// How it behaves with no graphics card.
    pub on_cpu: OnCpu,
    /// **Whether it can drive the verbs**, measured rather than claimed
    /// (ADR 0007). No serde default: an entry that does not state it fails to
    /// load, and [`Driving::NotMeasured`] is how an entry states that nobody
    /// has run the measurement yet.
    pub drives_verbs: Driving,
    /// **Where and when [`drives_verbs`](Model::drives_verbs) was earned** —
    /// the machine, with its memory, the day, and the runtime that served the
    /// weights.
    ///
    /// Present exactly when the entry states a grade, and
    /// [`Catalogue::parse`] refuses it either way round: a grade with no
    /// machine beside it is a claim, and a machine beside a grade nobody ran is
    /// a run nobody can find. [`crate::MeasuredOn`] says what each field is
    /// held to.
    #[serde(default)]
    pub measured: Option<MeasuredOn>,
    /// **Why an entry that says `not-measured` has no grade** — too large for
    /// the machine that measures the catalogue, a file the runtime could not
    /// answer with, or no published weights — and where that was found.
    ///
    /// Refused beside a grade. The catalogue this repository ships gives one to
    /// every unmeasured entry, and a test says so, because a person choosing a
    /// model reads *not measured* as *probably fine* unless they are told why.
    #[serde(default)]
    pub unmeasured: Option<Unmeasured>,
    /// **The grade the same weights earn when an agent turn holds them to the
    /// protocol's envelope** — a different way of asking, so a different
    /// measurement, never written over [`drives_verbs`](Model::drives_verbs)
    /// ([ADR 0032](../../../docs/decisions/0032-a-local-model-is-held-to-the-envelope-not-the-call.md)).
    ///
    /// **Not read by the recommendation.** [`Catalogue::agent_for_cpu`] reads
    /// the grade for the way turns actually ask, and until the agent turn asks a
    /// local model in the envelope that is `drives_verbs`. Present only beside
    /// [`measured_in_the_envelope`](Model::measured_in_the_envelope), with the
    /// counts, and never as `not-measured`.
    #[serde(default)]
    pub drives_verbs_in_the_envelope: Option<Driving>,
    /// Where, when, under which runtime and with what counts that grade was
    /// earned.
    #[serde(default)]
    pub measured_in_the_envelope: Option<MeasuredOn>,
    /// **The same weights at other quantisations, each graded on its own** —
    /// [`crate::AlsoAt`]. Empty on every entry nobody measured at more than one.
    /// Never read by the recommendation, and never a way to change which file
    /// this entry names.
    #[serde(default)]
    pub also_at: Vec<AlsoAt>,
    /// **The same weights asked in the envelope under other instructions, each
    /// graded on its own** — [`crate::AlsoUnder`]
    /// ([ADR 0034](../../../docs/decisions/0034-the-instructions-show-every-door-they-ask-a-model-to-choose.md)).
    /// Empty on every entry nobody measured under more than one set. Never read
    /// by the recommendation.
    #[serde(default)]
    pub also_under: Vec<AlsoUnder>,
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

    /// **Which file this entry's grade was earned against**, or [`None`] when
    /// nobody has measured it.
    ///
    /// The reporting half of ADR 0026: a grade belongs to an artefact and never
    /// to a model in general, so whatever shows [`drives_verbs`](Model::drives_verbs)
    /// can show the file it is about beside it. [`Catalogue::parse`] refuses a
    /// grade on an entry that names no artefact, so this answers [`Some`]
    /// exactly when the entry has been measured.
    #[must_use]
    pub fn graded_against(&self) -> Option<&str> {
        self.drives_verbs
            .has_been_measured()
            .then_some(self.artefact.as_deref())
            .flatten()
    }

    /// **What one parameter costs in the artefact this entry states**, in bytes.
    ///
    /// The arithmetic rule 5 is made of: a four-bit artefact lands near 0.6, a
    /// `bfloat16` release lands at 2.0, and an entry whose size and
    /// quantisation disagree lands on the wrong side of the line between them
    /// (`THE_PRECISION_LINE`). [`Catalogue::parse`] refuses that entry, so
    /// every model in a loaded catalogue answers this consistently with what it
    /// claims.
    ///
    /// [`f64`] rather than [`f32`] because the sizes are ten significant
    /// figures and the parameter counts are two: rounding the dividend to
    /// `f32` would move the last four digits of a size a curator copied off a
    /// publisher's manifest.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a download size is far below f64's exact-integer range"
    )]
    pub fn bytes_per_parameter(&self) -> f64 {
        self.download_bytes as f64 / (f64::from(self.parameters_b) * 1e9)
    }

    /// **What the weights alone occupy**, in the decimal gigabytes the rest of
    /// this crate counts in ([`crate::costing::GIGABYTE`]).
    ///
    /// The floor under [`min_vram_gb`](Model::min_vram_gb) and
    /// [`min_ram_gb`](Model::min_ram_gb), rather than an estimate of what
    /// running costs — [`crate::Cost`] says why nothing here multiplies a
    /// weights size by a number somebody guessed.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "a download size is far below f64's exact-integer range"
    )]
    pub fn weights_gb(&self) -> f64 {
        self.download_bytes as f64 / GIGABYTE as f64
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
            if !model.parameters_b.is_finite() || model.parameters_b <= 0.0 {
                return Err(invalid(
                    "a parameter count must be stated, or the size beside it can be checked \
                     against nothing",
                ));
            }
            if model.min_vram_gb <= 0.0 {
                return Err(invalid("required video memory must be stated"));
            }
            if model.min_ram_gb <= 0.0 {
                return Err(invalid(
                    "required system memory must be stated: it is the question that decides \
                     whether an ordinary laptop can run this at all",
                ));
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
            // Rule 6: a grade belongs to the file it was earned against, so an
            // entry that names no file may not carry one. `artefact` is what
            // `ALO_DRIVING_MODEL` was set to, and a measurement with nothing
            // behind it is a number about no weights in particular — which is
            // the shape ADR 0007's "measured by us" breaks in if a grade can
            // float free of what was measured.
            if model.drives_verbs.has_been_measured() && model.quantised_at().is_none() {
                return Err(invalid(
                    "a grade on an entry that names no artefact: a measurement is earned against a \
                     file, so name the one it was run against or state no grade",
                ));
            }
            // And on which machine it was earned. A grade is a measurement,
            // and a measurement names its instrument: without one, nobody who
            // disagrees with it knows what to run it on to find out.
            match (model.drives_verbs.has_been_measured(), &model.measured) {
                (true, None) => {
                    return Err(invalid(
                        "a grade with no machine beside it: say which machine the run was made on, \
                         when, and under which runtime, in [model.measured], or the grade is a claim",
                    ));
                }
                (false, Some(_)) => {
                    return Err(invalid(
                        "a machine named beside a measurement nobody ran: an entry that says \
                         not-measured carries no [model.measured]",
                    ));
                }
                (true, Some(measured)) => {
                    if let Some(what) = measured.what_is_wrong_with_it() {
                        return Err(invalid(what));
                    }
                    // The catalogue's grades are re-derivable or they are not
                    // grades: how many attempts drove, of how many, beside each.
                    // `alo-driving` holds the numbers to the bar that made them.
                    if measured.drove.is_none() || measured.of.is_none() {
                        return Err(invalid(
                            "a grade with no counts beside it: say how many attempts drove the \
                             verbs and how many were made, so a reader can re-derive it",
                        ));
                    }
                    if measured.instructions.is_none() {
                        return Err(invalid(THE_INSTRUCTIONS_UNNAMED));
                    }
                }
                (false, None) => {}
            }
            // The envelope's grade: a measurement or nothing, placed and counted.
            match (
                model.drives_verbs_in_the_envelope,
                &model.measured_in_the_envelope,
            ) {
                (None, None) => {}
                (Some(grade), Some(measured)) if grade.has_been_measured() => {
                    if let Some(what) = measured.what_is_wrong_with_it() {
                        return Err(invalid(what));
                    }
                    if measured.drove.is_none() || measured.of.is_none() {
                        return Err(invalid(
                            "an envelope grade with no counts beside it: say how many attempts \
                             drove the verbs and how many were made",
                        ));
                    }
                    if measured.instructions.is_none() {
                        return Err(invalid(THE_INSTRUCTIONS_UNNAMED));
                    }
                }
                (Some(_), Some(_)) => {
                    return Err(invalid(
                        "an envelope grade of not-measured: leave the grade out rather than state \
                         a measurement nobody ran",
                    ));
                }
                (Some(_), None) | (None, Some(_)) => {
                    return Err(invalid(
                        "half an envelope grade: a grade earned in the envelope is written with the \
                         machine it was earned on, or not at all",
                    ));
                }
            }
            match (model.drives_verbs.has_been_measured(), &model.unmeasured) {
                (true, Some(_)) => {
                    return Err(invalid(
                        "a reason an entry was not measured, beside the grade it was measured to: \
                         keep the one that is true",
                    ));
                }
                (false, Some(unmeasured)) => {
                    if let Some(what) = unmeasured.what_is_wrong_with_it() {
                        return Err(invalid(what));
                    }
                }
                (true | false, None) => {}
            }
            for also in &model.also_at {
                if let Some(what) = also.what_is_wrong_with_it(model.quantised_at()) {
                    return Err(invalid(what));
                }
            }
            let the_entrys = model
                .measured_in_the_envelope
                .as_ref()
                .and_then(|on| on.instructions.as_deref());
            for (at, also) in model.also_under.iter().enumerate() {
                if let Some(what) = also.what_is_wrong_with_it(the_entrys) {
                    return Err(invalid(what));
                }
                if model
                    .also_under
                    .iter()
                    .skip(at + 1)
                    .any(|later| later.instructions() == also.instructions())
                {
                    return Err(invalid(
                        "two grades under the same other instructions: write the larger sample \
                         once",
                    ));
                }
            }
            // A template is configuration of a file, so it needs a file, and it
            // is the publisher's or it is not carried.
            if let Some(template) = &model.chat_template {
                if model.quantised_at().is_none() {
                    return Err(invalid(
                        "a chat template beside no artefact: a template is how a file is asked, \
                         and this entry names no file",
                    ));
                }
                if let Some(what) = template.what_is_wrong_with_it() {
                    return Err(invalid(what));
                }
            }
            // And whose file that is, where it is not the publisher's own.
            if let Some(requantised) = &model.requantised
                && let Some(what) = requantised
                    .what_is_wrong_with_it(&model.publisher, model.quantised_at().is_some())
            {
                return Err(invalid(what));
            }
            // Rule 5: the size belongs to the artefact the entry names. A
            // quantisation says which file, and this says the size beside it is
            // that file's — the half of the same claim that had nothing
            // checking it until two entries stopped claiming a quantisation and
            // kept its size.
            let per_parameter = model.bytes_per_parameter();
            if per_parameter < NO_ARTEFACT_IS_THIS_SMALL {
                return Err(invalid(
                    "a size far smaller than any artefact of this many parameters could be: the \
                     bytes and the parameter count are not about the same model",
                ));
            }
            if per_parameter > NOTHING_IS_THIS_LARGE {
                return Err(invalid(
                    "a size larger than this many parameters can be at full precision, so it is \
                     not the weights alone that were measured",
                ));
            }
            match (
                model.quantised_at().is_some(),
                per_parameter < THE_PRECISION_LINE,
            ) {
                (true, true) | (false, false) => {}
                (true, false) => {
                    return Err(invalid(
                        "a full-precision size on an entry that names a quantised artefact: the \
                         size is what that artefact costs, not what the release it came from does",
                    ));
                }
                (false, true) => {
                    return Err(invalid(
                        "a quantised size on an entry that claims no quantisation, which is a \
                         figure for a file this catalogue never chose: state the publisher's own \
                         release, or name the artefact the size belongs to",
                    ));
                }
            }
            let weights_gb = model.weights_gb();
            if f64::from(model.min_vram_gb) < weights_gb {
                return Err(invalid(
                    "a card too small to hold the weights this entry states, which cannot run \
                     them at any speed",
                ));
            }
            if f64::from(model.min_ram_gb) < weights_gb {
                return Err(invalid(
                    "system memory too small to hold the weights this entry states, which is the \
                     figure a person checks their laptop against",
                ));
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
download_bytes = 4_370_000_000
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
measured = { machine = "a test fixture, 16 GB", date = "2026-09-13", runtime = "Ollama 0.34.0", drove = 10, of = 20, instructions = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce" }
upstream = "https://example.test/one"
licence = { name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }

[[model]]
id = "same"
name = "Two"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 4_370_000_000
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
measured = { machine = "a test fixture, 16 GB", date = "2026-09-13", runtime = "Ollama 0.34.0", drove = 10, of = 20, instructions = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce" }
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
download_bytes = 4_370_000_000
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
measured = { machine = "a test fixture, 16 GB", date = "2026-09-13", runtime = "Ollama 0.34.0", drove = 10, of = 20, instructions = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce" }
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
        // Rule 5 binds the size to the claim, so the size travels with it: a
        // four-bit figure for 1.7 billion parameters, and the `bfloat16`
        // release for the shape that claims no quantisation at all.
        let entry = |lines: &str, bytes: u64| {
            format!(
                r#"
[[model]]
id = "pointed"
name = "Pointed"
publisher = "p"
parameters_b = 1.7
{lines}
download_bytes = {bytes}
min_vram_gb = 4.0
min_ram_gb = 6.0
on_cpu = "comfortable"
drives_verbs = "not-measured"
upstream = "https://example.test/pointed"
licence = {{ name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }}
"#
            )
        };
        let four_bit = 1_060_000_000;
        let full_precision = 3_400_000_000;

        // Both halves, and neither half: the two shapes an entry may take.
        for (stated, bytes) in [
            (
                "quantisation = \"Q4_K_M\"\nartefact = \"runtime:pointed-q4_K_M\"",
                four_bit,
            ),
            ("", full_precision),
        ] {
            let catalogue = Catalogue::parse(&entry(stated, bytes)).unwrap();
            let model = catalogue.models.first().unwrap();
            assert_eq!(model.quantised_at().is_some(), !stated.is_empty());
        }

        for half in [
            "quantisation = \"Q4_K_M\"",
            "artefact = \"runtime:pointed-q4_K_M\"",
            "quantisation = \"Q4_K_M\"\nartefact = \"   \"",
            "quantisation = \"  \"\nartefact = \"runtime:pointed-q4_K_M\"",
        ] {
            let refused = Catalogue::parse(&entry(half, four_bit)).unwrap_err();
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

    /// **Every size in the catalogue we ship belongs to the artefact its entry
    /// names**, which is rule 5 asked of the data rather than of a fixture.
    ///
    /// The companion to the test above, one field on: rule 4 made the
    /// quantisation point at a file, and this is the size beside it doing the
    /// same. `crates/alo-models/tests/sizes_an_entry_can_point_at.rs` holds the
    /// two entries that have no artefact to a publisher's own manifest, and
    /// puts each refusal in front of the loader.
    #[test]
    fn the_catalogue_we_ship_states_no_size_from_a_precision_it_does_not_claim() {
        for m in Catalogue::built_in().unwrap().models {
            let per_parameter = m.bytes_per_parameter();
            assert_eq!(
                m.quantised_at().is_some(),
                per_parameter < THE_PRECISION_LINE,
                "{} states {per_parameter:.2} bytes per parameter, which is not what the artefact \
                 it names costs",
                m.id
            );
            assert!(
                f64::from(m.min_ram_gb) >= m.weights_gb()
                    && f64::from(m.min_vram_gb) >= m.weights_gb(),
                "{} asks for less memory than its own weights occupy",
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
download_bytes = 4_370_000_000
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "reliably"
measured = { machine = "a test fixture, 16 GB", date = "2026-09-13", runtime = "Ollama 0.34.0", drove = 10, of = 20, instructions = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce" }
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
download_bytes = 4_370_000_000
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
    ///
    /// **The eighth is the first 7B entry, and the first grade made on a
    /// machine that could hold it**: `qwen2.5-7b-instruct`, on an Apple M3 with
    /// 8 GB on 2026-09-13, 4 of 10. `Rarely`, like the seven before it — and
    /// failing a different way, which `docs/quirks.md` has.
    #[test]
    fn the_catalogue_we_ship_claims_no_measurement_it_did_not_make() {
        /// Every entry anybody has run `alo-driving` against, and the grade it
        /// earned.
        const MEASURED: [(&str, Driving); 13] = [
            ("phi-3-mini-instruct", Driving::Rarely),
            ("llama-3.2-3b-instruct", Driving::Rarely),
            ("qwen2.5-3b-instruct", Driving::Rarely),
            ("gemma-2-2b-instruct", Driving::Rarely),
            ("smollm2-1.7b-instruct", Driving::Rarely),
            ("qwen3-1.7b", Driving::Rarely),
            ("granite-3.2-2b-instruct", Driving::Rarely),
            ("qwen2.5-7b-instruct", Driving::Rarely),
            ("mistral-7b-instruct", Driving::Rarely),
            ("llama-3.1-8b-instruct", Driving::Rarely),
            ("teuken-7b-instruct", Driving::Rarely),
            ("qwen3-4b", Driving::Rarely),
            ("qwen3-8b", Driving::Sometimes),
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

    /// One entry, graded, with `measured` standing in for whatever block the
    /// test puts beside the grade.
    fn graded_with(grade: &str, measured: &str) -> String {
        format!(
            r#"
[[model]]
id = "graded"
name = "Graded"
publisher = "p"
parameters_b = 7.0
quantisation = "Q4_K_M"
artefact = "runtime:tag-q4_K_M"
download_bytes = 4_370_000_000
min_vram_gb = 8.0
min_ram_gb = 10.0
on_cpu = "workable"
drives_verbs = "{grade}"
upstream = "https://example.test/graded"
licence = {{ name = "Apache-2.0", spdx = "Apache-2.0", commercial_use = "permitted" }}
{measured}
"#
        )
    }

    /// The instructions every fixture's grade was earned under.
    const A_DIGEST: &str = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce";

    /// A machine that says all three things.
    const A_MACHINE: &str = r#"
[model.measured]
machine = "Apple M3, 8 GB unified memory"
date = "2026-09-13"
runtime = "Ollama 0.34.0"
instructions = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce"
drove = 8
of = 20
"#;

    /// **A grade with no machine beside it is refused**, and so is a machine
    /// beside a grade nobody ran. The first is the plan's own test; the second
    /// is the same claim made the other way round.
    #[test]
    fn a_grade_with_no_machine_beside_it_is_refused() {
        let with_one = Catalogue::parse(&graded_with("rarely", A_MACHINE)).unwrap();
        let measured = with_one.models.first().unwrap().measured.as_ref().unwrap();
        assert_eq!(measured.machine, "Apple M3, 8 GB unified memory");
        assert_eq!(measured.date, "2026-09-13");
        assert_eq!(measured.runtime, "Ollama 0.34.0");

        for grade in ["reliably", "sometimes", "rarely"] {
            let refused = Catalogue::parse(&graded_with(grade, "")).unwrap_err();
            assert!(
                refused.to_string().contains("no machine beside it"),
                "{grade} with no machine: {refused}"
            );
        }
        let refused = Catalogue::parse(&graded_with("not-measured", A_MACHINE)).unwrap_err();
        assert!(
            refused.to_string().contains("nobody ran"),
            "a machine beside no grade: {refused}"
        );
        assert!(Catalogue::parse(&graded_with("not-measured", "")).is_ok());
    }

    /// **The block is held to what it says**, not only to being present: a
    /// machine with no memory figure is refused through the loader, and a block
    /// spelled wrongly or carrying a field nobody reads fails to load rather
    /// than vanishing.
    #[test]
    fn a_machine_that_says_nothing_checkable_is_refused_by_the_loader() {
        let no_memory = A_MACHINE.replace(", 8 GB unified memory", "");
        let refused = Catalogue::parse(&graded_with("rarely", &no_memory)).unwrap_err();
        assert!(refused.to_string().contains("how much memory"), "{refused}");

        let misspelt = A_MACHINE.replace("[model.measured]", "[model.measurd]");
        assert!(Catalogue::parse(&graded_with("rarely", &misspelt)).is_err());

        let extra = format!("{A_MACHINE}cores = 8\n");
        assert!(Catalogue::parse(&graded_with("rarely", &extra)).is_err());
    }

    /// **Every entry the catalogue ships is graded or says why it is not.**
    /// `not-measured` on its own reads as *probably fine, nobody checked*, and a
    /// person choosing a model is owed the reason.
    #[test]
    fn every_entry_we_ship_is_graded_or_says_why_not() {
        for m in Catalogue::built_in().unwrap().models {
            assert_ne!(
                m.drives_verbs.has_been_measured(),
                m.unmeasured.is_some(),
                "{} is neither graded nor says why it is not",
                m.id
            );
        }
    }

    /// **A reason beside a grade is refused**, and a reason is held to the
    /// same machine, day and runtime a grade is.
    #[test]
    fn a_reason_beside_a_grade_is_refused() {
        let reason = A_MACHINE
            .replace("drove = 8\nof = 20\n", "")
            .replace(&format!("instructions = \"{A_DIGEST}\"\n"), "")
            .replace("[model.measured]", "[model.unmeasured]")
            .replace(
                "machine =",
                "because = \"weights-not-published\"\nmachine =",
            );
        assert!(Catalogue::parse(&graded_with("not-measured", &reason)).is_ok());
        let refused =
            Catalogue::parse(&graded_with("rarely", &format!("{A_MACHINE}{reason}"))).unwrap_err();
        assert!(
            refused.to_string().contains("keep the one that is true"),
            "{refused}"
        );
        let no_memory = reason.replace(", 8 GB unified memory", "");
        let refused = Catalogue::parse(&graded_with("not-measured", &no_memory)).unwrap_err();
        assert!(refused.to_string().contains("how much memory"), "{refused}");
    }

    /// **A catalogue grade with no counts beside it is refused**, because a
    /// grade a reader cannot re-derive is one they have to take on trust.
    #[test]
    fn a_catalogue_grade_with_no_counts_is_refused() {
        let no_counts = A_MACHINE.replace("drove = 8\nof = 20\n", "");
        let refused = Catalogue::parse(&graded_with("rarely", &no_counts)).unwrap_err();
        assert!(refused.to_string().contains("no counts"), "{refused}");
        let half = A_MACHINE.replace("of = 20\n", "");
        let refused = Catalogue::parse(&graded_with("rarely", &half)).unwrap_err();
        assert!(refused.to_string().contains("half a count"), "{refused}");
    }

    /// **A catalogue grade that does not name its instructions is refused**, on
    /// either door — two grades under different instructions are two
    /// measurements (ADR 0034), and a reader cannot tell them apart without it.
    #[test]
    fn a_catalogue_grade_that_does_not_name_its_instructions_is_refused() {
        let unnamed = A_MACHINE.replace(&format!("instructions = \"{A_DIGEST}\"\n"), "");
        let refused = Catalogue::parse(&graded_with("rarely", &unnamed)).unwrap_err();
        assert!(refused.to_string().contains("instructions"), "{refused}");

        let envelope_unnamed = graded_with("rarely", A_MACHINE).replacen(
            "upstream = ",
            "drives_verbs_in_the_envelope = \"sometimes\"\nupstream = ",
            1,
        ) + IN_THE_ENVELOPE
            .split_once("\n\n")
            .map_or("", |(_, table)| table)
            .replace(&format!("instructions = \"{A_DIGEST}\"\n"), "")
            .as_str();
        let refused = Catalogue::parse(&envelope_unnamed).unwrap_err();
        assert!(refused.to_string().contains("instructions"), "{refused}");
        assert!(
            Catalogue::parse(
                &(graded_with("rarely", A_MACHINE).replacen(
                    "upstream = ",
                    "drives_verbs_in_the_envelope = \"sometimes\"\nupstream = ",
                    1,
                ) + IN_THE_ENVELOPE
                    .split_once("\n\n")
                    .map_or("", |(_, table)| table))
            )
            .is_ok(),
            "the same entry with its instructions named loads"
        );
    }

    /// **A grade under other instructions sits beside the entry's own, once.**
    #[test]
    fn a_grade_under_other_instructions_is_read_beside_the_entrys_and_written_once() {
        const UNDER: &str = r#"
[[model.also_under]]
drives_verbs_in_the_envelope = "reliably"

[model.also_under.measured_in_the_envelope]
machine = "Apple M3, 8 GB unified memory"
date = "2026-09-14"
runtime = "Ollama 0.34.0"
drove = 80
of = 80
loaded_bytes = 5_197_833_172
on_the_gpu_bytes = 4_583_210_351
instructions = "93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02"
"#;
        let entry = graded_with("rarely", A_MACHINE).replacen(
            "upstream = ",
            "drives_verbs_in_the_envelope = \"sometimes\"\nupstream = ",
            1,
        ) + IN_THE_ENVELOPE
            .split_once("\n\n")
            .map_or("", |(_, table)| table);

        let read = Catalogue::parse(&(entry.clone() + UNDER)).unwrap();
        let model = read.models.first().unwrap();
        assert_eq!(model.drives_verbs_in_the_envelope, Some(Driving::Sometimes));
        let also = model.also_under.first().unwrap();
        assert_eq!(also.drives_verbs_in_the_envelope, Driving::Reliably);
        assert_ne!(
            also.instructions(),
            model
                .measured_in_the_envelope
                .as_ref()
                .unwrap()
                .instructions
                .as_deref()
        );

        let twice = Catalogue::parse(&(entry.clone() + UNDER + UNDER)).unwrap_err();
        assert!(
            twice.to_string().contains("write the larger sample once"),
            "{twice}"
        );
        let the_entrys = Catalogue::parse(
            &(entry
                + &UNDER.replace(
                    "93a7f458ce9d017d6d12759a281e5f0b347a0d6963c04eae03b4c30581aebd02",
                    A_DIGEST,
                )),
        )
        .unwrap_err();
        assert!(
            the_entrys.to_string().contains("the entry's own"),
            "{the_entrys}"
        );
        let alone = Catalogue::parse(&(graded_with("rarely", A_MACHINE) + UNDER)).unwrap_err();
        assert!(
            alone.to_string().contains("no envelope grade of its own"),
            "{alone}"
        );
    }

    /// An envelope grade and its machine, as a curator writes them.
    const IN_THE_ENVELOPE: &str = r#"
drives_verbs_in_the_envelope = "reliably"

[model.measured_in_the_envelope]
machine = "Apple M3, 8 GB unified memory"
date = "2026-09-14"
runtime = "Ollama 0.34.0"
instructions = "d468e469651d778ae369c53e37816fce62c80f703de729a074bcf8ff44a5adce"
drove = 19
of = 20
"#;

    /// **An envelope grade is a measurement, placed and counted, or it is not
    /// there** — and it never takes the place of the free grade (ADR 0032).
    #[test]
    fn an_envelope_grade_is_placed_counted_and_kept_apart() {
        // The keys belong to the entry, so they go before its first table.
        let with = |extra: &str| {
            graded_with("rarely", A_MACHINE).replacen(
                "upstream = ",
                &format!(
                    "{}\nupstream = ",
                    extra.trim().split("\n\n").next().unwrap()
                ),
                1,
            ) + extra
                .trim()
                .split_once("\n\n")
                .map_or("", |(_, table)| table)
        };
        let read = Catalogue::parse(&with(IN_THE_ENVELOPE)).unwrap();
        let model = read.models.first().unwrap();
        assert_eq!(model.drives_verbs, Driving::Rarely);
        assert_eq!(model.drives_verbs_in_the_envelope, Some(Driving::Reliably));
        assert_eq!(
            model.measured_in_the_envelope.as_ref().unwrap().drove,
            Some(19)
        );

        let half =
            with(IN_THE_ENVELOPE).replace("[model.measured_in_the_envelope]", "[model.nothing]");
        assert!(Catalogue::parse(&half).is_err());
        let only_grade = graded_with("rarely", A_MACHINE).replacen(
            "upstream = ",
            "drives_verbs_in_the_envelope = \"reliably\"\nupstream = ",
            1,
        );
        let refused = Catalogue::parse(&only_grade).unwrap_err();
        assert!(
            refused.to_string().contains("half an envelope grade"),
            "{refused}"
        );
        let unmeasured = with(IN_THE_ENVELOPE).replace(
            "drives_verbs_in_the_envelope = \"reliably\"",
            "drives_verbs_in_the_envelope = \"not-measured\"",
        );
        let refused = Catalogue::parse(&unmeasured).unwrap_err();
        assert!(refused.to_string().contains("nobody ran"), "{refused}");
        let uncounted = with(IN_THE_ENVELOPE).replace("drove = 19\nof = 20", "");
        let refused = Catalogue::parse(&uncounted).unwrap_err();
        assert!(refused.to_string().contains("no counts"), "{refused}");
    }

    /// **The recommendation does not read the envelope's grade** until turns ask
    /// that way: a model that clears the bar only in the envelope is not given
    /// the agent by a machine whose turns do not hold it there.
    #[test]
    fn the_recommendation_reads_the_grade_for_the_way_turns_ask() {
        let text = graded_with("rarely", A_MACHINE).replacen(
            "upstream = ",
            "drives_verbs_in_the_envelope = \"reliably\"\nupstream = ",
            1,
        ) + IN_THE_ENVELOPE.trim().split_once("\n\n").unwrap().1;
        let read = Catalogue::parse(&text).unwrap();
        assert!(!read.models.first().unwrap().can_be_the_agent());
        assert!(read.agent_for_cpu(64.0).is_err());
    }

    /// **Every grade the catalogue ships names the machine it was earned on.**
    /// The loader already refuses one that does not; this says so about the
    /// file a machine is actually built with.
    #[test]
    fn every_grade_we_ship_names_its_machine() {
        for m in Catalogue::built_in().unwrap().models {
            assert_eq!(
                m.drives_verbs.has_been_measured(),
                m.measured.is_some(),
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
