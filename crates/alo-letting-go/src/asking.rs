//! The one act a person asks for: how it is left for the machine, and how the
//! machine takes it.
//!
//! [ADR 0045](../../../docs/decisions/0045-what-undoing-rewinds-to.md) point 5:
//! *what an undo may keep is visible and forgettable, and **forgetting it is
//! one act**.* `alo_keeping_up::WhatWasKept::forgetting` has been the sentence a
//! person approves since that point was built, and until now nothing carried it
//! out — a sentence a machine could show and could not act on.
//!
//! # The road, and the two roads it is not
//!
//! Removing a read-only snapshot needs `CAP_SYS_ADMIN` and a person's session
//! holds none, so the act has to cross into something privileged. The owner
//! narrowed that on 2026-09-22: **it is a person's act in Settings, and never a
//! broker verb.** The seventh term's reason is why — the broker's road exists to
//! carry acts an agent may propose under an approval, so a road that can carry
//! this act *at all* is a road worth attacking, and keeping it off that list
//! costs a person nothing, because a person is already at their own machine.
//!
//! What is left is a file. The person's session writes one into
//! [`THE_ASKING`] — a folder on `/run` that anybody may put a name in and
//! nobody may read — and a `systemd` path unit starts
//! `alo-forgetting.service` the moment one appears. It is the shape
//! [ADR 0049](../../../docs/decisions/0049-the-network-is-changed-through-the-broker-and-its-password-never-reaches-the-agent.md)
//! §3 and [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
//! already use for what a door cannot carry, with the door taken out of it.
//!
//! # The file names nothing, and that is the whole design
//!
//! [`Asked`] carries a format number and the moment the person approved. It
//! carries **no person, no folder, no path and no turn** — there is nothing in
//! it to aim. Whose act it is, is the user the filesystem records as having
//! written it ([`crate::WhoOwns`]), which the kernel sets and a writer cannot
//! spell for itself; the machine matches that against the owner of the settings
//! folder each person's own session wrote down beside what it kept, and forgets
//! for whoever matches and for nobody else.
//!
//! So a person who leaves an asking can cause exactly one thing: their own
//! machine forgetting **their own** undo. Somebody else's is not a refusal that
//! had to be got right — it is not expressible.
//!
//! # Every asking is taken, and taken first
//!
//! [`taken`] removes everything it looked at before anything is decided, and
//! whatever it decides. Two reasons, and both are about a destructive act:
//!
//! - **One approval is one execution** (ADR 0001 §5). An approval that survived
//!   being acted on is one that can be acted on again, at a moment nobody
//!   chose.
//! - **And a path unit re-fires while its folder is not empty.** An asking left
//!   behind because the disk would not answer is a unit started again, and
//!   again, for as long as the fault lasts.
//!
//! What it costs is written down rather than hidden: an asking taken on a
//! machine that could not carry it out is **spent**, the journal says so, and
//! the person asks again. That is the right way round for an act that removes a
//! person's own history — the wrong way round is an approval lying about on a
//! disk waiting for a moment that suits it.
//!
//! # And it lapses
//!
//! An approval is never a session (ADR 0001 §5), so an asking older than
//! [`THE_LIFETIME`] is taken and not carried out. `/run` is cleared when the
//! machine starts, so no asking survives a restart either; what this adds is
//! the machine that was left running with the unit stopped, where an hour-old
//! approval would otherwise forget turns made since it was given.

use std::collections::BTreeSet;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_keeping_up::WhatWasKept;
use alo_strings::{Filling, Said, Strings, Word};
use serde::{Deserialize, Serialize};

use crate::whose::WhoOwns;
use crate::words;

/// Where a person's session leaves the act it was approved for.
///
/// On `/run` on purpose: it is cleared when the machine starts, so an approval
/// can never outlive the session it was given in. The directory is
/// `1733 root:root` — anybody may put a name in it, nobody may read what is in
/// it, and nobody may remove anybody else's — which is the ordinary drop box,
/// and it is all the folder has to be, because nothing in it carries authority.
pub const THE_ASKING: &str = "/run/alo/asked-to-forget";

/// The shape of the file, written as `"format":1` at its top.
pub const THE_FORMAT: u32 = 1;

/// How long an approval to forget is worth acting on.
///
/// **An hour**, and it is a bound rather than a promise of promptness: the path
/// unit starts the machine's half within moments of the file appearing, so an
/// asking that reaches this age is one a stopped or masked unit sat on. Acting
/// on it then would forget turns the person made after they approved anything,
/// which is a machine answering a question nobody has asked for an hour.
pub const THE_LIFETIME: Duration = Duration::from_secs(60 * 60);

