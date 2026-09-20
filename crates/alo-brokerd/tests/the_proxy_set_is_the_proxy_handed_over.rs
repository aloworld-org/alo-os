//! `network.set-proxy` sets the machine's proxy — and the password it signs in
//! with — to exactly what a person handed over, and in every other case leaves
//! both as they were.
//!
//! Task 3 of `docs/autonomy/v0-5-the-broker-and-the-disk-plan.md`: *set the proxy
//! `alo-proxy` holds*, by a closed type. The broker is told thirty-two bytes and
//! finds the proxy for itself, in the folder it made for the person; these are
//! the ways that can go wrong, each of them tried.
//!
//! Task 13 of `docs/autonomy/v0-5-software-and-the-web-plan.md` added the
//! password, in the same act and under the same approval (ADR 0060 §1). The
//! tests below that name one are that half: the credential is written **before**
//! the proxy file, only under the one name a person's own machine keeps a proxy
//! password by, and a machine where the write fails changes nothing at all.

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
use alo_networks::proxy_password::handed_over_together;
use alo_proxy::{
    Kept, Password, ProxyAddress, SetBy, SpokenTo, THE_PERSONS_PROXY_PASSWORD,
    TheMachinesCredentials, TheProxy, WhereThePasswordIs, handed_over,
};

/// A machine of this test's own: the folder a proxy is handed over in, where
/// the machine's proxy file goes, and the credential store.
struct AMachine {
    /// The handed-over proxy.
    wanted: PathBuf,
    /// The handed-over password.
    wanted_password: PathBuf,
    /// The machine's file.
    machines: PathBuf,
    /// The machine's credential store.
    store: PathBuf,
    /// What encrypts a credential, or something that will not.
    tool: PathBuf,
}

impl AMachine {
    /// Made fresh, with whatever this machine has that encrypts a credential.
    fn made(what: &str) -> Self {
        Self::with(what, the_real_tool().unwrap_or_else(nothing_that_encrypts))
    }

