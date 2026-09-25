//! Every sentence the installer says, and the English beside each one.
//!
//! The person reading these downloaded a program and started it on the only
//! computer they may have. Everything it found is said before anything is
//! asked, everything it will do is said before it is agreed to, and every
//! refusal says that nothing was changed — because it is true, and it is the
//! first thing a person needs to know.
//!
//! # Nothing a person reads names the machinery, a key or a signature
//!
//! The installer plan's task 3: *no screen, prompt or sentence names a key, a
//! password or a signature*, and a failed check is *this download is not a
//! genuine alo OS, so nothing was changed* — [`NOT_GENUINE`], word for word. Nor
//! do these sentences name the tools that do the work (`docs/features.md`: *a
//! person never learns the name of anything we rented*). A test at the bottom
//! reads every sentence for those names.
//!
//! # Secure Boot is refused, never argued with
//!
//! ADR 0033 §4: with Secure Boot on the installer *refuses to proceed and says
//! why; it never asks anybody to switch it off*. [`SECURE_BOOT_ON`] says why and
//! offers nothing else, and a test holds it to that.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Starting, and whether the download is genuine.
// ---------------------------------------------------------------------------

/// The first line.
pub const STARTING: Word = Word::saying(
    "installer.starting",
    "This is the alo OS installer. It checks this computer first, and changes nothing until you \
     agree",
)
.noting(
    "The first line a person reads after starting the program they downloaded. The promise that \
     nothing changes until they agree is kept by the program and is the point of the sentence.",
);

/// Checking the download.
pub const CHECKING_THE_DOWNLOAD: Word = Word::saying(
    "installer.checking-the-download",
    "Checking that this download is a genuine alo OS",
)
.noting(
    "Genuine means it was published by the makers of alo OS and has not been changed since. Do not \
     use a word for a signature, a certificate or a key.",
);

/// The download is genuine.
pub const GENUINE: Word = Word::saying("installer.genuine", "This download is a genuine alo OS")
    .noting("Said once the check above has passed.");

/// Checking the computer.
pub const CHECKING_THE_COMPUTER: Word = Word::saying(
    "installer.checking-the-computer",
    "Checking this computer. This only reads, and changes nothing",
)
.noting("Said before the list of what was found.");

// ---------------------------------------------------------------------------
// What was found. Each check is said, whatever it found.
// ---------------------------------------------------------------------------

/// UEFI.
pub const FOUND_UEFI: Word = Word::saying(
    "installer.found.uefi",
    "This computer starts with UEFI, which alo OS needs",
)
.noting("UEFI is the name of the modern way a computer starts; keep it as UEFI.");

/// Not UEFI.
pub const FOUND_NOT_UEFI: Word = Word::saying(
    "installer.found.not-uefi",
    "This computer starts in the older BIOS way",
)
.noting("BIOS is the name of the older way a computer starts; keep it as BIOS.");

/// How the computer starts could not be read.
pub const FOUND_STARTING_NOT_READ: Word = Word::saying(
    "installer.found.starting-not-read",
    "How this computer starts could not be found out",
)
.noting("Said when Windows did not answer whether the computer starts with UEFI or BIOS.");

/// Secure Boot on.
pub const FOUND_SECURE_BOOT_ON: Word = Word::saying(
    "installer.found.secure-boot-on",
    "Secure Boot is on",
)
.noting(
    "Secure Boot is the name of the setting as the computer's own menus show it; use the name those \
     menus use in this language, which is usually Secure Boot.",
);

/// Secure Boot off.
pub const FOUND_SECURE_BOOT_OFF: Word =
    Word::saying("installer.found.secure-boot-off", "Secure Boot is off")
        .noting("Secure Boot is the name of the setting as the computer's own menus show it.");

/// Secure Boot could not be read.
pub const FOUND_SECURE_BOOT_NOT_READ: Word = Word::saying(
    "installer.found.secure-boot-not-read",
    "Whether Secure Boot is on could not be found out",
)
.noting("Secure Boot is the name of the setting as the computer's own menus show it.");

/// A TPM that is ready.
pub const FOUND_TPM_READY: Word = Word::saying(
    "installer.found.tpm-ready",
    "This computer has a security chip (TPM), and it is ready",
)
.noting(
    "TPM is the chip's name as Windows shows it. Said for information; it does not change what the \
     installer does.",
);

/// A TPM that is not ready.
pub const FOUND_TPM_NOT_READY: Word = Word::saying(
    "installer.found.tpm-not-ready",
    "This computer has a security chip (TPM), and it is not ready for use",
)
.noting("TPM is the chip's name as Windows shows it. Said for information only.");

/// No TPM.
pub const FOUND_NO_TPM: Word = Word::saying(
    "installer.found.no-tpm",
    "This computer has no security chip (TPM)",
)
.noting("TPM is the chip's name as Windows shows it. Said for information only.");

