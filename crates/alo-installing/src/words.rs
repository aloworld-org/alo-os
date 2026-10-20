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

// ---------------------------------------------------------------------------
// The five screens this environment draws, adopted by the owner on 2026-10-20
// as `530:30834`. One page with a different question on it each time; the
// layout is `crate::where_a_screen_goes`'.
//
// **Every one of these is said on a screen, not written to a console.** The
// lines above are the running commentary a person watches while an install
// proceeds; these are things they are asked.
// ---------------------------------------------------------------------------

/// S2's eyebrow: where the person is.
pub const FROM_A_STICK: Word = Word::saying("installing.from-a-stick", "FROM A STICK").noting(
    "The small line above the question on the first screen somebody meets after starting this \
     computer from a USB stick. Capitals in English; use whatever a reader of your language \
     reads as a quiet label above a heading, and do not add punctuation.",
);

/// S2's question.
pub const TRY_OR_INSTALL: Word =
    Word::saying("installing.try-or-install", "Try alo OS, or install it")
        .noting("The question on the screen somebody meets after starting from a stick.");

/// S2's sentence under the question.
pub const NOTHING_HAS_BEEN_CHANGED_YET: Word = Word::saying(
    "installing.nothing-changed-yet",
    "You are running alo OS from the stick. Nothing on this computer has been changed",
)
.noting(
    "Under the question on the first screen from a stick. Both halves matter: where they are \
     running it from, and that their own computer is untouched. Somebody who has just started a \
     strange operating system on their own machine is being told the second thing before they \
     are asked anything.",
);

/// S2's first choice.
pub const TRY_IT_FIRST: Word = Word::saying("installing.try-it-first", "Try it first").noting(
    "The first of two choices. Trying means using the whole system from the stick, changing \
     nothing. Keep it short: it is a heading on a card, with its own sentence under it.",
);

/// What trying costs and gives.
pub const WHAT_TRYING_IS: Word = Word::saying(
    "installing.what-trying-is",
    "Use the whole system from the stick. See that your screen, your Wi-Fi and your trackpad work",
)
.noting(
    "Under *Try it first*. The three things named are the three that most often do not work on a \
     new operating system, which is why they are named rather than summarised as *your \
     hardware*. Substitute the three a reader of your language would worry about if those are \
     not they.",
);

/// S2's second choice, and S4's own action.
pub const INSTALL_ALO_OS: Word = Word::saying("installing.install-alo-os", "Install alo OS")
    .noting(
        "The second of two choices, and the name of what this program does. *alo OS* is the \
         product's name and is never translated.",
    );

/// What installing will ask for.
pub const WHAT_INSTALLING_IS: Word = Word::saying(
    "installing.what-installing-is",
    "Choose a disk to install on. You will be told exactly what changes before anything does",
)
.noting(
    "Under *Install alo OS*. The promise in the second sentence is one this program keeps and \
     must not be softened: nothing is written until the person has typed the name of the disk \
     they chose.",
);

/// S2's line under the choices.
pub const INSTALL_WHENEVER: Word = Word::saying(
    "installing.install-whenever",
    "You can install at any time while you are trying it",
)
.noting(
    "The quiet line under the two choices. It is there so that choosing *Try it first* does not \
     read as giving something up.",
);

/// S3's eyebrow.
pub const NETWORK: Word = Word::saying("installing.network", "NETWORK").noting(
    "The quiet label above the question about connecting. Capitals in English; whatever reads \
     as a label above a heading in your language.",
);

/// S3's question.
pub const HOW_SHOULD_THIS_COMPUTER_CONNECT: Word = Word::saying(
    "installing.how-to-connect",
    "How should this computer connect?",
)
.noting(
    "The question on the screen that chooses a network. *This computer* rather than *your computer*: the \
     person may be installing on a machine that is not theirs.",
);

