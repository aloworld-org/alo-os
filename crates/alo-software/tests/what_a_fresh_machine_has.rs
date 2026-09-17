//! What a fresh machine has, so it is not helpless — held to each clause of the
//! plan's acceptance, with each refusal beside what it refuses.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 2: a decided list of
//! pinned upstream applications, installed, updated and removed the way a
//! person does any application; any of them removable, the browser included;
//! and **the terminal a person's and not an agent's**, held against every verb
//! alo OS ships (ADR 0043).
//!
//! The rented tool is a stand-in ([`AFreshMachine`]) that offers every shipped
//! application from the place a fresh machine installs from and records every
//! act it is asked to do. What these tests cannot show is the real tool on a
//! real machine; the task's report says what would.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_applications::Application;
use alo_by_hand::{THE_WORKSPACE, whoever_declares_verbs};
use alo_capability::{
    Agent, Applicant, Ask, Authorised, Given, Grant, GrantError, GrantId, Grantee, Grants, Held,
    NotAuthorised, NotGranted, Proposal, ProposalError, Reach, Takes, Verb, Verbs,
};
use alo_egress::{Destination, Errand, Indicator};
use alo_software::{
    Bound, Configured, Enabled, Failed, NotShipped, Role, Shipped, SourceName, Tool, apply,
    applying, install, installing, looking_for_updates, offered, remove, shipped,
};

// ---------------------------------------------------------------------------
// The machine these tests look at.
// ---------------------------------------------------------------------------

/// The rented tool, standing in: Flathub set up, everything shipped on offer
/// there with a newer release of each, and everything it was asked.
struct AFreshMachine {
    installed: RefCell<Vec<String>>,
    acts: RefCell<Vec<String>>,
}

impl AFreshMachine {
    fn new() -> Self {
        Self {
            installed: RefCell::new(Vec::new()),
            acts: RefCell::new(Vec::new()),
        }
    }

    fn shipped() -> Vec<String> {
        Shipped::decided()
            .unwrap()
            .every()
            .iter()
            .map(|pinned| pinned.application().identifier().to_owned())
            .collect()
    }

    fn acted(&self) -> Vec<String> {
        self.acts.borrow().clone()
    }
}

impl Tool for AFreshMachine {
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

    fn install(&self, source: &SourceName, application: &Application) -> Result<(), Failed> {
        self.acts.borrow_mut().push(format!(
            "install {} from {}",
            application.identifier(),
            source.as_str()
        ));
        if !Self::shipped().contains(&application.identifier().to_owned()) {
            return Err(Failed::NotOffered);
        }
        self.installed
            .borrow_mut()
            .push(application.identifier().to_owned());
        Ok(())
    }

    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Ok(Self::shipped())
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

/// This repository, from the crate this test is in.
fn the_repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .unwrap()
}

/// Install everything a fresh machine ships, each through the same two steps a
/// person's installation takes.
fn installing_everything(tool: &AFreshMachine, enabled: &Enabled, indicator: &mut Indicator) {
    for pinned in Shipped::decided().unwrap().every() {
        let wanted = pinned.wanted();
        let errand = installing(enabled, &wanted, tool).unwrap();
        let underway = indicator.beginning_on_its_own(errand, noon());
        install(&underway, enabled, wanted, tool).unwrap();
        indicator.ended_on_its_own(underway);
    }
}

// ---------------------------------------------------------------------------
// The decided list.
// ---------------------------------------------------------------------------

