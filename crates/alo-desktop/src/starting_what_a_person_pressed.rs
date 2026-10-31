//! **Starting the application a person pressed**, the way every Linux desktop
//! does.
//!
//! `alo_launching` decides; this acts on the decision over the session bus:
//!
//! - an entry that says `DBusActivatable=true` is sent `Activate` on its own bus
//!   name (`alo_portals::activated`), and the bus starts it;
//! - any other is started from its own `Exec` line, as a **transient service in
//!   the person's own systemd manager** — `StartTransientUnit`, under
//!   `app.slice`, named `app-alo-<id>@<n>.service` — handed alo's display.
//!
//! **No process is started by this desktop.** The person's manager starts the
//! program, in a unit of its own, so it is in their session and not a child of
//! the compositor: when the desktop restarts, systemd does not stop it. (Most
//! applications quit by themselves when the display they drew on goes away;
//! that is theirs to decide, and the unit does not outlive them.)
//!
//! **The agent's asks keep the road they had.** An `open` the agent's door
//! recorded is sent over the bus only, as before this file existed; see
//! `alo_shell::WhoAskedToStart`.
//!
//! # Every refusal is said
//!
//! Each press that starts nothing writes one line naming the application and
//! the reason, before the shell's own *could not start it* line — so a person
//! asking *why did Firefox not open* is answered by the journal rather than by
//! a guess.

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use alo_launching::{HowItStarts, TheEntry, WhyItCannotStart};
use alo_shell::WhoAskedToStart;
use zbus::zvariant::Value;

/// What this desktop needs to start an application, gathered once.
pub struct HowThisMachineStartsThings {
    /// The session bus, kept open: a press must not wait for a handshake.
    connection: zbus::Connection,
    /// The `applications/` directories, most specific first.
    entries: Vec<PathBuf>,
    /// What a started application is handed, if it can be handed anything.
    handed: Option<Vec<String>>,
    /// Where a bare program name is looked for: this desktop's own `PATH`.
    search: Option<OsString>,
    /// Counts the starts, so two in one moment are two units.
    starts: AtomicU64,
}

impl HowThisMachineStartsThings {
    /// Gather it: the bus, the places entries live, and the display.
    #[must_use]
    pub fn on(connection: zbus::Connection, wayland: &Path) -> Self {
        let looking = crate::what_this_machine_offers::WhereToLook::on_this_machine();
        let handed = alo_launching::what_an_application_is_handed(wayland, &looking.data_dirs);
        if handed.is_none() {
            eprintln!(
                "alo-desktop: the display at {} or the data folders are not a path an \
                 application can be handed, so an application started from its own command \
                 would have nowhere to draw and none will be",
                wayland.display()
            );
        }
        Self {
            connection,
            entries: looking.entries,
            handed,
            search: std::env::var_os("PATH"),
            starts: AtomicU64::new(0),
        }
    }

    /// Start `identifier` for whoever asked, and whether it was started.
    #[must_use]
    pub fn started(&self, identifier: &str, who: WhoAskedToStart) -> bool {
        match who {
            WhoAskedToStart::TheirAgent => self.over_the_bus(identifier),
            WhoAskedToStart::ThePerson => self.for_the_person(identifier),
        }
    }

    /// The person's road: the entry decides.
    fn for_the_person(&self, identifier: &str) -> bool {
        let Some(location) = alo_launching::where_the_entry_is(identifier, &self.entries) else {
            return said(identifier, "no entry by that name is installed");
        };
        let Some(entry) = std::fs::read_to_string(&location)
            .ok()
            .as_deref()
            .and_then(TheEntry::read)
        else {
            return said(
                identifier,
                &format!("its entry at {} could not be read", location.display()),
            );
        };
        match alo_launching::how_it_starts(identifier, &entry, location.to_str()) {
            Ok(HowItStarts::OverTheBus { or_else }) => {
                if self.over_the_bus(identifier) {
                    return true;
                }
                let Some(command) = or_else else {
                    return said(
                        identifier,
                        "the bus could not start it and its entry has no command of its own",
                    );
                };
                eprintln!(
                    "alo-desktop: the bus could not start {identifier}, so it is started from \
                     its own entry's command instead"
                );
                self.started_from_its_command(identifier, &entry, &command)
            }
            Ok(HowItStarts::FromItsCommand(command)) => {
                self.started_from_its_command(identifier, &entry, &command)
            }
            Err(why) => said(identifier, &the_reason(&why)),
        }
    }

