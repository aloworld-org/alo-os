//! A change to the network, from the moment an agent proposes it — or a person
//! chooses it in Settings — through the privileged broker's real door, to the
//! network manager's side, and every way it is stopped on the way.
//!
//! Task 3 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`:
//!
//! - *the broker's network verbs — join a network the machine can see, forget a
//!   network, turn the radio on or off, set the proxy `alo-proxy` holds — take
//!   closed types (a network by the identity the rented network manager
//!   reported, never a typed name);*
//! - *a network change that would cut a turn's own connection says so before it
//!   is approved.*
//!
//! Everything here is real except two things, and both are said: the network
//! manager is a stand-in that reports what the test gives it and remembers what
//! it was asked to change (`alo-networks` holds the real client to the network
//! manager's own interface on a bus), and the door is handed to whoever runs
//! the test rather than to the person's login. The broker, its key hand-over,
//! its socket, its record, its carriers and this crate's road are the ones a
//! machine runs.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, SystemTime};

use alo_broker::handing_over::hand_over_a_fresh_key;
use alo_broker::listening::Listening;
use alo_broker::{BY_HAND, Broker, Door, Identity, Switch, SystemVerb, our_group, our_user};
use alo_brokerd::{Carriers, Network, Proxy, Storage};
use alo_capability::{Approvals, Authorised, Given, Grantee, Grants, NotAuthorised, Proposal};
use alo_changing_network::words::{NETWORK, THIS_CONVERSATION, WIRELESS};
use alo_changing_network::{
    Answered, Change, JOIN_NETWORK, Listed, NotChanged, SWITCH_WIRELESS, TheBroker,
    carry_out_approved, carry_out_by_hand, changed_said, network_verbs, proposable,
    set_proxy_by_hand,
};
use alo_drives::{Drive, DriveService, Drives, Filesystem, LoginName, TheDrives};
use alo_networks::proxy_file::{kept_on_this_machine, machines};
use alo_networks::{
    NetworkName, NetworkService, Networks, NotAnswering, NotDone, Primary, Protection, Saved,
    TheNetworks, Visible,
};
use alo_proxy::{
    Kept, Password, ProxyAddress, SpokenTo, THE_PERSONS_PROXY_PASSWORD, TheProxy,
    WhereThePasswordIs,
};
use alo_record::{AtTheBroker, Happened, Record};
use alo_strings::{Strings, Vocabulary};

/// A network manager standing in for the machine's: it reports what it is
/// given, and remembers what it was asked to change.
#[derive(Debug, Clone)]
struct Standing {
    /// What it reports.
    networks: Arc<Mutex<TheNetworks>>,
    /// What it was asked to change.
    asked: Arc<Mutex<Vec<String>>>,
}

impl Standing {
    /// Reporting this.
    fn reporting(networks: TheNetworks) -> Self {
        Self {
            networks: Arc::new(Mutex::new(networks)),
            asked: Arc::default(),
        }
    }

    /// What it was asked to change.
    fn asked(&self) -> Vec<String> {
        self.asked.lock().unwrap().clone()
    }
}

impl Networks for Standing {
    fn now(&self) -> Result<TheNetworks, NotAnswering> {
        Ok(self.networks.lock().unwrap().clone())
    }
}

impl NetworkService for Standing {
    fn join(&self, network: &Visible) -> Result<(), NotDone> {
        self.asked
            .lock()
            .unwrap()
            .push(format!("join {}", network.name().called().unwrap_or("?")));
        Ok(())
    }

    fn forget(&self, network: &Saved) -> Result<(), NotDone> {
        self.asked
            .lock()
            .unwrap()
            .push(format!("forget {}", network.uuid()));
        Ok(())
    }

    fn switch_wireless(&self, on: bool) -> Result<(), NotDone> {
        self.asked.lock().unwrap().push(format!("wireless {on}"));
        Ok(())
    }
}

