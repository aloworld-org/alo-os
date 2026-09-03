//! Whether a model can drive the verbs, as a property every catalogue entry
//! states.
//!
//! [ADR 0007](../../../docs/decisions/0007-the-cpu-is-the-default.md), in its
//! *since it was accepted* section, is the whole of why this file exists. An
//! entry states `parameters_b`, `min_ram_gb`, `on_cpu` and `licence` —
//! everything about whether a model will **run**, and nothing about whether it
//! can **work**. An agent turn asks for a typed verb call with valid arguments
//! several times over, and that is the thing small models are worst at:
//! sentences they manage, structure they lose. A model that runs beautifully on
//! a laptop and cannot emit a valid call is useless as an agent, and a
//! catalogue that knew only about memory would recommend it.
//!
//! # It is measured, and the measurement is not in this crate
//!
//! `alo-driving` puts a fixed set of requests to a model and scores what comes
//! back through the daemon's own door and the same validation a real turn does.
//! What it produces is one of these values, and it is copied into
//! `data/catalogue.toml` by whoever added the entry.
//!
//! The split is deliberate. This crate is what a machine reads when it is
//! deciding what to offer somebody, and it must be able to answer that with no
//! model, no socket and no verbs loaded — so the grade is **data here** and
//! **measured there**. `alo-driving` depends on this crate; nothing depends on
//! it.
//!
//! # Not measured is a value, not a missing field
//!
//! [`Driving::NotMeasured`] is [`crate::Region::Unknown`] one file over, and for
//! the same reason: a model nobody has measured is not a model that is probably
//! fine. `serde` has no default for this field, so an entry that says nothing
//! fails to load rather than loading with a blank — and an entry that says
//! *not measured* has stated it.
//!
//! What follows from that is the honest thing rather than the convenient one:
//! **an unmeasured model is not given the agent.** The direction of that error
//! matters. Refusing a model that would in fact have driven the verbs costs
//! somebody a choice they can still make by hand; offering one that cannot
//! costs them an agent that proposes the wrong thing three times out of five,
//! which is a product nobody keeps.

use serde::Deserialize;

/// How dependably a model produces a verb call this machine would act on.
///
/// Four values rather than a boolean, and the fourth is the one that carries
/// the honesty: *nobody has measured this* is a different statement from *this
/// cannot do it*, and flattening them would make the catalogue claim a
/// measurement it never ran.
///
/// The three that are measured are a coarse scale on purpose. A percentage in
/// the catalogue would invite somebody to compare 88% with 91% across two
/// measurements that used different models of laptop, different quantisations
/// and different days; what the machine actually needs to know is whether this
/// model may be given the agent, and the bar for that is one line in
/// `alo-driving`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Driving {
    /// It produces a call this machine would act on almost every time. This is
    /// what a model must be to be given the agent.
    Reliably,
    /// It produces valid calls, and not dependably enough to be given the
    /// agent. Not hidden — somebody may want it for its answers rather than for
    /// its calls.
    Sometimes,
    /// It manages sentences and loses structure. Offered for what it is; never
    /// the agent.
    Rarely,
    /// Nobody has measured it. **Not a synonym for "probably fine"** — see this
    /// file's header for which way that error is allowed to fall.
    NotMeasured,
}

impl Driving {
    /// Whether a model with this grade may be given the agent on a machine.
    ///
    /// Only [`Reliably`](Self::Reliably), and deliberately so: the other three
    /// are *it sometimes works*, *it does not*, and *we do not know*, and none
    /// of those is a thing to hand somebody's files to.
    #[must_use]
    pub fn clears_the_bar(self) -> bool {
        matches!(self, Self::Reliably)
    }

    /// Whether anybody has measured this at all.
    ///
    /// Separate from [`clears_the_bar`](Self::clears_the_bar) because the two
    /// answer different questions for a person: *this machine will not give the
    /// agent to that model* and *this machine does not know whether it could*
    /// send somebody to two different places.
    #[must_use]
    pub fn has_been_measured(self) -> bool {
        !matches!(self, Self::NotMeasured)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Only an outright measurement qualifies. This is the whole of what the
    /// grade is for, so it is a test rather than a sentence.
    #[test]
    fn only_a_model_measured_driving_reliably_may_be_given_the_agent() {
        assert!(Driving::Reliably.clears_the_bar());
        for grade in [Driving::Sometimes, Driving::Rarely, Driving::NotMeasured] {
            assert!(!grade.clears_the_bar(), "{grade:?}");
        }
    }

    /// **Unmeasured is not "probably fine", and it is not "cannot" either.**
    /// A machine that had one word for both would tell somebody their model
    /// failed a test nobody ran.
    #[test]
    fn not_measured_is_its_own_answer_and_not_a_verdict() {
        assert!(!Driving::NotMeasured.has_been_measured());
        assert!(!Driving::NotMeasured.clears_the_bar());
        for grade in [Driving::Reliably, Driving::Sometimes, Driving::Rarely] {
            assert!(grade.has_been_measured(), "{grade:?}");
        }
    }

    /// The four as a catalogue writes them, and nothing else.
    #[test]
    fn the_catalogue_spells_them_the_way_this_file_does() {
        for (written, grade) in [
            ("\"reliably\"", Driving::Reliably),
            ("\"sometimes\"", Driving::Sometimes),
            ("\"rarely\"", Driving::Rarely),
            ("\"not-measured\"", Driving::NotMeasured),
        ] {
            assert_eq!(serde_json::from_str::<Driving>(written).ok(), Some(grade));
        }
        for invented in ["\"probably\"", "\"yes\"", "\"not_measured\"", "true"] {
            assert!(
                serde_json::from_str::<Driving>(invented).is_err(),
                "{invented}"
            );
        }
    }
}
