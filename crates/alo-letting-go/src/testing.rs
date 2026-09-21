//! The folders, clocks and stand-ins this crate's own unit tests are written
//! against.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//! What a stand-in stands in for is always something a developer's machine
//! cannot be: a filesystem with subvolumes, and a program that needs
//! `CAP_SYS_ADMIN` to run. Everything else — the folder, the two files, the
//! walk — is a real directory on a real disk, because that is how the unit
//! meets them.

#![expect(
    clippy::unwrap_used,
    reason = "in a test fixture, a panic on a None or an Err is the failure being reported"
)]

use std::cell::RefCell;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, SystemTime};

use alo_record::Entry;
use alo_strings::Strings;

use crate::removing::{NotRemoved, Removing};
use crate::the_disk::{AskingTheDisk, THE_FLOOR, WhatItIs};
use crate::the_folder::{AFTER, BEFORE, THE_TURN, THEIRS, TheTurn, Theirs};
use crate::words::letting_go_words;
use crate::writing_it_down::WritingItDown;

/// How long a day is, for a test that moves a clock past a window.
pub(crate) const A_DAY: Duration = Duration::from_secs(24 * 60 * 60);

/// This crate's own words, with nothing translated.
pub(crate) fn in_english() -> Strings {
    Strings::of(letting_go_words().unwrap())
}

/// A moment far enough from the epoch to read as a real one.
pub(crate) fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// A directory of this test's own, on a real disk.
pub(crate) fn a_folder(what: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!(
        "alo-letting-go-{what}-{}",
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// One person's directory under `under`, with their settings folder written
/// down exactly as their own session would write it.
pub(crate) fn one_person(under: &Path, named: &str, settings: &str) -> PathBuf {
    let at = under.join(named);
    std::fs::create_dir_all(&at).unwrap();
    let says = at.join(THEIRS);
    let theirs = Theirs::at(Path::new(settings), &says).unwrap();
    std::fs::write(&says, serde_json::to_string(&theirs).unwrap()).unwrap();
    at
}

/// One kept turn under a person's directory: the two snapshots, as ordinary
/// directories a developer's machine can make, and what describes them.
pub(crate) fn one_kept_turn(whose: &Path, named: &str, taken: SystemTime, did: &str) -> PathBuf {
    let at = whose.join(named);
    std::fs::create_dir_all(at.join(BEFORE)).unwrap();
    std::fs::create_dir_all(at.join(AFTER)).unwrap();
    let says = at.join(THE_TURN);
    let turn = TheTurn::done(taken, did, &says).unwrap();
    std::fs::write(&says, serde_json::to_string(&turn).unwrap()).unwrap();
    at
}

/// A disk that answers what the test arranged, and whose free space a remover
/// can give back to — the one thing about `btrfs` a developer's machine cannot
/// show, which is that removing a snapshot frees room the next question reads.
#[derive(Debug)]
pub(crate) struct ADisk {
    /// What it says it is.
    what: WhatItIs,
    /// How much is free now, shared with whoever gives room back to it.
    free: Rc<RefCell<u64>>,
}

impl ADisk {
    /// A disk that keeps, with room to spare.
    pub(crate) fn with_room() -> Self {
        Self {
            what: WhatItIs::OneThatKeeps,
            free: Rc::new(RefCell::new(THE_FLOOR * 2)),
        }
    }

    /// A disk that keeps, with this much free — below the floor, if the test
    /// says so.
    pub(crate) fn short_of_room(free: u64) -> Self {
        Self {
            what: WhatItIs::OneThatKeeps,
            free: Rc::new(RefCell::new(free)),
        }
    }

    /// A disk with no subvolumes on it: a machine installed before the
    /// filesystem was decided.
    pub(crate) fn that_keeps_nothing() -> Self {
        Self {
            what: WhatItIs::NotOneThatKeeps,
            free: Rc::new(RefCell::new(THE_FLOOR * 2)),
        }
    }

    /// What a remover gives room back to.
    pub(crate) fn purse(&self) -> Rc<RefCell<u64>> {
        Rc::clone(&self.free)
    }
}

impl AskingTheDisk for ADisk {
    fn what_it_is(&self, _at: &Path) -> io::Result<WhatItIs> {
        Ok(self.what)
    }

    fn free(&self, _at: &Path) -> io::Result<u64> {
        Ok(*self.free.borrow())
    }
}

/// A remover that really removes the directories a test made, and writes down
/// what it was asked — standing in for the one thing a developer's machine
/// cannot do, which is `btrfs subvolume delete`.
#[derive(Debug, Default)]
pub(crate) struct ARemover {
    /// Every kept turn it was asked to remove, in order.
    asked: RefCell<Vec<PathBuf>>,
    /// What it refuses, by directory name.
    refuses: Option<String>,
    /// A disk's free space, and how much each removal gives back to it.
    gives_back: Option<(Rc<RefCell<u64>>, u64)>,
}

impl ARemover {
    /// One that removes whatever it is given.
    pub(crate) fn willing() -> Self {
        Self::default()
    }

    /// One that refuses the kept turn whose directory is named `named`, the
    /// way a machine holding no capability refuses every one of them.
    pub(crate) fn refusing(named: &str) -> Self {
        Self {
            asked: RefCell::new(Vec::new()),
            refuses: Some(named.to_owned()),
            gives_back: None,
        }
    }

    /// One that removes, and gives `bytes` back to a disk's `purse` each time
    /// it does.
    pub(crate) fn giving_back(purse: Rc<RefCell<u64>>, bytes: u64) -> Self {
        Self {
            asked: RefCell::new(Vec::new()),
            refuses: None,
            gives_back: Some((purse, bytes)),
        }
    }

    /// Everything it was asked to remove, in the order it was asked.
    pub(crate) fn asked(&self) -> Vec<PathBuf> {
        self.asked.borrow().clone()
    }
}

impl Removing for ARemover {
    fn remove(&self, at: &Path) -> Result<(), NotRemoved> {
        self.asked.borrow_mut().push(at.to_owned());
        if self.refuses.as_deref() == at.file_name().and_then(|named| named.to_str()) {
            return Err(NotRemoved::Refused {
                at: at.to_owned(),
                said: "Operation not permitted".to_owned(),
            });
        }
        std::fs::remove_dir_all(at).map_err(|why| NotRemoved::NotCleared {
            at: at.to_owned(),
            why,
        })?;
        if let Some((purse, bytes)) = &self.gives_back {
            *purse.borrow_mut() += bytes;
        }
        Ok(())
    }
}

/// A record that keeps what it was given in memory.
#[derive(Debug, Default)]
pub(crate) struct ARecord {
    /// Every entry, in order.
    kept: Vec<Entry>,
    /// Whether it refuses everything.
    refuses: bool,
}

impl ARecord {
    /// One that keeps whatever it is given.
    pub(crate) fn willing() -> Self {
        Self::default()
    }

    /// One that will not keep anything — a full disk, at the worst moment.
    pub(crate) fn refusing() -> Self {
        Self {
            kept: Vec::new(),
            refuses: true,
        }
    }

    /// Everything it kept.
    pub(crate) fn kept(&self) -> &[Entry] {
        &self.kept
    }
}

impl WritingItDown for ARecord {
    fn keep(&mut self, entry: Entry) -> Result<(), String> {
        if self.refuses {
            return Err("the record would not take it".to_owned());
        }
        self.kept.push(entry);
        Ok(())
    }
}