/// A name.
fn named(name: &str) -> NetworkName {
    NetworkName::announced(name.as_bytes()).unwrap()
}

/// A flat, sending through its saved Home network over Wi-Fi, with a café in
/// range and a twin calling itself Home.
fn a_flat_on_wifi() -> TheNetworks {
    TheNetworks {
        visible: vec![
            Visible::reported(named("Home"), Protection::Password, 80),
            Visible::reported(named("Cafe"), Protection::Open, 40),
            Visible::reported(named("Library"), Protection::Open, 30),
            Visible::reported(named("Library"), Protection::Password, 35),
        ],
        saved: vec![Saved::reported(named("Home"), "home-uuid")],
        wireless_on: true,
        primary: Some(Primary::reported(true, "home-uuid")),
    }
}

/// The same flat with a cable plugged in.
fn a_flat_on_a_cable() -> TheNetworks {
    TheNetworks {
        primary: Some(Primary::reported(false, "cable-uuid")),
        ..a_flat_on_wifi()
    }
}

/// The words, as a shell holds them.
fn in_english() -> Strings {
    let mut vocabulary = Vocabulary::empty();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    alo_proxy::declare_into(&mut vocabulary).unwrap();
    alo_changing_network::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// The agent a proposal is made under.
fn the_agent() -> Grantee {
    Grantee::named("alo")
}

/// A disk service that reports no drives, and fails the test if a network
/// change ever reaches a drive.
#[derive(Debug)]
struct NoDrives;

impl Drives for NoDrives {
    fn now(&self) -> Result<TheDrives, alo_drives::NotAnswering> {
        Ok(TheDrives::default())
    }
}

impl DriveService for NoDrives {
    fn mount(&self, _: &Drive, _: &Filesystem, _: &LoginName) -> Result<(), alo_drives::NotDone> {
        panic!("a network change mounted a drive")
    }

    fn eject(&self, _: &Drive) -> Result<(), alo_drives::NotDone> {
        panic!("a network change ejected a drive")
    }
}

/// A group this test may hand a file to that is not root's.
fn a_group() -> u32 {
    if our_group() == 0 { 4242 } else { our_group() }
}

/// A broker of the test's own, answering `how_many` requests at a real door.
struct Running {
    /// Where its door and key are.
    reached: TheBroker,
    /// The machine's proxy file.
    machines_proxy: PathBuf,
    /// The machine's credential store.
    store: PathBuf,
    /// Where a person hands a proxy over.
    wanted: PathBuf,
    /// Where a person hands its password over.
    wanted_password: PathBuf,
    /// Where its approving key is, for a test that plants another.
    key: PathBuf,
    /// The broker's thread, which hands back what it wrote down.
    answering: JoinHandle<Record>,
}

/// Start a broker in a directory of its own, carrying the network's verbs out
/// against `service`.
fn a_broker(what: &str, service: &Standing, how_many: usize) -> Running {
    let here = std::env::temp_dir().join(format!(
        "alo-changing-network-{}-{what}",
        std::process::id()
    ));
    drop(std::fs::remove_dir_all(&here));
    std::fs::create_dir_all(here.join("wanted")).unwrap();
    let (door, key) = (here.join("door.sock"), here.join("approving.key"));
    let wanted = here.join("wanted").join("proxy.json");
    let wanted_password = here.join("wanted").join("proxy-password");
    let machines_proxy = here.join("proxy.json");
    let store = here.join("credstore.encrypted");
    let approving = hand_over_a_fresh_key(&key, a_group()).unwrap();
    let listening = Listening::at(&door, a_group()).unwrap();
    let carriers = Carriers::of(
        Network::against(service.clone()),
        Proxy::handed_over(
            &wanted,
            &wanted_password,
            &machines_proxy,
            alo_proxy::TheMachinesCredentials::at(&store, &the_tool()),
            our_user(),
        ),
        Storage::against(NoDrives, &here.join("passwd"), our_user()),
    );
    let answering = std::thread::spawn(move || {
        let mut broker = Broker::new(
            Door::handed_to(our_user()),
            approving,
            Record::default(),
            carriers,
        );
        for _ in 0..how_many {
            listening.answer_one(&mut broker).unwrap();
        }
        broker.recording().clone()
    });
    Running {
        reached: TheBroker::at(&door, &key, &wanted, &machines_proxy, our_user()),
        machines_proxy,
        store,
        wanted,
        wanted_password,
        key,
        answering,
    }
}

/// What the broker wrote down: each entry's approval and refusal.
fn kept(record: &Record) -> Vec<(Option<u64>, Option<AtTheBroker>)> {
    record
        .everything()
        .map(|entry| match entry.happened() {
            Happened::Brokered {
                from_approval,
                refused,
                ..
            } => (*from_approval, *refused),
            other => panic!("the broker wrote down something else: {other:?}"),
        })
        .collect()
}

/// The call an agent makes.
fn called(verb: &str, first: (&str, &str), this_conversation: &str) -> alo_capability::Call {
    network_verbs()
        .unwrap()
        .call(
            verb,
            &[
                (first.0, Given::text(first.1)),
                (THIS_CONVERSATION, Given::text(this_conversation)),
            ],
        )
        .unwrap()
}

/// Propose a call, approve it, and redeem the approval: the authority a turn
/// hands on, and the approval's number.
fn approved(call: &alo_capability::Call) -> (Authorised, u64) {
    let grants = Grants::default();
    let now = SystemTime::now();
    let mut approvals = Approvals::default();
    let id = approvals.propose(
        Proposal::checked(call, &the_agent(), &grants, now, Duration::from_secs(300)).unwrap(),
    );
    let authorised = approvals
        .approve(id, now)
        .unwrap()
        .redeem(&grants, now)
        .unwrap();
    (authorised, id.as_u64())
}

/// **An agent's proposal, approved, joins exactly the network the person
/// approved by name — through the broker, once — and the sentence they
/// approved said the conversation would lose its connection, because it
/// would.** The agent's own call is a change that waits and reaches nothing;
/// the approved one is found, crosses the door, is written down under the same
/// approval number, and the carrier finds the network again and joins that one.
#[test]
fn an_approved_proposal_joins_exactly_that_network_and_said_what_it_cuts() {
    let service = Standing::reporting(a_flat_on_wifi());
    let strings = in_english();
    let call = called(JOIN_NETWORK, (NETWORK, "Cafe"), "loses_its_connection");
    assert_eq!(
        call.sentence(&strings).text(),
        "join the Wi-Fi network Cafe, and this conversation loses its connection until this \
         machine is connected again"
    );
    assert_eq!(
        proposable(&call, &service, Answered::OverTheNetwork),
        Ok(())
    );
    let by_itself =
        Authorised::read(&call, &the_agent(), &Grants::default(), SystemTime::now()).unwrap_err();
    assert!(matches!(by_itself.why(), NotAuthorised::ChangeWaits { .. }));
    assert!(service.asked().is_empty());

    let running = a_broker("join", &service, 1);
    let (authorised, approval) = approved(&call);
    let verb = carry_out_approved(
        authorised,
        &service,
        Answered::OverTheNetwork,
        &running.reached,
        SystemTime::now(),
    )
    .unwrap();

    let cafe = Visible::reported(named("Cafe"), Protection::Open, 0).as_reported();
    assert_eq!(
        verb,
        SystemVerb::JoinNetwork(Identity::of_what_was_reported(&cafe))
    );
    assert_eq!(service.asked(), ["join Cafe"]);
    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [(Some(approval), None)]
    );
    assert_eq!(
        changed_said(&Change::Join("Cafe".into()), &strings).text(),
        "This machine is connected to Cafe"
    );
}

