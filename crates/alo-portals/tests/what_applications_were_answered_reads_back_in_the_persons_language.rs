//! What applications were answered reads back in the person's language.
//!
//! Task 10 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`,
//! each clause of its acceptance a test here:
//!
//! - **a `KeptAnswer` is worded through `alo-strings` in the language the
//!   person reads: the application or *an application that could not be
//!   named*, the portal's sentence, and what it was answered with** —
//!   [`the_file::an_answer_is_said_as_who_the_portal_and_what_it_was_told`], and
//!   in another language than the source,
//!   [`the_file::an_answer_is_said_in_the_language_the_person_reads`];
//! - **reusing the words `Outcome::said` already uses rather than a second
//!   set** —
//!   [`the_file::an_answer_reads_back_in_the_words_it_was_answered_with`];
//! - **every `KeptOutcome` kind worded, and no key shown in place of a
//!   sentence** — [`the_file::every_kind_of_kept_outcome_is_a_sentence`];
//! - **`ReadBack` says whether the file is whole or shortened, and from when,
//!   the moment kept as a moment** —
//!   [`the_file::a_whole_file_and_a_shortened_one_say_so_and_from_when`];
//! - **lines that did not read are said as a count beside everything that did,
//!   never dropped** —
//!   [`the_file::lines_that_did_not_read_are_counted_beside_what_did`];
//! - **every new string is declared in `alo_portals::words` and collected by
//!   `alo-saying`** — [`every_string_read_back_is_collected`], and the
//!   contract, [`the_contract_says_how_the_file_is_read_to_a_person`].

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

use alo_strings::Key;

/// **Every string this change says is in the machine's one vocabulary** — the
/// sentences, the clauses and the count of lines that did not read.
#[test]
fn every_string_read_back_is_collected() {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    let read_back: Vec<_> = alo_portals::EVERY_WORD
        .iter()
        .filter(|word| word.named().starts_with("portals.read-back."))
        .collect();
    assert_eq!(read_back.len(), 8, "the sentences and clauses read back");
    for word in read_back {
        assert!(
            vocabulary.phrase(&word.key()).is_some(),
            "{} is not collected",
            word.named()
        );
    }
    for clause in alo_portals::CLAUSES {
        assert!(
            alo_portals::EVERY_WORD.contains(&clause),
            "{}",
            clause.named()
        );
    }
    let counted = Key::named(alo_portals::words::LINES_NOT_READ.named()).unwrap();
    assert!(
        vocabulary.plural(&counted).is_some(),
        "{counted} is not collected"
    );
}

/// **The contract says the file is read to a person through these two doors**,
/// and that the moment is not written into a sentence.
#[test]
fn the_contract_says_how_the_file_is_read_to_a_person() {
    let contract = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/contracts/portal-answers-file.md"),
    )
    .expect("docs/contracts/portal-answers-file.md is there");
    for named in [
        "## Reading it to a person",
        "`KeptAnswer::said`",
        "`ReadBack::said`",
        "an application that could not be named",
    ] {
        assert!(contract.contains(named), "{named}");
    }
}

#[cfg(unix)]
mod the_file {
    use std::collections::BTreeSet;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::{Duration, SystemTime};

    use alo_capability::{Applicant, Facility, Grant, Grants, Reach};
    use alo_keeping::Keeping;
    use alo_portals::{
        Answered, AnswersFile, KeptAnswer, KeptOutcome, NotARequest, NothingOpensAs, Outcome,
        Portal, ReadBack, Recording, RefusedAs, Request, Unanswered,
    };
    use alo_strings::{Key, Language, Said, Strings, Translation};

    const FRACTAL: &str = "org.gnome.Fractal";
    const STRANGER: &str = "org.example.Stranger";
    const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

    /// A folder of this test's own, empty.
    fn a_folder(what: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-portal-answers-said-{what}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// Everything this machine can say, in the source language.
    fn in_english() -> Strings {
        Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
    }

    /// **A sentence, and not a key in its place**: nothing marked as a bug, no
    /// gap left empty, and no key's name anywhere in the text.
    fn is_a_sentence(said: &Said) {
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}: {:?}", said.unfilled());
        for key_shaped in ["«", "portals.", "capability.", "applications.", "opening."] {
            assert!(!said.text().contains(key_shaped), "{said}");
        }
        assert!(!said.text().trim().is_empty());
    }

