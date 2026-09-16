//! The README's *Try it*, and whether it says anything `docs/hardware.md` does
//! not.
//!
//! The installer plan's task 5: *the README gains a `Try it` section — the
//! requirements from `docs/hardware.md`, the fact that Windows stays, and the
//! one sentence about Secure Boot — and nothing else, because the README's rule
//! since 2026-09-13 is no noise.*
//!
//! Three things can go wrong in it, and every one is a green build:
//!
//! - **a requirement drifts.** The README says 16 GB while the hardware
//!   document says 32, and a person buys the wrong machine on our word;
//! - **the section grows.** A README that answers every question is a README
//!   nobody reads, and the download instruction is the one thing in it somebody
//!   actually needs;
//! - **the Secure Boot sentence turns into advice.**
//!   [ADR 0033](../../../docs/decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md)
//!   §4 is absolute: a person is never asked to switch Secure Boot off. It is
//!   an easy sentence to write helpfully and it is the one sentence in this
//!   product that may not be written at all.
//!
//! # What is read, and what is left as prose
//!
//! The section's list items — `- **Memory** — 32 GB` — are read as
//! requirements, each with the name of a row in `docs/hardware.md`'s *What to
//! buy first* table. Everything else in the section is counted as paragraphs
//! and read only for the two sentences that have to be there. Nothing here
//! judges the writing; `crate::released` is where the comparison happens.

/// Where the README is, in this repository.
pub const THE_README: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../README.md");

/// Where the hardware document is, beside it.
pub const THE_HARDWARE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/hardware.md");

/// The section this module reads.
pub const THE_SECTION: &str = "## Try it";

/// How a section begins.
const A_HEADING: &str = "## ";

/// How a list item begins.
const AN_ITEM: &str = "- ";

/// What a requirement's name is wrapped in.
const BOLD: &str = "**";

/// What separates a requirement's name from what it has to be.
const THEN: char = '—';

/// How many paragraphs of prose the section may hold: what it is, where the
/// requirements come from, that Windows stays, and Secure Boot.
const AT_MOST: usize = 4;

/// One thing a computer has to be, as the README states it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Requirement {
    /// The row in `docs/hardware.md` it names.
    pub named: String,
    /// What the README says it has to be.
    pub said: String,
}

/// The README's *Try it*, read off its text.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TheTryIt {
    /// Whether the section is there at all.
    there: bool,
    /// Every requirement it lists, in order.
    requirements: Vec<Requirement>,
    /// Every paragraph of prose in it, in order.
    paragraphs: Vec<String>,
}

impl TheTryIt {
    /// What the README's *Try it* says.
    #[must_use]
    pub fn read(readme: &str) -> Self {
        let Some((_, from)) = readme.split_once(&format!("{THE_SECTION}\n")) else {
            return Self::default();
        };
        let section = from.split_once(A_HEADING).map_or(from, |(it, _)| it);

        let mut read = Self {
            there: true,
            ..Self::default()
        };
        let mut paragraph = String::new();
        for line in section.lines() {
            let line = line.trim();
            if let Some(item) = line.strip_prefix(AN_ITEM) {
                if let Some(requirement) = a_requirement(item) {
                    read.requirements.push(requirement);
                }
                continue;
            }
            if line.is_empty() {
                if !paragraph.is_empty() {
                    read.paragraphs.push(std::mem::take(&mut paragraph));
                }
                continue;
            }
            if !paragraph.is_empty() {
                paragraph.push(' ');
            }
            paragraph.push_str(line);
        }
        if !paragraph.is_empty() {
            read.paragraphs.push(paragraph);
        }
        read
    }

    /// Whether the README has the section at all.
    #[must_use]
    pub const fn is_there(&self) -> bool {
        self.there
    }

    /// Every requirement it lists.
    #[must_use]
    pub fn requirements(&self) -> &[Requirement] {
        &self.requirements
    }

    /// Whether it says no more than the four paragraphs it is allowed.
    #[must_use]
    pub fn says_nothing_else(&self) -> bool {
        self.paragraphs.len() <= AT_MOST
    }

