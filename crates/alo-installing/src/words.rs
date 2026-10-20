//! Every sentence the boot environment says, and the English beside each one.
//!
//! The person reading these has no other window. The machine restarted out of
//! the Windows they know into a black screen with text on it, and for the next
//! quarter of an hour this list is the whole of what alo OS says to them — so
//! every step is said as it begins, every refusal says that nothing was changed
//! (or, once the disk has been written, exactly what was), and nothing is said
//! twice in different words.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the tool that writes the disk, not the one that checks the download, not
//! an image, a registry, a signature or a key (`docs/features.md`: *a person
//! never learns the name of anything we rented*). The installer plan says the
//! same about the program that runs before this one: *no screen, prompt or
//! sentence names a key, a password or a signature* — a failed check is *this
//! download is not a genuine alo OS, so nothing was changed*, and that sentence
//! is [`NOT_GENUINE`] here word for word. A test at the bottom of this file
//! reads every sentence for the forbidden names.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The steps, in the order they are said.
// ---------------------------------------------------------------------------

/// The first line on the screen.
pub const STARTING: Word = Word::saying(
    "installing.starting",
    "alo OS is being installed on this computer. Each step is written here as it happens",
)
.noting(
    "The first line shown after the computer restarts into the installer. The screen has nothing \
     else on it and the person has no other window, so the sentence promises that every step \
     will be written on this screen.",
);

/// Reading which disk the person chose before the restart.
pub const READING_THE_CHOICE: Word = Word::saying(
    "installing.reading-the-choice",
    "Reading which disk you chose before the restart",
)
.noting(
    "The person chose a disk in the installer on their previous system, before restarting. This \
     step reads that choice back; nothing is being chosen now.",
);

/// Looking for that disk.
pub const LOOKING_FOR_THE_DISK: Word = Word::saying(
    "installing.looking-for-the-disk",
    "Looking for the disk you chose: {disk}",
)
.noting(
    "{disk} is the disk's own name as the computer reports it — usually its maker, model and \
     serial number joined together, for example Samsung_SSD_980_1TB_S64. It is not translated.",
);

/// Checking that the disk may be written.
pub const CHECKING_THE_DISK: Word = Word::saying(
    "installing.checking-the-disk",
    "Checking that {disk} is safe to install onto",
)
.noting(
    "{disk} is the disk's own name, not translated. Safe means: it is a whole disk, it is not in \
     use, it does not hold this installer, and it does not hold another operating system.",
);

/// Waiting for the network.
pub const CONNECTING: Word = Word::saying(
    "installing.connecting",
    "Connecting to the internet",
)
.noting(
    "Said while the computer waits for its wired network connection to come up, which can take a \
     few seconds after it starts. Nothing has been downloaded yet.",
);

/// Checking the download is genuine.
pub const CHECKING_IT_IS_GENUINE: Word = Word::saying(
    "installing.checking-it-is-genuine",
    "Checking over the internet that this download is a genuine alo OS",
)
.noting(
    "Said before anything is written to any disk. Genuine means it was published by the makers \
     of alo OS and has not been changed since. Do not use a word for a signature, a certificate \
     or a key.",
);

/// The download is genuine.
pub const GENUINE: Word = Word::saying("installing.genuine", "This is a genuine alo OS")
    .noting("Said once the check above has passed, immediately before the disk is written.");

/// Writing the disk.
pub const INSTALLING: Word = Word::saying(
    "installing.installing",
    "Installing alo OS onto {disk}. Everything that was on that disk is being replaced. This \
     takes a while, and this screen will say when it is done",
)
.noting(
    "{disk} is the disk's own name, not translated. The person already agreed, before the \
     restart, that this disk is replaced; the sentence says so again because from this moment it \
     is true.",
);

/// Still writing the disk.
pub const STILL_INSTALLING: Word = Word::saying(
    "installing.still-installing",
    "Still installing alo OS. Leave the computer on",
)
.noting(
    "Repeated about once a minute while the disk is being written, so that a screen that has not \
     changed for a long time is not mistaken for a computer that has stopped. Turning the \
     computer off now would leave the disk half written.",
);