/// **A proposal that says the wrong thing about this conversation's connection
/// is never put to the person**, in either direction: keeping it when it would
/// be lost, and losing it when it would be kept — and a conversation answered
/// on this machine keeps its connection whatever the Wi-Fi does.
#[test]
fn a_proposal_wrong_about_this_conversation_is_never_put_to_the_person() {
    let on_wifi = Standing::reporting(a_flat_on_wifi());
    let on_a_cable = Standing::reporting(a_flat_on_a_cable());
    let wrong = || Err(NotChanged::SaysTheWrongThingAboutThisConversation);
    let over = Answered::OverTheNetwork;
    let here = Answered::OnThisMachine;

    let off_keeps = called(SWITCH_WIRELESS, (WIRELESS, "off"), "keeps_its_connection");
    let off_loses = called(SWITCH_WIRELESS, (WIRELESS, "off"), "loses_its_connection");
    assert_eq!(proposable(&off_keeps, &on_wifi, over), wrong());
    assert_eq!(proposable(&off_loses, &on_wifi, over), Ok(()));
    assert_eq!(proposable(&off_keeps, &on_a_cable, over), Ok(()));
    assert_eq!(proposable(&off_loses, &on_a_cable, over), wrong());
    assert_eq!(proposable(&off_keeps, &on_wifi, here), Ok(()));
    assert_eq!(proposable(&off_loses, &on_wifi, here), wrong());

    let forget = called("forget_network", (NETWORK, "Home"), "keeps_its_connection");
    assert_eq!(proposable(&forget, &on_wifi, over), wrong());
    let join_home = called(JOIN_NETWORK, (NETWORK, "Home"), "keeps_its_connection");
    assert_eq!(proposable(&join_home, &on_wifi, over), Ok(()));

    assert!(on_wifi.asked().is_empty() && on_a_cable.asked().is_empty());
}

