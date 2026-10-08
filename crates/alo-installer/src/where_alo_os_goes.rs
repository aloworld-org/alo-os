//! Where alo OS goes, once a person has said they want both systems.
//!
//! The installer plan's task 4, and the owner's instruction of 2026-10-06:
//! *"I want us to include the function/feature where the users chooses to run 2
//! os or run only alo os"*. The first half of that is
//! `crate::the_replacing_road::which_road` — both systems, or alo OS on its
//! own. This is the question that follows the first answer, and only sometimes.
//!
//! # Asked only when there is something to answer
//!
//! A computer with one disk has one place alo OS can go: the disk Windows is
//! on, beside it. A computer whose only empty disk is big enough has one place
//! too: that disk. Neither is a choice, and **a question with one answer is not
//! a question** — it is an invitation to type something that will be refused.
//! So this is asked when a person's computer has both places and not otherwise.
//!
//! # The two places cost different things, and each says its own
//!
//! `crate::words::ASK_WHICH_ROAD` used to hold a question, a description of
//! each road and a promise — *keeping it changes nothing you cannot undo* — in
//! one sentence. That promise is honest about the empty-disk road, where
//! Windows lends the installer's area and gets it back whether the install
//! finished or not. It is **not** honest about the same-disk road, where
//! Windows gives up its space and does not get it back for as long as alo OS is
//! there. That is the road working, not the road failing.
//!
//! So the promise came apart into `WINDOWS_DOES_NOT_GET_IT_BACK` and
//! `WINDOWS_LENDS_THE_AREA`, each said on the road it is true of, and that key
//! retired (ADR 0068). **Neither of them says the space comes back if alo OS is
//! removed later**, because today it does not: `crate::removing` clears a disk
//! alo OS has to itself and refuses a disk Windows is on. What is owed there is
//! in the installer plan's task 4.
//!
//! # The empty disk is what an answer nobody understands means
//!
//! The same rule `which_road` follows, for the same reason, and here it picks
//! the empty disk: that road takes nothing of the person's beyond an area the
//! installer gives back. Nothing a person types by accident, and nothing a
//! console returns when it is gone, can repartition the disk their Windows is
//! on.

use alo_strings::{Filling, Strings};

use crate::asking::{ASKED_AGAIN, is_the_word};
use crate::deciding::{ForAloOs, Offer};
use crate::machine::TheMachine;
use crate::sequence::say;
use crate::words;

/// Where alo OS goes, on the road that keeps Windows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Place {
    /// On the disk Windows is on, in space Windows gives up. Nothing of the
    /// person's is erased, and Windows does not get that space back.
    TheSameDisk,
    /// On an empty disk of its own, which alo OS takes whole. **What an answer
    /// nobody understands means**, wherever it is a place at all.
    TheOtherDisk,
}

/// The places this computer actually has, and so which question to ask.
///
/// Read from the offer rather than from a setting, because **a disk carrying
/// the keep-Windows shrink is the disk Windows is on** — that is what the
/// shrink is for, and `crate::deciding` gives it to no other disk. So this
/// asks which disk is which by what it would do rather than by its number, and
/// there is no second switch that can disagree with the first.
///
/// It read nothing until 2026-10-08, when the installer plan's task 6 opened
/// the road by offering that disk — term 5 of the owner's ruling of
/// 2026-10-06, *flipped last, when the whole road exists and not before.*
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Places<'a> {
    /// Both, so the person is asked.
    Both {
        /// The disk Windows is on.
        same: &'a ForAloOs,
        /// The empty disks, in the order they are offered.
        other: Vec<&'a ForAloOs>,
    },
    /// Only the disk Windows is on, so nothing is asked.
    OnlyTheSameDisk(&'a ForAloOs),
    /// Only empty disks, so nothing is asked.
    OnlyOtherDisks(Vec<&'a ForAloOs>),
    /// Neither, which `crate::deciding` refuses before this is reached.
    None,
}

/// Which places this computer has.
#[must_use]
pub fn places(offer: &Offer) -> Places<'_> {
    // A disk carrying the shrink that keeps Windows **is** the disk Windows is
    // on: that is what the shrink is for, and `deciding.rs` gives it to no
    // other disk. So this asks which disk is which by what it would do rather
    // than by its number, and a disk that stopped carrying the shrink would
    // stop being offered as the same-disk road in the same change.
    let same = offer
        .disks_for_alo_os
        .iter()
        .find(|disk| disk.beside.is_some());
    let other: Vec<&ForAloOs> = offer
        .disks_for_alo_os
        .iter()
        .filter(|disk| disk.beside.is_none())
        .collect();
    match (same, other.is_empty()) {
        (Some(same), false) => Places::Both { same, other },
        (Some(same), true) => Places::OnlyTheSameDisk(same),
        (None, false) => Places::OnlyOtherDisks(other),
        (None, true) => Places::None,
    }
}

