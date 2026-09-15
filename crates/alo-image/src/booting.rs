//! What `docs/booting.md` tells a person to do, read as something checkable.
//!
//! The disk this repository produces is described in two places on purpose: the
//! `alo.disk.*` labels in `image/Containerfile`, which are what the tool is
//! actually given, and this document, which is what a person reads before
//! selecting a firmware in a dialog box. Neither can be dropped — a declaration
//! nobody can follow is not a disk, and a document nothing holds to the recipe
//! is how the firmware sentence stays where it was while the image moved.
//!
//! # A document with seven facts in it
//!
//! The interesting half of the document is prose and stays prose. What is read
//! here is the indented block of `key: value` lines near the top — the tool, the
//! firmware, the virtual machine generation and the disk's filename — and the
//! one under *Installing the published image* — the registry, the tag and the
//! digest — plus the headings the document promises and the commands it gives,
//! each joined across its continued lines. `crate::checking` compares the first
//! block with the recipe, and `crate::publishing` the second with
//! `image/pinned.toml`.
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

/// The fact naming the registry a published image is pulled from.
pub const THE_REGISTRY: &str = "registry";

/// The fact naming the published release's tag.
pub const THE_TAG: &str = "tag";

/// The fact naming the digest that release is pulled by.
pub const THE_DIGEST: &str = "digest";

/// The facts this document states, and the only keys read out of it: four
/// about the disk, three about the published image.
///
/// A fixed set rather than every `key: value` line, because ordinary prose is
/// full of colons and a reader that took them all would read a sentence as a
/// promise.
const THE_FACTS: [&str; 7] = [
    THE_TOOL,
    THE_FIRMWARE,
    THE_GENERATION,
    THE_DISK,
    THE_REGISTRY,
    THE_TAG,
    THE_DIGEST,
];

/// What ends a command line that carries on onto the next.
const CONTINUED: char = '\\';

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
    lines: Vec<String>,
    /// Those lines joined into the commands a person types, in the order the
    /// document gives them.
    commands: Vec<String>,
}

impl TheDocument {
    /// What this document says, read off its text.
    #[must_use]
    pub fn read(document: &str) -> Self {
        let mut read = Self::default();
        let mut heading = String::new();
        let mut typing: Option<String> = None;

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
                typing = None;
                continue;
            }
            match stated(line.trim()) {
                Some((fact, said)) => {
                    typing = None;
                    read.facts.entry(fact).or_default().push(said);
                }
                None => {
                    let said = line.trim();
                    read.lines.push(said.to_owned());
                    let part = said.trim_end_matches(CONTINUED).trim_end();
                    match typing.take() {
                        Some(mut command) => {
                            command.push(' ');
                            command.push_str(part);
                            typing = Some(command);
                        }
                        None => typing = Some(part.to_owned()),
                    }
                    if !said.ends_with(CONTINUED)
                        && let Some(command) = typing.take()
                    {
                        read.commands.push(command);
                    }
                }
            }
        }
        read.commands.extend(typing);

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
        self.lines.iter().any(|line| line.contains(tool))
    }

    /// Every command this document gives, with its continued lines joined, in
    /// the order a person would type them.
    ///
    /// One command is one entry however many lines it was written across: a
    /// `podman run` on one line and the image it runs on the next are still one
    /// thing somebody types, and a check that looked at them a line at a time
    /// could not ask what that one thing pulls.
    #[must_use]
    pub fn commands(&self) -> &[String] {
        &self.commands
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

    /// **A command written across lines is one command**, and the commands come
    /// back in the order a person types them.
    #[test]
    fn a_command_across_lines_is_one_command() {
        let read = saying(&[
            "    cosign verify --key image/signing/alo-os.pub \\",
            "      registry@sha256:abc",
            "",
            "    podman run --rm \\",
            "      registry@sha256:abc \\",
            "      bootc install to-disk /output/alo-os.raw",
            "    registry: somewhere",
        ]);

        assert_eq!(
            read.commands(),
            [
                "cosign verify --key image/signing/alo-os.pub registry@sha256:abc",
                "podman run --rm registry@sha256:abc bootc install to-disk /output/alo-os.raw",
            ]
        );
        assert_eq!(read.says(THE_REGISTRY), Some("somewhere"));
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
