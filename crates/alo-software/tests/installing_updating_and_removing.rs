//! Installing, updating and removing an application, held to each clause of the
//! plan's acceptance — and each refusal beside the act it refuses.
//!
//! The rented tool is a stand-in here ([`AMachine`]): it keeps what is
//! installed and open, what each place offers, and **every act it was asked to
//! do**, so a refusal is shown to have reached nothing rather than only to have
//! returned an error. What these tests cannot show is the real tool on a real
//! machine; this task's report says so, and says what would.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_applications::Application;
use alo_capability::{
    Agent, Applicant, Approvals, Ask, Authorised, Facility, Given, Grant, Grantee, NotAuthorised,
    Proposal, Reach,
};
use alo_egress::{Destination, Errand, Indicator, OnItsOwn};
use alo_granted::Listing;
use alo_software::{
    Bound, Configured, Enabled, Failed, NotAnInstallation, NotDone, SetBy, SourceName, Stopped,
    Tool, Wanted, apply, applying, approved, install, installing, looking_for_updates, offered,
    remove, software_verbs,
};
use alo_strings::{Strings, Vocabulary};

// ---------------------------------------------------------------------------
// The machine these tests look at.
// ---------------------------------------------------------------------------

/// The rented tool, standing in: what it holds, and everything it was asked.
struct AMachine {
    sources: Vec<Configured>,
    installed: RefCell<Vec<String>>,
    open: RefCell<Vec<String>>,
    offers: BTreeMap<String, Vec<String>>,
    newer: BTreeMap<String, Vec<String>>,
    signature_fails: bool,
    acts: RefCell<Vec<String>>,
}

impl AMachine {
    /// Two places set up — Flathub and the organisation's — and one set up
    /// with signature checking off; the text editor and the image viewer on
    /// offer, and a newer image viewer.
    fn ordinary() -> Self {
        let place = |name: &str, address: &str, checks: bool| Configured {
            name: name.to_owned(),
            address: address.to_owned(),
            checks_signatures: checks,
            switched_off: false,
        };
        Self {
            sources: vec![
                place("flathub", "https://dl.flathub.org/repo/", true),
                place("acme-apps", "https://apps.acme.example/repo", true),
                place("loose", "https://loose.example/repo", false),
            ],
            installed: RefCell::new(vec!["org.gnome.Loupe".to_owned()]),
            open: RefCell::new(Vec::new()),
            offers: BTreeMap::from([
                (
                    "flathub".to_owned(),
                    vec![
                        "org.gnome.TextEditor".to_owned(),
                        "org.gnome.Loupe".to_owned(),
                    ],
                ),
                ("loose".to_owned(), vec!["org.gnome.TextEditor".to_owned()]),
            ]),
            newer: BTreeMap::from([(
                "flathub".to_owned(),
                vec![
                    "org.gnome.Loupe".to_owned(),
                    "org.example.NotHere".to_owned(),
                ],
            )]),
            signature_fails: false,
            acts: RefCell::new(Vec::new()),
        }
    }

    fn acted(&self) -> Vec<String> {
        self.acts.borrow().clone()
    }
}