    /// An answers file at `at` holding its first line and then `lines`,
    /// exactly, as a file written by the backend and read later would.
    fn a_file_holding(at: &std::path::Path, lines: &[String]) {
        drop(AnswersFile::opened(at).unwrap());
        let mut file = std::fs::OpenOptions::new().append(true).open(at).unwrap();
        for line in lines {
            writeln!(file, "{line}").unwrap();
        }
    }

    /// One answer as the file writes it: `outcome` for `portal`, from
    /// `application` or nobody, `second` seconds after noon.
    fn a_line(
        second: u64,
        application: Option<&str>,
        portal: Portal,
        outcome: &KeptOutcome,
    ) -> String {
        let mut answer = serde_json::to_value(outcome).unwrap();
        let fields = answer.as_object_mut().unwrap();
        fields.insert(
            "at".to_owned(),
            serde_json::to_value(noon() + Duration::from_secs(second)).unwrap(),
        );
        if let Some(application) = application {
            fields.insert("application".to_owned(), application.into());
        }
        fields.insert("portal".to_owned(), serde_json::to_value(portal).unwrap());
        serde_json::to_string(&answer).unwrap()
    }

    /// Which kind of kept outcome this is, to the depth that picks a sentence.
    ///
    /// Exhaustive on purpose: a new way of keeping an answer does not compile
    /// until this test words it.
    fn kind_of(outcome: &KeptOutcome) -> String {
        match outcome {
            KeptOutcome::SecretHandedOver { .. } => "secret-handed-over".to_owned(),
            KeptOutcome::Opened { .. } => "opened".to_owned(),
            KeptOutcome::AppearanceRead { .. } => "appearance-read".to_owned(),
            KeptOutcome::AppearanceSent { .. } => "appearance-sent".to_owned(),
            KeptOutcome::NetworkRead { .. } => "network-read".to_owned(),
            KeptOutcome::Refused { why } => format!(
                "refused/{}",
                match why {
                    RefusedAs::NothingGranted => "nothing-granted",
                    RefusedAs::NeverGranted => "never-granted",
                    RefusedAs::Lapsed => "lapsed",
                }
            ),
            KeptOutcome::NotARequest { why } => format!(
                "not-a-request/{}",
                match why {
                    NotARequest::NoApplication => "no-application",
                    NotARequest::NotAnIdentifier => "not-an-identifier",
                    NotARequest::NeedsAPath => "needs-a-path",
                    NotARequest::NotOverAPath => "not-over-a-path",
                    NotARequest::NotAFullPath => "not-a-full-path",
                    NotARequest::CouldLeadElsewhere => "could-lead-elsewhere",
                }
            ),
            KeptOutcome::NothingOpens { why } => format!(
                "nothing-opens/{}",
                match why {
                    NothingOpensAs::NoApplication { chosen: None } => "no-application",
                    NothingOpensAs::NoApplication { chosen: Some(_) } => "chosen-not-installed",
                    NothingOpensAs::TheFile => "the-file",
                    NothingOpensAs::Unreadable => "unreadable",
                }
            ),
            KeptOutcome::Unanswered { why } => format!(
                "unanswered/{}",
                match why {
                    Unanswered::NotIdentified => "not-identified",
                    Unanswered::NotAToken => "not-a-token",
                    Unanswered::GrantsUnread => "grants-unread",
                    Unanswered::ApplicationsUnread => "applications-unread",
                    Unanswered::KeyringUnavailable => "keyring-unavailable",
                    Unanswered::NotWritten => "not-written",
                    Unanswered::NotAFile => "not-a-file",
                    Unanswered::NotOpened { .. } => "not-opened",
                    Unanswered::NotDecidedHere => "not-decided-here",
                    Unanswered::NoSuchSetting => "no-such-setting",
                    Unanswered::AppearanceUnread => "appearance-unread",
                    Unanswered::NetworkUnread => "network-unread",
                }
            ),
        }
    }

