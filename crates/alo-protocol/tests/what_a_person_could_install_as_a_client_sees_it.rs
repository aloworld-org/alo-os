//! **The catalogue really read, carried back onto the wire.**
//!
//! Its own file beside `an_installation_as_a_client_sees_it.rs` because it is
//! the other half of the same verb list and the opposite kind of thing: that
//! one is a change a person approves, this one is a read that answers inside
//! the turn with nobody asked (ADR 0001 §5).
//!
//! What it asserts is the line a client reads, from a **real** turn: a call
//! validated against the declaration, authorised as a read, routed to
//! `alo_turn::looking_at_the_catalogue`, answered from the catalogue this
//! repository ships, and written as JSON. The unit test in `src/done.rs`
//! asserts the shape from a hand-built value and cannot tell you that what a
//! real read produces *is* that shape. This can.
//!
//! # Three things this file is here to hold, and each was once wrong somewhere
//!
//! - **Nobody is asked.** A read that waited for an approval would be a person
//!   being asked to approve a question, and `turning.reading` is the road that
//!   cannot do it — `alo_capability::Authorised::read` refuses a change offered
//!   as a read, and this asserts the other direction from the outside.
//! - **The answer carries nothing about this machine.** Not what is installed,
//!   not what is open, not what the tool said. The machine below has a tool
//!   with an application already in it, and the answer does not mention it.
//! - **It is written down.** A read leaves a record exactly as a change does:
//!   `CLAUDE.md`'s gate asks for *every execution and every refusal*, not every
//!   change.
//!
//! The tool is a list in memory, for the installation file's reason: what is
//! asked here is what crosses the wire.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_applications::Application;
use alo_capability::{Given, Grants};
use alo_context::{Context, Document};
use alo_egress::Indicator;
use alo_files::{OnThisMachine, Resolving as _};
use alo_protocol::{Done, ToAnAgent};
use alo_record::Record;
use alo_software::{
    Bound, Configured, Failed, SourceName, Tool, WHAT_CAN_BE_INSTALLED, words::KIND_OF_WORK,
};
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Machine, Turning};

/// What a verb did, as the wire carries it.
///
/// The fourth copy of the match `alo-protocol` cannot write for itself, and the
/// only one of the four whose own test takes this arm.
fn as_a_client_sees_it(did: &alo_turn::WhatAVerbDid) -> Done {
    match did {
        alo_turn::WhatAVerbDid::ToFiles(answer) => Done::of_a_file_verb(answer),
        alo_turn::WhatAVerbDid::AnApplicationInstalled(installed) => {
            Done::of_an_installation(installed)
        }
        alo_turn::WhatAVerbDid::AnApplicationWasReached(reached) => Done::of_an_application_act(
            reached.acting().on_the_wire(),
            reached.acting().application().identifier(),
        ),
        alo_turn::WhatAVerbDid::WhatCanBeInstalled(holds) => Done::of_a_catalogue_read(holds),
        // The fifth family, 2026-10-18, naming this copy along with the real
        // ones exactly as the third and fourth did.
        alo_turn::WhatAVerbDid::WhatTouchedAFile(found) => Done::of_a_record_read(found, noon()),
    }
}

/// A fixed moment, so nothing here depends on when it runs.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn lasts.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Every word a machine answering this has loaded.
fn everything_this_machine_says() -> Vocabulary {
    let mut vocabulary = alo_files::file_words().unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_applications::words::declare_into(&mut vocabulary).unwrap();
    alo_software::words::declare_into(&mut vocabulary).unwrap();
    alo_turn::declare_into(&mut vocabulary).unwrap();
    alo_protocol::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// A folder of this test's own with a file in it, for the invocation's context.
fn a_folder_of_our_own() -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-protocol-catalogue-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    fs::write(folder.join("a-note.txt"), "anything").unwrap();
    folder
}

/// An application this machine has and the catalogue does not list.
///
/// `alo_software::Catalogue::decided` refuses anything already shipped and the
/// catalogue is a fixed file, so no entry in it can ever be this — which is
/// what makes its absence from the answer mean something.
const THE_ONE_ONLY_THIS_MACHINE_HAS: &str = "org.example.OnlyOnThisMachine";

/// A machine with one place set up **and something already installed on it**.
///
/// The *already installed* part is the point: it is what the answer must not
/// mention. A tool with an empty list would make this file pass for the wrong
/// reason.
struct OnePlaceWithSomethingOnIt {
    /// What it reports as installed.
    installed: RefCell<Vec<String>>,
}

