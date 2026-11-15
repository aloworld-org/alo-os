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
//! manager as a scope of its own, `app-alocopy-<id>-<n>.scope`. The process is
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
use std::os::fd::{AsRawFd as _, OwnedFd};
use std::os::unix::fs::{DirBuilderExt as _, MetadataExt as _, PermissionsExt as _};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use alo_acting::Opened;
use alo_launching::{HowItStarts, TheEntry, WhyNotWithAFile};
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
    /// **Every group the process was seen in while it lived** — above all the
    /// scope Flatpak moved it into. The process alo starts is `flatpak run`,
    /// which moves itself into a scope of Flatpak's and then ends, while the
    /// application runs on there; the walk of 2026-11-15 (run 43726518930)
    /// saw VSCodium's window in that scope answered *not alo's copy*, because
    /// the copy was forgotten when its first process ended. Kept from the
    /// kernel's own account of that process, and from the instance number
    /// Flatpak writes on the copy's own pipe (`Running::instance`) — a number
    /// only something holding that pipe's write end can write.
    seen: Vec<PathBuf>,
    /// **Where Flatpak says which instance the copy is**, until it has: the
    /// read end of the pipe `flatpak run` was told to write its instance on
    /// (`alo_launching::flatpak_instance`). The scope that names is kept in
    /// [`Running::seen`], however fast the process that moved there ended.
    instance: Option<OwnedFd>,
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
    /// What was last said about each process that is not alo's copy, by
    /// application and process, so each is logged once each time it changes.
    said_not_alos: BTreeMap<(String, u32), String>,
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
            said_not_alos: BTreeMap::new(),
        }
    }

    /// The copy of this application if it is still running, found again from
    /// what was written down if this desktop did not start it.
    fn still_running(&mut self, application: &str) -> bool {
        if !self.running.contains_key(application)
            && let Some(found) = self.written_down(application)
        {
            self.running.insert(application.to_owned(), found);
        }
        let Some(copy) = self.running.get_mut(application) else {
            return false;
        };
        let launcher = match copy.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => began_at(copy.pid) == Some(copy.began),
        };
        // **What Flatpak said it is**, once it has: the scope that instance
        // runs in is kept, however fast the process that moved there ended.
        if let Some(read) = copy.instance.as_ref() {
            match the_instance_heard(read) {
                Heard::Nothing => {}
                Heard::It(instance) => {
                    if let Some(scope) =
                        alo_launching::the_scope_of_an_instance(application, &instance)
                    {
                        let group = self.slice.join(scope);
                        if !copy.seen.contains(&group) && copy.seen.len() < THE_MOST_SEEN {
                            copy.seen.push(group);
                        }
                    }
                    copy.instance = None;
                }
                Heard::Never => copy.instance = None,
            }
        }
        // **Where it is now, kept**, while the process alo started still says.
        // Where this desktop did not start it (a copy found again after a
        // restart), the process is not held unreaped, so its number could be
        // given to another process between the two reads: it is asked when it
        // began again after its group is read, and a changed answer keeps
        // nothing.
        if launcher
            && let Some(now) = where_it_runs(copy.pid, &self.slice)
            && (copy.child.is_some() || began_at(copy.pid) == Some(copy.began))
            && !copy.seen.contains(&now)
            && copy.seen.len() < THE_MOST_SEEN
        {
            copy.seen.push(now);
        }
        // **Running while anything of it runs**: the process alo started, or
        // any process still in a group it was seen in.
        let alive = launcher || copy.seen.iter().any(|group| holds_a_process(group, 0));
        if !alive {
            self.running.remove(application);
            if let Some(file) = self.the_file_for(application) {
                let _ = std::fs::remove_file(file);
            }
        }
        alive
    }

    /// Start a copy, and answer whether it is running.
    ///
    /// `command` is the installed entry's own, with a file in it where one was
    /// asked to be opened (`Self::the_command_with_a_file`).
    fn start(&mut self, application: &str, command: &[String]) -> Result<(), NoCopy> {
        let Some(home) = self.home.clone() else {
            return Err(said(
                application,
                "this desktop does not know the person's home",
            ));
        };
        let Some(copy) = alo_launching::alos_copy(application, &home) else {
            return Err(NoCopy::OfItsOwn);
        };
        // **Told to say which Flatpak instance it is**, where it is `flatpak
        // run`, so the scope it runs in is named by Flatpak rather than caught
        // in flight (`alo_launching::flatpak_instance`). A command that is not
        // gets no pipe at all.
        let pipe = an_instance_pipe();
        let told = pipe
            .as_ref()
            .and_then(|(_, write)| alo_launching::told_its_instance(command, write.as_raw_fd()));
        let pipe = if told.is_some() { pipe } else { None };
        let command: &[String] = told.as_deref().unwrap_or(command);
        let Some((first, rest)) = command.split_first() else {
            return Err(said(application, "its command names nothing"));
        };
        let Some(program) = alo_launching::the_program(first, self.search.as_deref(), is_runnable)
        else {
            return Err(said(
                application,
                &format!("{first} is not a program on this machine"),
            ));
        };
        let Some(mut handed) = self.handed.clone() else {
            return Err(said(application, "there is no display to hand it"));
        };
        // **alo's copy starts with the person's session, and whether it
        // describes itself is handed on top**, so the person's answer is what
        // it gets whatever the session last held. Off, it does not describe
        // itself, and alo cannot work inside it — which is what the switch
        // says it costs.
        let answer = crate::telling_applications_to_describe_themselves::the_persons_answer();
        handed.extend(alo_launching::what_a_start_is_handed(answer.describe));
        let environment = in_the_persons_session(application, &handed);
        let gains = alo_launching::what_its_command_gains(
            application,
            answer.describe,
            answer.alo_works_fully,
        );
        if let Err(why) = std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(copy.profile())
        {
            return Err(said(
                application,
                &format!(
                    "its profile at {} could not be made: {why}",
                    copy.profile().display()
                ),
            ));
        }
        // A code editor's copy is set so that what alo types is what lands
        // (ADR 0153), in this profile only. Not a reason to refuse the copy:
        // what alo types is read back before it is saved.
        if let Some(file) = alo_launching::its_settings_file(application, &copy)
            && let Err(why) = crate::typing_left_as_it_is::leave_typing_alone_in(&file)
        {
            eprintln!(
                "alo-desktop: alo's own copy of {application} was not set to type exactly, \
                 so what alo types there may be changed by the editor and will then not be \
                 saved: {why}"
            );
        }
        let child = Command::new(program)
            .args(rest)
            .args(copy.arguments())
            .args(gains)
            .env_clear()
            .envs(environment)
            .current_dir(&home)
            .stdin(Stdio::null())
            .spawn();
        let child = match child {
            Ok(child) => child,
            Err(why) => return Err(said(application, &format!("it would not start: {why}"))),
        };
        // The child holds its own copy of the write end now; this desktop's is
        // let go, so the read end ends when Flatpak's does.
        let instance = pipe.map(|(read, write)| {
            drop(write);
            read
        });
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
                seen: Vec::new(),
                instance,
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
            Err(why) => Err(said(
                application,
                &format!("its entry cannot start it: {why:?}"),
            )),
        }
    }

    /// The installed entry's own command **with `file` where the entry says a
    /// file goes** (`alo_launching::how_it_starts_with_a_file`), or what the
    /// daemon is told instead.
    fn the_command_with_a_file(
        &self,
        application: &str,
        file: &str,
    ) -> Result<Vec<String>, Opened> {
        let Some(location) = alo_launching::where_the_entry_is(application, &self.entries) else {
            return Err(Opened::NotInstalled);
        };
        let Some(entry) = std::fs::read_to_string(&location)
            .ok()
            .as_deref()
            .and_then(TheEntry::read)
        else {
            eprintln!("alo-desktop: the entry of {application} could not be read");
            return Err(Opened::CouldNotStart);
        };
        match alo_launching::how_it_starts_with_a_file(application, &entry, location.to_str(), file)
        {
            Ok(
                HowItStarts::FromItsCommand(command)
                | HowItStarts::OverTheBus {
                    or_else: Some(command),
                },
            ) => Ok(command),
            // Over the bus only: one name, one window, no copy of its own.
            Ok(HowItStarts::OverTheBus { or_else: None }) => Err(Opened::NoCopyOfItsOwn),
            Err(WhyNotWithAFile::TakesNoFile) => Err(Opened::TakesNoFile),
            Err(WhyNotWithAFile::CannotStart(why)) => {
                eprintln!("alo-desktop: the entry of {application} cannot start it: {why:?}");
                Err(Opened::CouldNotStart)
            }
        }
    }

    /// **Hand `command` — the entry's own, with the file in it — to the copy
    /// already running**, by starting it once more over the copy's own
    /// profile (`alo_launching::AlosCopy::handing_over`). The process that
    /// hands over ends by itself, and is waited for on a thread of its own so
    /// it is reaped without this desktop waiting.
    fn handed_over(&self, application: &str, command: &[String]) -> Opened {
        let Some(home) = self.home.clone() else {
            return Opened::CouldNotStart;
        };
        let Some(copy) = alo_launching::alos_copy(application, &home) else {
            return Opened::NoCopyOfItsOwn;
        };
        let Some((first, rest)) = command.split_first() else {
            return Opened::CouldNotStart;
        };
        let Some(program) = alo_launching::the_program(first, self.search.as_deref(), is_runnable)
        else {
            return Opened::CouldNotStart;
        };
        let Some(mut handed) = self.handed.clone() else {
            return Opened::CouldNotStart;
        };
        let answer = crate::telling_applications_to_describe_themselves::the_persons_answer();
        handed.extend(alo_launching::what_a_start_is_handed(answer.describe));
        let started = Command::new(program)
            .args(rest)
            .args(copy.handing_over())
            .env_clear()
            .envs(in_the_persons_session(application, &handed))
            .current_dir(&home)
            .stdin(Stdio::null())
            .spawn();
        match started {
            Ok(mut child) => {
                std::thread::spawn(move || {
                    let _ = child.wait();
                });
                eprintln!("alo-desktop: a file was handed to alo's own copy of {application}");
                Opened::InAlosCopy
            }
            Err(why) => {
                eprintln!(
                    "alo-desktop: a file could not be handed to alo's own copy of \
                     {application}: {why}"
                );
                Opened::CouldNotStart
            }
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
            seen: Vec::new(),
            instance: None,
        })
    }
}