    /// `Activate` on the application's own bus name.
    fn over_the_bus(&self, identifier: &str) -> bool {
        zbus::block_on(alo_portals::activated(&self.connection, identifier))
    }

    /// A transient service in the person's own manager, running this command.
    fn started_from_its_command(
        &self,
        identifier: &str,
        entry: &TheEntry,
        command: &[String],
    ) -> bool {
        let Some(handed) = self.handed.clone() else {
            return said(identifier, "there is no display to hand it");
        };
        let Some((first, _)) = command.split_first() else {
            return said(identifier, "its command names nothing");
        };
        let Some(program) = alo_launching::the_program(first, self.search.as_deref(), is_runnable)
        else {
            return said(
                identifier,
                &format!("its command names {first}, which is not a program on this machine"),
            );
        };
        let Some(program) = program.to_str().map(str::to_owned) else {
            return said(identifier, "its program's path is not text a unit can hold");
        };
        let Some(unit) = alo_launching::the_unit_for(identifier, self.distinct()) else {
            return said(identifier, "its name is too long for a unit");
        };
        let folder = entry
            .folder
            .as_deref()
            .filter(|it| it.starts_with('/'))
            .unwrap_or("~");
        let properties: Vec<(&str, Value<'_>)> = vec![
            (
                "Description",
                Value::from(format!("{identifier}, started from the alo desktop")),
            ),
            (
                "ExecStart",
                Value::from(vec![(program, command.to_vec(), false)]),
            ),
            ("Environment", Value::from(handed)),
            ("WorkingDirectory", Value::from(folder)),
            ("Slice", Value::from("app.slice")),
            ("Type", Value::from("exec")),
            // An application whose first process hands over to another and
            // exits — LibreOffice's launcher does — is still running.
            ("ExitType", Value::from("cgroup")),
            ("CollectMode", Value::from("inactive-or-failed")),
        ];
        let auxiliary: Vec<(&str, Vec<(&str, Value<'_>)>)> = Vec::new();
        let asked = zbus::block_on(self.connection.call_method(
            Some("org.freedesktop.systemd1"),
            "/org/freedesktop/systemd1",
            Some("org.freedesktop.systemd1.Manager"),
            "StartTransientUnit",
            &(unit.as_str(), "fail", properties, auxiliary),
        ));
        match asked {
            Ok(_) => {
                eprintln!("alo-desktop: {identifier} was started as {unit}");
                true
            }
            Err(why) => said(
                identifier,
                &format!("the person's systemd manager would not start {unit}: {why}"),
            ),
        }
    }

    /// A number no other start from this desktop has had.
    fn distinct(&self) -> u64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |it| u64::try_from(it.as_nanos()).unwrap_or(u64::MAX));
        now.wrapping_add(self.starts.fetch_add(1, Ordering::Relaxed))
    }
}

/// Whether the file at `path` is one somebody may run.
fn is_runnable(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|it| it.is_file() && it.permissions().mode() & 0o111 != 0)
}

/// Say why `identifier` was not started, and answer that it was not.
fn said(identifier: &str, why: &str) -> bool {
    eprintln!("alo-desktop: {identifier} was not started: {why}");
    false
}

/// A refusal, in the words the journal says it in.
fn the_reason(why: &WhyItCannotStart) -> String {
    match why {
        WhyItCannotStart::Deleted => "its entry says it is deleted (Hidden=true)".to_owned(),
        WhyItCannotStart::NotAnApplication => "its entry is not an application".to_owned(),
        WhyItCannotStart::NeedsATerminal => {
            "its entry asks for a terminal (Terminal=true), and a press opens none".to_owned()
        }
        WhyItCannotStart::NothingToStartItWith => {
            "its entry has no Exec line and is not started over the bus".to_owned()
        }
        WhyItCannotStart::ItsCommandIsMalformed(what) => {
            format!("its Exec line is not one the specification allows: {what:?}")
        }
    }
}
