//! `.docx`, `.xlsx`, `.pptx` — opened, and what the conversion cost.
//!
//! The documents people are actually sent, converted on this machine into a PDF
//! copy by a rented engine that can reach nothing, with **what the copy could
//! not carry said by name**. [ADR 0039] is the decision this crate is the code
//! of: option A, a converting service of our own that runs the engine with no
//! network and no files, reached by a verb that hands it two open descriptors.
//!
//! | | |
//! |---|---|
//! | [`verbs`] | `convert_document(file, into)`, the one thing an agent may ask for |
//! | [`convert()`], [`Done`], [`Converted`], [`NotConverted`] | The verb, carried out |
//! | [`Carried`], [`NotCarried`] | Everything, or not everything and exactly what |
//! | [`Conversion`] | The closed set of three conversions |
//! | [`ConvertingService`] | This machine's converting service, on this machine only |
//! | [`with_what_converts`] | What `alo-opening` is told this machine converts |
//! | [`serving`], [`engine`] | The service's side, and the one file naming the engine |
//! | `inventory` | What an original holds and what its copy contains |
//! | [`recording`] | What the record keeps about a conversion |
//!
//! # Lost nothing, and did not check, cannot read the same
//!
//! The original is inventoried before it is converted and the copy after, from
//! the documents themselves. [`Carried::Everything`] is reached only when both
//! inventories completed and nothing differs; an inventory that cannot complete
//! is a refusal to show the copy. There is no third answer.
//!
//! # The copy is never the original
//!
//! The original is opened read-only. The copy is created in the folder the
//! person chose, under the document's own name ending in `.pdf`, with `O_EXCL`:
//! a name already there is a refusal and nothing is replaced. A copy that
//! converting did not finish is removed by the verb that created it.
//!
//! # Nothing is uploaded
//!
//! The service is reached over a Unix socket on this machine and nowhere else;
//! its unit has no network at all, so a picture a document links from elsewhere
//! is reported as not fetched rather than fetched. If a format could only be
//! converted somewhere else, the answer would be that it cannot be opened.
//!
//! # What this crate deliberately does not do
//!
//! **It draws nothing.** Which window shows the copy, and where the sentences
//! appear, is the shell's.
//!
//! **It converts only the three current formats.** `alo-opening` recognises
//! older Office and OpenDocument files; each further kind is a registration and
//! a test against a real file, in a later change (ADR 0039).
//!
//! [ADR 0039]: https://github.com/aloworld-org/alo-os/blob/main/docs/decisions/0039-a-document-is-converted-by-an-engine-that-can-reach-nothing.md

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod carried;
pub mod conversion;
pub mod converting;
pub mod engine;
mod inventory;
pub mod machine;
pub mod opening;
#[cfg(target_os = "linux")]
mod passing;
pub mod recording;
pub mod service;
#[cfg(target_os = "linux")]
pub mod serving;
pub mod verbs;
pub mod wire;
pub mod words;
mod xml;
mod zip;

#[cfg(test)]
mod testing;

pub use carried::{Carried, Field, FontName, Linked, NotCarried};
pub use conversion::Conversion;
pub use converting::{Converted, Done, NotConverted, convert};
pub use machine::with_what_converts;
pub use service::{ConvertingService, THE_SOCKET, Unconverted};
pub use verbs::{CONVERT_DOCUMENT, converting_verbs, declare_into as declare_verbs_into};
pub use wire::Refusal;
pub use words::{EVERY_WORD, WordsError, converting_words, declare_into};