/// How far ahead of this machine's clock an asking may say it was approved.
///
/// The session and the unit read one machine's clock, so this is the width of a
/// second boundary between two reads of it and nothing more. It is
/// `alo_broker::spent`'s allowance, at the same width and for the same reason,
/// and it is spelt here rather than borrowed because this crate deliberately
/// names no door.
const AHEAD: Duration = Duration::from_secs(2);

/// What a person's session leaves when they approve forgetting everything their
/// machine was keeping for them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Asked {
    /// [`THE_FORMAT`].
    format: u32,
    /// The moment they approved it, in whole seconds since the epoch.
    approved: u64,
}

/// Why an asking was not carried out.
///
/// Every one of them is an asking that was **taken** all the same, for the
/// reason this module's documentation gives.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotTheAsking {
    /// It is not JSON, or not this file's shape.
    #[error("{at} is not an asking as alo OS writes it, so nothing was forgotten for it: {why}")]
    NotItsShape {
        /// The file.
        at: PathBuf,
        /// What the reader said.
        why: String,
    },
    /// It says it is a shape this alo OS does not read.
    #[error("{at} says it is format {found}, and this alo OS writes format 1")]
    AnotherFormat {
        /// The file.
        at: PathBuf,
        /// What it said.
        found: u32,
    },
    /// It was approved too long ago, or at a moment that has not happened.
    #[error(
        "{at} was approved more than an hour before this machine looked at it, so it was not carried out"
    )]
    Lapsed {
        /// The file.
        at: PathBuf,
    },
    /// The machine would not say who left it, or would not let it be read.
    #[error("{at} was not read, so nothing was forgotten for it: {why}")]
    NotRead {
        /// The file.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },
}

/// A person's session could not leave the asking, so nothing on this machine
/// has been forgotten and nothing has changed.
///
/// There is no `Display`, for [`crate::FileNotRead`]'s reason: the only road to
/// words is [`NotAsked::said`], in the language the person reads.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotAsked {
    /// Where the asking would have gone.
    at: PathBuf,
    /// What the machine said, for whoever is fixing alo OS.
    why: io::ErrorKind,
}

/// Who asked, and everything the walk would not use.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WhatWasAsked {
    /// The users the filesystem says left an asking this machine will act on.
    by: BTreeSet<u32>,
    /// Everything it would not use, in English, for the journal.
    said: Vec<String>,
}

impl Asked {
    /// A person approved it at `at`.
    #[must_use]
    pub fn at(at: SystemTime) -> Self {
        Self {
            format: THE_FORMAT,
            approved: at
                .duration_since(SystemTime::UNIX_EPOCH)
                .map_or(0, |since| since.as_secs()),
        }
    }

    /// The moment they approved it.
    #[must_use]
    pub fn approved(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(self.approved)
    }

    /// The sentence a person approves before any of this.
    ///
    /// It is `alo_keeping_up::WhatWasKept::forgetting` and nothing else — the
    /// one door, so that a surface offering this act cannot word it itself.
    /// ADR 0045 point 5 asks the sentence to say both halves, that nothing an
    /// agent has changed can be put back afterwards and that the space it holds
    /// is freed, and that sentence already says them. A second wording of one
    /// moment is the thing this repository refuses everywhere else and there is
    /// no reason for it to be different here.
    #[must_use]
    pub fn the_sentence(strings: &Strings) -> Said {
        WhatWasKept::forgetting(strings)
    }

