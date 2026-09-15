//! What a printer is, how one is found and set up, and what is wrong with it
//! when it stops.
//!
//! ★ *Printers, solved — found, set up, and fixed when they stop.* The star is
//! on the last clause. Finding a printer is a solved problem everywhere;
//! **saying what is wrong in a sentence a person can act on** is solved
//! nowhere, and it is the reason printing is on a list of promises at all.
//!
//! | | |
//! |---|---|
//! | [`PrintingService`] | This machine's printing service, reached on this machine and nowhere else |
//! | [`find`], [`Found`] | Every printer on the local network and on a cable, found and never added |
//! | [`set_up`], [`CannotSetUp`] | One printer a person chose, set up without a driver being chosen |
//! | [`Reached`], [`Speaks`] | Where a printer is — and so whether printing on it leaves this machine |
//! | [`how_is`], [`Condition`], [`Stopped`], [`Tried`] | What is wrong, as a closed set, each with what to do |
//! | [`this_machines_printer`], [`NoPrinter`] | The one printer this machine prints on |
//! | [`printable`], [`NotPrintable`] | Whether a file is something a printer takes, from its bytes |
//! | [`print()`], [`Printed`], [`NotPrinted`] | The verb, carried out |
//! | [`verbs`] | `print_document`, the one thing an agent may ask for |
//! | [`ipp`] | The wire format the printing service speaks |
//!
//! # Rented, configured, never patched
//!
//! The printing service is CUPS, pinned in the image (ADR 0011). Its backends
//! find printers and its filters turn a document into what each printer
//! speaks. Nothing here patches it, writes a driver or runs a maker's
//! installer, and nothing here starts a program: the service is spoken to in
//! its own protocol over its own socket, in `ipp.rs` and `http.rs`, and every
//! byte this crate sends it is written in those two files.
//!
//! # The machinery has names, and a person meets none of them
//!
//! A printer is known by the name it gave itself. Its queue is derived and
//! never shown, its driver is its own description of itself, and its protocol
//! is the address it was found at. A test reads every sentence in [`words`]
//! for a driver, a queue, a protocol, a status or a code.
//!
//! # A printer is never added silently
//!
//! [`find`] lists and changes nothing; [`set_up`] adds the one printer it is
//! handed; and no verb an agent can ask for reaches either. A machine that
//! acquired a printer by itself has made a place a document can go without
//! anybody choosing it.
//!
//! # A document to a printer across the network is a document leaving
//!
//! [`print()`] refuses to send anything to a printer [`Reached::OnTheNetwork`]
//! without the [`alo_egress::Departing`] the indicator hands out for exactly
//! that egress, so the indicator has lit before a connection opens.
//!
//! # What this crate deliberately does not do
//!
//! **It draws nothing.** The print dialogue and the list of printers are the
//! shell's; this hands them the decisions and the sentences.
//!
//! **It keeps no record.** What was printed, and what was refused, is written
//! down by whatever carries a verb out, from [`Printed`] and [`NotPrinted`],
//! the way every other verb's executor does.
//!
//! **It has never printed on certified hardware.** Its tests put a service that
//! speaks the protocol on this machine's loopback and watch both sides; a real
//! printer on a real machine is owed, and the report says so.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod asking;
pub mod document;
pub mod found;
mod http;
pub mod ipp;
pub mod printer;
pub mod printing;
pub mod reached;
pub mod service;
pub mod setting_up;
pub mod stopped;
pub mod verbs;
pub mod words;

pub use asking::{Condition, NoPrinter, how_is, this_machines_printer};
pub use document::{NotPrintable, Printable, printable};
pub use found::{Found, find};
pub use printer::{Called, Printer};
pub use printing::{NotPrinted, Printed, print};
pub use reached::{Reached, Speaks};
pub use service::{NotThisMachine, PrintingService, THE_SOCKET, Unanswered};
pub use setting_up::{CannotSetUp, set_up, set_up_said};
pub use stopped::{Stopped, Tried};
pub use verbs::{PRINT_DOCUMENT, printing_verbs};
pub use words::{EVERY_WORD, WordsError, declare_into, printing_words};
