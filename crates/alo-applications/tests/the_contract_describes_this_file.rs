//! `docs/contracts/person-settings.md` describes `what-opens-what.toml` as this
//! crate keeps it, and a test is what keeps that true.
//!
//! A person opens the file in an editor with that section beside it. A
//! contract that drifted from the crate would be a person typing a kind the
//! section lists and having the whole file refused. **Where the two disagree
//! the crate is right and the document is what changes**, so everything here
//! asks the crate and holds the words to its answer: the keys the section lists
//! are the keys this crate writes, the kinds it lists are every kind and no
//! other, the file the section says alo OS writes is byte for byte the file it
//! writes, every example the section says reads does read, and every example it
//! says is refused is refused, in the sentence the section names.
//!
//! Every example goes through a real file on a real disk, because that is how a
//! person's edit arrives.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_applications::keeping::{self, FORMAT, THE_FILE};
use alo_applications::spelled::spelled;
use alo_applications::{Application, Chosen, FileNotRead, application_words};
use alo_opening::Kind;
use alo_strings::Strings;

/// One fenced block of the section: what follows the three backticks, the
/// text inside, and the prose after it up to the next block.
struct Fence {
    /// `toml`, or `toml refused`.
    info: String,
    /// The block's text, with a newline at the end of every line.
    text: String,
    /// What the section says after the block.
    after: String,
}

/// The contract, as the repository holds it.
fn the_contract() -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/person-settings.md");
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// How deep a Markdown heading is — two for `##` — or `None` for a line that
/// is not one.
fn heading_depth(line: &str) -> Option<usize> {
    let hashes = line.chars().take_while(|c| *c == '#').count();
    (hashes > 0 && line.chars().nth(hashes) == Some(' ')).then_some(hashes)
}

/// From the heading that begins `starting` to the next heading as shallow as
/// it, outside any fenced block.
fn under_heading(text: &str, starting: &str) -> String {
    let mut lines = text.lines().skip_while(|line| !line.starts_with(starting));
    let heading = lines
        .next()
        .unwrap_or_else(|| panic!("no heading beginning {starting:?}"));
    let depth = heading_depth(heading).expect("a heading");
    let mut fenced = false;
    let mut body = vec![heading];
    for line in lines {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        if !fenced && heading_depth(line).is_some_and(|it| it <= depth) {
            break;
        }
        body.push(line);
    }
    format!("{}\n", body.join("\n"))
}

/// This file's section of the contract.
fn the_section() -> String {
    under_heading(&the_contract(), &format!("## `{THE_FILE}`"))
}

/// One part of the section, by the start of its heading.
fn the_part(title: &str) -> String {
    under_heading(&the_section(), &format!("### {title}"))
}

/// The first column of every row of the table in this part.
fn keys_listed(part: &str) -> Vec<String> {
    part.lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .filter_map(|row| row.split('`').next())
        .map(str::to_owned)
        .collect()
}

/// Every fenced block in the section.
fn fences(section: &str) -> Vec<Fence> {
    let mut fences: Vec<Fence> = Vec::new();
    let mut inside: Option<(String, String)> = None;
    for line in section.lines() {
        let fence = line.strip_prefix("```");
        match (inside.take(), fence) {
            (None, Some(info)) => inside = Some((info.trim().to_owned(), String::new())),
            (Some((info, text)), Some(_)) => fences.push(Fence {
                info,
                text,
                after: String::new(),
            }),
            (Some((info, mut text)), None) => {
                text.push_str(line);
                text.push('\n');
                inside = Some((info, text));
            }
            (None, None) => {
                if let Some(last) = fences.last_mut() {
                    last.after.push_str(line);
                    last.after.push('\n');
                }
            }
        }
    }
    assert!(inside.is_none(), "a block in the section is never closed");
    fences
}

/// The first code span in this prose that begins with `prefix`.
fn code_span_beginning(prose: &str, prefix: &str) -> Option<String> {
    prose
        .split('`')
        .skip(1)
        .step_by(2)
        .find(|span| span.starts_with(prefix))
        .map(str::to_owned)
}

