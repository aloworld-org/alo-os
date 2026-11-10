//! **alo's own copies of applications**: started, found again, and told apart
//! from the person's own windows
//! ([ADR 0143](../../../docs/decisions/0143-alo-works-inside-applications-and-may-post-in-the-persons-name.md)
//! Q6 and the owner's answers of 2026-11-10).
//!
//! `alo_launching::alos_copy` decides which applications have a copy of their
//! own and with what arguments; this starts one and answers which control
//! groups it runs in, so `alo_shell::alo_at_work` counts it and nothing of the
//! person's.
//!
//! # Why this desktop starts the process itself
//!
//! A person's press is started by their systemd manager as a service
//! (`starting_what_a_person_pressed`), and this desktop never knows its process.
//! **alo's copy has to be known by its process**, because Flatpak moves every
//! application it runs into a scope named `app-flatpak-<id>-<instance>.scope`
//! (Flatpak's own `flatpak_run_in_transient_unit`, read 2026-11-10), and nothing
//! in that name says whose start it was. The process this desktop started is
//! the one Flatpak moves, so `/proc/<it>/cgroup` names alo's copy's scope and
//! nobody else's.
//!
//! So the process is started here, from the installed entry's own `Exec` line
//! with the copy's arguments after it, and handed at once to the person's own
//! manager as a scope of its own, `app-alocopy-<id>@<n>.scope`. The process is
//! held unreaped while it runs, so its number cannot be given to another
//! process while this desktop is still reading it.
//!
//! **Its number is also written down** in the person's runtime folder, with
//! when the process began, so a desktop that restarts finds the copy that is
//! still running rather than starting a second over the same profile.
//!
//! # What has not been measured
//!
//! **None of this has run on an alo machine.** Which scope a Flatpak copy is
//! moved into is read from Flatpak's source, and each application's arguments
//! from its own documentation; stage 1 of the Demo 2 plan is where both are
//! measured. A copy whose groups cannot be found is shown as one this machine
//! cannot count, so a wrong guess here is said on the indicator rather than
//! hidden.

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::os::unix::fs::{DirBuilderExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use alo_launching::{HowItStarts, TheEntry};
use alo_shell::alo_at_work::{NoCopy, StartsAlosCopy};
use zbus::zvariant::Value;

/// Where the person's own control groups are.
const CGROUPS: &str = "/sys/fs/cgroup";

/// One of alo's copies, running.
struct Running {
    /// Its process — the one Flatpak moves into its scope.
    pid: u32,
    /// When that process began, in the kernel's ticks since boot, so a number
    /// given to a later process is never read as this one.
    began: u64,
    /// The process, when this desktop started it: held so it is reaped.
    child: Option<Child>,
    /// The scope it was handed to, if the manager took it.
    unit: Option<String>,
}

/// alo's copies on this machine.
pub struct AlosCopies {
    /// The session bus, for the person's own manager.
    connection: zbus::Connection,
    /// The `applications/` directories, most specific first.
    entries: Vec<PathBuf>,
    /// What a started application is handed.
    handed: Option<Vec<String>>,
    /// Where a bare program name is looked for.
    search: Option<OsString>,
    /// The person's home.
    home: Option<PathBuf>,
    /// The person's manager's `app.slice`.
    slice: PathBuf,
    /// Where the copies' numbers are written down.
    written: Option<PathBuf>,
    /// The copies running now, by application.
    running: BTreeMap<String, Running>,
    /// Counts the starts, so two are two scopes.
    starts: u64,
}

impl AlosCopies {
    /// alo's copies, started over this bus and drawing on `wayland`.
    #[must_use]
    pub fn on(connection: zbus::Connection, wayland: &Path) -> Self {
        let looking = crate::what_this_machine_offers::WhereToLook::on_this_machine();
        let person = rustix::process::geteuid().as_raw();
        Self {
            connection,
            entries: looking.entries,
            handed: alo_launching::what_an_application_is_handed(wayland, &looking.data_dirs),
            search: std::env::var_os("PATH"),
            home: std::env::var_os("HOME").map(PathBuf::from),
            slice: Path::new(CGROUPS)
                .join("user.slice")
                .join(format!("user-{person}.slice"))
                .join(format!("user@{person}.service"))
                .join("app.slice"),
            written: std::env::var_os("XDG_RUNTIME_DIR")
                .map(|runtime| PathBuf::from(runtime).join("alo-copies")),
            running: BTreeMap::new(),
            starts: 0,
        }
    }

    /// The copy of this application if it is still running, found again from
    /// what was written down if this desktop did not start it.
    fn still_running(&mut self, application: &str) -> bool {
        if !self.running.contains_key(application) {
            if let Some(found) = self.written_down(application) {
                self.running.insert(application.to_owned(), found);
            }
        }
        let Some(copy) = self.running.get_mut(application) else {
            return false;
        };
        let alive = match copy.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => began_at(copy.pid) == Some(copy.began),
        };
        if !alive {
            self.running.remove(application);
            if let Some(file) = self.the_file_for(application) {
                let _ = std::fs::remove_file(file);
            }
        }
        alive
    }

    /// Start a copy, and answer whether it is running.
    fn start(&mut self, application: &str) -> Result<(), NoCopy> {
        let Some(home) = self.home.clone() else {
            return Err(said(application, "this desktop does not know the person's home"));
        };
        let Some(copy) = alo_launching::alos_copy(application, &home) else {
            return Err(NoCopy::OfItsOwn);
        };
        let command = self.the_command(application)?;
        let Some((first, rest)) = command.split_first() else {
            return Err(said(application, "its command names nothing"));
        };
        let Some(program) = alo_launching::the_program(first, self.search.as_deref(), is_runnable)
        else {
            return Err(said(application, &format!("{first} is not a program on this machine")));
        };
        let Some(handed) = self.handed.clone() else {
            return Err(said(application, "there is no display to hand it"));
        };
        if let Err(why) = std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(copy.profile())
        {
            return Err(said(
                application,
                &format!("its profile at {} could not be made: {why}", copy.profile().display()),
            ));
        }
        let child = Command::new(program)
            .args(rest)
            .args(copy.arguments())
            .env_clear()
            .envs(handed.iter().filter_map(|pair| pair.split_once('=')))
            .current_dir(&home)
            .stdin(Stdio::null())
            .spawn();
        let child = match child {
            Ok(child) => child,
            Err(why) => return Err(said(application, &format!("it would not start: {why}"))),
        };
        let pid = child.id();
        self.starts = self.starts.wrapping_add(1);
        let unit = self.handed_to_the_manager(application, pid);
        let began = began_at(pid).unwrap_or(0);
        eprintln!(
            "alo-desktop: alo's own copy of {application} was started as process {pid}{}",
            unit.as_deref()
                .map_or_else(String::new, |unit| format!(", in {unit}"))
        );
        self.write_down(application, pid, began);
        self.running.insert(
            application.to_owned(),
            Running {
                pid,
                began,
                child: Some(child),
                unit,
            },
        );
        Ok(())
    }

    /// The installed entry's own command, which the copy's arguments follow.
    fn the_command(&self, application: &str) -> Result<Vec<String>, NoCopy> {
        let Some(location) = alo_launching::where_the_entry_is(application, &self.entries) else {
            return Err(said(application, "no entry by that name is installed"));
        };
        let Some(entry) = std::fs::read_to_string(&location)
            .ok()
            .as_deref()
            .and_then(TheEntry::read)
        else {
            return Err(said(application, "its entry could not be read"));
        };
        match alo_launching::how_it_starts(application, &entry, location.to_str()) {
            Ok(
                HowItStarts::FromItsCommand(command)
                | HowItStarts::OverTheBus {
                    or_else: Some(command),
                },
            ) => Ok(command),
            // Started over the bus only, the bus has one name for it, and a
            // second start is the first one's window: no copy of its own.
            Ok(HowItStarts::OverTheBus { or_else: None }) => Err(NoCopy::OfItsOwn),
            Err(why) => Err(said(application, &format!("its entry cannot start it: {why:?}"))),
        }
    }

    /// Hand the process to the person's own manager as a scope of its own.
    fn handed_to_the_manager(&self, application: &str, pid: u32) -> Option<String> {
        let unit = alo_launching::the_unit_for_alos_copy(application, self.starts)?;
        let properties: Vec<(&str, Value<'_>)> = vec![
            (
                "Description",
                Value::from(format!("alo's own copy of {application}")),
            ),
            ("PIDs", Value::from(vec![pid])),
            ("Slice", Value::from("app.slice")),
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
            Ok(_) => Some(unit),
            Err(why) => {
                eprintln!(
                    "alo-desktop: the person's manager would not take alo's copy of \
                     {application} as {unit}, so only where the application moves itself can \
                     be counted: {why}"
                );
                None
            }
        }
    }

    /// The file a copy's number is written in.
    fn the_file_for(&self, application: &str) -> Option<PathBuf> {
        self.written.as_ref().map(|folder| folder.join(application))
    }

    /// Write a copy's number down, so a restarted desktop finds it.
    fn write_down(&self, application: &str, pid: u32, began: u64) {
        let (Some(folder), Some(file)) = (self.written.as_ref(), self.the_file_for(application))
        else {
            return;
        };
        let wrote = std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(folder)
            .and_then(|()| std::fs::write(&file, format!("{pid} {began}\n")));
        if let Err(why) = wrote {
            eprintln!(
                "alo-desktop: where alo's copy of {application} runs could not be written \
                 down, so a restarted desktop will not find it: {why}"
            );
        }
    }

    /// A copy written down by an earlier desktop, if its process is still the
    /// one that was written down.
    fn written_down(&self, application: &str) -> Option<Running> {
        let said = std::fs::read_to_string(self.the_file_for(application)?).ok()?;
        let (pid, began) = said.trim().split_once(' ')?;
        let (pid, began) = (pid.parse().ok()?, began.parse().ok()?);
        (began_at(pid) == Some(began)).then_some(Running {
            pid,
            began,
            child: None,
            unit: None,
        })
    }
}

