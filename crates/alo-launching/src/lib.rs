//! **How a person's press starts an installed application.**
//!
//! `docs/features.md` promises *Launcher and window management: open, focus,
//! close, tile*, and the alo Dock *shows what you can open*. Until this crate,
//! a press reached an application only by `Activate` on its own bus name
//! (`alo_portals::activated`), which only an application whose entry says
//! `DBusActivatable=true` answers. Firefox, Dolphin and LibreOffice do not, so
//! a fresh machine listed them on the Dock and in the alo Bar and **refused to
//! start them**.
//!
//! This is the ordinary freedesktop road, the one every Linux desktop takes:
//!
//! - an entry with `DBusActivatable=true` is started **over the bus**;
//! - any other is started from **its own `Exec` line**, with the field codes
//!   handled as the Desktop Entry specification says ([`the_command`]);
//! - and what runs is put in **a unit of its own** in the person's own systemd
//!   manager, named by the XDG convention ([`the_unit`]), handed the
//!   compositor's socket ([`the_environment`]) — so it is in the person's
//!   session, draws on alo's display, and is not a child of the desktop.
//!
//! # Why this is permitted, and by whom
//!
//! Law 2 binds **the agent, never the person**. A person pressing an icon on
//! their own Dock, or pressing Enter on an application in the alo Bar, *is*
//! the choice the law requires. And what runs is never text a caller supplied:
//! it is the `Exec` line of a desktop entry already installed on the machine,
//! found by its identifier, the same line every other desktop would run. There
//! is no argument a caller can add.
//!
//! **The agent's approved `open` takes this road too**, by the owner's
//! decision of 2026-10-31 (ADR 0108): after the person's approval and their
//! grant for that application, alo starts it exactly as a press would. It is
//! still only an installed application, by its identifier, from its entry's own
//! `Exec` — and `alo-agentd` reads the same decisions from this crate, so the
//! record of that `open` names the road. Until that day the agent's ask was
//! bus activation only.
//!
//! # Nothing here touches a bus or starts a process
//!
//! Every function is a decision over text and paths. The calls that act on the
//! decisions — `StartTransientUnit` and `Activate` — are `alo-desktop`'s, so
//! each decision is tested here without a session, which a test against a real
//! manager could not do on a gate machine.

pub mod how_it_starts;
pub mod the_command;
pub mod the_entry;
pub mod the_environment;
pub mod the_unit;
pub mod where_it_is;

pub use how_it_starts::{HowItStarts, WhyItCannotStart, how_it_starts};
pub use the_command::{Filling, WhyNotACommand, the_command_in};
pub use the_entry::TheEntry;
pub use the_environment::{what_an_application_is_handed, what_the_session_is_told};
pub use the_unit::the_unit_for;
pub use where_it_is::{the_program, where_the_entry_is};