    /// One of every kept outcome, to the depth that picks a sentence.
    fn every_kept_outcome() -> Vec<KeptOutcome> {
        let mut every = vec![
            KeptOutcome::SecretHandedOver {
                against: Vec::new(),
            },
            KeptOutcome::Opened {
                opener: "org.gnome.Papers".to_owned(),
                chosen: true,
                against: Vec::new(),
            },
            KeptOutcome::AppearanceRead {
                against: Vec::new(),
            },
            KeptOutcome::AppearanceSent {
                against: Vec::new(),
            },
            KeptOutcome::NetworkRead {
                against: Vec::new(),
            },
        ];
        for why in [
            RefusedAs::NothingGranted,
            RefusedAs::NeverGranted,
            RefusedAs::Lapsed,
        ] {
            every.push(KeptOutcome::Refused { why });
        }
        for why in [
            NotARequest::NoApplication,
            NotARequest::NotAnIdentifier,
            NotARequest::NeedsAPath,
            NotARequest::NotOverAPath,
            NotARequest::NotAFullPath,
            NotARequest::CouldLeadElsewhere,
        ] {
            every.push(KeptOutcome::NotARequest { why });
        }
        for why in [
            NothingOpensAs::NoApplication { chosen: None },
            NothingOpensAs::NoApplication {
                chosen: Some("org.kde.okular".to_owned()),
            },
            NothingOpensAs::TheFile,
            NothingOpensAs::Unreadable,
        ] {
            every.push(KeptOutcome::NothingOpens { why });
        }
        for why in [
            Unanswered::NotIdentified,
            Unanswered::NotAToken,
            Unanswered::GrantsUnread,
            Unanswered::ApplicationsUnread,
            Unanswered::KeyringUnavailable,
            Unanswered::NotWritten,
            Unanswered::NotAFile,
            Unanswered::NotOpened {
                opener: "org.gnome.Papers".to_owned(),
            },
            Unanswered::NotDecidedHere,
            Unanswered::NoSuchSetting,
            Unanswered::AppearanceUnread,
            Unanswered::NetworkUnread,
        ] {
            every.push(KeptOutcome::Unanswered { why });
        }
        every
    }

    /// **Every kind of kept outcome is a sentence** — from a named application
    /// and from nobody named, to a portal over a facility and to one over a
    /// file — read off a real answers file, and no key is ever shown in place
    /// of one.
    #[test]
    fn every_kind_of_kept_outcome_is_a_sentence() {
        let folder = a_folder("every-kind");
        let at = folder.join("portal-answers.jsonl");
        let outcomes = every_kept_outcome();
        let kinds: BTreeSet<String> = outcomes.iter().map(kind_of).collect();
        assert_eq!(kinds.len(), outcomes.len(), "each kind once");
        assert_eq!(kinds.len(), 30);

        let mut lines = Vec::new();
        let mut second = 0;
        for outcome in &outcomes {
            for application in [Some(FRACTAL), None] {
                for portal in [Portal::Camera, Portal::OpenWith] {
                    lines.push(a_line(second, application, portal, outcome));
                    second += 1;
                }
            }
        }
        a_file_holding(&at, &lines);

        let read = AnswersFile::read_back(&at).unwrap();
        assert!(read.unreadable.is_empty(), "{:?}", read.unreadable);
        assert_eq!(read.answers.len(), lines.len());
        let strings = in_english();
        let said = read.said(&strings);
        assert_eq!(said.answers().len(), lines.len(), "none dropped");
        for (kept, said) in read.answers.iter().zip(said.answers()) {
            for sentence in said.sentences() {
                is_a_sentence(sentence);
            }
            assert_eq!(said.at(), kept.at());
            assert_eq!(said.portal(), &kept.portal().said(&strings));
            match kept.application() {
                Some(application) => {
                    assert!(said.who().text().contains(application), "{}", said.who());
                }
                None => assert!(
                    said.who()
                        .text()
                        .contains("an application that could not be named"),
                    "{}",
                    said.who()
                ),
            }
        }
        is_a_sentence(said.reaches());
        std::fs::remove_dir_all(folder).unwrap();
    }