/// A folder under the temporary directory that is this test's alone, emptied.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-applications-contract-{}-{what}",
        std::process::id()
    ));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// This text, put where the file goes and read as a person's file is.
fn read_from_a_file(what: &str, text: &str) -> (PathBuf, Result<Chosen, FileNotRead>) {
    let at = a_folder_of_our_own(what).join(THE_FILE);
    std::fs::write(&at, text).unwrap();
    let read = keeping::read(&at);
    (at, read)
}

/// The choices the section's first example is of.
fn what_the_example_chose() -> Chosen {
    let mut chosen = Chosen::untouched();
    chosen.choose(
        Kind::Pdf,
        &Application::identified("org.kde.okular").unwrap(),
    );
    chosen.choose(
        Kind::PngImage,
        &Application::identified("org.gnome.Papers").unwrap(),
    );
    chosen
}

/// What this crate writes for these choices, off the disk.
fn written_for(what: &str, chosen: &Chosen) -> String {
    let at = a_folder_of_our_own(what).join(THE_FILE);
    keeping::keep(&at, chosen).unwrap();
    std::fs::read_to_string(&at).unwrap()
}

/// **The keys the section lists are exactly the keys this crate writes**, the
/// `format` it names is this crate's, and the kinds it lists are every kind
/// spelled as this crate spells it, in order.
#[test]
fn the_contract_lists_every_key_and_every_kind_this_crate_writes_and_no_other() {
    let written = written_for("keys", &what_the_example_chose());
    let table: toml::Table = toml::from_str(&written).unwrap();
    let mut writes: Vec<String> = table
        .keys()
        .filter(|key| *key != "format")
        .cloned()
        .collect();
    let mut listed = keys_listed(&the_part("Keys"));
    writes.sort();
    listed.sort();
    assert_eq!(
        listed, writes,
        "the section's keys are not the keys the crate writes"
    );
    assert!(
        the_part("`format`").contains(&format!("`format = {FORMAT}`")),
        "the section does not name format {FORMAT}"
    );

    let values = the_part("Values");
    let kinds_paragraph = values
        .split("\n\n")
        .find(|paragraph| paragraph.contains("`pdf`"))
        .expect("the section lists the kinds");
    let listed: Vec<String> = kinds_paragraph
        .split('`')
        .skip(1)
        .step_by(2)
        .filter(|span| !span.contains(':') && !span.contains('['))
        .map(str::to_owned)
        .collect();
    let spelled_here: Vec<String> = Kind::EVERY
        .into_iter()
        .map(|kind| spelled(kind).to_owned())
        .collect();
    assert_eq!(
        listed, spelled_here,
        "the section's kinds are not every kind"
    );
}

/// **The file the section says alo OS writes is, byte for byte, the file this
/// crate writes**, and every example the section offers as reading reads.
#[test]
fn what_the_contract_says_alo_os_writes_is_what_it_writes_and_every_example_reads() {
    let reading: Vec<Fence> = fences(&the_section())
        .into_iter()
        .filter(|fence| fence.info == "toml")
        .collect();
    let first = reading
        .first()
        .expect("the section shows what alo OS writes");
    assert_eq!(
        first.text,
        written_for("written", &what_the_example_chose()),
        "the section's file is not the file this crate writes"
    );
    for (number, fence) in reading.iter().enumerate() {
        let (at, read) = read_from_a_file(&format!("reads-{number}"), &fence.text);
        let read = read.unwrap_or_else(|refused| {
            panic!(
                "{} did not read: {:?}\n{}",
                at.display(),
                refused.why(),
                fence.text
            )
        });
        if number == 0 {
            assert_eq!(read, what_the_example_chose());
        }
    }
}

