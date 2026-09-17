//! The walk from nothing to a working application, as the sentences a person
//! meets on the way — held to being exactly the table in
//! `docs/autonomy/updates/every-sentence-and-the-walk-to-a-working-application.md`.
//!
//! `docs/autonomy/v0-5-software-and-the-web-plan.md`, task 7: *one walk — set a
//! proxy, install an application, open a web link in the browser, ask the agent
//! to use the reference adapter, remove the application — produces the exact
//! sequence a person meets, recorded as a table and held by one test.*
//!
//! # What *from nothing* is here
//!
//! A person's own machine with no proxy set, no grant made to anybody, and one
//! application on it: the web browser this machine ships
//! (`Shipped::the(Role::WebBrowser)`). The text editor is not on it yet, so
//! installing it is the first thing the person does with the machine's software
//! rather than something the image already did.
//!
//! # Every sentence comes from the crate that decided it, and says which word
//!
//! Nothing in this file writes a sentence. Each row is what a crate's own `said`
//! answered, through the vocabulary the whole machine collects
//! (`alo_saying::everything_this_machine_can_say`), and the key beside it is
//! **measured rather than claimed**: the strings the walk speaks prefer a
//! language in which every phrase is its English with its own key in front
//! ([`keyed`]). The key a row names is the one found at the front of what was
//! said, and the English is what is left once the keys are taken out — so a row
//! whose key is wrong fails as surely as a row whose words are.
//!
//! The rented tool, the application's bus and the browser are stand-ins, as in
//! every other test of these crates; the report says what the walk on a machine
//! would add.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use alo_adapters::{Delivers, Driving, Message, NotDelivered, shipped_adapters};
use alo_applications::Application;
use alo_capability::{
    Agent, Applicant, Approvals, Given, Grant, Grantee, Proposal, Reach, Refused,
};
use alo_egress::Indicator;
use alo_granted::Listing;
use alo_proxy::{
    ConfigurationAddress, Kept, NotEvaluated, ProxyAddress, Reaching, Road, Scheme, SpokenTo,
    TheEvaluator, TheProxy, the_way,
};
use alo_software::{
    Bound, Configured, Enabled, Failed, Role, Shipped, SourceName, Tool, Wanted,
    WhatOpensWebAddresses, install, installing, opened, remove,
};
use alo_strings::{Language, Said, Strings, Translation};

// ---------------------------------------------------------------------------
// The two tables.
// ---------------------------------------------------------------------------

/// What a person meets, in order: the step of the walk, the word, and the
/// sentence exactly as it reads.
const THE_WALK: &[(&str, &str, &str)] = &[
    (
        "Set a proxy",
        "proxy.address.carries-a-password",
        "That proxy address has a name and a password written into it. Type the address on its \
         own, and the name and the password in their own fields — the password is then kept \
         safely and never written into a file",
    ),
    (
        "Set a proxy",
        "proxy.the-proxy.manual",
        "This machine reaches the network through the proxy set here",
    ),
    ("Set a proxy", "proxy.set-by.this-person", "You set this"),
    (
        "Install the text editor",
        "egress.itself.installing-an-application",
        "alo OS is installing an application from dl.flathub.org",
    ),
    (
        "Install the text editor",
        "software.installed",
        "org.gnome.TextEditor is installed. It has been given nothing: it asks when it needs a \
         file, the camera or anything else, and you answer",
    ),
    (
        "Install the text editor",
        "granted.nothing-granted",
        "Nothing is granted right now. No agent and no application has been granted anything on \
         this machine, and there is nothing here to revoke",
    ),
    (
        "Open a web link",
        "software.web.refused.nothing-granted",
        "org.gnome.TextEditor has not been allowed anything on this machine, so the web address \
         was not opened. Allow it the web browser you want it to open addresses in, and it can \
         ask again",
    ),
    (
        "Open a web link",
        "granted.one-grant",
        "org.gnome.TextEditor has been granted the application org.mozilla.firefox",
    ),
    (
        "Open a web link",
        "software.web.opens.shipped",
        "org.mozilla.firefox opens web addresses. It came with this machine, and you can choose \
         another or remove it",
    ),
    (
        "Ask the agent to use the text editor",
        "granted.one-grant",
        "org.gnome.TextEditor has been granted the application org.mozilla.firefox",
    ),
    (
        "Ask the agent to use the text editor",
        "granted.one-grant",
        "@alo has been granted the application org.gnome.TextEditor",
    ),
    (
        "Ask the agent to use the text editor",
        "granted.one-grant",
        "@alo has been granted /home/anna/Notes and everything in it",
    ),
    (
        "Ask the agent to use the text editor",
        "adapters.text-editor.open-document.sentence",
        "open /home/anna/Notes/March.txt in GNOME Text Editor",
    ),
    (
        "Remove the text editor",
        "software.removed",
        "org.gnome.TextEditor is removed. Everything it was allowed has ended, and so has every \
         permission to use it",
    ),
    (
        "Remove the text editor",
        "granted.one-grant",
        "@alo has been granted /home/anna/Notes and everything in it",
    ),
];

