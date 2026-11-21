//! **Every application a change names is said by what a person calls it** —
//! on the card they approve and in the line that says it was done without
//! asking, at both levels.
//!
//! On 2026-11-20 a booted machine said *Done without asking you first … open
//! /home/alo/hello.txt in com.vscodium.codium* and *type the text below into
//! "hello.txt" in com.vscodium.codium*. #1393 made the card for a file opened
//! in a file manager name applications as their entries name them; this does
//! the same for every change, with **the same lookup**
//! (`crate::SaysWhatAFileOpensWith::called`), so the two cannot disagree.
//!
//! **Only the sentence changes** (`alo_capability::Call::naming_applications`):
//! what runs, what the grants are asked about and what the record keeps as
//! having run are still the identifier. A machine that was not told what
//! applications are called says the identifier, as before.
//!
//! **And every sentence the executors write about an application** — what the
//! record says was done, and every refusal — names it by the same
//! `Machine::what_a_person_calls`: opening a file
//! (`crate::opening_a_file_in_an_application`), opening, focusing and closing
//! an application (`crate::acting_on_an_application`), and the file chooser
//! (`crate::choosing_on_the_file_chooser`). The owner's rule of 2026-11-21:
//! a rule applies everywhere it is true, not where it was fixed.

use alo_capability::Call;

use crate::turning::Turning;

impl Turning<'_, '_> {
    /// `call`, with each application it names said by what a person calls it,
    /// where this machine can say.
    pub(crate) fn naming_its_applications(&mut self, call: Call) -> Call {
        let machine = self.machine();
        call.naming_applications(&|identifier| Some(machine.what_a_person_calls(identifier)))
    }
}

// Tested through a whole turn, at both levels, in
// `opening_a_file_in_an_application_tests.rs`: *no sentence names an
// application by its identifier*.
