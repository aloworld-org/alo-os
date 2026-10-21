//! The installer's reboot half: the boot environment that installs alo OS.
//!
//! [ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §2–3: the installer a person downloads *stages a minimal boot environment
//! and a UEFI boot entry, then reboots*, and that environment *runs `bootc
//! install`, pulling alo OS from the registry over HTTPS, verifying signatures
//! before writing.* This crate is the program inside that environment, and
//! `image/installing/` is the recipe the environment is built from.
//!
//! # What the environment is
//!
//! A kernel and an initramfs from the same pinned base the operating system is
//! built on, holding the tool that writes the disk, the container tooling it
//! needs, the checker for the owner's signature, and this program — and
//! nothing else. It is not a second operating system: it has no shell anybody
//! is offered, no login, and one unit, which runs [`install`] and stops.
//!
//! # What it is told, and what it never assumes
//!
//! What arrives from outside it is the choice the person made on the machine
//! they still had, carried on the kernel command line ([`Told`]): which disk,
//! and — on the road that keeps the system already on that disk — which two
//! partitions of it. What it installs and which key it accepts were built into
//! it ([`Environment`]) and never arrive over the network. It never touches a
//! disk it was not told to; it refuses a disk that holds this installer or
//! another operating system, and on the road that keeps one it refuses any
//! partition but the one the installer on Windows made and labelled
//! ([`Disks`]); and it writes nothing until the release is shown to be the
//! owner's ([`Verifying`]).
//!
//! # What it says
//!
//! Every step, as it begins, on every console the machine has (`console`),
//! in the words `alo-saying` collects (`words`). The person watching has no
//! other window.
//!
//! # What this crate is not
//!
//! **It is not the program a person downloads.** That is task 3 of the
//! installer plan — the Windows program that checks the machine, takes the
//! typed consent, stages this environment and adds the boot entry. What that
//! program owes this one is written in `docs/booting.md`: the environment's
//! files on a FAT partition labelled [`THIS_INSTALLER`], the chosen disk's name
//! in `EFI/BOOT/chosen.cfg` beside the loader's configuration, and a boot
//! entry reached once, through the firmware's next-boot choice, so that the
//! restart this environment ends with never lands back in it.
//!
//! On the road that keeps the system already on the disk it owes one thing
//! more: the space it made for alo OS carries the label
//! [`THE_SPACE_THE_INSTALLER_MADE`], and the two partitions' names are on the
//! command line ([`THE_PARTITION`], [`THE_EFI`]). The label is not a
//! convenience — it is the whole of this environment's permission to make a
//! file system on that partition, and without it a command line staged wrongly
//! would be the end of somebody's Windows.

mod a_key_reaches_here;
mod a_line_as_coverage;
mod a_screen_is_drawn;
mod a_surface_to_draw_on;
mod alos_own_face;
mod complaint;
mod console;
mod disk;
mod disks;
mod ended;
mod entries;
mod environment;
mod ink_on_the_surface;
mod joining_a_network;
mod machine;
mod program;
#[cfg(target_os = "linux")]
mod running;
mod sequence;
mod the_asking_walks;
mod the_environment_asks;
mod the_screen_that_lists_every_layout;
mod tidying;
mod told;
mod verifying;
mod what_a_person_typed;
mod what_display_there_is;
mod what_each_screen_says;
mod where_a_list_screen_goes;
mod where_a_screen_goes;
mod which_choices_are_shown;
mod which_disks_may_be_offered;
mod why_there_is_no_network;
// **Public, because whoever draws a screen needs the sentences on it.** The
// constants are what `what_each_screen_says` chooses among and what
// `main` says while a program runs; a drawer that could not reach them
// would have to hold its own copy of the English, which is the one thing this
// crate exists to prevent.
pub mod words;
mod wrapping;
mod writing;