/// S3's sentence under the question.
pub const WHY_A_NETWORK_IS_NEEDED: Word = Word::saying(
    "installing.why-a-network",
    "alo OS is downloaded over the network. Your password is used here and kept nowhere",
)
.noting(
    "Under the question. The second half is a promise this program keeps: the password a person \
     types reaches the network and is written to no disk, and a \
     translation that softens it into *not stored permanently* would be promising something \
     weaker than what happens.",
);

/// S3's line under the choices.
pub const THE_PASSWORD_IS_NEVER_WRITTEN: Word = Word::saying(
    "installing.password-never-written",
    "Your Wi-Fi password is never written to this computer's disk",
)
.noting(
    "The quiet line under the networks. *Wi-Fi* is a trademark and is spelled this way in every \
     language. Keep *never* and keep *written*: the sentence is about what is not put on their \
     disk, which is the thing a careful person is wondering about.",
);

/// S4's eyebrow.
pub const WHERE_TO_INSTALL: Word = Word::saying("installing.where-to-install", "WHERE TO INSTALL")
    .noting("The quiet label above the question that chooses a disk.");

/// S4's question.
pub const WHICH_DISK: Word = Word::saying("installing.which-disk", "Which disk should alo OS use?")
    .noting(
        "The question on the screen that chooses a disk. *alo OS* is \
     never translated.",
    );

/// S4's sentence under the question, and the promise it carries.
pub const NOTHING_UNTIL_YOU_TYPE_IT: Word = Word::saying(
    "installing.nothing-until-you-type-it",
    "Nothing is changed until you type the name of the disk you choose",
)
.noting(
    "Under the question that chooses a disk, and the same promise the Windows half makes in the \
     same words. It is exact and must stay exact: the program really does change nothing until \
     the person has typed that disk's own name.",
);

/// S4's line under the disks.
pub const THE_STICK_IS_NOT_SHOWN: Word = Word::saying(
    "installing.the-stick-is-not-shown",
    "The stick you started from is not shown here and cannot be chosen",
)
.noting(
    "The quiet line under the list of disks. It explains an absence — a person looking for their \
     USB stick in the list will not find it — which is why it is said rather than left for them \
     to wonder about.",
);

/// S5's eyebrow, and the sharpest one in this file.
pub const CHECK_THIS_CAREFULLY: Word =
    Word::saying("installing.check-this-carefully", "CHECK THIS CAREFULLY").noting(
        "The quiet label above the review of an erase. **It is the only one of these labels that \
         is a warning**, and it should read as one: this is the screen before a disk is wiped. \
         Do not soften it into *please review*.",
    );

/// S5's question, which is not a question.
pub const THIS_ERASES: Word = Word::saying("installing.this-erases", "This erases {disk}").noting(
    "The heading on the review screen. {disk} is the disk's own name as a person reads it — \
     *Samsung SSD 870 EVO · 1 TB* — and is not translated. \
     **Present tense and active**: not *this will erase* and not *this would erase*. The person \
     is about to do it.",
);

/// What goes, said plainly.
pub const WHAT_WILL_BE_GONE: Word = Word::saying(
    "installing.what-will-be-gone",
    "{holding} on this disk will be gone, and cannot be brought back",
)
.noting(
    "Under the heading on the review screen. {holding} is what is on the disk, already written \
     out by this program — *Windows and about 340 GB of files*. **Keep the second clause.** It \
     is the whole sentence: a person who misreads the first half and acts anyway is the person \
     this screen exists for.",
);

/// S5's instruction.
pub const TYPE_THE_NAME_TO_CONTINUE: Word = Word::saying(
    "installing.type-the-name",
    "Type the name of the disk to continue",
)
.noting(
    "The heading of the field a person types into to confirm. The name they must type is the \
     disk's own, shown beneath it, and matching is exact.",
);

/// What is not touched.
pub const WHAT_STAYS: Word = Word::saying("installing.what-stays", "What stays").noting(
    "The heading of the second card on the review screen, which says what is **not** touched. \
     It is there because a person about to erase one disk is wondering about the others.",
);