impl Tool for AMachine {
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Ok(self.sources.clone())
    }

    fn installed(&self) -> Result<Vec<String>, Failed> {
        Ok(self.installed.borrow().clone())
    }

    fn open(&self) -> Result<Vec<String>, Failed> {
        Ok(self.open.borrow().clone())
    }

    fn install(&self, source: &SourceName, application: &Application) -> Result<(), Failed> {
        self.acts.borrow_mut().push(format!(
            "install {} from {}",
            application.identifier(),
            source.as_str()
        ));
        if self.signature_fails {
            return Err(Failed::SignatureNotShown);
        }
        let offered = self
            .offers
            .get(source.as_str())
            .is_some_and(|offers| offers.iter().any(|app| app == application.identifier()));
        if !offered {
            return Err(Failed::NotOffered);
        }
        self.installed
            .borrow_mut()
            .push(application.identifier().to_owned());
        Ok(())
    }

    fn updates(&self, source: &SourceName) -> Result<Vec<String>, Failed> {
        Ok(self.newer.get(source.as_str()).cloned().unwrap_or_default())
    }

    fn update(&self, application: &Application) -> Result<(), Failed> {
        self.acts
            .borrow_mut()
            .push(format!("update {}", application.identifier()));
        Ok(())
    }

    fn remove(&self, application: &Application) -> Result<(), Failed> {
        self.acts
            .borrow_mut()
            .push(format!("remove {}", application.identifier()));
        let mut installed = self.installed.borrow_mut();
        let before = installed.len();
        installed.retain(|here| here != application.identifier());
        if installed.len() == before {
            return Err(Failed::NotInstalled);
        }
        Ok(())
    }
}

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

fn editor() -> Application {
    Application::identified("org.gnome.TextEditor").unwrap()
}

fn viewer() -> Application {
    Application::identified("org.gnome.Loupe").unwrap()
}

fn strings() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_egress::declare_into(&mut vocabulary).unwrap();
    alo_software::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A machine with an agent, holding a folder for `@files`, the camera for a
