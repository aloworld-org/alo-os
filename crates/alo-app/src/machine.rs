//! **Everything a command reaches on the machine**, as one trait, and what it
//! knows before it reaches anything, as one struct — so that every form and
//! every refusal can be run in a test with nothing real behind it.
//!
//! `main.rs` is the one implementation that reaches a real machine: the
//! desktop's door, the accessibility bus, the session bus, `alo-agentd`, the
//! person's record file and the terminal.

use std::path::Path;
use std::time::SystemTime;

use alo_adapters::{AccessibilityTree, Adapters, Delivers, NotDelivered, TreeFault};
use alo_applications::Installed;
use alo_capability::Verbs;
use alo_record::Entry;
use alo_strings::{Said, Strings};

use crate::door::TheDoor;

/// What a command knows before it asks the machine anything, read once when
/// it starts.
#[derive(Debug, Clone, Copy)]
pub struct Knowing<'a> {
    /// What the person reads, in their language.
    pub strings: &'a Strings,
    /// What is installed now.
    pub installed: &'a Installed,
    /// Every adapter alo OS ships.
    pub adapters: &'a Adapters,
    /// Their abilities, as the list alo's own calls are validated against.
    pub verbs: &'a Verbs,
    /// The folder the person's shell is in, which a relative path is taken
    /// from.
    pub standing_in: &'a Path,
}

/// Whether a place to return to was kept before the first act.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum APlace {
    /// It was, and this is what it is called.
    Kept(String),
    /// It was, by a daemon older than the field that names it, so no entry
    /// can name it.
    KeptUnnamed,
    /// It was not, and this is the sentence the person is told why, already
    /// in their language.
    NotKept(String),
}

/// The machine a command acts on.
pub trait Machine {
    /// The moment it is now.
    fn now(&self) -> SystemTime;

    /// The desktop's door.
    fn door(&mut self) -> &mut dyn TheDoor;

    /// This person's accessibility tree, opened the first time it is asked
    /// for and kept for the rest of the command.
    ///
    /// # Errors
    /// [`TreeFault`]: there is none to open.
    fn tree(&mut self) -> Result<&dyn AccessibilityTree, TreeFault>;

    /// This person's session bus, to send a declared ability to.
    ///
    /// # Errors
    /// [`NotDelivered`]: there is none to reach.
    fn bus(&mut self) -> Result<&dyn Delivers, NotDelivered>;

    /// Ask `alo-agentd` to keep a place to return to.
    fn a_place_to_return_to(&mut self, strings: &Strings) -> APlace;

    /// Make sure the person's record can be written, before the first act —
    /// an act that cannot be written down does not happen.
    ///
    /// # Errors
    /// The sentence saying why it cannot.
    fn the_record_is_open(&mut self, strings: &Strings) -> Result<(), Said>;

    /// Write one entry in the person's record.
    ///
    /// # Errors
    /// The sentence saying why it was not written.
    fn keep(&mut self, entry: &Entry, strings: &Strings) -> Result<(), Said>;

    /// The next line on the pipe, without its line ending; [`None`] at its end.
    fn from_the_pipe(&mut self) -> Option<String>;

    /// One line of what the command gives back, on its output.
    fn told(&mut self, line: &str);

    /// One line for the person on the error output: a refusal, a warning.
    fn warned(&mut self, line: &str);
}
