//! The bracket is held to the three things this shape gets wrong: a turn name
//! that reaches out of the person's directory, a missing snapshot read as a
//! failed turn, and a note nobody can read months later.
//!
//! **Nothing here makes a real snapshot.** `btrfs` needs a real filesystem, and
//! `tests/a_bracket_on_a_real_btrfs_machine.rs` is where that is measured —
//! root, a loop device and the base's own program, run by name. What is held
//! here is everything that does not need one, which is most of it.

use super::*;

/// A person's own undo directory, in a temporary place.
fn a_persons_directory() -> tempfile::TempDir {
    tempfile::tempdir().expect("a temporary directory")
}

/// **A turn's directory is made, and it is inside the person's own.**
#[test]
fn a_turns_directory_is_made_under_the_persons() {
    let whose = a_persons_directory();
    let kept = WhereThisTurnIsKept::made(whose.path(), "4f1c8a").expect("a turn's directory");

    assert!(kept.at().is_dir(), "the directory was not made");
    assert_eq!(
        kept.at().parent(),
        Some(whose.path()),
        "the turn's directory is not directly inside the person's"
    );
}

/// **A name that is not a turn's name is refused before anything is made**, and
/// the refusal names what was offered.
///
/// This is the one refusal here that is about safety rather than about room: a
/// caller handing `..` would otherwise be handed a path outside the person's own
/// directory, and the undo folder is `0711` precisely so that one person cannot
/// reach another's.
#[test]
fn a_name_that_could_leave_the_persons_directory_is_refused() {
    let whose = a_persons_directory();
    for offered in [
        "..",
        "../..",
        "a/b",
        "/etc",
        ".",
        "",
        "4F1C8A",
        "four-f-one",
        "4f1c8a.",
        "with space",
    ] {
        let refused = WhereThisTurnIsKept::made(whose.path(), offered)
            .expect_err(&format!("{offered:?} was accepted as a turn's name"));
        assert_eq!(
            refused,
            WhyNot::ThatIsNotATurnsName {
                offered: offered.to_owned()
            },
            "{offered:?} was refused for the wrong reason"
        );
    }
    // And nothing was made while refusing, which is the half a test of the
    // error alone would not see.
    let made: Vec<_> = std::fs::read_dir(whose.path())
        .expect("the person's directory")
        .collect();
    assert!(made.is_empty(), "a refused name still made something");
}

/// **And an ordinary hex name is accepted**, which is the control: without it
/// the test above would pass on a function that refused everything.
#[test]
fn an_ordinary_turn_name_is_accepted() {
    let whose = a_persons_directory();
    for offered in ["4f1c8a", "0", "deadbeef", &"a".repeat(64)] {
        assert!(
            WhereThisTurnIsKept::made(whose.path(), offered).is_ok(),
            "{offered:?} was refused and it is a turn's name"
        );
    }
    // One character past the bound, to show the bound is the bound.
    assert!(
        WhereThisTurnIsKept::made(whose.path(), &"a".repeat(65)).is_err(),
        "a name longer than the bound was accepted"
    );
}

/// **A side that could not be kept is not an error**, and the type is what
/// makes that true rather than a caller remembering it.
///
/// There is no `btrfs` on a machine running this test, or there is and the
/// temporary directory is not a subvolume — either way the answer is
/// `NotKept`, and a caller **cannot** `?` it into a failed turn because it is
/// not a `Result`. ADR 0045's third term: *a verb that refused to run for want
/// of an undo would be a machine that stopped working to protect a feature.*
#[test]
fn a_side_that_cannot_be_kept_answers_why_rather_than_failing() {
    let whose = a_persons_directory();
    let kept = WhereThisTurnIsKept::made(whose.path(), "4f1c8a").expect("a turn's directory");

    let side = kept.keeping(alo_kept_undo::BEFORE, whose.path());
    assert!(
        side.kept_at().is_none(),
        "a temporary directory was snapshotted, which this test is not about"
    );
    let why = side.why_not().expect("a reason it was not kept");
    assert!(
        matches!(
            why,
            WhyNot::TheProgramWouldNotRun { .. } | WhyNot::ItRefused { .. }
        ),
        "the reason is neither the program nor its refusal: {why:?}"
    );
}