impl Default for OnePlaceWithSomethingOnIt {
    fn default() -> Self {
        Self {
            // **Deliberately not anything in the catalogue.** Krita was here
            // first and made `what_this_machine_already_has_does_not_cross`
            // useless: Krita is *in* the catalogue, so its identifier crosses
            // legitimately and no search of the line could tell the two
            // reasons apart. An identifier nothing ships can only be here
            // because the tool was asked.
            installed: RefCell::new(vec![THE_ONE_ONLY_THIS_MACHINE_HAS.to_owned()]),
        }
    }
}

impl Tool for OnePlaceWithSomethingOnIt {
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Ok(vec![Configured {
            name: "flathub".to_owned(),
            address: "https://dl.flathub.org/repo/".to_owned(),
            checks_signatures: true,
            switched_off: false,
        }])
    }

    fn installed(&self) -> Result<Vec<String>, Failed> {
        Ok(self.installed.borrow().clone())
    }

    fn open(&self) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
    }

    fn install(&self, _: &SourceName, application: &Application) -> Result<(), Failed> {
        self.installed
            .borrow_mut()
            .push(application.identifier().to_owned());
        Ok(())
    }

    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
    }

    fn update(&self, _: &Application) -> Result<(), Failed> {
        Ok(())
    }

    fn remove(&self, _: &Application) -> Result<(), Failed> {
        Ok(())
    }
}