    /// This file's text, read — the one door, so nothing reaches a reader
    /// without having been through every refusal above.
    ///
    /// # Errors
    /// [`NotTheAsking`]. Nothing is forgotten for an asking this refused.
    pub fn read(text: &str, at: &Path, now: SystemTime) -> Result<Self, NotTheAsking> {
        let said: serde_json::Value =
            serde_json::from_str(text).map_err(|why| NotTheAsking::NotItsShape {
                at: at.to_owned(),
                why: why.to_string(),
            })?;
        match said.get("format").and_then(serde_json::Value::as_u64) {
            Some(found) if found == u64::from(THE_FORMAT) => {}
            Some(found) => {
                return Err(NotTheAsking::AnotherFormat {
                    at: at.to_owned(),
                    found: u32::try_from(found).unwrap_or(u32::MAX),
                });
            }
            None => {
                return Err(NotTheAsking::NotItsShape {
                    at: at.to_owned(),
                    why: "it does not say which format it is".to_owned(),
                });
            }
        }
        let read: Self = serde_json::from_str(text).map_err(|why| NotTheAsking::NotItsShape {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
        read.still_worth_acting_on(at, now)?;
        Ok(read)
    }

    /// Whether this approval is one this machine will act on now.
    fn still_worth_acting_on(self, at: &Path, now: SystemTime) -> Result<(), NotTheAsking> {
        let approved = self.approved();
        let lapsed = match now.duration_since(approved) {
            Ok(since) => since > THE_LIFETIME,
            // Approved after the moment the machine is reading it at: a second
            // boundary between two reads of one clock is allowed, and anything
            // further is a file claiming a moment that has not happened.
            Err(ahead) => ahead.duration() > AHEAD,
        };
        if lapsed {
            return Err(NotTheAsking::Lapsed { at: at.to_owned() });
        }
        Ok(())
    }
}

impl NotAsked {
    /// Where the asking would have gone.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// What the machine said, for whoever is fixing alo OS.
    #[must_use]
    pub const fn why(&self) -> io::ErrorKind {
        self.why
    }

    /// The string this crate declares for this refusal.
    ///
    /// One, not several: what a person does about it is the same whichever of
    /// them it was — nothing was forgotten and nothing has changed — and the
    /// machine's own reason is on the journal for whoever is fixing alo OS.
    #[must_use]
    pub const fn word(&self) -> Word {
        words::FORGETTING_NOT_ASKED_FOR
    }

    /// What this says, in the language the person reads. Never fails and never
    /// panics.
    ///
    /// **The file is not in it**, deliberately. Where an asking would have gone
    /// is `/run/alo/asked-to-forget`, which is the machine's own plumbing: a
    /// person cannot act on it, and naming it would send them looking at a
    /// folder they may not read. It is on the journal instead, for whoever
    /// administers the machine, and [`NotAsked::at`] is how a surface gets it
    /// there.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

impl WhatWasAsked {
    /// The users who asked, as the filesystem named them.
    #[must_use]
    pub fn by(&self) -> &BTreeSet<u32> {
        &self.by
    }

    /// Whether anybody asked at all.
    #[must_use]
    pub fn nobody(&self) -> bool {
        self.by.is_empty()
    }

    /// Everything it would not use.
    #[must_use]
    pub fn said(&self) -> &[String] {
        &self.said
    }
}

/// Take everything in `folder`: read each asking, remove it, and answer with
/// the users whose askings this machine will act on.
///
/// **Nothing is left behind**, whatever was decided about it — this module's
/// documentation has both reasons, and they are the two halves of not letting a
/// destructive approval outlive the moment it was looked at.
#[must_use]
pub fn taken(folder: &Path, owner: &dyn WhoOwns, now: SystemTime) -> WhatWasAsked {
    let mut asked = WhatWasAsked::default();
    let entries = match std::fs::read_dir(folder) {
        Ok(entries) => entries,
        Err(why) => {
            if folder.exists() {
                asked
                    .said
                    .push(format!("{} was not read: {why}", folder.display()));
            }
            return asked;
        }
    };
    for entry in entries {
        let at = match entry {
            Ok(entry) => entry.path(),
            Err(why) => {
                asked
                    .said
                    .push(format!("{} was not walked: {why}", folder.display()));
                continue;
            }
        };
        match one_asking(&at, owner, now) {
            Ok(by) => {
                asked.by.insert(by);
            }
            Err(why) => asked.said.push(why.to_string()),
        }
        cleared(&at, &mut asked.said);
    }
    asked
}

/// One asking, read: who left it, and whether it is one to act on.
fn one_asking(at: &Path, owner: &dyn WhoOwns, now: SystemTime) -> Result<u32, NotTheAsking> {
    let what = std::fs::symlink_metadata(at).map_err(|why| NotTheAsking::NotRead {
        at: at.to_owned(),
        why: why.to_string(),
    })?;
    if !what.is_file() {
        return Err(NotTheAsking::NotRead {
            at: at.to_owned(),
            why: "it is not a file of its own".to_owned(),
        });
    }
    let by = owner.of(at).map_err(|why| NotTheAsking::NotRead {
        at: at.to_owned(),
        why: why.to_string(),
    })?;
    let text = std::fs::read_to_string(at).map_err(|why| NotTheAsking::NotRead {
        at: at.to_owned(),
        why: why.to_string(),
    })?;
    Asked::read(&text, at, now)?;
    Ok(by)
}

/// Remove one name from the folder, whatever it turned out to be.
fn cleared(at: &Path, said: &mut Vec<String>) {
    let gone = if at.is_dir() {
        std::fs::remove_dir_all(at)
    } else {
        std::fs::remove_file(at)
    };
    if let Err(why) = gone {
        said.push(format!(
            "{} was looked at and not taken, so it may be looked at again: {why}",
            at.display()
        ));
    }
}

/// Leave one asking in `folder`, from the session of the person who approved it.
///
/// This is the **person's** half and holds no capability: it writes one small
/// file, owned by them because they wrote it, and that is the whole of what
/// crosses into the privileged half.
///
/// The name is unique and **is never read** — it exists so that two askings
/// cannot collide and so that nobody can stop somebody else asking by taking a
/// name first. What the machine reads is the file's content and who owns it.
///
/// # Errors
/// [`NotAsked`], in a sentence the person reads. Nothing has changed, and
/// nothing on this machine has been forgotten.
pub fn ask(folder: &Path, now: SystemTime) -> Result<PathBuf, NotAsked> {
    let asked = Asked::at(now);
    let text = serde_json::to_string(&asked).map_err(|_| NotAsked {
        at: folder.to_owned(),
        why: io::ErrorKind::InvalidData,
    })?;
    let at = folder.join(a_name(now));
    let written = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&at)
        .and_then(|mut file| {
            use std::io::Write;

            file.write_all(text.as_bytes())
                .and_then(|()| file.sync_all())
        });
    match written {
        Ok(()) => Ok(at),
        Err(why) => {
            // A half-written asking is worse than none: the machine's half
            // would read it, refuse it and take it, and the person would be
            // told nothing at all.
            let _ = std::fs::remove_file(&at);
            Err(NotAsked {
                at,
                why: why.kind(),
            })
        }
    }
}

/// A name no other asking will have, and that nothing ever reads.
fn a_name(now: SystemTime) -> String {
    let since = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |since| since.as_nanos());
    format!("{}-{since}", std::process::id())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None, Err or index is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{AnOwner, a_folder, in_english, in_english_with_the_undo_words, noon};

    /// A folder for askings, on a real disk.
    fn an_asking_folder(what: &str) -> PathBuf {
        a_folder(&format!("asking-{what}"))
    }

    /// **What the person approves is the sentence `alo-keeping-up` already
    /// says**, rendered from the machine's vocabulary rather than written a
    /// second time here — and this crate declares no sentence about forgetting
    /// at all.
    #[test]
    fn what_the_person_approves_is_the_sentence_alo_keeping_up_already_says() {
        let strings = in_english_with_the_undo_words();
        let said = Asked::the_sentence(&strings);
        assert_eq!(said.text(), WhatWasKept::forgetting(&strings).text());
        assert!(!said.is_a_bug(), "{said}");
        assert!(
            said.text().contains("none of them can be put back"),
            "{said}"
        );
        assert!(said.text().contains("space"), "{said}");

        // And this crate words the act nowhere. The one sentence of its own
        // that mentions forgetting at all is the refusal, which is about the
        // act **not** having happened.
        for word in words::EVERY_WORD {
            assert_ne!(
                word.says(),
                said.text(),
                "{} is a second wording of the sentence a person approves",
                word.named()
            );
        }
        let about: Vec<&str> = words::EVERY_WORD
            .iter()
            .filter(|word| word.says().contains("forget"))
            .map(|word| word.named())
            .collect();
        assert_eq!(about, [words::FORGETTING_NOT_ASKED_FOR.named()]);
        assert!(
            words::FORGETTING_NOT_ASKED_FOR
                .says()
                .contains("nothing has changed")
        );
    }

    /// **An asking a person's session left is the one this machine acts on**,
    /// and who left it is the filesystem's answer rather than anything in the
    /// file.
    #[test]
    fn an_asking_is_left_by_a_session_and_taken_by_the_machine() {
        let folder = an_asking_folder("taken");
        let at = ask(&folder, noon()).unwrap();
        assert!(at.exists());

        let asked = taken(&folder, &AnOwner::everything_owned_by(1000), noon());
        assert_eq!(asked.by(), &BTreeSet::from([1000]));
        assert!(asked.said().is_empty(), "{:?}", asked.said());
        assert!(!at.exists(), "the asking was left behind");
    }

    /// **The file names nobody and nothing.** There is no person, no folder, no
    /// path and no turn in it, so there is nothing in it to aim.
    #[test]
    fn the_file_names_nobody_and_nothing() {
        let text = serde_json::to_string(&Asked::at(noon())).unwrap();
        let table: serde_json::Value = serde_json::from_str(&text).unwrap();
        let keys: BTreeSet<&str> = table
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, BTreeSet::from(["format", "approved"]), "{text}");
        assert_eq!(
            Asked::read(&text, Path::new("/run/alo/asked-to-forget/one"), noon())
                .unwrap()
                .approved(),
            noon()
        );
    }