/// Ask where alo OS goes, and take the answer.
///
/// **The empty disk is what an answer nobody understands means**, every time
/// and after the last time. Asked `ASKED_AGAIN` times — a private item of
/// `crate::asking`, so this names it rather than linking to it — because a
/// question asked for ever is a computer a person cannot get out of.
///
/// `named` is the empty disk this names in the question — the first of them,
/// which is the one a person is shown. Which empty disk alo OS goes on, where
/// there is more than one, is the consent that follows and types a name.
pub fn where_alo_os_goes(machine: &mut impl TheMachine, strings: &Strings, named: &str) -> Place {
    let answers = Filling::of(
        "same",
        strings
            .say(&words::ANSWER_THE_SAME_DISK.key(), &Filling::nothing())
            .into_text(),
    )
    .and(
        "other",
        strings
            .say(&words::ANSWER_THE_OTHER_DISK.key(), &Filling::nothing())
            .into_text(),
    );
    for _ in 0..ASKED_AGAIN {
        say(
            machine,
            strings,
            words::ASK_WHERE_ALO_OS_GOES,
            &Filling::of("disk", named),
        );
        let typed = machine.ask(&strings.say(&words::TYPE_SAME_OR_OTHER.key(), &answers));
        if is_the_word(&typed, words::ANSWER_THE_SAME_DISK, strings) {
            return Place::TheSameDisk;
        }
        if is_the_word(&typed, words::ANSWER_THE_OTHER_DISK, strings) {
            return Place::TheOtherDisk;
        }
    }
    Place::TheOtherDisk
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming what is wrong is the failure being reported"
)]
mod tests {
    use alo_installing::DiskName;

    use super::*;
    use crate::bitlocker::BitLocker;
    use crate::deciding::decide;
    use crate::disks::Disks;
    use crate::ended::Refusal;
    use crate::fast_startup::FastStartup;
    use crate::found::Found;
    use crate::identities::{DiskNumber, PartitionNumber};
    use crate::machine::tests::Answering;
    use crate::security_chip::SecurityChip;
    use crate::starting::Starting;
    use crate::windows_volume::{BesideWindows, WindowsVolume};

