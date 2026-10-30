//! The units a sign-in starts for a person, over systemd's system bus.
//!
//! `crate::their_folder` and `crate::their_name` are the decisions — whether
//! each is asked for and in what order — and run on any host. This is the part
//! that needs a machine, and it is `cfg(target_os = "linux")` for
//! `crate::machine`'s reason: there is no unit to start anywhere else.
//!
//! **Three units, and one reason to change.** Each is a constant of this
//! repository instanced by an account number, started the same way:
//! [`THE_UNIT`] makes the person their place for what an undo would put
//! back ([ADR 0121](../../../docs/decisions/0121-the-thing-that-opens-a-session-makes-the-persons-folder.md)),
//! [`THE_NAMES_UNIT`] gives them their own name where their desktop can
//! read it ([ADR 0135](../../../docs/decisions/0135-a-persons-name-reaches-their-desktop-through-a-file-only-they-can-read.md)),
//! and [`THE_DESKTOPS_UNIT`] is their desktop, inside the login session its own
//! PAM stack opens ([ADR 0141](../../../docs/decisions/0141-the-desktop-is-put-inside-the-persons-session-by-what-already-has-the-privilege.md)).
//! The first two are waited for until they finish, the third until it runs.
//! This file was `making_a_folder.rs` until the second arrived; what it does —
//! *start a unit named by a constant and a number, and wait for it* — did not
//! change, so it was renamed rather than split.
//!
//! # Its own file, because `machine.rs` had gained a second reason to change
//!
//! `crate::machine` is `logind` and the accounts file. This is systemd's
//! manager, a different service with different methods and a different reason
//! to be edited. `CLAUDE.md` law 4 — *a file that gains a second reason to
//! change gets split in the same change that discovered it* — and this is that
//! split, made when the two were briefly in one file and it was 475 lines.
//!
//! # What it may say to systemd, and why that is a list
//!
//! Three methods, all of them read-or-start, in [`EVERY_METHOD`]. **Nothing
//! here stops, kills, masks, reloads, enables or writes a unit.** The list is a
//! constant and [`call`] refuses anything not on it, so the list cannot fall
//! out of date with the code beneath it — and
//! `tests/this_component_only_starts_its_own_units.rs` reads this file for the
//! verbs that are missing.
//!
//! This is `alo-brokerd/src/units.rs`' shape, deliberately. That component made
//! the same decision for the same reason, and two privileged components talking
//! to systemd in two different shapes is one of them being reviewed less
//! carefully than the other.

use zbus::blocking::Connection;
use zbus::zvariant::{OwnedObjectPath, OwnedValue};

/// systemd itself, which is what starts the units a sign-in asks for.
const SYSTEMD: &str = "org.freedesktop.systemd1";

/// The object systemd's manager is at.
const SYSTEMDS_MANAGER: &str = "/org/freedesktop/systemd1";

/// The interface the two methods are on.
const SYSTEMDS_INTERFACE: &str = "org.freedesktop.systemd1.Manager";

/// Reading one property of one object.
const PROPERTIES: &str = "org.freedesktop.DBus.Properties";

/// The interface a service's own result is on.
const SERVICE: &str = "org.freedesktop.systemd1.Service";

/// Starting a unit.
const START_UNIT: &str = "StartUnit";

/// Finding a started unit's object, so its result can be read.
const GET_UNIT: &str = "GetUnit";

/// Reading one property.
const GET: &str = "Get";

/// Every method this component calls on systemd, and there is no other.
///
/// Three, all of them read-or-start. **Nothing here stops, kills, masks,
/// reloads, enables or writes a unit**, and
/// `tests/this_component_only_starts_its_own_units.rs` reads this file and
/// holds it to that — the same shape `alo-brokerd/src/units.rs` uses, for the
/// same reason: a privileged component that could be talked into *any* systemd
/// verb is a privileged component with a shell in it.
pub const EVERY_METHOD: [&str; 3] = [START_UNIT, GET_UNIT, GET];

/// The unit that makes a person their folder, before the `@` that carries the
/// number.
///
/// **A constant of this repository and not a name from anywhere.**
/// [`TheirFolder::made_for`](crate::TheirFolder::made_for) builds the instance
/// from a `u32` here, so there is no spelling of *start whatever unit this
/// caller happens to name* — which would be `CLAUDE.md` law 2 by the back door,
/// this component running an arbitrary command because systemd was willing to
/// look the name up.
///
/// `crates/alo-image` is what holds this against the unit the image actually
/// ships, because a constant agreeing with another constant proves nothing.
pub const THE_UNIT: &str = "alo-giving-a-folder";