    /// **Who, the portal, and what it was told**, for a named application and
    /// for a caller nobody could name.
    #[test]
    fn an_answer_is_said_as_who_the_portal_and_what_it_was_told() {
        let strings = in_english();
        let named = KeptAnswer::from(&Answered::new(
            noon(),
            Some(Applicant::named(STRANGER)),
            Portal::Secret,
            Outcome::Refused(
                Request::of(STRANGER, Portal::Secret)
                    .unwrap()
                    .judged(&Grants::default(), noon())
                    .unwrap_err(),
            ),
        ))
        .said(&strings);
        assert_eq!(named.who().text(), format!("Asked by {STRANGER}"));
        assert_eq!(named.portal(), &Portal::Secret.said(&strings));
        assert!(
            named.answer().text().contains(STRANGER),
            "{}",
            named.answer()
        );
        assert!(
            named.answer().text().contains("without asking you"),
            "{}",
            named.answer()
        );
        assert_eq!(named.at(), noon());

        let nobody = KeptAnswer::from(&Answered::new(
            noon(),
            None,
            Portal::OpenWith,
            Outcome::Unanswered(Unanswered::NotIdentified),
        ))
        .said(&strings);
        assert_eq!(
            nobody.who().text(),
            "Asked by an application that could not be named"
        );
        assert_eq!(
            nobody.answer(),
            &Unanswered::NotIdentified.said(&strings),
            "said as the backend said it"
        );
    }

    /// **The same words the backend answered with**: for every answer whose
    /// decision the file keeps whole, the sentence read back is the sentence
    /// `Outcome::said` gave while the backend held the decision — not a
    /// second account of it.
    #[test]
    fn an_answer_reads_back_in_the_words_it_was_answered_with() {
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked_for(
                &Applicant::named(FRACTAL).grantee(),
                Reach::Facility(Facility::Secrets),
                noon(),
                Duration::from_secs(60),
            )
            .unwrap(),
        );
        grants.grant(
            Grant::checked_for(
                &Applicant::named(FRACTAL).grantee(),
                Reach::Facility(Facility::Camera),
                noon(),
                Duration::from_secs(60 * 60),
            )
            .unwrap(),
        );
        let secret = Request::of(FRACTAL, Portal::Secret).unwrap();
        let allowed = secret.judged(&grants, noon()).unwrap();
        let refused = |application: &str, portal: Portal, at: SystemTime| {
            Outcome::Refused(
                Request::of(application, portal)
                    .unwrap()
                    .judged(&grants, at)
                    .unwrap_err(),
            )
        };
        let answers = vec![
            (
                Some(FRACTAL),
                Portal::Secret,
                Outcome::SecretHandedOver(allowed.clone()),
            ),
            (
                Some(FRACTAL),
                Portal::Settings,
                Outcome::AppearanceRead(allowed.clone()),
            ),
            (
                Some(FRACTAL),
                Portal::Settings,
                Outcome::AppearanceSent(allowed),
            ),
            (
                Some(STRANGER),
                Portal::Secret,
                refused(STRANGER, Portal::Secret, noon()),
            ),
            (
                Some(FRACTAL),
                Portal::Microphone,
                refused(FRACTAL, Portal::Microphone, noon()),
            ),
            (
                Some(FRACTAL),
                Portal::Secret,
                refused(FRACTAL, Portal::Secret, noon() + Duration::from_secs(120)),
            ),
            (
                Some(FRACTAL),
                Portal::OpenWith,
                Outcome::NotARequest(NotARequest::CouldLeadElsewhere),
            ),
            (
                None,
                Portal::OpenWith,
                Outcome::Unanswered(Unanswered::NotIdentified),
            ),
            (
                Some(FRACTAL),
                Portal::OpenWith,
                Outcome::Unanswered(Unanswered::NotOpened {
                    opener: "org.gnome.Papers".to_owned(),
                }),
            ),
        ];
        let refusals: BTreeSet<String> = answers
            .iter()
            .map(|(_, _, outcome)| kind_of(&KeptOutcome::from(outcome)))
            .filter(|kind| kind.starts_with("refused/"))
            .collect();
        assert_eq!(
            refusals.len(),
            3,
            "each refusal of the grants: {refusals:?}"
        );

