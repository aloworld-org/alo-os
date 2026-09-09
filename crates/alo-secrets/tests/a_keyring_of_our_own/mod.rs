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
//! - **No service activation.** The generated bus configuration has **no
//!   `<servicedir>`**, so the bus cannot start anything on demand. Without that,
//!   `org.freedesktop.secrets.service` — which the package installs — would let
//!   the bus launch a keyring daemon *with the bus's own environment*, outside
//!   every isolation below, and a test could pass against the wrong store
//!   without anybody noticing.
//! - **Its own storage.** `XDG_DATA_HOME` in the same temporary directory, so
//!   the keyring files are written there and **no existing user's keyring is
//!   read, written or unlocked**.
//! - **Its own control directory**, `0700` as the daemon requires.
//! - **`--components=secrets`** and nothing else: no pkcs11, no ssh agent.
//! - **Never `--replace`.** That is the flag that would make this fixture the
//!   machine's normal keyring, which is exactly what it must not become.
//! - **A synthetic password**, which unlocks nothing that exists anywhere else.
//!
//! # What it cleans up
//!
//! The two processes it started and the directory it made. Nothing else — the
//! packages stay installed, and `docs/autonomy/updates/` records what they are.

#![allow(
    dead_code,
    reason = "each test binary uses the part of the fixture its own subject needs"
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
        // **No service activation.** A session bus finds what it may start in
        // `XDG_DATA_DIRS`, and the package installs
        // `org.freedesktop.secrets.service` into the machine's. Pointing the
        // bus at an empty directory of ours means the only Secret Service on it
        // is the one started below, with the isolation below — rather than one
        // the bus launched with its own environment, which a test could pass
        // against without anybody noticing.
        let nothing_to_activate = place.join("empty");
        std::fs::create_dir_all(&nothing_to_activate).expect("a directory can be made");

        let bus = Command::new("dbus-daemon")
            .arg("--session")
            .arg("--address")
            .arg(format!("unix:path={}", at.display()))
            .arg("--nofork")
            .arg("--nopidfile")
            .env("XDG_DATA_DIRS", &nothing_to_activate)
            .stdout(Stdio::null())
            .stderr(Stdio::from(
                std::fs::File::create(place.join("bus.err")).expect("a log can be made"),
            ))
            .spawn()
            .expect(
                "this machine has `dbus-daemon`. Install it with `gnome-keyring dbus-daemon`;                  docs/autonomy/updates/ records the prerequisites",
            );

        let mut ours = Self {
            place,
            bus,
            keyring: None,
        };
        ours.wait_for(&at);

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