/// The unit that gives a person their own name, before the `@` that carries
/// the number.
///
/// A constant for [`THE_UNIT`]'s reason, instanced from a `u32` by
/// [`TheirName::given_to`](crate::TheirName::given_to) — and these two, with
/// [`THE_DESKTOPS_UNIT`], are the whole of what this component can ask systemd
/// to start.
pub const THE_NAMES_UNIT: &str = "alo-giving-a-name";

/// The person's desktop, before the `@` that carries the number.
///
/// **The third unit, and the one a sign-in is for** (ADR 0141). Its PAM stack
/// opens the person's login session and pid 1 becomes them, so the desktop is
/// the process `logind` puts in the session's scope — which is what makes
/// restart, power off and the time zone theirs without a password. A constant
/// for [`THE_UNIT`]'s reason, instanced from a `u32` by
/// [`TheirDesktop::started_for`](crate::TheirDesktop::started_for).
pub const THE_DESKTOPS_UNIT: &str = "alo-desktop";

/// How systemd is asked to enqueue the job: fail rather than disturb anything
/// already going on.
const MODE: &str = "fail";

/// How long one bus call may take.
const ANSWERING: std::time::Duration = std::time::Duration::from_secs(10);

/// How long one unit may take before the person is signed in regardless.
///
/// Each unit makes a directory or writes one line, and changes one owner. Ten
/// seconds is three orders of magnitude more than that needs, and it is a
/// **ceiling on how long a person waits at a sign-in screen** rather than an
/// estimate of the work — which is why it is short and why running out of it is
/// not an error about the folder or the name, it is a decision not to keep
/// somebody standing there.
const LONGEST_A_PERSON_WAITS: std::time::Duration = std::time::Duration::from_secs(10);

/// How often systemd is asked whether the unit has finished.
const ASKED_AGAIN_EVERY: std::time::Duration = std::time::Duration::from_millis(50);

/// systemd, asked to make a person their place for what an undo would put back.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheMachinesFolders;

impl crate::their_folder::TheirFolder for TheMachinesFolders {
    fn made_for(&self, person: u32) -> Result<(), crate::refusing::NotTheirs> {
        started_and_finished(person, &format!("{THE_UNIT}@{person}.service"))
    }
}

/// systemd, asked to give a person their own name where their desktop can
/// read it.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheMachinesNames;

impl crate::their_name::TheirName for TheMachinesNames {
    fn given_to(&self, person: u32) -> Result<(), crate::refusing::NotTheirs> {
        started_and_finished(person, &format!("{THE_NAMES_UNIT}@{person}.service"))
    }
}

/// How long a person waits at the sign-in screen for their desktop to run.
///
/// Longer than [`LONGEST_A_PERSON_WAITS`], because this one includes PAM
/// opening the session and `logind` starting the person's own manager, which
/// on a cold machine is seconds rather than milliseconds.
const LONGEST_A_DESKTOP_TAKES: std::time::Duration = std::time::Duration::from_secs(30);

/// systemd, asked to start a person's desktop inside a session of its own.
#[derive(Debug, Clone, Copy, Default)]
pub struct TheMachinesDesktops;

impl crate::their_desktop::TheirDesktop for TheMachinesDesktops {
    fn started_for(&mut self, person: u32) -> Result<(), crate::refusing::NotOpened> {
        started_and_running(person, &the_desktop_of(person))
    }

    fn is_running_for(&self, person: u32) -> Result<bool, crate::refusing::NotOpened> {
        still_running(person, &the_desktop_of(person))
    }
}

/// The desktop unit's instance for one account, built in one place.
fn the_desktop_of(person: u32) -> String {
    format!("{THE_DESKTOPS_UNIT}@{person}.service")
}

/// The system bus, with this component's ceiling on one call.
fn the_bus() -> Result<Connection, String> {
    zbus::blocking::connection::Builder::system()
        .map(|building| building.method_timeout(ANSWERING))
        .and_then(zbus::blocking::connection::Builder::build)
        .map_err(|why| why.to_string())
}

