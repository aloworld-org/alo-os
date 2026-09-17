//! An installation from a place an organisation's description does not name,
//! refused in the words naming the organisation — beside the same installation
//! on a machine whose description names no places at all.
//!
//! The acceptance for reading `[applications]`, and every step but the rented
//! tool is the production one: the description is read by the same
//! `crate::describing::read` `Described::at` uses, as a file root wrote (which
//! is what makes a rule an organisation's, and which a test process that is not
//! root cannot put on a disk — `tests/what_a_machine_says_about_itself.rs` reads
//! the section off a real disk under whichever owner the suite runs as); the
//! bound it produces is handed to `alo_software::Enabled::read` exactly as it
//! comes back; the installation is `alo_software`'s own two steps around the
//! machine's one indicator; and the refusal is rendered in this machine's whole
//! vocabulary, the one `crate::starting` loads. The tool is a list in memory,
//! because what is asked here is whether anything reached it.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::Path;
use std::time::{Duration, SystemTime};

use alo_applications::Application;
use alo_egress::Indicator;
use alo_software::{
    Bound, Configured, Enabled, Failed, NotDone, SetBy, SourceName, Tool, Wanted, install,
    installing,
};

use crate::described::Described;
use crate::describing::read;
use crate::trusting::WhoDescribedIt;

/// A machine that has two places set up to install from, and remembers every
/// installation it was asked for.
#[derive(Default)]
struct TwoPlaces {
    /// Every application installed, by identifier.
    installed: RefCell<Vec<String>>,
}

impl Tool for TwoPlaces {
    fn sources(&self) -> Result<Vec<Configured>, Failed> {
        Ok(vec![
            Configured {
                name: "flathub".to_owned(),
                address: "https://dl.flathub.org/repo/".to_owned(),
                checks_signatures: true,
                switched_off: false,
            },
            Configured {
                name: "acme-apps".to_owned(),
                address: "https://apps.acme.example/repo".to_owned(),
                checks_signatures: true,
                switched_off: false,
            },
        ])
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

/// A description of an ordinary machine in the newest shape, with whatever
/// else is written after it.
fn a_description_with(after: &str) -> String {
    format!(
        r#"format = 3

[logins]
person = 1000
agent = 989
group = 989

[agent]
name = "alo"
turn-seconds = 900
proposal-seconds = 300

[record]
path = "/var/lib/alo/record"
keeping = "forever"
{after}"#
    )
}

/// That description, read as root wrote it.
fn written_by_an_administrator(said: &str) -> Described {
    read(
        said,
        Path::new("/etc/alo/agentd.toml"),
        WhoDescribedIt::AnAdministrator,
    )
    .unwrap()
}

/// The text editor, from Flathub, asked for by hand.
fn the_text_editor_from_flathub() -> Wanted {
    Wanted::by_hand(
        Application::identified("org.gnome.TextEditor").unwrap(),
        "flathub",
    )
}

/// A moment to put a line on the indicator at.
fn now() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// **A place the organisation's description does not name is refused, in the
/// words naming the organisation, and nothing leaves or is installed** — and
/// the same installation, on a machine whose description has no
/// `[applications]`, goes ahead.
#[test]
fn a_place_the_organisations_description_does_not_name_is_refused_in_its_words() {
    let loaded = crate::starting::what_this_machine_says().unwrap();
    let strings = loaded.strings();

    // The organisation's machine: only its own place is permitted.
    let managed = written_by_an_administrator(&a_description_with(
        "\n[applications]\nmay-come-from = [\"acme-apps\"]\n",
    ));
    let tool = TwoPlaces::default();
    let enabled = Enabled::read(&tool, managed.applications().clone()).unwrap();
    let mut indicator = Indicator::default();

    let refused = installing(&enabled, &the_text_editor_from_flathub(), &tool).unwrap_err();
    assert_eq!(
        refused,
        NotDone::OutsideTheBound {
            source: "flathub".to_owned(),
            set_by: SetBy::AnAdministrator,
        }
    );
    let said = refused.said(strings);
    assert!(!said.is_a_bug(), "{said}");
    assert!(
        said.text()
            .starts_with("The organisation that manages this machine does not permit"),
        "{said}"
    );
    assert!(said.text().contains("flathub"), "{said}");
    assert!(
        indicator.is_quiet(),
        "a refused installation left the machine"
    );
    assert!(
        tool.installed.borrow().is_empty(),
        "it was installed anyway"
    );

    // The place it does name is installed from on the same machine.
    let from_acme = Wanted::by_hand(
        Application::identified("org.gnome.TextEditor").unwrap(),
        "acme-apps",
    );
    let errand = installing(&enabled, &from_acme, &tool).unwrap();
    let underway = indicator.beginning_on_its_own(errand, now());
    install(&underway, &enabled, from_acme, &tool).unwrap();
    indicator.ended_on_its_own(underway);
    assert_eq!(*tool.installed.borrow(), ["org.gnome.TextEditor"]);

    // The unmanaged machine: the same description with no section in it.
    let unmanaged = written_by_an_administrator(&a_description_with(""));
    assert_eq!(unmanaged.applications(), &Bound::Nobodys);
    let tool = TwoPlaces::default();
    let enabled = Enabled::read(&tool, unmanaged.applications().clone()).unwrap();
    let mut indicator = Indicator::default();

    let errand = installing(&enabled, &the_text_editor_from_flathub(), &tool).unwrap();
    let underway = indicator.beginning_on_its_own(errand, now());
    assert!(
        !indicator.is_quiet(),
        "the installation was not on the indicator"
    );
    let installed = install(&underway, &enabled, the_text_editor_from_flathub(), &tool).unwrap();
    indicator.ended_on_its_own(underway);
    assert_eq!(installed.application().identifier(), "org.gnome.TextEditor");
    assert_eq!(*tool.installed.borrow(), ["org.gnome.TextEditor"]);
}

/// **The same list, in the person's own description, is refused in words that
/// name no organisation**, because nobody else set it.
#[test]
fn the_same_list_in_the_persons_own_description_names_no_organisation() {
    let loaded = crate::starting::what_this_machine_says().unwrap();
    let theirs = read(
        &a_description_with("\n[applications]\nmay-come-from = [\"acme-apps\"]\n"),
        Path::new("/etc/alo/agentd.toml"),
        WhoDescribedIt::ThePerson,
    )
    .unwrap();
    let tool = TwoPlaces::default();
    let enabled = Enabled::read(&tool, theirs.applications().clone()).unwrap();

    let refused = installing(&enabled, &the_text_editor_from_flathub(), &tool).unwrap_err();
    assert_eq!(
        refused,
        NotDone::OutsideTheBound {
            source: "flathub".to_owned(),
            set_by: SetBy::ThisMachine,
        }
    );
    let said = refused.said(loaded.strings());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("No organisation set this"), "{said}");
    assert!(tool.installed.borrow().is_empty());
}
