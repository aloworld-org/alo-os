use super::*;

fn called(identifier: &str) -> Application {
    Application::identified(identifier).unwrap()
}

/// **What is written is what is read**, a space in the folder and in the name
/// written so the line keeps its words.
#[test]
fn a_line_written_reads_back_as_itself() {
    let asked =
        ToAnswerAFileChooser::of(PathBuf::from("/var/home/alo/My Documents"), "notes for March.txt")
            .unwrap();
    assert_eq!(
        asked.written(),
        "alo-chooses-file /var/home/alo/My%20Documents notes%20for%20March.txt"
    );
    assert_eq!(ToAnswerAFileChooser::read(&asked.written()), Some(Ok(asked.clone())));
    assert_eq!(asked.folder(), Path::new("/var/home/alo/My Documents"));
    assert_eq!(asked.name(), "notes for March.txt");
    assert_eq!(
        asked.file(),
        PathBuf::from("/var/home/alo/My Documents/notes for March.txt")
    );
}

/// **Another act is not this one**, and a line that began as this one and is
/// not exactly it is refused.
#[test]
fn what_is_not_this_act_is_refused() {
    for line in ["choose-file org.gnome.TextEditor open /home", "front", "alo-chooses"] {
        assert_eq!(ToAnswerAFileChooser::read(line), None, "{line}");
    }
    for line in [
        "alo-chooses-file",
        "alo-chooses-file /home",
        "alo-chooses-file home a.txt",
        "alo-chooses-file /home/../etc passwd",
        "alo-chooses-file /home a%2Fb",
        "alo-chooses-file /home ..",
        "alo-chooses-file /home a.txt more",
        "alo-chooses-file /home a%0A",
    ] {
        assert!(
            matches!(ToAnswerAFileChooser::read(line), Some(Err(_))),
            "{line}"
        );
    }
    assert!(ToAnswerAFileChooser::of(PathBuf::from("relative"), "a.txt").is_err());
    assert!(ToAnswerAFileChooser::of(PathBuf::from("/home"), "a/b").is_err());
}

/// **Every answer reads back, and an unknown word is no answer.**
#[test]
fn an_answer_is_one_of_three_or_nothing() {
    for answered in [
        AnsweredTheChooser::Chosen(called("org.gnome.TextEditor")),
        AnsweredTheChooser::NoSheet,
        AnsweredTheChooser::NotBelieved,
    ] {
        assert_eq!(AnsweredTheChooser::read(&answered.written()), Some(answered.clone()));
        assert_eq!(
            AnsweredTheChooser::read(&format!("{}\n", answered.written())),
            Some(answered)
        );
    }
    for heard in ["chosen", "chosen /usr/bin/x", "opened", "", "no-sheet please"] {
        assert_eq!(AnsweredTheChooser::read(heard), None, "{heard:?}");
    }
}