/// **A decided list — a web browser, a file manager with trash and what opens
/// its archives, a text editor, an image viewer, a document viewer and a
/// terminal — each a pinned upstream application named by its source
/// identifier and version**, under its licence, from the place a fresh machine
/// installs from; and the file the installer plan reads is the list this crate
/// holds, byte for byte.
#[test]
fn the_decided_list_names_each_role_as_a_pinned_upstream_application() {
    let shipped = Shipped::decided().unwrap();
    let decided = [
        (Role::WebBrowser, "org.mozilla.firefox", "156.0", "MPL-2.0"),
        (
            Role::FileManager,
            "org.kde.dolphin",
            "26.04.3",
            "GPL-2.0-or-later",
        ),
        (Role::Archives, "org.kde.ark", "26.04.3", "GPL-2.0-or-later"),
        (
            Role::TextEditor,
            "org.gnome.TextEditor",
            "50.1",
            "GPL-3.0-or-later",
        ),
        (
            Role::ImageViewer,
            "org.gnome.Loupe",
            "50.0",
            "GPL-3.0-or-later",
        ),
        (
            Role::DocumentViewer,
            "org.gnome.Papers",
            "50.2",
            "GPL-2.0-or-later",
        ),
        (
            Role::Terminal,
            "app.devsuite.Ptyxis",
            "50.1",
            "GPL-3.0-or-later",
        ),
    ];
    assert_eq!(shipped.every().len(), decided.len());
    for (role, identifier, version, licence) in decided {
        let pinned = shipped.the(role);
        assert_eq!(pinned.role(), role);
        assert_eq!(pinned.application().identifier(), identifier, "{role:?}");
        assert_eq!(pinned.version(), version, "{role:?}");
        assert_eq!(pinned.licence(), licence, "{role:?}");
        assert_eq!(pinned.source().as_str(), "flathub", "{role:?}");
    }

    let on_the_disk = fs::read_to_string(the_repository().join(shipped::WHERE_IT_IS)).unwrap();
    assert_eq!(on_the_disk, shipped::THE_LIST);
    assert_eq!(Shipped::read(&on_the_disk).unwrap(), shipped);
}

/// **Each is installed the way a person installs one**, so updating and
/// removing it is no different: the same errands on the indicator, named at the
/// same place, arriving granted nothing, offered an update and updated with the
/// application closed, and removed.
#[test]
fn a_fresh_machine_installs_updates_and_removes_them_as_a_person_does() {
    let tool = AFreshMachine::new();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let flathub = Destination::at("dl.flathub.org").unwrap();
    let mut machine = Agent::present();
    let mut indicator = Indicator::default();

    for pinned in Shipped::decided().unwrap().every() {
        let wanted = pinned.wanted();
        let errand = installing(&enabled, &wanted, &tool).unwrap();
        assert_eq!(errand.errand(), Errand::InstallingAnApplication);
        assert_eq!(errand.destination(), &flathub);
        let underway = indicator.beginning_on_its_own(errand, noon());
        assert!(!indicator.is_quiet());
        install(&underway, &enabled, wanted, &tool).unwrap();
        indicator.ended_on_its_own(underway);
    }
    assert!(indicator.is_quiet());
    assert_eq!(
        machine.grants().unwrap().len(),
        0,
        "a shipped application arrived holding a grant"
    );
    assert_eq!(*tool.installed.borrow(), AFreshMachine::shipped());

    // Installed once is installed: a second installation of a shipped
    // application is refused like any other's.
    let browser = Shipped::decided().unwrap().the(Role::WebBrowser).wanted();
    assert!(installing(&enabled, &browser, &tool).is_err());

    // Offered, and applied through the same two steps.
    let underway =
        indicator.beginning_on_its_own(looking_for_updates(&enabled, "flathub").unwrap(), noon());
    let offers = offered(&underway, &enabled, "flathub", &tool).unwrap();
    indicator.ended_on_its_own(underway);
    assert_eq!(offers.len(), AFreshMachine::shipped().len());
    for offer in offers {
        let errand = applying(&enabled, &offer, &tool).unwrap();
        assert_eq!(errand.errand(), Errand::UpdatingAnApplication);
        let underway = indicator.beginning_on_its_own(errand, noon());
        apply(&underway, &enabled, offer, &tool).unwrap();
        indicator.ended_on_its_own(underway);
    }
    let every = AFreshMachine::shipped();
    let expected: Vec<String> = every
        .iter()
        .map(|identifier| format!("install {identifier} from flathub"))
        .chain(
            every
                .iter()
                .map(|identifier| format!("update {identifier}")),
        )
        .collect();
    assert_eq!(
        tool.acted(),
        expected,
        "the rented tool was asked for something no person's installation asks"
    );

    for identifier in AFreshMachine::shipped() {
        remove(
            Application::identified(&identifier).unwrap(),
            &tool,
            &mut machine,
            noon(),
        )
        .unwrap();
    }
    assert!(tool.installed.borrow().is_empty());
    assert!(indicator.is_quiet());
}

