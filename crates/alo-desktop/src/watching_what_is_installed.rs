//! **Noticing that the machine's applications changed**, so a reading is taken
//! again without anybody signing out.
//!
//! A fresh machine installs its applications after its first boot — minutes on
//! real hardware, hours in a virtual machine — and a person who signs in
//! meanwhile must see each one as it arrives. The same holds for anything they
//! install from the market window, and anything they remove.
//!
//! # Looked at, not subscribed to, and why
//!
//! Every two seconds this lists the `applications/` directories and compares
//! what it sees — each entry's path, size and modification time, and whether
//! `alo-shipping` has finished — with what it saw last. When anything differs
//! it takes a whole new reading and hands it over.
//!
//! **Not inotify**, for two reasons that are facts about this machine rather
//! than taste. The directory that matters most does not exist until the first
//! flatpak is installed — `/var/lib/flatpak/exports/share/applications` — and
//! a watch cannot be put on a directory that is not there; watching its
//! parents for its arrival is the fragile half of every inotify watcher. And
//! a flatpak export is a symbolic link whose *target* changes on an update,
//! which a watch on the directory does not see. A listing of a few dozen
//! entries every two seconds costs nothing a person could notice, and it
//! cannot miss either case.
//!
//! **The reading itself is the expensive part** — it runs the rented tool, a
//! subprocess — so it is taken only when the listing changed, never on a
//! timer.
//!
//! # What is not looked at
//!
//! The icon directories. They hold thousands of files on an ordinary machine,
//! and flatpak exports an application's icons in the same transaction as its
//! entry, so the entry's arrival is the moment to read both. An icon that
//! arrived later than its entry would be drawn as a letter until the next
//! change; that is **not measured**, and is said here so a walk can look.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, SystemTime};

use crate::what_this_machine_offers::{Offered, WhereToLook};

/// How often the directories are looked at.
///
/// Two seconds: short enough that an application appears while the person is
/// still looking for it, long enough that the listing is noise on any machine.
pub const HOW_OFTEN: Duration = Duration::from_secs(2);

/// What the directories looked like, in a form that can be compared.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatWasThere {
    /// Every entry file, with its size and when it last changed — of the file a
    /// link points at, so an update that moves a flatpak link is a change.
    entries: Vec<(PathBuf, Option<SystemTime>, Option<u64>)>,
    /// Whether the fresh machine's applications were walked.
    walked: bool,
}

/// Look at these places now.
#[must_use]
pub fn what_is_there(at: &WhereToLook) -> WhatWasThere {
    let mut entries = Vec::new();
    for place in &at.entries {
        listed(place, &mut entries);
    }
    entries.sort();
    WhatWasThere {
        entries,
        walked: at.arrived.already(),
    }
}

/// Every `.desktop` file under one place, into `into`.
///
/// **A place that is not there adds nothing**, and that is the case that
/// matters: when it appears, what it holds is new, so the next look differs.
fn listed(place: &Path, into: &mut Vec<(PathBuf, Option<SystemTime>, Option<u64>)>) {
    let Ok(listing) = std::fs::read_dir(place) else {
        return;
    };
    for entry in listing.flatten() {
        let path = entry.path();
        if path.is_dir() {
            listed(&path, into);
        } else if path.extension().is_some_and(|it| it == "desktop") {
            // **Followed, not the link's own**, so a link whose target moved is
            // a change. A link to nothing is still listed, with no time.
            let followed = std::fs::metadata(&path).ok();
            into.push((
                path,
                followed.as_ref().and_then(|it| it.modified().ok()),
                followed.as_ref().map(std::fs::Metadata::len),
            ));
        }
    }
}

/// Look every `every`, and hand over a new reading whenever what is there
/// changed from `last`.
///
/// Returns when nobody is listening any more — the desktop that held the other
/// end has gone — which is the only way this ends.
pub fn kept_watching(
    at: &WhereToLook,
    mut last: WhatWasThere,
    every: Duration,
    read: impl Fn(&WhereToLook) -> Offered,
    tell: &Sender<Offered>,
) {
    loop {
        std::thread::sleep(every);
        let now = what_is_there(at);
        if now == last {
            continue;
        }
        last = now;
        if tell.send(read(at)).is_err() {
            return;
        }
    }
}

#[cfg(test)]
#[path = "watching_what_is_installed_tests.rs"]
mod tests;
