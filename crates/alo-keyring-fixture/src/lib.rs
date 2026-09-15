//! A real Secret Service, on a bus nobody else can reach, holding nothing but
//! what a test put there.
//!
//! # Why a real one
//!
//! `Unavailable` can be proved against a socket that is not there. *Locked*,
//! *missing* and *denied* cannot: they need a service that can be locked, be
//! empty, and refuse. And the session is **encrypted** — `EncryptionType::Dh` —
//! so a stub would mean writing the handshake ourselves, and a store whose
//! crypto we wrote to satisfy our own test proves less than it appears to.
//!
//! So this starts `gnome-keyring-daemon`, which is the implementation a machine
//! would really have.
//!
//! # How it is isolated, and every part of that is deliberate
//!
//! - **Its own bus.** A `dbus-daemon` on a socket in a temporary directory.
//! - **No service activation, on every bus it starts.** Each bus is started
//!   from a configuration this fixture writes, which names **no `<servicedir>`
//!   and no `<standard_session_servicedirs/>`**, so the bus cannot start
//!   anything on demand. Without that, `org.freedesktop.secrets.service` —
//!   which the package installs — lets the bus launch the machine's keyring
//!   *with the bus's own environment*, outside every isolation below.
//!
//!   It used to be done for most fixtures by starting `dbus-daemon --session`
//!   with an empty `XDG_DATA_DIRS`, and that did not hold: `--session` adds the
//!   compiled-in `/usr/share/dbus-1/services` and whatever `XDG_DATA_HOME`
//!   names, so a slow fixture keyring lost its name to the machine's
//!   (`docs/quirks.md`, 2026-09-13). One keyring on the bus is now a property
//!   of the configuration, and
//!   `alo-secrets/tests/one_keyring_behind_the_secret_portal.rs` holds it with a
//!   decoy activation file in both places the old way read.
//! - **Its own storage.** `XDG_DATA_HOME` in the same temporary directory, so
//!   the keyring files are written there and **no existing user's keyring is
//!   read, written or unlocked**.
//! - **Its own control directory**, `0700` as the daemon requires.
//! - **`--components=secrets`** and nothing else: no pkcs11, no ssh agent.
//! - **Never `--replace`.** That is the flag that would make this fixture the
//!   machine's normal keyring, which is exactly what it must not become.
//! - **A synthetic password**, which unlocks nothing that exists anywhere else.
//!
//! # Who uses it, and why it is a crate
//!
//! Two crates need a real keyring to ask questions of: `alo-secrets`, which owns
//! the store, and `alo-agentd`, which asks it for a provider's key. Writing this
//! twice would mean two fixtures drifting apart, and the one that drifted would
//! be the one whose tests still passed.
//!
//! It is reached **only through `dev-dependencies`** and is never published.
//! `image/Containerfile` builds `--package alo-agentd --package alo-boundaryd`,
//! and a `--package` release build compiles no dev-dependency, so nothing here
//! reaches a machine.
//!
//! # What it cleans up
//!
//! The two processes it started and the directory it made. Nothing else — the
//! packages stay installed, and `docs/autonomy/updates/` records what they are.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "a fixture that cannot reach the state it promises must fail the test loudly,               and it is only ever linked into one"
)]

use std::io::Write as _;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use alo_secrets::TheBus;

/// The password this fixture unlocks its own keyring with.
///
/// Synthetic, and a credential to nothing: it exists for the length of one test
/// and unlocks a keyring made in a temporary directory.
pub const A_SYNTHETIC_PASSWORD: &str = "a-fixture-password-not-a-credential";

/// A keyring of this test's own: a bus, a daemon, and a place for both.
pub struct AKeyringOfOurOwn {
    /// Where everything it made lives.
    place: PathBuf,

    /// The bus.
    bus: Child,

    /// The daemon serving `org.freedesktop.secrets` on it.
    keyring: Option<Child>,

    /// The bus's configuration file.
    ///
    /// Every bus is started from one, so that no bus this fixture makes can
    /// activate a service; the refusal tests also rewrite and reload it, which
    /// is how a **real** access denial is produced rather than simulated.
    config: PathBuf,
}

impl AKeyringOfOurOwn {
    /// Start one, or say plainly why this machine cannot.
    ///
    /// # Panics
    /// When `dbus-daemon` or `gnome-keyring-daemon` is not installed, naming
    /// both — a test that skipped itself here would report green on every
    /// machine in the world, including the ones with no Secret Service at all.
    #[must_use]
    pub fn started(what: &str) -> Self {
        Self::start(what, true)
    }

    /// One whose bus's policy can be rewritten and reloaded while it runs.
    ///
    /// Since every bus is started from a configuration file this is
    /// [`Self::started`] under the name the refusal tests read best by — same
    /// isolation, same synthetic password, still never `--replace`.
    ///
    /// # Panics
    /// As [`Self::started`].
    #[must_use]
    pub fn started_where_the_bus_can_refuse(what: &str) -> Self {
        Self::start(what, true)
    }

