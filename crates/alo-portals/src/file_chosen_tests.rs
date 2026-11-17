use super::*;
use alo_strings::Vocabulary;

fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    crate::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// **Only the person's cancel is `1`**, and each reason says something
/// different, naming who asked.
#[test]
fn only_the_persons_cancel_is_one_and_each_says_why() {
    let strings = in_english();
    let mut seen = Vec::new();
    for why in [
        NoFile::Cancelled,
        NoFile::Unanswered,
        NoFile::CannotShow,
        NoFile::NoDesktop,
        NoFile::CouldNotAsk,
        NoFile::NotBelieved,
        NoFile::NotHandedOver(NotExported::NoDocumentsPortal),
        NoFile::NotHandedOver(NotExported::NotThere),
        NoFile::NotHandedOver(NotExported::Refused),
        NoFile::NotOfferedYet,
    ] {
        assert_eq!(why.response(), u32::from(why != NoFile::Cancelled) + 1);
        let said = FileUnchosen::of(Applicant::named("org.gnome.TextEditor"), why).said(&strings);
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(said.text().contains("org.gnome.TextEditor"), "{said}");
        seen.push(said.text().to_owned());
    }
    let how_many = seen.len();
    seen.sort();
    seen.dedup();
    assert_eq!(seen.len(), how_many, "two reasons said the same thing");
}

/// **A file chosen names who asked and the file**, to open and to save
/// differently.
#[test]
fn a_file_chosen_names_who_asked_and_the_file() {
    let strings = in_english();
    let opening = FileChosen::of(
        Applicant::named("org.gnome.TextEditor"),
        PathBuf::from("/home/anna/march.txt"),
        false,
    )
    .said(&strings);
    let saving = FileChosen::of(
        Applicant::named("org.gnome.TextEditor"),
        PathBuf::from("/home/anna/march.txt"),
        true,
    )
    .said(&strings);
    for said in [&opening, &saving] {
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("org.gnome.TextEditor"), "{said}");
        assert!(said.text().contains("/home/anna/march.txt"), "{said}");
    }
    assert_ne!(opening.text(), saving.text());
}

/// **The file keeps the reason by its spelling.**
#[test]
fn the_reason_is_kept_by_its_spelling() {
    assert_eq!(
        serde_json::to_string(&NoFile::NotBelieved).unwrap(),
        r#""not-believed""#
    );
    assert_eq!(
        serde_json::to_string(&NoFile::NotHandedOver(NotExported::NoDocumentsPortal)).unwrap(),
        r#"{"not-handed-over":"no-documents-portal"}"#
    );
}