/// Which is the answer to that.
pub const NO_OTHER_DISK_IS_TOUCHED: Word = Word::saying(
    "installing.no-other-disk-is-touched",
    "Nothing on any other disk is touched, including the stick you started from",
)
.noting("Under *What stays*. Both halves are true and both are what somebody is wondering.");

/// S5's line under the cards.
pub const NOTHING_HAS_CHANGED_YET: Word = Word::saying(
    "installing.nothing-has-changed-yet",
    "Nothing has been changed yet. Going back changes nothing",
)
.noting(
    "The quiet line on the review screen. Two short sentences on purpose: the first is where \
     they are, the second is that leaving is free. Do not merge them.",
);

/// S5's action.
pub const ERASE_AND_INSTALL: Word =
    Word::saying("installing.erase-and-install", "Erase and install").noting(
        "The button on the review screen. **It names what it does**, which is why it is not \
         *Continue*: a button that erases a disk should say so on its face.",
    );

/// S6's eyebrow.
pub const NOTHING_WAS_CHANGED: Word =
    Word::saying("installing.nothing-was-changed", "NOTHING WAS CHANGED").noting(
        "The quiet label above a refusal. It is the first thing a person reads on a screen that \
         is telling them they cannot go on, and it answers the question they have before they \
         have asked it.",
    );

/// S6's heading.
pub const NO_DISK_IS_BIG_ENOUGH: Word = Word::saying(
    "installing.no-disk-is-big-enough",
    "No disk here is big enough",
)
.noting(
    "The heading when nothing on this computer can hold alo OS. \
     *Here* means this computer, and it matters: a disk somewhere else would do.",
);

/// S6's next step, which is what makes it a boundary rather than a dead end.
pub const CONNECT_ANOTHER_DISK: Word =
    Word::saying("installing.connect-another-disk", "Connect another disk").noting(
        "The first of two ways on, on the screen that refuses. A refusal that names no next step \
         is a dead end, and this product does not leave one.",
    );

/// And the other.
pub const INSTALL_FROM_WINDOWS_INSTEAD: Word = Word::saying(
    "installing.install-from-windows-instead",
    "Install from Windows instead",
)
.noting(
    "The second way on. *Windows* is a product name and is not translated. Installing from \
     Windows uses space Windows itself frees, so a disk too small for a whole install may still \
     have room beside it.",
);

// ---------------------------------------------------------------------------
// The keyboard, which comes before anything is typed. `530:30876`.
//
// **It is first because typing presupposes it.** The installed system decided
// the same thing for the same reason — its language and keyboard sheet is the
// first screen at first start — and here it is sharper: the two things this
// environment asks a person to type are a network password and a disk's own
// name, and both fail silently and identically on the wrong layout.
// ---------------------------------------------------------------------------

/// S0's eyebrow.
pub const BEFORE_ANYTHING_IS_TYPED: Word = Word::saying(
    "installing.before-anything-typed",
    "BEFORE ANYTHING IS TYPED",
)
.noting("The quiet label above the keyboard question. It says why this screen comes first.");

/// S0's question.
pub const WHICH_KEYBOARD: Word = Word::saying(
    "installing.which-keyboard",
    "Which keyboard are you using?",
)
.noting(
    "The question on the first screen. *Using* rather than *have*: a person may be installing on \
     somebody else's computer with their own keyboard plugged in.",
);

/// S0's sentence under the question.
pub const WHY_THE_KEYBOARD_COMES_FIRST: Word = Word::saying(
    "installing.why-the-keyboard-first",
    "You will type a network password and the name of a disk. Both have to be exact, so the \
     keyboard has to be right first",
)
.noting(
    "Under the question. It names the two things that will be typed, because a person asked \
     about their keyboard before anything else has happened deserves to know why they are being \
     asked. *Exact* is the word that matters: neither is checked for near-enough.",
);