/// The same walk, stopped at each step by the refusal a person could meet there.
const WHERE_IT_STOPS: &[(&str, &str, &str)] = &[
    (
        "Set a proxy",
        "proxy.the-proxy.automatic",
        "This machine asks the network where each connection should go",
    ),
    (
        "Set a proxy",
        "proxy.set-by.an-organisation",
        "Your organisation set this",
    ),
    (
        "Set a proxy",
        "proxy.not-changed.an-organisation-set-it",
        "Your organisation set the proxy for this machine, so it cannot be changed here. Ask \
         whoever manages it",
    ),
    (
        "Install the text editor",
        "egress.itself.installing-an-application",
        "alo OS is installing an application from dl.flathub.org",
    ),
    (
        "Install the text editor",
        "proxy.refused.nothing-works-it-out",
        "This machine cannot read the rule this network publishes, so nothing was sent. Ask \
         whoever runs this network for the address of its proxy",
    ),
    (
        "Open a web link",
        "software.removed",
        "org.mozilla.firefox is removed. Everything it was allowed has ended, and so has every \
         permission to use it",
    ),
    (
        "Open a web link",
        "software.web.nothing-opens",
        "No application on this machine opens web addresses, so nothing was opened. Install a \
         web browser, and it opens them",
    ),
    (
        "Ask the agent to use the text editor",
        "software.removed",
        "org.gnome.TextEditor is removed. Everything it was allowed has ended, and so has every \
         permission to use it",
    ),
    (
        "Ask the agent to use the text editor",
        "capability.refused.never-granted",
        "@alo has not been granted the application org.gnome.TextEditor — grants are made by \
         picking a folder, never by asking for one",
    ),
    (
        "Remove the text editor",
        "software.refused.not-installed",
        "org.gnome.TextEditor is not installed on this machine, so there is nothing to update or \
         remove. Check the application's name",
    ),
];

// ---------------------------------------------------------------------------
// Words, with the key each came from.
// ---------------------------------------------------------------------------

/// What opens a key at the front of a sentence in [`keyed`].
const OPENS: &str = "⟦";

/// What closes it, with the blank after it.
const CLOSES: &str = "⟧ ";

/// The whole machine's vocabulary, spoken in a language in which each phrase is
/// its own English with its key in front.
///
/// A sentence filled from another sentence — a place named inside an
/// indicator's line — carries that key too, inside it; the row's key is the one
/// at the very front, and the English is the text with every key taken out.
fn keyed() -> Strings {
    let vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    let marking = Language::written("qaa").unwrap();
    let mut translation = Translation::into_language(marking.clone());
    for phrase in vocabulary.phrases() {
        translation = translation.says(
            phrase.key().clone(),
            format!(
                "{OPENS}{}{CLOSES}{}",
                phrase.key(),
                phrase.source().as_written()
            ),
        );
    }
    let speaking = vocabulary.check(translation).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[marking]);
    strings
}

