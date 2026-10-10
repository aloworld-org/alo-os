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
//! **`A_DOCK_MAY_TAKE_ONE_PART_IN` was the example this paragraph was written
//! about, and it is gone.** It was not taste: it was fixed by a requirement —
//! *text reaches 200% without losing content* (EN 301 549, by way of
//! WCAG 1.4.4) — because one part tighter and the row of names under the icons
//! would have gone at exactly the size the standard says they must survive.
//!
//! The owner removed that row on 2026-10-10, so nothing was left for the share
//! to be tight against; and because the shortest side a screen may have was
//! computed from it, the share had quietly become the thing that decided
//! **which displays this product supports**. Measuring the bar at 76 moved the
//! floor from 384 to 456 and dropped a band of displays nobody had decided to
//! drop. The ruling that followed separated the two, and
//! [`THE_LEAST_A_SIDE_CAN_BE`] is a stated number now.
//!
//! **The lesson the paragraph was making is intact and is worth more than the
//! example was**: a number fixed by a requirement is not taste, and a number
//! that quietly acquires a second job — here, deciding display eligibility —
//! has stopped being fixed by the requirement that justified it.
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

/// How much of a horizontal dock's thickness sits above and below the icon.
///
/// **Measured off the design, and it is not [`MARGIN`].** The snapshot's
/// `Dock + alo Bar` is **76** tall with the `Browser · focus hit area` 48 tall
/// inside it — 14 above and 14 below. `MARGIN` is 8 and is the room at the
/// bar's *ends*, along the edge it runs on; this is the room across it. The two
/// were one number while the thickness was `MARGIN + ICON + MARGIN` = 64, and
/// that is what made 64 look measured when it was proposed.
///
/// **The owner's ruling of 2026-10-10**, after this file's own table and the
/// Figma and the code gave three different heights:
///
/// > Remove the unused label row and use the measured 76px horizontal Dock. My
/// > earlier 64px proposal is superseded by this decision. … Dock height: 76
/// > logical pixels, with the measured 48px application target centred
/// > vertically: 14px above and below.
pub const ABOVE_AND_BELOW_AN_ICON: u32 = 14;

/// The side of the artwork drawn inside [`ICON`].
///
/// **Thirty-two inside forty-eight, settled by the owner on 2026-10-09**:
/// *I recommend 32px icons inside 48px clickable areas, with no automatic
/// shrinking or hover enlargement.*
///
/// # Why this is a second number and not a smaller [`ICON`]
///
/// [`ICON`] is the **clickable area** — what a hand has to hit — and this is
/// the **artwork** inside it. Keeping them apart is what lets the target stay
/// 48 while the drawing is 32, with eight logical pixels of quiet on every
/// side. Collapsing them into one number is the mistake this file has already
/// watched twice in one afternoon: a drawn extent compared against a hit
/// target, and a glyph compared against a cell, each nearly published as a
/// contradiction that was not there.
///
/// # No shrinking, and the owner's reason is the whole of it
///
/// The Figma implementation contract said *icons may reduce from 32 to 28*.
/// **The owner revised their own earlier suggestion**, in these words:
/// *smaller artwork alone does not create more usable space*. So when the Dock
/// runs out of edge it uses overflow rather than shaving the artwork, because
/// a 28-pixel glyph in a 48-pixel target buys four pixels of room and costs
/// legibility for the person who could least spare it.
///
/// **And no hover enlargement**, by the same decision. §10 of the Dock's
/// specification already forbids magnification that pushes neighbours sideways
/// — *the person should be able to aim once and click* — and a fixed size is
/// how that is kept rather than tuned.
///
/// # What a person may change, and when
///
/// Icon size becomes *an explicit personal setting* — the owner's words — and
/// that belongs with **`[v0.5]` The dock's size, and whether it hides when a
/// window needs the room** in `docs/features.md`. Until then this is fixed,
/// which is a number nobody has been offered rather than a choice withdrawn.
///
/// Utility symbols — search, overflow — are 20 to 24 by the same decision, in
/// the same 48 target. They are not this constant: a magnifying glass is not
/// an application.
pub const GLYPH: u32 = 32;

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
/// The shortest a screen's side may be and still be laid out for.
///
/// **A stated floor, not a derived one, and that is the whole point of it.** It
/// was `A_DOCK_MAY_TAKE_ONE_PART_IN * a_dock_of_icons()` — one part in six of
/// the screen, times the dock's thickness — which quietly made *how thick the
/// Dock is* decide *which displays this product supports*. Measuring the bar at
/// 76 instead of 64 on 2026-10-10 therefore moved the floor from 384 to 456 and
/// dropped a band of displays nobody had decided to drop.
///
/// The owner's ruling of that day:
///
/// > Separate Dock sizing from display eligibility. Keep the measured 76px
/// > horizontal bar, but do not reject a previously supported display simply
/// > because its height is less than six times the Dock's thickness. Remove
/// > that coupling and retain coverage for the previously accepted smaller
/// > displays.
///
/// **384 is what was accepted before**, kept so that no display this product
/// has laid out for stops being one. It is a compatibility floor rather than an
/// arithmetic result: raising the bar's thickness must never move it again, and
/// a change to *this* number is a decision about which machines alo OS runs on.
///
/// The ratio it replaced is gone. Its own note justified it as *what leaves
/// room for a name at the size EN 301 549 requires the layout to survive* — and
/// the row of names it left room for was removed the same day, so nothing
/// documented was left for it to serve.
pub const THE_LEAST_A_SIDE_CAN_BE: u32 = 384;

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