/// S0's check, which is the card rather than a list.
pub const PRESS_THESE_TO_CHECK: Word = Word::saying(
    "installing.press-these-to-check",
    "Press the keys shown to check: {keys}",
)
.noting(
    "Under the layout this screen thinks is right. {keys} are characters, not words, and are \
     never translated — they are chosen because they move between layouts. **This card asks the \
     keyboard to prove itself rather than asking the person to recognise a name**: somebody who \
     cannot tell one layout from another in a list can tell whether what appeared matches what \
     is printed on their key.",
);

/// S0's other choice.
pub const SOMETHING_ELSE: Word = Word::saying("installing.something-else", "Something else")
    .noting(
        "The second choice on the keyboard screen, which opens the full list. Short on purpose: \
         it is a way out of a guess, not a description.",
    );

/// What that opens.
pub const EVERY_LAYOUT_THIS_COMPUTER_KNOWS: Word = Word::saying(
    "installing.every-layout",
    "Choose from every layout this computer knows",
)
.noting("Under *Something else*.");

/// S0's line under the cards.
pub const IF_IT_DOES_NOT_MATCH: Word = Word::saying(
    "installing.if-it-does-not-match",
    "If what you type below does not match, choose another layout",
)
.noting(
    "The quiet line on the keyboard screen. It tells a person what to do about a result rather \
     than only what the screen is for.",
);

// ---------------------------------------------------------------------------
// Writing a stick, which the owner ruled on 2026-10-20 that alo OS does
// itself as well as publishing an image a person writes with their own tool.
// `530:30904` to `530:30988`.
//
// **The only road in this product that erases a device nobody chose for alo
// OS.** `docs/features.md` says so in the line that adds it, and every
// sentence here is written on that footing: what is offered, what is on it,
// what goes, and nothing written until the person types that device's name.
// ---------------------------------------------------------------------------

/// S7's eyebrow.
pub const WRITE_A_STICK: Word = Word::saying("installing.write-a-stick", "WRITE A STICK")
    .noting("The quiet label above the question that chooses a stick to write.");

/// S7's question.
pub const WHICH_STICK: Word = Word::saying(
    "installing.which-stick",
    "Which stick should alo OS write?",
)
.noting(
    "The question on the screen that chooses a device to write. *Stick* is the ordinary word for \
     a USB memory device in English; use whatever ordinary word a reader of your language would \
     use, not a technical one.",
);

/// S7's sentence under the question.
pub const THE_STICK_WILL_BE_ERASED: Word = Word::saying(
    "installing.the-stick-will-be-erased",
    "Everything on the stick you choose will be erased. Nothing is changed until you type its \
     name",
)
.noting(
    "Under the question. Both halves are promises: the first is what happens, the second is when \
     — and the second is the same consent the disk takes, in the same words. **Keep \
     *everything*.** There is no road that writes a stick and keeps what was on it.",
);

/// S7's line under the devices.
pub const ONLY_REMOVABLE_DEVICES: Word = Word::saying(
    "installing.only-removable-devices",
    "Only removable devices are shown. The disk this computer runs on is not one of them",
)
.noting(
    "The quiet line under the list of sticks. It explains an absence — somebody looking for \
     their computer's own disk will not find it — and says why, which is that this screen makes \
     something you carry to another machine.",
);

/// S8's heading, which names the device rather than the act.
pub const THIS_ERASES_THE_STICK: Word =
    Word::saying("installing.this-erases-the-stick", "This erases {stick}").noting(
        "The heading on the review before a stick is written. {stick} is the device's own name \
         as a person reads it — *SanDisk Ultra* — and is not translated. Present tense and \
         active, as on the disk's own review: not *will erase*.",
    );

/// What goes from it.
pub const WHAT_GOES_FROM_THE_STICK: Word = Word::saying(
    "installing.what-goes-from-the-stick",
    "{holding} on this stick will be gone, and cannot be brought back",
)
.noting(
    "Under the heading on the stick's review. {holding} is what is on it, already written out — \
     *About 4 GB of files*. **Keep the second clause**: it is the sentence, and the person who \
     misreads the first half is who this screen exists for.",
);