        let folder = a_folder("same-words");
        let at = folder.join("portal-answers.jsonl");
        let record = AnswersFile::opened(&at).unwrap();
        let answered: Vec<Answered> = answers
            .into_iter()
            .map(|(application, portal, outcome)| {
                Answered::new(noon(), application.map(Applicant::named), portal, outcome)
            })
            .collect();
        for answer in &answered {
            record.keep(answer.clone()).unwrap();
        }
        drop(record);

        let strings = in_english();
        let said = AnswersFile::read_back(&at).unwrap().said(&strings);
        assert_eq!(said.answers().len(), answered.len());
        for (answer, said) in answered.iter().zip(said.answers()) {
            assert_eq!(
                said.answer(),
                &answer.outcome().said(&strings),
                "{:?}",
                answer.outcome()
            );
            is_a_sentence(said.answer());
        }
        std::fs::remove_dir_all(folder).unwrap();
    }

    /// **In the language the person reads**: with a translation preferred, who
    /// asked is said in it — the application's identifier untouched, and the
    /// words for nobody named translated — and says it was translated.
    #[test]
    fn an_answer_is_said_in_the_language_the_person_reads() {
        let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
        let german = Language::written("de").unwrap();
        let checked = vocabulary
            .check(
                Translation::into_language(german.clone())
                    .says(
                        Key::named("portals.read-back.asked-by").unwrap(),
                        "Gefragt von {application}",
                    )
                    .says(
                        Key::named("portals.read-back.nobody-named").unwrap(),
                        "einer Anwendung, die nicht benannt werden konnte",
                    ),
            )
            .map_err(|wrongs| wrongs.to_string())
            .unwrap();
        let mut strings = Strings::of(vocabulary);
        strings.speaks(checked).unwrap();
        strings.prefers(&[german]);

        let named = KeptAnswer::from(&Answered::new(
            noon(),
            Some(Applicant::named(FRACTAL)),
            Portal::Secret,
            Outcome::Unanswered(Unanswered::KeyringUnavailable),
        ))
        .said(&strings);
        assert_eq!(named.who().text(), format!("Gefragt von {FRACTAL}"));
        assert!(named.who().is_translated());

        let nobody = KeptAnswer::from(&Answered::new(
            noon(),
            None,
            Portal::Secret,
            Outcome::Unanswered(Unanswered::NotIdentified),
        ))
        .said(&strings);
        assert_eq!(
            nobody.who().text(),
            "Gefragt von einer Anwendung, die nicht benannt werden konnte"
        );
        assert!(nobody.who().is_translated());
    }

    /// **Whole says so; shortened says it does not go all the way back, and
    /// from when** — the moment handed back as a moment, never written into
    /// any sentence, and a file shortened twice still says it was.
    #[test]
    fn a_whole_file_and_a_shortened_one_say_so_and_from_when() {
        let folder = a_folder("whole-or-shortened");
        let at = folder.join("portal-answers.jsonl");
        let lines: Vec<String> = [0, 10, 20]
            .into_iter()
            .map(|day| {
                a_line(
                    day * A_DAY.as_secs(),
                    Some(FRACTAL),
                    Portal::Secret,
                    &KeptOutcome::Unanswered {
                        why: Unanswered::KeyringUnavailable,
                    },
                )
            })
            .collect();
        a_file_holding(&at, &lines);
        let strings = in_english();

        let whole = AnswersFile::read_back(&at).unwrap();
        assert!(whole.is_whole());
        let said = whole.said(&strings);
        is_a_sentence(said.reaches());
        assert!(
            said.reaches()
                .text()
                .starts_with("Nothing has been removed"),
            "{}",
            said.reaches()
        );
        assert_eq!(said.since(), None);
        assert_eq!(said.under(), None);
        assert_eq!(said.answers().len(), 3);

        let week = Keeping::for_days(7).unwrap();
        let record = AnswersFile::opened(&at).unwrap();
        record.shortened(week, noon() + 15 * A_DAY).unwrap();
        drop(record);
        let shortened = AnswersFile::read_back(&at).unwrap();
        assert!(!shortened.is_whole());
        let said = shortened.said(&strings);
        is_a_sentence(said.reaches());
        assert!(
            said.reaches()
                .text()
                .contains("does not go all the way back"),
            "{}",
            said.reaches()
        );
        assert_eq!(said.since(), shortened.since);
        assert_eq!(said.since(), Some(noon() + 8 * A_DAY));
        let under = said.under().expect("the rule it was shortened under");
        is_a_sentence(under);
        assert_eq!(under, &week.said(&strings));
        assert_eq!(
            said.answers().len(),
            2,
            "the answer older than the week is gone, and the two inside it are not"
        );
        assert!(
            !said
                .reaches()
                .text()
                .chars()
                .any(|char| char.is_ascii_digit()),
            "no moment is written into {}",
            said.reaches()
        );

        // Shortened again under a rule reaching back further, it still says it
        // was, and from the later moment.
        let record = AnswersFile::opened(&at).unwrap();
        record
            .shortened(Keeping::for_days(1).unwrap(), noon() + 30 * A_DAY)
            .unwrap();
        record
            .shortened(Keeping::Forever, noon() + 31 * A_DAY)
            .unwrap();
        drop(record);
        let twice = AnswersFile::read_back(&at).unwrap().said(&strings);
        assert!(
            twice
                .reaches()
                .text()
                .contains("does not go all the way back"),
            "{}",
            twice.reaches()
        );
        assert_eq!(twice.since(), Some(noon() + 29 * A_DAY));
        assert!(twice.answers().is_empty());
        std::fs::remove_dir_all(folder).unwrap();
    }

    /// **A line that did not read is counted beside what did** — never
    /// dropped silently and never in place of the answers — in the reader's
    /// plural form, including a line naming something that is not an
    /// application and a torn last line.
    #[test]
    fn lines_that_did_not_read_are_counted_beside_what_did() {
        let folder = a_folder("not-read");
        let at = folder.join("portal-answers.jsonl");
        let strings = in_english();
        let good = |second| {
            a_line(
                second,
                Some(FRACTAL),
                Portal::Camera,
                &KeptOutcome::Refused {
                    why: RefusedAs::NeverGranted,
                },
            )
        };
        let not_an_application = a_line(
            5,
            Some("org.gnome Fractal"),
            Portal::Camera,
            &KeptOutcome::Refused {
                why: RefusedAs::NeverGranted,
            },
        );

        a_file_holding(&at, &[good(1), not_an_application.clone(), good(2)]);
        let read: ReadBack = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.unreadable, [3]);
        let said = read.said(&strings);
        assert_eq!(said.answers().len(), 2, "what did read is all there");
        let one = said.not_read().expect("the line that did not read is said");
        is_a_sentence(one);
        assert!(one.text().starts_with("One line"), "{one}");

        std::fs::remove_file(&at).unwrap();
        a_file_holding(&at, &[good(1), not_an_application, good(2)]);
        let mut file = std::fs::OpenOptions::new().append(true).open(&at).unwrap();
        write!(file, "{{\"at\":{{\"secs_since_epo").unwrap();
        drop(file);
        let read = AnswersFile::read_back(&at).unwrap();
        assert_eq!(read.unreadable, [3, 5]);
        let said = read.said(&strings);
        assert_eq!(said.answers().len(), 2);
        let two = said
            .not_read()
            .expect("both lines that did not read are said");
        is_a_sentence(two);
        assert!(two.text().starts_with("2 lines"), "{two}");

        std::fs::remove_file(&at).unwrap();
        a_file_holding(&at, &[good(1)]);
        let said = AnswersFile::read_back(&at).unwrap().said(&strings);
        assert_eq!(said.not_read(), None, "every line read");
        assert_eq!(said.answers().len(), 1);
        std::fs::remove_dir_all(folder).unwrap();
    }
}
