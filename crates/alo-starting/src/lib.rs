//! The menu a machine with two systems on it starts at, and the way out of alo
//! OS into the Windows beside it.
//!
//! A person who installs alo OS *alongside* Windows
//! ([ADR 0023](../../../docs/decisions/0023-installed-from-the-machine-it-replaces.md)
//! §4,
//! [ADR 0033](../../../docs/decisions/0033-the-certified-laptop-is-installed-the-way-a-customer-installs.md))
//! has two systems on one computer and wants to move between them without
//! learning which key their firmware hides its own menu behind.
//! [ADR 0062](../../../docs/decisions/0062-the-menu-a-machine-starts-at-is-alo-oss-and-windows-stands-behind-it.md)
//! decided how: the menu is alo OS's, Windows stands behind it in the
//! firmware's own order, and the last choice is kept in one place that both
//! sides read. This crate is that decision.
//!
//! | | |
//! |---|---|
//! | [`System`], [`THE_WINDOWS_LOADER`] | The two a machine can start, and the file Windows is started by |
//! | [`Menu`], [`THE_MENU`] | The menu, generated as configuration for the base's own loader |
//! | [`EnvironmentBlock`], [`THE_ENVIRONMENT_BLOCK`], [`THE_BLOCK_ON_THE_ESP`] | The loader's own file on the partition both systems share, read and written |
//! | [`TheStartingChoice`] | Which system starts when nobody chooses — kept there, and nowhere else |
//! | [`TheLoader`], [`TheLoadersFiles`] | Those two files as whoever changes them reaches them |
//! | [`start_by_default`], [`NotSet`] | A person's approved choice, carried out into that file |
//! | [`the_identity_of`], [`the_system_named`], [`to_start_by_default`] | Which system, named the one way both sides of the door name it |
//! | [`Entry`], [`Firmware`], [`TheFirmware`] | What the firmware reports, and the one thing it is ever told |
//! | [`Change`] | The two changes on this road, and the broker verbs they are |
//! | [`NotChanged`], [`changed_said`], [`starts_at_said`] | What a person reads, either way |
//! | [`words`] | Every sentence, with a note for whoever translates it |
//!
//! # Two changes, and they are two on purpose
//!
//! *Restart into Windows* sets the firmware's **next start**, for one start,
//! and leaves the machine's ordinary order exactly as it was. *This computer
//! starts this system when nobody chooses* changes the loader's own saved
//! default and touches the next start not at all. One is *take me across once*;
//! the other is *this is what this computer does from now on*, and a single
//! verb doing both would be a sentence a person approved that meant two things.
//!
//! Both are verbs on the broker's list, because writing a firmware variable and
//! writing a file under `/boot` are both privileged and
//! [ADR 0001](../../../docs/decisions/0001-the-capability-model.md)
//! §2 puts privilege behind the broker. What carries them out is
//! `alo_brokerd::NextStart` against [`Firmware`], and `alo_brokerd::ByDefault`
//! against [`TheLoader`]
//! ([ADR 0066](../../../docs/decisions/0066-which-system-a-machine-starts-by-default-is-changed-by-a-verb.md)).
//!
//! **And neither restarts anything.** The restart is the person's own,
//! afterwards, exactly as it is for the update verbs: nothing in this workspace
//! holds `CAP_SYS_BOOT` and nothing here asks for it.
//!
//! # Configured, never patched
//!
//! The loader is the base's
//! ([ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)).
//! [`Menu::written`] produces a whole file of ours that the base's own
//! configuration reads, and nothing here edits a line the base wrote.
//!
//! # It mounts nothing, and could not
//!
//! Fast Startup is the person's answer to a question the installer asks
//! (ADR 0064 term 9, decided 2026-09-22), and **alo OS never mounts the
//! Windows partition read-write** holds whichever they answer. This crate is where the way across
//! to Windows lives, and it reaches Windows by handing Windows's own start-up
//! program to the firmware. It depends on no disk service, names no filesystem,
//! and has no method that could mount anything —
//! `tests/windows_is_never_mounted.rs` holds all three, over this crate and
//! over the files the image ships.
//!
//! # What is deliberately not here
//!
//! **No second copy of the last choice.** It is the loader's saved default, in
//! the loader's own file, and [`TheStartingChoice`] is the only reader and the
//! only writer of it in alo OS — ADR 0062's third term, held by
//! `tests/the_last_choice_has_one_place_and_no_copy.rs`. The road a person's
//! change travels through the broker does not add one: what crosses the door is
//! thirty-two bytes naming a system, and the file is read and written on this
//! side of it.
//!
//! **No way to change the machine's start-up order**, to add a start-up entry
//! or to remove one. There is no method for any of them, which is a stronger
//! statement than a rule about not calling one.
//!
//! **No agent verb of its own.** The broker verb is the road, and it is
//! reached from an approval like the update and storage verbs, which likewise
//! have no verb of their own on the agent's list. A verb an agent proposes
//! would need a promise in `docs/features.md` with a tier to be answered
//! against (`alo-by-hand`, ADR 0009), and adding one is not this task's to
//! make.
//!
//! **Nothing here has run on a machine.** `docs/booting.md` says what is owed
//! and to whom: the walk belongs to the virtual machine on the development PC,
//! and what the base's loader does with a saved default across an update is
//! measured there rather than assumed here.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

mod by_default;
mod by_hand;
mod chosen;
mod efi_variables;
mod firmware;
mod loader;
mod menu;
mod offered;
mod on_this_machine;
mod refusing;
mod saved;
mod systems;
pub mod testing;
mod wanted;
pub mod words;

pub use by_default::{NotSet, start_by_default};
pub use by_hand::to_start_by_default;
pub use chosen::{SAVED_ENTRY, THE_BLOCK_ON_THE_ESP, THE_ENVIRONMENT_BLOCK, TheStartingChoice};
pub use efi_variables::{
    NEXT_START, THE_GLOBAL_GROUP, THE_VARIABLES, TheFirmware, the_entry_named,
    the_next_start_written,
};
pub use firmware::{AS_REPORTED, Entry, Firmware, NotAnEntry, NotAnswering, NotDone};
pub use loader::{NotRead, NotWritten, TheLoader};
pub use menu::{LONGEST_COUNTDOWN, LONGEST_TITLE, Menu, NotAMenu, THE_COUNTDOWN, THE_MENU};
pub use offered::{AS_OFFERED, the_identity_of, the_system_named};
pub use on_this_machine::TheLoadersFiles;
pub use refusing::{NotChanged, changed_said, starts_at_said};
pub use saved::{EnvironmentBlock, LENGTH, NotAnEnvironmentBlock, SIGNATURE};
pub use systems::{System, THE_WINDOWS_ENTRY, THE_WINDOWS_LOADER, THE_WINDOWS_LOADER_IN_A_PATH};
pub use wanted::Change;
pub use words::{EVERY_WORD, WordsError, declare_into, starting_words};