/// The TPM could not be read.
pub const FOUND_TPM_NOT_READ: Word = Word::saying(
    "installer.found.tpm-not-read",
    "Whether this computer has a security chip (TPM) could not be found out",
)
.noting("TPM is the chip's name as Windows shows it. Said for information only.");

/// Windows is encrypted with BitLocker.
pub const FOUND_BITLOCKER_ON: Word = Word::saying(
    "installer.found.bitlocker-on",
    "Windows on {volume} is encrypted with BitLocker. It stays encrypted, and the installer does \
     not read what is on it",
)
.noting(
    "{volume} is the drive Windows is on, as Windows names it, for example C:. BitLocker is the \
     name of Windows' encryption as Windows shows it.",
);

/// Windows is not encrypted.
pub const FOUND_BITLOCKER_OFF: Word = Word::saying(
    "installer.found.bitlocker-off",
    "Windows on {volume} is not encrypted with BitLocker",
)
.noting("{volume} is the drive Windows is on, for example C:.");

/// BitLocker is encrypting or decrypting.
pub const FOUND_BITLOCKER_CHANGING: Word = Word::saying(
    "installer.found.bitlocker-changing",
    "BitLocker is encrypting or decrypting Windows on {volume} right now",
)
.noting("{volume} is the drive Windows is on, for example C:.");

/// BitLocker could not be read.
pub const FOUND_BITLOCKER_NOT_READ: Word = Word::saying(
    "installer.found.bitlocker-not-read",
    "Whether Windows on {volume} is encrypted with BitLocker could not be found out",
)
.noting("{volume} is the drive Windows is on, for example C:.");

/// Free space on the Windows volume.
pub const FOUND_SPACE: Word = Word::saying(
    "installer.found.space",
    "Windows on {volume} has {free} GB free of {size} GB",
)
.noting(
    "{volume} is the drive Windows is on, for example C:. {free} and {size} are whole numbers of \
     gigabytes.",
);

/// Memory.
pub const FOUND_MEMORY: Word = Word::saying(
    "installer.found.memory",
    "This computer has {memory} GB of memory",
)
.noting("{memory} is a whole number of gigabytes.");

/// Less memory than alo OS is made for.
pub const FOUND_LITTLE_MEMORY: Word = Word::saying(
    "installer.found.little-memory",
    "This computer has {memory} GB of memory. alo OS is made for 16 GB or more, and is slow with \
     less",
)
.noting(
    "{memory} is a whole number of gigabytes. Said for information; the installer still offers to \
     install.",
);

/// Memory could not be read.
pub const FOUND_MEMORY_NOT_READ: Word = Word::saying(
    "installer.found.memory-not-read",
    "How much memory this computer has could not be found out",
)
.noting("Said for information only.");

/// The disk Windows is on.
pub const FOUND_WINDOWS_DISK: Word = Word::saying(
    "installer.found.windows-disk",
    "Windows is on the disk {disk} ({size} GB)",
)
.noting(
    "{disk} is the disk's name as its maker gives it, not translated. {size} is a whole number of \
     gigabytes.",
);

/// A disk alo OS can be installed onto.
pub const FOUND_A_DISK_FOR_ALO_OS: Word = Word::saying(
    "installer.found.a-disk-for-alo-os",
    "The disk {disk} ({size} GB) is empty, and alo OS can be installed onto it",
)
.noting(
    "{disk} is the disk's name as its maker gives it, not translated. Empty means it holds no files \
     and no other system that Windows can see.",
);

/// A disk holding files or a system.
pub const FOUND_A_DISK_IN_USE: Word = Word::saying(
    "installer.found.a-disk-in-use",
    "The disk {disk} ({size} GB) already holds files or a system, so alo OS does not use it",
)
.noting("{disk} is the disk's name as its maker gives it, not translated.");

/// A disk too small.
pub const FOUND_A_DISK_TOO_SMALL: Word = Word::saying(
    "installer.found.a-disk-too-small",
    "The disk {disk} ({size} GB) is smaller than the {least} GB alo OS needs",
)
.noting("{disk} is the disk's name as its maker gives it, not translated.");

/// A disk that cannot be written or is not recognised.
pub const FOUND_A_DISK_NOT_USABLE: Word = Word::saying(
    "installer.found.a-disk-not-usable",
    "The disk {disk} ({size} GB) cannot be written to, or is connected in a way this installer \
     does not recognise yet, so alo OS does not use it",
)
.noting(
    "{disk} is the disk's name as its maker gives it, not translated. Said for a read-only disk, and \
     for a disk connected by USB or another way the installer cannot yet name reliably after a \
     restart.",
);