/// Done.
pub const INSTALLED: Word = Word::saying(
    "installing.installed",
    "alo OS is installed. This computer restarts in a few seconds",
)
.noting("The last line of a successful installation. The restart happens by itself.");

// ---------------------------------------------------------------------------
// Putting right what the install leaves behind.
// ---------------------------------------------------------------------------

/// Tidying up, said before it begins.
pub const TIDYING: Word = Word::saying(
    "installing.tidying",
    "Tidying up: naming alo OS in the start-up menu, putting Windows behind it, and taking the      installer's own space back",
)
.noting(
    "Said after alo OS is installed and before the computer restarts. The start-up menu is the      computer's own, which it shows before either system starts.",
);

/// Tidying up finished.
pub const TIDIED: Word = Word::saying(
    "installing.tidied",
    "This computer now starts alo OS, with Windows behind it, and the installer's space is back",
)
.noting("Said when everything above was done.");

/// Some of it could not be done.
pub const TIDY_NOT_WHOLE: Word = Word::saying(
    "installing.tidy-not-whole",
    "alo OS is installed and starts. Some tidying up could not be finished, and you can do it      from alo OS once it has started",
)
.noting(
    "Said when one of those three could not be done. alo OS is installed either way, and the      sentence says so first.",
);

/// The firmware's list could not be read.
pub const TIDY_ENTRIES_NOT_READ: Word = Word::saying(
    "installing.tidy-entries-not-read",
    "The list of systems this computer can start could not be read, so it was left as it is",
)
.noting("Said when the firmware's own tool did not answer, or answered with nothing to act on.");

// ---------------------------------------------------------------------------
// The refusals. Every one of them before the disk is written says that nothing
// was changed, because that is the first thing a person needs to know.
// ---------------------------------------------------------------------------

/// The installer on this computer is damaged.
pub const DAMAGED: Word = Word::saying(
    "installing.damaged",
    "This installer is damaged, so nothing was changed. Download alo OS again",
)
.noting(
    "Said when the installer's own files on this computer are missing or unreadable — not the \
     download of alo OS itself, but the small program that fetches it.",
);

/// No disk was chosen.
pub const NO_DISK_CHOSEN: Word = Word::saying(
    "installing.no-disk-chosen",
    "No disk was chosen for alo OS, so nothing was changed",
)
.noting(
    "Said when the computer restarted into the installer without a disk having been chosen \
     before the restart. The installer never picks a disk by itself.",
);

/// The choice could not be understood.
pub const CHOICE_NOT_UNDERSTOOD: Word = Word::saying(
    "installing.choice-not-understood",
    "The disk chosen for alo OS could not be understood, so nothing was changed",
)
.noting(
    "Said when the record of which disk was chosen is malformed, or names more than one disk. The \
     installer never guesses which one was meant.",
);

/// The disk is not there.
pub const DISK_NOT_CONNECTED: Word = Word::saying(
    "installing.disk-not-connected",
    "The disk you chose, {disk}, is not connected to this computer, so nothing was changed",
)
.noting("{disk} is the disk's own name, not translated.");

/// The disks could not be read.
pub const DISKS_NOT_READ: Word = Word::saying(
    "installing.disks-not-read",
    "The disks in this computer could not be read, so nothing was changed",
)
.noting(
    "Said when the installer cannot find out what is on the disks. Without knowing that it does \
     not write to any of them.",
);

/// Part of a disk was named, not a whole disk.
pub const NOT_A_WHOLE_DISK: Word = Word::saying(
    "installing.not-a-whole-disk",
    "{disk} is part of a disk rather than a whole disk, so nothing was changed",
)
.noting(
    "{disk} is the name of what was chosen, not translated. This installer replaces whole disks \
     only.",
);

/// The disk holds this installer.
pub const HOLDS_THIS_INSTALLER: Word = Word::saying(
    "installing.holds-this-installer",
    "The disk you chose, {disk}, holds this installer itself, so nothing was changed",
)
.noting(
    "{disk} is the disk's own name, not translated. The installer runs from a small area of that \
     disk and refuses to install over itself.",
);

