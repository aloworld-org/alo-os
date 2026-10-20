//! Which disks a person may be offered, what each is called, and why one is not.
//!
//! **This file exists because the environment has to ask.** On the road from
//! Windows it never did: the person chose a disk in the Windows program and the
//! environment was handed the answer as one word on the kernel's command line —
//! `05 · Prepare and restart` says so to them in as many words, *it will not
//! ask you to choose the disk again*. Started from removable media there is no
//! Windows to have asked, so the environment arrives knowing nothing and must
//! put the question itself (`docs/features.md`: *the environment that installs
//! asks what it needs to know, on the screen, at the moment it needs it*).
//!
//! # It decides and says; it reads nothing
//!
//! No file is opened here and no program is run. `crate::disks` holds what the
//! machine reported and `crate::sequence` does the asking; this is the
//! judgement between them, which is why every case below can be exercised
//! without a disk.
//!
//! # The two rules a person's disk depends on
//!
//! **The medium this installer is running from is never offered.** A stick that
//! offered itself is a person erasing the installer half way through installing.
//! It is keyed on `crate::disks::THIS_INSTALLER`, the label the writer puts on
//! the staged area, and **not on whether a disk looks removable** — somebody
//! installing onto an external drive is ordinary, and *removable* would refuse
//! them.
//!
//! **A medium with no label refuses every disk that is not plainly empty.** A
//! stick somebody wrote by hand carries no label, so there is no way to know
//! which disk the installer is running from — and the only set where being
//! wrong costs nothing is the set where nothing would be lost. It is said
//! rather than silently applied.
//!
//! # Erasing is offered, because the person decides
//!
//! A disk holding another system **is** offered, with `erases` true and a
//! sentence naming what goes. That is the design's own road — `03 · Keep
//! Windows or replace it` and `04a · Review · Erase disk` are drawn frames in
//! the design file — and law 5's question, *can the person choose this, knowing
//! what it costs*, is answered by the review that follows rather than by hiding
//! the disk. What this file guarantees is that the cost is **said**; what
//! `crate::sequence` guarantees is that nothing is written until the person has
//! typed that disk's own name.
//!
//! # What is still owed
//!
//! **A disk too small to hold alo OS is offered here and should not be.** The
//! rule lived only in the Windows half, as `THE_LEAST_DISK`, and this crate
//! could not see it — copying the number would have made two answers to one
//! question, which is the fault this repository keeps paying for, so it was
//! left undone and named here instead.
//!
//! **The Mac lane is giving it one home** — `alo_image::room::can_hold_alo_os`,
//! in the crate both roads already depend on, after measuring that from a stick
//! *nothing* refuses a disk too small and `bootc install to-disk --wipe` is one
//! program: the disk is erased, and then the install fails. The moment it is on
//! `main` this file calls it, and `S6 · No disk can be used` is the screen that
//! says so — *alo OS needs 24 GB. The only disk in this computer has 16 GB.*
//! Until then **the sentence exists and the rule does not**, which is worth
//! knowing in exactly that order.

use alo_strings::{Filling, Strings};

use crate::disks::{AWholeDisk, Disks, Replacing, Unsuitable};
use crate::words;

/// What this installer is running from, as a listing of the machine's disks
/// alone can say.
///
/// **Derivable from the listing and from nothing else**, deliberately: not the
/// kernel's command line, not a firmware variable. The label is the only
/// identification that is true on the road from Windows, on a stick the writer
/// made, and on a stick somebody wrote themselves — and the last of those is
/// the one that must not be assumed away.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TheMedium {
    /// The labelled area was found, and it is on this disk.
    ThisDisk(String),
    /// No labelled area anywhere: a medium written by hand.
    Unlabelled,
}

impl TheMedium {
    /// Which medium this is, from what the machine listed.
    #[must_use]
    pub fn of(disks: &Disks) -> Self {
        disks
            .the_staging_area()
            .map_or(Self::Unlabelled, |(disk, _)| Self::ThisDisk(disk))
    }

    /// Whether this disk is the one the installer is running from.
    fn is_the_disk(&self, name: &str) -> bool {
        matches!(self, Self::ThisDisk(disk) if disk == name)
    }
}

/// One disk a person may choose, and what they read about it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Offered {
    /// Its whole path, which is what the install step is given.
    pub disk: String,
    /// What a person reads it as: *Samsung SSD 870 EVO · 1 TB*.
    pub called: String,
    /// What is on it, in words.
    pub holding: String,
    /// Whether choosing it destroys something.
    ///
    /// **A flag and not a sentence**, because the fact is the same on every
    /// road while the sentence is not — and because a caller that has to read
    /// English to find out whether a choice is destructive is a caller that
    /// will get it wrong in a language nobody here reads.
    pub erases: bool,
}

/// One disk a person is not offered, and why.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotOffered {
    /// Its whole path, for the record rather than for the person.
    pub disk: String,
    /// What a person reads it as, so a refusal names something they can see.
    pub called: String,
    /// Why, as a sentence they can act on.
    pub why: String,
}

/// Everything the question *which disk?* has to show.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatToShow {
    /// The disks a person may choose, in the order the machine listed them.
    pub offered: Vec<Offered>,
    /// The disks they may not, each with its reason.
    ///
    /// **Shown rather than hidden.** A person whose only disk is missing from a
    /// list learns nothing from its absence; what they need is the sentence
    /// saying why, which is the difference between meeting a boundary and
    /// meeting what looks like a fault.
    pub refused: Vec<NotOffered>,
}