    /// Made fresh, with this as the tool that encrypts.
    fn with(what: &str, tool: PathBuf) -> Self {
        let root =
            std::env::temp_dir().join(format!("alo-brokerd-proxy-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&root));
        std::fs::create_dir_all(root.join("wanted")).unwrap();
        std::fs::create_dir_all(root.join("etc")).unwrap();
        Self {
            wanted: root.join("wanted").join("proxy.json"),
            wanted_password: root.join("wanted").join("proxy-password"),
            machines: root.join("etc").join("proxy.json"),
            store: root.join("etc").join("credstore.encrypted"),
            tool,
        }
    }

    /// The broker's carrier, for a person who is this test.
    fn proxy(&self) -> Proxy {
        Proxy::handed_over(
            &self.wanted,
            &self.wanted_password,
            &self.machines,
            TheMachinesCredentials::at(&self.store, &self.tool),
            our_user(),
        )
    }

    /// Hand this proxy over, and say the identity a person approved for it.
    fn hand_over(&self, proxy: &TheProxy) -> Identity {
        let bytes = wanted(proxy).unwrap();
        std::fs::write(&self.wanted, &bytes).unwrap();
        drop(std::fs::remove_file(&self.wanted_password));
        Identity::of_what_was_reported(&bytes)
    }

    /// Hand this proxy and this password over, and say the identity a person
    /// approved for the two of them together.
    fn hand_over_with(&self, proxy: &TheProxy, password: &str) -> Identity {
        let bytes = wanted(proxy).unwrap();
        let handed = handed_over(&Password::typed(password).unwrap()).unwrap();
        std::fs::write(&self.wanted, &bytes).unwrap();
        std::fs::write(&self.wanted_password, &handed).unwrap();
        Identity::of_what_was_reported(&handed_over_together(&bytes, &handed))
    }

    /// What the machine's file says, if there is one.
    fn kept(&self) -> Option<Kept> {
        std::fs::read(&self.machines)
            .ok()
            .map(|bytes| kept_on_this_machine(&bytes).unwrap())
    }

    /// The credential this machine holds for its proxy, if there is one.
    fn credential(&self) -> Option<Vec<u8>> {
        std::fs::read(self.store.join(THE_PERSONS_PROXY_PASSWORD)).ok()
    }
}

/// The real tool, where the machine running these tests has one.
fn the_real_tool() -> Option<PathBuf> {
    let at = PathBuf::from(alo_proxy::THE_TOOL);
    at.is_file().then_some(at)
}

/// A program that is not the tool and writes nothing.
fn nothing_that_encrypts() -> PathBuf {
    PathBuf::from("/bin/false")
}

/// A company's proxy, asking nobody who they are.
fn the_company_proxy() -> TheProxy {
    TheProxy::one(ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128).unwrap())
}

/// The same proxy, signing in where a person's own machine keeps a password.
fn signing_in_on_this_machine() -> TheProxy {
    TheProxy::one(
        ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128)
            .unwrap()
            .signing_in("anna", WhereThePasswordIs::on_this_machine())
            .unwrap(),
    )
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

/// **A proxy handed over with no password is asked for under exactly its own
/// bytes**, which is the sentence `docs/contracts/machine-proxy-file.md`
/// publishes and which adding a password may not break — and nothing is read
/// from the credential store or written into it.
#[test]
fn a_proxy_with_no_password_is_asked_for_under_exactly_its_own_bytes() {
    let machine = AMachine::with("no-password", nothing_that_encrypts());
    let bytes = wanted(&the_company_proxy()).unwrap();
    std::fs::write(&machine.wanted, &bytes).unwrap();

    machine
        .proxy()
        .set(Identity::of_what_was_reported(&bytes))
        .unwrap();
    assert_eq!(
        machine.kept(),
        Some(Kept::by_this_person(the_company_proxy()))
    );
    assert!(!machine.store.exists(), "the store was opened for nothing");
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

/// **A password changed after the approval is a different approval**, and
/// neither the proxy nor the credential is set — the identity covers both files
/// (ADR 0060 §3).
#[test]
fn a_password_changed_after_it_was_approved_sets_nothing() {
    let machine = AMachine::made("password-changed");
    let approved = machine.hand_over_with(&signing_in_on_this_machine(), "hunter2");
    let swapped = handed_over(&Password::typed("somebody elses").unwrap()).unwrap();
    std::fs::write(&machine.wanted_password, &swapped).unwrap();

    assert!(machine.proxy().set(approved).is_err());
    assert_eq!(machine.kept(), None);
    assert_eq!(machine.credential(), None);
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
        &machine.wanted_password,
        &machine.machines,
        TheMachinesCredentials::at(&machine.store, &machine.tool),
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

/// **A handed-over password is believed the same way**: a link where the file
/// should be is not followed, and something far longer than a password is not
/// read as one. Nothing is set either way.
#[test]
fn only_a_plain_password_file_of_the_persons_is_read() {
    let machine = AMachine::made("password-shape");
    let proxy = wanted(&signing_in_on_this_machine()).unwrap();

    let elsewhere = machine.wanted_password.with_file_name("elsewhere");
    std::fs::write(&elsewhere, b"not the person's").unwrap();
    std::os::unix::fs::symlink(&elsewhere, &machine.wanted_password).unwrap();
    std::fs::write(&machine.wanted, &proxy).unwrap();
    assert!(
        machine
            .proxy()
            .set(Identity::of_what_was_reported(&proxy))
            .is_err(),
        "a link was followed"
    );
    assert_eq!(machine.kept(), None);

    let enormous = vec![b'x'; 8 * 1024];
    std::fs::write(&machine.wanted, &proxy).unwrap();
    std::fs::write(&machine.wanted_password, &enormous).unwrap();
    assert!(
        machine
            .proxy()
            .set(Identity::of_what_was_reported(&handed_over_together(
                &proxy, &enormous
            )))
            .is_err()
    );
    assert_eq!(machine.kept(), None);
    assert_eq!(machine.credential(), None);
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

/// **A password handed over for a proxy that keeps one somewhere else is
/// refused, and nothing at all is set** — ADR 0060 §2, which is what keeps a
/// root process from writing a file somebody else named into the machine's
/// credential store.
#[test]
fn a_password_for_a_name_that_is_not_this_machines_own_sets_nothing() {
    for named in [
        "the company proxy",
        "shadow",
        "../the-proxy-on-this-machine",
    ] {
        let machine = AMachine::made("another-name");
        let proxy = TheProxy::one(
            ProxyAddress::checked(SpokenTo::Http, "proxy.example.com", 3128)
                .unwrap()
                .signing_in("anna", WhereThePasswordIs::named(named).unwrap())
                .unwrap(),
        );
        let identity = machine.hand_over_with(&proxy, "hunter2");
        assert!(machine.proxy().set(identity).is_err(), "{named}");
        assert_eq!(machine.kept(), None, "{named}");
        assert!(!machine.store.exists(), "{named}");
    }

    // And a password handed over for a proxy that asks for no name at all.
    let machine = AMachine::made("no-name-at-all");
    let identity = machine.hand_over_with(&the_company_proxy(), "hunter2");
    assert!(machine.proxy().set(identity).is_err());
    assert_eq!(machine.kept(), None);
}

/// **A machine where the credential cannot be written sets nothing**, rather
/// than leaving a proxy set that it cannot sign in to (ADR 0060 §1) — and the
/// handed-over password is gone from `/run` even so.
#[test]
fn a_machine_that_cannot_write_the_credential_sets_nothing() {
    let machine = AMachine::with("no-credential", nothing_that_encrypts());
    let identity = machine.hand_over_with(&signing_in_on_this_machine(), "hunter2");

    assert!(machine.proxy().set(identity).is_err());
    assert_eq!(machine.kept(), None, "a proxy was set that has no password");
    assert_eq!(machine.credential(), None);
    assert!(
        !machine.wanted_password.exists(),
        "a password was left in /run"
    );
}

/// **The password is written where ADR 0059 says it lives, before the proxy
/// file, readable by nobody but its owner — and the bytes the person handed
/// over are gone.**
///
/// Run against the real `systemd-creds` where the machine running the tests has
/// one. Where it has none, the test above is the one that runs and the act is
/// refused rather than half done.
#[test]
fn the_password_is_written_and_then_the_proxy_is() {
    let Some(tool) = the_real_tool() else {
        return;
    };
    let machine = AMachine::with("written", tool);
    let identity = machine.hand_over_with(&signing_in_on_this_machine(), "hunter2");
    if machine.proxy().set(identity).is_err() {
        // A machine whose host key cannot be made has nothing to measure here;
        // that it then sets nothing is the test above.
        assert_eq!(machine.kept(), None);
        return;
    }

    assert_eq!(
        machine.kept(),
        Some(Kept::by_this_person(signing_in_on_this_machine())),
        "the proxy names where its password is, which is how a panel knows one is set"
    );
    let at = machine.store.join(THE_PERSONS_PROXY_PASSWORD);
    let mode = std::fs::metadata(&at).unwrap().permissions().mode();
    assert_eq!(mode & 0o7777, 0o600, "{at:?}");
    assert!(!machine.wanted_password.exists());
    assert!(!machine.wanted.exists());

    // What is in the store is not the password: it is an encrypted credential.
    let held = machine.credential().unwrap();
    assert!(
        !held.windows(7).any(|window| window == b"hunter2"),
        "the store holds the password"
    );
}
