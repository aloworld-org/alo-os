//! What `docs/booting.md` tells a person to do, read as something checkable.
//!
//! The disk this repository produces is described in two places on purpose: the
//! `alo.disk.*` labels in `image/Containerfile`, which are what the tool is
//! actually given, and this document, which is what a person reads before
//! selecting a firmware in a dialog box. Neither can be dropped — a declaration
//! nobody can follow is not a disk, and a document nothing holds to the recipe
//! is how the firmware sentence stays where it was while the image moved.
//!
//! # A document with four facts in it
//!
//! The interesting half of the document is prose and stays prose. What is read
//! here is the indented block of `key: value` lines near the top — the tool, the
//! firmware, the virtual machine generation and the disk's filename — plus the
//! headings the document promises and the commands it gives. `crate::checking`
//! is what compares them with the recipe.
//!
//! # Read leniently, judged strictly
//!
//! [`TheDocument::read`] never refuses, which is `crate::runtime`'s and
//! `crate::disk`'s shape: a document missing every fact reads as one that states
//! none. A fact stated twice reads as stated by nobody, because a second line
//! silently winning is the failure this module would otherwise hide.

use std::collections::BTreeMap;

/// The fact naming the tool that writes the disk.
pub const THE_TOOL: &str = "tool";

/// The fact naming the firmware the disk is installed for.
pub const THE_FIRMWARE: &str = "firmware";

/// The fact naming the virtual machine generation a person is told to select.
pub const THE_GENERATION: &str = "generation";

/// The fact naming the file the disk is written to.
pub const THE_DISK: &str = "disk";

/// The four facts this document states, and the only keys read out of it.
///
/// A fixed set rather than every `key: value` line, because ordinary prose is
/// full of colons and a reader that took them all would read a sentence as a
/// promise.
const THE_FACTS: [&str; 4] = [THE_TOOL, THE_FIRMWARE, THE_GENERATION, THE_DISK];

/// What an indented block looks like in Markdown, which is where both the facts
/// and the commands are.
const INDENTED: &str = "    ";

/// How a section begins.
const A_HEADING: &str = "## ";

/// What `docs/booting.md` says, read off its text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheDocument {
    /// Every value stated for each fact, in the order they were stated.
    facts: BTreeMap<String, Vec<String>>,
    /// Every section, by its heading, as text.
    sections: BTreeMap<String, String>,
    /// Every indented line that is not a fact, which is where the commands are.
    commands: Vec<String>,
}

impl TheDocument {
    /// What this document says, read off its text.
    #[must_use]
    pub fn read(document: &str) -> Self {
        let mut read = Self::default();
        let mut heading = String::new();

        for line in document.lines() {
            if let Some(said) = line.strip_prefix(A_HEADING) {
                heading = said.trim().to_owned();
                read.sections.entry(heading.clone()).or_default();
                continue;
            }
            read.sections
                .entry(heading.clone())
                .or_default()
                .push_str(line);
            read.sections.entry(heading.clone()).or_default().push('\n');

            if !line.starts_with(INDENTED) {
                continue;
            }
            match stated(line.trim()) {
                Some((fact, said)) => read.facts.entry(fact).or_default().push(said),
                None => read.commands.push(line.trim().to_owned()),
            }
        }

        read
    }

    /// What this document states for one fact.
    ///
    /// [`None`] where it states none — and [`None`] where it states two, which
    /// is a document a person would read the first of and a checker the last of.
    #[must_use]
    pub fn says(&self, fact: &str) -> Option<&str> {
        match self.facts.get(fact)?.as_slice() {
            [only] => Some(only.as_str()),
            _ => None,
        }
    }

    /// Whether this document has a section under this exact heading.
    #[must_use]
    pub fn has_a_section(&self, heading: &str) -> bool {
        self.sections.contains_key(heading)
    }

    /// Whether the section under this heading names this, anywhere in it.
    ///
    /// The whole document is not asked: a sentence about the GPU under
    /// *attaching it to Hyper-V* is not the paragraph saying a virtual GPU
    /// proves nothing about a certified machine.
    #[must_use]
    pub fn names_under(&self, heading: &str, what: &str) -> bool {
        self.sections
            .get(heading)
            .is_some_and(|section| section.to_lowercase().contains(&what.to_lowercase()))
    }

