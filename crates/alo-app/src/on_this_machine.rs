//! The one [`Machine`] that reaches a real machine.
//!
//! # Where this person's doors are
//!
//! `/run/alo/<uid>` holds both: the desktop's `shell.sock` and `alo-agentd`'s
//! `agentd.sock`. **`ALO_APP_DOORS_IN` names another folder holding them**,
//! and is how a test runs this very binary against doors of its own. It moves
//! nothing a person does not already have: both doors admit only the person's
//! own login, decided by the kernel, and whoever can set this person's
//! environment can already run anything as them.

use std::ffi::OsString;
use std::io::{BufRead as _, IsTerminal as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::SystemTime;

use alo_adapters::{
    AccessibilityTree, AccessibleSession, Delivers, NotDelivered, SessionBus, TreeFault,
    adapter_verbs, shipped_adapters,
};
use alo_app::at_the_daemon::a_place_kept_in;
use alo_app::at_the_door::AtTheDoor;
use alo_app::door::TheDoor;
use alo_app::{APlace, Ended, Knowing, Machine, run};
use alo_choosing::{Settings, the_persons_folder};
use alo_keeping::Writing;
use alo_record::Entry;
use alo_strings::{Said, Strings, Vocabulary};

/// The folder this person's doors are in, where a test puts its own.
const THE_DOORS_IN: &str = "ALO_APP_DOORS_IN";

/// Everything `alo-app` can say: its own words, and every sentence it prints
/// that another crate already says.
fn what_it_can_say() -> Result<Vocabulary, String> {
    let mut vocabulary = Vocabulary::empty();
    alo_app::declare_into(&mut vocabulary).map_err(|why| why.to_string())?;
    alo_adapters::words::declare_into(&mut vocabulary).map_err(|why| why.to_string())?;
    alo_applications::words::declare_into(&mut vocabulary).map_err(|why| why.to_string())?;
    alo_capability::declare_into(&mut vocabulary).map_err(|why| why.to_string())?;
    alo_keeping::declare_into(&mut vocabulary).map_err(|why| why.to_string())?;
    alo_choosing::declare_into(&mut vocabulary).map_err(|why| why.to_string())?;
    Ok(vocabulary)
}

/// The machine, as one command reaches it.
struct OnThisMachine {
    /// The desktop's door.
    door: AtTheDoor,
    /// The folder the doors are in.
    doors: PathBuf,
    /// The accessibility tree, once opened.
    tree: Option<AccessibleSession>,
    /// The session bus, once opened.
    bus: Option<SessionBus>,
    /// The person's record file.
    keeping: PathBuf,
    /// The record, once opened.
    record: Option<Writing>,
    /// The pipe.
    pipe: std::io::Lines<std::io::StdinLock<'static>>,
}

impl Machine for OnThisMachine {
    fn now(&self) -> SystemTime {
        SystemTime::now()
    }

    fn door(&mut self) -> &mut dyn TheDoor {
        &mut self.door
    }

    fn tree(&mut self) -> Result<&dyn AccessibilityTree, TreeFault> {
        if self.tree.is_none() {
            self.tree = Some(AccessibleSession::of_this_person()?);
        }
        self.tree
            .as_ref()
            .map(|tree| tree as &dyn AccessibilityTree)
            .ok_or(TreeFault::NotThere)
    }

    fn bus(&mut self) -> Result<&dyn Delivers, NotDelivered> {
        if self.bus.is_none() {
            self.bus = Some(SessionBus::this_persons()?);
        }
        self.bus
            .as_ref()
            .map(|bus| bus as &dyn Delivers)
            .ok_or(NotDelivered::NotThere)
    }

    fn access(&mut self) -> alo_access::TurnedOn {
        // **The same file, read the same way, as the desktop reads it at each
        // start** (`alo-desktop`'s `telling_applications_to_describe_themselves`),
        // so what a refusal says and how the application was started agree.
        alo_choosing::where_the_folder_is(
            std::env::var_os(alo_choosing::CONFIG_HOME).as_deref(),
            std::env::var_os(alo_choosing::HOME).as_deref(),
        )
        .map_or_else(alo_access::TurnedOn::nothing, |folder| {
            alo_access::keeping::at_sign_in(&folder).0
        })
    }

    fn a_place_to_return_to(&mut self, strings: &Strings) -> APlace {
        a_place_kept_in(&self.doors, strings)
    }

    fn the_record_is_open(&mut self, strings: &Strings) -> Result<(), Said> {
        if self.record.is_some() {
            return Ok(());
        }
        if let Some(folder) = self.keeping.parent() {
            // A folder that could not be made is said by the opening below.
            let _ = std::fs::create_dir_all(folder);
        }
        let record = Writing::opening(&self.keeping).map_err(|why| why.said(strings))?;
        self.record = Some(record);
        Ok(())
    }

    fn keep(&mut self, entry: &Entry, strings: &Strings) -> Result<(), Said> {
        self.the_record_is_open(strings)?;
        match self.record.as_mut() {
            Some(record) => record.keep(entry).map_err(|why| why.said(strings)),
            None => Ok(()),
        }
    }

    fn next_on_the_pipe(&mut self) -> Option<String> {
        match self.pipe.next()? {
            Ok(line) => Some(line.strip_suffix('\r').map(str::to_owned).unwrap_or(line)),
            Err(_) => None,
        }
    }

    fn told(&mut self, line: &str) {
        let _ = writeln!(std::io::stdout().lock(), "{line}");
    }

    fn warned(&mut self, line: &str) {
        let _ = writeln!(std::io::stderr().lock(), "{line}");
    }
}

/// The folder this person's doors are in.
fn where_the_doors_are() -> PathBuf {
    std::env::var_os(THE_DOORS_IN).map_or_else(
        || {
            Path::new(alo_protocol::where_the_door_is::THE_ROOT)
                .join(rustix::process::getuid().as_raw().to_string())
        },
        PathBuf::from,
    )
}

/// What is installed, read where the desktop reads it.
fn what_is_installed(home: &Path) -> alo_applications::Installed {
    let data_home = std::env::var_os("XDG_DATA_HOME");
    let data_dirs = std::env::var_os("XDG_DATA_DIRS");
    let dirs = alo_installed::with_what_flatpak_exports(
        Path::new(alo_installed::THE_MACHINES_INSTALLATION),
        Some(home),
        data_home.as_deref(),
        data_dirs.as_deref(),
    );
    alo_installed::read(&alo_installed::where_entries_live(
        Some(home),
        data_home.as_deref(),
        Some(&dirs),
    ))
}

/// What each installed application's package lets it reach, read from both
/// flatpak installations — the person's and the machine's.
fn what_they_may_reach(
    home: &Path,
    installed: &alo_applications::Installed,
) -> alo_installed::WhatTheyMayReach {
    let data_home = std::env::var_os("XDG_DATA_HOME");
    let persons = alo_installed::the_persons_installation(Some(home), data_home.as_deref());
    alo_installed::what_they_may_reach(
        Path::new(alo_installed::THE_MACHINES_INSTALLATION),
        persons.as_deref(),
        installed
            .all()
            .map(alo_applications::Application::identifier),
    )
}

/// The process.
pub fn main() -> ExitCode {
    let vocabulary = match what_it_can_say() {
        Ok(vocabulary) => vocabulary,
        // English, and the only English a person could see here: the sentence
        // that would say this in their language is the one that would not build.
        Err(why) => {
            eprintln!("alo-app: {why}");
            return ExitCode::FAILURE;
        }
    };
    let mut strings = Strings::of(vocabulary);
    let settings = the_persons_folder(
        std::env::var_os(alo_choosing::CONFIG_HOME).as_deref(),
        std::env::var_os(alo_choosing::HOME).as_deref(),
    )
    .ok()
    .and_then(|folder| Settings::at(&folder.path_of(alo_choosing::THE_SETTINGS)).ok())
    .unwrap_or_else(Settings::untouched);
    strings.prefers(settings.languages());

    // **What the person typed, before the machine is touched.** A word that is
    // not text cannot be shown or matched, so it is refused as a value.
    let typed: Result<Vec<String>, OsString> = std::env::args_os()
        .skip(1)
        .map(OsString::into_string)
        .collect();
    let Ok(typed) = typed else {
        eprintln!(
            "{}",
            strings.say(
                &alo_app::words::UNREADABLE_VALUE.key(),
                &alo_strings::Filling::nothing()
            )
        );
        return Ended::ValueRefused.into();
    };

    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        eprintln!(
            "alo-app: this session has no home directory, so there is nowhere to write down what is done"
        );
        return Ended::NotOffered.into();
    };
    let Ok(standing_in) = std::env::current_dir() else {
        eprintln!(
            "alo-app: this process has no current directory, so a relative path cannot be read"
        );
        return Ended::NotOffered.into();
    };
    let (Ok(adapters), Ok(verbs)) = (shipped_adapters(), adapter_verbs()) else {
        // The shipped declarations are loaded by a test; this cannot happen
        // with them as written.
        eprintln!("alo-app: the adapters this release ships would not load");
        return Ended::NotOffered.into();
    };
    // **Nothing is read off the machine for a line that is not the shape**:
    // `run` reads the shape first and refuses it before it looks at anything,
    // so an empty list stands in for one that would never be asked.
    let installed = if alo_app::typed::read(&typed).is_ok() {
        what_is_installed(&home)
    } else {
        alo_applications::Installed::nothing()
    };
    let doors = where_the_doors_are();
    let mut machine = OnThisMachine {
        door: AtTheDoor::in_(&doors),
        doors,
        tree: None,
        bus: None,
        keeping: alo_keeping::the_record_a_person_keeps(&home),
        record: None,
        pipe: std::io::stdin().lock().lines(),
    };
    let reaching = what_they_may_reach(&home, &installed);
    let knowing = Knowing {
        strings: &strings,
        installed: &installed,
        reaching: &reaching,
        adapters: &adapters,
        verbs: &verbs,
        standing_in: &standing_in,
    };
    run(
        &typed,
        std::io::stdout().is_terminal(),
        knowing,
        &mut machine,
    )
    .into()
}