/// **A change whose effect on this conversation changed after it was approved
/// is not carried out, and the broker is not asked**: approved on a cable as
/// keeping the connection, and the cable unplugged before it was made.
#[test]
fn an_approval_no_longer_true_about_this_conversation_changes_nothing() {
    let service = Standing::reporting(a_flat_on_a_cable());
    let call = called(SWITCH_WIRELESS, (WIRELESS, "off"), "keeps_its_connection");
    assert_eq!(
        proposable(&call, &service, Answered::OverTheNetwork),
        Ok(())
    );
    let (authorised, _) = approved(&call);
    *service.networks.lock().unwrap() = a_flat_on_wifi();

    let running = a_broker("unplugged", &service, 0);
    assert_eq!(
        carry_out_approved(
            authorised,
            &service,
            Answered::OverTheNetwork,
            &running.reached,
            SystemTime::now(),
        ),
        Err(NotChanged::ConnectionChangedSinceApproval)
    );
    assert!(service.asked().is_empty());
    assert!(kept(&running.answering.join().unwrap()).is_empty());
}

/// **A name no network has, a name two networks share, and a network asking
/// for an organisation's sign-in each change nothing and ask the broker
/// nothing** — the second being the open network next to a protected one of
/// the same name.
#[test]
fn a_name_nobody_has_or_two_share_asks_the_broker_nothing() {
    let mut flat = a_flat_on_a_cable();
    flat.visible
        .push(Visible::reported(named("Work"), Protection::Enterprise, 50));
    let service = Standing::reporting(flat);
    let running = a_broker("names", &service, 0);
    for (name, refused) in [
        ("Airport", NotChanged::NoneVisibleCalled("Airport".into())),
        ("Library", NotChanged::MoreThanOneCalled("Library".into())),
        ("Work", NotChanged::AsksForASignIn("Work".into())),
    ] {
        let call = called(JOIN_NETWORK, (NETWORK, name), "keeps_its_connection");
        assert_eq!(
            proposable(&call, &service, Answered::OverTheNetwork),
            Err(refused.clone())
        );
        let (authorised, _) = approved(&call);
        assert_eq!(
            carry_out_approved(
                authorised,
                &service,
                Answered::OverTheNetwork,
                &running.reached,
                SystemTime::now(),
            ),
            Err(refused)
        );
    }
    assert!(service.asked().is_empty());
    assert!(kept(&running.answering.join().unwrap()).is_empty());
}