/// **Every example the section says is refused is refused, in the sentence the
/// section names**, naming the key where the section says it does — and nothing
/// in it is chosen.
#[test]
fn every_example_the_contract_refuses_is_refused_in_the_words_it_names() {
    let mut vocabulary = application_words().unwrap();
    alo_opening::declare_into(&mut vocabulary).unwrap();
    let strings = Strings::of(vocabulary);
    let refused: Vec<Fence> = fences(&the_section())
        .into_iter()
        .filter(|fence| fence.info == "toml refused")
        .collect();
    assert!(refused.len() >= 3, "the section shows what does not read");
    for (number, fence) in refused.iter().enumerate() {
        let (at, read) = read_from_a_file(&format!("refused-{number}"), &fence.text);
        let Err(why) = read else {
            panic!("the section says this is refused:\n{}", fence.text)
        };
        let named = code_span_beginning(&fence.after, "applications.kept.")
            .unwrap_or_else(|| panic!("no sentence is named after:\n{}", fence.text));
        assert_eq!(why.word().named(), named, "{}", fence.text);

        let said = why.said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains(&at.display().to_string()), "{said}");
        if let Some(key) = fence
            .after
            .split("naming `")
            .nth(1)
            .and_then(|rest| rest.split('`').next())
        {
            assert_eq!(why.key(), Some(key), "{}", fence.text);
        }

        let (chosen, beside) = keeping::at_sign_in(&at);
        assert_eq!(
            chosen,
            Chosen::untouched(),
            "nothing in the file is honoured"
        );
        assert_eq!(beside, Some(why));
    }
}

/// **No file is what the section says it is**: a person who has chosen
/// nothing, not an error, and nothing written.
#[test]
fn a_missing_file_is_a_person_who_chose_nothing() {
    assert!(the_part("What a missing file means").contains("The person has chosen nothing"));
    let at = a_folder_of_our_own("missing").join(THE_FILE);
    assert_eq!(keeping::read(&at).unwrap(), Chosen::untouched());
    assert_eq!(keeping::at_sign_in(&at), (Chosen::untouched(), None));
    assert!(!at.exists(), "reading wrote nothing");
}

/// **The section's *Writing it* says what the writer does with a file that did
/// not read, and the crate does it.**
#[test]
fn the_contract_says_a_file_that_did_not_read_is_not_written_over() {
    let writing = the_part("Writing it");
    assert!(
        writing.contains("A file that does not read is not written over"),
        "{writing}"
    );
    let named = code_span_beginning(&writing, "applications.kept.not-replaced")
        .expect("the section names the sentence a change over a broken file is refused with");
    assert!(
        writing.contains("alo_applications::keeping::put_back_as_shipped"),
        "the section does not name the one door that replaces such a file"
    );

    let refused = fences(&the_section())
        .into_iter()
        .find(|fence| fence.info == "toml refused")
        .expect("the section shows a file that does not read");
    let (at, read) = read_from_a_file("not-written-over", &refused.text);
    assert!(read.is_err());
    let before = std::fs::read(&at).unwrap();

    let why = keeping::keep(&at, &what_the_example_chose()).unwrap_err();
    assert_eq!(why.word().named(), named);
    assert_eq!(std::fs::read(&at).unwrap(), before);

    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        format!("format = {FORMAT}\n")
    );
    assert_eq!(keeping::read(&at).unwrap(), Chosen::untouched());
}

/// **The section names the crate that answers the question and the portal that
/// reads the file, and says only a person's choice is written.**
#[test]
fn the_contract_says_who_reads_this_file_and_who_writes_it() {
    let section = the_section();
    for named in [
        "alo_applications::WhatOpensWhat",
        "alo_portals::open_with",
        "Only a person's choice is written here",
    ] {
        assert!(
            section.contains(named),
            "the section does not say {named:?}"
        );
    }
    let table = under_heading(
        &the_contract(),
        "### The folder, and the files beside this one",
    );
    assert!(
        table.contains(&format!(
            "| `{THE_FILE}` | `alo_applications::keeping` | `{FORMAT}` |"
        )),
        "the list of files in the person's folder does not name this one"
    );
}
