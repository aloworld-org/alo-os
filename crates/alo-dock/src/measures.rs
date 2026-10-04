//! The numbers this crate is built out of, and what each of them answers to.
//!
//! **A threshold picked by eye is a threshold nobody can test.** The v0.01
//! promise in `docs/features.md` says labels give way to icons *where the short
//! edge demands it*, and the whole of the design work in this crate was turning
//! that clause into arithmetic. These are the numbers that arithmetic runs on,
//! kept in one file so that changing one is a visible act rather than a
//! discovery.
//!
//! # Two of these are picked, and the rest follow from a standard
//!
//! [`ICON`] and [`TEXT_AT_ORDINARY`] are the shell's own design: how big a thing
//! you press is, and how big the text was drawn at. Both are held to a floor
//! rather than left to taste — an icon is a target, and the floors it is held to
//! are `alo_appearance::targets`, which is where a standard's figures belong
//! because they are not this crate's design and bind every surface that draws.
//!
//! **This file holds what a Dock is built out of, and only that.** It said so
//! already and had stopped being true: a WCAG floor lived here for as long as a
//! dock icon was the first thing that needed one. That is the shape to watch —
//! a file whose header describes it correctly on the day it is written and is
//! quietly outgrown by the second thing that needs what it holds.
//!
//! [`A_DOCK_MAY_TAKE_ONE_PART_IN`] is not taste at all. It is fixed by a
//! requirement: **text reaches 200% without losing content** (EN 301 549, by way
//! of WCAG 1.4.4), on the smallest screen alo OS lays out for
//! ([`crate::Screen::the_smallest`]). It is as generous as that requirement
//! allows and no more, and `crate::layout`'s tests are what says so — loosen it
//! and a dock on a 1366×768 screen takes more of it than the person's work;
//! tighten it and the names go at exactly the size the standard says they must
//! survive.
//!
//! **`LABEL_EMS` was the second such number** — how much width a name needed
//! beside an icon on a dock down the side of the screen. ADR 0076 fixed the dock
//! along the bottom, so there is no name beside an icon and no floor to hold.
//! The share survived that removal still being the tightest one the standard
//! allows, which was the open question and is now a test.
//!
//! # The unit
//!
//! Everything here is in **logical pixels**: the units a compositor lays a
//! screen out in, before it is scaled for a dense display. A machine with twice
//! the pixels draws the same dock twice as sharp rather than half as big, so
//! nothing in this crate has to know how dense a screen is.

/// The side of an application's icon in the dock.
///
/// It does not grow with the text at v0.01. *The dock's size* is v0.5 in
/// `docs/features.md`, and a person who needs bigger icons before then has the
/// same answer as a person who needs a bigger dock: it is a setting that does
/// not exist yet, rather than one this crate guesses at.
pub const ICON: u32 = 48;

// **The standards' target floors left this file on 2026-09-30.**
// `SMALLEST_TARGET` was here because a dock icon was the first thing that
// needed it, and it was never a number this crate is built out of: it is WCAG
// 2.5.8 by way of EN 301 549, a rule about hands, and it binds a window's name
// band and a status item exactly as much as a dock icon. It is now
// `alo_appearance::targets::SMALLEST_TARGET`, beside `ENHANCED_TARGET` — WCAG
// 2.5.5's 44, which is what this product's controls are built to — in the crate
// every surface already depends on.
//
// It is deleted rather than kept as an alias here. A name that still resolves
// is a name the next person will use, and then there are two homes for one
// standard. The Mac lane found this on the day a second surface was about to
// produce a second copy of the same 24 under a different name.

/// The room between an icon and its name, and between one dock item and the
/// next.
pub const GAP: u32 = 8;

/// The room between what the dock holds and each of the dock's two faces.
pub const MARGIN: u32 = 8;

/// How big the shell's text is at 100%, which is the size it was drawn at.
pub const TEXT_AT_ORDINARY: u32 = 15;

/// A line of text is this many fifths of the text's own size — 1.4, written as
/// a fraction because this crate does no floating-point arithmetic and a layout
/// that rounded differently on two machines would be two layouts.
pub const LINE_IN_FIFTHS: u32 = 7;

/// The most of a screen's side a dock may take: one part in this many.
///
/// **A dock is on the screen all day**, so what it takes it takes from
/// everything else. One part in six is what leaves room for a name at the size
/// EN 301 549 requires the layout to survive, on the smallest screen alo OS
/// lays out for, and nothing beyond that.
pub const A_DOCK_MAY_TAKE_ONE_PART_IN: u32 = 6;

/// How far the bar floats above the bottom edge of the screen.
///
/// **The Dock is a bar with room under it, not a band stuck to the edge.** The
/// gap is what makes it read as something laid on the canvas rather than part
/// of the screen's frame, and it is the same measure as the room inside the bar
/// so that the two do not disagree by a pixel nobody chose.
pub const FLOATING_ABOVE_THE_EDGE: u32 = MARGIN;

/// The text size EN 301 549 requires a layout to survive, as a percentage.
///
/// It is `alo_appearance`'s number as well — that crate asserts its ceiling
/// reaches it — and it is repeated here because it is the thing this crate's
/// thresholds are measured against rather than a size this crate offers.
pub const THE_STANDARDS_TEXT: u16 = 200;

/// How much usable text width a name gets beside an icon on a side dock.
///
/// **The owner's figure, 2026-10-04, and not yet in the design file** — their own
/// words: *this is the implementation specification; I have not yet added that
/// measurement to Figma.* So this constant is the source, and
/// `docs/design/the-interface-in-the-file.md` should not be read as holding it.
///
/// This crate had the figure once as `LABEL_EMS` and lost it with the four-edge
/// choice; it comes back in logical pixels rather than ems because that is the
/// unit the owner gave and the unit everything else here is in.
pub const A_NAME_BESIDE_AN_ICON: u32 = 200;

/// The padding on each side of a name shown beside an icon.
///
/// So the tooltip is [`A_NAME_BESIDE_AN_ICON`] + twice this = **224** logical
/// pixels wide. Its own constant rather than a number folded into the width,
/// because *how much room the text gets* and *how much room is around it* are two
/// figures the owner gave separately and a reader may need to change apart.
pub const AROUND_A_NAME_BESIDE_AN_ICON: u32 = 12;
