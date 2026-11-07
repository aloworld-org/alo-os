//! Every form a person can type, and every refusal of one, against a machine
//! with nothing real behind it.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None, Err or variant is the failure being reported"
)]

use alo_acting::{Answer, Counted, InFront, NotActed, WhereAlosTreeIs};
use alo_record::{Answered, Entry, Happened, Road, SentDuring};

use crate::door::Unanswered;
use crate::ended::Ended;
use crate::machine::APlace;
use crate::testing::{ALOS_TREE, AMachine, ATree, Counting, EDITOR, Known};

/// The parts of one written entry a test asks about.
struct Written<'a> {
    /// The application, by identifier.
    application: &'a str,
    /// The ability.
    ability: &'a str,
    /// The road, if any.
    road: Option<Road>,
    /// How many arguments were kept.
    arguments: Vec<(String, String)>,
    /// What came of it.
    answered: &'a Answered,
    /// What was sent.
    sent: Option<&'a SentDuring>,
}

/// One entry, read.
fn written(entry: &Entry) -> Written<'_> {
    match entry.happened() {
        Happened::AskedAnApplication {
            application,
            ability,
            road,
            arguments,
            answered,
            sent,
        } => Written {
            application: application.as_str(),
            ability: ability.as_str(),
            road: *road,
            arguments: arguments
                .iter()
                .map(|given| {
                    (
                        given.name.as_str().to_owned(),
                        given.value.as_str().to_owned(),
                    )
                })
                .collect(),
            answered,
            sent: sent.as_ref(),
        },
        other => panic!("not a person's own command: {other:?}"),
    }
}

/// The editor's window, with a Save button, on the tree.
fn the_editors_window() -> ATree {
    ATree::default().with(
        ":1.20",
        Some(EDITOR),
        "today.txt",
        &["Save", "Help", "Help"],
        "Hello",
    )
}

// ---------------------------------------------------------------------------
// Looking writes nothing.
// ---------------------------------------------------------------------------

/// **A look writes nothing in the record**, and never asks for a place.
#[test]
fn list_show_controls_and_read_write_nothing() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(the_editors_window()),
        ..AMachine::default()
    };
    for line in [
        &["list"][..],
        &["show", "texteditor"],
        &["show", "texteditor", "open_document"],
        &["texteditor", "controls"],
        &["texteditor", "read"],
    ] {
        assert_eq!(known.running(line, &mut machine), Ended::Done, "{line:?}");
    }
    assert!(machine.kept.is_empty(), "a look was written down");
    assert_eq!(machine.places_asked, 0);
    assert!(
        machine
            .told
            .iter()
            .any(|line| line.contains("button “Save”")),
        "{:?}",
        machine.told
    );
    assert!(machine.told.iter().any(|line| line == "Hello"));
}

/// **On a pipe, a control is one line of values**, never translated.
#[test]
fn controls_on_a_pipe_are_one_value_per_line() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(the_editors_window()),
        ..AMachine::default()
    };
    assert_eq!(
        known.piped(&["texteditor", "controls"], &mut machine),
        Ended::Done
    );
    assert!(
        machine.told.contains(&"button\tSave\t".to_owned()),
        "{:?}",
        machine.told
    );
}

/// **`show` says the roads it did not take**, rather than leaving them out.
#[test]
fn show_names_what_was_not_looked_for() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["show", "texteditor"], &mut machine),
        Ended::Done
    );
    let shown = machine.told.join("\n");
    assert!(shown.contains("open_document"), "{shown}");
    assert!(shown.contains("does not look there yet"), "{shown}");
    assert!(
        shown.contains("Reading the screen is not built yet"),
        "{shown}"
    );
    assert!(
        shown.contains("which this session does not have now"),
        "{shown}"
    );
}

// ---------------------------------------------------------------------------
// Acts write one entry each, with what was sent.
// ---------------------------------------------------------------------------