    /// **The machine this road was built for**: the testing NUC, one disk with
    /// Windows on it.
    const ONE_DISK: &str = r#"[
      {"Number":0,"FriendlyName":"CT120BX500SSD1","SerialNumber":"1838E15788A1",
       "BusType":"SATA","UniqueId":"","Size":120034123776,"PartitionStyle":"GPT",
       "IsReadOnly":false,
       "Partitions":[{"PartitionNumber":1,"GptType":"{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}","Label":""},
                     {"PartitionNumber":3,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Windows"}]}
    ]"#;

    /// What Windows prints for that volume.
    ///
    /// **Which of these numbers are measurements, and which are not.** This
    /// matters because an earlier version of this fixture had a size chosen to
    /// be *like* the NUC's, and figures computed from it were then reported as
    /// that machine's own. A fixture written to resemble a machine reads, in
    /// the hands of whoever wrote it, as the machine.
    ///
    /// - `Size` and `Offset` are **measured**: that volume's own bytes, read on
    ///   the machine 2026-10-08.
    /// - `SizeRemaining` is the **floor of what the machine displayed** on the
    ///   walk of 2026-10-07 — it said *69 GB free of 110 GB*, and free space is
    ///   rounded down, so 69 GiB is the least it can have been.
    /// - `SizeMin` is **measured, at a moment**: the machine's own
    ///   `Get-PartitionSupportedSize` at 2026-10-07T15:49:53Z, the most recent
    ///   of five elevated readings. **It moves.** Those five span 42,804,543,488
    ///   to 46,402,490,368 — 3.35 GiB across 36 hours — so this is not a
    ///   property of the volume and nothing here may treat it as one. It is far
    ///   below the size this road shrinks Windows to, which is why the road is
    ///   open on this machine at every one of those five moments.
    const THE_VOLUME: &str = r#"{"DriveLetter":"C","DiskNumber":0,"PartitionNumber":3,
      "Offset":227540992,"Size":118873915392,"SizeMin":44640423936,
      "SizeRemaining":74088902656}"#;

    /// The same disk, with no partition the firmware starts from.
    ///
    /// Windows reports no label for a start-up area at all, so this differs
    /// from the one above only in the GPT type of its first partition — which
    /// is the one thing a Windows program may ask about it and get a reliable
    /// answer.
    const NO_START_UP_AREA: &str = r#"[
      {"Number":0,"FriendlyName":"CT120BX500SSD1","SerialNumber":"1838E15788A1",
       "BusType":"SATA","UniqueId":"","Size":120034123776,"PartitionStyle":"GPT",
       "IsReadOnly":false,
       "Partitions":[{"PartitionNumber":1,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Recovery"},
                     {"PartitionNumber":3,"GptType":"{ebd0a0a2-b9e5-4433-87c0-68b6b72699c7}","Label":"Windows"}]}
    ]"#;

    /// The same volume with no room to give: free space below what this road
    /// asks, which is the area, the least disk and what Windows keeps free.
    const THE_FULL_VOLUME: &str = r#"{"DriveLetter":"C","DiskNumber":0,"PartitionNumber":3,
      "Offset":227540992,"Size":118873915392,"SizeMin":44640423936,
      "SizeRemaining":2147483648}"#;

    /// A machine that was read, with these disks and the volume above.
    fn found(disks: &str) -> Found {
        found_with(disks, THE_VOLUME)
    }

    /// The same, for a volume that is not the usual one.
    fn found_with(disks: &str, volume: &str) -> Found {
        Found {
            starting: Starting {
                uefi: Some(true),
                secure_boot: Some(false),
            },
            chip: SecurityChip::Ready,
            bitlocker: BitLocker::Off,
            memory: Some(16 * 1024 * 1024 * 1024),
            windows: WindowsVolume::read(Some(volume)),
            disks: Disks::read(Some(disks)),
            an_entry_is_named_alo_os: Some(false),
            fast_startup: FastStartup::Off,
        }
    }

    /// This installer's own words.
    fn source() -> Strings {
        Strings::of(words::installer_words().unwrap())
    }

    /// A disk alo OS may go on, with or without the shrink that keeps Windows.
    fn a_disk(number: u32, shown: &str, beside: Option<BesideWindows>) -> ForAloOs {
        ForAloOs {
            number: DiskNumber(number),
            shown: shown.to_owned(),
            after_the_restart: DiskName::named("ata-CT120BX500SSD1_1838E15788A1").unwrap(),
            beside,
        }
    }

    /// An offer carrying these disks and nothing else that matters here.
    fn offering(disks_for_alo_os: Vec<ForAloOs>) -> Offer {
        let windows = WindowsVolume::read(Some(THE_VOLUME)).unwrap();
        Offer {
            shrink: windows.shrink().unwrap(),
            beside_windows: windows.beside_windows().ok(),
            the_start_up_area: None,
            windows,
            windows_disk: "CT120BX500SSD1".to_owned(),
            disks_for_alo_os,
            the_windows_disk: None,
        }
    }

    /// The shrink that keeps Windows, from the machine above.
    fn the_shrink() -> BesideWindows {
        WindowsVolume::read(Some(THE_VOLUME))
            .unwrap()
            .beside_windows()
            .unwrap()
    }