/// Start the desktop's unit, and wait until it runs.
///
/// Private, and handed only the name [`TheMachinesDesktops`] built from a
/// constant and a `u32`.
fn started_and_running(person: u32, named: &str) -> Result<(), crate::refusing::NotOpened> {
    use crate::refusing::NotOpened;
    use crate::their_desktop::Starting;

    let bus = the_bus().map_err(|why| NotOpened::NoBus { why })?;
    let refused = |why: String| NotOpened::Refused {
        person,
        named: the_error_name(&why),
        why,
    };
    let _job: OwnedObjectPath = call(
        &bus,
        SYSTEMDS_MANAGER,
        SYSTEMDS_INTERFACE,
        START_UNIT,
        &(named, MODE),
    )
    .map_err(refused)?;
    let at: OwnedObjectPath = call(
        &bus,
        SYSTEMDS_MANAGER,
        SYSTEMDS_INTERFACE,
        GET_UNIT,
        &(named,),
    )
    .map_err(refused)?;

    let until = std::time::Instant::now() + LONGEST_A_DESKTOP_TAKES;
    loop {
        let state: String = property(&bus, &at, "org.freedesktop.systemd1.Unit", "ActiveState")
            .map_err(|why| NotOpened::NotUnderstood { why })?;
        let result: String = property(&bus, &at, SERVICE, "Result")
            .map_err(|why| NotOpened::NotUnderstood { why })?;
        match Starting::from_systemd(&state, &result) {
            Starting::Running => return Ok(()),
            Starting::Failed(said) => {
                return Err(NotOpened::NotRunning {
                    person,
                    unit: named.to_owned(),
                    said,
                });
            }
            Starting::StillGoing => {}
        }
        if std::time::Instant::now() >= until {
            if Starting::at_the_longest_a_person_waits(&state) {
                eprintln!(
                    "alo-sessiond: {named} is still starting after {} seconds; it is let \
                     through, and its own OnFailure= puts the sign-in screen back if it never runs",
                    LONGEST_A_DESKTOP_TAKES.as_secs()
                );
                return Ok(());
            }
            return Err(NotOpened::NotRunning {
                person,
                unit: named.to_owned(),
                said: format!(
                    "it was still {state} after {} seconds",
                    LONGEST_A_DESKTOP_TAKES.as_secs()
                ),
            });
        }
        std::thread::sleep(ASKED_AGAIN_EVERY);
    }
}

/// Whether the desktop's unit has not ended.
///
/// **A unit systemd no longer has loaded is one that ended**: an inactive
/// instance is let go of, and `GetUnit` then answers `NoSuchUnit`.
fn still_running(person: u32, named: &str) -> Result<bool, crate::refusing::NotOpened> {
    use crate::refusing::NotOpened;

    let bus = the_bus().map_err(|why| NotOpened::NoBus { why })?;
    let at: OwnedObjectPath = match call(
        &bus,
        SYSTEMDS_MANAGER,
        SYSTEMDS_INTERFACE,
        GET_UNIT,
        &(named,),
    ) {
        Ok(at) => at,
        Err(why) if why.contains(NO_SUCH_UNIT) => return Ok(false),
        Err(why) => {
            return Err(NotOpened::Refused {
                person,
                named: the_error_name(&why),
                why,
            });
        }
    };
    let state: String = property(&bus, &at, "org.freedesktop.systemd1.Unit", "ActiveState")
        .map_err(|why| NotOpened::NotUnderstood { why })?;
    Ok(crate::their_desktop::has_not_ended(&state))
}

/// systemd's error name for a unit it does not have loaded.
const NO_SUCH_UNIT: &str = "org.freedesktop.systemd1.NoSuchUnit";

/// The D-Bus error name in a sentence `call` made, or empty where there is none.
fn the_error_name(why: &str) -> String {
    why.split_whitespace()
        .find(|word| word.starts_with("org.freedesktop."))
        .unwrap_or_default()
        .trim_end_matches(':')
        .to_owned()
}

/// Start one of this component's two helper units for `person`, and wait for
/// it to finish.
///
/// Private, and handed only names the two impls above built from a constant and
/// a `u32`: there is no public road from a caller's text to this.
fn started_and_finished(person: u32, named: &str) -> Result<(), crate::refusing::NotTheirs> {
    let bus = the_bus().map_err(|why| crate::refusing::NotTheirs::NoBus {
        person,
        unit: named.to_owned(),
        why,
    })?;

    let _job: OwnedObjectPath = call(
        &bus,
        SYSTEMDS_MANAGER,
        SYSTEMDS_INTERFACE,
        START_UNIT,
        &(named, MODE),
    )
    .map_err(|why| refused_to_start(person, named, &why))?;

    waited_for(&bus, person, named)
}

