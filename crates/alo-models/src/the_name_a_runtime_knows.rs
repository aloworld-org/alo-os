//! Turning the name a person chose into the name a runtime answers to.
//!
//! **One question, and nothing in this workspace asked it until now.** A
//! catalogue entry has two names: the `id` a person and every document use,
//! and the `artefact` the runtime holding the weights spells. They are not the
//! same string and they are near enough to look it:
//!
//! | | |
//! |---|---|
//! | catalogue id | `smollm2-1.7b-instruct` |
//! | runtime artefact | `smollm2:1.7b-instruct-q4_K_M` |
//!
//! # What went wrong without it, measured
//!
//! On 2026-10-11 a question was put to a real model on this machine with the
//! catalogue id as `Question::asked`'s `of_model`. The runtime was listening,
//! the model was installed, the policy permitted it, and the answer was
//! `NoModelThere` — because ollama had never heard of `smollm2-1.7b-instruct`.
//! Asking again with the artefact answered in one try.
//!
//! **The refusal was correct and told a person the wrong thing.** *There is no
//! model there* is true of the name that was sent and false of the machine:
//! the weights were on the disk the whole time. A person reading it would go
//! looking for a download.
//!
//! # Why it is central rather than at each call site
//!
//! The owner's direction: *resolve catalogue-to-runtime model names centrally.
//! The UI should select a catalogue entry; the runtime adapter should receive
//! its correct artifact name.* So the conversion happens here and the two ends
//! each hold one kind of name — a surface that offered an artefact to a person
//! would be showing them a quantisation suffix, and an adapter handed an id
//! would ask for a model that does not exist.
//!
//! `crate::ollama::Ollama::catalogue_id` is the other direction and is not its
//! inverse: it strips a `:latest` suffix and nothing else, so what it returns
//! from `installed()` is the **artefact**, not an id. That asymmetry is why
//! `installed()` cannot be searched for a catalogue id, and why this function
//! exists rather than a lookup over that list.
//!
//! # The entry's own artefact, not an alternative
//!
//! `Model::also_at` holds the same weights at other quantisations, each with
//! its own artefact. Its header is explicit that those are *never a way to
//! change which file the entry names*, so this resolves the entry's
//! `artefact` and only that. A caller deliberately choosing another
//! quantisation is choosing a different file and says so by reading `also_at`
//! itself.

use crate::catalogue::Catalogue;

/// Why a catalogue id could not be turned into a runtime's name.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotNamed {
    /// No entry in the catalogue has this id.
    ///
    /// A person cannot have chosen this from a list this catalogue produced,
    /// so it is a fault in whatever supplied the id rather than a missing
    /// download.
    #[error(
        "no model in this catalogue is called `{0}`, so there is no artefact to ask a runtime for"
    )]
    NoSuchModel(String),
    /// The entry exists and names no artefact.
    ///
    /// Legal in the catalogue: an entry may be recorded before anybody has
    /// pinned a file for it, and `crate::catalogue`'s own validation allows
    /// `artefact` to be absent as long as no quantisation or grade claims one.
    /// So this is *not yet* rather than *wrong*, and it is a different thing to
    /// tell a person than a missing model.
    #[error("`{0}` is in this catalogue and names no artefact, so no runtime can be asked for it")]
    NoArtefact(String),
}

/// The name the runtime knows this catalogue entry by.
///
/// # Errors
/// [`NotNamed`], which is two things and they mean different things to a
/// person: an id this catalogue does not have, and an entry that names no file
/// yet. Neither is *the model is not installed* — that is the runtime's answer
/// and it can only be asked once this has succeeded.
pub fn the_name_a_runtime_knows<'a>(
    catalogue: &'a Catalogue,
    id: &str,
) -> Result<&'a str, NotNamed> {
    let model = catalogue
        .get(id)
        .ok_or_else(|| NotNamed::NoSuchModel(id.to_owned()))?;
    model
        .artefact
        .as_deref()
        .ok_or_else(|| NotNamed::NoArtefact(id.to_owned()))
}

#[cfg(test)]
#[path = "the_name_a_runtime_knows_tests.rs"]
mod tests;