/// **A press writes exactly one entry, with its road, its count and its
/// place**, and the place is asked for once.
#[test]
fn a_press_writes_one_entry_with_its_count_and_place() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(the_editors_window()),
        ..AMachine::default()
    };
    machine.door.counting = Counting::Ends(Some(Counted::Sent(512)));
    let ended = known.running(&["texteditor", "press", "button", "Save"], &mut machine);
    assert_eq!(ended, Ended::Done);
    assert_eq!(machine.places_asked, 1);
    let [entry] = machine.kept.as_slice() else {
        panic!("{:?}", machine.kept)
    };
    let entry_read = written(entry);
    assert_eq!(entry_read.application, EDITOR);
    assert_eq!(entry_read.ability, "press");
    assert_eq!(entry_read.road, Some(Road::Tree));
    assert_eq!(entry_read.answered, &Answered::Done);
    assert_eq!(entry_read.sent, Some(&SentDuring::Counted { bytes: 512 }));
    assert!(
        entry
            .kept_before_it()
            .is_some_and(|place| place.is("1762516800-42-their-own"))
    );
    assert!(entry.agent().is_none(), "the person acted, not an agent");
    assert!(machine.door.asked.contains(&format!("count {EDITOR}")));
    assert_eq!(
        machine.told,
        ["Pressed the button “Save” in org.gnome.TextEditor"]
    );
}

/// **A declared ability is validated, sent once, and written down with what
/// it was given** — a relative path taken from the folder the shell is in.
#[test]
fn a_declared_ability_is_sent_once_and_written_down() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    let ended = known.running(&["texteditor", "open_document", "today.txt"], &mut machine);
    assert_eq!(ended, Ended::Done, "{:?}", machine.warned);
    assert_eq!(machine.bus.sent.borrow().len(), 1);
    let [entry] = machine.kept.as_slice() else {
        panic!("{:?}", machine.kept)
    };
    let entry_read = written(entry);
    assert_eq!(entry_read.road, Some(Road::Declared));
    assert_eq!(
        entry_read.arguments,
        [(
            "document".to_owned(),
            "/home/anna/Notes/today.txt".to_owned()
        )]
    );
    assert!(machine.told.iter().any(|line| line.starts_with("Done: ")));
}

/// **`--document` reaches the same argument by name**, and naming it twice is
/// refused before anything is sent.
#[test]
fn an_argument_is_given_once_by_order_or_by_name() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(
            &["texteditor", "open_document", "--document", "/tmp/a.txt"],
            &mut machine
        ),
        Ended::Done
    );
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(
            &["texteditor", "open_document", "/a", "--document", "/b"],
            &mut machine
        ),
        Ended::NotThisShape
    );
    assert!(machine.bus.sent.borrow().is_empty());
}

/// **A window act goes through the desktop, and writes one entry with no
/// place**, because it changes no file.
#[test]
fn a_window_act_is_asked_of_the_desktop_and_written_down() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["texteditor", "close"], &mut machine),
        Ended::Done
    );
    assert!(machine.door.asked.contains(&format!("close {EDITOR}")));
    assert_eq!(machine.places_asked, 0);
    let [entry] = machine.kept.as_slice() else {
        panic!()
    };
    assert_eq!(written(entry).road, Some(Road::Window));

    for (answer, ended) in [
        (Ok(Answer::NotActed(NotActed::NoWindow)), Ended::Refused),
        (Ok(Answer::NotActed(NotActed::ItDidNot)), Ended::Refused),
        (
            Ok(Answer::NotActed(NotActed::CouldNotAsk)),
            Ended::NotOffered,
        ),
        (
            Ok(Answer::NotActed(NotActed::NotInstalled)),
            Ended::NotOffered,
        ),
        (Err(Unanswered::NotReached), Ended::NotOffered),
        (Err(Unanswered::NotUnderstood), Ended::NotKnown),
    ] {
        let mut machine = AMachine::default();
        machine.door.acting = answer;
        assert_eq!(
            known.running(&["texteditor", "focus"], &mut machine),
            ended,
            "{answer:?}"
        );
        assert_eq!(machine.kept.len(), 1, "{answer:?}");
        assert!(!machine.warned.is_empty(), "{answer:?} said nothing");
    }
}

// ---------------------------------------------------------------------------
// What the application sends.
// ---------------------------------------------------------------------------

