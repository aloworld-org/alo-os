//! `docs/contracts/person-settings.md` describes `displays.toml` as this crate
//! keeps it, and a test is what keeps that true.
//!
//! A person opens the file in an editor with that section beside it, and a
//! backup tool, a migration or an organisation's provisioning reads the
//! contract's table rather than this crate's source. A contract that drifted
//! from the crate would be a person typing a key the section lists and having
//! the whole file refused. **Where the two disagree the crate is right and the
//! document is what changes**, so everything here asks the crate and holds the
//! words to its answer: the keys the section lists are the keys this crate
//! writes, the file the section says alo OS writes is byte for byte the file it
//! writes, every example the section says reads does read, and every example it
//! says is refused is refused, in the sentence the section names.
//!
//! **And the count is held too.** The folder's own section said *four* for six
//! days while the folder held ten files, which is the failure this file exists
//! to stop repeating: the number in that sentence is read here against the rows
//! of the table under it, so a file added without a row, or a row added without
//! the number moving, fails in the change that does it.
//!
//! Every example goes through a real file on a real disk rather than text in
//! memory, because that is how a person's edit arrives.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use alo_displays::keeping::{self, FORMAT, THE_FILE};
use alo_displays::{
    Arrangement, Between, Changes, FileNotRead, Identity, NightLight, Nightly, Panel, Placed,
    Position, Scale, Socket, TimeOfDay, Warmth, display_words,
};
use alo_strings::Strings;

/// The whole numbers as the contract's prose writes them, so that a sentence a
/// person reads is also a sentence this test can check.
const IN_WORDS: [&str; 21] = [
    "zero",
    "one",
    "two",
    "three",
    "four",
    "five",
    "six",
    "seven",
    "eight",
    "nine",
    "ten",
    "eleven",
    "twelve",
    "thirteen",
    "fourteen",
    "fifteen",
    "sixteen",
    "seventeen",
    "eighteen",
    "nineteen",
    "twenty",
];

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

/// The part of the contract that lists every file in the person's folder.
fn the_folder() -> String {
    under_heading(&the_contract(), "### The folder, and the files beside")
}

/// Every row of the folder's table, each the whole line.
fn rows_of_the_folders_table(folder: &str) -> Vec<&str> {
    folder
        .lines()
        .filter(|line| line.starts_with("| `") && line.ends_with('|'))
        .collect()
}

/// How many files the sentence that counts them says there are.
fn how_many_the_sentence_says(folder: &str) -> usize {
    let counted = folder
        .split("there are **")
        .nth(1)
        .and_then(|rest| rest.split("**").next())
        .expect("the folder's section counts the files in it");
    IN_WORDS
        .iter()
        .position(|word| *word == counted)
        .unwrap_or_else(|| panic!("{counted:?} is not a number this test can read"))
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
    let folder = std::env::temp_dir().join(format!("alo-displays-contract-{what}"));
    if folder.exists() {
        std::fs::remove_dir_all(&folder).unwrap();
    }
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// This text, put where the file goes and read as a person's file is.
fn read_from_a_file(what: &str, text: &str) -> (PathBuf, Result<Changes, FileNotRead>) {
    let at = a_folder_of_our_own(what).join(THE_FILE);
    std::fs::write(&at, text).unwrap();
    let read = keeping::read(&at);
    (at, read)
}

/// The laptop's own panel, which says nothing about itself and is therefore
/// remembered by the socket it is in.
fn the_laptop() -> Identity {
    Identity::Socket(Socket::named("eDP-1").unwrap())
}

/// The screen at the office, which reports a serial number.
fn the_office_screen() -> Identity {
    Identity::Panel(Panel::of("Example", "U2723QE", Some("G4H2K7")).unwrap())
}

/// The screen at home, which does not.
fn the_home_screen() -> Identity {
    Identity::Panel(Panel::of("Example", "P2419H", None).unwrap())
}

/// Two sets of screens arranged and night light asked for, so that every key
/// and both ways of naming a screen are written — the value the section's
/// first example is of.
fn everything_changed() -> Changes {
    let mut changes = Changes::untouched();
    changes.remember(
        Arrangement::of(vec![
            (
                the_laptop(),
                Placed::at(Position::at(0, 0), Scale::per_cent(200).unwrap()).as_the_main_screen(),
            ),
            (
                the_office_screen(),
                Placed::at(Position::at(-2560, 0), Scale::per_cent(150).unwrap()),
            ),
        ])
        .unwrap(),
    );
    changes.remember(
        Arrangement::of(vec![
            (
                the_laptop(),
                Placed::at(Position::at(1920, 240), Scale::per_cent(200).unwrap()),
            ),
            (
                the_home_screen(),
                Placed::at(Position::at(0, 0), Scale::per_cent(100).unwrap()).as_the_main_screen(),
            ),
        ])
        .unwrap(),
    );
    changes.set_night_light(NightLight::of(
        Nightly::Between(
            Between::these_two_times(
                TimeOfDay::written("21:30").unwrap(),
                TimeOfDay::written("07:00").unwrap(),
            )
            .unwrap(),
        ),
        Warmth::kelvin(3400).unwrap(),
    ));
    changes
}

/// What this crate writes for these changes, off the disk.
fn written_for(what: &str, changes: &Changes) -> String {
    let at = a_folder_of_our_own(what).join(THE_FILE);
    keeping::keep(&at, changes).unwrap();
    std::fs::read_to_string(&at).unwrap()
}

/// **The folder's table names this file, and the sentence that counts the
/// files in the folder is true of that table** — the check the contract went
/// six days without, while it told every reader there were four.
#[test]
fn the_folders_table_names_this_file_and_its_count_is_the_rows_it_has() {
    let folder = the_folder();
    let rows = rows_of_the_folders_table(&folder);
    assert_eq!(
        how_many_the_sentence_says(&folder),
        rows.len(),
        "the sentence counting the files does not count the rows under it"
    );

    let beginning = format!("| `{THE_FILE}` |");
    let row = rows
        .iter()
        .find(|row| row.starts_with(&beginning))
        .unwrap_or_else(|| panic!("no row in the folder's table for {THE_FILE}"));
    assert!(
        row.contains("`alo_displays::keeping`"),
        "the row does not name the crate that keeps this file: {row}"
    );
    assert!(
        row.contains(&format!("| `{FORMAT}` |")),
        "the row does not name format {FORMAT}: {row}"
    );
    for key in keys_listed(&the_part("Keys")) {
        assert!(
            row.contains(&format!("`{key}`")),
            "the row does not name {key}: {row}"
        );
    }
}

/// **The keys the section lists are exactly the keys this crate writes**, and
/// the `format` it names is this crate's.
#[test]
fn the_contract_lists_every_key_this_crate_writes_and_no_other() {
    let written = written_for("keys", &everything_changed());
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
}

/// **The file the section says alo OS writes is, byte for byte, the file this
/// crate writes**, and every example the section offers as reading reads — the
/// first of them as exactly the arrangements it was written from.
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
        written_for("written", &everything_changed()),
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
            assert_eq!(read, everything_changed());
        }
    }
}

