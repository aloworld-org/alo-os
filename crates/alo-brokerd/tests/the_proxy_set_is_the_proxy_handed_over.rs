//! `network.set-proxy` sets the machine's proxy to exactly the one a person
//! handed over, and in every other case leaves it as it was.
//!
//! Task 3 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *set the proxy
//! `alo-proxy` holds*, by a closed type. The broker is told thirty-two bytes and
//! finds the proxy for itself, in the folder it made for the person; these are
//! the ways that can go wrong, each of them tried.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::PathBuf;

use alo_broker::{Identity, our_user};
use alo_brokerd::Proxy;
use alo_networks::proxy_file::{kept_on_this_machine, machines, wanted};
use alo_proxy::{Kept, ProxyAddress, SetBy, SpokenTo, TheProxy};

/// A machine of this test's own: the folder a proxy is handed over in, and
/// where the machine's proxy file goes.
struct AMachine {
    /// The handed-over file.
    wanted: PathBuf,
    /// The machine's file.
    machines: PathBuf,
}

impl AMachine {
    /// Made fresh.
    fn made(what: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("alo-brokerd-proxy-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(root.join("wanted")).unwrap();
        std::fs::create_dir_all(root.join("etc")).unwrap();
        Self {
            wanted: root.join("wanted").join("proxy.json"),
            machines: root.join("etc").join("proxy.json"),
        }
    }

    /// The broker's carrier, for a person who is this test.
    fn proxy(&self) -> Proxy {
        Proxy::handed_over(&self.wanted, &self.machines, our_user())
    }

    /// Hand this proxy over, and say the identity a person approved for it.
    fn hand_over(&self, proxy: &TheProxy) -> Identity {
        let bytes = wanted(proxy).unwrap();
        std::fs::write(&self.wanted, &bytes).unwrap();
        Identity::of_what_was_reported(&bytes)
    }

    /// What the machine's file says, if there is one.
    fn kept(&self) -> Option<Kept> {
        std::fs::read(&self.machines)
            .ok()
            .map(|bytes| kept_on_this_machine(&bytes).unwrap())
    }
}

/// A company's proxy.
fn the_company_proxy() -> TheProxy {
    TheProxy::one(ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128).unwrap())
}

/// **The proxy handed over becomes the machine's, as the person's**: the file
/// holds exactly it, everybody may read it and only its owner write it, and
/// what was handed over is gone.
#[test]
fn the_proxy_handed_over_becomes_the_machines_as_the_persons() {
    let machine = AMachine::made("set");
    let identity = machine.hand_over(&the_company_proxy());
    machine.proxy().set(identity).unwrap();

    assert_eq!(
        machine.kept(),
        Some(Kept::by_this_person(the_company_proxy()))
    );
    let mode = std::fs::metadata(&machine.machines)
        .unwrap()
        .permissions()
        .mode();
    assert_eq!(mode & 0o7777, 0o644);
    assert!(!machine.wanted.exists());

    let identity = machine.hand_over(&TheProxy::None);
    machine.proxy().set(identity).unwrap();
    assert_eq!(machine.kept(), Some(Kept::by_this_person(TheProxy::None)));
}

/// **Bytes changed after the approval are a different proxy**, and nothing is
/// set.
#[test]
fn a_proxy_changed_after_it_was_approved_is_not_set() {
    let machine = AMachine::made("changed");
    let approved = machine.hand_over(&the_company_proxy());
    let swapped =
        TheProxy::one(ProxyAddress::checked(SpokenTo::Http, "attacker.example", 8080).unwrap());
    std::fs::write(&machine.wanted, wanted(&swapped).unwrap()).unwrap();
    assert!(machine.proxy().set(approved).is_err());
    assert_eq!(machine.kept(), None);
}

/// **What was handed over is believed only as a plain file of the person's**:
/// a link to somebody else's file is not followed, a file owned by anybody but
/// the person is not read, nothing handed over is nothing set, and neither is
/// something far longer than a proxy setting.
#[test]
fn only_a_plain_file_of_the_persons_is_read() {
    let machine = AMachine::made("shape");
    let bytes = wanted(&the_company_proxy()).unwrap();
    let identity = Identity::of_what_was_reported(&bytes);

    assert!(
        machine.proxy().set(identity).is_err(),
        "nothing handed over"
    );

    let elsewhere = machine.wanted.with_file_name("elsewhere.json");
    std::fs::write(&elsewhere, &bytes).unwrap();
    std::os::unix::fs::symlink(&elsewhere, &machine.wanted).unwrap();
    assert!(
        machine.proxy().set(identity).is_err(),
        "a link was followed"
    );
    std::fs::remove_file(&machine.wanted).unwrap();

    std::fs::write(&machine.wanted, &bytes).unwrap();
    let somebody_else = Proxy::handed_over(
        &machine.wanted,
        &machine.machines,
        our_user().wrapping_add(1),
    );
    assert!(
        somebody_else.set(identity).is_err(),
        "another user's file was read"
    );

    let enormous = vec![b' '; 70 * 1024];
    std::fs::write(&machine.wanted, &enormous).unwrap();
    assert!(
        machine
            .proxy()
            .set(Identity::of_what_was_reported(&enormous))
            .is_err()
    );

    assert_eq!(machine.kept(), None);
}

/// **A setting `alo-proxy` would not have made is not set**, even when the
/// identity approved is exactly its bytes.
#[test]
fn a_setting_alo_proxy_would_not_have_made_is_not_set() {
    let machine = AMachine::made("unchecked");
    let written =
        br#"{"proxy":"manual","http":{"spoken_to":"http","host":"proxy example.com","port":3128}}"#;
    std::fs::write(&machine.wanted, written).unwrap();
    assert!(
        machine
            .proxy()
            .set(Identity::of_what_was_reported(written))
            .is_err()
    );
    assert_eq!(machine.kept(), None);
}

/// **A proxy the organisation set is not replaced by a person's**, and a
/// machine's file that is not one this machine wrote is left alone.
#[test]
fn an_organisations_proxy_is_not_replaced_by_a_persons() {
    let machine = AMachine::made("organisation");
    let theirs = Kept::by_an_organisation(the_company_proxy());
    std::fs::write(&machine.machines, machines(&theirs).unwrap()).unwrap();
    let identity = machine.hand_over(&TheProxy::None);
    assert!(machine.proxy().set(identity).is_err());
    assert_eq!(
        machine.kept().map(|kept| kept.set_by()),
        Some(SetBy::AnOrganisation)
    );

    let garbled = AMachine::made("garbled");
    std::fs::write(&garbled.machines, b"not a proxy file").unwrap();
    let identity = garbled.hand_over(&TheProxy::None);
    assert!(garbled.proxy().set(identity).is_err());
    assert_eq!(
        std::fs::read(&garbled.machines).unwrap(),
        b"not a proxy file"
    );
}