    /// Whether this document names this anywhere at all, prose included.
    #[must_use]
    pub fn names(&self, what: &str) -> bool {
        self.sections
            .values()
            .any(|section| section.to_lowercase().contains(&what.to_lowercase()))
    }

    /// Whether this document actually gives a command that runs this tool.
    ///
    /// The fact stating the tool's name is deliberately not enough: a document
    /// that names a tool and tells nobody how to run it is the one thing this
    /// whole task exists to stop shipping.
    #[must_use]
    pub fn gives_the_command(&self, tool: &str) -> bool {
        self.commands.iter().any(|command| command.contains(tool))
    }
}

/// The fact and the value on this line, where it states one.
fn stated(line: &str) -> Option<(String, String)> {
    let (fact, said) = line.split_once(": ")?;
    THE_FACTS
        .contains(&fact)
        .then(|| (fact.to_owned(), said.trim().to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A document holding exactly the lines a test names.
    fn saying(lines: &[&str]) -> TheDocument {
        TheDocument::read(&lines.join("\n"))
    }

    /// **The four facts read off an indented block**, which is what the rest of
    /// the checks are made of.
    #[test]
    fn a_document_stating_its_facts_reads_as_one() {
        let read = saying(&[
            "# Turning the image into a disk",
            "",
            "    tool: bootc install to-disk",
            "    firmware: uefi",
            "    generation: 2",
            "    disk: alo-os.raw",
        ]);

        assert_eq!(read.says(THE_TOOL), Some("bootc install to-disk"));
        assert_eq!(read.says(THE_FIRMWARE), Some("uefi"));
        assert_eq!(read.says(THE_GENERATION), Some("2"));
        assert_eq!(read.says(THE_DISK), Some("alo-os.raw"));
    }

    /// **A fact stated twice is stated by nobody.** A person reads the first and
    /// a checker would read the last, which is the disagreement that never shows
    /// up in a diff.
    #[test]
    fn a_fact_stated_twice_is_not_stated() {
        let read = saying(&["    firmware: uefi", "    firmware: bios"]);

        assert_eq!(read.says(THE_FIRMWARE), None);
    }

    /// **Prose is not a fact.** Ordinary sentences are full of colons, and a
    /// reader that took them all would read one as a promise.
    #[test]
    fn a_sentence_with_a_colon_in_it_states_nothing() {
        let read = saying(&[
            "The disk: what it is and what it is not.",
            "    memory: 8192 MB",
        ]);

        assert_eq!(read.says(THE_DISK), None);
        assert!(!read.gives_the_command("bootc install to-disk"));
    }

    /// **A tool named in a fact is not a tool anybody was told how to run.**
    #[test]
    fn a_document_that_names_a_tool_without_running_it_gives_no_command() {
        let named = saying(&["    tool: bootc install to-disk"]);
        assert!(!named.gives_the_command("bootc install to-disk"));

        let given = saying(&[
            "    tool: bootc install to-disk",
            "    podman run --rm --privileged localhost/alo-os:dev \\",
            "      bootc install to-disk --via-loopback /output/alo-os.raw",
        ]);
        assert!(given.gives_the_command("bootc install to-disk"));
    }

    /// **A section is what is under its own heading**, so a word in the wrong
    /// paragraph is not the paragraph that promised it.
    #[test]
    fn what_a_section_names_is_what_is_under_it() {
        let read = saying(&[
            "## Attaching it to Hyper-V",
            "The GPU is a synthetic one.",
            "## What a virtual machine cannot show",
            "The firmware here is tame.",
        ]);

        assert!(read.has_a_section("What a virtual machine cannot show"));
        assert!(!read.has_a_section("What you need installed"));
        assert!(read.names_under("What a virtual machine cannot show", "firmware"));
        assert!(!read.names_under("What a virtual machine cannot show", "GPU"));
        assert!(read.names("GPU"));
    }
}
