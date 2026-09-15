//! An open-with request is answered from what opens what, and nowhere else.
//!
//! Task 4 of `docs/autonomy/v0-5-applications-and-what-they-expect-plan.md`:
//! *an open-with portal request (task 1) is answered from this and nowhere
//! else* — and its constraints, that nothing here launches anything and no
//! association is set by an application on its own behalf. The rest of the
//! task's acceptance is `alo-applications`' `tests/what_opens_what.rs`.
//!
//! Each refusal is tested beside the answer: a request granted nothing, a
//! request for a file it was not granted — refused **before the file is read**
//! — an opener the asking application was not granted, and a file nothing
//! opens.

#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::fs;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use alo_applications::{
    Application, Because, Chosen, Declared, Installed, NothingOpens, WhatOpensWhat,
};
use alo_capability::{Applicant, Ask, Grant, Grants, NotAllowed, Reach};
use alo_opening::{Cannot, Kind};
use alo_portals::open_with::answered;
use alo_portals::{NotARequest, NotOpened, Portal, Refused};
use alo_strings::Strings;

/// A PDF, as its first and last bytes make it one.
const A_PDF: &[u8] = b"%PDF-1.7\n1 0 obj << >> endobj\n%%EOF\n";

/// The start of a program.
const A_PROGRAM: &[u8] = b"\x7fELF\x02\x01\x01\0\0\0\0\0\0\0\0\0";

/// The application asking to have a file opened.
const MAIL: &str = "org.gnome.Geary";

/// Where the file it asks about is, and the folder it was granted.
const THE_INVOICE: &str = "/home/anna/Invoices/march.pdf";

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn papers() -> Application {
    Application::called("org.gnome.Papers", "Papers").unwrap()
}

fn okular() -> Application {
    Application::called("org.kde.okular", "Okular").unwrap()
}

fn terminal() -> Application {
    Application::called("org.gnome.Terminal", "Terminal").unwrap()
}

/// A grant to an application, as a person's pick makes one.
fn granted_to(application: &str, reach: Reach) -> Grant {
    Grant::checked_for(
        &Applicant::named(application).grantee(),
        reach,
        noon(),
        hour(),
    )
    .unwrap()
}

/// Mail may reach the invoices folder, and Papers and Okular.
fn mails_grants() -> Grants {
    let mut grants = Grants::default();
    grants.grant(granted_to(
        MAIL,
        Reach::Folder(PathBuf::from("/home/anna/Invoices")),
    ));
    for application in [papers(), okular()] {
        grants.grant(granted_to(
            MAIL,
            Reach::Application(application.identifier().to_owned()),
        ));
    }
    grants
}

/// Papers, Okular and a terminal installed; Papers declares PDFs first.
fn a_machine() -> (Installed, Declared) {
    let installed = Installed::holding([papers(), okular(), terminal()]);
    let mut declared = Declared::nothing();
    declared.declares_media_types(&papers(), ["application/pdf"]);
    declared.declares_media_types(&okular(), ["application/pdf"]);
    declared.declares_media_types(&terminal(), ["text/plain"]);
    (installed, declared)
}

fn in_english() -> Strings {
    Strings::of(alo_saying::everything_this_machine_can_say().unwrap())
}

/// A file that counts every read, so a test can say none happened.
struct Counted {
    /// The bytes.
    bytes: Cursor<Vec<u8>>,
    /// How many reads were asked of it.
    reads: Rc<Cell<usize>>,
}

impl Counted {
    fn holding(bytes: &[u8]) -> (Self, Rc<Cell<usize>>) {
        let reads = Rc::new(Cell::new(0));
        (
            Self {
                bytes: Cursor::new(bytes.to_vec()),
                reads: Rc::clone(&reads),
            },
            reads,
        )
    }
}

impl Read for Counted {
    fn read(&mut self, into: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        self.bytes.read(into)
    }
}

impl Seek for Counted {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        self.reads.set(self.reads.get() + 1);
        self.bytes.seek(to)
    }
}