// ---------------------------------------------------------------------------
// What will happen, and the question.
// ---------------------------------------------------------------------------

/// Nothing changed yet.
pub const NOTHING_CHANGED_YET: Word = Word::saying(
    "installer.nothing-changed-yet",
    "Nothing on this computer has been changed yet. If you agree, this is what happens, in this \
     order:",
)
.noting("Said before the list of steps below, which follow it one per line.");

/// Shrinking Windows.
pub const WILL_SHRINK_WINDOWS: Word = Word::saying(
    "installer.will.shrink-windows",
    "Windows on {volume} is made {area} GB smaller. Your files and applications stay where they \
     are",
)
.noting(
    "{volume} is the drive Windows is on, for example C:. {area} is a whole number of gigabytes.",
);

/// Making the installer's area.
pub const WILL_MAKE_THE_AREA: Word = Word::saying(
    "installer.will.make-the-area",
    "A {area} GB area for the installer is made in that space, on the disk {disk}",
)
.noting(
    "{disk} is the disk Windows is on, by its maker's name, not translated. The area holds the small \
     program the computer restarts into.",
);

/// Adding the entry.
pub const WILL_ADD_THE_ENTRY: Word = Word::saying(
    "installer.will.add-the-entry",
    "An entry named alo OS is added to the systems this computer can start. Windows stays the one \
     it starts normally",
)
.noting("alo OS is the product's name and is not translated.");

/// Restarting into the installer.
pub const WILL_RESTART: Word = Word::saying(
    "installer.will.restart",
    "This computer restarts once into the installer, which replaces everything on the disk you \
     name below with alo OS, then restarts again",
)
.noting(
    "The restart happens by itself at the end of these steps. The disk named below is the one the \
     person types the name of.",
);

/// Fast Startup is on.
pub const FOUND_FAST_STARTUP_ON: Word = Word::saying(
    "installer.found.fast-startup-on",
    "Windows' Fast Startup is on",
)
.noting(
    "Fast Startup is the name of the setting as Windows' own settings show it; use the name \
         those settings use in this language.",
);

/// Fast Startup is off.
pub const FOUND_FAST_STARTUP_OFF: Word = Word::saying(
    "installer.found.fast-startup-off",
    "Windows' Fast Startup is off",
)
.noting("Fast Startup is the name of the setting as Windows' own settings show it.");

/// Fast Startup could not be read.
pub const FOUND_FAST_STARTUP_NOT_READ: Word = Word::saying(
    "installer.found.fast-startup-not-read",
    "Whether Windows' Fast Startup is on could not be found out",
)
.noting("Fast Startup is the name of the setting as Windows' own settings show it.");

/// The question about Fast Startup, asked only when it is on.
pub const ASK_FAST_STARTUP: Word = Word::saying(
    "installer.ask.fast-startup",
    "Windows' Fast Startup is on. It can make Windows and alo OS disagree about the disk. Turn it \
     off? (Recommended when sharing a disk.)",
)
.noting(
    "The owner's words (ADR 0064). Fast Startup is the name Windows' own settings use. The two \
     answers are the two short sentences below, and the person types one of them.",
);

/// How the two answers are typed.
pub const TYPE_ONE_OF_THESE_ANSWERS: Word = Word::saying(
    "installer.ask.type-one-of-these-answers",
    "Type {off} or {on}, and press Enter",
)
.noting(
    "The gaps hold the two answers below, in this language. A person types one of them; anything \
     else is not an answer and the question is asked again.",
);

/// The answer that turns Fast Startup off.
pub const ANSWER_TURN_OFF: Word = Word::saying("installer.answer.turn-off", "turn off").noting(
    "One of the two answers to the Fast Startup question, typed by the person. Keep it to two \
     words at most, and unlike the other answer.",
);

/// The answer that leaves Fast Startup on.
pub const ANSWER_LEAVE_ON: Word = Word::saying("installer.answer.leave-on", "leave on").noting(
    "The other answer to the Fast Startup question, typed by the person. Keep it to two words at \
     most.",
);

/// Fast Startup stays as it is.
pub const FAST_STARTUP_LEFT_ON: Word = Word::saying(
    "installer.fast-startup-left-on",
    "Windows' Fast Startup is being left on, and nothing about it is changed",
)
.noting(
    "Said when the person answers that it should be left on, and when nothing that is an answer \
     was typed.",
);

/// Turning Fast Startup off, said before it is done.
pub const TURNING_FAST_STARTUP_OFF: Word = Word::saying(
    "installer.turning-fast-startup-off",
    "Turning Windows' Fast Startup off. Windows keeps hibernation, and starts and shuts down as \
     before",
)
.noting(
    "Said before the setting is changed. Hibernation itself is not removed; only Fast Startup is \
     switched off.",
);