/// Wait for the unit to finish and read what it did.
///
/// **Waited for rather than fired and forgotten**, because what each unit makes
/// has to exist before the session that uses it: `StartUnit` answers as soon as
/// the job is enqueued, which says nothing at all about whether a directory or a
/// name is there.
fn waited_for(
    bus: &Connection,
    person: u32,
    named: &str,
) -> Result<(), crate::refusing::NotTheirs> {
    let at: OwnedObjectPath = call(
        bus,
        SYSTEMDS_MANAGER,
        SYSTEMDS_INTERFACE,
        GET_UNIT,
        &(named,),
    )
    .map_err(|why| refused_to_start(person, named, &why))?;

    let until = std::time::Instant::now() + LONGEST_A_PERSON_WAITS;
    loop {
        let state: String = property(bus, &at, "org.freedesktop.systemd1.Unit", "ActiveState")
            .map_err(|why| not_finished(person, named, &why))?;
        // `oneshot` with `RemainAfterExit=no` ends as `inactive` on success and
        // `failed` on anything else, so those two are the only resting states
        // and everything else means it is still going.
        if state != "activating" && state != "deactivating" && state != "reloading" {
            let result: String = property(bus, &at, SERVICE, "Result")
                .map_err(|why| not_finished(person, named, &why))?;
            return if result == "success" {
                Ok(())
            } else {
                Err(crate::refusing::NotTheirs::NotFinished {
                    person,
                    unit: named.to_owned(),
                    said: result,
                })
            };
        }
        if std::time::Instant::now() >= until {
            return Err(crate::refusing::NotTheirs::NotFinished {
                person,
                unit: named.to_owned(),
                said: format!(
                    "it was still {state} after {} seconds, and a person is not kept waiting \
                     longer than that",
                    LONGEST_A_PERSON_WAITS.as_secs()
                ),
            });
        }
        std::thread::sleep(ASKED_AGAIN_EVERY);
    }
}

/// One property of one object, as `T`.
fn property<T>(
    bus: &Connection,
    at: &OwnedObjectPath,
    interface: &str,
    named: &str,
) -> Result<T, String>
where
    T: TryFrom<OwnedValue>,
{
    let value: OwnedValue = call(bus, at.as_str(), PROPERTIES, GET, &(interface, named))?;
    T::try_from(value).map_err(|_| format!("{named} was not what systemd says it is"))
}

/// Call one of [`EVERY_METHOD`], and read its answer as `T`.
///
/// The guard is not decoration: it is what makes [`EVERY_METHOD`] the whole
/// list rather than a comment that fell out of date. A method not on it cannot
/// be sent from here even by a mistake in this file.
fn call<T, B>(
    bus: &Connection,
    at: &str,
    interface: &str,
    method: &str,
    body: &B,
) -> Result<T, String>
where
    T: for<'d> zbus::export::serde::Deserialize<'d> + zbus::zvariant::Type,
    B: zbus::export::serde::Serialize + zbus::zvariant::DynamicType,
{
    if !EVERY_METHOD.contains(&method) {
        return Err(format!("{method} is not a method alo-sessiond calls"));
    }
    bus.call_method(Some(SYSTEMD), at, Some(interface), method, body)
        .map_err(|why| format!("{method}: {why}"))?
        .body()
        .deserialize::<T>()
        .map_err(|why| format!("{method}: {why}"))
}

/// A unit systemd would not start, with the name to decide on beside the
/// sentence to read.
fn refused_to_start(person: u32, named: &str, why: &str) -> crate::refusing::NotTheirs {
    // The D-Bus error name, where the failure carried one. `call` folds the
    // method into the sentence, so the name is recovered from the text rather
    // than from the error — which is why this looks for the prefix every
    // `zbus::Error::MethodError` sentence begins with.
    let name = why
        .split_whitespace()
        .find(|word| word.starts_with("org.freedesktop.DBus.Error."))
        .unwrap_or_default()
        .trim_end_matches(':')
        .to_owned();
    crate::refusing::NotTheirs::Refused {
        person,
        unit: named.to_owned(),
        named: name,
        why: why.to_owned(),
    }
}

/// A unit that started and whose result could not be read, or was not success.
fn not_finished(person: u32, named: &str, why: &str) -> crate::refusing::NotTheirs {
    crate::refusing::NotTheirs::NotFinished {
        person,
        unit: named.to_owned(),
        said: why.to_owned(),
    }
}