/// **Every example the section says is refused is refused, in the sentence the
/// section names**, naming the key or the line where the section says it does —
/// and the machine is then one that has arranged nothing, with not one
/// arrangement of the file honoured.
#[test]
fn every_example_the_contract_refuses_is_refused_in_the_words_it_names() {
    let strings = Strings::of(display_words().unwrap());
    let refused: Vec<Fence> = fences(&the_section())
        .into_iter()
        .filter(|fence| fence.info == "toml refused")
        .collect();
    assert!(
        refused.len() >= 5,
        "the section shows every way this file does not read"
    );
    for (number, fence) in refused.iter().enumerate() {
        let (at, read) = read_from_a_file(&format!("refused-{number}"), &fence.text);
        let Err(why) = read else {
            panic!("the section says this is refused:\n{}", fence.text)
        };
        let named = code_span_beginning(&fence.after, "displays.kept.")
            .unwrap_or_else(|| panic!("no sentence is named after:\n{}", fence.text));
        assert_eq!(why.word().named(), named, "{}", fence.text);

        let said = why.said(&strings);
        assert!(said.text().contains(&at.display().to_string()), "{said}");
        if let Some(key) = fence
            .after
            .split("naming `")
            .nth(1)
            .and_then(|rest| rest.split('`').next())
        {
            assert_eq!(why.key(), Some(key), "{}", fence.text);
        }
        if let Some(line) = fence
            .after
            .split("naming line ")
            .nth(1)
            .and_then(|rest| rest.split(|c: char| !c.is_ascii_digit()).next())
        {
            assert!(said.text().contains(&format!("line {line}")), "{said}");
        }

        let (arranged, beside) = keeping::at_sign_in(&at);
        assert_eq!(
            arranged,
            Changes::untouched(),
            "nothing in the file is honoured"
        );
        assert_eq!(beside, Some(why));
    }
}