/// video-call application, and the image viewer's own grant to the camera and
/// an agent's grant over the image viewer.
fn a_machine_with_grants() -> Agent {
    let mut machine = Agent::present();
    let grants = machine.grants_mut().unwrap();
    grants.grant(
        Grant::checked(
            "@files",
            Reach::Folder(PathBuf::from("/home/anna/Invoices")),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked_for(
            &Applicant::named("org.example.Calls").grantee(),
            Reach::Facility(Facility::Camera),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked_for(
            &Applicant::named("org.gnome.Loupe").grantee(),
            Reach::Facility(Facility::Camera),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked(
            "@alo",
            Reach::Application("org.gnome.Loupe".to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    machine
}

/// Install the editor from this place the whole way, on this indicator.
fn installing_the_editor_from(
    tool: &AMachine,
    enabled: &Enabled,
    source: &str,
    indicator: &mut Indicator,
) -> Result<alo_software::Installed, Stopped> {
    let wanted = Wanted::by_hand(editor(), source);
    let errand = installing(enabled, &wanted, tool)?;
    let underway = indicator.beginning_on_its_own(errand, noon());
    let installed = install(&underway, enabled, wanted, tool);
    indicator.ended_on_its_own(underway);
    installed
}

// ---------------------------------------------------------------------------
// It arrives with no grants.
// ---------------------------------------------------------------------------

/// **An installed application arrives with no grants.** The one list is read
/// before and after installing, and nothing new is on it: no row for the
/// application, no reach for it anywhere, and every row that was there still
/// there.
#[test]
fn an_installed_application_arrives_with_no_grants() {
    let tool = AMachine::ordinary();
    let machine = a_machine_with_grants();
    let before = Listing::of(machine.allowed(), noon());

    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();
    let installed = installing_the_editor_from(&tool, &enabled, "flathub", &mut indicator).unwrap();
    assert_eq!(installed.application(), &editor());
    assert!(
        tool.installed()
            .unwrap()
            .contains(&"org.gnome.TextEditor".to_owned())
    );

    let after = Listing::of(machine.allowed(), noon());
    assert_eq!(before, after, "installing changed the one list");
    assert!(
        after
            .rows()
            .iter()
            .all(|row| row.to() != "org.gnome.TextEditor"),
        "the new application has a row"
    );
    let newcomer = Applicant::named("org.gnome.TextEditor");
    assert!(!machine.allowed().allows_anything(&newcomer, noon()));
    for facility in [Facility::Camera, Facility::Microphone] {
        assert!(
            machine
                .allowing(&newcomer, &Ask::facility(facility), noon())
                .is_err()
        );
    }
    let said = installed.said(&strings());
    assert!(said.text().contains("given nothing"), "{said}");
}

// ---------------------------------------------------------------------------
// What leaves is on the indicator, named as what it is.
// ---------------------------------------------------------------------------

/// **Installing, looking for updates and updating each leave on the indicator,
/// named as what they are** — the line is showing while the rented tool is
/// reached, it names the place, and it goes when the act ends.
#[test]
fn installing_and_updating_are_on_the_indicator_named_as_what_they_are() {
    let tool = AMachine::ordinary();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let strings = strings();
    let mut indicator = Indicator::default();

    // Installing.
    let wanted = Wanted::by_hand(editor(), "flathub");
    let errand = installing(&enabled, &wanted, &tool).unwrap();
    assert!(
        indicator.is_quiet(),
        "something was shown before it was begun"
    );
    assert!(
        tool.acted().is_empty(),
        "the tool was reached before the line was shown"
    );
    let underway = indicator.beginning_on_its_own(errand, noon());
    assert_eq!(
        indicator.showing().first().unwrap().said(&strings).text(),
        "alo OS is installing an application from dl.flathub.org"
    );
    install(&underway, &enabled, wanted, &tool).unwrap();
    assert_eq!(tool.acted(), ["install org.gnome.TextEditor from flathub"]);
    assert!(indicator.ended_on_its_own(underway));
    assert!(indicator.is_quiet());

    // Looking for updates.
    let underway =
        indicator.beginning_on_its_own(looking_for_updates(&enabled, "flathub").unwrap(), noon());
    assert_eq!(
        indicator.showing().first().unwrap().said(&strings).text(),
        "alo OS is checking for application updates at dl.flathub.org"
    );
    let offers = offered(&underway, &enabled, "flathub", &tool).unwrap();
    indicator.ended_on_its_own(underway);
    let [offer] = <[alo_software::Offer; 1]>::try_from(offers).unwrap();
    assert_eq!(offer.application(), &viewer());

    // Updating.
    let underway =
        indicator.beginning_on_its_own(applying(&enabled, &offer, &tool).unwrap(), noon());
    assert_eq!(
        indicator.showing().first().unwrap().said(&strings).text(),
        "alo OS is updating an application from dl.flathub.org"
    );
    apply(&underway, &enabled, offer, &tool).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    assert!(indicator.is_quiet());
    assert_eq!(
        tool.acted(),
        [
            "install org.gnome.TextEditor from flathub",
            "update org.gnome.Loupe"
        ]
    );
}

/// **An act under a line that does not describe it reaches nothing.** Another
/// errand's line, or the same errand's naming another place, is refused before
/// the rented tool is asked anything.
#[test]
fn an_act_under_another_line_reaches_nothing() {
    let tool = AMachine::ordinary();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();
    let flathub = Destination::at("dl.flathub.org").unwrap();

    let signing_in =
        indicator.beginning_on_its_own(OnItsOwn::for_(Errand::SigningIn, flathub), noon());
    let refused = install(
        &signing_in,
        &enabled,
        Wanted::by_hand(editor(), "flathub"),
        &tool,
    );
    assert!(
        matches!(&refused, Err(Stopped::NotShown(not)) if not.showing() == Errand::SigningIn),
        "{refused:?}"
    );

    let elsewhere = indicator.beginning_on_its_own(
        OnItsOwn::for_(
            Errand::InstallingAnApplication,
            Destination::at("apps.acme.example").unwrap(),
        ),
        noon(),
    );
    assert!(matches!(
        install(
            &elsewhere,
            &enabled,
            Wanted::by_hand(editor(), "flathub"),
            &tool
        ),
        Err(Stopped::NotShown(_))
    ));
    assert!(matches!(
        offered(&elsewhere, &enabled, "flathub", &tool),
        Err(Stopped::NotShown(_))
    ));
    assert!(tool.acted().is_empty(), "{:?}", tool.acted());
}

/// **Removing reaches no network, so the indicator stays quiet** — a line for
/// it would say something left that did not.
#[test]
fn removing_puts_nothing_on_the_indicator() {
    let tool = AMachine::ordinary();
    let mut machine = a_machine_with_grants();
    let indicator = Indicator::default();
    remove(viewer(), &tool, &mut machine, noon()).unwrap();
    assert!(indicator.is_quiet());
    assert_eq!(tool.acted(), ["remove org.gnome.Loupe"]);
    assert!(
        Errand::EVERY
            .iter()
            .all(|errand| !format!("{errand:?}").contains("Remov"))
    );
}

// ---------------------------------------------------------------------------
// An update is offered, never applied while the application runs.
// ---------------------------------------------------------------------------

/// **An update is offered, and never applied while the application is open.**
/// Looking finds it and applies nothing; choosing it while the viewer is open
/// is refused in words before anything leaves; the viewer opened between the
/// two steps is still refused under the line; closed, it updates — and nothing
/// else is closed or restarted.
#[test]
fn an_update_is_offered_and_never_applied_while_the_application_is_open() {
    let tool = AMachine::ordinary();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let strings = strings();
    let mut indicator = Indicator::default();

    let look = |indicator: &mut Indicator| {
        let underway = indicator
            .beginning_on_its_own(looking_for_updates(&enabled, "flathub").unwrap(), noon());
        let offers = offered(&underway, &enabled, "flathub", &tool).unwrap();
        indicator.ended_on_its_own(underway);
        offers
    };

    // Offered: only what is installed, and nothing applied by looking.
    let offers = look(&mut indicator);
    assert_eq!(
        offers.len(),
        1,
        "an update for something not installed was offered"
    );
    assert!(
        tool.acted().is_empty(),
        "looking applied something: {:?}",
        tool.acted()
    );
    let offer = offers.into_iter().next().unwrap();
    let said = offer.said(&strings);
    assert!(said.text().contains("never while it is open"), "{said}");

    // Open: refused before anything leaves.
    tool.open.borrow_mut().push("org.gnome.Loupe".to_owned());
    assert_eq!(
        applying(&enabled, &offer, &tool).unwrap_err(),
        NotDone::StillOpen {
            application: "org.gnome.Loupe".to_owned()
        }
    );
    assert!(indicator.is_quiet());
    let said = NotDone::StillOpen {
        application: "org.gnome.Loupe".to_owned(),
    }
    .said(&strings);
    assert!(
        said.text().contains("never closes an application"),
        "{said}"
    );

    // Closed when asked, opened again before it ran: refused under the line.
    tool.open.borrow_mut().clear();
    let underway =
        indicator.beginning_on_its_own(applying(&enabled, &offer, &tool).unwrap(), noon());
    tool.open.borrow_mut().push("org.gnome.Loupe".to_owned());
    assert_eq!(
        apply(&underway, &enabled, offer, &tool).unwrap_err(),
        Stopped::Refused(NotDone::StillOpen {
            application: "org.gnome.Loupe".to_owned()
        })
    );
    indicator.ended_on_its_own(underway);
    assert!(tool.acted().is_empty(), "an open application was updated");

    // Closed: updated, and the offer had to be looked for again.
    tool.open.borrow_mut().clear();
    let offer = look(&mut indicator).into_iter().next().unwrap();
    let underway =
        indicator.beginning_on_its_own(applying(&enabled, &offer, &tool).unwrap(), noon());
    let updated = apply(&underway, &enabled, offer, &tool).unwrap();
    indicator.ended_on_its_own(underway);
    assert_eq!(tool.acted(), ["update org.gnome.Loupe"]);
    let said = updated.said(&strings);
    assert!(
        said.text().contains("Nothing else was closed or restarted"),
        "{said}"
    );
}

// ---------------------------------------------------------------------------
// Removing ends its grants in the same act.
// ---------------------------------------------------------------------------

/// **Removing an application ends its grants in the one list in the same
/// act**: the grant it held and the agent's grant over it are gone from the
/// list and permit nothing the moment `remove` returns, and every other row
/// stays exactly as it was.
#[test]
fn removing_an_application_ends_its_grants_in_the_same_act() {
    let tool = AMachine::ordinary();
    let mut machine = a_machine_with_grants();
    let rows = |machine: &Agent| -> Vec<String> {
        Listing::of(machine.allowed(), noon())
            .rows()
            .iter()
            .map(|row| row.to().to_owned())
            .collect()
    };
    assert_eq!(
        rows(&machine),
        ["@files", "org.example.Calls", "org.gnome.Loupe", "@alo"]
    );

    let removed = remove(viewer(), &tool, &mut machine, noon()).unwrap();
    assert_eq!(removed.grants_ended(), 2);
    assert_eq!(rows(&machine), ["@files", "org.example.Calls"]);
    assert!(
        machine
            .allowing(
                &Applicant::named("org.gnome.Loupe"),
                &Ask::facility(Facility::Camera),
                noon()
            )
            .is_err(),
        "the removed application can still use the camera"
    );
    assert!(
        !machine.permits(
            &Grantee::named("@alo"),
            &Ask::Application("org.gnome.Loupe".to_owned()),
            noon()
        ),
        "an agent can still reach whatever next answers to the removed application's name"
    );
    assert!(
        machine
            .allowing(
                &Applicant::named("org.example.Calls"),
                &Ask::facility(Facility::Camera),
                noon()
            )
            .is_ok()
    );
    let said = removed.said(&strings());
    assert!(said.text().contains("has ended"), "{said}");
}

/// **On a machine that declined the agent, removing still ends what the
/// application held** — through the only door such a machine has.
#[test]
fn removing_on_a_machine_without_an_agent_ends_what_the_application_held() {
    let tool = AMachine::ordinary();
    let mut machine = Agent::declined();
    machine.allow(
        Grant::checked_for(
            &Applicant::named("org.gnome.Loupe").grantee(),
            Reach::Facility(Facility::Camera),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let removed = remove(viewer(), &tool, &mut machine, noon()).unwrap();
    assert_eq!(removed.grants_ended(), 1);
    assert!(Listing::of(machine.allowed(), noon()).is_nothing_granted());
}

/// **A removal that did not happen ends nothing.** Not installed, or refused by
/// the tool: the one list is exactly as it was.
#[test]
fn a_removal_that_did_not_happen_ends_no_grant() {
    let tool = AMachine::ordinary();
    let mut machine = a_machine_with_grants();
    let before = Listing::of(machine.allowed(), noon());

    assert_eq!(
        remove(editor(), &tool, &mut machine, noon()).unwrap_err(),
        NotDone::NotInstalled {
            application: "org.gnome.TextEditor".to_owned()
        }
    );
    assert!(tool.acted().is_empty());
    assert_eq!(Listing::of(machine.allowed(), noon()), before);

    /// A tool that lists the viewer and will not remove it.
    struct Stubborn(AMachine);
    impl Tool for Stubborn {
        fn sources(&self) -> Result<Vec<Configured>, Failed> {
            self.0.sources()
        }
        fn installed(&self) -> Result<Vec<String>, Failed> {
            self.0.installed()
        }
        fn open(&self) -> Result<Vec<String>, Failed> {
            self.0.open()
        }
        fn install(&self, source: &SourceName, application: &Application) -> Result<(), Failed> {
            self.0.install(source, application)
        }
        fn updates(&self, source: &SourceName) -> Result<Vec<String>, Failed> {
            self.0.updates(source)
        }
        fn update(&self, application: &Application) -> Result<(), Failed> {
            self.0.update(application)
        }
        fn remove(&self, _: &Application) -> Result<(), Failed> {
            Err(Failed::DidNotAnswer {
                said: "error: Unable to connect to system bus".to_owned(),
            })
        }
    }
    let stubborn = Stubborn(AMachine::ordinary());
    let refused = remove(viewer(), &stubborn, &mut machine, noon()).unwrap_err();
    assert!(
        matches!(refused, NotDone::DidNotAnswer { .. }),
        "{refused:?}"
    );
    assert_eq!(Listing::of(machine.allowed(), noon()), before);
    let said = refused.said(&strings());
    assert!(!said.text().contains("system bus"), "{said}");
}

// ---------------------------------------------------------------------------
// A source that fails verification is refused in words, never installed from.
// ---------------------------------------------------------------------------

/// **A place set up without signature checking is refused in words, before
/// anything leaves**, and so is a delivery whose signature the rented tool
/// could not verify — in which case nothing is installed.
#[test]
fn a_source_that_fails_verification_is_refused_in_words_and_never_installed_from() {
    let strings = strings();
    let mut tool = AMachine::ordinary();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();

    // Set up not to check: refused before the line, before the tool.
    let refused = installing_the_editor_from(&tool, &enabled, "loose", &mut indicator).unwrap_err();
    assert_eq!(
        refused,
        Stopped::Refused(NotDone::NotVerified {
            source: "loose".to_owned()
        })
    );
    assert!(indicator.is_quiet());
    assert!(tool.acted().is_empty());
    assert!(
        looking_for_updates(&enabled, "loose").is_err(),
        "updates were looked for at a place that does not check signatures"
    );
    let said = NotDone::NotVerified {
        source: "loose".to_owned(),
    }
    .said(&strings);
    assert!(said.text().contains("really comes from it"), "{said}");

    // Checks, and the delivery fails the check: refused, nothing installed.
    tool.signature_fails = true;
    let refused =
        installing_the_editor_from(&tool, &enabled, "flathub", &mut indicator).unwrap_err();
    assert_eq!(
        refused,
        Stopped::Refused(NotDone::SignatureNotShown {
            source: "flathub".to_owned(),
            application: "org.gnome.TextEditor".to_owned()
        })
    );
    assert!(
        !tool
            .installed()
            .unwrap()
            .contains(&"org.gnome.TextEditor".to_owned())
    );
    assert!(
        indicator.is_quiet(),
        "the line outlived the refused installation"
    );
    let said = NotDone::SignatureNotShown {
        source: "flathub".to_owned(),
        application: "org.gnome.TextEditor".to_owned(),
    }
    .said(&strings);
    assert!(
        said.text()
            .contains("could not be shown to come from flathub"),
        "{said}"
    );
}

/// **Every other way an installation is refused names what it refused**, and
/// none of them reaches the tool's install: a place not set up, an application
/// already here, and one the place does not offer.
#[test]
fn an_installation_is_refused_in_words_for_every_other_reason() {
    let tool = AMachine::ordinary();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();

    assert_eq!(
        installing(&enabled, &Wanted::by_hand(editor(), "nowhere"), &tool).unwrap_err(),
        NotDone::not_enabled("nowhere")
    );
    assert_eq!(
        installing(&enabled, &Wanted::by_hand(viewer(), "flathub"), &tool).unwrap_err(),
        NotDone::AlreadyInstalled {
            application: "org.gnome.Loupe".to_owned()
        }
    );
    assert!(tool.acted().is_empty());

    let refused =
        installing_the_editor_from(&tool, &enabled, "acme-apps", &mut indicator).unwrap_err();
    assert_eq!(
        refused,
        Stopped::Refused(NotDone::NotOffered {
            source: "acme-apps".to_owned(),
            application: "org.gnome.TextEditor".to_owned()
        })
    );
    assert!(indicator.is_quiet());
}

// ---------------------------------------------------------------------------
// The organisation bounds; the refusal names who set it.
// ---------------------------------------------------------------------------

/// **A place outside the organisation's rule is refused out loud, naming who
/// set it**, for installing and for looking for updates — and a permitted place
/// is used as before. Nothing is quietly swapped for a place that is permitted.
#[test]
fn a_place_outside_the_rule_is_refused_naming_who_set_it() {
    let tool = AMachine::ordinary();
    let enabled = Enabled::read(
        &tool,
        Bound::only(
            [SourceName::checked("acme-apps").unwrap()],
            SetBy::AnAdministrator,
        ),
    )
    .unwrap();
    let outside = NotDone::OutsideTheBound {
        source: "flathub".to_owned(),
        set_by: SetBy::AnAdministrator,
    };
    assert_eq!(
        installing(&enabled, &Wanted::by_hand(editor(), "flathub"), &tool).unwrap_err(),
        outside
    );
    assert_eq!(
        looking_for_updates(&enabled, "flathub").unwrap_err(),
        outside
    );
    assert!(tool.acted().is_empty());
    let said = outside.said(&strings());
    assert!(
        said.text()
            .contains("organisation that manages this machine"),
        "{said}"
    );
    assert!(installing(&enabled, &Wanted::by_hand(editor(), "acme-apps"), &tool).is_ok());
}

// ---------------------------------------------------------------------------
// An agent proposes; a person approves; it never installs by itself.
// ---------------------------------------------------------------------------

/// **An agent may propose installing an application through a verb a person
/// approves, and never install one by itself.** Its call waits for approval
/// and cannot be authorised as a read; approved once, it becomes exactly one
/// installation; and an authority that is not an approved installation
/// installs nothing.
#[test]
fn an_agent_proposes_an_installation_and_never_installs_one_by_itself() {
    let tool = AMachine::ordinary();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let machine = a_machine_with_grants();
    let agent = Grantee::named("@alo");
    let verbs = software_verbs().unwrap();
    let call = verbs
        .call(
            "install_application",
            &[
                ("application", Given::text("org.gnome.TextEditor")),
                ("source", Given::text("flathub")),
            ],
        )
        .unwrap();
    assert!(call.waits_for_approval());
    assert_eq!(
        call.sentence(&strings()).text(),
        "install org.gnome.TextEditor from flathub"
    );

    // By itself: refused as a change that waits.
    let by_itself = Authorised::read(&call, &agent, machine.grants().unwrap(), noon()).unwrap_err();
    assert!(matches!(by_itself.why(), NotAuthorised::ChangeWaits { .. }));
    assert!(tool.acted().is_empty());

    // Proposed, approved once, redeemed once: one installation.
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(&call, &agent, machine.grants().unwrap(), noon(), hour()).unwrap(),
    );
    let authorised = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(machine.grants().unwrap(), noon())
        .unwrap();
    let wanted = approved(&authorised).unwrap();
    assert!(
        approvals.approve(id, noon()).is_err(),
        "one approval was answered twice"
    );
    let mut indicator = Indicator::default();
    let underway =
        indicator.beginning_on_its_own(installing(&enabled, &wanted, &tool).unwrap(), noon());
    install(&underway, &enabled, wanted, &tool).unwrap();
    indicator.ended_on_its_own(underway);
    assert_eq!(tool.acted(), ["install org.gnome.TextEditor from flathub"]);

    // Another verb's approved authority installs nothing.
    let open = alo_applications::application_verbs()
        .unwrap()
        .call(
            "open_application",
            &[("application", Given::text("org.gnome.Loupe"))],
        )
        .unwrap();
    let id = approvals.propose(
        Proposal::checked(&open, &agent, machine.grants().unwrap(), noon(), hour()).unwrap(),
    );
    let other = approvals
        .approve(id, noon())
        .unwrap()
        .redeem(machine.grants().unwrap(), noon())
        .unwrap();
    assert_eq!(
        approved(&other).unwrap_err(),
        NotAnInstallation::AnotherVerb {
            verb: "open_application".to_owned()
        }
    );
}
