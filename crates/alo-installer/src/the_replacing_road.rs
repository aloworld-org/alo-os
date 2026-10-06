//! Whether this computer may take the road that erases its disk.
//!
//! Task 7 names three refusals. One of them,
//! [`Refusal::NotTheWord`](crate::ended::Refusal::NotTheWord), is about what a
//! person typed and lives with the consent that reads it
//! ([`crate::erasing_consent`]). The other two are about the machine, and they
//! are decided here, before the question is ever asked.
//!
//! **Asked before, not after.** A person should never be told what replacing
//! costs, shown their own files, and asked to type the word, only to be refused
//! afterwards for something the installer knew from the start. Everything here
//! is read during [`crate::checking`], long before anything is said about
//! erasing.
//!
//! # The order is deliberate: the refusal a person cannot fix comes first
//!
//! A person whose Windows volume is protected can decrypt it and come back. A
//! person whose computer has one disk and nothing else to start from cannot
//! change that by preparing better. So the unfixable one is checked first, and
//! somebody in that position learns it immediately instead of decrypting a whole
//! disk and then hitting a wall.

use alo_installing::Replacing;
use alo_strings::{Filling, Strings, Word};

use crate::asking::is_the_word;
use crate::bitlocker::BitLocker;
use crate::deciding::{ForAloOs, Offer};
use crate::ended::Refusal;
use crate::found::Found;
use crate::machine::TheMachine;
use crate::program::Program;
use crate::sizes;
use crate::the_point_of_no_return::crossed;
use crate::what_replacing_destroys::WhatReplacingDestroys;
use crate::words;

/// The fewest disks a machine may have and still be offered this road.
///
/// Two: the one alo OS replaces, and anything at all to start from afterwards.
/// A recovery drive a person made is a disk, an external drive is a disk, and a
/// second internal drive is a disk — this does not try to tell them apart,
/// because *something else exists* is the whole question.
const AND_SOMETHING_TO_COME_BACK_TO: usize = 2;

/// Whether this machine may be offered the road that erases its disk.
///
/// # Errors
/// [`Refusal::NoWayBackAtAll`] when the machine has one disk and nothing else it
/// could start from, and [`Refusal::BitLockerNotConfirmed`] when the Windows
/// volume is protected.
pub fn may_replace(found: &Found, offer: &Offer) -> Result<Replacing, Refusal> {
    let how_many = found
        .disks
        .as_ref()
        .map_or(0, |disks| disks.every().count());
    if how_many < AND_SOMETHING_TO_COME_BACK_TO {
        return Err(Refusal::NoWayBackAtAll);
    }

    // **Anything but `Off` is a refusal, including `NotRead`.** Task 7 asks for
    // a refusal when the installer *cannot confirm* the volume is unlocked and
    // backed up — and it can never confirm the second half at all: nothing
    // Windows will tell this program says whether what unlocks the disk is
    // written down anywhere else. So a protected volume is refused on this road,
    // full stop, and "Windows did not say" is not a reason to erase somebody's
    // disk.
    //
    // `Changing` is already refused on every road by `crate::deciding`, so it
    // cannot arrive here; it is matched anyway rather than trusting a caller's
    // ordering to keep a destructive road safe.
    if found.bitlocker != BitLocker::Off {
        return Err(Refusal::BitLockerNotConfirmed(offer.windows_disk.clone()));
    }

    Ok(Replacing::TheSystemOnTheDisk)
}

/// Which road the person chose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Road {
    /// Windows stays, with alo OS beside it. **What an unrecognised answer
    /// means**, and what ADR 0023 §4 calls the default where the disk allows.
    KeepWindows,
    /// Windows is replaced. Only ever a word somebody typed.
    ReplaceWindows,
}

/// How many times the road is asked again before it is left alone.
///
/// The same number `sequence.rs` uses for Fast Startup, and for the same reason:
/// a question asked for ever is a computer a person cannot get out of. The answer
/// it ends at is the one that changes nothing.
const ASKED_AGAIN: usize = 3;