/// **The section says an arrangement has exactly its own keys, and shows one
/// refused example per table.**
///
/// It said the opposite until this was written — a paragraph headed *One check
/// `appearance.toml` has that this file does not yet*, describing a key inside
/// an `[[arrangements]]` table or one of its `screens` rows as read past. The
/// crate now refuses it, and a contract still carrying that paragraph would
/// tell a person their stray key is honoured while their whole file is being
/// refused, which is the worst of the three things that sentence could say.
///
/// So both examples are put on a real disk: each is refused, in the sentence
/// the section names and with nothing in the file honoured; neither names a key
/// at the top of the file, because the key is inside a value; and each **reads**
/// with the stray line taken out, so what is refused is that line and not the
/// shape around it.
#[test]
fn the_contract_says_a_key_inside_an_arrangement_refuses_the_whole_file() {
    let values = the_part("Values");
    assert!(
        !values.contains("does not yet"),
        "the section still describes a check the crate now has: {values}"
    );
    assert!(
        values.contains("has exactly its own keys"),
        "the section does not say an arrangement has exactly its own keys: {values}"
    );

    let refused: Vec<Fence> = fences(&the_section())
        .into_iter()
        .filter(|fence| fence.info == "toml refused")
        .collect();
    for (number, (inside, stray)) in [
        ("[[arrangements]]", "desk = "),
        ("[[arrangements.screens]]", "brightness = "),
    ]
    .into_iter()
    .enumerate()
    {
        let fence = refused
            .iter()
            .find(|fence| fence.text.contains(inside) && fence.text.contains(stray))
            .unwrap_or_else(|| {
                panic!("the section shows no refused example of {stray:?} inside {inside}")
            });
        let named = code_span_beginning(&fence.after, "displays.kept.")
            .unwrap_or_else(|| panic!("no sentence is named after:\n{}", fence.text));
        assert_eq!(
            named, "displays.kept.not-understood",
            "a key inside a value is not the sentence for a key at the top of the file"
        );

        let (at, read) = read_from_a_file(&format!("a-key-inside-{number}"), &fence.text);
        let Err(why) = read else {
            panic!("the section says this is refused:\n{}", fence.text)
        };
        assert_eq!(why.word().named(), named, "{}", fence.text);
        assert_eq!(
            why.key(),
            None,
            "the key is inside a value, so there is no key at the top to name"
        );
        assert_eq!(
            keeping::at_sign_in(&at),
            (Changes::untouched(), Some(why)),
            "nothing in the file is honoured"
        );

        let without: String = fence
            .text
            .lines()
            .filter(|line| !line.starts_with(stray))
            .map(|line| format!("{line}\n"))
            .collect();
        assert_ne!(
            without, fence.text,
            "the stray line was not found to remove"
        );
        let (_, read) = read_from_a_file(&format!("without-the-key-{number}"), &without);
        let read = read.unwrap_or_else(|refused| {
            panic!(
                "the same file without {stray:?} does not read: {:?}\n{without}",
                refused.why()
            )
        });
        assert_eq!(
            read.how_many(),
            1,
            "the arrangement the stray key was in is honoured once the key is gone"
        );
    }
}

/// **No file is what the section says it is**: a person who has arranged
/// nothing and never asked for night light, not an error, and nothing written.
#[test]
fn a_missing_file_is_a_person_who_arranged_nothing() {
    assert!(
        the_part("What a missing file means")
            .contains("The person has arranged nothing and has never asked for night light")
    );
    let at = a_folder_of_our_own("missing").join(THE_FILE);
    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
    assert_eq!(keeping::at_sign_in(&at), (Changes::untouched(), None));
    assert!(!at.exists(), "reading wrote nothing");
}

/// **The section's *Writing it* says what the writer does with a file that did
/// not read, and the crate does it**: the sentence it names is the one a change
/// over the section's first refused example is refused with, that file's bytes
/// are unchanged, and the door it names for putting the section back replaces
/// the file with the `format` line alone.
#[test]
fn the_contract_says_a_file_that_did_not_read_is_not_written_over() {
    let writing = the_part("Writing it");
    assert!(
        writing.contains("A file that does not read is not written over"),
        "{writing}"
    );
    assert!(
        !writing.contains("not yet"),
        "the section still describes a rule the crate does not hold: {writing}"
    );
    let named = code_span_beginning(&writing, "displays.kept.not-replaced")
        .expect("the section names the sentence a change over a broken file is refused with");
    assert!(
        writing.contains("alo_displays::keeping::put_back_as_shipped"),
        "the section does not name the one door that replaces such a file"
    );
    assert!(
        writing.contains(&format!("`format = {FORMAT}`")),
        "{writing}"
    );

    let refused = fences(&the_section())
        .into_iter()
        .find(|fence| fence.info == "toml refused")
        .expect("the section shows a file that does not read");
    let (at, read) = read_from_a_file("not-written-over", &refused.text);
    assert!(read.is_err());
    let before = std::fs::read(&at).unwrap();

    let why = keeping::keep(&at, &everything_changed()).unwrap_err();
    assert_eq!(why.word().named(), named);
    assert!(
        why.did_not_read().is_some(),
        "the refusal does not say what is wrong with the file"
    );
    assert_eq!(std::fs::read(&at).unwrap(), before);

    keeping::put_back_as_shipped(&at).unwrap();
    assert_eq!(
        std::fs::read_to_string(&at).unwrap(),
        format!("format = {FORMAT}\n")
    );
    assert_eq!(keeping::read(&at).unwrap(), Changes::untouched());
}
