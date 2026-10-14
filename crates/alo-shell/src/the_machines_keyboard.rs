//! Which keyboard this machine has, decided before anybody has signed in.
//!
//! The sign-in screen needs a layout at the moment it draws, and the person who
//! will use it has not identified themselves yet — that is what they are about
//! to do, by typing a password. So the layout cannot come from their settings,
//! and until this file existed it came from `Environment=ALO_KEYBOARD=gb` in
//! `alo-compositor.service`: the same on every machine ever written, and
//! unchangeable by anything running on one.
//!
//! This reads [`alo_keyboards::THE_MACHINES_KEYBOARDS`] when it is there and
//! falls back to the unit file's word when it is not, which is what makes a
//! machine that nobody has set up yet still signable-in.
//!
//! **And something writes it now.** `crate::language_sheet` asks a person
//! which language they speak before it asks for a name, and
//! `crate::the_keyboard_a_person_chose` keeps the answer here — so the file
//! this reads at the next boot is the one they chose at the first, and the
//! fallback below is for a machine on its very first frame rather than for
//! every machine forever. That was not true when this file was written: the
//! choice was reachable and nothing could make it.
//!
//! # Why an unreadable file does not stop the screen
//!
//! `alo_keyboards::keeping` refuses a file that is there and wrong, whole, and
//! honours nothing in it — the right rule for a person's settings, where the
//! alternative is honouring half of a hand edit. Here the alternative is a
//! machine with no sign-in screen on it, so a file that did not read is
//! reported in the log and the unit file's layout is used. The report is the
//! point: [`WhoSaidSo::TheUnitFileBecauseTheMachinesFileDidNotRead`] names
//! the file and what was wrong with it, so the log says which file to
//! mend, rather than a person finding out by typing.

use std::path::Path;

/// This machine's keyboard, and where that was learnt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheKeyboardThisMachineHas {
    /// The XKB layout name: `gb`, `de`, `fr`.
    layout: String,
    /// The XKB variant, or empty for the layout's own default.
    ///
    /// A [`String`] rather than an [`Option`] because that is what XKB takes:
    /// its own word for *no variant* is the empty one, and a second spelling of
    /// the same absence would be a second thing to translate.
    variant: String,
    /// Which of the two places answered.
    from: WhoSaidSo,
}

/// Which place named the keyboard a machine is about to use.
///
/// Kept and reported rather than thrown away, because the two cases are
/// different things to go and look at: a machine using its own file is set up,
/// and a machine using the unit file's word is one nobody has told yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhoSaidSo {
    /// This machine's own `keyboards.toml`, which somebody chose.
    ThisMachinesFile,
    /// The unit file, because this machine has no `keyboards.toml` yet.
    TheUnitFile,
    /// The unit file, because this machine's own file would not read.
    TheUnitFileBecauseTheMachinesFileDidNotRead(String),
}

/// A machine that named no keyboard anywhere.
#[derive(Debug, thiserror::Error)]
#[error(
    "this machine has no {THE_FILE_IS} and its unit file names no ALO_KEYBOARD, so there is no \
     layout to put a sign-in screen up with"
)]
pub struct NoKeyboardNamed;

/// The file's name, for the sentence above.
const THE_FILE_IS: &str = alo_keyboards::THE_MACHINES_KEYBOARDS;

impl TheKeyboardThisMachineHas {
    /// The XKB layout name.
    #[must_use]
    pub fn layout(&self) -> &str {
        &self.layout
    }

    /// The XKB variant, empty when the layout's default is wanted.
    #[must_use]
    pub fn variant(&self) -> &str {
        &self.variant
    }

    /// Which place named it.
    #[must_use]
    pub fn from(&self) -> &WhoSaidSo {
        &self.from
    }
}

/// Decide this machine's keyboard: its own file first, the unit file after.
///
/// `kept` is the path to this machine's `keyboards.toml` — normally
/// [`alo_keyboards::THE_MACHINES_KEYBOARDS`], and a path a test writes in a
/// test, for the same reason every other path in this crate is a parameter.
/// `named` is what the unit file said, or [`None`] when it said nothing.
///
/// # Errors
/// [`NoKeyboardNamed`] when neither place has an answer, which is a machine
/// that cannot put a sign-in screen up and must say so rather than guess.
pub fn the_keyboard_this_machine_has(
    kept: &Path,
    named: Option<&str>,
) -> Result<TheKeyboardThisMachineHas, NoKeyboardNamed> {
    // **Existence is asked before reading, and that is not belt and braces.**
    // `keeping::read` answers a missing file with the keyboards the release
    // ships — `us` — so a machine nobody has set up would read as a machine
    // that chose an American layout, and the unit file's word would never be
    // reached. The two have to be told apart here or not at all.
    if kept.exists() {
        match alo_keyboards::keeping::read(kept) {
            Ok(keyboards) => {
                let layout = keyboards.in_use();
                return Ok(TheKeyboardThisMachineHas {
                    layout: layout.name().to_owned(),
                    variant: layout.variant().unwrap_or_default().to_owned(),
                    from: WhoSaidSo::ThisMachinesFile,
                });
            }
            Err(why) => {
                // **`FileNotRead` has no `Display` on purpose**, and the
                // reason is in `alo_keyboards::unkept`: the only road to words
                // is `said`, in the language the person reads. There is no
                // person here and no `Strings` either — this is a line in a
                // root-owned log, read by whoever is mending the file — so it
                // is composed from the two facts that say which file and what
                // was wrong with it, and nothing of its contents is quoted.
                return fall_back_to(
                    named,
                    WhoSaidSo::TheUnitFileBecauseTheMachinesFileDidNotRead(format!(
                        "{} — {:?}",
                        why.at().display(),
                        why.why()
                    )),
                );
            }
        }
    }
    fall_back_to(named, WhoSaidSo::TheUnitFile)
}

/// The unit file's layout, under the reason it is being used.
fn fall_back_to(
    named: Option<&str>,
    from: WhoSaidSo,
) -> Result<TheKeyboardThisMachineHas, NoKeyboardNamed> {
    // **An empty `ALO_KEYBOARD=` is no answer.** `std::env::var` hands back an
    // empty string for a variable set to nothing, and XKB given an empty layout
    // takes its own default rather than refusing — so a unit file with the
    // right line and the wrong side of the equals sign would look like it
    // worked and would not be the layout anybody chose.
    let named = named.map(str::trim).filter(|it| !it.is_empty());
    named.map_or(Err(NoKeyboardNamed), |layout| {
        Ok(TheKeyboardThisMachineHas {
            layout: layout.to_owned(),
            // The unit file names a layout and has never named a variant. A
            // machine that needs `fr(oss)` says so in its own file.
            variant: String::new(),
            from,
        })
    })
}

#[cfg(test)]
#[path = "the_machines_keyboard_tests.rs"]
mod tests;