/// **A count that cannot be held does not stop the act**: the person is told,
/// in one sentence, and the record keeps that sentence.
#[test]
fn an_act_that_cannot_be_counted_runs_and_says_so() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(the_editors_window()),
        ..AMachine::default()
    };
    machine.door.counting = Counting::Cannot;
    assert_eq!(
        known.running(&["texteditor", "press", "button", "Save"], &mut machine),
        Ended::Done
    );
    assert!(
        machine
            .warned
            .iter()
            .any(|line| line
                == "alo cannot show what org.gnome.TextEditor sends during this command"),
        "{:?}",
        machine.warned
    );
    let [entry] = machine.kept.as_slice() else {
        panic!()
    };
    assert!(matches!(
        written(entry).sent,
        Some(SentDuring::NotCounted { why }) if why.as_str().starts_with("alo cannot show")
    ));
}

/// **A count the desktop stopped is kept as at least**, never as the whole.
#[test]
fn a_stopped_count_is_kept_as_at_least() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    machine.door.counting = Counting::Ends(Some(Counted::Stopped(9)));
    assert_eq!(
        known.running(&["texteditor", "open"], &mut machine),
        Ended::Done
    );
    assert_eq!(
        written(machine.kept.first().unwrap()).sent,
        Some(&SentDuring::AtLeast { bytes: 9 })
    );
}

/// **A place that was not kept is said once**, and the command still runs.
#[test]
fn no_place_is_said_once_and_the_command_runs() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        place: APlace::NotKept("No place was kept".to_owned()),
        ..AMachine::default()
    };
    machine
        .pipe
        .extend(["/a.txt".to_owned(), "/b.txt".to_owned()]);
    assert_eq!(
        known.running(&["texteditor", "open_document", "-"], &mut machine),
        Ended::Done
    );
    assert_eq!(machine.places_asked, 1, "one place per command line");
    assert_eq!(
        machine
            .warned
            .iter()
            .filter(|line| *line == "No place was kept")
            .count(),
        1
    );
    assert_eq!(machine.kept.len(), 2);
    assert!(
        machine
            .kept
            .iter()
            .all(|entry| entry.kept_before_it().is_none())
    );
}

// ---------------------------------------------------------------------------
// Pipes.
// ---------------------------------------------------------------------------

/// **One line, one call, one entry; a refused line does not stop the others;
/// the command ends with the worst.**
#[test]
fn a_refused_line_of_the_pipe_does_not_stop_the_others() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    machine.pipe.extend([
        "/one.txt".to_owned(),
        "/two/../etc/passwd".to_owned(),
        "a\u{1b}[31m".to_owned(),
        "/three.txt".to_owned(),
    ]);
    let ended = known.running(&["texteditor", "open_document", "-"], &mut machine);
    assert_eq!(ended, Ended::ValueRefused);
    assert_eq!(
        machine.bus.sent.borrow().len(),
        2,
        "the two good lines were sent"
    );
    assert_eq!(machine.kept.len(), 4, "every line is one entry");
    let refused: Vec<_> = machine
        .kept
        .iter()
        .map(written)
        .filter(|one| one.road.is_none())
        .collect();
    assert_eq!(refused.len(), 2);
    assert!(
        refused.iter().all(|one| one.arguments.is_empty()),
        "a refusal kept what arrived"
    );
    assert!(
        machine.warned.iter().all(|line| !line.contains('\u{1b}')),
        "an escape sequence reached the terminal"
    );
}

/// **Two values from the pipe are refused**, before anything is read.
#[test]
fn two_pipes_are_refused() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    machine.pipe.push_back("x".to_owned());
    assert_eq!(
        known.running(&["texteditor", "press", "-", "-"], &mut machine),
        Ended::NotThisShape
    );
    assert_eq!(machine.pipe.len(), 1, "the pipe was read");
}

// ---------------------------------------------------------------------------
// Naming, and its refusals.
// ---------------------------------------------------------------------------