/// Ask which road, and take the answer.
///
/// **Keeping Windows is what an unrecognised answer means**, every time and after
/// the last time. Nothing a person types by accident, and nothing a console
/// returns when it is gone, can reach the road that erases a disk — that road is
/// only ever the word for it, typed.
pub fn which_road(machine: &mut impl TheMachine, strings: &Strings) -> Road {
    let answers = Filling::of(
        "keep",
        strings
            .say(&words::ANSWER_KEEP_WINDOWS.key(), &Filling::nothing())
            .into_text(),
    )
    .and(
        "replace",
        strings
            .say(&words::ANSWER_REPLACE_WINDOWS.key(), &Filling::nothing())
            .into_text(),
    );
    for _ in 0..ASKED_AGAIN {
        let typed = machine.ask(&strings.say(&words::ASK_WHICH_ROAD.key(), &answers));
        if is_the_word(&typed, words::ANSWER_REPLACE_WINDOWS, strings) {
            return Road::ReplaceWindows;
        }
        if is_the_word(&typed, words::ANSWER_KEEP_WINDOWS, strings) {
            return Road::KeepWindows;
        }
    }
    Road::KeepWindows
}

/// Say what replacing this disk would destroy, from what was read.
///
/// Three sentences, and the third is the one that makes a person stop: how much
/// is on the Windows volume and when the newest of their own files changed. When
/// the walk found none of their files it says that instead, rather than leaving
/// the sentence out — *nothing was found* and *nothing was looked for* read the
/// same to somebody who is not told which happened.
fn say_what_would_be_lost(
    machine: &mut impl TheMachine,
    strings: &Strings,
    offer: &Offer,
    disk: &ForAloOs,
) {
    let named = Filling::of("disk", disk.shown.as_str());
    say(machine, strings, words::REPLACING_DESTROYS, &named);
    say(
        machine,
        strings,
        words::WHAT_IS_ON_IT_NOW,
        &Filling::of("disk", disk.shown.as_str()).and("volume", offer.windows.letter.drive()),
    );

    let read = machine
        .run(&Program::ReadingWhatReplacingDestroys)
        .ok()
        .filter(|ran| ran.succeeded)
        .and_then(|ran| WhatReplacingDestroys::read(Some(ran.printed.as_str())));
    let Some(read) = read else {
        // Nothing is said about how much, because nothing was measured, and a
        // sentence made up here would be the one thing this screen must not be.
        return;
    };
    let used = sizes::had(read.used());
    match read.newest() {
        Some(day) => say(
            machine,
            strings,
            words::HOW_MUCH_AND_HOW_RECENT,
            &Filling::of("files", read.files().to_string())
                .and("disk", disk.shown.as_str())
                .and("used", used.as_str())
                .and("newest", day.as_str()),
        ),
        None => say(
            machine,
            strings,
            words::HOW_MUCH_AND_NOTHING_OF_YOURS,
            &Filling::of("used", used.as_str()).and("disk", disk.shown.as_str()),
        ),
    }
}

/// One sentence, looked up and put in front of the person.
fn say(machine: &mut impl TheMachine, strings: &Strings, word: Word, filling: &Filling) {
    machine.say(&strings.say(&word.key(), filling));
}