/// The question.
pub const TYPE_THE_DISKS_NAME: Word = Word::saying(
    "installer.type-the-disks-name",
    "To agree, type the name of the disk alo OS replaces, exactly as it is written above, and \
     press Enter. Everything on that disk is replaced. To stop, press Enter without typing \
     anything",
)
.noting(
    "The person agrees by typing a disk's name, never by pressing a button, because what is \
     replaced is a whole disk and the name says which one.",
);

// ---------------------------------------------------------------------------
// Preparing, once agreed.
// ---------------------------------------------------------------------------

/// Shrinking Windows.
pub const SHRINKING_WINDOWS: Word = Word::saying(
    "installer.shrinking-windows",
    "Making Windows on {volume} {area} GB smaller",
)
.noting("{volume} is the drive Windows is on, for example C:.");

/// Making the area.
pub const MAKING_THE_AREA: Word = Word::saying(
    "installer.making-the-area",
    "Making the installer's area on the disk {disk}",
)
.noting("{disk} is the disk's name as its maker gives it, not translated.");

/// Copying the installer.
pub const COPYING_THE_INSTALLER: Word = Word::saying(
    "installer.copying-the-installer",
    "Copying the installer into its area, and checking the copy",
)
.noting("The copy is read back and compared before the computer is told to start it.");

/// Adding the entry.
pub const ADDING_THE_ENTRY: Word = Word::saying(
    "installer.adding-the-entry",
    "Adding alo OS to the systems this computer can start",
)
.noting("alo OS is the product's name and is not translated.");

/// Putting things back.
pub const PUTTING_IT_BACK: Word = Word::saying(
    "installer.putting-it-back",
    "Something went wrong, so everything changed so far is being put back",
)
.noting("Said before the installer undoes the steps it has already taken, in reverse order.");

/// Restarting.
pub const RESTARTING: Word = Word::saying(
    "installer.restarting",
    "Everything is ready. This computer restarts in a few seconds, and installing continues after \
     the restart",
)
.noting("The last line before the restart, which happens by itself.");

/// The restart did not happen.
pub const RESTART_IT_YOURSELF: Word = Word::saying(
    "installer.restart-it-yourself",
    "Everything is ready, and this computer could not restart by itself. Restart it to continue \
     installing alo OS",
)
.noting("Said when everything was prepared and Windows did not carry out the restart.");

// ---------------------------------------------------------------------------
// The refusals. Every one of them says that nothing was changed.
// ---------------------------------------------------------------------------

/// Not started as administrator.
pub const NOT_AN_ADMINISTRATOR: Word = Word::saying(
    "installer.not-an-administrator",
    "This installer needs permission to change how this computer starts, so nothing was changed. \
     Start it again by choosing Run as administrator",
)
.noting(
    "Run as administrator is the name of the choice in Windows' own menu when a program is \
     right-clicked; use the name Windows uses in this language.",
);

/// The download is incomplete.
pub const INCOMPLETE: Word = Word::saying(
    "installer.incomplete",
    "This download is incomplete, so nothing was changed. Download alo OS again",
)
.noting(
    "Said when files that came with the installer are missing or cannot be read. It is deliberately \
     different from the sentence about a download that is not genuine.",
);

/// The download is not genuine.
pub const NOT_GENUINE: Word = Word::saying(
    "installer.not-genuine",
    "This download is not a genuine alo OS, so nothing was changed",
)
.noting(
    "Said when the download could not be shown to come from the makers of alo OS unchanged. There \
     is no way past it and the sentence offers none. Do not use a word for a signature, a \
     certificate or a key.",
);

/// Not UEFI.
pub const NOT_UEFI: Word = Word::saying(
    "installer.not-uefi",
    "alo OS needs a computer that starts with UEFI, and this one starts in the older BIOS way, so \
     nothing was changed",
)
.noting("UEFI and BIOS are names; keep them as they are.");

/// How the computer starts could not be read.
pub const STARTING_NOT_READ: Word = Word::saying(
    "installer.starting-not-read",
    "How this computer starts could not be found out, so nothing was changed",
)
.noting("The installer does not change a computer whose way of starting it could not confirm.");

/// Secure Boot is on.
pub const SECURE_BOOT_ON: Word = Word::saying(
    "installer.secure-boot-on",
    "Secure Boot is on, and the part of alo OS that starts a computer is not yet approved to start \
     with it, so nothing was changed",
)
.noting(
    "The reason, and nothing else. Do not add advice about the setting: alo OS never asks anybody \
     to change it. Secure Boot is the name of the setting as the computer's own menus show it.",
);

/// Secure Boot could not be read.
pub const SECURE_BOOT_NOT_READ: Word = Word::saying(
    "installer.secure-boot-not-read",
    "Whether Secure Boot is on could not be found out, so nothing was changed",
)
.noting(
    "The installer does not continue on a guess about Secure Boot. Do not add advice about the \
     setting.",
);

