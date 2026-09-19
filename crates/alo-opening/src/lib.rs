//! What this machine can do with a file, and what it cannot.
//!
//! Before anything converts, opens or prints, the machine has to be able to
//! answer one question honestly: *given this file, what are my options?* This
//! crate is that answer, and nothing else.
//!
//! ```
//! use std::ffi::OsStr;
//! use std::io::Cursor;
//!
//! use alo_opening::{Cannot, Decided, Kind, Named, Outcome, ThisMachine, decide};
//!
//! // What this machine has is handed in by whoever knows what is installed.
//! let machine = ThisMachine::with_nothing().opens(Kind::Pdf).map_err(|_| "a machine")?;
//!
//! // A PDF opens as it is.
//! let mut pdf = Cursor::new(b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n".to_vec());
//! assert!(matches!(
//!     decide(&mut pdf, OsStr::new("letter.pdf"), &machine).map_err(|_| "read")?,
//!     Decided::AsItIs(Outcome::OpensAsItIs { kind: Kind::Pdf, .. })
//! ));
//!
//! // A program named as one is the finding, and is never opened.
//! let mut program = Cursor::new(b"\x7fELF\x02\x01\x01\0".to_vec());
//! assert_eq!(
//!     decide(&mut program, OsStr::new("invoice.pdf"), &machine).map_err(|_| "read")?,
//!     Decided::NotWhatItsNameSays {
//!         named: Named::A(Kind::Pdf),
//!         outcome: Outcome::CannotOpen(Cannot::AProgram),
//!     }
//! );
//! # Ok::<(), &str>(())
//! ```
//!
//! | | |
//! |---|---|
//! | [`decide`] | The question, asked of an open file |
//! | [`Decided`] | The answer, with a lying name as a finding it cannot be read past |
//! | [`Outcome`] | Opens as it is, converts a copy, or cannot — and no *probably* |
//! | [`Cannot`] | Each reason a file cannot be opened, each its own sentence |
//! | [`Would`] | What would open it instead, said after every reason |
//! | [`Costs`] | What converting costs, as far as it is known before it runs |
//! | [`Kind`], [`Appears`], [`Container`], [`Macros`] | What the bytes say |
//! | [`Named`] | What the name claims, read only to notice a lie |
//! | [`ThisMachine`], [`NotAnAbility`] | What this machine has, handed in |
//! | [`Unreadable`] | A file that would not answer, so nothing was decided |
//!
//! # From the content, never from the name
//!
//! An extension is a claim by whoever sent the file, and a machine that trusts
//! it can be handed anything. So what a file is comes from its own bytes —
//! a signature at its start, the list of contents a zip keeps at its end, the
//! directory the older Office format keeps inside it, or text all the way
//! through — and the name is compared afterwards only to notice when it lies.
//! When it does, the lie is reported as exactly that, first, and the outcome for
//! what the file really is comes after it. Nothing is silently corrected.
//!
//! # Deciding reads as little as deciding requires
//!
//! The first 512 bytes; then only what a rule those bytes matched asks for — the
//! last kilobyte of a PDF, a zip's list of contents, a directory's sectors, four
//! bytes of a program's header. No part of any document is decompressed or
//! read. Text is the one rule that reads to the end, because text has no
//! signature and *the first page was text* is a guess about the rest.
//! `tests/deciding_reads_only_what_it_needs.rs` counts the bytes.
//!
//! # What this crate deliberately does not do
//!
//! **It does not open, convert, print or run anything.** Converting is task 2 of
//! `docs/autonomy/v0-5-documents-and-paper-plan.md`, and printing task 3; what
//! they are owed from here is the decision.
//!
//! **It does not take a path.** Which file may be read is a grant, and the
//! crate that holds grants opens the file; this one is handed it.
//!
//! **It asks nothing off this machine.** It depends on `alo-strings` and
//! `thiserror` and nothing else, so no answer here can be an errand.
//! `tests/deciding_never_leaves_the_machine.rs` reads the manifest and the
//! source to hold that.
//!
//! **It does not send a file anywhere to find out what it is.** A file this
//! machine cannot open is explained — what it is, why, and what would open it
//! ([`Cannot::explained`]) — and no explanation offers an upload, a lookup or a
//! service.
//!
//! **It does not guess which older character set a text file is in**, or open
//! a document protected with a password. Both are findings with a sentence, and
//! both are said as what this machine cannot do rather than as a fault in the
//! file.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod appears;
mod compound;
pub mod decided;
pub mod deciding;
mod iso_media;
pub mod kind;
mod looking;
pub mod machine;
pub mod naming;
pub mod outcome;
mod reading;
mod text;
pub mod words;
pub mod would;
mod zip;

#[cfg(test)]
mod testing;

pub use appears::{Appears, Container, Macros};
pub use decided::Decided;
pub use deciding::{Unreadable, decide};
pub use kind::Kind;
pub use looking::THE_HEAD;
pub use machine::{NotAnAbility, ThisMachine};
pub use naming::Named;
pub use outcome::{Cannot, Costs, Outcome};
pub use words::{EVERY_WORD, Word, WordsError, declare_into, opening_words};
pub use would::Would;
