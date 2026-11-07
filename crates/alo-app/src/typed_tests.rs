//! The shape of a command line, and every way it is not the shape.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use super::*;

/// The words of a command line, as a shell hands them over.
fn typed(line: &[&str]) -> Vec<String> {
    line.iter().map(|word| (*word).to_owned()).collect()
}

/// Read one line.
fn reading(line: &[&str]) -> Result<Typed, NotTyped> {
    read(&typed(line))
}

#[test]
fn list_and_show_are_read_as_looks() {
    assert_eq!(reading(&["list"]).unwrap().asked, Asked::List);
    assert_eq!(
        reading(&["show", "texteditor"]).unwrap().asked,
        Asked::Show {
            application: "texteditor".to_owned(),
            ability: None
        }
    );
    assert_eq!(
        reading(&["--json", "show", "texteditor", "open_document"]).unwrap(),
        Typed {
            asking: Asking::Json,
            asked: Asked::Show {
                application: "texteditor".to_owned(),
                ability: Some("open_document".to_owned())
            }
        }
    );
}

#[test]
fn every_form_is_read_as_itself() {
    let of = |line: &[&str]| match reading(line).unwrap().asked {
        Asked::Of { doing, within, .. } => (doing, within),
        other => panic!("not an act: {other:?}"),
    };
    assert_eq!(of(&["texteditor", "controls"]).0, Doing::Controls);
    assert_eq!(of(&["texteditor", "read"]).0, Doing::Read);
    assert_eq!(
        of(&["texteditor", "close"]).0,
        Doing::Window(OnTheWindow::Close)
    );
    assert_eq!(
        of(&["texteditor", "press", "button", "Save", "--in", "today.txt"]),
        (
            Doing::Press {
                kind: "button".to_owned(),
                control: "Save".to_owned()
            },
            Some("today.txt".to_owned())
        )
    );
    assert_eq!(
        of(&["texteditor", "open_document", "--document", "/a", "-"]).0,
        Doing::Ability {
            ability: "open_document".to_owned(),
            given: vec![
                GivenAs::ByName {
                    argument: "document".to_owned(),
                    value: "/a".to_owned()
                },
                GivenAs::InOrder("-".to_owned()),
            ]
        }
    );
}

/// **`--` ends the options**, so a control called `--json` can be pressed.
#[test]
fn a_double_dash_ends_the_options() {
    let read = reading(&["texteditor", "press", "--", "button", "--json"]).unwrap();
    assert_eq!(read.asking, Asking::AsTheOutputIs);
    assert!(matches!(
        read.asked,
        Asked::Of { doing: Doing::Press { control, .. }, .. } if control == "--json"
    ));
}

/// **Every way a line is not the shape says which way**, and none of them is
/// read as something else.
#[test]
fn each_way_a_line_is_not_the_shape_is_its_own_refusal() {
    assert_eq!(reading(&[]).unwrap_err(), NotTyped::Nothing);
    assert_eq!(reading(&["show"]).unwrap_err(), NotTyped::Nothing);
    assert_eq!(
        reading(&["texteditor"]).unwrap_err(),
        NotTyped::NothingToDo {
            application: "texteditor".to_owned()
        }
    );
    assert!(matches!(
        reading(&["list", "texteditor"]).unwrap_err(),
        NotTyped::LeftOver { typed, act: None } if typed == "texteditor"
    ));
    assert!(matches!(
        reading(&["show", "a", "b", "c"]).unwrap_err(),
        NotTyped::LeftOver { typed, .. } if typed == "c"
    ));
    assert_eq!(
        reading(&["texteditor", "press", "button"]).unwrap_err(),
        NotTyped::PressNeedsTwo {
            application: "texteditor".to_owned()
        }
    );
    assert!(matches!(
        reading(&["texteditor", "press", "button", "Save", "Now"]).unwrap_err(),
        NotTyped::LeftOver { typed, act: Some(_) } if typed == "Now"
    ));
    assert_eq!(
        reading(&["texteditor", "controls", "--in"]).unwrap_err(),
        NotTyped::InNeedsAWindow
    );
    assert_eq!(
        reading(&["texteditor", "controls", "--in", "a", "--in", "b"]).unwrap_err(),
        NotTyped::InTwice
    );
    assert_eq!(
        reading(&["--plain", "--json", "list"]).unwrap_err(),
        NotTyped::PlainAndJson
    );
    assert_eq!(
        reading(&["--json", "texteditor", "controls"]).unwrap_err(),
        NotTyped::JsonNotHere
    );
    assert!(matches!(
        reading(&["texteditor", "open_document", "--document"]).unwrap_err(),
        NotTyped::NamedNeedsAValue { argument, .. } if argument == "document"
    ));
    assert!(matches!(
        reading(&["list", "--in", "x"]).unwrap_err(),
        NotTyped::LeftOver { typed, .. } if typed == "--in x"
    ));
}

/// **A refusal of an act says which act**, so it can be written down; a
/// refusal of a look or of nothing in particular does not.
#[test]
fn only_a_refused_act_names_what_was_refused() {
    assert_eq!(
        reading(&["texteditor", "press", "button"])
            .unwrap_err()
            .of_an_act(),
        Some(("texteditor", "press"))
    );
    assert_eq!(
        reading(&["texteditor", "close", "now"])
            .unwrap_err()
            .of_an_act(),
        Some(("texteditor", "close"))
    );
    assert_eq!(
        reading(&["texteditor", "read", "now"])
            .unwrap_err()
            .of_an_act(),
        None
    );
    assert_eq!(reading(&[]).unwrap_err().of_an_act(), None);
}

#[test]
fn a_look_is_not_an_act_and_everything_else_is() {
    assert!(!Doing::Controls.acts());
    assert!(!Doing::Read.acts());
    assert!(Doing::Window(OnTheWindow::Open).acts());
    assert!(
        Doing::Press {
            kind: String::new(),
            control: String::new()
        }
        .acts()
    );
}