/// Do one read of the catalogue on a real machine, and hand back what crossed,
/// how long the record is, and whether the indicator stayed quiet.
fn one_read(asking_for: &str) -> (ToAnAgent, usize, bool) {
    let strings = Strings::of(everything_this_machine_says());
    let folder = a_folder_of_our_own();
    let note = folder.join("a-note.txt");
    let tool = OnePlaceWithSomethingOnIt::default();
    let mut record = Record::default();

    let (told, quiet) = {
        let mut indicator = Indicator::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_and_software_verbs(
            &strings,
            &OnThisMachine,
            &tool,
            Bound::Nobodys,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();

        // **No grant for anything.** The read requires none, because the
        // catalogue is a file this machine ships and there is nothing on the
        // machine to grant over. A test that granted something would be
        // testing a machine nobody has.
        let mut grants = Grants::default();
        let offered = Context::at_invocation(noon()).and_document(Document::open(&note).unwrap());
        let told = {
            let mut turning =
                Turning::beginning(offered, "@software", hour(), &mut grants, &mut machine)
                    .unwrap();
            let did = turning
                .reading(
                    WHAT_CAN_BE_INSTALLED,
                    &[(KIND_OF_WORK, Given::text(asking_for))],
                    &grants,
                    noon(),
                )
                .expect("looking at the catalogue answers inside the turn");
            ToAnAgent::did(as_a_client_sees_it(&did))
        };
        // **Law 1's surface never moved.** Nothing left the machine, so there
        // was never a line to put up — this is not *the line came down again*,
        // it is *there was no errand*.
        (told, machine.showing().is_quiet())
    };

    (told, record.len(), quiet)
}

/// **The whole catalogue crosses as one line, with nothing of this machine in
/// it.**
#[test]
fn a_real_read_of_the_catalogue_crosses_as_the_whole_shipped_list() {
    let (told, written_down, quiet) = one_read("everything");

    assert!(quiet, "a read put something on the egress indicator");
    assert_eq!(written_down, 1, "the read was not written down");

    let Some(Done::CanBeInstalled {
        kind_of_work,
        applications,
    }) = told.done().cloned()
    else {
        unreachable!("a catalogue read is what a catalogue read answers with")
    };

    assert_eq!(kind_of_work, None, "everything narrowed to something");
    assert!(
        applications.len() >= 8,
        "{} applications crossed",
        applications.len()
    );
    for one in &applications {
        assert!(!one.application.is_empty());
        assert!(!one.source.is_empty());
        assert!(!one.version.is_empty());
        assert!(!one.licence.is_empty());
        assert!(!one.kind_of_work.is_empty());
    }
    assert!(
        !told.done().unwrap().left_something_out(),
        "a shipped list of fixed length said it was cut short"
    );

    // And it reads back as itself off the wire.
    let line = told.written().unwrap();
    assert!(!line.contains('\n'), "a message is one line: {line}");
    assert_eq!(ToAnAgent::read(&line).unwrap(), told);
}

/// **Narrowing crosses as that shelf and names the shelf it is.**
///
/// The option's own name crosses, never the word a person reads: a wire carries
/// the identity and the words are looked up where a person is.
#[test]
fn narrowing_crosses_as_one_shelf_named_by_its_option() {
    let (told, _, _) = one_read("page_layout");

    let Some(Done::CanBeInstalled {
        kind_of_work,
        applications,
    }) = told.done().cloned()
    else {
        unreachable!("a catalogue read is what a catalogue read answers with")
    };

    assert_eq!(kind_of_work, Some("page_layout".to_owned()));
    assert!(!applications.is_empty(), "the shelf crossed empty");
    for one in &applications {
        assert_eq!(
            one.kind_of_work, "page_layout",
            "{} is on the wrong shelf",
            one.application
        );
    }
    // Fewer than the whole list, which is what narrowing means.
    let (whole, _, _) = one_read("everything");
    let Some(Done::CanBeInstalled {
        applications: all, ..
    }) = whole.done().cloned()
    else {
        unreachable!()
    };
    assert!(applications.len() < all.len());
}

/// **Nothing about this machine crosses**, which is the one promise this verb
/// makes that could be broken silently.
///
/// The tool above reports [`THE_ONE_ONLY_THIS_MACHINE_HAS`] as installed, and
/// the catalogue cannot list it. So if a future change taught the read to say
/// *you already have this*, or to answer from the tool at all, that identifier
/// would appear in the line — and nothing else could put it there.
///
/// # This test was wrong first, in the way that matters
///
/// It searched the written line for the words `installed`, `sandboxed`,
/// `already` and `have`, and failed at once: the JSON key for this answer is
/// `can-be-installed`. **A pattern that matches the text of a thing is not a
/// pattern that matches the thing** — and worse, the tool then reported Krita,
/// which *is* in the catalogue, so a search could not have told a legitimate
/// entry from a leak even with the words fixed.
///
/// The question is *did anything come from the tool*, and the only way to ask
/// it is to put something in the tool that could come from nowhere else.
#[test]
fn what_this_machine_already_has_does_not_cross() {
    let (told, _, _) = one_read("everything");
    let line = told.written().unwrap();

    assert!(
        !line.contains(THE_ONE_ONLY_THIS_MACHINE_HAS),
        "an application only the tool knows about crossed in a catalogue answer: {line}"
    );
    // **And the control**, so a passing assertion means the line was read
    // rather than empty: something the catalogue really does hold is in it.
    assert!(
        line.contains("org.kde.krita"),
        "the catalogue's own entries did not cross either, so the test above proved nothing: \
         {line}"
    );
}

/// **A read asks nobody, and the road that would have asked refuses it.**
///
/// `Turning::proposing` is how a change is put to a person. Offered this verb
/// it must refuse, because a person approving *see what can be installed* would
/// be approving a question — ADR 0001 §5 is exactly this line, and
/// `alo_capability::Proposal::checked` is where it is held.
#[test]
fn the_read_cannot_be_put_to_a_person_for_approval() {
    let strings = Strings::of(everything_this_machine_says());
    let folder = a_folder_of_our_own();
    let note = folder.join("a-note.txt");
    let tool = OnePlaceWithSomethingOnIt::default();
    let mut record = Record::default();
    let mut indicator = Indicator::default();
    let mut bounding = NothingIsBounded;
    let mut machine = Machine::carrying_out_file_and_software_verbs(
        &strings,
        &OnThisMachine,
        &tool,
        Bound::Nobodys,
        &mut bounding,
        &mut indicator,
        &mut record,
    )
    .unwrap();
    let mut grants = Grants::default();
    let offered = Context::at_invocation(noon()).and_document(Document::open(&note).unwrap());
    let mut turning =
        Turning::beginning(offered, "@software", hour(), &mut grants, &mut machine).unwrap();

    let refused = turning.proposing(
        WHAT_CAN_BE_INSTALLED,
        &[(KIND_OF_WORK, Given::text("everything"))],
        &grants,
        hour(),
        noon(),
    );

    assert!(
        refused.is_err(),
        "a person was offered a question to approve"
    );
}

/// A boundary that does what it is asked, because what this file is about is
/// the wire rather than the kernel.
struct NothingIsBounded;

impl alo_turn::Bounding for NothingIsBounded {
    fn carrying_out(
        &mut self,
        _reaching: &alo_files::Reaching,
        doing: alo_turn::Doing<'_>,
    ) -> Result<alo_turn::Done, alo_turn::NoBoundary> {
        Ok(doing.done())
    }

    /// **Never reached by this file at all**, and worth saying rather than
    /// leaving as a stub the trait made necessary: looking at the catalogue
    /// goes nowhere. There is no departure to bound because there is no
    /// departure — `alo_turn::looking_at_the_catalogue` parses a string
    /// compiled into this binary.
    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        doing();
        Ok(())
    }
}
