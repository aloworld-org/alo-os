//! The half that is really a machine: `systemd-logind` over the system bus,
//! and the accounts file on the disk.
//!
//! Both are one method wide, and both implement a trait whose other
//! implementations are fixtures — so everything this crate *decides* is decided
//! in `crate::opening`, on any host, in the gate. What is here is the part that
//! can only be shown working on a Linux machine with a bus.
//!
//! # The call, and the measurement behind every argument
//!
//! ADR 0024 asked whether `logind` will open a session for a caller that is not
//! `pam_systemd`. It will, provided the caller is privileged:
//! `CreateSession` answers root with *Leader PID is not valid* — the call
//! authorised, and only its contents refused — and an unprivileged caller with
//! *Access denied*. That was measured on the development machine before the
//! decision was taken, and **again on the pinned Fedora base** while this was
//! built; `docs/quirks.md` has both, including the one thing that differs
//! between them.
//!
//! So no PAM module is owed, no C ABI, and no `unsafe` exemption on the
//! authentication path — which was the price ADR 0024 priced for the other
//! option.
//!
//! # The seat, the VT, and what this component still does not decide
//!
//! **The seat and the virtual terminal, which were empty and zero until
//! 2026-10-14.** They are now `seat0` and `1`, and the paragraph that stood
//! here said why they were not:
//!
//! > A seat and a VT are facts about *what draws*, and this component must
//! > not draw, start a compositor, or decide what a person sees [...] Both
//! > are additive to change — a later task that starts a compositor is the
//! > one that knows which VT it is on, and it will be changing an argument
//! > here rather than adding a door.
//!
//! That was right, and this is that later task. A booted machine showed the
//! cost of the seatless session: a person made their first account, two
//! sessions were opened for them, and `alo-desktop` found no seat six times
//! over. [ADR 0115](../../../docs/decisions/0115-the-desktop-runs-inside-the-session-alo-sessiond-opened.md)
//! has the measurement, including the half of it this change does not fix:
//! a seated session still answers nothing for a process `logind` cannot place
//! in it, and putting the desktop inside this one is its own change.
//!
//! **The session type is still `unspecified`**, and that is now a narrower
//! claim than it was. It said *the compositor it would be asserting about has
//! no binary yet*, which has stopped being true. What is still true is that
//! this component does not assert which protocol draws; the seat is what a
//! card is lent from, and the type is not.
//!
//! # The session lasts as long as this process holds it
//!
//! `CreateSession` answers with a descriptor, and the session lives while
//! somebody holds it. [`TheMachinesLogind`] keeps it, which is why
//! [`crate::Logind`] takes `&mut self`: signing out is this process letting go,
//! and there is nothing else it could be.

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use alo_accounts::THE_ACCOUNTS;
use zbus::blocking::Connection;
use zbus::zvariant::{OwnedFd, OwnedObjectPath, Value};

use crate::accounts::TheAccounts;
use crate::logind::{Logind, TheSessionOpened};
use crate::refusing::{NotAsked, NotOpened};
use crate::the_persons_desktop::{NotStarted, StartsADesktop, TheDesktopToStart};

/// The service that opens sessions on a systemd machine.
const LOGIND: &str = "org.freedesktop.login1";

/// The object its manager is at.
const THE_MANAGER: &str = "/org/freedesktop/login1";

/// The interface the one method is on.
const THE_INTERFACE: &str = "org.freedesktop.login1.Manager";

/// The one method this component calls, on the whole machine.
const CREATE_SESSION: &str = "CreateSession";

/// What this component calls itself where `logind` records who asked.
const US: &str = "alo-sessiond";

/// The class of session a person signing in gets.
const A_PERSONS: &str = "user";

/// What this component will not claim about what is drawing.
const NOT_SAID: &str = "unspecified";

/// The seat a person sits at, named rather than left empty.
///
/// **This was the empty string until 2026-10-14, and that was half of why a
/// desktop could not take the card.** A session created with no seat is a
/// session with no seat to lend, so `libseat` would have found nothing even
/// once the desktop was inside it. The module header predicted this change in
/// as many words: *a later task that starts a compositor is the one that knows
/// which VT it is on, and it will be changing an argument here rather than
/// adding a door.* This is that task.
///
/// One seat, because alo OS certifies one machine rather than every machine
/// (ADR 0002). A machine with two seats is a decision nobody has taken.
const THE_SEAT: &str = "seat0";