/// The disk holds another operating system.
pub const HOLDS_ANOTHER_SYSTEM: Word = Word::saying(
    "installing.holds-another-system",
    "The disk you chose, {disk}, holds another operating system, so nothing was changed",
)
.noting(
    "{disk} is the disk's own name, not translated. Said when the disk has areas that Windows \
     makes or uses. Replacing another operating system is a separate choice that asks twice, and \
     this installer does not make it.",
);

/// The disk cannot be written.
pub const CANNOT_BE_WRITTEN: Word = Word::saying(
    "installing.cannot-be-written",
    "The disk you chose, {disk}, is in use or cannot be written to, so nothing was changed",
)
.noting("{disk} is the disk's own name, not translated.");

/// There is no network card at all.
pub const NO_NETWORK_CARD: Word = Word::saying(
    "installing.no-network-card",
    "This computer's network card was not found, so nothing was changed and alo OS was not \
     installed. Windows is still there and starts as it did before. This needs a computer \
     whose network card this version of alo OS knows",
)
.noting(
    "Said when the machine has no network card the system can see at all — so it deliberately \
     does not mention a cable, because there is nowhere to plug one in. alo OS and Windows are \
     product names and are not translated.",
);

/// There is a network card and nothing on the wire.
pub const NOTHING_CONNECTED: Word = Word::saying(
    "installing.nothing-connected",
    "No network cable is plugged in, so nothing was changed and alo OS was not installed. \
     Plug in a network cable, then restart to try again",
)
.noting(
    "Said when the computer has a network card and nothing is connected to it. \"A network \
     cable\" is wired ethernet. alo OS is the product's name and is not translated.",
);

/// The internet could not be reached.
///
/// **Kept for the case it was always right about**: a live network card and
/// the download still did not arrive. [`NO_NETWORK_CARD`] and
/// [`NOTHING_CONNECTED`] took the two cases this sentence could not be acted
/// on for.
pub const NOT_REACHABLE: Word = Word::saying(
    "installing.not-reachable",
    "alo OS could not be downloaded, so nothing was changed. Check that this computer is \
     connected to the internet by a network cable, then restart to try again",
)
.noting(
    "Said when the download could not be reached at all — no network, or the connection broke. \
     It is deliberately different from the sentence about a download that is not genuine. The \
     installer connects by cable only, so the sentence names a cable and nothing else.",
);

/// The download is not genuine.
pub const NOT_GENUINE: Word = Word::saying(
    "installing.not-genuine",
    "This download is not a genuine alo OS, so nothing was changed",
)
.noting(
    "Said when the download was reached and could not be shown to come from the makers of alo OS \
     unchanged. There is no way past it and the sentence offers none. Do not use a word for a \
     signature, a certificate or a key.",
);

/// Writing the disk did not finish.
pub const NOT_INSTALLED: Word = Word::saying(
    "installing.not-installed",
    "alo OS could not be installed onto {disk}. That disk may now hold part of alo OS; nothing \
     else on this computer was changed. Restart to try again",
)
.noting(
    "{disk} is the disk's own name, not translated. Said only after writing began, which is why \
     it does not say that nothing was changed: the chosen disk was, and nothing else was.",
);

/// What to do after any refusal.
pub const RESTART_WHEN_READY: Word = Word::saying(
    "installing.restart-when-ready",
    "You can turn this computer off or restart it now",
)
.noting(
    "The last line after the installer has stopped without installing. The installer does not \
     restart by itself in that case, so the person has time to read why.",
);

/// The road that replaces the system already on the disk.
pub const REPLACING_WHAT_IS_THERE: Word = Word::saying(
    "installing.replacing-what-is-there",
    "Everything on {disk} is being replaced, including the system that was on it. This is the \
     road you agreed to twice, and there is no way back from here",
)
.noting(
    "Said only on the road that replaces Windows, after the disk has been read and before \
     anything is written. {disk} is the disk's own name.",
);

