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

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod catalogue;
pub mod ollama;
pub mod provider;
pub mod runtime;
pub mod source;

pub use catalogue::{Catalogue, CatalogueError, CommercialUse, Licence, Model};
pub use ollama::Ollama;
pub use provider::{Provider, ProviderError, Providers, SecretRef};
pub use runtime::{Installed, Loaded, ModelRuntime, Progress, ProgressSink, RuntimeError};
pub use source::{InferenceSource, Region, SourcePolicy};
