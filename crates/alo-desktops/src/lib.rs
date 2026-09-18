//! A person's desktops: which windows are on each, what is on all of them, and
//! how a person moves between them.
//!
//! `ROADMAP.md` v0.5 and `docs/features.md`: *virtual desktops*, and *touchpad
//! gestures: scroll, zoom, swipe between workspaces*. This crate is the part
//! that decides; nothing here draws.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`display`] | Which display a row of desktops belongs to |
//! | [`desktops`] | Every display's desktops, and the person's chords |
//! | [`on_a_display`] | One display's row: add, remove, name, reorder, switch |
//! | [`desktop`] | One desktop: its name, its windows and its division |
//! | [`naming`] | What a person calls a desktop, and what is not a name |
//! | [`position`] | Where a desktop sits in the order, counted from one |
//! | [`switching`] | The closed set of ways a person asks for another desktop |
//! | [`chords`] | Those ways from the keyboard, without taking a chord from `alo-shortcuts` |
//! | [`always`] | The three things that are on every desktop and cannot be moved |
//! | [`refusing`] | Why nothing changed |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_desktops::{Desktops, DisplayId, Promises, Switch};
//! use alo_dividing::{Area, Point, Size, WindowId};
//!
//! let area = Area::of(Point::at(0, 0), Size::of(1920, 1080))?;
//! let laptop = DisplayId::from_compositor(1);
//! let promises = Promises::of(
//!     WindowId::from_compositor(9001),
//!     WindowId::from_compositor(9002),
//!     WindowId::from_compositor(9003),
//! )?;
//!
//! let mut desktops = Desktops::default();
//! let on_laptop = desktops.plug_in(laptop, area, promises)?;
//!
//! // A display starts with one desktop. A person adds a second and swipes to it.
//! let first = on_laptop.current();
//! let second = on_laptop.add().map_err(|refused| format!("{refused:?}"))?;
//! on_laptop.put_on(first, WindowId::from_compositor(1)).map_err(|r| format!("{r:?}"))?;
//! assert_eq!(on_laptop.switch(Switch::Next), Ok(second));
//!
//! // Removing the desktop the windows are on moves them, and closes nothing.
//! let took = on_laptop.remove(first).map_err(|refused| format!("{refused:?}"))?;
//! assert_eq!(took, second);
//! assert_eq!(on_laptop.where_is(WindowId::from_compositor(1)), Some(second));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # The three promises are on every desktop at once
//!
//! The egress indicator, the approval surface and the agent overlay. A display's
//! desktops cannot be made without all three, and every road that could move a
//! window refuses one of them: a promise a person could only see on desktop 1
//! would be no promise at all. [`always`] is the argument, and
//! `tests/every_promise_is_on_every_desktop.rs` is the test.
//!
//! # Nothing here draws
//!
//! No pixels, no surfaces, no pointer, no gesture events. Each desktop keeps its
//! own `alo_dividing::Division`, in logical units, and this crate never does that
//! arithmetic itself. The compositor draws the desktop a person is on; task 5 of
//! this crate's plan turns a swipe into a [`Switch`] this crate carries out.
//!
//! # Nothing here is the agent's
//!
//! A desktop is added, named, reordered and switched by a person's hands. The
//! agent's *arrange* verb (v0.01) proposes a division and is approved like any
//! change; nothing in this crate gives it a road, and nothing here reads a
//! clock, a file or the network.
//!
//! # Nothing here says anything in English by itself
//!
//! What a desktop is called, what a switch asks for and why nothing changed are
//! declared in [`words`] and answered through `alo-strings`. [`Refused`],
//! [`NameError`] and [`Always`] have `said`, not `Display`. The two exceptions
//! are [`NotADisplay`] and [`TwoPromises`], which mean the shell and this crate
//! disagree about what exists — alo OS's own bug, with nothing to ask a person.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod always;
pub mod chords;
pub mod desktop;
pub mod desktops;
pub mod display;
pub mod naming;
pub mod on_a_display;
pub mod position;
pub mod refusing;
pub mod switching;
pub mod words;

#[cfg(test)]
mod testing;

pub use always::{Always, Promises, TwoPromises};
pub use chords::DesktopChords;
pub use desktop::{Desktop, DesktopId};
pub use desktops::Desktops;
pub use display::{DisplayId, NotADisplay};
pub use naming::{MOST_CHARACTERS, Name, NameError};
pub use on_a_display::{MOST_DESKTOPS, OnADisplay};
pub use position::Position;
pub use refusing::Refused;
pub use switching::Switch;
pub use words::{Word, WordsError, declare_into, desktop_words};