    /// **An asking that lapsed is taken and never carried out** — an approval
    /// is never a session, and one an hour old would forget turns made after it
    /// was given.
    #[test]
    fn an_asking_that_lapsed_is_taken_and_never_carried_out() {
        let folder = an_asking_folder("lapsed");
        let at = ask(&folder, noon() - THE_LIFETIME - Duration::from_secs(1)).unwrap();

        let asked = taken(&folder, &AnOwner::everything_owned_by(1000), noon());
        assert!(asked.nobody(), "a lapsed approval was acted on");
        assert_eq!(asked.said().len(), 1);
        assert!(asked.said()[0].contains("more than an hour"));
        assert!(!at.exists(), "a lapsed asking was left to be found again");
    }

    /// **An asking from a clock that has not got there yet is refused**, beyond
    /// the width of one second boundary between two reads of one clock.
    #[test]
    fn an_asking_from_a_moment_that_has_not_happened_is_refused() {
        let at = Path::new("/run/alo/asked-to-forget/one");
        let text = serde_json::to_string(&Asked::at(noon() + AHEAD)).unwrap();
        assert!(Asked::read(&text, at, noon()).is_ok());

        let ahead = serde_json::to_string(&Asked::at(noon() + Duration::from_secs(60))).unwrap();
        assert!(matches!(
            Asked::read(&ahead, at, noon()),
            Err(NotTheAsking::Lapsed { .. })
        ));
    }

