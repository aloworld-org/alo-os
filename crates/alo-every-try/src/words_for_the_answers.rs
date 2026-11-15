//! **The words the scripted answers are said in**: every word this machine
//! can say, from the one place the daemon reads them
//! (`alo_saying::everything_this_machine_can_say`, as
//! `alo-agentd`'s `what_this_machine_says` loads it), in English.
//!
//! English because the set's requests are English, and the model is told the
//! machine's answers in the language the machine speaks to it.
//!
//! **Until 2026-11-15 this joined six crates' words by hand**, and the file
//! verbs', the catalogue's and the record's were not among them: a rename done
//! at full access was told to the model as *"Done without asking you first …:
//! «files.verb.rename-file.sentence»"*, a key where a sentence belongs. A list
//! typed in a second place had drifted from the first, which is rule 3 of
//! ADR 0156; it is read from the first now.

use alo_strings::Strings;

/// The machine's words would not collect.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("this machine's words could not be collected: {why}")]
pub struct NotTheWords {
    /// What `alo-saying` said went wrong.
    why: String,
}

/// Every word this machine can say, as the daemon has them before any
/// translation is laid over them.
///
/// # Errors
/// [`NotTheWords`], naming what would not collect.
pub fn the_words() -> Result<Strings, NotTheWords> {
    alo_saying::everything_this_machine_can_say()
        .map(Strings::of)
        .map_err(|why| NotTheWords {
            why: format!("{why:?}"),
        })
}

#[cfg(test)]
mod tests {
    use super::the_words;

    #[test]
    fn every_word_the_machine_says_collects() {
        assert!(the_words().is_ok(), "{:?}", the_words().err());
    }
}
