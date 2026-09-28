//! The one place this program stops being able to put anything back.
//!
//! Installer plan task 7's constraint, word for word: *the point of no return is
//! one place in the code, named, and everything before it is reversible* — with
//! the difference from the alongside road that after it there is no bail-out,
//! and the program says so in the sentence that precedes it.
//!
//! # Why it takes what happens next, rather than being called before it
//!
//! [`crossed`] is handed the thing that cannot be undone. That is the whole
//! design: there is no way to reach the other side without going through the
//! sentence, because the sentence and the crossing are one call. A function that
//! only *said* the sentence could be forgotten, called twice, or called after the
//! disk was already gone, and nothing would fail.
//!
//! A test below holds the other half of *one place*: no other file in this crate
//! names this sentence. If a second place ever needs it, that test fails, and the
//! question — *is this really a second point of no return?* — gets asked by a
//! person instead of answered by a merge.
//!
//! # What this is not
//!
//! It is not a confirmation. Both of those already happened, in
//! [`crate::consent`] and [`crate::erasing_consent`], and by the time a person
//! reaches here they have named the disk twice and typed the word once. There is
//! no third question, no *are you sure?* checkbox and no timer that proceeds on
//! silence — the plan forbids all three, and a machine that asks again after a
//! person has decided is a machine that trained them to stop reading.
//!
//! It is a statement, made in the moment it becomes true and not before: from
//! here, nothing can be put back.

use alo_strings::{Filling, Strings, Word};

use crate::machine::TheMachine;
use crate::words;

/// Say that there is no way back, then do the thing there is no way back from.
///
/// `disk` is the disk's own name as the person was shown it — the same name they
/// typed, not a device path, because the sentence has to be checkable against
/// what they agreed to.
///
/// Everything before this call is reversible: every refusal says nothing was
/// changed, and `crate::staging` puts back each step it took. Nothing after it
/// is.
pub fn crossed<M: TheMachine, T>(
    machine: &mut M,
    strings: &Strings,
    disk: &str,
    beyond: impl FnOnce(&mut M) -> T,
) -> T {
    say(
        machine,
        strings,
        words::NO_WAY_BACK_AFTER_THIS,
        &Filling::of("disk", disk),
    );
    beyond(machine)
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use alo_strings::Said;

    use super::*;
    use crate::machine::Ran;
    use crate::program::Program;

    /// A machine that remembers the order it was spoken to in.
    ///
    /// Everything this crossing does not do is [`unreachable`] rather than a
    /// quiet default: the point of this file is that exactly one thing happens
    /// here, and a double that silently tolerated a program being run would hide
    /// the failure it exists to catch.
    #[derive(Default)]
    struct Watching {
        /// Every sentence said, in order.
        said: Vec<String>,
        /// How many times the far side of the crossing ran.
        beyond: usize,
    }

    impl TheMachine for Watching {
        fn say(&mut self, said: &Said) {
            self.said.push(said.text().to_owned());
        }

        fn ask(&mut self, _said: &Said) -> String {
            unreachable!("nothing is asked here: both consents were already given")
        }

        fn run(&mut self, _program: &Program) -> std::io::Result<Ran> {
            unreachable!("the point of no return runs no program of its own")
        }

        fn downloaded_into(&mut self) -> std::io::Result<PathBuf> {
            unreachable!("nothing here looks for the download")
        }

        fn this_program(&mut self) -> std::io::Result<PathBuf> {
            unreachable!("nothing here looks for this program")
        }

        fn read(&mut self, _file: &Path) -> std::io::Result<Vec<u8>> {
            unreachable!("nothing is read here")
        }

        fn write(&mut self, _file: &Path, _bytes: &[u8]) -> std::io::Result<()> {
            unreachable!("nothing is written here")
        }

        fn pause(&mut self, _for_as_long_as: Duration) {
            unreachable!("no timer proceeds on silence here; the plan forbids one")
        }
    }

    /// Everything this crate can say.
    fn strings() -> Strings {
        Strings::of(words::installer_words().unwrap())
    }

    /// **The sentence is said before the crossing, and the crossing happens
    /// exactly once.**
    #[test]
    fn the_sentence_comes_first_and_the_crossing_happens_once() {
        let strings = strings();
        let mut machine = Watching::default();

        let answer = crossed(
            &mut machine,
            &strings,
            "Samsung SSD 870 EVO",
            |machine: &mut Watching| {
                // Already said, by the time anything beyond this runs.
                assert_eq!(machine.said.len(), 1);
                machine.beyond += 1;
                "gone"
            },
        );

        assert_eq!(answer, "gone");
        assert_eq!(machine.beyond, 1);
        assert_eq!(machine.said.len(), 1);
    }

    /// **The sentence names the disk and comes out whole**, with nothing left
    /// unfilled.
    #[test]
    fn the_sentence_names_the_disk_and_comes_out_whole() {
        let strings = strings();
        let mut machine = Watching::default();
        crossed(&mut machine, &strings, "Samsung SSD 870 EVO", |_| ());

        let said = machine.said.first().unwrap();
        assert!(said.contains("Samsung SSD 870 EVO"), "{said}");
        assert!(!said.contains('{'), "{said}");
    }

    /// **One place, and this is it** — no other file in this crate names it.
    ///
    /// The other half of the plan's constraint. `words.rs` declares the sentence
    /// and lists it, which is not a second crossing; anywhere else is.
    #[test]
    fn nowhere_else_in_this_crate_says_there_is_no_way_back() {
        let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut elsewhere = Vec::new();
        for entry in fs::read_dir(&here).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.extension().is_none_or(|end| end != "rs")
                || name == "words.rs"
                || name == "the_point_of_no_return.rs"
            {
                continue;
            }
            if fs::read_to_string(&path)
                .unwrap()
                .contains("NO_WAY_BACK_AFTER_THIS")
            {
                elsewhere.push(name);
            }
        }
        assert!(
            elsewhere.is_empty(),
            "the point of no return is meant to be one place; also named in {elsewhere:?}"
        );
    }
}
