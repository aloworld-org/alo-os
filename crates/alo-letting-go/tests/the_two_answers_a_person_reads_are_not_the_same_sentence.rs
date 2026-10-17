//! The two things this crate can find are reported as two different sentences.
//!
//! There are two ways for a run to remove nothing, and they are **not the same
//! fact**:
//!
//! | what happened | what a person should do |
//! |---|---|
//! | the filesystem cannot keep an undo at all | reinstall, or accept that undo will never work here |
//! | the folder was looked at and held nothing | nothing; this is a machine working |
//!
//! Until 2026-10-17 the first was written to the journal as *"this machine keeps
//! nothing an undo could put back"*, which reads as the second. One sentence for
//! two facts, and the one it reads as is the one that needs no action — so the
//! machine that can never keep an undo was the one nobody would look into.
//!
//! # Why this reads the source rather than running the binaries
//!
//! Because the sentences are `eprintln!` in two `main`s, going to a service
//! journal. Reaching them means a machine, a `btrfs` filesystem and a unit
//! start; and the fault being guarded is **a wording somebody reverts**, which
//! no input can provoke. So the thing to read is the file.
//!
//! It is the shape `alo-sessiond`'s systemd guard and
//! `alo-giving-a-folder`'s owner guard already have, for the same reason: a
//! fault that cannot be reached from a test is still a fault, and the source is
//! where it lives.
//!
//! # These are not translated, and that is not an oversight
//!
//! `CLAUDE.md` requires user-facing strings to be externalised. These are not
//! user-facing: they go to `StandardError=journal`, read by whoever administers
//! the machine, and both units' own comments say so. **What a *person* reads is
//! the record the run wrote**, in their own language, through
//! `alo-changing-undo`. A journal line in the language of the service log is
//! the convention every other unit in this repository follows.

#![expect(
    clippy::panic,
    reason = "in a test, a panic on a source file that no longer says what it must is the \
              failure being reported"
)]

use std::path::{Path, PathBuf};

/// The two programs that report this, and the word each uses for its own job.
const BOTH: [(&str, &str); 2] = [
    ("alo-letting-go.rs", "let go of"),
    ("alo-forgetting.rs", "forget"),
];

/// The phrasing that blurred the two facts, and may not come back.
///
/// Matched as a phrase rather than by meaning, which is all a test can do — but
/// it is the exact sentence that was there, so a revert is what it catches, and
/// a fresh wording of the same mistake is what it cannot. Named here so that the
/// next person at least meets the question.
const THE_BLURRED_ONE: &str = "keeps nothing an undo could put back";

/// Where a program's source is.
fn source_of(named: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/bin")
        .join(named)
}

/// What a program's source says, with its line continuations joined.
///
/// **Normalised before matching, and that is not a convenience.** Rust string
/// literals in this repository are wrapped with a trailing `\\` and the next
/// line's leading whitespace is swallowed — so a sentence a reader sees as one
/// phrase is two fragments in the file, and a plain `contains` answers *no* for
/// a phrase that is plainly there. The first version of this test did exactly
/// that and failed on a sentence it was asserting about.
///
/// So: drop every backslash-newline and the indentation after it, then collapse
/// runs of whitespace to one space. What is matched is then the sentence as a
/// person reads it in the journal, which is the subject of this test.
fn said_by(named: &str) -> String {
    let at = source_of(named);
    let raw = std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()));
    let joined = raw.replace("\\\n", "");
    joined.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// **Neither program uses the sentence that meant both things.**
#[test]
fn neither_program_says_the_sentence_that_blurred_the_two() {
    for (named, _) in BOTH {
        let said = said_by(named);
        assert!(
            !said.contains(THE_BLURRED_ONE),
            "{} is back to {THE_BLURRED_ONE:?}, which a person reads as *nothing is kept right \
             now* while it means *nothing can ever be kept here*",
            source_of(named).display()
        );
    }
}

/// **And each says, in the filesystem case, that nothing *can* be kept.**
///
/// The positive half. Without it, deleting the sentence entirely would pass the
/// test above — which is the usual way a forbidden-phrase check gets satisfied.
#[test]
fn each_program_says_the_filesystem_cannot_rather_than_does_not() {
    for (named, what) in BOTH {
        let said = said_by(named);
        assert!(
            said.contains("cannot keep an undo at all"),
            "{} does not say the filesystem *cannot* keep an undo, so a person cannot tell this \
             case from a machine that simply has nothing to {what}",
            source_of(named).display()
        );
        assert!(
            said.contains("it is that nothing can"),
            "{} does not say that nothing *can* be kept, which is the half that distinguishes \
             this from the ordinary empty case",
            source_of(named).display()
        );
    }
}

/// **And it names what would change it**, because a sentence a person can do
/// nothing about is a sentence they learn to skip.
#[test]
fn each_program_says_what_would_change_it() {
    for (named, _) in BOTH {
        let said = said_by(named);
        assert!(
            said.contains("reinstalled"),
            "{} says the machine cannot keep an undo and not what would change that",
            source_of(named).display()
        );
        assert!(
            said.contains("ADR 0045 term 6"),
            "{} does not cite the term that makes this an answer rather than a failure",
            source_of(named).display()
        );
    }
}

/// **The two programs' sentences are different from each other.**
///
/// Each says *let go of* or *forget* in its own, because a journal holding both
/// units' lines should not need the unit name to tell which ran. A copy-paste
/// that left one saying the other's job is what this catches.
#[test]
fn the_two_programs_do_not_claim_each_others_job() {
    let letting_go = said_by("alo-letting-go.rs");
    let forgetting = said_by("alo-forgetting.rs");

    assert!(
        letting_go.contains("nothing to let go of"),
        "alo-letting-go.rs does not say it had nothing to let go of"
    );
    assert!(
        forgetting.contains("nothing to forget"),
        "alo-forgetting.rs does not say it had nothing to forget"
    );
    assert!(
        !letting_go.contains("nothing to forget"),
        "alo-letting-go.rs claims the forgetter's job"
    );
    assert!(
        !forgetting.contains("nothing to let go of"),
        "alo-forgetting.rs claims the remover's job"
    );
}

/// **The control**: this check can fail, and the files it reads are the right
/// ones.
///
/// Two things, because both are ways the tests above pass while measuring
/// nothing. A `said_by` that returned an empty string would satisfy every
/// `!contains` above — so each file is asserted to be a program at all; and the
/// forbidden-phrase filter is run over a line that does contain it, so finding
/// none above means something.
#[test]
fn the_files_this_reads_are_programs_and_the_filter_can_find_the_phrase() {
    for (named, _) in BOTH {
        let said = said_by(named);
        assert!(
            said.contains("fn main()") || said.contains("fn main ()"),
            "{} is not a program, so reading it proves nothing",
            source_of(named).display()
        );
        assert!(
            said.len() > 500,
            "{} is {} bytes, which is too short to be the program this test is about",
            source_of(named).display(),
            said.len()
        );
    }

    let pretend = format!("eprintln!(\"this machine {THE_BLURRED_ONE}\");");
    assert!(
        pretend.contains(THE_BLURRED_ONE),
        "the filter cannot find the phrase when it is there, so finding it nowhere meant nothing"
    );
}
