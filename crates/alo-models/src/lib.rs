//! The models alo OS will run, and what may be done with them.
//!
//! alo OS AI exists so that a language model can run on hardware the customer
//! owns — so the first thing the system needs is not a way to *call* a model
//! (`alo-workplace` has spoken to an OpenAI-compatible endpoint for a year),
//! but a way to know **which** models exist, what each one costs in disk and
//! video memory, and what an organisation is legally permitted to do with it.
//!
//! This crate is that knowledge. Downloading, serving and unloading are built
//! on top of it, in their own modules, because a file that both knows what a
//! model *is* and manages a process has two reasons to change (law 4).
//!
//! # Two lists, and only one of them is ours
//!
//! [`Catalogue`] is what alo OS offers, with a licence gate on it, because
//! offering something is what makes its licence ours to state. [`Brought`] is
//! what the person put on their own machine, and it exists so that *the
//! catalogue recommends; it does not gate* is working code rather than a
//! sentence in `docs/features.md`. The differences between a [`Model`] and a
//! set of [`Weights`] are two: what it costs here [warns](Cost) and never
//! refuses, and there is no licence field at all — see [`weights`] for why an
//! *unknown* one would have been worse than none.
//!
//! # What this crate says, and in whose language
//!
//! Everything here that a person reads goes through `alo-strings` (item 9f):
//! [`words`] is the list of it, `said(&Strings)` is the only road to a
//! sentence, and none of the types a person meets has a `Display`. What is left
//! in English is [`CatalogueError`], which refuses the catalogue **this
//! repository ships** and is read by whoever is fixing that file.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod address;
pub mod brought;
pub mod catalogue;
pub mod choosing;
pub mod costing;
pub mod driving;
pub mod ollama;
pub mod provider;
pub mod refusing;
pub mod runtime;
pub mod secret;
pub mod source;
pub mod tried;
pub mod trying;
pub mod weights;
pub mod words;

#[cfg(test)]
mod testing;

pub use brought::Brought;
pub use catalogue::{Catalogue, CatalogueError, CommercialUse, Licence, Model, OnCpu};
pub use choosing::NoAgentHere;
pub use costing::Cost;
pub use driving::Driving;
pub use ollama::Ollama;
pub use provider::{Provider, ProviderError, Providers, SecretRef};
pub use refusing::NotAllowed;
pub use runtime::{Installed, Loaded, ModelRuntime, Progress, ProgressSink, RuntimeError};
pub use secret::{Secret, SecretError};
pub use source::{InferenceSource, Region, SourcePolicy};
pub use tried::{NotTried, Tried};
pub use trying::Trying;
pub use weights::{Weights, WeightsError};
pub use words::{Word, WordsError, declare_into, model_words};