/// S8's instruction.
pub const TYPE_THE_STICKS_NAME: Word = Word::saying(
    "installing.type-the-sticks-name",
    "Type the name of the stick to continue",
)
.noting("The heading of the field that confirms. Matching is exact.");

/// What a stick's review promises is untouched.
pub const ONLY_THE_STICK_IS_WRITTEN: Word = Word::saying(
    "installing.only-the-stick-is-written",
    "Nothing on this computer is touched. Only the stick is written",
)
.noting(
    "Under *What stays* on the stick's review. Somebody about to let a program erase something \
     is wondering what else it can reach, and this is the answer.",
);

/// S8's action, which names what it does.
pub const ERASE_AND_WRITE: Word = Word::saying("installing.erase-and-write", "Erase and write")
    .noting(
        "The button on the stick's review. It names what it does rather than saying *Continue*, \
         for the reason *Erase and install* does: a button that erases should say so on its \
         face.",
    );

/// S9's eyebrow.
pub const WRITING: Word = Word::saying("installing.writing", "WRITING")
    .noting("The quiet label while a stick is being written.");

/// S9's heading.
pub const WRITING_TO_THE_STICK: Word = Word::saying(
    "installing.writing-to-the-stick",
    "Writing alo OS to {stick}",
)
.noting(
    "The heading while the write runs. {stick} is the device's own name and is not translated. \
     *alo OS* is the product's name and is not either.",
);

/// The one thing a person must not do.
pub const LEAVE_THE_STICK_IN: Word = Word::saying(
    "installing.leave-the-stick-in",
    "Leave the stick in until this finishes. Taking it out now leaves it unusable",
)
.noting(
    "Under the heading while writing. It is the only instruction on this screen and the only \
     thing a person can get wrong here, which is why it is said where they are looking rather \
     than in a warning they already dismissed.",
);

/// What was checked before anything was written.
pub const CHECKED_BEFORE_IT_WAS_WRITTEN: Word = Word::saying(
    "installing.checked-before-written",
    "Checked before it was written",
)
.noting(
    "The heading of the first card while writing. What was checked is that what is being written \
     is a genuine alo OS; the sentence under it says so. Do not name how it was checked.",
);

/// And what that means.
pub const ONLY_A_GENUINE_ALO_OS: Word = Word::saying(
    "installing.only-a-genuine-alo-os",
    "Only a genuine alo OS is written to a stick",
)
.noting(
    "Under *Checked before it was written*. **Do not name the machinery**: not the signature, \
     not the key, not where it came from. A person needs to know that it was checked, and \
     learning the names of the tools tells them only that something complicated happened.",
);

/// How far it has got.
pub const HOW_FAR_THE_WRITE_HAS_GOT: Word =
    Word::saying("installing.how-far-the-write", "Writing  \u{b7}  {how_far}").noting(
        "The second card while writing. {how_far} is already written out as a percentage and is \
         not translated. The separator is a middle dot with two spaces each side, as on every \
         other row in this product.",
    );

/// And roughly how long is left.
pub const ABOUT_HOW_LONG_IS_LEFT: Word =
    Word::saying("installing.about-how-long", "About {how_long} left").noting(
        "Under the progress. {how_long} is already written out — *two minutes* — and **about** \
         must be kept: the number is an estimate and saying it exactly would be claiming \
         something this program does not know.",
    );

/// What a person may do meanwhile.
pub const KEEP_USING_THIS_COMPUTER: Word = Word::saying(
    "installing.keep-using-this-computer",
    "You can keep using this computer while it writes",
)
.noting(
    "The quiet line while writing. It is said because a person watching a progress bar on their \
     own working computer does not otherwise know whether they are allowed to go away.",
);

/// S10's eyebrow.
pub const READY: Word = Word::saying("installing.ready", "READY")
    .noting("The quiet label when a stick has been written and nothing is left to do.");

/// S10's heading.
pub const THE_STICK_IS_READY: Word =
    Word::saying("installing.the-stick-is-ready", "The stick is ready").noting(
        "The heading when the write has finished. One short line: there is nothing to explain.",
    );