/// The disks could not be read.
pub const DISKS_NOT_READ: Word = Word::saying(
    "installer.disks-not-read",
    "The disks in this computer could not be read, so nothing was changed",
)
.noting("Without knowing what is on the disks, the installer does not change any of them.");

/// The list of systems could not be read.
pub const ENTRIES_NOT_READ: Word = Word::saying(
    "installer.entries-not-read",
    "The list of systems this computer can start could not be read, so nothing was changed",
)
.noting("Said when Windows did not answer which systems the computer can start.");

/// The Windows disk is laid out in a way not supported.
pub const WINDOWS_DISK_NOT_SUPPORTED: Word = Word::saying(
    "installer.windows-disk-not-supported",
    "The disk {disk} that Windows is on is laid out in an older way this installer does not \
     support, so nothing was changed",
)
.noting(
    "{disk} is the disk's name as its maker gives it, not translated. Said for a disk that is not \
     laid out in the modern way UEFI computers use.",
);

/// BitLocker is still changing the volume.
pub const BITLOCKER_CHANGING: Word = Word::saying(
    "installer.bitlocker-changing",
    "BitLocker is still encrypting or decrypting Windows on {volume}, so nothing was changed. Run \
     this installer again when it has finished",
)
.noting("{volume} is the drive Windows is on, for example C:.");

/// Not enough space.
pub const NOT_ENOUGH_SPACE: Word = Word::saying(
    "installer.not-enough-space",
    "Windows on {volume} needs {needed} GB free for this and has {free} GB, so nothing was changed",
)
.noting(
    "{volume} is the drive Windows is on. {needed} includes the space Windows keeps for itself \
     afterwards. Both numbers are whole gigabytes.",
);

/// An earlier start left something.
pub const ALREADY_STARTED: Word = Word::saying(
    "installer.already-started",
    "An earlier start of this installer left its area or its entry on this computer, so nothing \
     was changed",
)
.noting(
    "Said when the installer finds what it would make already there. It does not make a second \
     one, and it does not remove what it did not make in this run.",
);

/// No disk to install onto.
pub const NO_DISK_FOR_ALO_OS: Word = Word::saying(
    "installer.no-disk-for-alo-os",
    "This computer has no empty disk of at least {least} GB beside the one Windows is on, and \
     this installer puts alo OS on a disk of its own, so nothing was changed",
)
.noting("{least} is a whole number of gigabytes.");

/// Nothing typed.
pub const NOT_AGREED: Word = Word::saying(
    "installer.not-agreed",
    "You stopped the installer, so nothing was changed",
)
.noting("Said when the person pressed Enter without typing a disk's name.");

/// Something typed that is not a disk's name.
pub const NOT_A_DISKS_NAME: Word = Word::saying(
    "installer.not-a-disks-name",
    "What was typed is not the name of a disk alo OS can be installed onto, so nothing was changed",
)
.noting(
    "Said when the person typed something that does not match any disk offered. The installer \
     never guesses which disk was meant.",
);

/// Preparing failed and was put back.
pub const PUT_BACK: Word = Word::saying(
    "installer.put-back",
    "Preparing this computer did not finish. Everything that was changed has been put back, so \
     nothing was changed",
)
.noting("Said after the installer undid every step it had taken.");

// ---------------------------------------------------------------------------
// When putting back did not finish: exactly what remains.
// ---------------------------------------------------------------------------

/// Not everything could be put back.
pub const NOT_PUT_BACK: Word = Word::saying(
    "installer.not-put-back",
    "Preparing this computer did not finish, and not everything could be put back. Windows still \
     starts. This remains:",
)
.noting(
    "Deliberately does not say that nothing was changed, because something was. The lines below \
     say exactly what.",
);

/// Windows is smaller.
pub const REMAINS_SMALLER: Word = Word::saying(
    "installer.remains.smaller",
    "Windows on {volume} is {area} GB smaller than it was",
)
.noting("{volume} is the drive Windows is on, for example C:.");

/// The area remains.
pub const REMAINS_THE_AREA: Word = Word::saying(
    "installer.remains.the-area",
    "An area labelled ALO-INSTALL remains on the disk {disk}",
)
.noting("ALO-INSTALL is the label Windows shows for the area, and is not translated.");

/// The entry remains.
pub const REMAINS_THE_ENTRY: Word = Word::saying(
    "installer.remains.the-entry",
    "An entry named alo OS remains among the systems this computer can start",
)
.noting("alo OS is the product's name and is not translated.");

/// The next restart goes to the installer.
pub const REMAINS_THE_NEXT_START: Word = Word::saying(
    "installer.remains.the-next-start",
    "The next restart starts the alo OS installer, which installs onto the disk you named",
)
.noting("Said only when the one-time choice of what starts next could not be taken back.");