/// **An unknown application is refused with 69, and the refused act is
/// written down; a refused look is not.**
#[test]
fn an_unknown_application_is_refused() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["gedit", "press", "button", "Save"], &mut machine),
        Ended::NotOffered
    );
    let [entry] = machine.kept.as_slice() else {
        panic!()
    };
    assert_eq!(written(entry).application, "gedit");
    assert_eq!(written(entry).ability, "press");
    assert!(written(entry).arguments.is_empty());

    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["gedit", "controls"], &mut machine),
        Ended::NotOffered
    );
    assert!(machine.kept.is_empty());
}

/// **A short name two applications share is refused, naming both.**
#[test]
fn a_shared_short_name_is_refused_naming_both() {
    let known = Known::holding(&[EDITOR, "org.kde.TextEditor"]);
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["texteditor", "open"], &mut machine),
        Ended::ValueRefused
    );
    let said = machine.warned.join("\n");
    assert!(
        said.contains("org.gnome.TextEditor, org.kde.TextEditor"),
        "{said}"
    );
    assert!(
        !machine
            .door
            .asked
            .iter()
            .any(|line| line.starts_with("open"))
    );
}

/// **A kind `press` does not take is refused, and the entry keeps no values.**
#[test]
fn an_unknown_kind_is_refused() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(the_editors_window()),
        ..AMachine::default()
    };
    assert_eq!(
        known.running(&["texteditor", "press", "slider", "Volume"], &mut machine),
        Ended::ValueRefused
    );
    assert_eq!(*machine.tree.as_ref().unwrap().pressed.borrow(), 0);
    let [entry] = machine.kept.as_slice() else {
        panic!()
    };
    assert!(written(entry).arguments.is_empty());
    assert!(
        machine
            .warned
            .iter()
            .any(|line| line.contains("button, check_box"))
    );
}

/// **The wrong number of values is the wrong shape (64)**, and an ability the
/// application does not offer is 69.
#[test]
fn the_wrong_count_of_values_and_an_unknown_ability_are_refused() {
    let known = Known::holding(&[EDITOR]);
    for (line, ended) in [
        (&["texteditor", "open_document"][..], Ended::NotThisShape),
        (
            &["texteditor", "open_document", "/a", "/b"],
            Ended::NotThisShape,
        ),
        (&["texteditor", "new_window", "/a"], Ended::NotThisShape),
        (&["texteditor", "print"], Ended::NotOffered),
        (&["texteditor", "press", "button"], Ended::NotThisShape),
    ] {
        let mut machine = AMachine::default();
        assert_eq!(known.running(line, &mut machine), ended, "{line:?}");
        assert!(machine.bus.sent.borrow().is_empty(), "{line:?}");
        assert_eq!(machine.kept.len(), 1, "{line:?} was not written down");
    }
}

/// **A control that is not there, or is there twice, is refused with 1 and
/// nothing is pressed.**
#[test]
fn a_control_that_is_missing_or_doubled_is_not_pressed() {
    let known = Known::holding(&[EDITOR]);
    for name in ["Forgot", "Help"] {
        let mut machine = AMachine {
            tree: Some(the_editors_window()),
            ..AMachine::default()
        };
        assert_eq!(
            known.running(&["texteditor", "press", "button", name], &mut machine),
            Ended::Refused,
            "{name}"
        );
        assert_eq!(
            *machine.tree.as_ref().unwrap().pressed.borrow(),
            0,
            "{name}"
        );
    }
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["texteditor", "press", "button", "Save"], &mut machine),
        Ended::NotOffered,
        "no tree at all"
    );
}

// ---------------------------------------------------------------------------
// front and alo.
// ---------------------------------------------------------------------------

/// **`front` acts on that window, and the record keeps the real identifier
/// and the window, never the word.**
#[test]
fn front_acts_on_the_window_and_records_what_it_was() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(the_editors_window()),
        ..AMachine::default()
    };
    machine.door.front = Ok(InFront::window(17, EDITOR, "today.txt"));
    assert_eq!(
        known.running(&["front", "press", "button", "Save"], &mut machine),
        Ended::Done
    );
    assert_eq!(
        machine
            .door
            .asked
            .iter()
            .filter(|line| *line == "front")
            .count(),
        1,
        "front is asked once"
    );
    let entry = written(machine.kept.first().unwrap());
    assert_eq!(entry.application, EDITOR);
    assert!(
        entry
            .arguments
            .contains(&("window".to_owned(), "today.txt".to_owned()))
    );
    assert!(
        machine.told.iter().all(|line| line.contains("today.txt")),
        "{:?}",
        machine.told
    );
}