/// **The note says when the turn ran and what it did**, and it reads back.
///
/// Written and then read through `alo-kept-undo`'s own reader, because a note
/// this crate can write and that crate cannot read is a note nobody will ever
/// see: the privileged remover reads it, and a person reads it months later.
#[test]
fn the_note_is_written_and_reads_back() {
    let whose = a_persons_directory();
    let kept = WhereThisTurnIsKept::made(whose.path(), "4f1c8a").expect("a turn's directory");

    let noon = std::time::SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000);
    let said = "move /home/anna/Invoices/march.pdf into /home/anna/Archive";
    kept.what_it_did(noon, said).expect("the note is written");

    let at = kept.at().join(alo_kept_undo::THE_TURN);
    let text = std::fs::read_to_string(&at).expect("the note");
    let read = alo_kept_undo::TheTurn::read(&text, &at).expect("the note reads back");
    assert_eq!(
        read.did(),
        said,
        "the sentence the person approved was lost"
    );
    assert_eq!(read.taken(), noon);
}

/// **A note with nothing in it is refused**, because a turn whose sentence is
/// blank is one a person cannot be told about.
#[test]
fn a_note_with_no_sentence_is_refused() {
    let whose = a_persons_directory();
    let kept = WhereThisTurnIsKept::made(whose.path(), "4f1c8a").expect("a turn's directory");

    for blank in ["", "   ", "\t\n"] {
        assert!(
            kept.what_it_did(std::time::SystemTime::UNIX_EPOCH, blank)
                .is_err(),
            "a blank sentence was written down: {blank:?}"
        );
    }
    assert!(
        !kept.at().join(alo_kept_undo::THE_TURN).exists(),
        "a refused note left a file behind"
    );
}

/// **Where their settings are is written once and not rewritten.**
///
/// Without the file the privileged remover steps over the person's directory
/// whole and their disk is never tidied; rewriting it every turn would make the
/// one file it depends on the file most often half-written.
#[test]
fn where_their_settings_are_is_written_once() {
    let whose = a_persons_directory();
    let settings = Path::new("/home/anna/.config/alo");

    where_their_settings_are(whose.path(), settings).expect("it is written");
    let at = whose.path().join(alo_kept_undo::THEIRS);
    let first = std::fs::read_to_string(&at).expect("the file");
    let read = alo_kept_undo::Theirs::read(&first, &at).expect("it reads back");
    assert_eq!(read.settings(), settings);

    // Again, with a different folder: the file must not move.
    where_their_settings_are(whose.path(), Path::new("/home/somebody-else/.config/alo"))
        .expect("the second call is not an error");
    assert_eq!(
        std::fs::read_to_string(&at).expect("the file"),
        first,
        "a second call rewrote the file the remover depends on"
    );
}

/// **A settings folder that is not absolute is refused**, because a relative
/// one would be followed from wherever a unit happened to start.
#[test]
fn a_relative_settings_folder_is_refused() {
    let whose = a_persons_directory();
    assert!(
        where_their_settings_are(whose.path(), Path::new(".config/alo")).is_err(),
        "a relative settings folder was written down"
    );
    assert!(
        !whose.path().join(alo_kept_undo::THEIRS).exists(),
        "a refused settings folder left a file behind"
    );
}

/// **One program is started here and it is the one this file names.**
///
/// The source is read because the promise is about what is *not* here: a second
/// road to a snapshot would be a second implementation to keep correct, which
/// ADR 0045 forbids and `CLAUDE.md` calls *engines are configured, never
/// written in*. The same shape `alo-letting-go`'s remover already has.
#[test]
fn one_program_is_started_and_it_is_the_named_one() {
    let source = include_str!("the_bracket_a_turn_takes.rs");
    let started = source.matches("Command::new(").count();
    assert_eq!(started, 1, "more than one program is started here");
    assert!(
        source.contains("Command::new(THE_PROGRAM)"),
        "the program started is not the one this file names"
    );
    // And it is asked for a read-only snapshot, which is what #752's decision
    // rests on: the agent may neither alter nor destroy a bracket.
    assert!(
        THE_ARGUMENTS.contains(&"-r"),
        "the snapshot is not asked for read-only: {THE_ARGUMENTS:?}"
    );
}
