//! The machine's own passwords, as somebody sets one — the writer beside
//! `crate::provisioned`'s reader.
//!
//! [ADR 0059](../../../docs/decisions/0059-where-a-machine-wide-proxy-password-is-kept.md)
//! decided where a machine-wide proxy password lives and built the half that
//! reads it, for the case that pays for it: an organisation provisions the
//! credential with the machine. **On a personal machine there is nobody else to
//! do it**, and that record said so and left it —
//! [ADR 0060](../../../docs/decisions/0060-a-persons-own-proxy-password-is-set-with-the-proxy-in-one-act.md)
//! is the decision that closes it. This file is its §3 and §4: how a password a
//! person typed is handed over, and how the credential is written.
//!
//! | | |
//! |---|---|
//! | [`handed_over`], [`what_was_handed_over`] | the bytes a person's password crosses the broker's door as, and reading them back |
//! | [`TheMachinesCredentials`] | the store, written through the tool the base already has |
//! | [`NotProvisioned`] | every way it is refused, and on each of them **nothing was written** |
//!
//! # Salted, because a digest of a password is a password
//!
//! The broker's door takes no text, so what crosses it is the digest of bytes
//! the broker finds for itself — and that digest is written into the broker's
//! record. SHA-256 of what somebody typed is a dictionary attack away from what
//! they typed, so [`handed_over`] puts [`NONCE_BYTES`] of the kernel's
//! randomness in front of the password and the identity is taken over both. The
//! record then names an act and says nothing about the credential.
//!
//! # No encryption of ours
//!
//! `systemd-creds` is a mechanism the base already has, and
//! [ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)
//! is why that settles it: the base is rented, configured and never patched, and
//! a key derivation or an envelope written here would be the security bug the
//! rest of this workspace is organised to avoid. It is started directly, with a
//! cleared environment, and **the password goes on its standard input** — never
//! in an argument, because an argument list is readable and a secret is not.
//!
//! # A refusal quotes nothing, including what the tool said
//!
//! [`NotProvisioned::TheToolRefused`] carries the exit status and **not one byte
//! of what the program printed**. That is deliberate and it costs something: a
//! machine where `systemd-creds` fails for a reason of its own says only that it
//! failed. The alternative is a refusal that could carry whatever the tool
//! decided to echo, in a crate whose whole subject is a credential, and
//! `crate::NotSignedIn` already made the same trade on the reading side.

use std::fs;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::evaluator::nothing_of_this_machines;
use crate::password::{NotAPassword, Password, WhereThePasswordIs};
use crate::provisioned::{LONGEST_PASSWORD, one_thing_to_look_up};

/// Where a machine keeps the credentials it was given, encrypted at rest.
///
/// Root's, `0700`, and the path the `LoadCredentialEncrypted=` line in a unit
/// file names on its right-hand side (ADR 0059).
pub const THE_ENCRYPTED_STORE: &str = "/etc/credstore.encrypted";

/// The program that encrypts a credential this machine can decrypt again.
///
/// The base's own, at the path an alo OS machine has one at — exactly as
/// `crate::THE_EVALUATOR` is. **A machine with nothing there writes no
/// credential** ([`NotProvisioned::NothingEncryptsIt`]) rather than keeping a
/// password some other way, and a test holds that.
pub const THE_TOOL: &str = "/usr/bin/systemd-creds";

/// How many bytes of the kernel's randomness a handed-over password carries in
/// front of it.
///
/// Thirty-two, which is what the identity the broker is asked for is made of,
/// and enough that the digest of a handed-over password says nothing about the
/// password.
pub const NONCE_BYTES: usize = 32;

/// The most bytes a handed-over password may be: the nonce and a password.
pub const LONGEST_HANDED_OVER: u64 = NONCE_BYTES as u64 + LONGEST_PASSWORD;

/// The kernel gave no randomness, so nothing was handed over.
///
/// There is no fallback generator, for `alo_broker::NoRandomness`' reason: a
/// nonce from anywhere else is a nonce somebody could work out, and the whole
/// point of this one is that nobody can.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("the kernel gave no randomness to hand a proxy password over under: {0}")]
pub struct NoRandomness(pub String);