/// Fast Startup is still off.
pub const REMAINS_FAST_STARTUP_OFF: Word = Word::saying(
    "installer.remains.fast-startup-off",
    "Windows' Fast Startup is still off. You can turn it back on in Windows' own power settings",
)
.noting(
    "Said only when the setting the person asked to be turned off could not be put back. Fast \
     Startup is the name Windows' own settings use.",
);

/// The copy left in place is still there.
pub const REMAINS_THE_WAY_BACK: Word = Word::saying(
    "installer.remains.the-way-back",
    "The program that restarts this computer into alo OS is still installed. You can remove it in \
     Windows' own settings",
)
.noting("Said only when the copy the installer left in place could not be taken away again.");

// ---------------------------------------------------------------------------
// The way back into alo OS, from inside Windows.
// ---------------------------------------------------------------------------

/// Leaving the way back in place, said before it is done.
pub const LEAVING_THE_WAY_BACK: Word = Word::saying(
    "installer.leaving-the-way-back",
    "Putting a program in the Start menu that restarts this computer into alo OS",
)
.noting(
    "Said during the steps, before the copy is made. The Start menu is Windows' own name for \
     where a person finds their programs.",
);

/// The switch's first line.
pub const SWITCH_STARTING: Word = Word::saying(
    "installer.switch.starting",
    "This restarts this computer into alo OS. Nothing else about this computer is changed",
)
.noting(
    "The first line of the small program the installer leaves in place, which does one thing: \
     restart into alo OS once.",
);

/// What will happen, before the person agrees to it.
pub const SWITCH_WILL_RESTART: Word = Word::saying(
    "installer.switch.will-restart",
    "This computer restarts into alo OS once. Windows stays the one it starts normally, and \
     restarting again comes back to Windows",
)
.noting(
    "Said before the person agrees. The choice is for one restart: it does not change which \
     system the computer starts by default.",
);

/// The question.
pub const SWITCH_TYPE_TO_AGREE: Word = Word::saying(
    "installer.switch.type-to-agree",
    "To restart into alo OS now, type {word} and press Enter. To leave everything as it is, press \
     Enter without typing anything",
)
.noting("{word} holds the word below, in this language.");

/// The word that agrees, which the person types.
pub const SWITCH_AGREED: Word = Word::saying("installer.switch.agreed", "restart").noting(
    "The word a person types to agree to restarting into alo OS. Keep it to one word, and to a \
     word that means starting the computer again.",
);

/// The person did not agree.
pub const SWITCH_NOT_AGREED: Word = Word::saying(
    "installer.switch.not-agreed",
    "Nothing was changed, and this computer starts as it did",
)
.noting("Said when the person pressed Enter without typing the word.");

/// alo OS is not among the systems this computer can start.
pub const SWITCH_NOT_THERE: Word = Word::saying(
    "installer.switch.not-there",
    "alo OS is not among the systems this computer can start, so there is nothing to restart \
     into. Nothing was changed",
)
.noting(
    "Said when the firmware lists no entry for alo OS — on a computer where alo OS was never \
     installed, or was removed.",
);

/// The firmware's list could not be read.
pub const SWITCH_NOT_READ: Word = Word::saying(
    "installer.switch.not-read",
    "The systems this computer can start could not be read, so nothing was changed",
)
.noting("Said when Windows' own start-up tool did not answer.");

/// The next start could not be set.
pub const SWITCH_NOT_SET: Word = Word::saying(
    "installer.switch.not-set",
    "This computer could not be told to start alo OS next, so nothing was changed",
)
.noting("Said when setting the one-time choice failed; the computer starts as it did.");

/// Restarting.
pub const SWITCH_RESTARTING: Word = Word::saying(
    "installer.switch.restarting",
    "Starting alo OS. This computer restarts in a few seconds",
)
.noting("The last line before the restart.");

/// The restart did not happen.
pub const SWITCH_RESTART_IT_YOURSELF: Word = Word::saying(
    "installer.switch.restart-it-yourself",
    "This computer is set to start alo OS next. Restart it yourself when you are ready",
)
.noting("Said when the restart could not be asked for; the one-time choice is set and waits.");

// ---------------------------------------------------------------------------
// Which system this computer starts when nobody chooses.
// ---------------------------------------------------------------------------

/// alo OS, as a person reads it.
pub const THE_SYSTEM_ALO_OS: Word = Word::saying("installer.system.alo-os", "alo OS")
    .noting("The product's name. It is not translated.");

/// Windows, as a person reads it.
pub const THE_SYSTEM_WINDOWS: Word = Word::saying("installer.system.windows", "Windows")
    .noting("Microsoft's name for their system, as their own installer shows it.");