impl StartsAlosCopy for AlosCopies {
    fn started(&mut self, application: &str) -> Result<(), NoCopy> {
        if self.still_running(application) {
            return Ok(());
        }
        let command = self.the_command(application)?;
        self.start(application, &command)
    }

    fn started_with_a_file(&mut self, application: &str, file: &Path) -> Opened {
        // A file whose name is not text cannot be put in a command line, and
        // the door never hands one over: said rather than assumed.
        let Some(file) = file.to_str() else {
            return Opened::CouldNotStart;
        };
        let command = match self.the_command_with_a_file(application, file) {
            Ok(command) => command,
            Err(why) => return why,
        };
        if self.still_running(application) {
            return self.handed_over(application, &command);
        }
        match self.start(application, &command) {
            Ok(()) => Opened::InAlosCopy,
            Err(NoCopy::OfItsOwn) => Opened::NoCopyOfItsOwn,
            Err(NoCopy::CouldNotStart) => Opened::CouldNotStart,
        }
    }

    fn is_running(&mut self, application: &str) -> bool {
        self.still_running(application)
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
        // Each group once: where the process is now, and where it was seen,
        // which usually holds the same one — walked twice it would count twice
        // towards how many groups can be held.
        let mut walked: Vec<PathBuf> = Vec::new();
        let now = where_it_runs(copy.pid, &self.slice);
        for group in now.iter().chain(copy.seen.iter()) {
            if !walked.contains(group) {
                crate::where_an_application_runs::with_everything_beneath(group, 0, &mut found);
                walked.push(group.clone());
            }
        }
        found.sort_unstable();
        found.dedup();
        found
    }