    /// **An asking that is not this file is taken and refused whole**, another
    /// format first of all, so that a file a later alo OS wrote is not reported
    /// as a missing field.
    #[test]
    fn anything_that_is_not_this_file_is_taken_and_refused_whole() {
        let folder = an_asking_folder("not-the-shape");
        std::fs::write(folder.join("one"), "not json at all").unwrap();
        std::fs::write(folder.join("two"), r#"{"format":9,"approved":1760000000}"#).unwrap();
        std::fs::write(folder.join("three"), r#"{"approved":1760000000}"#).unwrap();

        let asked = taken(&folder, &AnOwner::everything_owned_by(1000), noon());
        assert!(asked.nobody(), "{:?}", asked.by());
        assert_eq!(asked.said().len(), 3);
        assert!(asked.said().iter().any(|said| said.contains("format 9")));
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
    }

    /// **Nothing that is not a file of its own is read**, which is the oldest
    /// way there is of making a privileged reader answer about somebody else's
    /// name — and it is taken too, so a folder anybody may write to cannot be
    /// filled with names that start this unit again and again.
    #[test]
    fn nothing_that_is_not_a_file_of_its_own_is_read_and_it_is_taken_anyway() {
        let folder = an_asking_folder("not-a-file");
        std::fs::create_dir_all(folder.join("a-directory")).unwrap();

        let asked = taken(&folder, &AnOwner::everything_owned_by(1000), noon());
        assert!(asked.nobody());
        assert_eq!(asked.said().len(), 1);
        assert!(asked.said()[0].contains("not a file of its own"));
        assert_eq!(std::fs::read_dir(&folder).unwrap().count(), 0);
    }

    /// **A folder that is not there is nobody, not a refusal** — the ordinary
    /// state of a machine where nobody has asked for anything.
    #[test]
    fn a_folder_that_is_not_there_is_nobody() {
        let asked = taken(
            &an_asking_folder("nowhere").join("never-made"),
            &AnOwner::everything_owned_by(1000),
            noon(),
        );
        assert!(asked.nobody());
        assert!(asked.said().is_empty());
    }

    /// **A person whose asking could not be written is told, and nothing has
    /// changed** — never a surface that reported an act nothing carried out.
    #[test]
    fn an_asking_that_could_not_be_written_is_said_and_changes_nothing() {
        let folder = an_asking_folder("no-folder").join("not-there");
        let refused = ask(&folder, noon()).unwrap_err();
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(!folder.exists());
    }
}