/// Why some bytes are not a password somebody handed over.
///
/// **No `Display`**, for `crate::NotSignedIn`'s reason: every one of these is
/// about a credential, and a sentence one `to_string()` away from a credential
/// is a sentence somebody will eventually log. What a person reads when their
/// proxy was not set is the setting surface's own sentence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotHandedOver {
    /// Shorter than the nonce, or longer than a nonce and a password.
    NotTheShapeOfOne,
    /// What was handed over is not a password.
    NotAPassword(NotAPassword),
}

/// Why a credential was not written, on every one of which **nothing was
/// written**.
///
/// **No `Display`** either, and for the same reason as [`NotHandedOver`]. The
/// broker says which of these it met in its own service log, in English, for
/// whoever stands the machine up; the person in Settings reads that their proxy
/// was not set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotProvisioned {
    /// The name is not the one a person's own machine keeps its proxy password
    /// under (ADR 0060 §2), so nothing was written under a name nobody reads.
    NotThisMachinesOwnName,
    /// The name is not one thing that can be looked up, so nothing was written
    /// beside the store or above it.
    NotALookableName,
    /// The store is not there and could not be made.
    NoStore(ErrorKind),
    /// This machine has nothing that encrypts a credential.
    NothingEncryptsIt,
    /// The tool ran and would not do it.
    ///
    /// Carries the exit status and **nothing the program printed**.
    TheToolRefused(Option<i32>),
    /// What it wrote could not be put in place.
    NotWritten(ErrorKind),
    /// What it wrote could be read by somebody other than its owner, so it was
    /// removed rather than left in the store.
    ReadableByAnybody,
}

impl NotProvisioned {
    /// **Nothing was written**, and what was there before is still there.
    ///
    /// Stated as a method rather than as a comment, the way
    /// `crate::NotSignedIn::nothing_was_sent` is: a credential is written
    /// beside its place and renamed over it, so every refusal here leaves the
    /// store exactly as it was.
    #[must_use]
    pub const fn nothing_was_written(self) -> bool {
        true
    }
}

/// The bytes a person's password is handed to the broker as.
///
/// [`NONCE_BYTES`] of the kernel's randomness, then the password. The digest of
/// these is what crosses the door; see this file's own documentation for why
/// the nonce is there.
///
/// # Errors
/// [`NoRandomness`] when the kernel would give none, and nothing is handed over.
pub fn handed_over(password: &Password) -> Result<Vec<u8>, NoRandomness> {
    let mut nonce = [0_u8; NONCE_BYTES];
    getrandom::fill(&mut nonce).map_err(|why| NoRandomness(why.to_string()))?;
    let mut bytes = Vec::with_capacity(NONCE_BYTES.saturating_add(password.as_str().len()));
    bytes.extend_from_slice(&nonce);
    bytes.extend_from_slice(password.as_str().as_bytes());
    Ok(bytes)
}

/// The password in what somebody handed over, rebuilt through
/// [`Password::typed`]'s own checks.
///
/// Nothing read out of a file is believed: what is past the nonce is a password
/// only if that constructor says so, exactly as `crate::provisioned` believes
/// nothing it finds in a credentials directory.
///
/// # Errors
/// [`NotHandedOver`].
pub fn what_was_handed_over(bytes: &[u8]) -> Result<Password, NotHandedOver> {
    let how_many = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if bytes.len() <= NONCE_BYTES || how_many > LONGEST_HANDED_OVER {
        return Err(NotHandedOver::NotTheShapeOfOne);
    }
    let held = bytes
        .get(NONCE_BYTES..)
        .ok_or(NotHandedOver::NotTheShapeOfOne)?;
    let held = std::str::from_utf8(held)
        .map_err(|_| NotHandedOver::NotAPassword(NotAPassword::NotSendable))?;
    Password::typed(held).map_err(NotHandedOver::NotAPassword)
}

/// What the tool is asked, as an argument list.
///
/// Kept apart from starting the process so the list is testable as a list on
/// any machine — including one where nothing is installed at [`THE_TOOL`] —
/// which is `crate::arguments`' reason for the evaluator.
///
/// `--with-key=auto` is the tool's own default and is said anyway: it is the
/// line that decides a credential is sealed to this machine's TPM where there
/// is one and to its root-only host key where there is not, and a default is a
/// poor place to keep a decision ADR 0059 wrote down.
#[must_use]
pub fn arguments(entry: &str, into: &Path) -> Vec<String> {
    vec![
        "encrypt".to_owned(),
        "--with-key=auto".to_owned(),
        format!("--name={entry}"),
        "-".to_owned(),
        into.display().to_string(),
    ]
}