/// On the road that keeps what is there: the space for alo OS is not there.
pub const NOT_THE_SPACE_THE_INSTALLER_MADE: Word = Word::saying(
    "installing.not-the-space-the-installer-made",
    "The space for alo OS on the disk {disk} is not there, so nothing was changed. Start Windows \
     and run the alo OS installer again",
)
.noting(
    "{disk} is the disk's own name, not translated. Said only on the road that keeps the system \
     already on the disk, when the space that installer made for alo OS is missing, is not the \
     space it made, or is being used by something else. The person is told one thing, because \
     there is one thing to do about all of them. 'Space' is the word for a part of a disk \
     throughout these sentences; do not use a word for a partition or a volume.",
);

// ---------------------------------------------------------------------------
// Asking which disk. The person chose nothing before the restart, because
// there was no Windows to choose it on — see `crate::which_disks_may_be_offered`.
// ---------------------------------------------------------------------------

/// What a person reads a disk as, when the machine says both.
pub const A_DISK_AND_ITS_SIZE: Word =
    Word::saying("installing.a-disk-and-its-size", "{disk}  ·  {size}").noting(
        "{disk} is what the maker calls the disk — Samsung SSD 870 EVO, WD Elements — and is not \
         translated. {size} is already written out, for example 1 TB, and is not translated \
         either; the unit is the one the disk was sold in. The separator is a middle dot with \
         two spaces each side. Translate nothing here; the line exists so the two parts are \
         joined the same way in every language.",
    );

/// A disk with nothing on it.
pub const NOTHING_IS_ON_IT: Word = Word::saying(
    "installing.nothing-is-on-it",
    "Empty. Nothing would be lost",
)
.noting(
    "Said under a disk a person may choose, when the machine found no filesystem on it at all. \
     Two short sentences on purpose: the first is what the disk is, the second is what choosing \
     it costs them. Do not merge them into a clause.",
);

/// A disk holding another system, and how much of it is in use.
pub const ANOTHER_SYSTEM_AND_FILES: Word = Word::saying(
    "installing.another-system-and-files",
    "Holds another operating system and about {files} of files",
)
.noting(
    "Said under a disk a person may choose, when choosing it would erase what is on it. {files} \
     is already written out, for example 340 GB, and is not translated. 'About' matters and must \
     be kept: the number is what the machine reports in use and is not exact to the byte. Do not \
     name the other system — this environment cannot tell which one it is, and a wrong name here \
     is read as certainty.",
);

/// A disk holding something, where how much could not be measured.
pub const SOMETHING_IS_ON_IT: Word = Word::saying(
    "installing.something-is-on-it",
    "Holds another operating system. Everything on this disk would be lost",
)
.noting(
    "Said instead of the sentence above when the machine could not measure how much is in use. \
     It says less rather than guessing a number. Keep 'everything', because the person is being \
     told the whole disk goes.",
);

/// The disk this installer is running from.
pub const THAT_IS_THIS_INSTALLER: Word = Word::saying(
    "installing.that-is-this-installer",
    "This is the disk alo OS is being installed from, so it cannot be installed onto",
)
.noting(
    "Said beside a disk a person is not offered. They are most likely looking at the USB stick \
     they started this computer from. The sentence says why rather than hiding the disk, because \
     a disk missing from a list teaches nobody anything.",
);

/// A disk something else is using.
pub const THAT_DISK_IS_IN_USE: Word = Word::saying(
    "installing.that-disk-is-in-use",
    "Something on this computer is using this disk, so it cannot be written",
)
.noting(
    "Said beside a disk a person is not offered: it is read-only, or something on it is open. \
     Do not name the thing using it — this environment does not know what it is.",
);

