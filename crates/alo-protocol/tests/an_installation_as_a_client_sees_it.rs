//! **An application really installed, carried back onto the wire.**
//!
//! Its own file beside `what_the_daemon_answers_with.rs` because it is the
//! second verb family, and because of what it has to do to be honest:
//! `alo_software::Installed` has no public constructor, deliberately — one of
//! those is evidence that an installation happened, and anything able to make
//! one could claim it did. So the only way to test the mapping from that
//! evidence onto the wire is to **install something**, through the same road a
//! person's approval takes.
//!
//! What it asserts is the line a client reads. `docs/contracts/daemon-protocol.md`
//! prints that line, and the unit test in `src/done.rs` asserts its shape from
//! a hand-built value; neither of them can tell you that what a real
//! installation produces *is* that shape. This can.
//!
//! The tool is a list in memory rather than the rented one: what is asked here
//! is what crosses the wire, and whether `/usr/bin/flatpak` can be driven is
//! `alo-software`'s question and is tested there.

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
use alo_protocol::{Done, ToAPerson};
use alo_record::Record;
use alo_software::{Bound, Configured, Failed, INSTALL_APPLICATION, SourceName, Tool};
use alo_strings::{Strings, Vocabulary};
use alo_turn::{Machine, Turning};

/// What a verb did, as the wire carries it.
///
/// The three-arm match this crate cannot write for itself: it holds `alo-turn`
/// as a dev-dependency, so `Done` has one constructor per family and whoever
/// knows what a turn is picks between them.
///
/// It was two arms until 2026-10-13. A third family of verbs arrived and the
/// compiler named this copy along with the two real ones, which is what a
/// closed enum is for.
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
    }
}

/// A fixed moment, so nothing here depends on when it runs.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the turn and the question last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Every word a machine answering this has loaded — including the two families'
/// own, so a refusal would read as a sentence rather than a missing key.
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
        "alo-protocol-installs-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    let folder = OnThisMachine.real(&folder).unwrap().into_path_buf();
    fs::write(folder.join("a-note.txt"), "anything").unwrap();
    folder
}

/// A machine with one place set up, which remembers what it installed.
#[derive(Default)]
struct OnePlace {
    /// Every application it was asked to install, in order.
    installed: RefCell<Vec<String>>,
}

impl Tool for OnePlace {
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

/// **A real installation crosses as one line, and it is the line the contract
/// prints.**
///
/// The whole chain in one test: a call validated against the declaration, a
/// change put to a person, one approval, `alo-software`'s two steps either side
/// of the indicator, the evidence that came back, and the JSON a client reads.
#[test]
fn an_application_really_installed_crosses_as_the_line_the_contract_prints() {
    let strings = Strings::of(everything_this_machine_says());
    let folder = a_folder_of_our_own();
    let note = folder.join("a-note.txt");
    let tool = OnePlace::default();
    let mut record = Record::default();

    // The machine borrows the record and the indicator, so both live outside
    // the block and are read after it ends. `what_the_daemon_answers_with.rs`
    // uses a closure for the same reason; a block is enough here because there
    // is one test.
    let told = {
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

        // No grant for anything: `install_application` requires none, because
        // the application is not on the machine yet and arrives granted
        // nothing. A test that granted something would be testing a machine
        // nobody has.
        let mut grants = Grants::default();
        let offered = Context::at_invocation(noon()).and_document(Document::open(&note).unwrap());
        let told = {
            let mut turning =
                Turning::beginning(offered, "@software", hour(), &mut grants, &mut machine)
                    .unwrap();

            let waiting = turning
                .proposing(
                    INSTALL_APPLICATION,
                    &[
                        ("application", Given::text("org.gnome.TextEditor")),
                        ("source", Given::text("flathub")),
                    ],
                    &grants,
                    hour(),
                    noon(),
                )
                .expect("an installation is a change, so it waits for a person");
            assert!(
                tool.installed.borrow().is_empty(),
                "it was installed before anybody approved it"
            );

            let did = turning
                .approving(waiting, &grants, noon())
                .expect("the person approved the sentence");
            assert_eq!(
                *tool.installed.borrow(),
                ["org.gnome.TextEditor"],
                "the tool was reached once, with what was approved"
            );
            // This is the half no other test can reach: evidence a real
            // installation produced, turned into the line a client reads.
            ToAPerson::did(as_a_client_sees_it(&did))
        };

        // Law 1's surface is quiet again. An installation reaches somewhere
        // else, so the line went up and came down; one left lit would tell a
        // person something is leaving when nothing is.
        assert!(
            machine.showing().is_quiet(),
            "the installation left the indicator lit"
        );
        told
    };

    assert_eq!(
        told.done().unwrap(),
        &Done::Installed {
            application: "org.gnome.TextEditor".to_owned(),
            sandboxed: true,
        }
    );
    // The line itself, as it goes down the socket. `Told` is what is written,
    // and `docs/contracts/daemon-protocol.md` prints exactly this.
    let written = told.written().unwrap();
    assert!(!written.contains('\n'), "a message is one line: {written}");
    assert_eq!(
        written,
        r#"{"format":1,"tells":{"did":{"installed":{"application":"org.gnome.TextEditor","sandboxed":true}}}}"#
    );

    // And it is written down. A change that happened with no record of it is
    // the one thing `alo-turn` will not do.
    assert_eq!(record.len(), 1);
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

    /// And a network request, carried out where it stands. The same argument as
    /// above: what this test needs is a boundary that is not one, written where
    /// whoever reads it can see that is what it is.
    ///
    /// **Not reached by an installation.** `alo-turn`'s installing executor puts
    /// no boundary around the rented tool and says at length why — there are no
    /// resolved paths to make a reach out of. It is here because the trait has
    /// two methods.
    fn carrying_out_a_departure(
        &mut self,
        _to: &[std::net::SocketAddr],
        doing: &mut dyn FnMut(),
    ) -> Result<(), alo_turn::NoBoundary> {
        doing();
        Ok(())
    }
}