/// The virtual terminal that seat draws on.
///
/// One, because that is what the image puts a screen on: both
/// `alo-compositor.service` and `alo-desktop.service` carry
/// `TTYPath=/dev/tty1`, and `getty@tty1` is masked in the recipe so nothing
/// else owns it. The greeter measured what happens without it: *a session
/// with no TTY is given no seat -- logind answered seat= vtnr=0. With it, the
/// same machine answered seat=seat0 vtnr=1.*
const THE_VT: u32 = 1;

/// The terminal that VT is, as logind spells it.
///
/// Passed because a seated session is expected to name one, and the greeter
/// gets its seat by way of `pam_systemd` reading exactly this from `PAM_TTY`.
const THE_TERMINAL: &str = "tty1";

/// `systemd-logind`, on this machine's system bus.
#[derive(Debug, Default)]
pub struct TheMachinesLogind {
    /// What holds the session open, once there is one.
    ///
    /// Never read, and that is the whole of its job: the session lasts while
    /// this is alive, so it is dropped when this process ends and not before.
    held: Option<OwnedFd>,
}

impl Logind for TheMachinesLogind {
    fn open_a_session_for(&mut self, person: u32) -> Result<TheSessionOpened, NotOpened> {
        let connection = Connection::system().map_err(|why| NotOpened::NoBus {
            why: why.to_string(),
        })?;

        let nothing: Vec<(String, Value<'_>)> = Vec::new();
        let reply = connection
            .call_method(
                Some(LOGIND),
                THE_MANAGER,
                Some(THE_INTERFACE),
                CREATE_SESSION,
                &(
                    person,
                    std::process::id(),
                    US,
                    NOT_SAID,
                    A_PERSONS,
                    "",
                    THE_SEAT,
                    THE_VT,
                    THE_TERMINAL,
                    "",
                    false,
                    "",
                    "",
                    nothing,
                ),
            )
            .map_err(|why| refused(person, &why))?;

        let (_called, _object, runtime, held, _uid, _seat, _vtnr, _existing): (
            String,
            OwnedObjectPath,
            String,
            OwnedFd,
            u32,
            String,
            u32,
            bool,
        ) = reply
            .body()
            .deserialize()
            .map_err(|why| NotOpened::NotUnderstood {
                why: why.to_string(),
            })?;

        self.held = Some(held);
        Ok(TheSessionOpened::with(PathBuf::from(runtime)))
    }
}

/// A call the bus would not carry out, with the name to decide on beside the
/// sentence to read.
///
/// `zbus::Error::MethodError` is the only shape that carries a D-Bus error name;
/// everything else — a connection that dropped, a reply that never came — has
/// none, and an empty name is how this says so rather than inventing one.
fn refused(person: u32, why: &zbus::Error) -> NotOpened {
    let named = match why {
        zbus::Error::MethodError(name, _, _) => name.as_str().to_owned(),
        _ => String::new(),
    };
    NotOpened::Refused {
        person,
        named,
        why: why.to_string(),
    }
}

/// The accounts this machine has, as the file `alo-accounts` keeps.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheMachinesAccounts;

impl TheAccounts for TheMachinesAccounts {
    /// Whether this machine has an account with this number.
    ///
    /// **A machine with no accounts file is a machine with no accounts**, and
    /// not a failure. That is the state every alo OS arrives in — the image
    /// ships no store, which ADR 0024 settled and `crates/alo-image` tests —
    /// so a first boot answers *no* here rather than *this machine is broken*.
    /// Every other way the file can refuse is carried, because a store that
    /// somebody else could have written is one `alo-accounts` will not believe
    /// and this component must not believe either.
    fn an_account_numbered(&self, person: u32) -> Result<bool, NotAsked> {
        match alo_accounts::found(Path::new(THE_ACCOUNTS)) {
            Ok(accounts) => Ok(accounts.numbers(person)),
            Err(alo_accounts::NotKept::NotThere { .. }) => Ok(false),
            Err(why) => Err(NotAsked::about(THE_ACCOUNTS, &why)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The one method, the one interface and the one object are written
    /// once.** A privileged component whose bus address was assembled in two
    /// places would be one where a typo is a call to something else.
    #[test]
    fn there_is_one_method_this_component_can_call() {
        assert_eq!(LOGIND, "org.freedesktop.login1");
        assert_eq!(THE_MANAGER, "/org/freedesktop/login1");
        assert_eq!(THE_INTERFACE, "org.freedesktop.login1.Manager");
        assert_eq!(CREATE_SESSION, "CreateSession");
    }

    /// **The session is created with a seat, and an empty one here is the
    /// fault this test exists to stop coming back.**
    ///
    /// Until 2026-10-14 the seat was the empty string and the VT was zero, so
    /// every session this machine opened was a session with no seat to lend.
    /// A booted machine showed what that costs: a person made their first
    /// account, two sessions were opened for them, and `alo-desktop` found no
    /// seat six times over while PAM reported success each time.
    ///
    /// This asserts the values **and that they are not empty**, because the
    /// way this regresses is somebody restoring a default, not somebody
    /// choosing a different seat.
    #[test]
    fn the_session_this_opens_has_a_seat_to_lend() {
        assert_eq!(THE_SEAT, "seat0");
        assert_eq!(THE_VT, 1);
        assert_eq!(THE_TERMINAL, "tty1");

        assert!(
            !THE_SEAT.is_empty(),
            "an empty seat is what logind is told when nobody names one, and it is the \n             state this change ended: libseat asks a session for its seat and an empty \n             one has none to give"
        );
        assert_ne!(
            THE_VT, 0,
            "vtnr zero is logind's word for no virtual terminal, and the greeter measured \n             that a session with no TTY is given no seat at all"
        );
        assert!(!THE_TERMINAL.is_empty());
    }

    /// **A machine with no accounts file has no accounts**, rather than an
    /// unreadable one. This is the first boot of every alo OS there will ever
    /// be, and answering it as a failure would mean the first person to make an
    /// account could not then sign in with it.
    #[test]
    fn a_machine_with_no_accounts_file_answers_that_nobody_is_numbered() {
        // The real path, which on a machine running these tests is not there.
        // If it *is* there this asserts nothing false: the question is whether
        // a missing file is an error, and a present one simply answers.
        assert!(TheMachinesAccounts.an_account_numbered(1000).is_ok());
    }
}

/// What really starts a desktop on this machine.
///
/// # Why a child, and what that buys
///
/// `CreateSession` is passed this process's own pid as the session's leader,
/// so `logind` puts **this** process in `session-N.scope`. A child started
/// after that call inherits the control group, which is the question `logind`
/// answers when `libseat` asks *which session is this process in*. The order
/// is the whole mechanism and it is not written down anywhere the compiler can
/// see it: open the session, then start the desktop.
///
/// # Law 2, and why this is not its exec
///
/// `THE_DESKTOP` is a constant. No message, no configuration file and no
/// account names the program; what varies is who it runs as and which
/// directories are theirs, and both come from the session rather than from a
/// caller. There are no arguments at all.
#[derive(Debug, Default)]
pub struct TheMachineStartsIt;

impl StartsADesktop for TheMachineStartsIt {
    fn start(&mut self, what: &TheDesktopToStart) -> Result<(), NotStarted> {
        let refused = |why: String| NotStarted::TheMachineRefused {
            program: what.program().display().to_string(),
            person: what.person(),
            why,
        };

        // **The socket's directory is made before the desktop looks for it**,
        // as the unit file's `RuntimeDirectory=` did, and with the mode that
        // line carried: a desktop whose socket a second account could reach is
        // a desktop that is not this person's.
        std::fs::create_dir_all(what.runtime()).map_err(|why| refused(why.to_string()))?;
        std::fs::set_permissions(what.runtime(), Permissions::from_mode(0o700))
            .map_err(|why| refused(why.to_string()))?;
        std::os::unix::fs::chown(what.runtime(), Some(what.person()), None)
            .map_err(|why| refused(why.to_string()))?;

        // **`uid` and `gid` are safe calls**, which is the whole reason this
        // is a `Command` rather than a fork: `pre_exec` is the unsafe one and
        // nothing here needs it. ADR 0024 refused to pay an `unsafe` exemption
        // on the authentication path, and this does not spend one either.
        Command::new(what.program())
            .uid(what.person())
            .gid(what.person())
            .env("HOME", what.home())
            .env("RUNTIME_DIRECTORY", what.runtime())
            .env(
                "XDG_RUNTIME_DIR",
                what.runtime().parent().unwrap_or(what.runtime()),
            )
            .spawn()
            .map(drop)
            .map_err(|why| refused(why.to_string()))
    }
}