    /// **The word, and only the word, puts alo OS on the disk Windows is on.**
    #[test]
    fn the_word_puts_alo_os_on_the_same_disk() {
        for typed in ["same", "Same", "  SAME  "] {
            let mut machine = Answering::of(&[typed]);
            assert_eq!(
                where_alo_os_goes(&mut machine, &source(), "Samsung SSD 870 EVO"),
                Place::TheSameDisk,
                "{typed:?}"
            );
            assert_eq!(machine.asked, 1, "it should not ask twice after an answer");
        }
    }

    /// **The other word gives alo OS an empty disk**, read as forgivingly.
    #[test]
    fn the_other_word_gives_alo_os_an_empty_disk() {
        for typed in ["other", "Other", " OTHER "] {
            let mut machine = Answering::of(&[typed]);
            assert_eq!(
                where_alo_os_goes(&mut machine, &source(), "Samsung SSD 870 EVO"),
                Place::TheOtherDisk,
                "{typed:?}"
            );
            assert_eq!(machine.asked, 1);
        }
    }

    /// **Nothing else reaches the disk Windows is on**, and running out of
    /// attempts gives alo OS the empty disk rather than falling through.
    ///
    /// The console that has gone away is in here on purpose: `ask` answers with
    /// an empty line when it cannot read, and an installer that read silence as
    /// *the same disk* would repartition the disk somebody's Windows is on with
    /// nobody sitting at it.
    #[test]
    fn nothing_else_reaches_the_disk_windows_is_on() {
        for typed in ["", "   ", "sam", "s", "yes", "no", "disk", "the same disk"] {
            let mut machine = Answering::of(&[typed, typed, typed]);
            assert_eq!(
                where_alo_os_goes(&mut machine, &source(), "Samsung SSD 870 EVO"),
                Place::TheOtherDisk,
                "{typed:?}"
            );
            assert_eq!(
                machine.asked, 3,
                "it should ask again before giving up, and then stop: {typed:?}"
            );
        }
    }

    /// **A question with one answer is not asked**, and which places exist is
    /// read from the offer rather than from a setting.
    #[test]
    fn a_question_with_one_answer_is_not_asked() {
        let same = a_disk(0, "CT120BX500SSD1", Some(the_shrink()));
        let other = a_disk(1, "Samsung SSD 870 EVO", None);

        assert_eq!(
            places(&offering(vec![same.clone()])),
            Places::OnlyTheSameDisk(&a_disk(0, "CT120BX500SSD1", Some(the_shrink())))
        );
        assert!(matches!(
            places(&offering(vec![other.clone()])),
            Places::OnlyOtherDisks(_)
        ));
        assert!(matches!(
            places(&offering(vec![same, other])),
            Places::Both { .. }
        ));
        assert_eq!(places(&offering(Vec::new())), Places::None);
    }

    /// **What this road would do to the testing NUC, from that machine's own
    /// bytes.**
    ///
    /// Not a restatement of the arithmetic - `windows_volume`'s own tests check
    /// the shrink against itself. This checks the **numbers a person on that
    /// machine will be shown**, because those are what the walk compares
    /// against, and because figures for this machine were once reported from a
    /// fixture rather than from it.
    ///
    /// The three sentences a person reads must add up:
    /// `installer.will.give-alo-os`'s own documentation says so, and a person
    /// who adds the area and alo OS's share should get the amount Windows gave
    /// up.
    #[test]
    fn what_this_road_does_to_the_testing_nuc() {
        let windows = WindowsVolume::read(Some(THE_VOLUME)).unwrap();
        let beside = windows.beside_windows().unwrap();

        assert_eq!(windows.size, 118_873_915_392, "not that machine's volume");
        assert_eq!(
            beside.to, 92_030_369_792,
            "Windows is not left where it was"
        );
        assert_eq!(beside.area_begins, 92_257_910_784);
        assert_eq!(beside.alo_os_begins, 93_331_652_608);
        assert_eq!(beside.alo_os, 24 * crate::sizes::GIB);

        // **The road is open at every moment `SizeMin` was measured.**
        // Windows will not shrink below what it reports it can, and what it
        // reports moved by 3.35 GiB across 36 hours on this machine - so a
        // road that is open against one reading and shut against another is
        // a road whose availability depends on when the person ran the
        // installer. These are the five the machine read, elevated.
        for smallest in [
            45_234_069_504_u64,
            46_402_490_368,
            42_804_543_488,
            42_812_432_384,
            44_640_423_936,
        ] {
            assert!(
                beside.to > smallest,
                "this road shrinks Windows to {} , below the {smallest} it reported it \
                 could at one of the five moments it was read",
                beside.to
            );
        }

        // **And the sentences add up.** What Windows gives up is the area plus
        // alo OS's share, and each is said as a whole number of gigabytes, so
        // the addition has to hold after rounding and not only before it.
        let gives_up = windows.size - beside.to;
        assert_eq!(gives_up, crate::sizes::THE_AREA + beside.alo_os);
        assert_eq!(crate::sizes::taken(gives_up), "25");
        assert_eq!(crate::sizes::taken(crate::sizes::THE_AREA), "1");
        assert_eq!(crate::sizes::taken(beside.alo_os), "24");
    }

