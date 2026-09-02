//! What the capability model says, in a language somebody actually reads.
//!
//! The unit tests beside each file ask whether one refusal says the right
//! thing. This one asks the question the crate exists to answer from outside:
//! **can a person be told no, in their own language, by every part of this
//! model, and does anything say so when nobody has translated it?**
//!
//! German, because these are sentences rather than labels and German moves the
//! verb — a translation that read like English with the words swapped would not
//! be exercising anything. The German here is the test's, not a translation
//! this repository ships: there are still zero of those.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_capability::{
    Arg, ArgError, Ask, Authorised, Call, Effect, Given, Grant, GrantError, Grantee, Grants,
    NotGranted, Proposal, Reach, Requires, Takes, Verb, capability_words, declare_into, words,
};
use alo_strings::{Filling, Key, Language, Strings, Translation, Vocabulary};

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants in these tests last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// German, as `alo-strings` names a language.
fn german() -> Language {
    Language::written("de").unwrap()
}

/// The same, with these said in German and German preferred.
fn speaking_german(said: &[(alo_strings::Word, &str)]) -> Strings {
    let vocabulary = capability_words().unwrap();
    let mut translation = Translation::into_language(german());
    for (word, says) in said {
        translation = translation.says(word.key(), *says);
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german()]);
    strings
}

/// The one verb these tests are about.
fn move_file() -> Verb {
    Verb::checked(
        "move_file",
        "move a file into a folder",
        Effect::Change,
        vec![
            Arg::taking("file", "the file to move", Takes::Path),
            Arg::taking("into", "the folder it goes into", Takes::Path),
        ],
        Requires::grants_over(["file", "into"]),
        "move {file} into {into}",
    )
    .unwrap()
}

/// Moving one file into a folder, as a validated call.
fn moving() -> Call {
    Call::of(
        &move_file(),
        &[
            ("file", Given::text("/home/anna/Invoices/march.pdf")),
            ("into", Given::text("/home/anna/Archive")),
        ],
    )
    .unwrap()
}

/// One agent's grant over one folder.
fn granting(folder: &str) -> Grants {
    let mut grants = Grants::default();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from(folder)),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants
}

/// **Every string this crate can say declares**, and it declares into a
/// vocabulary that already holds somebody else's.
///
/// That is the arrangement on a real machine: one vocabulary, one area per
/// crate, every crate adding its own.
#[test]
fn everything_this_crate_says_declares_beside_everybody_elses() {
    let mut vocabulary = Vocabulary::empty();
    vocabulary
        .says(
            alo_strings::Phrase::says(Key::named("files.example").unwrap(), "something else")
                .unwrap(),
        )
        .unwrap();
    declare_into(&mut vocabulary).unwrap();

    assert_eq!(vocabulary.how_many(), words::EVERY_WORD.len() + 2);
    for word in words::EVERY_WORD {
        assert!(vocabulary.phrase(&word.key()).is_some(), "{}", word.named());
    }
    assert!(vocabulary.plural(&words::TOO_LONG.key()).is_some());
}

/// **The whole model can say no in German**: the grant a person could not make,
/// the argument an agent sent, the grants themselves, and the change that was
/// never put to anybody.
#[test]
fn a_person_is_told_no_in_the_language_they_read() {
    let strings = speaking_german(&[
        (
            words::THE_WHOLE_MACHINE,
            "es gibt keine Berechtigung für den ganzen Rechner — wählen Sie den Ordner, den Sie \
             wirklich meinen",
        ),
        (
            words::ARGUMENT_NOT_A_FULL_PATH,
            "geben Sie {argument} als vollständigen Pfad an, damit er überall dasselbe bedeutet",
        ),
        (
            words::NEVER_GRANTED,
            "{agent} wurde {wanted} nicht gewährt — Berechtigungen entstehen dadurch, dass jemand \
             einen Ordner auswählt, niemals dadurch, dass danach gefragt wird",
        ),
        (
            words::READ_DOES_NOT_WAIT,
            "{verb} beantwortet eine Frage, statt etwas zu ändern — führen Sie es sofort aus, \
             statt danach zu fragen",
        ),
    ]);

    let grant =
        Grant::checked("@files", Reach::Folder(PathBuf::from("/")), noon(), hour()).unwrap_err();
    assert_eq!(grant, GrantError::TheWholeMachine);
    assert!(grant.said(&strings).text().starts_with("es gibt keine"));

    let argument = Arg::taking("folder", "the folder to list", Takes::Path)
        .validate(&Given::text("Invoices"))
        .unwrap_err();
    assert_eq!(
        argument,
        ArgError::NotAFullPath {
            argument: "folder".to_owned()
        }
    );
    assert!(
        argument
            .said(&strings)
            .text()
            .contains("vollständigen Pfad")
    );

    // Half a move: the file is granted and the folder it would go to is not.
    let refused = moving()
        .refusal(
            &granting("/home/anna/Invoices"),
            &Grantee::named("@files"),
            noon(),
        )
        .unwrap();
    let said = refused.said(&strings);
    assert!(said.is_translated());
    assert!(said.text().contains("nicht gewährt"), "{said}");
    // The path and the agent are the machine's, and are not translated.
    assert!(said.text().contains("/home/anna/Archive"), "{said}");
    assert!(said.text().contains("@files"), "{said}");

    let read = Call::of(
        &Verb::checked(
            "list_folder",
            "list what is in a folder",
            Effect::Read,
            vec![Arg::taking("folder", "the folder to list", Takes::Path)],
            Requires::grants_over(["folder"]),
            "list what is in {folder}",
        )
        .unwrap(),
        &[("folder", Given::text("/home/anna/Invoices"))],
    )
    .unwrap();
    let proposed = Proposal::checked(
        &read,
        &Grantee::named("@files"),
        &granting("/home/anna/Invoices"),
        noon(),
        hour(),
    )
    .unwrap_err();
    assert!(
        proposed
            .said(&strings)
            .text()
            .starts_with("list_folder beantwortet")
    );
}