impl WhatToShow {
    /// Whether there is anything at all for a person to choose.
    #[must_use]
    pub fn has_anything(&self) -> bool {
        !self.offered.is_empty()
    }
}

/// What to show a person asking which disk alo OS should use.
#[must_use]
pub fn what_may_be_offered(disks: &Disks, from: &TheMedium, strings: &Strings) -> WhatToShow {
    let mut offered = Vec::new();
    let mut refused = Vec::new();
    // **`into_text` here and not `Said` on the struct.** `Said` carries where a
    // sentence came from and whether it was translated, which is a fact about
    // the vocabulary rather than about a disk — and a caller drawing a list
    // wants the words. Whoever draws the screen asks the vocabulary that
    // question once, for the screen, rather than once per disk.
    let say = |word: &alo_strings::Word| strings.say(&word.key(), &Filling::nothing()).into_text();

    for disk in disks.every_whole_disk() {
        let called = called(&disk, strings);
        if from.is_the_disk(&disk.name) {
            refused.push(NotOffered {
                disk: disk.name,
                called,
                why: say(&words::THAT_IS_THIS_INSTALLER),
            });
            continue;
        }

        // **Asked as the road that keeps what is there**, so a disk holding
        // another system comes back refused — and that refusal is how this file
        // learns the disk is not empty. Asking as though the person had already
        // chosen to replace would be answering a question they have not been
        // put yet.
        match disks.may_receive(std::path::Path::new(&disk.name), Replacing::Nothing) {
            Ok(()) => offered.push(Offered {
                disk: disk.name,
                called,
                holding: say(&words::NOTHING_IS_ON_IT),
                erases: false,
            }),
            // **Offered with the cost said, unless we cannot tell which disk
            // this installer is on.** Hiding it would decide for the person;
            // the review that follows is where they decide.
            Err(Unsuitable::HoldsAnotherSystem) => match from {
                TheMedium::Unlabelled => refused.push(NotOffered {
                    disk: disk.name,
                    called,
                    why: say(&words::CANNOT_TELL_WHICH_DISK_THIS_IS),
                }),
                // **Said before the name is handed over**, because the
                // sentence is worked out from the disk and the struct takes
                // the disk's name by value.
                TheMedium::ThisDisk(_) => {
                    let on_it = holding(&disk, strings);
                    offered.push(Offered {
                        disk: disk.name,
                        called,
                        holding: on_it,
                        erases: true,
                    });
                }
            },
            Err(Unsuitable::HoldsThisInstaller) => refused.push(NotOffered {
                disk: disk.name,
                called,
                why: say(&words::THAT_IS_THIS_INSTALLER),
            }),
            // **Neither of the last two can be reached, and both are answered
            // anyway.** Every name in this loop came from the lister a moment
            // ago and every one of them is a whole disk, so *not listed* and
            // *not a whole disk* are impossible here — and an exhaustive match
            // with no catch-all is what makes a sixth kind of refusal a
            // compile error rather than a disk quietly offered.
            Err(
                Unsuitable::CannotBeWritten | Unsuitable::NotListed | Unsuitable::NotAWholeDisk,
            ) => {
                refused.push(NotOffered {
                    disk: disk.name,
                    called,
                    why: say(&words::THAT_DISK_IS_IN_USE),
                });
            }
        }
    }

    WhatToShow { offered, refused }
}

/// What a person reads a disk as.
///
/// **The maker's name and the size**, which is how somebody tells one disk from
/// another when both are 1 TB. A device that reports no maker is read as its
/// own path — honest, and what this fall-back must never do is invent a name,
/// because a person choosing a disk by a name we made up is choosing blind.
fn called(disk: &AWholeDisk, strings: &Strings) -> String {
    let both = |name: &str, bytes: u64| {
        strings
            .say(
                &words::A_DISK_AND_ITS_SIZE.key(),
                &Filling::of("disk", name).and("size", roughly(bytes)),
            )
            .into_text()
    };
    match (disk.model.as_deref(), disk.bytes) {
        (Some(model), Some(bytes)) => both(model, bytes),
        (Some(model), None) => model.to_owned(),
        (None, Some(bytes)) => both(&disk.name, bytes),
        (None, None) => disk.name.clone(),
    }
}

/// What is on a disk, in words.
fn holding(disk: &AWholeDisk, strings: &Strings) -> String {
    disk.used.map_or_else(
        || {
            strings
                .say(&words::SOMETHING_IS_ON_IT.key(), &Filling::nothing())
                .into_text()
        },
        |used| {
            strings
                .say(
                    &words::ANOTHER_SYSTEM_AND_FILES.key(),
                    &Filling::of("files", roughly(used)),
                )
                .into_text()
        },
    )
}

/// A size as a person says it, in the units the disk was sold in.
///
/// **Powers of a thousand and not of 1024**, which is how every disk is
/// labelled and sold: a disk somebody bought as 1 TB holds 1,000,204,886,016
/// bytes, and reading it back to them as *931 GB* is correct arithmetic about a
/// question they did not ask.
fn roughly(bytes: u64) -> String {
    const TB: u64 = 1_000_000_000_000;
    const GB: u64 = 1_000_000_000;
    if bytes >= TB {
        let tenths = (bytes / (TB / 10)).max(10);
        if tenths.is_multiple_of(10) {
            format!("{} TB", tenths / 10)
        } else {
            format!("{}.{} TB", tenths / 10, tenths % 10)
        }
    } else {
        format!("{} GB", bytes / GB)
    }
}

#[cfg(test)]
#[path = "which_disks_may_be_offered_tests.rs"]
mod tests;