/// The credentials this machine keeps, as somebody writes one.
///
/// The counterpart of `crate::TheMachinesPasswords`: that one reads what a unit
/// was given, this one writes what the unit will be given at its next start.
/// Holds two paths and nothing else; making one reaches nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheMachinesCredentials {
    /// The store written into.
    store: PathBuf,
    /// The program that encrypts.
    tool: PathBuf,
}

impl Default for TheMachinesCredentials {
    fn default() -> Self {
        Self::at(Path::new(THE_ENCRYPTED_STORE), Path::new(THE_TOOL))
    }
}

impl TheMachinesCredentials {
    /// The store and the tool where an alo OS machine has them.
    #[must_use]
    pub fn on_this_machine() -> Self {
        Self::default()
    }

    /// A store and a tool somewhere else — a test's.
    ///
    /// Public for the reason `crate::TheMachinesPasswords::at` is: the one
    /// caller that ships is in another crate, and a test that cannot say *write
    /// it here, with this* could only show that the writing compiles. It points
    /// a **writer** and a **program**, which the reader's door deliberately does
    /// not, so it is worth saying what bounds it: the name written under is
    /// never this parameter's — it comes from the setting and must be
    /// [`crate::THE_PERSONS_PROXY_PASSWORD`] — and the one process that ships
    /// builds this with [`TheMachinesCredentials::on_this_machine`], which takes
    /// nothing at all.
    #[must_use]
    pub fn at(store: &Path, tool: &Path) -> Self {
        Self {
            store: store.to_path_buf(),
            tool: tool.to_path_buf(),
        }
    }

    /// The store, for a caller that has to say where it wrote.
    #[must_use]
    pub fn store(&self) -> &Path {
        &self.store
    }

    /// The program, by path.
    #[must_use]
    pub fn tool(&self) -> &Path {
        &self.tool
    }

    /// Write this password into the store, under the name the setting keeps it
    /// by.
    ///
    /// Beside its place, `0600`, and renamed over — so a store that already held
    /// this credential still holds that one if anything fails.
    ///
    /// # Errors
    /// [`NotProvisioned`], on every one of which nothing was written.
    pub fn written(
        &self,
        kept: &WhereThePasswordIs,
        password: &Password,
    ) -> Result<(), NotProvisioned> {
        if !kept.is_this_machines_own() {
            return Err(NotProvisioned::NotThisMachinesOwnName);
        }
        let entry = one_thing_to_look_up(kept.as_str()).ok_or(NotProvisioned::NotALookableName)?;
        self.made()?;

        let at = self.store.join(entry);
        let beside = self.store.join(format!("{entry}.writing"));
        drop(fs::remove_file(&beside));
        let written = self.encrypted(entry, &beside, password).and_then(|()| {
            only_the_owners(&beside)?;
            fs::rename(&beside, &at).map_err(|why| NotProvisioned::NotWritten(why.kind()))
        });
        if written.is_err() {
            drop(fs::remove_file(&beside));
        }
        written
    }

    /// The store, made root's and root's alone if it is not there.
    fn made(&self) -> Result<(), NotProvisioned> {
        match fs::create_dir_all(&self.store) {
            Ok(()) => only_this_machines(&self.store),
            Err(why) => Err(NotProvisioned::NoStore(why.kind())),
        }
    }

    /// The tool, run over this password, writing here.
    fn encrypted(
        &self,
        entry: &str,
        beside: &Path,
        password: &Password,
    ) -> Result<(), NotProvisioned> {
        let mut command = Command::new(&self.tool);
        command
            .args(arguments(entry, beside))
            .env_clear()
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        for (name, value) in nothing_of_this_machines() {
            command.env(name, value);
        }
        let mut running = command.spawn().map_err(|why| {
            if why.kind() == ErrorKind::NotFound {
                NotProvisioned::NothingEncryptsIt
            } else {
                NotProvisioned::NotWritten(why.kind())
            }
        })?;
        // The password, and nowhere else. A credential shorter than
        // `LONGEST_PASSWORD` fits a pipe without anything reading it, so this
        // cannot wait on the other end.
        let handing_over = running.stdin.take().map_or(
            Err(NotProvisioned::NotWritten(ErrorKind::BrokenPipe)),
            |mut stdin| {
                stdin
                    .write_all(password.as_str().as_bytes())
                    .map_err(|why| NotProvisioned::NotWritten(why.kind()))
            },
        );
        let finished = running
            .wait()
            .map_err(|why| NotProvisioned::NotWritten(why.kind()))?;
        handing_over?;
        if finished.success() {
            Ok(())
        } else {
            Err(NotProvisioned::TheToolRefused(finished.code()))
        }
    }
}

