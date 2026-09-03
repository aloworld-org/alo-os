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
//! rather than left to taste — an icon is a target, and EN 301 549 carries WCAG
//! 2.5.8's [`SMALLEST_TARGET`] for one.
//!
//! The other two are not taste at all. [`A_DOCK_MAY_TAKE_ONE_PART_IN`] and
//! [`LABEL_EMS`] are fixed by a requirement: **text reaches 200% without losing
//! content** (EN 301 549, by way of WCAG 1.4.4), on the smallest screen alo OS
//! lays out for ([`crate::Screen::the_smallest`]). They are as generous as that
//! requirement allows and no more, and `crate::room`'s tests are what says so —
//! loosen either one and a dock on a 1366×768 screen takes more of it than the
//! person's work; tighten either one and the names go at exactly the size the
//! standard says they must survive.
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

/// The smallest a thing a person presses may be, from WCAG 2.5.8 by way of
/// EN 301 549 — the standard an EU public-sector desktop is procured against.
///
/// [`ICON`] is held to it by a test rather than by a comment, which is the same
/// shape `alo_appearance::TextScale` holds its 200% in.
pub const SMALLEST_TARGET: u32 = 24;

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

/// How much room a name needs beside an icon, counted in **ems** — multiples of
/// the text's own size.
///
/// **This crate cannot measure text**, and an em is the one unit that does not
/// need it to: it is the text's own size, so it grows with the text without a
/// font in the room. Five of them is a floor on *room*, never a promise about a
/// particular name — a name too long for the room it is given is elided by
/// whoever draws it, and a name with five ems in front of it is still a name
/// somebody recognises. Below that it is a fragment, and a fragment that starts
/// two application names is worse than the icon on its own.
pub const LABEL_EMS: u32 = 5;

/// The most of a screen's side a dock may take: one part in this many.
///
/// **A dock is on the screen all day**, so what it takes it takes from
/// everything else. One part in six is what leaves room for a name at the size
/// EN 301 549 requires the layout to survive, on the smallest screen alo OS
/// lays out for, and nothing beyond that.
pub const A_DOCK_MAY_TAKE_ONE_PART_IN: u32 = 6;

/// The text size EN 301 549 requires a layout to survive, as a percentage.
///
/// It is `alo_appearance`'s number as well — that crate asserts its ceiling
/// reaches it — and it is repeated here because it is the thing this crate's
/// thresholds are measured against rather than a size this crate offers.
pub const THE_STANDARDS_TEXT: u16 = 200;