/// One row: the key at the front of what was said, and its English.
fn heard(step: &'static str, said: &Said) -> (&'static str, String, String) {
    assert!(!said.is_a_bug(), "{step}: {said}");
    assert!(said.unfilled().is_empty(), "{step}: {said}");
    let text = said.text();
    let Some(rest) = text.strip_prefix(OPENS) else {
        panic!("{step}: {text:?} came from no phrase of the vocabulary");
    };
    let (key, _) = rest.split_once(CLOSES).unwrap();
    let mut english = String::new();
    let mut left = text;
    while let Some(at) = left.find(OPENS) {
        english.push_str(left.get(..at).unwrap());
        let after = left.get(at..).unwrap();
        let ends = after.find(CLOSES).unwrap() + CLOSES.len();
        left = after.get(ends..).unwrap();
    }
    english.push_str(left);
    (step, key.to_owned(), english)
}

/// Every row, in the order a person met them.
#[derive(Default)]
struct Met(Vec<(&'static str, String, String)>);

impl Met {
    fn said(&mut self, step: &'static str, said: &Said) {
        self.0.push(heard(step, said));
    }

    /// The rows against a table, row by row, so a failure names the first one
    /// that differs rather than two walls of text.
    fn are(&self, table: &[(&str, &str, &str)]) {
        for (at, (met, written)) in self.0.iter().zip(table).enumerate() {
            let met = (met.0, met.1.as_str(), met.2.as_str());
            assert_eq!(met, *written, "row {} differs", at + 1);
        }
        assert_eq!(
            self.0.len(),
            table.len(),
            "the walk said {} sentences and the table holds {}: {:#?}",
            self.0.len(),
            table.len(),
            self.0
        );
    }
}

// ---------------------------------------------------------------------------
// The machine the walk is taken on.
// ---------------------------------------------------------------------------

/// Twelve o'clock, and every step a minute after the last.
fn at(minute: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000 + minute * 60)
}

/// How long a grant a person makes in the walk stands.
fn a_day() -> Duration {
    Duration::from_secs(24 * 60 * 60)
}

/// The text editor, whose adapter the agent uses.
const THE_TEXT_EDITOR: &str = "org.gnome.TextEditor";

/// The document the person asks the agent to open.
const THE_DOCUMENT: &str = "/home/anna/Notes/March.txt";

/// The rented tool, standing in: the place a fresh machine installs from, what
/// is installed, and every act it was asked to carry out.
struct AMachine {
    installed: RefCell<Vec<String>>,
    acts: RefCell<Vec<String>>,
}

impl AMachine {
    /// A machine with only the browser this machine ships on it.
    fn with_only_the_browser() -> Self {
        let browser = Shipped::decided()
            .unwrap()
            .the(Role::WebBrowser)
            .application()
            .identifier()
            .to_owned();
        Self {
            installed: RefCell::new(vec![browser]),
            acts: RefCell::new(Vec::new()),
        }
    }

    /// What is installed, as `alo-applications` keeps it.
    fn installed_here(&self) -> alo_applications::Installed {
        alo_applications::Installed::holding(
            self.installed
                .borrow()
                .iter()
                .map(|identifier| Application::identified(identifier).unwrap()),
        )
    }
}

impl Tool for AMachine {
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
        self.installed
            .borrow_mut()
            .push(application.identifier().to_owned());
        Ok(())
    }

    fn updates(&self, _: &SourceName) -> Result<Vec<String>, Failed> {
        Ok(Vec::new())
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

/// The text editor on the person's session, standing in: it keeps every message
/// it is handed and answers.
#[derive(Default)]
struct TheEditorsBus {
    handed: RefCell<Vec<Message>>,
}

impl Delivers for TheEditorsBus {
    fn deliver(&self, message: &Message) -> Result<(), NotDelivered> {
        self.handed.borrow_mut().push(message.clone());
        Ok(())
    }
}

/// A network's rule, as a machine with nothing able to read one answers.
struct NothingReadsTheRule;

impl TheEvaluator for NothingReadsTheRule {
    fn asked(&self, _: &ConfigurationAddress, _: &str, _: &str) -> Result<String, NotEvaluated> {
        Err(NotEvaluated::NothingEvaluatesIt)
    }
}

/// A walk with a manual proxy set asks no network's rule.
struct NeverAsked;

impl TheEvaluator for NeverAsked {
    fn asked(
        &self,
        _: &ConfigurationAddress,
        address: &str,
        _: &str,
    ) -> Result<String, NotEvaluated> {
        panic!("nothing should have asked a network's rule about {address}")
    }
}

/// The company's proxy, typed as the person was handed it.
fn the_companys_proxy() -> ProxyAddress {
    ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 8080).unwrap()
}

/// Where applications come from, as the road out reaches it.
fn the_place_applications_come_from() -> Reaching {
    Reaching::over(Scheme::Https, "dl.flathub.org").unwrap()
}

/// The agent the person talks to.
fn the_agent() -> Grantee {
    Grantee::named("@alo")
}

// ---------------------------------------------------------------------------
// The walk.
// ---------------------------------------------------------------------------

/// **The walk from nothing to a working application produces exactly the
/// table**, in order — and each step does what its sentence says it did.
#[test]
fn the_walk_from_nothing_to_a_working_application_is_the_table() {
    let strings = keyed();
    let mut met = Met::default();
    let tool = AMachine::with_only_the_browser();
    let mut machine = Agent::present();
    let mut indicator = Indicator::default();
    let shipped = Shipped::decided().unwrap();

    // 1. Set a proxy. The address is pasted as the company's message wrote it,
    //    with a name and a password in it, and refused; then typed on its own.
    let pasted =
        ProxyAddress::checked(SpokenTo::Http, "anna:hunter2@proxy.example.com", 8080).unwrap_err();
    met.said("Set a proxy", &pasted.said(&strings));
    let kept = Kept::by_this_person(TheProxy::None)
        .changed_by_the_person(TheProxy::one(the_companys_proxy()))
        .unwrap();
    met.said("Set a proxy", &kept.proxy().said(&strings));
    met.said("Set a proxy", &kept.set_by().said(&strings));

    // 2. Install the text editor, by hand, from the place a fresh machine
    //    installs from. The line on the indicator names where it goes — not the
    //    proxy it goes through.
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let wanted = Wanted::by_hand(Application::identified(THE_TEXT_EDITOR).unwrap(), "flathub");
    let errand = installing(&enabled, &wanted, &tool).unwrap();
    let underway = indicator.beginning_on_its_own(errand, at(1));
    let line = indicator.showing().first().unwrap().said(&strings);
    met.said("Install the text editor", &line);
    let way = the_way(
        kept.proxy(),
        Road::InstallingAnApplication,
        &the_place_applications_come_from(),
        &NeverAsked,
    )
    .unwrap();
    assert_eq!(way.through(), Some(&the_companys_proxy()));
    assert!(line.text().contains("dl.flathub.org"), "{line}");
    assert!(!line.text().contains("proxy.example.com"), "{line}");
    let installed = install(&underway, &enabled, wanted, &tool).unwrap();
    indicator.ended_on_its_own(underway);
    assert!(indicator.is_quiet());
    met.said("Install the text editor", &installed.said(&strings));
    // Nothing is granted, and that is a sentence rather than an empty list.
    let listing = Listing::of(machine.allowed(), at(2));
    met.said("Install the text editor", &listing.said(&strings).unwrap());

    // 3. Open a web link in the browser, from the text editor. It was given
    //    nothing, so it is refused and told what to allow; the person allows it
    //    the browser, and the link opens there.
    let installed_here = tool.installed_here();
    let what_opens = WhatOpensWebAddresses::on(&installed_here, &shipped, None);
    let refused = opened(
        THE_TEXT_EDITOR,
        "https://example.com/march",
        &what_opens,
        machine.allowed(),
        at(3),
    )
    .unwrap_err();
    met.said("Open a web link", &refused.said(&strings));
    let browser = shipped.the(Role::WebBrowser).application().identifier();
    machine.grants_mut().unwrap().grant(
        Grant::checked_for(
            &Applicant::named(THE_TEXT_EDITOR).grantee(),
            Reach::Application(browser.to_owned()),
            at(4),
            a_day(),
        )
        .unwrap(),
    );
    for row in Listing::of(machine.allowed(), at(4)).rows() {
        met.said("Open a web link", &row.said(&strings));
    }
    let opened_it = opened(
        THE_TEXT_EDITOR,
        "https://example.com/march",
        &what_opens,
        machine.allowed(),
        at(5),
    )
    .unwrap();
    assert_eq!(opened_it.browser().application().identifier(), browser);
    met.said("Open a web link", &opened_it.browser().said(&strings));

    // 4. Ask the agent to use the reference adapter: the person lets it use the
    //    text editor and the notes folder, it proposes one sentence, the person
    //    approves it, and the document is sent once.
    for reach in [
        Reach::Application(THE_TEXT_EDITOR.to_owned()),
        Reach::Folder(PathBuf::from("/home/anna/Notes")),
    ] {
        machine
            .grants_mut()
            .unwrap()
            .grant(Grant::checked("@alo", reach, at(6), a_day()).unwrap());
    }
    for row in Listing::of(machine.allowed(), at(6)).rows() {
        met.said("Ask the agent to use the text editor", &row.said(&strings));
    }
    let adapters = shipped_adapters().unwrap();
    let offered = adapters
        .offered_to(&the_agent(), machine.allowed(), at(7))
        .unwrap();
    let call = offered
        .call(
            "text_editor.open_document",
            &[("document", Given::text(THE_DOCUMENT))],
        )
        .unwrap();
    let mut approvals = Approvals::default();
    let proposal =
        Proposal::checked(&call, &the_agent(), machine.allowed(), at(7), a_day()).unwrap();
    met.said(
        "Ask the agent to use the text editor",
        &proposal.sentence(&strings),
    );
    let id = approvals.propose(proposal);
    let authorised = approvals
        .approve(id, at(8))
        .unwrap()
        .redeem(machine.allowed(), at(8))
        .unwrap();
    let bus = TheEditorsBus::default();
    let driven = Driving::of(
        authorised,
        &adapters,
        machine.allowed(),
        &tool.installed_here(),
        &strings,
    )
    .unwrap()
    .deliver(&bus, &strings)
    .unwrap();
    assert!(
        driven.said(&strings).is_none(),
        "an application that answered is told nothing beyond what was approved"
    );
    assert_eq!(bus.handed.borrow().len(), 1, "one approval, one message");

    // 5. Remove the text editor. Nothing leaves, so nothing is shown; what it
    //    was allowed and what was allowed over it end in the same act, and the
    //    one list keeps only the notes folder.
    let removed = remove(
        Application::identified(THE_TEXT_EDITOR).unwrap(),
        &tool,
        &mut machine,
        at(9),
    )
    .unwrap();
    assert!(
        indicator.is_quiet(),
        "removing puts nothing on the indicator"
    );
    assert_eq!(removed.grants_ended(), 2);
    met.said("Remove the text editor", &removed.said(&strings));
    for row in Listing::of(machine.allowed(), at(9)).rows() {
        met.said("Remove the text editor", &row.said(&strings));
    }
    assert!(
        adapters
            .offered_to(&the_agent(), machine.allowed(), at(10))
            .unwrap()
            .of("text_editor.open_document")
            .is_none(),
        "the agent is offered nothing in an application that is gone"
    );

    assert_eq!(
        *tool.acts.borrow(),
        [
            "install org.gnome.TextEditor from flathub",
            "remove org.gnome.TextEditor"
        ]
    );
    met.are(THE_WALK);
}

/// **Where the walk stops, it stops in words, and nothing past the refusal
/// happens** — one refusal a person can meet at each step, each shown to have
/// reached nothing.
#[test]
fn where_the_walk_stops_is_the_table_and_nothing_past_it_happens() {
    let strings = keyed();
    let mut met = Met::default();
    let tool = AMachine::with_only_the_browser();
    let mut machine = Agent::present();
    let mut indicator = Indicator::default();
    let shipped = Shipped::decided().unwrap();

    // 1. On a machine an organisation manages, the person cannot change the
    //    proxy, and is told who can.
    let managed = Kept::by_an_organisation(TheProxy::Automatic {
        at: ConfigurationAddress::checked("http://wpad.example.com/proxy.config").unwrap(),
    });
    met.said("Set a proxy", &managed.proxy().said(&strings));
    met.said("Set a proxy", &managed.set_by().said(&strings));
    let refused = managed
        .clone()
        .changed_by_the_person(TheProxy::None)
        .unwrap_err();
    met.said("Set a proxy", &refused.said(&strings));

    // 2. The network's rule cannot be read on this machine, so the installation
    //    is refused while its line is showing, and the tool is never asked.
    let enabled = Enabled::read(&tool, Bound::Nobodys).unwrap();
    let wanted = Wanted::by_hand(Application::identified(THE_TEXT_EDITOR).unwrap(), "flathub");
    let errand = installing(&enabled, &wanted, &tool).unwrap();
    let underway = indicator.beginning_on_its_own(errand, at(1));
    met.said(
        "Install the text editor",
        &indicator.showing().first().unwrap().said(&strings),
    );
    let not_taken = the_way(
        managed.proxy(),
        Road::InstallingAnApplication,
        &the_place_applications_come_from(),
        &NothingReadsTheRule,
    )
    .unwrap_err();
    indicator.ended_on_its_own(underway);
    met.said("Install the text editor", &not_taken.said(&strings));
    assert!(
        tool.acts.borrow().is_empty(),
        "nothing was sent to the tool"
    );

    // 3. With the browser removed, a link opens in nothing, and says so.
    let the_browser = shipped.the(Role::WebBrowser).application().clone();
    met.said(
        "Open a web link",
        &remove(the_browser, &tool, &mut machine, at(2))
            .unwrap()
            .said(&strings),
    );
    tool.install(
        &SourceName::checked("flathub").unwrap(),
        &Application::identified(THE_TEXT_EDITOR).unwrap(),
    )
    .unwrap();
    let installed_here = tool.installed_here();
    let what_opens = WhatOpensWebAddresses::on(&installed_here, &shipped, None);
    let nothing = what_opens.what_opens_them().unwrap_err();
    met.said("Open a web link", &nothing.said(&strings));

    // 4. The person approved opening the document, and removed the text editor
    //    before it was sent: the approval is refused at the moment it would run,
    //    and the application is sent nothing.
    for reach in [
        Reach::Application(THE_TEXT_EDITOR.to_owned()),
        Reach::Folder(PathBuf::from("/home/anna/Notes")),
    ] {
        machine
            .grants_mut()
            .unwrap()
            .grant(Grant::checked("@alo", reach, at(3), a_day()).unwrap());
    }
    let adapters = shipped_adapters().unwrap();
    let call = adapters
        .offered_to(&the_agent(), machine.allowed(), at(4))
        .unwrap()
        .call(
            "text_editor.open_document",
            &[("document", Given::text(THE_DOCUMENT))],
        )
        .unwrap();
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(&call, &the_agent(), machine.allowed(), at(4), a_day()).unwrap(),
    );
    let authorised = approvals
        .approve(id, at(5))
        .unwrap()
        .redeem(machine.allowed(), at(5))
        .unwrap();
    let removed = remove(
        Application::identified(THE_TEXT_EDITOR).unwrap(),
        &tool,
        &mut machine,
        at(6),
    )
    .unwrap();
    met.said(
        "Ask the agent to use the text editor",
        &removed.said(&strings),
    );
    let refused: Refused = Driving::of(
        authorised,
        &adapters,
        machine.allowed(),
        &tool.installed_here(),
        &strings,
    )
    .unwrap_err();
    met.said(
        "Ask the agent to use the text editor",
        &refused.said(&strings),
    );

    // 5. Removing it again removes nothing, and says what to check.
    let again = remove(
        Application::identified(THE_TEXT_EDITOR).unwrap(),
        &tool,
        &mut machine,
        at(7),
    )
    .unwrap_err();
    met.said("Remove the text editor", &again.said(&strings));

    assert!(indicator.is_quiet());
    met.are(WHERE_IT_STOPS);
}
