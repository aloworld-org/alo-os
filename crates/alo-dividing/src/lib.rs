//! Which window has which share of a display, and how the shares hold together.
//!
//! ★ `ROADMAP.md` v0.5: *divide the screen: halves and quarters by drag or
//! keyboard, splits that hold while you work*. **The star is on hold.** Every
//! system can snap a window to half the screen; almost none keeps the two halves
//! a pair, so resizing one leaves the other where it was, overlapping or leaving
//! a gap. Here a division is a tree of shares (`node.rs`), the boundary between
//! two shares is one number, and moving it moves both.
//!
//! # What is here
//!
//! | | |
//! |---|---|
//! | [`area`] | Points, sizes and rectangles, in logical units |
//! | [`scale`] | Logical units into pixels, edge by edge, so shares still meet |
//! | [`side`] | Which way a share is cut, and which side a window goes on |
//! | [`window`] | A window as a division knows it: which one, and its minimum size |
//! | [`share`] | One window's share, laid out |
//! | `node` | The tree itself, and the only arithmetic that moves a boundary |
//! | [`division`] | One display's division: reading, halving, resizing, closing |
//! | [`dropping`] | A window dragged to an edge or corner, proposed before it is done |
//! | [`keyboard`] | The focused share divided with the next window, through `alo-shortcuts` |
//! | [`place`] | What a half, a quarter or a part is called |
//! | [`refusing`] | Why a division was left as it was |
//! | [`words`] | Every string this crate can say, and the English beside each |
//!
//! ```
//! use alo_dividing::{Area, Division, Offer, Place, Point, Side, Size, Window, WindowId};
//!
//! let display = Area::of(Point::at(0, 0), Size::of(1920, 1080))?;
//! let mut division = Division::of(display);
//! let mail = Window::any_size(WindowId::from_compositor(1));
//! let notes = Window::any_size(WindowId::from_compositor(2));
//!
//! // Mail dragged to the left edge, with notes in front: a half, proposed.
//! let Offer::Proposed(proposal) = division.propose_drop(Point::at(2, 400), mail, Some(notes))
//! else { unreachable!() };
//! assert_eq!(proposal.place(), Place::LeftHalf);
//! division.commit(proposal).map_err(|refused| format!("{refused:?}"))?;
//!
//! // The boundary dragged to 1200: both halves follow.
//! division.move_boundary(mail.id(), Side::Right, 1200).map_err(|refused| format!("{refused:?}"))?;
//! assert_eq!(division.share_of(notes.id()).map(|a| (a.x(), a.width())), Some((1200, 720)));
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Nothing here draws
//!
//! No pixels, no surfaces, no pointer. Geometry is in logical units a display's
//! scale turns into pixels ([`scale`]), so a division means the same thing on a
//! scaled screen. The compositor draws what this crate decides; the shell's
//! v0.01 `window_tiling` is an independent half-screen tile and is replaced by
//! this, not extended.
//!
//! # Nothing here is the agent's
//!
//! A division is changed by a person's hands. The agent's *arrange* verb (v0.01)
//! is planned to propose a division through these same types and be approved
//! like any change (task 2 of this crate's plan); nothing in this crate gives it
//! a road of its own, and nothing here reads a clock, a file or the network.
//!
//! # Nothing here says anything in English by itself
//!
//! What a proposal is called and why a division was refused are declared in
//! [`words`] and answered through `alo-strings`; [`Refused`] and [`Place`] have
//! `said`, not `Display`. The one exception is [`AreaError`], which is said to
//! the compositor about a number it passed.

#![doc(html_root_url = "https://github.com/aloworld-org/alo-os")]

pub mod area;
pub mod division;
pub mod dropping;
pub mod keeping;
pub mod keyboard;
mod node;
pub mod place;
pub mod refusing;
pub mod remembering;
pub mod scale;
pub mod share;
pub mod side;
pub mod window;
pub mod words;

#[cfg(test)]
mod testing;

pub use area::{Area, AreaError, LARGEST, Point, Size};
pub use division::Division;
pub use dropping::{NEAR_A_CORNER, NEAR_AN_EDGE, Offer, Proposal};
pub use keyboard::{side_bound_to, side_for};
pub use place::Place;
pub use refusing::Refused;
pub use remembering::{Divisions, HeldBy, NotRemembered, OnADisplay, Remembered};
pub use scale::{Pixels, Scale};
pub use share::Share;
pub use side::{Axis, Side};
pub use window::{Window, WindowId};
pub use words::{Word, WordsError, declare_into, dividing_words};