/// **A token under any key but the broker's changes nothing**: a key planted
/// where the broker's was is not the broker's, the door refuses what it
/// proves, the refusal is written down, and the person is told the approval was
/// not accepted.
#[test]
fn a_token_under_any_key_but_the_brokers_changes_nothing() {
    let service = Standing::reporting(a_flat_on_wifi());
    let running = a_broker("planted", &service, 1);
    hand_over_a_fresh_key(&running.key, a_group()).unwrap();
    let call = called(JOIN_NETWORK, (NETWORK, "Home"), "keeps_its_connection");
    let (authorised, _) = approved(&call);
    assert_eq!(
        carry_out_approved(
            authorised,
            &service,
            Answered::OverTheNetwork,
            &running.reached,
            SystemTime::now(),
        ),
        Err(NotChanged::ApprovalNotAccepted)
    );
    assert!(service.asked().is_empty());
    let record = running.answering.join().unwrap();
    assert_eq!(kept(&record), [(None, Some(AtTheBroker::NotApproved))]);
}

/// **A person in Settings changes the network through the same verbs**: joining
/// a network by picking it, forgetting a saved one, turning Wi-Fi off, and
/// setting the machine's proxy — each written down as a change they made
/// themselves, and each exactly what an agent's approved proposal would have
/// asked for.
#[test]
fn a_person_in_settings_changes_the_network_through_the_same_verbs() {
    let flat = a_flat_on_wifi();
    let service = Standing::reporting(flat.clone());
    let running = a_broker("by-hand", &service, 4);
    let now = SystemTime::now();

    let cafe = Listed::in_range(&flat)
        .into_iter()
        .find(|listed| listed.name() == &named("Cafe"))
        .unwrap();
    assert_eq!(
        Some(cafe.picked()),
        alo_changing_network::chosen(&Change::Join("Cafe".into()), &flat).ok()
    );
    carry_out_by_hand(cafe.picked(), &running.reached, now).unwrap();
    let home = Listed::saved(&flat).into_iter().next().unwrap();
    carry_out_by_hand(home.picked(), &running.reached, now).unwrap();
    carry_out_by_hand(SystemVerb::SetRadio(Switch::Off), &running.reached, now).unwrap();

    let proxy =
        TheProxy::one(ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128).unwrap());
    set_proxy_by_hand(&proxy, None, &running.reached, now).unwrap();

    assert_eq!(
        service.asked(),
        ["join Cafe", "forget home-uuid", "wireless false"]
    );
    let machines = std::fs::read(&running.machines_proxy).unwrap();
    assert_eq!(
        kept_on_this_machine(&machines).unwrap(),
        Kept::by_this_person(proxy)
    );
    assert_eq!(
        kept(&running.answering.join().unwrap()),
        [(Some(BY_HAND), None); 4]
    );
}