/// The store held to being nobody's but its owner's, once it has been made.
#[cfg(unix)]
fn only_this_machines(store: &Path) -> Result<(), NotProvisioned> {
    use std::os::unix::fs::PermissionsExt as _;

    let about = fs::metadata(store).map_err(|why| NotProvisioned::NoStore(why.kind()))?;
    if about.permissions().mode() & 0o077 == 0 {
        return Ok(());
    }
    fs::set_permissions(store, fs::Permissions::from_mode(0o700))
        .map_err(|why| NotProvisioned::NoStore(why.kind()))
}

/// On a host that does not say it in a mode, there is nothing to hold it to.
#[cfg(not(unix))]
#[expect(
    clippy::missing_const_for_fn,
    reason = "the same signature as the unix half, which reads permissions"
)]
fn only_this_machines(_store: &Path) -> Result<(), NotProvisioned> {
    Ok(())
}

/// What the tool wrote, held to being readable by nobody but its owner.
///
/// The tool's own mode is its business and this does not take it on trust —
/// `crate::provisioned`'s reader does not take systemd's on trust either. A
/// credential that came out readable by everybody is removed rather than left
/// in the store, which is what [`NotProvisioned::ReadableByAnybody`] says.
#[cfg(unix)]
fn only_the_owners(beside: &Path) -> Result<(), NotProvisioned> {
    use std::os::unix::fs::PermissionsExt as _;

    use crate::provisioned::only_its_owners;

    fs::set_permissions(beside, fs::Permissions::from_mode(0o600))
        .map_err(|why| NotProvisioned::NotWritten(why.kind()))?;
    let about = fs::metadata(beside).map_err(|why| NotProvisioned::NotWritten(why.kind()))?;
    if only_its_owners(&about) {
        Ok(())
    } else {
        Err(NotProvisioned::ReadableByAnybody)
    }
}