/// Walk the replacing road to the point of no return, and cross it.
///
/// In this order, and the order is the plan's:
///
/// 1. **refuse before asking** — [`may_replace`], so nobody is shown their own
///    files and asked to type a word only to be told the machine was never
///    eligible;
/// 2. **say before asking** — what is destroyed, what is on the disk, and how
///    much of it was changed how recently, all from what was read;
/// 3. **ask the second time** — the disk's name and the word together;
/// 4. **cross once**, through [`crossed`], which says there is no way back and
///    then does the thing there is no way back from.
///
/// `erase` is handed the road's answer so that the one irreversible act lives
/// with the caller that owns the disk, and this file cannot perform it by itself.
///
/// # Errors
/// Any [`Refusal`] on the way. Every one of them leaves the machine as it was.
pub fn walk<M: TheMachine, T>(
    machine: &mut M,
    strings: &Strings,
    found: &Found,
    offer: &Offer,
    disk: &ForAloOs,
    erase: impl FnOnce(&mut M, Replacing) -> T,
) -> Result<T, Refusal> {
    let replacing = may_replace(found, offer)?;
    say_what_would_be_lost(machine, strings, offer, disk);

    let word = strings
        .say(&words::ERASING_WORD.key(), &Filling::nothing())
        .into_text();
    let typed = machine.ask(&strings.say(
        &words::TYPE_THE_DISK_AND_THE_WORD.key(),
        &Filling::of("disk", disk.shown.as_str()).and("word", word.as_str()),
    ));
    crate::erasing_consent::erasing(&typed, disk, &word)?;

    Ok(crossed(machine, strings, &disk.shown, |machine| {
        erase(machine, replacing)
    }))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::disks::Disks;
    // `disks.rs`' own description of what Windows prints, reused rather than
    // copied, so this crate holds one idea of what a machine's disks look like.
    use crate::disks::tests::PRINTED as SEVERAL_DISKS;
    use alo_installing::DiskName;

    use crate::fast_startup::FastStartup;
    use crate::identities::DiskNumber;
    use crate::security_chip::SecurityChip;
    use crate::starting::Starting;
    use crate::windows_volume::WindowsVolume;

    /// A machine with one disk and nothing else to start from.
    const ONE_DISK: &str = r#"[
      {"Number":0,"FriendlyName":"Samsung SSD 870 EVO","SerialNumber":"S5Y1NJ0R123456",
       "BusType":"SATA","UniqueId":"","Size":274877906944,"PartitionStyle":"GPT",
       "IsReadOnly":false,
       "Partitions":[{"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Label":""},
                     {"PartitionNumber":3,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Windows"}]}
    ]"#;

    /// What Windows prints for the Windows volume.
    const THE_VOLUME: &str = r#"{"DriveLetter":"C","DiskNumber":0,"PartitionNumber":3,
      "Offset":123731968,"Size":274341593088,"SizeMin":34341593088,
      "SizeRemaining":128849018880}"#;

    /// A machine that was read, with this encryption and these disks.
    fn found(bitlocker: BitLocker, disks: &str) -> Found {
        Found {
            starting: Starting {
                uefi: Some(true),
                secure_boot: Some(false),
            },
            chip: SecurityChip::Ready,
            bitlocker,
            memory: Some(16 * 1024 * 1024 * 1024),
            windows: WindowsVolume::read(Some(THE_VOLUME)),
            disks: Disks::read(Some(disks)),
            an_entry_is_named_alo_os: Some(false),
            fast_startup: FastStartup::Off,
        }
    }

    /// The disk the road would erase, as the offer carries it.
    fn the_disk() -> ForAloOs {
        ForAloOs {
            number: DiskNumber(0),
            shown: "Samsung SSD 870 EVO".to_owned(),
            after_the_restart: DiskName::named("ata-Samsung_SSD_870_EVO_S5Y1").unwrap(),
            beside: None,
        }
    }

    /// The offer made about that machine.
    fn offer() -> Offer {
        Offer {
            windows: WindowsVolume::read(Some(THE_VOLUME)).unwrap(),
            shrink: WindowsVolume::read(Some(THE_VOLUME))
                .unwrap()
                .shrink()
                .unwrap(),
            windows_disk: "Samsung SSD 870 EVO".to_owned(),
            disks_for_alo_os: Vec::new(),
            the_windows_disk: Some(the_disk()),
            // This module is about the road that *replaces* Windows, and these
            // tests say what it does when that is the only road offered. The
            // same-disk road is a different one and is left absent here on
            // purpose, so nothing below silently starts depending on it.
            beside_windows: None,
        }
    }

    /// A machine that answers each question with the next of these.
    struct Answering {
        /// What it will say, in order.
        answers: Vec<String>,
        /// How many questions it was asked.
        asked: usize,
    }

    impl Answering {
        fn of(answers: &[&str]) -> Self {
            Self {
                answers: answers.iter().map(|a| (*a).to_owned()).collect(),
                asked: 0,
            }
        }
    }

    impl TheMachine for Answering {
        fn say(&mut self, _said: &alo_strings::Said) {}

        fn ask(&mut self, _said: &alo_strings::Said) -> String {
            let answer = self.answers.get(self.asked).cloned().unwrap_or_default();
            self.asked += 1;
            answer
        }

        fn run(&mut self, _program: &Program) -> std::io::Result<crate::machine::Ran> {
            unreachable!("the question runs no program")
        }

        fn downloaded_into(&mut self) -> std::io::Result<std::path::PathBuf> {
            unreachable!()
        }

        fn this_program(&mut self) -> std::io::Result<std::path::PathBuf> {
            unreachable!()
        }

        fn read(&mut self, _file: &std::path::Path) -> std::io::Result<Vec<u8>> {
            unreachable!()
        }

        fn write(&mut self, _file: &std::path::Path, _bytes: &[u8]) -> std::io::Result<()> {
            unreachable!()
        }

        fn pause(&mut self, _for_as_long_as: std::time::Duration) {
            unreachable!("nothing here waits on a timer")
        }
    }

    /// This installer's own words.
    fn source() -> Strings {
        Strings::of(words::installer_words().unwrap())
    }

    /// **The word, and only the word, opens the replacing road.**
    #[test]
    fn the_word_opens_the_replacing_road() {
        for typed in ["replace", "  Replace ", "REPLACE"] {
            let mut machine = Answering::of(&[typed]);
            assert_eq!(
                which_road(&mut machine, &source()),
                Road::ReplaceWindows,
                "{typed:?}"
            );
            assert_eq!(machine.asked, 1, "it should not ask twice after an answer");
        }
    }

    /// **The other word keeps Windows**, and is also read forgivingly.
    #[test]
    fn the_other_word_keeps_windows() {
        for typed in ["keep", "Keep", "  KEEP  "] {
            let mut machine = Answering::of(&[typed]);
            assert_eq!(
                which_road(&mut machine, &source()),
                Road::KeepWindows,
                "{typed:?}"
            );
            assert_eq!(machine.asked, 1);
        }
    }

    /// **Nothing else reaches the replacing road** — and running out of
    /// attempts keeps Windows rather than falling through to the other answer.
    ///
    /// The console that is gone is in here on purpose: `ask` answers with an
    /// empty line when it cannot read, and an installer that read silence as
    /// *replace* would erase a disk nobody was sitting at.
    #[test]
    fn nothing_else_reaches_the_replacing_road() {
        for answers in [
            vec![""],
            vec!["", "", ""],
            vec!["yes"],
            vec!["y"],
            vec!["erase"],
            vec!["replac"],
            vec!["replace windows"],
            vec!["no", "maybe", "what"],
        ] {
            let mut machine = Answering::of(&answers);
            assert_eq!(
                which_road(&mut machine, &source()),
                Road::KeepWindows,
                "{answers:?}"
            );
        }
    }

    /// **It gives up rather than asking for ever**, and what it gives up on is
    /// keeping Windows.
    #[test]
    fn it_asks_a_bounded_number_of_times() {
        let mut machine = Answering::of(&["what", "eh", "pardon", "replace"]);
        assert_eq!(which_road(&mut machine, &source()), Road::KeepWindows);
        assert_eq!(machine.asked, ASKED_AGAIN);
    }

    /// **A machine with somewhere to come back to, and no protection, may take
    /// this road.**
    #[test]
    fn several_disks_and_no_protection_may_take_this_road() {
        assert_eq!(
            may_replace(&found(BitLocker::Off, SEVERAL_DISKS), &offer()),
            Ok(Replacing::TheSystemOnTheDisk)
        );
    }

    /// **One disk is refused, and it is refused before anything else** — the one
    /// a person cannot fix by preparing better.
    ///
    /// Checked with the volume protected too, which would also be a refusal:
    /// the answer must still be the unfixable one.
    #[test]
    fn one_disk_is_refused_first_because_it_cannot_be_fixed() {
        for bitlocker in [BitLocker::Off, BitLocker::On, BitLocker::NotRead] {
            assert_eq!(
                may_replace(&found(bitlocker, ONE_DISK), &offer()),
                Err(Refusal::NoWayBackAtAll),
                "{bitlocker:?}"
            );
        }
    }

    /// **Anything but "not encrypted" is refused, including "Windows did not
    /// say".**
    ///
    /// The installer can never confirm that what unlocks the disk is kept
    /// anywhere else, so a protected volume does not take this road — and an
    /// unknown state is not a reason to erase somebody's disk.
    #[test]
    fn anything_but_unencrypted_is_refused_and_names_the_disk() {
        for bitlocker in [BitLocker::On, BitLocker::Changing, BitLocker::NotRead] {
            assert_eq!(
                may_replace(&found(bitlocker, SEVERAL_DISKS), &offer()),
                Err(Refusal::BitLockerNotConfirmed(
                    "Samsung SSD 870 EVO".to_owned()
                )),
                "{bitlocker:?}"
            );
        }
    }

    /// **Disks that could not be read are no disks**, so the road is refused.
    ///
    /// A machine this installer could not count is not a machine it may erase.
    #[test]
    fn disks_that_could_not_be_read_are_refused() {
        assert_eq!(
            may_replace(&found(BitLocker::Off, "not json"), &offer()),
            Err(Refusal::NoWayBackAtAll)
        );
    }
}