/// How wide the lane is that a dock down either side of the screen occupies.
///
/// **Measured off the design file's frames on 2026-10-10**, not off its prose:
/// `Dock / Left` is `x=24` and **70** wide, `Dock / Right` is `x=1346` and 70 wide,
/// and 1416 is 1440 − 24 — so the 24 is a margin from the screen's edge and this is
/// the lane itself. `docs/design/the-alo-dock.md` carries the reading.
///
/// **It is not [`MARGIN`] + [`ICON`] + [`MARGIN`], and that is the point of it
/// being here.** That sum is 64; the frames say 70, six pixels and three a side,
/// and nothing in this file accounts for them. Deriving 70 out of the other three
/// would mean moving `MARGIN` for every edge so that one edge came out right, which
/// is fitting the measures to a frame — and the bottom dock's thickness is held to
/// this same design file elsewhere and would move with it.
///
/// So it is a measured figure with its provenance rather than an arithmetic one,
/// and what the six pixels are *for* is written down as an open question rather
/// than guessed at.
pub const A_SIDE_DOCKS_LANE: u32 = 70;

/// The padding on each side of a name shown beside an icon.
///
/// So the tooltip is [`A_NAME_BESIDE_AN_ICON`] + twice this = **224** logical
/// pixels wide. Its own constant rather than a number folded into the width,
/// because *how much room the text gets* and *how much room is around it* are two
/// figures the owner gave separately and a reader may need to change apart.
pub const AROUND_A_NAME_BESIDE_AN_ICON: u32 = 12;

// **The owner's sizes, held at compile time rather than by a test.**
//
// Settled 2026-10-09: *32px icons inside 48px clickable areas, with no
// automatic shrinking or hover enlargement*, and *horizontal Dock height 64px,
// vertical Dock width 64px*.
//
// These are relationships between constants, so a runtime test would assert
// something the compiler already knows — which is what clippy says when it
// refuses `assert!` on two consts. A `const` assertion fails the **build**
// instead, so a spacing unit cannot be changed into an inconsistent set and
// land anywhere.

/// The artwork fits inside the target with one spacing unit on every side.
///
/// Eight logical pixels, which is also [`MARGIN`] and [`GAP`]: one unit, used
/// three ways. If somebody changes any of the four, this names which.
const _: () = assert!(
    GLYPH < ICON && (ICON - GLYPH) / 2 == MARGIN && (ICON - GLYPH) / 2 == GAP,
    "the owner's sizes no longer agree: the artwork must sit inside its target \
     with one spacing unit of quiet on each side"
);

/// A Dock holding one application is 64 logical pixels thick.
///
/// The owner gave 64 as its own figure as well as the three it is built from,
/// so both are held here and a change to a spacing unit says what it costs.
const _: () = assert!(
    MARGIN + ICON + MARGIN == 64,
    "the Dock is no longer 64 logical pixels thick"
);

/// The target is at or above the standard's enhanced floor.
///
/// WCAG 2.5.5's 44, which `alo_appearance::targets::ENHANCED_TARGET` holds for
/// every surface. 48 clears it by four, and §8 of the Dock's specification says
/// larger targets are valid — so this holds the direction, not the equality.
const _: () = assert!(
    ICON >= alo_appearance::targets::ENHANCED_TARGET,
    "the Dock's target is below the floor every other control is built to"
);