    /// A real bus with **no Secret Service on it at all**.
    ///
    /// This is the machine whose image ships no keyring: the bus is up and
    /// answering, and nothing owns `org.freedesktop.secrets`. It is reached by
    /// not starting the daemon rather than by stopping one — an earlier attempt
    /// killed a running keyring and something went on serving the name, so the
    /// test asserted a state it had not produced. A state you can decline to
    /// create is never worth destroying.
    ///
    /// # Panics
    /// When `dbus-daemon` is not installed, or its socket never appears.
    #[must_use]
    pub fn a_bus_with_no_keyring_on_it(what: &str) -> Self {
        Self::start(what, false)
    }

    /// The three of them, which differ only in whether anything is put on the
    /// bus.
    fn start(what: &str, serving: bool) -> Self {
        // Only characters a D-Bus address may carry unescaped: the socket
        // under this directory becomes one, and `(` from a thread id is exactly
        // what `dbus-daemon` refuses.
        let place = std::env::temp_dir().join(format!("alo-keyring-{}-{what}", std::process::id()));
        drop(std::fs::remove_dir_all(&place));
        for under in ["data", "run", "control"] {
            std::fs::create_dir_all(place.join(under)).expect("a temporary directory can be made");
        }
        // The daemon refuses a control directory anybody else could read.
        std::fs::set_permissions(
            place.join("control"),
            <std::fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o700),
        )
        .expect("a directory of ours can be made private");

        let at = place.join("bus");
        // **No service activation.** The configuration names no service
        // directory, so the only Secret Service on this bus is the one started
        // below, with the isolation below — never one the bus launched with its
        // own environment, which a test could pass against without anybody
        // noticing. Not `--session`: that reads the machine's directories
        // whatever the environment says.
        let config = place.join("bus.conf");
        std::fs::write(&config, policy_allowing_everything(&at))
            .expect("a configuration can be written");