impl StartsAlosCopy for AlosCopies {
    fn started(&mut self, application: &str) -> Result<(), NoCopy> {
        if self.still_running(application) {
            return Ok(());
        }
        self.start(application)
    }

    fn groups_of(&mut self, application: &str) -> Vec<u64> {
        if !self.still_running(application) {
            return Vec::new();
        }
        let Some(copy) = self.running.get(application) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        if let Some(unit) = &copy.unit {
            crate::where_an_application_runs::with_everything_beneath(
                &self.slice.join(unit),
                0,
                &mut found,
            );
        }
        if let Some(moved) = where_it_runs(copy.pid, &self.slice) {
            crate::where_an_application_runs::with_everything_beneath(&moved, 0, &mut found);
        }
        found.sort_unstable();
        found.dedup();
        found
    }

    fn every_copys_groups(&mut self) -> BTreeSet<u64> {
        let applications: Vec<String> = self.running.keys().cloned().collect();
        applications
            .iter()
            .flat_map(|application| self.groups_of(application))
            .collect()
    }
}

/// The control group a process runs in, if it is in the person's `app.slice`
/// and is not the slice itself — anywhere else is not an application's.
fn where_it_runs(pid: u32, slice: &Path) -> Option<PathBuf> {
    the_group_in(&std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?, slice)
}