    fn runs_as_alos_copy(&mut self, application: &str, pid: u32) -> bool {
        // The control group the kernel says this process is in, held to the
        // person's own slice, by the kernel's number for it — and that number
        // among the groups alo's copy runs in, which never holds one of the
        // person's own (`groups_of`).
        let Some(group) = where_it_runs(pid, &self.slice) else {
            self.say_not_alos(
                application,
                pid,
                "outside the person's applications, or not at all any more",
                "",
            );
            return false;
        };
        let Ok(about) = std::fs::symlink_metadata(&group) else {
            self.say_not_alos(application, pid, "in a group that went", "");
            return false;
        };
        // Known before `groups_of`, which forgets a copy whose process ended.
        let was_known = self.running.contains_key(application)
            || self
                .the_file_for(application)
                .is_some_and(|file| file.exists());
        let groups = self.groups_of(application);
        let is = groups.contains(&about.ino());
        if !is {
            // **Which group, and which the copy's are**: the walk of 2026-11-15
            // (run 43703842775) saw VSCodium's window answered *not alo's to
            // type into* with a second Flatpak scope started after the copy's,
            // and nothing said which group the window's process was in.
            let theirs = group.strip_prefix(&self.slice).map_or_else(
                |_| group.display().to_string(),
                |it| it.display().to_string(),
            );
            let copys = self.the_copys_groups_said(application, &groups, was_known);
            self.say_not_alos(application, pid, &format!("in {theirs}"), &copys);
        }
        is
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
    the_group_in(
        &std::fs::read_to_string(format!("/proc/{pid}/cgroup")).ok()?,
        slice,
    )
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

/// **What a process of alo's copy of `application` starts with**: the
/// person's session, as their systemd manager holds it, with `handed` laid
/// over it (`alo_launching::alos_copy_starts_with`) — or, where the manager
/// cannot be asked, `handed` alone, which is said on the log because a copy
/// with no session bus or runtime folder may start and show nothing.
fn in_the_persons_session(application: &str, handed: &[String]) -> Vec<(String, String)> {
    let read = alo_shell::the_persons_session::what_their_manager_holds();
    let session = read.unwrap_or_else(|why| {
        eprintln!(
            "alo-desktop: the person's session could not be read from their systemd manager \
             ({why}), so alo's own copy of {application} starts with only where alo draws"
        );
        Vec::new()
    });
    alo_launching::alos_copy_starts_with(&session, handed)
}

impl AlosCopies {
    /// **Why a process is not alo's copy**, logged once for each application
    /// and process each time it changes: which group it runs in, and which the
    /// copy's are. Group names and process numbers only.
    fn say_not_alos(&mut self, application: &str, pid: u32, theirs: &str, copys: &str) {
        let said = if copys.is_empty() {
            format!(
                "alo-desktop: process {pid} of {application} is not alo's own copy: it runs \
                 {theirs}"
            )
        } else {
            format!(
                "alo-desktop: process {pid} of {application} is not alo's own copy: it runs \
                 {theirs}; {copys}"
            )
        };
        let key = (application.to_owned(), pid);
        if self.said_not_alos.get(&key) != Some(&said) {
            eprintln!("{said}");
            // Bounded: a desktop up for days meets many processes.
            if self.said_not_alos.len() >= 64 {
                self.said_not_alos.clear();
            }
            self.said_not_alos.insert(key, said);
        }
    }

    /// The copy's process, whether it still runs, and the groups it is
    /// counted in, by name, for the log. `was_known` is whether a copy was
    /// known before this look, which tells a copy whose process ended from one
    /// never started.
    fn the_copys_groups_said(&self, application: &str, groups: &[u64], was_known: bool) -> String {
        let Some(copy) = self.running.get(application) else {
            return if was_known {
                "alo's copy of it was started and its process has ended".to_owned()
            } else {
                "alo's copy of it was never started".to_owned()
            };
        };
        let mut named: Vec<String> = Vec::new();
        if let Some(unit) = &copy.unit {
            named.push(unit.clone());
        }
        for seen in &copy.seen {
            named.push(seen.strip_prefix(&self.slice).map_or_else(
                |_| seen.display().to_string(),
                |it| it.display().to_string(),
            ));
        }
        if let Some(moved) = where_it_runs(copy.pid, &self.slice)
            && !copy.seen.contains(&moved)
        {
            named.push(moved.strip_prefix(&self.slice).map_or_else(
                |_| moved.display().to_string(),
                |it| it.display().to_string(),
            ));
        }
        format!(
            "alo's copy is process {}, counted in its group(s) {} ({} with those beneath them)",
            copy.pid,
            if named.is_empty() {
                "none found".to_owned()
            } else {
                named.join(", ")
            },
            groups.len()
        )
    }
}

/// **A pipe Flatpak can say its instance on**: the read end this desktop's
/// own, closed in every program it starts; the write end inherited by the next
/// one, under the number `--instance-id-fd` names. Neither end blocks.
///
/// **The write end is inheritable from here until the copy has started**, and a
/// program another thread of this desktop starts in that moment (the screen
/// reader, `gh`) inherits it too: the read end then never sees its end, which
/// only keeps it until the number arrives, and that program could write on it.
/// Mapping one descriptor into one child alone needs `pre_exec`, which is
/// `unsafe` and forbidden here, so the moment is kept short instead.
/// [`None`] where the kernel will not make one, and the copy is then found as
/// before, by looking at its process.
fn an_instance_pipe() -> Option<(OwnedFd, OwnedFd)> {
    use rustix::pipe::PipeFlags;
    let (read, write) = rustix::pipe::pipe_with(PipeFlags::CLOEXEC | PipeFlags::NONBLOCK).ok()?;
    rustix::io::fcntl_setfd(&write, rustix::io::FdFlags::empty()).ok()?;
    Some((read, write))
}

/// What one look at the instance pipe heard.
#[derive(Debug, PartialEq, Eq)]
enum Heard {
    /// Nothing yet.
    Nothing,
    /// The instance number.
    It(String),
    /// It closed, or said something that is not an instance: nothing will come.
    Never,
}

/// Look once at what Flatpak wrote on the instance pipe, never waiting.
fn the_instance_heard(read: &OwnedFd) -> Heard {
    let mut said = [0_u8; 32];
    match rustix::io::read(read, said.as_mut_slice()) {
        Ok(0) => Heard::Never,
        Ok(length) => said
            .get(..length)
            .and_then(alo_launching::an_instance_number)
            .map_or(Heard::Never, Heard::It),
        Err(why) if why == rustix::io::Errno::AGAIN || why == rustix::io::Errno::INTR => {
            Heard::Nothing
        }
        Err(_) => Heard::Never,
    }
}

/// The most groups one copy is remembered in: the unit, Flatpak's scope, and
/// room for a move nobody has measured.
const THE_MOST_SEEN: usize = 8;

/// How deep beneath a group a process is looked for.
const DEEPEST_LOOKED_IN: usize = 4;

/// Whether any process runs in `group` or in a group beneath it — read off
/// each group's own `cgroup.procs`, the kernel's list. A group that has gone
/// holds none.
fn holds_a_process(group: &Path, depth: usize) -> bool {
    if std::fs::read_to_string(group.join("cgroup.procs"))
        .is_ok_and(|procs| procs.lines().any(|line| !line.trim().is_empty()))
    {
        return true;
    }
    if depth >= DEEPEST_LOOKED_IN {
        return false;
    }
    std::fs::read_dir(group).is_ok_and(|beneath| {
        beneath.flatten().any(|one| {
            one.file_type().is_ok_and(|kind| kind.is_dir())
                && holds_a_process(&one.path(), depth + 1)
        })
    })
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