    /// **A computer with one disk is offered the road, and asked nothing.**
    ///
    /// This replaces `nothing_offers_the_same_disk_road_yet`, which asserted the
    /// opposite and failed on 2026-10-08 when the installer plan's task 6
    /// opened the road — which is what it was written to do. Its message said
    /// the question becomes reachable and what then needs a test is what a
    /// person is actually shown, so that is what this checks.
    ///
    /// The testing NUC's own shape: one disk, Windows on it, nowhere else for
    /// alo OS to go. Until task 6 that machine was refused `NoDiskForAloOs`
    /// and told *this installer puts alo OS on a disk of its own*.
    #[test]
    fn a_one_disk_computer_is_offered_the_road_and_asked_nothing() {
        let offer = decide(&found(ONE_DISK)).expect("a one-disk computer is offered a road");

        // Exactly one place, and it is the disk Windows is on, carrying the
        // shrink that keeps Windows.
        let [only] = offer.disks_for_alo_os.as_slice() else {
            panic!(
                "one disk should offer one place: {:?}",
                offer.disks_for_alo_os
            );
        };
        assert_eq!(only.number, DiskNumber(0), "not the disk Windows is on");
        assert!(
            only.beside.is_some(),
            "the road offered does not keep Windows"
        );

        // **And nothing is asked.** There is one place, so a question would be
        // an invitation to type an answer that gets refused.
        assert_eq!(places(&offer), Places::OnlyTheSameDisk(only));

        // The start-up area is named, which is what the boot environment is
        // told and what the road is gated on.
        assert_eq!(offer.the_start_up_area, Some(PartitionNumber(1)));
    }

    /// **A disk the firmware does not start from is no beside road**, and on a
    /// one-disk computer that is still a refusal.
    ///
    /// alo OS's loader goes into a directory of its own inside Windows' own
    /// start-up area, so a disk with none is a machine that does not start the
    /// way this installer believes it does. Task 6 opened the road **where it
    /// can be walked** and this is the half of that sentence a test has to
    /// hold: the refusal did not become a guess.
    #[test]
    fn a_disk_with_no_start_up_area_is_still_refused() {
        let offer = decide(&found(NO_START_UP_AREA));
        assert!(
            matches!(offer, Err(Refusal::NoDiskForAloOs)),
            "a computer whose firmware partition cannot be named was offered a road: {offer:?}"
        );
    }

    /// **A volume that cannot give up the space is still refused, and told how
    /// much.**
    ///
    /// The other half of *where it can be walked*. `NotEnoughSpace` carries
    /// what this road asks rather than what the old one did, so a person short
    /// of room is told the number for the road they cannot walk.
    #[test]
    fn a_volume_with_no_room_is_still_refused_and_told_how_much() {
        let offer = decide(&found_with(ONE_DISK, THE_FULL_VOLUME));
        let Err(Refusal::NotEnoughSpace { needed, free, .. }) = offer else {
            panic!("a full volume was offered a road: {offer:?}");
        };
        assert!(
            needed > free,
            "it was refused for want of space while having enough: needed {needed}, free {free}"
        );
    }
}