/// **`front` with nothing in front, or a window nobody can name, is refused
/// in the contract's sentence**, and nothing is written about a word.
#[test]
fn front_with_nothing_to_name_is_refused() {
    let known = Known::holding(&[EDITOR]);
    for (answer, says) in [
        (
            Ok(InFront::None),
            "There is no window you were working in before this terminal",
        ),
        (
            Ok(InFront::Unnamed),
            "does not say which application it belongs to",
        ),
        (
            Ok(InFront::window(3, "org.example.NotHere", "x")),
            "does not say which application it belongs to",
        ),
        (Err(Unanswered::NotReached), "The desktop did not answer"),
    ] {
        let mut machine = AMachine::default();
        machine.door.front = answer.clone();
        assert_eq!(
            known.running(&["front", "press", "button", "Save"], &mut machine),
            Ended::NotOffered,
            "{answer:?}"
        );
        assert!(
            machine.warned.iter().any(|line| line.contains(says)),
            "{:?}",
            machine.warned
        );
        assert!(machine.kept.is_empty(), "the word front was written down");
    }
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["front", "close"], &mut machine),
        Ended::NotThisShape,
        "front cannot close a whole application"
    );
    assert_eq!(
        known.running(
            &["front", "controls", "--in", "x"],
            &mut AMachine::default()
        ),
        Ended::NotThisShape
    );
}

/// **`alo` is reached at the name the desktop gives**, by the tree only.
#[test]
fn alo_is_reached_through_its_own_tree_only() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        tree: Some(ATree::default().with(ALOS_TREE, None, "Settings", &["Wi-Fi"], "")),
        ..AMachine::default()
    };
    machine.door.alos_tree = Ok(WhereAlosTreeIs::At(ALOS_TREE.to_owned()));
    assert_eq!(
        known.running(&["alo", "press", "button", "Wi-Fi"], &mut machine),
        Ended::Done,
        "{:?}",
        machine.warned
    );
    let entry = written(machine.kept.first().unwrap());
    assert_eq!(entry.application, "alo");
    assert!(matches!(entry.sent, Some(SentDuring::NotCounted { .. })));
    assert!(
        !machine
            .door
            .asked
            .iter()
            .any(|line| line.starts_with("count"))
    );

    for (answer, ended) in [
        (Ok(WhereAlosTreeIs::Nowhere), Ended::NotOffered),
        (Err(Unanswered::NotUnderstood), Ended::NotOffered),
    ] {
        let mut machine = AMachine::default();
        machine.door.alos_tree = answer.clone();
        assert_eq!(
            known.running(&["alo", "controls"], &mut machine),
            ended,
            "{answer:?}"
        );
    }
    let mut machine = AMachine::default();
    assert_eq!(
        known.running(&["alo", "open"], &mut machine),
        Ended::NotOffered
    );
    assert!(
        !machine
            .door
            .asked
            .iter()
            .any(|line| line.starts_with("open")),
        "alo's own surfaces went by another road"
    );
}

/// **A record that cannot be opened stops the act before anything happens.**
#[test]
fn an_act_that_cannot_be_written_down_does_not_happen() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine {
        record_opens: false,
        ..AMachine::default()
    };
    assert_eq!(
        known.running(&["texteditor", "open_document", "/a.txt"], &mut machine),
        Ended::NotOffered
    );
    assert!(machine.bus.sent.borrow().is_empty());
    assert!(machine.door.asked.is_empty());
}

/// **A line not in the shape is 64, and nothing is asked of the machine.**
#[test]
fn a_line_not_in_the_shape_asks_nothing() {
    let known = Known::holding(&[EDITOR]);
    let mut machine = AMachine::default();
    assert_eq!(known.running(&[], &mut machine), Ended::NotThisShape);
    assert!(machine.door.asked.is_empty());
    assert!(
        machine
            .warned
            .first()
            .unwrap()
            .starts_with("Nothing was asked")
    );
}
