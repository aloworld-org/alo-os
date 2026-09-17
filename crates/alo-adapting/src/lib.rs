//! **What a guided fine-tune is trained on, and what it may never reach.**
//!
//! `docs/features.md`: *LoRA/QLoRA over a granted folder or a tenant's records,
//! as a flow rather than a toolchain; the dataset, the adapter and the resulting
//! weights never leave the machine.* Task 1 of
//! `docs/autonomy/v0-5-models-a-person-adapts-and-subscribes-to-plan.md`.
//!
//! # The thing this crate exists to prevent
//!
//! **An adapted model contains the documents it was trained on.** Not a summary
//! of them, not statistics about them — the sentences themselves are recoverable
//! from the weights, and that is what fine-tuning is. So sending an adapted
//! model anywhere is **sending the person's documents**, and it is that whether
//! the destination is a stranger's service or alo's own.
//!
//! [`leaving`] is where that is written down and held: this crate has no road to
//! the network at all, a test reads its own source to keep it that way, and the
//! errand such a departure would have to be made under **does not exist yet** —
//! deliberately, and named there rather than worked around.
//!
//! # The base model is never written to
//!
//! What a fine-tune learns lives in an **adapter** beside the model, tied to the
//! granted folder that taught it, applied when the model answers
//! ([ADR 0048](../../../docs/decisions/0048-an-adapter-is-the-learning-and-the-base-weights-are-never-touched.md)).
//! So deleting one adapter takes back what one folder taught and leaves
//! everything else — which is only possible because the bytes were never mixed.
//! [`adapter`] is where that lives, and [`TheBaseIsUntouched`] is what catches a
//! stack that would merge.
//!
//! # A fine-tune is a change, so a person approves it and the machine writes it down
//!
//! Training changes what the machine will say afterwards, so it is proposed and
//! waits for one approval (ADR 0001), and what it trained on is written into the
//! record — *what has my model learned from?* is a question a person must be
//! able to answer a year later. [`learned_from`] is the statement that answers
//! it.
//!
//! # No trainer here
//!
//! This crate decides the **input**. The training itself is a rented
//! fine-tuning stack, configured and never patched, and it is task 2's.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod adapter;
pub mod dataset;
pub mod learned_from;
pub mod leaving;
pub mod words;
pub mod working_folder;

pub use adapter::{Adapter, Composed, TheBaseIsUntouched, TheBaseMoved};
pub use dataset::{Dataset, NotTrainedOn, Skipped};
pub use learned_from::LearnedFrom;
pub use leaving::{WhatItWouldBe, why_an_adapted_model_cannot_simply_be_sent};
pub use working_folder::WorkingFolder;
