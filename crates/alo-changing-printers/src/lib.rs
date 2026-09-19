//! Changing this machine's printers — for an agent's proposal a person
//! approved, and for a person's own choice in Settings — through the privileged
//! broker's three printer verbs, and by no other road.
//!
//! ★ *Printers, solved*. Setting a printer up, removing one and choosing the one
//! this machine prints on are changes to the whole machine, so ADR 0001 §2 puts
//! them behind the broker, with no free-form parameter: what crosses its door is
//! a verb and the digest of what the printing service reported, never a
//! printer's address, queue or driver. ADR 0055 is the decision: a printer is
//! named by the name it gave itself, the verbs need no grant and say why, and a
//! person in Settings takes the same road.
//!
//! | | |
//! |---|---|
//! | [`verbs`], [`approved`] | `add_printer`, `remove_printer`, `set_default_printer`: changes an agent proposes and a person approves |
//! | [`Listed`], [`ThisMachinesPrinters`], [`chosen`] | This machine's printers as a change is chosen among them, and the one printer an approved name is |
//! | [`by_hand::picked`] | A person's own choice in Settings, as the same broker verb |
//! | [`carry_out_approved`], [`carry_out_by_hand`], [`TheBroker`] | The one road from an approval to the broker's door (Unix only) |
//! | [`NotChanged`], [`changed_said`] | What a person reads, either way |
//! | [`words`] | Every sentence, with a note for whoever translates it |
//!
//! # Who decides what
//!
//! `alo-printing` decides what a printer is and what is wrong with it. A person
//! decides whether — by approving a sentence, or by choosing in Settings. The
//! broker decides only that what arrived is exactly one of its verbs under an
//! approval, and writes that down before carrying it out. This crate decides
//! which verb to ask for, and says what happened.
//!
//! # What it has not done
//!
//! **It is not yet reached from a turn.** `alo-turn`'s machine carries out the
//! file verbs and offers only those, and it is not this plan's to change; like
//! `print_document` and `install_application`, these verbs are declared,
//! carried out from an approved authority, and tested end to end against a real
//! door, and handing a turn's redeemed approval to [`carry_out_approved`] is
//! the daemon's wiring, owed where the report for this task says.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod by_hand;
#[cfg(unix)]
mod carrying_out;
mod choosing;
mod refusing;
#[cfg(test)]
mod testing;
pub mod verbs;
mod wanted;
pub mod words;

#[cfg(unix)]
pub use carrying_out::{TheBroker, carry_out_approved, carry_out_by_hand};
pub use choosing::{Listed, NotAnswering, ThisMachinesPrinters, chosen};
pub use refusing::{NotChanged, changed_said};
pub use verbs::{
    ADD_PRINTER, NotAPrinterChange, REMOVE_PRINTER, SET_DEFAULT_PRINTER, approved, printer_verbs,
};
pub use wanted::{Change, Wanted};
pub use words::{EVERY_WORD, WordsError, changing_printers_words, declare_into};