/// On a host that does not say it in a mode, there is nothing to hold it to.
#[cfg(not(unix))]
#[expect(
    clippy::missing_const_for_fn,
    reason = "the same signature as the unix half, which sets and reads permissions"
)]
fn only_the_owners(_beside: &Path) -> Result<(), NotProvisioned> {
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::password::THE_PERSONS_PROXY_PASSWORD;
    use crate::testing::a_directory_of_its_own;

    /// The name a person's own machine keeps its proxy password under.
    fn named() -> WhereThePasswordIs {
        WhereThePasswordIs::on_this_machine()
    }

    /// The real tool, where the machine running these tests has one.
    #[cfg(unix)]
    fn the_real_tool() -> Option<PathBuf> {
        let at = PathBuf::from(THE_TOOL);
        at.is_file().then_some(at)
    }

    /// A program that is not the tool: this test binary itself, which exits
    /// without writing anything where a credential goes.
    fn something_that_is_not_the_tool() -> PathBuf {
        PathBuf::from("/bin/false")
    }

    /// **A handed-over password reads back as exactly the password**, and
    /// carries a nonce in front of it so that the digest of it says nothing.
    #[test]
    fn a_handed_over_password_reads_back_and_is_salted() {
        let password = Password::typed("hunter2").unwrap();
        let bytes = handed_over(&password).unwrap();
        assert_eq!(bytes.len(), NONCE_BYTES + "hunter2".len());

        let read = what_was_handed_over(&bytes).unwrap();
        assert_eq!(format!("{read:?}"), "Password(…)");

        // Two hand-overs of one password are different bytes, which is the
        // whole of what the nonce buys: the digest in the record is not a
        // digest anybody can compute from a guess.
        let again = handed_over(&password).unwrap();
        assert_ne!(bytes, again);
        assert_ne!(
            bytes.get(..NONCE_BYTES).unwrap(),
            again.get(..NONCE_BYTES).unwrap()
        );
    }

    /// **Bytes that are not a handed-over password are refused**, and none of
    /// them is read as one: nothing at all, a nonce with nothing after it, a
    /// nonce and blank space, a nonce and something unsendable, and more than a
    /// password.
    #[test]
    fn bytes_that_are_not_a_handed_over_password_are_refused() {
        let nonce = [7_u8; NONCE_BYTES];
        let with = |after: &[u8]| {
            let mut bytes = nonce.to_vec();
            bytes.extend_from_slice(after);
            bytes
        };
        assert_eq!(
            what_was_handed_over(&[]).unwrap_err(),
            NotHandedOver::NotTheShapeOfOne
        );
        assert_eq!(
            what_was_handed_over(&nonce).unwrap_err(),
            NotHandedOver::NotTheShapeOfOne
        );
        assert_eq!(
            what_was_handed_over(&with(&[b'x'; 4097])).unwrap_err(),
            NotHandedOver::NotTheShapeOfOne
        );
        assert_eq!(
            what_was_handed_over(&with(b"   \n")).unwrap_err(),
            NotHandedOver::NotAPassword(NotAPassword::Blank)
        );
        assert_eq!(
            what_was_handed_over(&with(b"hun\x1bter2")).unwrap_err(),
            NotHandedOver::NotAPassword(NotAPassword::NotSendable)
        );
        assert_eq!(
            what_was_handed_over(&with(&[0xff, 0xfe])).unwrap_err(),
            NotHandedOver::NotAPassword(NotAPassword::NotSendable)
        );
    }

    /// **The argument list says what it is doing and carries no password**, and
    /// the password is not in it because it goes on the standard input.
    #[test]
    fn the_argument_list_carries_no_password() {
        let into = Path::new("/etc/credstore.encrypted/the-proxy-on-this-machine.writing");
        let arguments = arguments(THE_PERSONS_PROXY_PASSWORD, into);
        assert_eq!(
            arguments,
            [
                "encrypt".to_owned(),
                "--with-key=auto".to_owned(),
                format!("--name={THE_PERSONS_PROXY_PASSWORD}"),
                "-".to_owned(),
                into.display().to_string(),
            ]
        );
        assert!(!arguments.join(" ").contains("hunter2"));
    }

    /// **A name that is not this machine's own is refused and nothing is
    /// written** — ADR 0060 §2, which is also what keeps a root process from
    /// writing a file somebody else named into the store.
    #[test]
    fn a_name_that_is_not_this_machines_own_writes_nothing() {
        let store = a_directory_of_its_own("not-this-machines-name");
        let credentials = TheMachinesCredentials::at(&store, &something_that_is_not_the_tool());
        for name in [
            "the company proxy",
            "../the-proxy-on-this-machine",
            "shadow",
        ] {
            let kept = WhereThePasswordIs::named(name).unwrap();
            let refused = credentials
                .written(&kept, &Password::typed("hunter2").unwrap())
                .unwrap_err();
            assert_eq!(refused, NotProvisioned::NotThisMachinesOwnName, "{name}");
            assert!(refused.nothing_was_written());
        }
        assert_eq!(fs::read_dir(&store).unwrap().count(), 0);
    }

    /// **A machine with nothing that encrypts writes nothing**, rather than
    /// keeping the password some other way.
    #[test]
    fn a_machine_with_nothing_that_encrypts_writes_nothing() {
        let store = a_directory_of_its_own("nothing-encrypts");
        let credentials =
            TheMachinesCredentials::at(&store, Path::new("/nowhere/at/all/systemd-creds"));
        let refused = credentials
            .written(&named(), &Password::typed("hunter2").unwrap())
            .unwrap_err();
        assert_eq!(refused, NotProvisioned::NothingEncryptsIt);
        assert!(refused.nothing_was_written());
        assert_eq!(fs::read_dir(&store).unwrap().count(), 0);
    }

    /// **A tool that refuses leaves the store as it was**, with the credential
    /// that was already there untouched and nothing half-written beside it.
    #[cfg(unix)]
    #[test]
    fn a_tool_that_refuses_leaves_what_was_there() {
        let store = a_directory_of_its_own("the-tool-refuses");
        let already = store.join(THE_PERSONS_PROXY_PASSWORD);
        fs::write(&already, b"the credential that was already here").unwrap();

        let credentials = TheMachinesCredentials::at(&store, &something_that_is_not_the_tool());
        let refused = credentials
            .written(&named(), &Password::typed("hunter2").unwrap())
            .unwrap_err();
        assert_eq!(refused, NotProvisioned::TheToolRefused(Some(1)));
        assert!(refused.nothing_was_written());
        assert_eq!(
            fs::read(&already).unwrap(),
            b"the credential that was already here"
        );
        assert!(
            !store
                .join(format!("{THE_PERSONS_PROXY_PASSWORD}.writing"))
                .exists()
        );
    }

    /// **The credential this machine writes is one it can read back**, and
    /// nobody but its owner may read the file.
    ///
    /// Run against the real `systemd-creds` where the machine running the tests
    /// has one, which is what makes this a measurement rather than a
    /// description. Where it has none there is nothing to measure and the test
    /// above — a machine with nothing that encrypts — is the one that runs.
    #[cfg(unix)]
    #[test]
    fn what_this_machine_writes_it_can_read_back() {
        use std::os::unix::fs::PermissionsExt as _;

        let Some(tool) = the_real_tool() else {
            return;
        };
        let store = a_directory_of_its_own("written-and-read-back");
        let credentials = TheMachinesCredentials::at(&store, &tool);
        match credentials.written(&named(), &Password::typed("hunter2").unwrap()) {
            Ok(()) => {}
            // A machine whose host key cannot be made — an unprivileged run —
            // has nothing to measure. That the refusal leaves the store as it
            // was is the test above, which runs everywhere.
            Err(NotProvisioned::TheToolRefused(_)) => return,
            Err(why) => panic!("the tool is there and this was not its answer: {why:?}"),
        }

        let at = store.join(THE_PERSONS_PROXY_PASSWORD);
        let mode = fs::metadata(&at).unwrap().permissions().mode();
        assert_eq!(mode & 0o7777, 0o600, "{at:?}");
        assert!(
            !store
                .join(format!("{THE_PERSONS_PROXY_PASSWORD}.writing"))
                .exists()
        );

        let read = Command::new(&tool)
            .args([
                "decrypt".to_owned(),
                format!("--name={THE_PERSONS_PROXY_PASSWORD}"),
                at.display().to_string(),
                "-".to_owned(),
            ])
            .output()
            .expect("the tool decrypts");
        assert!(read.status.success(), "{read:?}");
        assert_eq!(String::from_utf8_lossy(&read.stdout), "hunter2");
    }

    /// **The store is made root's alone**, on a machine that had none.
    #[cfg(unix)]
    #[test]
    fn a_store_this_machine_makes_is_nobodys_but_its_owners() {
        use std::os::unix::fs::PermissionsExt as _;

        let root = a_directory_of_its_own("a-store-made");
        let store = root.join("credstore.encrypted");
        let credentials = TheMachinesCredentials::at(&store, &something_that_is_not_the_tool());
        let _ = credentials.written(&named(), &Password::typed("hunter2").unwrap());
        assert!(store.is_dir());
        let mode = fs::metadata(&store).unwrap().permissions().mode();
        assert_eq!(mode & 0o077, 0, "{mode:o}");
    }

    /// **Nothing formatted here carries the password** — not the writer, not a
    /// refusal, not what was handed over.
    #[test]
    fn nothing_formatted_here_carries_the_password() {
        let store = a_directory_of_its_own("formatted");
        let credentials = TheMachinesCredentials::at(&store, &something_that_is_not_the_tool());
        let password = Password::typed("hunter2").unwrap();
        let handed = handed_over(&password).unwrap();
        for formatted in [
            format!("{credentials:?}"),
            format!("{:?}", what_was_handed_over(&handed).unwrap()),
            format!("{:?}", NotProvisioned::TheToolRefused(Some(1))),
            format!("{:?}", NotHandedOver::NotAPassword(NotAPassword::Blank)),
            format!("{:?}", arguments(THE_PERSONS_PROXY_PASSWORD, &store)),
        ] {
            assert!(!formatted.contains("hunter2"), "{formatted}");
        }
    }

    /// **The store and the tool are where this machine has them**, so the
    /// constant a unit file is written against and the code cannot drift.
    #[test]
    fn the_store_and_the_tool_are_where_this_machine_has_them() {
        let credentials = TheMachinesCredentials::on_this_machine();
        assert_eq!(credentials.store(), Path::new(THE_ENCRYPTED_STORE));
        assert_eq!(credentials.tool(), Path::new(THE_TOOL));
        assert_eq!(
            credentials.store().join(THE_PERSONS_PROXY_PASSWORD),
            Path::new("/etc/credstore.encrypted/the-proxy-on-this-machine")
        );
    }
}