/// **A person gives their own machine's proxy the password it asks for, from
/// the same place they set the proxy** — one approval, through the real door,
/// and the credential is written where ADR 0059 says it lives.
///
/// The proxy file the act leaves behind names where the password is kept, which
/// is the machine's own statement that one is set (ADR 0060 §1); nothing
/// anywhere reads it back out.
#[test]
fn a_person_gives_their_own_machines_proxy_the_password_it_asks_for() {
    let service = Standing::reporting(a_flat_on_wifi());
    let running = a_broker("a-password", &service, 1);
    let now = SystemTime::now();
    let proxy = signing_in_on_this_machine();
    let password = Password::typed("hunter2").unwrap();

    let set = set_proxy_by_hand(&proxy, Some(&password), &running.reached, now);
    let credential = running.store.join(THE_PERSONS_PROXY_PASSWORD);
    if the_tool().is_file() && set.is_ok() {
        let machines = std::fs::read(&running.machines_proxy).unwrap();
        assert_eq!(
            kept_on_this_machine(&machines).unwrap(),
            Kept::by_this_person(proxy)
        );
        let held = std::fs::read(&credential).unwrap();
        assert!(
            !held.windows(7).any(|window| window == b"hunter2"),
            "the store holds the password itself"
        );
    } else {
        // A machine with no `systemd-creds`, or one that cannot make a host
        // key, writes no credential — and then sets no proxy either, rather
        // than leaving one that cannot be signed in to.
        assert!(set.is_err());
        assert!(!running.machines_proxy.exists());
        assert!(!credential.exists());
    }

    // Either way the password the person typed is gone from the folder it was
    // handed over in.
    assert!(!running.wanted_password.exists());
    drop(running.answering.join().unwrap());
}

/// **A person on a machine an organisation manages is refused in the sentence
/// naming who can change it, and hands nothing over at all** — so the password
/// they typed never leaves the process they typed it into (ADR 0060 §5).
#[test]
fn a_managed_machine_refuses_a_persons_proxy_before_anything_is_handed_over() {
    let service = Standing::reporting(a_flat_on_wifi());
    let running = a_broker("managed", &service, 0);
    let theirs = Kept::by_an_organisation(TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "their-proxy.example.com", 3128).unwrap(),
    ));
    std::fs::write(&running.machines_proxy, machines(&theirs).unwrap()).unwrap();

    let refused = set_proxy_by_hand(
        &signing_in_on_this_machine(),
        Some(&Password::typed("hunter2").unwrap()),
        &running.reached,
        SystemTime::now(),
    )
    .unwrap_err();
    assert_eq!(refused, NotChanged::AnOrganisationSetTheProxy);

    let said = refused.said(&in_english());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("Ask whoever manages it"), "{said}");

    assert!(
        !running.wanted_password.exists(),
        "a password was handed over"
    );
    assert!(!running.wanted.exists(), "a proxy was handed over");
    assert_eq!(
        kept_on_this_machine(&std::fs::read(&running.machines_proxy).unwrap()).unwrap(),
        theirs
    );
    assert!(kept(&running.answering.join().unwrap()).is_empty());
}

/// A proxy that signs in where a person's own machine keeps a password.
fn signing_in_on_this_machine() -> TheProxy {
    TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128)
            .unwrap()
            .signing_in("anna", WhereThePasswordIs::on_this_machine())
            .unwrap(),
    )
}

/// What encrypts a credential on the machine running these tests — the real
/// tool where there is one, and a path there is nothing at otherwise.
fn the_tool() -> PathBuf {
    let at = PathBuf::from(alo_proxy::THE_TOOL);
    if at.is_file() {
        at
    } else {
        PathBuf::from("/nowhere-at-all/systemd-creds")
    }
}

/// **No broker, or no key, changes nothing**, and says the part of the machine
/// that makes changes is not running.
#[test]
fn no_broker_or_no_key_changes_nothing() {
    let service = Standing::reporting(a_flat_on_wifi());
    let running = a_broker("absent", &service, 0);
    let answered = running.answering.join().unwrap();
    assert!(kept(&answered).is_empty());

    assert_eq!(
        carry_out_by_hand(
            SystemVerb::SetRadio(Switch::On),
            &running.reached,
            SystemTime::now()
        ),
        Err(NotChanged::NothingMakesChanges)
    );
    std::fs::remove_file(&running.key).unwrap();
    assert_eq!(
        carry_out_by_hand(
            SystemVerb::SetRadio(Switch::On),
            &running.reached,
            SystemTime::now()
        ),
        Err(NotChanged::NothingMakesChanges)
    );
    assert!(service.asked().is_empty());
}