/// **The application that opens the file is what opens what answers** — the
/// declaration, then the person's choice once they make one — and never some
/// other application the asking one happens to hold a grant over.
#[test]
fn an_open_with_request_is_answered_from_what_opens_what_and_nowhere_else() {
    let (installed, declared) = a_machine();
    let grants = mails_grants();
    let before = serde_json::to_string(&grants).unwrap();

    // Nothing chosen: Papers, which declared PDFs first.
    let nothing_chosen = Chosen::untouched();
    let asking = WhatOpensWhat::on(&installed, &declared, &nothing_chosen);
    let opened = answered(
        MAIL,
        Path::new(THE_INVOICE),
        &mut Cursor::new(A_PDF.to_vec()),
        &asking,
        &grants,
        noon(),
    )
    .unwrap();
    let expected = asking.what_opens(&mut Cursor::new(A_PDF.to_vec())).unwrap();
    assert_eq!(opened.opener(), &expected);
    assert_eq!(opened.opener().application(), &papers());
    assert_eq!(opened.opener().because(), &Because::ItDeclaresIt);
    assert_eq!(opened.allowed().portal(), Portal::OpenWith);
    assert_eq!(opened.allowed().application(), &Applicant::named(MAIL));
    assert_eq!(
        opened.allowed().against().len(),
        2,
        "the file and the application that opens it"
    );

    // The person chooses Okular: the same request is answered with Okular.
    let mut chosen = Chosen::untouched();
    chosen.choose(Kind::Pdf, &okular());
    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    let opened = answered(
        MAIL,
        Path::new(THE_INVOICE),
        &mut Cursor::new(A_PDF.to_vec()),
        &asking,
        &grants,
        noon(),
    )
    .unwrap();
    assert_eq!(opened.opener().application(), &okular());
    assert!(opened.opener().was_chosen());
    let said = opened.opener().said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("you chose"), "{said}");

    // Answering widened nothing and chose nothing.
    assert_eq!(serde_json::to_string(&grants).unwrap(), before);
    assert_eq!(chosen.for_kind(Kind::Pdf), Some("org.kde.okular"));
    assert_eq!(chosen.all().count(), 1);
}

/// **An application granted nothing, or not granted the file, is refused —
/// and the file is never read** to find out what kind it is.
#[test]
fn a_request_not_granted_the_file_is_refused_before_the_file_is_read() {
    let (installed, declared) = a_machine();
    let chosen = Chosen::untouched();
    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    let grants = mails_grants();

    // Granted nothing at all.
    let (mut file, reads) = Counted::holding(A_PDF);
    let refused = answered(
        "org.example.Stranger",
        Path::new(THE_INVOICE),
        &mut file,
        &asking,
        &grants,
        noon(),
    )
    .unwrap_err();
    assert_eq!(
        refused,
        NotOpened::Refused(Refused::NothingGranted {
            application: Applicant::named("org.example.Stranger"),
            portal: Portal::OpenWith,
        })
    );
    assert_eq!(
        reads.get(),
        0,
        "the file was read for an application granted nothing"
    );
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("without asking you"), "{said}");

    // Granted things, and not this file.
    let (mut file, reads) = Counted::holding(A_PDF);
    let refused = answered(
        MAIL,
        Path::new("/home/anna/Taxes/2025.pdf"),
        &mut file,
        &asking,
        &grants,
        noon(),
    )
    .unwrap_err();
    assert!(
        matches!(
            &refused,
            NotOpened::Refused(Refused::NotAllowed {
                portal: Portal::OpenWith,
                why: NotAllowed::Never { wanted, .. },
            }) if *wanted == Ask::path("/home/anna/Taxes/2025.pdf")
        ),
        "{refused:?}"
    );
    assert_eq!(
        reads.get(),
        0,
        "the file was read before its grant was asked"
    );

    // Expired: the same.
    let (mut file, reads) = Counted::holding(A_PDF);
    assert!(matches!(
        answered(
            MAIL,
            Path::new(THE_INVOICE),
            &mut file,
            &asking,
            &grants,
            noon() + hour() * 2,
        ),
        Err(NotOpened::Refused(_))
    ));
    assert_eq!(reads.get(), 0);

    // What is not a request never reaches the grants or the file.
    let (mut file, reads) = Counted::holding(A_PDF);
    for (from, path, not) in [
        ("", THE_INVOICE, NotARequest::NoApplication),
        (MAIL, "march.pdf", NotARequest::NotAFullPath),
        (
            MAIL,
            "/home/anna/Invoices/../Taxes/2025.pdf",
            NotARequest::CouldLeadElsewhere,
        ),
    ] {
        assert_eq!(
            answered(from, Path::new(path), &mut file, &asking, &grants, noon()),
            Err(NotOpened::NotARequest(not)),
            "{from:?} {path:?}"
        );
    }
    assert_eq!(reads.get(), 0);
}