    /// Every paragraph in it naming this, in order.
    #[must_use]
    pub fn paragraphs_naming(&self, what: &str) -> Vec<&str> {
        self.paragraphs
            .iter()
            .filter(|it| it.contains(what))
            .map(String::as_str)
            .collect()
    }
}

/// A list item read as `**Name** — what it has to be`, where it is one.
fn a_requirement(item: &str) -> Option<Requirement> {
    let (named, said) = item.strip_prefix(BOLD)?.split_once(BOLD)?;
    let said = said.trim_start().strip_prefix(THEN)?;
    Some(Requirement {
        named: named.trim().to_owned(),
        said: said.trim().to_owned(),
    })
}

/// What `docs/hardware.md`'s *What to buy first* table says to buy, row by row.
///
/// Read off the table rather than copied into this crate, which is the whole
/// point: a requirement that moves there has to move in the README, and the two
/// are compared rather than both being spelled here.
#[must_use]
pub fn what_to_buy(hardware: &str) -> Vec<Requirement> {
    hardware
        .lines()
        .filter_map(|line| {
            let mut cells = line.trim().strip_prefix('|')?.split('|');
            let named = cells.next()?.trim().trim_matches('*').trim();
            let said = cells.next()?.trim();
            // A row of dashes is the table's rule rather than a thing to buy.
            let a_row = !named.trim_matches(['-', ':']).is_empty();
            (a_row && cells.next().is_some()).then(|| Requirement {
                named: named.to_owned(),
                said: said.replace(BOLD, ""),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A section, as the README writes one.
    const A_SECTION: &str = "# alo OS\n\n## Try it\n\nDownload it and run it.\n\n\
                             - **Memory** — 32 GB\n- **Firmware** — UEFI, TPM 2.0\n\n\
                             Windows stays.\n\n## What it is not\n\nNot a phone.\n";

    /// **The section is read, and stops at the next heading.**
    #[test]
    fn the_section_is_read_and_ends_where_it_ends() {
        let read = TheTryIt::read(A_SECTION);

        assert!(read.is_there());
        assert_eq!(
            read.requirements(),
            [
                Requirement {
                    named: "Memory".to_owned(),
                    said: "32 GB".to_owned(),
                },
                Requirement {
                    named: "Firmware".to_owned(),
                    said: "UEFI, TPM 2.0".to_owned(),
                },
            ]
        );
        assert_eq!(read.paragraphs_naming("Windows stays"), ["Windows stays."]);
        assert!(read.paragraphs_naming("Not a phone").is_empty());
    }

    /// **A README without the section reads as one without it**, rather than as
    /// one whose section says nothing.
    #[test]
    fn a_readme_without_the_section_says_so() {
        let read = TheTryIt::read("# alo OS\n\n## What it is not\n");

        assert!(!read.is_there());
        assert!(read.requirements().is_empty());
    }

    /// **A section that grew is caught**, at the paragraph after the fourth.
    #[test]
    fn a_section_that_grew_is_caught() {
        let prose = "\nand another thing.\n".repeat(AT_MOST + 1);
        let read = TheTryIt::read(&format!("## Try it\n{prose}"));

        assert!(!read.says_nothing_else());
        assert!(TheTryIt::read(A_SECTION).says_nothing_else());
    }

    /// **A list item that is not a requirement is not read as one**, so that an
    /// ordinary bullet in the section cannot become a promise about hardware.
    #[test]
    fn an_ordinary_bullet_is_not_a_requirement() {
        let read = TheTryIt::read("## Try it\n\n- just a bullet\n- **Bold** but no dash\n");

        assert!(read.requirements().is_empty());
    }

    /// **The hardware table is read row by row**, with its bold stripped and
    /// its heading and separator left out.
    #[test]
    fn the_table_is_read() {
        let table = "| | Buy | Because |\n|---|---|---|\n\
                     | **Memory** | **32 GB** | because |\n\
                     | **Firmware** | UEFI, Secure Boot, TPM 2.0 | because |\n";

        let buy = what_to_buy(table);

        assert!(
            buy.iter()
                .any(|it| it.named == "Memory" && it.said == "32 GB")
        );
        assert!(
            buy.iter()
                .any(|it| it.named == "Firmware" && it.said.contains("TPM 2.0"))
        );
        assert!(!buy.iter().any(|it| it.named == "---"));
    }
}