pub use a_key_reaches_here::{
    ONE_EVENT, Pressed, WHERE_THE_KEYS_ARE, has_key, is_a_keyboard, what_was_pressed,
};
pub use a_line_as_coverage::{ACoverage, a_line_as_coverage};
pub use a_screen_is_drawn::{
    ALineToDraw, TheWordsOfAScreen, WhatAScreenShows, what_a_screen_shows,
};
pub use a_surface_to_draw_on::{
    Grey, NoSurface, TheSurface, WHERE_THE_DEPTH_IS, WHERE_THE_PIXELS_GO, WHERE_THE_SIZE_IS,
    WHERE_THE_STRIDE_IS,
};
pub use alos_own_face::{AlosOwnFace, NoFace};
pub use ink_on_the_surface::a_row_of_ink;
// Gated with the item it names, which `alo-citing`'s own guard requires: a
// `pub use` of a `cfg`-gated item, without the same `cfg`, is what stopped
// `alo-portals` compiling for Windows from #635 to #839.
#[cfg(unix)]
pub use a_surface_to_draw_on::fill;
// The same gate, for the same reason, on the one function in
// `ink_on_the_surface` that writes.
#[cfg(unix)]
pub use a_screen_is_drawn::draw_a_screen;
pub use complaint::{Complaint, THE_LAST_LINES};
pub use console::{ACTIVE, every_console, every_serial_line};
pub use disk::{BY_ID, DiskName, NotADisk, NotAPartition, PartitionName};
pub use disks::{
    ANOTHER_SYSTEMS, AWholeDisk, Disks, NotKeeping, Replacing, THE_SPACE_THE_INSTALLER_MADE,
    THE_START_UP_AREA, THIS_INSTALLER, Unsuitable,
};
pub use ended::{Ended, Refusal};
pub use entries::{Entries, Entry, THE_ENTRYS_NAME, THE_LOADER_THE_BASE_INSTALLS};
pub use environment::{Environment, WHERE_IT_IS};
#[cfg(unix)]
pub use ink_on_the_surface::draw;
pub use joining_a_network::{ANetwork, NotJoined, TEMPORARY, every_network, join, why_not_joined};
pub use machine::{BEFORE_RESTARTING, STILL_EVERY, THE_DISK_APPEARS_WITHIN, TheMachine};
pub use program::{EVERY_PROGRAM, Program, Ran};
#[cfg(target_os = "linux")]
pub use running::OnThisMachine;
pub use sequence::install;
pub use the_asking_walks::{TheAnswers, TheAsking, WhatThePersonDid, WhatTheRingIsOn, WhichScreen};
pub use the_environment_asks::what_the_person_did;
pub use what_a_person_typed::{Typed, WhatAPersonTyped};
pub use what_each_screen_says::{how_many_cards, what_this_screen_says};
// Gated with the item it names: it opens the framebuffer and a keyboard, and
// this crate must still cross-compile for Windows.
#[cfg(unix)]
pub use the_environment_asks::ask;
pub use the_screen_that_lists_every_layout::{ALayoutsCard, EveryLayoutThisComputerKnows};
pub use tidying::{Tidied, tidy_up};
pub use told::{
    Beside, NotTold, REPLACING, THE_CHOICE, THE_EFI, THE_PARTITION, THE_REPLACING, Told,
};
pub use verifying::{Verified, Verifying};
pub use what_display_there_is::{
    Glass, WhatDisplayThereIs, from_a_connector, is_connected, the_glass, the_mode,
};
pub use where_a_list_screen_goes::{how_many_cards_fit, where_a_list_screen_goes};
pub use where_a_screen_goes::{
    AScreensPlaces, AnArea, HowMuchThereIs, THE_DESIGN_WAS_DRAWN_AT, THE_GAP, TheType,
    where_a_screen_goes,
};
pub use which_choices_are_shown::WhereInTheList;
pub use which_disks_may_be_offered::{
    NotOffered, Offered, TheMedium, WhatToShow, what_may_be_offered,
};
pub use why_there_is_no_network::WhyThereIsNoNetwork;
pub use words::{EVERY_WORD, WordsError, declare_into, installing_words};
pub use wrapping::{ALine, how_many_lines, wrapped};
pub use writing::{A_WATCHED_LINE, THE_PERSONS_SCREEN, TheConsole, Writing};