/// **A person may remove any of them, including the browser.** Removing the
/// browser ends what it and an agent held over it; every other shipped
/// application goes the same way — and a list that tried to mark one as kept is
/// refused rather than read.
#[test]
fn a_person_may_remove_any_of_them_the_browser_included() {
    let tool = AFreshMachine::new();
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();
    installing_everything(&tool, &enabled, &mut indicator);

    let shipped = Shipped::decided().unwrap();
    let browser = shipped.the(Role::WebBrowser).application().clone();
    let mut machine = Agent::present();
    let grants = machine.grants_mut().unwrap();
    grants.grant(
        Grant::checked(
            "@alo",
            Reach::Application(browser.identifier().to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    grants.grant(
        Grant::checked_for(
            &Applicant::named(browser.identifier()).grantee(),
            Reach::Folder(PathBuf::from("/home/anna/Downloads")),
            noon(),
            hour(),
        )
        .unwrap(),
    );

    let removed = remove(browser.clone(), &tool, &mut machine, noon()).unwrap();
    assert_eq!(removed.grants_ended(), 2);
    assert_eq!(machine.grants().unwrap().len(), 0);
    assert!(
        !tool
            .installed
            .borrow()
            .contains(&browser.identifier().to_owned())
    );

    for pinned in shipped
        .every()
        .iter()
        .filter(|p| p.role() != Role::WebBrowser)
    {
        remove(pinned.application().clone(), &tool, &mut machine, noon()).unwrap();
    }
    assert!(tool.installed.borrow().is_empty());

    // Removed is removed: the browser is not put back behind a person's back,
    // and removing it again is refused as for any application.
    assert!(remove(browser, &tool, &mut machine, noon()).is_err());

    // No key keeps one.
    for kept in ["removable = false", "kept = true", "protected = true"] {
        let list = shipped::THE_LIST.replacen(
            "role = \"web-browser\"",
            &format!("role = \"web-browser\"\n{kept}"),
            1,
        );
        assert!(
            matches!(Shipped::read(&list), Err(NotShipped::NotRead { .. })),
            "{kept} was read"
        );
    }
}

// ---------------------------------------------------------------------------
// The terminal is a person's, and never an agent's.
// ---------------------------------------------------------------------------

/// Every verb alo OS ships, on one list as a daemon is handed them — and the
/// list is held to the crates of this workspace that declare verbs, so a verb
/// added in a new crate cannot escape this test by not being handed to it.
fn every_verb_this_machine_ships() -> Verbs {
    let here = the_repository();
    let off_the_disk = move |named: &str| fs::read_to_string(here.join(named)).ok();
    let manifest = fs::read_to_string(the_repository().join(THE_WORKSPACE)).unwrap();
    let mut declaring = whoever_declares_verbs(&manifest, &off_the_disk);
    declaring.sort();
    assert_eq!(
        declaring,
        [
            "alo-adapters",
            "alo-applications",
            "alo-capturing",
            "alo-changing-network",
            "alo-converting",
            "alo-files",
            "alo-finding",
            "alo-measuring",
            "alo-printing",
            "alo-software",
        ],
        "a crate declares verbs that this test does not hand to the terminal's check"
    );

    let mut verbs = Verbs::default();
    alo_converting::verbs::declare_into(&mut verbs).unwrap();
    alo_files::declare_into(&mut verbs).unwrap();
    alo_applications::declare_into(&mut verbs).unwrap();
    alo_finding::verbs::declare_into(&mut verbs).unwrap();
    alo_measuring::verbs::declare_into(&mut verbs).unwrap();
    alo_printing::verbs::declare_into(&mut verbs).unwrap();
    alo_software::verbs::declare_into(&mut verbs).unwrap();
    alo_adapters::verbs::declare_into(&mut verbs).unwrap();
    alo_changing_network::verbs::declare_into(&mut verbs).unwrap();
    alo_capturing::verbs::declare_into(&mut verbs).unwrap();
    verbs
}

/// A value for every argument of this verb, naming `application` wherever it
/// takes one.
fn filling(verb: &Verb, application: &str) -> Vec<(String, Given)> {
    verb.args()
        .iter()
        .map(|arg| {
            let given = match arg.takes() {
                Takes::Application => Given::text(application),
                Takes::Path => Given::text("/home/anna/Invoices/march.pdf"),
                Takes::Name { .. } => Given::text("flathub"),
                Takes::Count { least, .. } => Given::number(*least),
                Takes::Choice(offered) => {
                    Given::text(offered.first().map_or("", |option| option.name()))
                }
            };
            (arg.name().to_owned(), given)
        })
        .collect()
}

/// **The terminal is a person's and not an agent's: no verb reaches it** (ADR
/// 0001 §1, ADR 0043). For every verb alo OS ships that names an application,
/// a call naming the shipped terminal is refused — proposed or read — even on a
/// list holding everything else a person could grant, including a grant over
/// the terminal written in by hand, because the grant itself cannot be made.
/// Beside each refusal, the same verb naming the text editor goes through.
#[test]
fn the_terminal_is_a_persons_and_no_verb_reaches_it() {
    let shipped = Shipped::decided().unwrap();
    let terminal = shipped.the(Role::Terminal).application().identifier();
    let editor = shipped.the(Role::TextEditor).application().identifier();
    let alo = Grantee::named("@alo");

    // The grant cannot be made, for as long as a person might ask.
    for lasting in [Duration::from_secs(1), hour(), hour() * 24 * 365] {
        assert_eq!(
            Grant::checked(
                "@alo",
                Reach::Application(terminal.to_owned()),
                noon(),
                lasting
            )
            .unwrap_err(),
            GrantError::APersonsOwn
        );
    }

    // Everything else a person could grant, and the terminal written in by hand.
    let by_hand = |id: u64, reach: Reach| Held {
        id: GrantId::numbered(id),
        grant: Grant {
            grantee: alo.clone(),
            reach,
            granted_at: noon(),
            expires: noon() + hour(),
        },
    };
    let grants = Grants::remembered(
        vec![
            by_hand(0, Reach::Folder(PathBuf::from("/home/anna"))),
            by_hand(1, Reach::Application(terminal.to_owned())),
            by_hand(2, Reach::Application(editor.to_owned())),
        ],
        3,
    )
    .unwrap();

    let verbs = every_verb_this_machine_ships();
    let naming_applications: Vec<&Verb> = verbs
        .all()
        .filter(|verb| {
            verb.args()
                .iter()
                .any(|arg| matches!(arg.takes(), Takes::Application))
        })
        .collect();
    assert!(
        naming_applications.len() >= 5,
        "the four application verbs and install_application were not all found"
    );

    let wanted = Ask::application(terminal);
    for verb in naming_applications {
        let call_naming = |application: &str| {
            let given = filling(verb, application);
            let given: Vec<(&str, Given)> = given
                .iter()
                .map(|(name, value)| (name.as_str(), value.clone()))
                .collect();
            verbs.call(verb.name(), &given).unwrap()
        };

        let call = call_naming(terminal);
        let refusal = NotGranted::Never {
            agent: "@alo".to_owned(),
            wanted: wanted.clone(),
        };
        assert_eq!(
            call.refusal(&grants, &alo, noon()),
            Some(refusal.clone()),
            "{}",
            verb.name()
        );
        if call.waits_for_approval() {
            assert_eq!(
                Proposal::checked(&call, &alo, &grants, noon(), hour()).unwrap_err(),
                ProposalError::NotGranted(refusal),
                "{} was put to a person naming the terminal",
                verb.name()
            );
            assert!(
                Proposal::checked(&call_naming(editor), &alo, &grants, noon(), hour()).is_ok(),
                "{} could not name the text editor",
                verb.name()
            );
        } else {
            let refused = Authorised::read(&call, &alo, &grants, noon()).unwrap_err();
            assert_eq!(
                refused.why(),
                &NotAuthorised::NotGranted(refusal),
                "{} read the terminal",
                verb.name()
            );
            assert!(Authorised::read(&call_naming(editor), &alo, &grants, noon()).is_ok());
        }
    }

    // The person keeps it: an application a person chose may still be allowed
    // to hand it something, through the same list.
    let mail = Applicant::named("org.example.Mail");
    let mut allowed = Grants::default();
    allowed.grant(
        Grant::checked_for(
            &mail.grantee(),
            Reach::Application(terminal.to_owned()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    assert!(allowed.allowing(&mail, &wanted, noon()).is_ok());
}
