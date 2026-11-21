//! **What a person calls one application**: the one answer every sentence a
//! person reads names an application by.
//!
//! The desktop entry's `Name`, in the person's language where the entry says it
//! in that language ([`crate::what_they_are_called`]), and otherwise the entry's
//! own `Name` ([`crate::read`]) — or, for an application this project packages,
//! our own word for it. **Never the identifier where the entry gives a name**:
//! `com.vscodium.codium` is what a grant is made over, and *VSCodium* is what a
//! person knows.
//!
//! # Why it is declared here, once
//!
//! #1393 made the card for a file opened in a file manager name applications
//! this way, in `alo-agentd`'s `what_files_open_with_here.rs`. On 2026-11-20 a
//! booted machine showed the egress corner, read by `alo-desktop`, saying
//! *VSCodium (com.vscodium.codium), in the copy alo is working in, is sending*:
//! the desktop named the application with [`Application::shown`], which is how
//! a **list** shows one — name and identifier together, so two applications
//! calling themselves the same thing can be told apart — and not how a
//! sentence does. Two readings of one fact had drifted, so both now ask this.
//!
//! **A list still shows both**; this is for a name placed inside a sentence.

use std::path::PathBuf;

use alo_applications::application::Application;
use alo_strings::Strings;

/// **What a person reading `strings` calls the application with this
/// identifier**, as these places' entries name it — or [`None`] where no entry
/// names it anything, and the caller says the identifier, which is all there
/// is.
#[must_use]
pub fn what_a_person_calls_it(
    places: &[PathBuf],
    strings: &Strings,
    identifier: &str,
) -> Option<String> {
    // **In the person's language first**, as a window's band names it, then
    // the entry's own name.
    let languages: Vec<&str> = strings.chain().iter().map(|it| it.tag()).collect();
    if let Some(name) = crate::what_they_are_called(places, &languages).remove(identifier) {
        return Some(name);
    }
    crate::read(places)
        .knows(identifier)
        .and_then(Application::name)
        .map(|called| called.shown(strings))
}

#[cfg(test)]
#[path = "what_a_person_calls_it_tests.rs"]
mod tests;