/// The control group `/proc/<pid>/cgroup` names, held to the person's slice.
fn the_group_in(said: &str, slice: &Path) -> Option<PathBuf> {
    let path = said
        .lines()
        .find_map(|line| line.strip_prefix("0::"))?
        .trim_start_matches('/');
    let group = Path::new(CGROUPS).join(path);
    (group.starts_with(slice) && group != slice).then_some(group)
}

/// When a process began, in the kernel's ticks since boot — field 22 of
/// `/proc/<pid>/stat`, read after the name, which may hold spaces.
fn began_at(pid: u32) -> Option<u64> {
    began_in(&std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?)
}

/// When the process `/proc/<pid>/stat` describes began.
fn began_in(stat: &str) -> Option<u64> {
    let (_, after_the_name) = stat.rsplit_once(')')?;
    let mut fields = after_the_name.split_whitespace();
    // The state is field 3; a process that has ended and not been reaped is
    // not running.
    if fields.next()? == "Z" {
        return None;
    }
    fields.nth(18)?.parse().ok()
}

/// Say why alo's copy of `application` did not start, and answer so.
fn said(application: &str, why: &str) -> NoCopy {
    eprintln!("alo-desktop: alo's own copy of {application} was not started: {why}");
    NoCopy::CouldNotStart
}

/// Whether the file at `path` is one somebody may run.
fn is_runnable(path: &Path) -> bool {
    std::fs::metadata(path).is_ok_and(|it| it.is_file() && it.permissions().mode() & 0o111 != 0)
}

#[cfg(test)]
#[path = "alos_copies_tests.rs"]
mod tests;