/// **What nobody has translated is English, and says so.** Half a vocabulary is
/// the ordinary state of a language somebody is still working on, and a machine
/// that could not tell the difference would be one where *shown English because
/// nobody translated it* is invisible.
#[test]
fn what_nobody_translated_is_english_and_says_so() {
    let strings = speaking_german(&[(
        words::THE_WHOLE_MACHINE,
        "es gibt keine Berechtigung für den ganzen Rechner",
    )]);

    let translated = GrantError::TheWholeMachine.said(&strings);
    assert!(translated.is_translated());

    let untranslated = GrantError::NoTime.said(&strings);
    assert!(!untranslated.is_translated());
    assert!(!untranslated.is_a_bug());
    assert_eq!(
        untranslated.text(),
        "say how long the grant should last — a grant for no time reaches nothing"
    );

    // And what is left to do is countable, which is what a release note needs.
    // The countable string counts as the forms *German* needs — two of them —
    // rather than as one row, because a language with three would not be two
    // thirds done and the number a translator is given has to say so.
    let left = strings.missing_from(&german());
    assert_eq!(left.len(), words::EVERY_WORD.len() - 1 + 2);
    assert!(left.contains(&words::GRANT_NO_TIME.key()));
    assert!(!left.contains(&words::THE_WHOLE_MACHINE.key()));
    assert!(left.contains(&words::TOO_LONG.key().for_form(alo_strings::Form::One)));
}

/// **A refusal never depends on a string table.** With no words at all, every
/// refusal here refuses exactly what it refused before and answers with the key
/// of the sentence, marked as a bug — so a shell that forgot to declare this
/// crate's words finds out, and nothing is quietly permitted.
#[test]
fn a_refusal_without_the_words_still_refuses_and_says_the_rule() {
    let nothing = Strings::of(Vocabulary::empty());

    let refused = Authorised::read(
        &moving(),
        &Grantee::named("@files"),
        &Grants::default(),
        noon(),
    )
    .unwrap_err();
    let said = refused.said(&nothing);
    assert!(said.is_a_bug());
    assert!(
        said.text().contains("capability.refused.change-waits"),
        "{said}"
    );

    let never = NotGranted::Never {
        agent: "@files".to_owned(),
        wanted: Ask::path("/etc/shadow"),
    };
    assert!(never.said(&nothing).is_a_bug());
}

/// **A translation that dropped a gap out of a refusal is refused.**
///
/// A refusal that lost `{wanted}` would tell somebody in their own language
/// that something was not granted, without saying what — which is worse than
/// the English, because nothing anywhere would say it had happened.
#[test]
fn a_translation_that_drops_what_was_refused_is_refused() {
    let vocabulary = capability_words().unwrap();
    let wrongs = vocabulary
        .check(
            Translation::into_language(german())
                .says(words::NEVER_GRANTED.key(), "{agent} darf das nicht")
                .says(
                    words::OUT_OF_RANGE.key(),
                    "geben Sie {argument} als Zahl zwischen {least} und {most} an",
                ),
        )
        .unwrap_err();
    assert_eq!(wrongs.how_many(), 1);
    assert!(wrongs.to_string().contains("wanted"), "{wrongs}");
}

/// The one gap that arrives from somewhere else arrives filled, and the
/// sentence around it is still the reader's.
///
/// `{sentence}` in *that was proposed too long ago* is the approval sentence,
/// which `alo_capability::Call` renders when the call is made — in the source
/// language, until item 9f moves it. This test is what that owes: the words
/// around it move now, and the quotation marks are the translator's.
#[test]
fn the_sentence_a_question_quotes_is_the_one_that_was_proposed() {
    let strings = speaking_german(&[(
        words::LAPSED,
        "„{sentence}“ wurde vor zu langer Zeit vorgeschlagen — fragen Sie erneut, wenn es noch \
         gewünscht ist",
    )]);
    let said = strings.say(
        &words::LAPSED.key(),
        &Filling::of("sentence", moving().sentence().to_owned()),
    );
    assert!(said.is_translated());
    assert!(said.text().starts_with('„'), "{said}");
    assert!(
        said.text()
            .contains("move /home/anna/Invoices/march.pdf into /home/anna/Archive"),
        "{said}"
    );
}