/// **The application that opens the file must be one the asking application was
/// granted** (ADR 0040: *the file, and the application to open it*). A person's
/// choice does not grant it; the refusal names it.
#[test]
fn the_application_that_opens_it_must_be_granted_too() {
    let (installed, declared) = a_machine();
    let grants = mails_grants();

    // The person chose the terminal for PDFs. Mail was never granted it.
    let mut chosen = Chosen::untouched();
    chosen.choose(Kind::Pdf, &terminal());
    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    let refused = answered(
        MAIL,
        Path::new(THE_INVOICE),
        &mut Cursor::new(A_PDF.to_vec()),
        &asking,
        &grants,
        noon(),
    )
    .unwrap_err();
    assert_eq!(
        refused,
        NotOpened::Refused(Refused::NotAllowed {
            portal: Portal::OpenWith,
            why: NotAllowed::Never {
                application: Applicant::named(MAIL),
                wanted: Ask::application("org.gnome.Terminal"),
            },
        })
    );
    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("org.gnome.Terminal"), "{said}");

    // Revoking Mail's grant over Papers ends the declared answer at the next
    // request.
    let mut grants = mails_grants();
    let papers_grant = grants
        .active_at(noon())
        .find(|held| held.grant.reach == Reach::Application("org.gnome.Papers".to_owned()))
        .map(|held| held.id)
        .expect("Mail holds a grant over Papers");
    let nothing_chosen = Chosen::untouched();
    let asking = WhatOpensWhat::on(&installed, &declared, &nothing_chosen);
    assert!(
        answered(
            MAIL,
            Path::new(THE_INVOICE),
            &mut Cursor::new(A_PDF.to_vec()),
            &asking,
            &grants,
            noon()
        )
        .is_ok()
    );
    assert!(grants.revoke(papers_grant));
    assert!(matches!(
        answered(
            MAIL,
            Path::new(THE_INVOICE),
            &mut Cursor::new(A_PDF.to_vec()),
            &asking,
            &grants,
            noon()
        ),
        Err(NotOpened::Refused(Refused::NotAllowed { .. }))
    ));
}

/// **A file nothing opens is refused with a sentence**, and the terminal that
/// declares text is not handed a PDF or a program.
#[test]
fn a_file_nothing_opens_is_a_sentence_and_not_a_fallback() {
    let installed = Installed::holding([terminal()]);
    let mut declared = Declared::nothing();
    declared.declares_media_types(&terminal(), ["text/plain"]);
    let chosen = Chosen::untouched();
    let asking = WhatOpensWhat::on(&installed, &declared, &chosen);
    let mut grants = mails_grants();
    grants.grant(granted_to(
        MAIL,
        Reach::Application("org.gnome.Terminal".to_owned()),
    ));
    let strings = in_english();

    let refused = answered(
        MAIL,
        Path::new(THE_INVOICE),
        &mut Cursor::new(A_PDF.to_vec()),
        &asking,
        &grants,
        noon(),
    )
    .unwrap_err();
    assert_eq!(
        refused,
        NotOpened::NothingOpens(NothingOpens::NoApplication {
            kind: Kind::Pdf,
            chosen: None
        })
    );
    let said = refused.said(&strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("a PDF document"), "{said}");

    let refused = answered(
        MAIL,
        Path::new(THE_INVOICE),
        &mut Cursor::new(A_PROGRAM.to_vec()),
        &asking,
        &grants,
        noon(),
    )
    .unwrap_err();
    assert_eq!(
        refused,
        NotOpened::NothingOpens(NothingOpens::TheFile(Cannot::AProgram))
    );
    assert!(!refused.said(&strings).is_a_bug());
}

/// **Nothing in the portal changes what opens what, or names an application
/// itself.** Its source never reaches for a door that changes a choice or a
/// declaration, never writes the person's file, and its open-with answer takes
/// what opens the file from `WhatOpensWhat` alone.
#[test]
fn nothing_in_the_portal_sets_an_association_or_picks_an_opener_itself() {
    let changing = [
        ".choose(",
        ".forget(",
        ".declares(",
        ".declares_media_types(",
        "keeping::",
        "put_back_as_shipped",
        "Chosen::",
        "Declared::",
        "Command::",
        "std::process",
    ];
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut read = 0;
    for entry in fs::read_dir(&source).unwrap() {
        let path = entry.unwrap().path();
        let text = fs::read_to_string(&path).unwrap();
        let code: String = text
            .split("#[cfg(test)]")
            .next()
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n");
        for word in changing {
            assert!(
                !code.contains(word),
                "{} reaches for `{word}`",
                path.display()
            );
        }
        read += 1;
    }
    assert!(read >= 7, "the crate's source was not found");

    // The one call that names the opener to the grants names what opens what
    // answered.
    let open_with = fs::read_to_string(source.join("open_with.rs")).unwrap();
    assert!(open_with.contains(".what_opens(file)"));
    assert!(
        open_with.contains("Request::opening_with(from, path, opener.application().identifier())")
    );
    assert_eq!(open_with.matches("Request::opening_with(").count(), 1);
}