/// What to do with it.
pub const TAKE_IT_TO_THE_OTHER_COMPUTER: Word = Word::saying(
    "installing.take-it-to-the-other-computer",
    "Take it to the computer you want alo OS on, and start that computer from it",
)
.noting(
    "Under the heading. It says the next step in the person's own terms — carrying a thing to a \
     machine — rather than naming what the machine does with it.",
);

/// S10's first choice, which is S1's own action.
pub const START_THIS_PC_FROM_IT: Word = Word::saying(
    "installing.start-this-pc-from-it",
    "Start this PC from it instead",
)
.noting(
    "The first of two choices once a stick is written. *Instead* matters: it is an alternative \
     to carrying it elsewhere, for somebody who has changed their mind. *PC* is the ordinary \
     word for the machine in front of them.",
);

/// What that does.
pub const RESTART_AND_START_FROM_IT: Word = Word::saying(
    "installing.restart-and-start-from-it",
    "Restart this computer and start from the stick you just wrote",
)
.noting("Under *Start this PC from it instead*.");

/// S10's second choice.
pub const WRITE_ANOTHER: Word = Word::saying("installing.write-another", "Write another")
    .noting("The second choice once a stick is written, for somebody making more than one.");

/// What that does.
pub const CHOOSE_ANOTHER_STICK: Word = Word::saying(
    "installing.choose-another-stick",
    "Choose another stick to write",
)
.noting("Under *Write another*.");

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 97] = [
    BEFORE_ANYTHING_IS_TYPED,
    WHICH_KEYBOARD,
    WHY_THE_KEYBOARD_COMES_FIRST,
    PRESS_THESE_TO_CHECK,
    SOMETHING_ELSE,
    EVERY_LAYOUT_THIS_COMPUTER_KNOWS,
    IF_IT_DOES_NOT_MATCH,
    WRITE_A_STICK,
    WHICH_STICK,
    THE_STICK_WILL_BE_ERASED,
    ONLY_REMOVABLE_DEVICES,
    THIS_ERASES_THE_STICK,
    WHAT_GOES_FROM_THE_STICK,
    TYPE_THE_STICKS_NAME,
    ONLY_THE_STICK_IS_WRITTEN,
    ERASE_AND_WRITE,
    WRITING,
    WRITING_TO_THE_STICK,
    LEAVE_THE_STICK_IN,
    CHECKED_BEFORE_IT_WAS_WRITTEN,
    ONLY_A_GENUINE_ALO_OS,
    HOW_FAR_THE_WRITE_HAS_GOT,
    ABOUT_HOW_LONG_IS_LEFT,
    KEEP_USING_THIS_COMPUTER,
    READY,
    THE_STICK_IS_READY,
    TAKE_IT_TO_THE_OTHER_COMPUTER,
    START_THIS_PC_FROM_IT,
    RESTART_AND_START_FROM_IT,
    WRITE_ANOTHER,
    CHOOSE_ANOTHER_STICK,
    FROM_A_STICK,
    TRY_OR_INSTALL,
    NOTHING_HAS_BEEN_CHANGED_YET,
    TRY_IT_FIRST,
    WHAT_TRYING_IS,
    INSTALL_ALO_OS,
    WHAT_INSTALLING_IS,
    INSTALL_WHENEVER,
    NETWORK,
    HOW_SHOULD_THIS_COMPUTER_CONNECT,
    WHY_A_NETWORK_IS_NEEDED,
    THE_PASSWORD_IS_NEVER_WRITTEN,
    WHERE_TO_INSTALL,
    WHICH_DISK,
    NOTHING_UNTIL_YOU_TYPE_IT,
    THE_STICK_IS_NOT_SHOWN,
    CHECK_THIS_CAREFULLY,
    THIS_ERASES,
    WHAT_WILL_BE_GONE,
    TYPE_THE_NAME_TO_CONTINUE,
    WHAT_STAYS,
    NO_OTHER_DISK_IS_TOUCHED,
    NOTHING_HAS_CHANGED_YET,
    ERASE_AND_INSTALL,
    NOTHING_WAS_CHANGED,
    NO_DISK_IS_BIG_ENOUGH,
    CONNECT_ANOTHER_DISK,
    INSTALL_FROM_WINDOWS_INSTEAD,
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
        const THE_GAPS: [&str; 8] = [
            // A device's own name, as the machine reports it. Never translated.
            "disk", // How large a disk is, already written out: *1 TB*.
            "size", // How much of a disk is in use, already written out: *340 GB*.
            "files",
            // What is on a disk, already said in words by this crate:
            // *Windows and about 340 GB of files*.
            "holding", // A removable device's own name: *SanDisk Ultra*.
            "stick",   // The characters a person presses to prove their keyboard.
            "keys",    // How far a write has got, already a percentage.
            "how_far", // How long is left, already written out: *two minutes*.
            "how_long",
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
    ///
    /// # The person's own password is not the machinery, and that distinction
    /// is the whole of the exception below
    ///
    /// This rule is `docs/features.md`'s *a person never learns the name of
    /// anything we rented*, and the installer plan's *no screen, prompt or
    /// sentence names a key, a password or a signature*. Both are about **our**
    /// keys: the one a release is signed with, the signature a download is
    /// checked against, the registry it came from. A person has no use for any
    /// of those names and learning them tells them only that something
    /// complicated happened.
    ///
    /// **Their Wi-Fi password is theirs.** It is the thing they type, on a
    /// screen that exists to ask for it, and every other system says the word
    /// on that screen. A sentence that promised *your secret is never written
    /// to this disk* would be this product being coy about the one thing a
    /// careful person wants stated plainly.
    ///
    /// So two sentences are excepted **by name**, and the assertion below is
    /// equality rather than containment: the list cannot grow without somebody
    /// editing this line, and an exception that stops being needed is one that
    /// fails here.
    #[test]
    fn nothing_here_names_the_machinery_a_key_or_a_signature() {
        const MAY_SAY_PASSWORD: [&str; 3] = [
            // *You will type a network password and the name of a disk* —
            // said on the screen that comes before anything is typed, which
            // exists because that password has to be exact.
            "installing.why-the-keyboard-first",
            // *Your password is used here and kept nowhere* — said while
            // asking for it.
            "installing.why-a-network",
            // *Your Wi-Fi password is never written to this computer's disk* —
            // the promise
            // [ADR 0128](../../../docs/decisions/0128-whether-a-wi-fi-password-may-be-written-to-the-staging-partition.md)
            // made.
            "installing.password-never-written",
        ];
        let excepted: Vec<String> = EVERY_WORD
            .iter()
            .filter(|word| word.says().to_lowercase().contains("password"))
            .map(|word| word.key().as_str().to_owned())
            .collect();
        assert_eq!(
            excepted, MAY_SAY_PASSWORD,
            "the sentences that say *password* are not the two that may. A person's own Wi-Fi \
             password may be named on the screen that asks for it; ours may not be named \
             anywhere"
        );

        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            if MAY_SAY_PASSWORD.contains(&word.key().as_str()) {
                continue;
            }
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
                // **Whole words, not substrings.** This read `contains` until
                // 2026-10-20, when the first sentence in this crate to say
                // *keyboard* failed on `key` — and *keyboard* is the one word
                // the screen before every typed thing cannot avoid. The rule
                // is about naming our machinery, and a word that merely
                // contains those letters names nothing.
                //
                // Nobody had noticed because nothing here had ever said
                // keyboard, monkey or keystroke. The same shape as the market
                // fault and the `the_bar:` guard: a match that is right about
                // the letters and wrong about the thing.
                let says_it = if forbidden.contains(' ') {
                    said.contains(forbidden)
                } else {
                    said.split(|c: char| !c.is_alphanumeric())
                        .any(|one| one == forbidden)
                };
                assert!(!says_it, "{} says \"{forbidden}\"", word.named());
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
