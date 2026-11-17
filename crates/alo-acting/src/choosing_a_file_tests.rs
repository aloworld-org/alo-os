use super::*;

fn called(identifier: &str) -> Application {
    Application::identified(identifier).unwrap()
}

fn to_save(name: &str) -> ToChooseAFile {
    ToChooseAFile::of(
        called("org.gnome.TextEditor"),
        ToChoose::Save {
            name: name.to_owned(),
        },
        PathBuf::from("/home/anna/My Documents"),
    )
    .unwrap()
}

/// **What is written is what is read**, a space in a folder and a name
/// written so the line still has its words.
#[test]
fn a_line_written_reads_back_as_itself() {
    let opening = ToChooseAFile::of(
        called("org.gnome.TextEditor"),
        ToChoose::Open,
        PathBuf::from("/home/anna/Documents"),
    )
    .unwrap();
    assert_eq!(
        opening.written(),
        "choose-file org.gnome.TextEditor open /home/anna/Documents"
    );
    assert_eq!(
        ToChooseAFile::read(&opening.written()),
        Some(Ok(opening.clone()))
    );
    let saving = to_save("notes for März.txt");
    assert_eq!(
        saving.written(),
        "choose-file org.gnome.TextEditor save /home/anna/My%20Documents \
         notes%20for%20M%C3%A4rz.txt"
    );
    assert_eq!(ToChooseAFile::read(&saving.written()), Some(Ok(saving)));
}

/// **Another act is not this one.**
#[test]
fn another_act_is_not_this_one() {
    for line in [
        "choose-link org.gnome.Papers https://a.example/ org.mozilla.firefox",
        "open-file org.gnome.TextEditor /home/anna/a.txt",
        "front",
        "choose-files org.gnome.TextEditor open /home",
    ] {
        assert_eq!(ToChooseAFile::read(line), None, "{line}");
    }
}

/// **Anything else after the act is refused**, and never read as a shorter
/// request.
#[test]
fn what_is_not_this_act_is_refused() {
    use NotAFileToChoose as Not;
    for (line, refused) in [
        ("choose-file", Not::NotEnoughWords),
        ("choose-file org.gnome.TextEditor open", Not::NotEnoughWords),
        (
            "choose-file org.gnome.TextEditor save /home/anna",
            Not::NotEnoughWords,
        ),
        (
            "choose-file org.gnome.TextEditor open /home/anna a.txt",
            Not::NotEnoughWords,
        ),
        (
            "choose-file org.gnome.TextEditor save /home/anna a.txt more",
            Not::NotEnoughWords,
        ),
        (
            "choose-file org.gnome.TextEditor print /home/anna",
            Not::NeitherOpenNorSave,
        ),
        (
            "choose-file /usr/bin/x open /home/anna",
            Not::NotAnApplication,
        ),
        (
            "choose-file org.gnome.TextEditor open home/anna",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor open /home/../etc",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor open /home/./anna",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor open /home//anna",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor open /home/a%0Ab",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor open /home/a%2",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor open /home/a%FF",
            Not::NotAFolder,
        ),
        (
            "choose-file org.gnome.TextEditor save /home/anna a%2Fb",
            Not::NotAName,
        ),
        (
            "choose-file org.gnome.TextEditor save /home/anna ..",
            Not::NotAName,
        ),
        (
            "choose-file org.gnome.TextEditor save /home/anna a/b",
            Not::NotAName,
        ),
        (
            "choose-file org.gnome.TextEditor save /home/anna a%09b",
            Not::NotAName,
        ),
    ] {
        assert_eq!(ToChooseAFile::read(line), Some(Err(refused)), "{line}");
    }
    let long = format!(
        "choose-file org.gnome.TextEditor open /{}",
        "a".repeat(THE_MOST)
    );
    assert_eq!(ToChooseAFile::read(&long), Some(Err(Not::TooLong)));
    let long_name = format!(
        "choose-file org.gnome.TextEditor save /home {}",
        "a".repeat(THE_LONGEST_NAME + 1)
    );
    assert_eq!(ToChooseAFile::read(&long_name), Some(Err(Not::NotAName)));
}

/// **A request that could not be read back is never made.**
#[test]
fn a_request_that_could_not_be_read_is_not_made() {
    for (folder, to) in [
        ("relative/folder", ToChoose::Open),
        ("/home/", ToChoose::Open),
        ("/home/a\nb", ToChoose::Open),
        (
            "/home",
            ToChoose::Save {
                name: String::new(),
            },
        ),
        (
            "/home",
            ToChoose::Save {
                name: "a/b".to_owned(),
            },
        ),
    ] {
        assert!(
            ToChooseAFile::of(called("org.gnome.TextEditor"), to, PathBuf::from(folder)).is_err(),
            "{folder}"
        );
    }
    assert!(
        ToChooseAFile::of(
            called("org.gnome.TextEditor"),
            ToChoose::Open,
            PathBuf::from("/")
        )
        .is_ok(),
        "the root is a folder"
    );
}

/// **Every answer reads back, and `chosen` of anything but a whole path to a
/// file, or an unknown word, is no answer at all.**
#[test]
fn an_answer_is_a_whole_path_or_nothing() {
    for chose in [
        ChoseAFile::Chosen(PathBuf::from("/home/anna/My Documents/März.txt")),
        ChoseAFile::Cancelled,
        ChoseAFile::Unanswered,
        ChoseAFile::CannotShow,
    ] {
        assert_eq!(ChoseAFile::read(&chose.written()), Some(chose.clone()));
        assert_eq!(
            ChoseAFile::read(&format!("{}\n", chose.written())),
            Some(chose)
        );
    }
    assert_eq!(
        ChoseAFile::Chosen(PathBuf::from("/home/anna/a b.txt")).written(),
        "chosen /home/anna/a%20b.txt"
    );
    for heard in [
        "chosen",
        "chosen ",
        "chosen relative.txt",
        "chosen /",
        "chosen /home/../etc/shadow",
        "chosen /home/anna/a b.txt",
        "chosen /home/a%0A",
        "opened",
        "",
    ] {
        assert_eq!(ChoseAFile::read(heard), None, "{heard:?}");
    }
}