        let bus = Command::new("dbus-daemon")
            .arg("--config-file")
            .arg(&config)
            .arg("--nofork")
            .arg("--nopidfile")
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                std::fs::File::create(place.join("bus.err")).expect("a log can be made"),
            ))
            .spawn()
            .expect(
                "this machine has `dbus-daemon`. Install it with `gnome-keyring dbus-daemon`; \
                 docs/autonomy/updates/ records the prerequisites",
            );

        let mut ours = Self {
            place,
            bus,
            keyring: None,
            config,
        };
        ours.wait_for(&at);
        if !serving {
            // Nothing is put on this bus, and nothing waits for a service that
            // is never coming.
            return ours;
        }

        // `--unlock` reads the password from stdin, and is incompatible with
        // `--start` — measured, not supposed. `--replace` is deliberately absent.
        let mut keyring = Command::new("gnome-keyring-daemon")
            .arg("--foreground")
            .arg("--unlock")
            .arg("--components=secrets")
            .arg("--control-directory")
            .arg(ours.place.join("control"))
            .env("XDG_DATA_HOME", ours.place.join("data"))
            .env("XDG_RUNTIME_DIR", ours.place.join("run"))
            .env(
                "DBUS_SESSION_BUS_ADDRESS",
                format!("unix:path={}", at.display()),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                std::fs::File::create(ours.place.join("keyring.err")).expect("a log can be made"),
            ))
            .spawn()
            .expect(
                "this machine has `gnome-keyring-daemon`. Install it with \
                 `gnome-keyring dbus-daemon`; docs/autonomy/updates/ records the prerequisites",
            );
        {
            let mut telling = keyring.stdin.take().expect("stdin was asked for");
            telling
                .write_all(A_SYNTHETIC_PASSWORD.as_bytes())
                .expect("the daemon is listening");
        }
        ours.keyring = Some(keyring);
        ours.wait_until_it_serves();
        ours
    }

    /// The bus it is on, as this crate checks one.
    ///
    /// # Panics
    /// When the socket it made is not one this crate will use, which would be
    /// the fixture and not the subject.
    #[must_use]
    pub fn bus(&self) -> TheBus {
        let ours = rustix::process::getuid().as_raw();
        TheBus::at(&self.place.join("bus"), ours).expect("the fixture's own bus is usable")
    }

    /// The process serving `org.freedesktop.secrets` for this fixture, or
    /// [`None`] on a bus that was started with no keyring.
    ///
    /// For a test that asks the bus *who* owns the name, and needs to know the
    /// answer should be this fixture's own daemon and nobody else.
    #[must_use]
    pub fn keyring_process(&self) -> Option<u32> {
        self.keyring.as_ref().map(Child::id)
    }

    /// The address of that bus, for a client built by hand.
    #[must_use]
    pub fn address(&self) -> String {
        format!("unix:path={}", self.place.join("bus").display())
    }

    /// Wait for the socket to appear.
    fn wait_for(&mut self, at: &std::path::Path) {
        let until = Instant::now() + Duration::from_secs(15);
        while Instant::now() < until {
            if at.exists() {
                return;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        let said = std::fs::read_to_string(self.place.join("bus.err")).unwrap_or_default();
        panic!(
            "the fixture's bus never appeared at {}: {said}",
            at.display()
        );
    }

    /// Wait until the keyring is ready to be *used*, not merely to answer.
    ///
    /// The daemon takes its name on the bus before it has finished making and
    /// unlocking the login keyring, so a connection can succeed while
    /// `get_default_collection` still says `NoResult`. Waiting only for the
    /// service is a race this lost the moment three fixtures ran at once, and a
    /// fixture that is sometimes ready is a test suite that sometimes fails for
    /// a reason having nothing to do with its subject.
    ///
    /// So readiness is the state the fixture actually promises: a collection to
    /// put a secret in.
    fn wait_until_it_serves(&mut self) {
        let until = Instant::now() + Duration::from_secs(20);
        while Instant::now() < until {
            if let Ok(connection) =
                zbus::blocking::connection::Builder::address(self.address().as_str())
                    .and_then(zbus::blocking::connection::Builder::build)
                && let Ok(service) = secret_service::blocking::SecretService::connect_with_existing(
                    secret_service::EncryptionType::Dh,
                    connection,
                )
                && service.get_default_collection().is_ok()
            {
                return;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        // Its own words, not a guess about them. A fixture that fails silently
        // gets diagnosed by theory, and the theory is usually wrong.
        let said = std::fs::read_to_string(self.place.join("keyring.err")).unwrap_or_default();
        let bus = std::fs::read_to_string(self.place.join("bus.err")).unwrap_or_default();
        panic!(
            "the fixture's keyring never offered a collection to store a secret in.\n\
             keyring said: {said}\nbus said: {bus}"
        );
    }
}

/// A session bus that allows what a session bus normally allows.
///
/// No `<servicedir>` and no `<standard_session_servicedirs/>`, so this bus can
/// start nothing on demand, whatever the machine has installed and whatever the
/// environment names.
fn policy_allowing_everything(at: &std::path::Path) -> String {
    format!(
        r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:path={at}</listen>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
"#,
        at = at.display()
    )
}

/// The same, with the Secret Service put out of reach.
///
/// The `<deny>` comes after the allows because the last matching rule wins, and
/// it names the well-known destination rather than a connection: this is the
/// bus refusing to carry the message, which is what a refusal actually is.
fn policy_refusing_the_secrets(at: &std::path::Path) -> String {
    format!(
        r#"<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-BUS Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <listen>unix:path={at}</listen>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
    <deny send_destination="org.freedesktop.secrets"/>
  </policy>
</busconfig>
"#,
        at = at.display()
    )
}

impl AKeyringOfOurOwn {
    /// Make the **bus itself** refuse to carry anything to the Secret Service,
    /// and have it take effect now.
    ///
    /// This is the real mechanism by which a caller is refused: not a keyring
    /// deciding, but the bus declining to deliver, which is what
    /// `org.freedesktop.DBus.Error.AccessDenied` means when a person meets it.
    /// The service stays running and stays reachable by anything the policy
    /// still allows, so what this produces is a **refusal** and not an outage —
    /// the distinction the four refusal states exist to make.
    ///
    /// # Panics
    /// When the configuration cannot be rewritten, or the bus will not reload.
    pub fn stop_letting_anyone_reach_the_keyring(&self) {
        std::fs::write(
            &self.config,
            policy_refusing_the_secrets(&self.place.join("bus")),
        )
        .expect("the configuration can be rewritten");

        // Reloading is asked of the bus over its own connection, which the new
        // policy still permits — the deny names the secrets service alone.
        let connection = zbus::blocking::connection::Builder::address(self.address().as_str())
            .expect("an address")
            .build()
            .expect("a connection");
        connection
            .call_method(
                Some("org.freedesktop.DBus"),
                "/org/freedesktop/DBus",
                Some("org.freedesktop.DBus"),
                "ReloadConfig",
                &(),
            )
            .expect("the bus reloads its configuration");
    }
}

impl Drop for AKeyringOfOurOwn {
    fn drop(&mut self) {
        // Only what this fixture started, and only what it made. The packages
        // stay installed and nothing else on the machine is touched.
        if let Some(keyring) = self.keyring.as_mut() {
            drop(keyring.kill());
            drop(keyring.wait());
        }
        drop(self.bus.kill());
        drop(self.bus.wait());
        drop(std::fs::remove_dir_all(&self.place));
    }
}