/// The first line of the program that shows the default.
pub const DEFAULT_STARTING: Word = Word::saying(
    "installer.default.starting",
    "This shows which system this computer starts when nobody chooses, and can change it",
)
.noting("The first line of the small program started to see or change the default.");

/// Which system starts now.
pub const DEFAULT_IS: Word = Word::saying(
    "installer.default.is",
    "This computer starts {system} when nobody chooses at the start-up menu",
)
.noting("{system} is one of the two system names above.");

/// The question.
pub const DEFAULT_TYPE_TO_CHANGE: Word = Word::saying(
    "installer.default.type-to-change",
    "To make this computer start {system} instead, type {change} and press Enter. To leave it as      it is, press Enter without typing anything",
)
.noting(
    "{system} is the other system's name and {change} is the word below, in this language. The      change is for every start from now on, and not for one restart.",
);

/// The word that changes it.
pub const DEFAULT_CHANGE_IT: Word = Word::saying("installer.default.change-it", "change").noting(
    "The word a person types to change which system the computer starts by default. One word.",
);

/// It was changed.
pub const DEFAULT_CHANGED: Word = Word::saying(
    "installer.default.changed",
    "Done. This computer now starts the other system when nobody chooses, and you can change it      back here or in alo OS's own settings",
)
.noting("Said after the change was written and read back.");

/// It was left as it was.
pub const DEFAULT_KEPT: Word = Word::saying(
    "installer.default.kept",
    "Nothing was changed, and this computer starts the same system it did",
)
.noting("Said when the person pressed Enter without typing the word.");

/// The loader's file is not there.
pub const DEFAULT_NOT_THERE: Word = Word::saying(
    "installer.default.not-there",
    "This computer keeps no choice of which system starts, so there is nothing to change here.      Nothing was changed",
)
.noting("Said when the file both systems keep the answer in is not on the start partition.");

/// The file could not be read or written.
pub const DEFAULT_NOT_READ: Word = Word::saying(
    "installer.default.not-read",
    "Which system this computer starts could not be read or changed, so nothing was changed",
)
.noting("Said when the file is not what it should be, or a change could not be written.");

/// The start partition could not be reached.
pub const DEFAULT_NOT_REACHED: Word = Word::saying(
    "installer.default.not-reached",
    "The part of the disk this computer starts from could not be reached, so nothing was changed",
)
.noting("Said when Windows would not give the start partition a drive letter.");

/// Closing.
pub const PRESS_ENTER_TO_CLOSE: Word = Word::saying(
    "installer.press-enter-to-close",
    "Press Enter to close this window",
)
.noting("The last line after the installer has stopped without restarting.");

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 96] = [
    STARTING,
    CHECKING_THE_DOWNLOAD,
    GENUINE,
    CHECKING_THE_COMPUTER,
    FOUND_UEFI,
    FOUND_NOT_UEFI,
    FOUND_STARTING_NOT_READ,
    FOUND_SECURE_BOOT_ON,
    FOUND_SECURE_BOOT_OFF,
    FOUND_SECURE_BOOT_NOT_READ,
    FOUND_TPM_READY,
    FOUND_TPM_NOT_READY,
    FOUND_NO_TPM,
    FOUND_TPM_NOT_READ,
    FOUND_BITLOCKER_ON,
    FOUND_BITLOCKER_OFF,
    FOUND_BITLOCKER_CHANGING,
    FOUND_BITLOCKER_NOT_READ,
    FOUND_SPACE,
    FOUND_MEMORY,
    FOUND_LITTLE_MEMORY,
    FOUND_MEMORY_NOT_READ,
    FOUND_WINDOWS_DISK,
    FOUND_A_DISK_FOR_ALO_OS,
    FOUND_A_DISK_IN_USE,
    FOUND_A_DISK_TOO_SMALL,
    FOUND_A_DISK_NOT_USABLE,
    NOTHING_CHANGED_YET,
    WILL_SHRINK_WINDOWS,
    WILL_MAKE_THE_AREA,
    WILL_ADD_THE_ENTRY,
    WILL_RESTART,
    FOUND_FAST_STARTUP_ON,
    FOUND_FAST_STARTUP_OFF,
    FOUND_FAST_STARTUP_NOT_READ,
    ASK_FAST_STARTUP,
    TYPE_ONE_OF_THESE_ANSWERS,
    ANSWER_TURN_OFF,
    ANSWER_LEAVE_ON,
    FAST_STARTUP_LEFT_ON,
    TURNING_FAST_STARTUP_OFF,
    TYPE_THE_DISKS_NAME,
    SHRINKING_WINDOWS,
    MAKING_THE_AREA,
    COPYING_THE_INSTALLER,
    ADDING_THE_ENTRY,
    PUTTING_IT_BACK,
    RESTARTING,
    RESTART_IT_YOURSELF,
    NOT_AN_ADMINISTRATOR,
    INCOMPLETE,
    NOT_GENUINE,
    NOT_UEFI,
    STARTING_NOT_READ,
    SECURE_BOOT_ON,
    SECURE_BOOT_NOT_READ,
    DISKS_NOT_READ,
    ENTRIES_NOT_READ,
    WINDOWS_DISK_NOT_SUPPORTED,
    BITLOCKER_CHANGING,
    NOT_ENOUGH_SPACE,
    ALREADY_STARTED,
    NO_DISK_FOR_ALO_OS,
    NOT_AGREED,
    NOT_A_DISKS_NAME,
    PUT_BACK,
    NOT_PUT_BACK,
    REMAINS_SMALLER,
    REMAINS_THE_AREA,
    REMAINS_THE_ENTRY,
    REMAINS_THE_NEXT_START,
    REMAINS_FAST_STARTUP_OFF,
    REMAINS_THE_WAY_BACK,
    LEAVING_THE_WAY_BACK,
    SWITCH_STARTING,
    SWITCH_WILL_RESTART,
    SWITCH_TYPE_TO_AGREE,
    SWITCH_AGREED,
    SWITCH_NOT_AGREED,
    SWITCH_NOT_THERE,
    SWITCH_NOT_READ,
    SWITCH_NOT_SET,
    SWITCH_RESTARTING,
    SWITCH_RESTART_IT_YOURSELF,
    THE_SYSTEM_ALO_OS,
    THE_SYSTEM_WINDOWS,
    DEFAULT_STARTING,
    DEFAULT_IS,
    DEFAULT_TYPE_TO_CHANGE,
    DEFAULT_CHANGE_IT,
    DEFAULT_CHANGED,
    DEFAULT_KEPT,
    DEFAULT_NOT_THERE,
    DEFAULT_NOT_READ,
    DEFAULT_NOT_REACHED,
    PRESS_ENTER_TO_CLOSE,
];