/// No labelled area anywhere, so which disk this installer is on is unknown.
pub const CANNOT_TELL_WHICH_DISK_THIS_IS: Word = Word::saying(
    "installing.cannot-tell-which-disk-this-is",
    "This installer cannot tell which disk it is running from, so it will not erase anything",
)
.noting(
    "Said beside a disk holding another system, when the medium alo OS is being installed from \
     carries no mark identifying it. Erasing could destroy the installer itself half way through \
     installing, so only disks with nothing on them are offered. The sentence is a boundary and \
     not a fault: nothing is wrong with the person's computer.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 38] = [
    A_DISK_AND_ITS_SIZE,
    NOTHING_IS_ON_IT,
    ANOTHER_SYSTEM_AND_FILES,
    SOMETHING_IS_ON_IT,
    THAT_IS_THIS_INSTALLER,
    THAT_DISK_IS_IN_USE,
    CANNOT_TELL_WHICH_DISK_THIS_IS,
    NO_NETWORK_CARD,
    NOTHING_CONNECTED,
    STARTING,
    REPLACING_WHAT_IS_THERE,
    TIDYING,
    TIDIED,
    TIDY_NOT_WHOLE,
    TIDY_ENTRIES_NOT_READ,
    READING_THE_CHOICE,
    LOOKING_FOR_THE_DISK,
    CHECKING_THE_DISK,
    CONNECTING,
    CHECKING_IT_IS_GENUINE,
    GENUINE,
    INSTALLING,
    STILL_INSTALLING,
    INSTALLED,
    DAMAGED,
    NO_DISK_CHOSEN,
    CHOICE_NOT_UNDERSTOOD,
    DISK_NOT_CONNECTED,
    DISKS_NOT_READ,
    NOT_A_WHOLE_DISK,
    HOLDS_THIS_INSTALLER,
    HOLDS_ANOTHER_SYSTEM,
    CANNOT_BE_WRITTEN,
    NOT_REACHABLE,
    NOT_GENUINE,
    NOT_THE_SPACE_THE_INSTALLER_MADE,
    NOT_INSTALLED,
    RESTART_WHEN_READY,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so — and [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn installing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "installing", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = installing_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and the only gap anywhere is the disk.**
    #[test]
    fn every_word_carries_a_note_and_its_only_gap_is_the_disk() {
        // **Three gaps, named, and nothing else.** It was one until 2026-10-20,
        // when the environment gained the question *which disk?* — asking it
        // means naming a disk the way a person bought it and saying what is on
        // it, and neither fits in `{disk}`, which is a device's own name.
        //
        // The list is written out rather than relaxed to *any gap*: a gap is
        // the one thing in a sentence a translator cannot see the meaning of,
        // so each one costs them a note, and a fourth appearing quietly is a
        // translator guessing. Adding one is a line here and a sentence in its
        // note saying what fills it.
        const THE_GAPS: [&str; 3] = [
            // A device's own name, as the machine reports it. Never translated.
            "disk", // How large a disk is, already written out: *1 TB*.
            "size", // How much of a disk is in use, already written out: *340 GB*.
            "files",
        ];
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(
                    THE_GAPS.contains(&gap.as_str()),
                    "{} has the gap {{{gap}}}, which is not one of {THE_GAPS:?} — add it there \
                     with what fills it, or word the sentence without it",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here names the machinery, a key, or a signature.**
    #[test]
    fn nothing_here_names_the_machinery_a_key_or_a_signature() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "bootc",
                "ostree",
                "cosign",
                "podman",
                "container",
                "image",
                "registry",
                "digest",
                "sha256",
                "pull",
                "signature",
                "signed",
                "key",
                "password",
                "certificate",
                "partition",
                "firmware",
                "uefi",
                "boot environment",
                "initramfs",
                "kernel",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
        }
    }

    /// **Every refusal before the disk is written says nothing was changed**,
    /// and the one after it does not claim that.
    #[test]
    fn every_refusal_before_writing_says_nothing_was_changed() {
        for word in [
            DAMAGED,
            NO_DISK_CHOSEN,
            CHOICE_NOT_UNDERSTOOD,
            DISK_NOT_CONNECTED,
            DISKS_NOT_READ,
            NOT_A_WHOLE_DISK,
            HOLDS_THIS_INSTALLER,
            HOLDS_ANOTHER_SYSTEM,
            CANNOT_BE_WRITTEN,
            NOT_REACHABLE,
            NOT_GENUINE,
        ] {
            assert!(
                word.says().contains("so nothing was changed"),
                "{}",
                word.named()
            );
        }
        assert!(!NOT_INSTALLED.says().contains("so nothing was changed"));
    }

    /// The sentence a failed check is said in is the one the installer plan
    /// gives, word for word.
    #[test]
    fn a_download_that_is_not_genuine_is_said_in_the_plans_words() {
        assert_eq!(
            NOT_GENUINE.says(),
            "This download is not a genuine alo OS, so nothing was changed"
        );
    }
}