/// Every refusal, each of which is said before anything was changed.
pub const EVERY_REFUSAL: [Word; 17] = [
    NOT_AN_ADMINISTRATOR,
    INCOMPLETE,
    NOT_GENUINE,
    NOT_UEFI,
    STARTING_NOT_READ,
    SECURE_BOOT_ON,
    SECURE_BOOT_NOT_READ,
    DISKS_NOT_READ,
    ENTRIES_NOT_READ,
    WINDOWS_DISK_NOT_SUPPORTED,
    BITLOCKER_CHANGING,
    NOT_ENOUGH_SPACE,
    ALREADY_STARTED,
    NO_DISK_FOR_ALO_OS,
    NOT_AGREED,
    NOT_A_DISKS_NAME,
    PUT_BACK,
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
pub fn installer_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "installer", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        for refusal in EVERY_REFUSAL {
            assert!(named.contains(refusal.named()), "{}", refusal.named());
        }
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = installer_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note**, because a translator cannot work without
    /// one.
    #[test]
    fn every_word_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Nothing here names the machinery, a key, a password or a signature.**
    #[test]
    fn nothing_here_names_the_machinery_a_key_or_a_signature() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "bootc",
                "cosign",
                "podman",
                "container",
                "image",
                "registry",
                "digest",
                "sha256",
                "checksum",
                "hash",
                "signature",
                "signed",
                "key",
                "password",
                "certificate",
                "partition",
                "firmware",
                "initramfs",
                "kernel",
                "bcdedit",
                "powershell",
                "shim",
                "grub",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
        }
    }

    /// **Every refusal says nothing was changed**, and the one ending where
    /// something remains does not claim that.
    #[test]
    fn every_refusal_says_nothing_was_changed() {
        for word in EVERY_REFUSAL {
            assert!(
                word.says().contains("so nothing was changed"),
                "{}",
                word.named()
            );
        }
        assert!(!NOT_PUT_BACK.says().contains("nothing was changed"));
    }

    /// **The Secure Boot refusal says why and suggests nothing** (ADR 0033 §4).
    #[test]
    fn the_secure_boot_refusal_never_suggests_changing_the_setting() {
        for word in [SECURE_BOOT_ON, SECURE_BOOT_NOT_READ, FOUND_SECURE_BOOT_ON] {
            let said = word.says().to_lowercase();
            for suggestion in [
                "turn",
                "switch",
                "disable",
                "change it",
                "change the",
                "changing",
                "off",
                "setting",
                "menu",
                "try",
            ] {
                assert!(
                    !said.contains(suggestion),
                    "{} says \"{suggestion}\"",
                    word.named()
                );
            }
        }
        assert!(SECURE_BOOT_ON.says().contains("not yet approved"));
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
