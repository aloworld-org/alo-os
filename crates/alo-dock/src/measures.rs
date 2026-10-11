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
/// **Twenty-eight inside forty-eight, by [ADR
/// 0101](../../../docs/decisions/0101-one-dock-component-family-and-the-renderer-that-can-draw-it.md)**:
/// *application artwork: 28 × 28, centred. This supersedes the earlier 32px
/// artwork proposal.*
///
/// It was 32 from 2026-10-09, which the owner gave as a **proposal** and said
/// so. The canonical design file draws 28 in every `Dock edges / Button / *`,
/// and the edge family is the baseline the same ruling chose.
///
/// # Four numbers, and the ruling says not to confuse them
///
/// The owner's sentence, which is the one most likely to be lost:
///
/// > The 52 × 48 bottom measurement may include spacing around a target;
/// > distinguish that from the target itself. Do not silently treat all these
/// > measurements as interchangeable.
///
/// So there are four, and this file keeps them apart by name:
///
/// | | | |
/// |---|---|---|
/// | the **target** | what a hand must hit | [`ICON`], 48 |
/// | the **artwork** | what an eye reads | this, 28 |
/// | the **quiet** | what is left over, each side | [`AROUND_THE_ARTWORK`], 10 |
/// | the **pitch** | one target to the next | [`ICON`] + [`GAP`] |
///
/// A fifth — the 52 in the bottom Dock's `52 × 48` slot — is spacing a layout
/// put around a target and is **not** any of these. This repository has already
/// published one contradiction by comparing two of the four; the cure is that
/// each has a name and none is derived from another by coincidence.
///
/// # No shrinking, and the owner's reason is the whole of it
///
/// The Figma implementation contract said *icons may reduce from 32 to 28*.
/// **The owner revised that**, in these words: *smaller artwork alone does not
/// create more usable space*. So when the Dock runs out of edge it uses
/// overflow rather than shaving the artwork — and 28 is now the **one** size
/// rather than the bottom of a range.
///
/// **And no hover enlargement**, by the same decision. §10 of the Dock's
/// specification already forbids magnification that pushes neighbours sideways
/// — *the person should be able to aim once and click.*
///
/// # What a person may change, and when
///
/// Icon size becomes *an explicit personal setting* — the owner's words — and
/// that belongs with **`[v0.5]` The dock's size, and whether it hides when a
/// window needs the room** in `docs/features.md`. Until then this is fixed,
/// which is a number nobody has been offered rather than a choice withdrawn.
pub const GLYPH: u32 = 28;

/// What is left over around the artwork, on each side of it.
///
/// **Derived, and named rather than left implicit**, because the three numbers
/// it sits between are each fixed by something different: the target by the
/// standard a hand is held to, the artwork by the design file, and this by
/// arithmetic. It was eight — the spacing unit — while the artwork was 32, and
/// that coincidence is what let a reader believe the three were one system.
/// They are not, and ADR 0101 says so.
pub const AROUND_THE_ARTWORK: u32 = (ICON - GLYPH) / 2;

/// How wide the mark under an application that has a window open is.
///
/// Measured at **4 × 3** on `Open window indicator` — `348:23886` on the top
/// Dock and `348:23686` on the left one, identically.
pub const AN_OPEN_MARK_IS_WIDE: u32 = 4;

/// How wide the mark under the application a person is working in is.
///
/// Measured at **12 × 3** on `Active window indicator` — `348:23880` and
/// `348:23680`.
///
/// **Three states told apart by width and not by colour**, which is what
/// `docs/design/the-alo-dock.md` has promised all along: focused is this, open
/// is [`AN_OPEN_MARK_IS_WIDE`], and pinned-but-not-open is nothing at all. The
/// design file's own words for the component: *open indicators distinguish
/// pinned from open.*
pub const A_FOCUSED_MARK_IS_WIDE: u32 = 12;

/// How tall either mark is.
pub const A_RUNNING_MARK_IS_TALL: u32 = 3;

/// The radius of the Dock's own corners.
///
/// `radius/lg` in the design file, measured on `Edge=Top, State=Overflow`
/// (`348:23874`) and ruled for every edge by ADR 0101. The bottom Dock's older
/// component draws 16; the ruling normalises both to this.
pub const THE_DOCKS_CORNER: u32 = 18;

/// How thick the hairline around the Dock is.
///
/// One logical pixel, in `border/default`. A *measure*, not a colour: which
/// colour it is belongs to `alo_appearance`, like every other colour this
/// product draws.
pub const THE_DOCKS_EDGE: u32 = 1;

/// How far the Dock's shadow is offset, and how far it spreads.
///
/// `Overlay / Glass` in the design file: `0 8px 24px rgba(7,19,31,0.08)`. Held
/// as two measures and a share rather than a string, because a shadow is drawn
/// and not written — and because the opacity is not this crate's to hold, for
/// the reason [`THE_DOCKS_EDGE`] gives about colour.
pub const THE_SHADOW_FALLS: u32 = 8;

/// How far the Dock's shadow spreads from its edge.
pub const THE_SHADOW_SPREADS: u32 = 24;

/// How much the canvas behind the Dock is blurred.
///
/// Eighteen, by ADR 0101 — and **with a designed opaque alternative when a
/// person has asked for reduced transparency.** That alternative is not a
/// fallback for a hard problem: it is a state the owner named, and it ships
/// whatever the blur costs.
pub const BEHIND_THE_DOCK_IS_BLURRED: u32 = 18;

/// How wide the composer — *Find, open, or ask…* — is at the reference size.
///
/// Measured at **344 × 48** on both Dock families (`349:25790` and
/// `I321:17430;74:53`), which is the one thing they already agreed on, and
/// preserved by ADR 0101.
pub const THE_COMPOSER_IS_WIDE: u32 = 344;

/// How tall the composer is: the same as a target, so it sits on their line.
pub const THE_COMPOSER_IS_TALL: u32 = ICON;

/// How big the key shown at the composer's end is drawn.
///
/// `Label/Small` in the design file: **Manrope SemiBold 11**, letter-spacing
/// 0.3 — measured on `Bar shortcut` (`I321:17430;74:55`) and `349:25792`.
pub const A_SMALL_LABEL_IS: u32 = 11;

/// How big a command's label is drawn.
///
/// `Label/Medium` in the design file: **Manrope SemiBold 13**, letter-spacing
/// 0.1 — measured on `Canvas zoom menu` and `Canvas options menu`. The face and
/// the weight are `crate::painted_text`'s already; this is the size.
pub const A_LABEL_IS: u32 = 13;

/// How wide the *Show all* control on the bottom Dock is.
///
/// Measured at **88 × 44** on `Canvas zoom menu` (`I321:17430;100:3008`). Its
/// height is a target's, by ADR 0101's normalisation; only its width is its
/// own, because it holds a word rather than a mark.
pub const SHOW_ALL_IS_WIDE: u32 = 88;

/// How long the rule between the applications and the commands is.
///
/// Measured at **1 × 28** on the top Dock's `Apps / commands divider`
/// (`348:23930`) and **28 × 1** on the left one's (`348:23730`) — the same rule
/// turned to suit the bar. So this is its **long** side; its short side is
/// [`THE_DOCKS_EDGE`], because a divider and a hairline are one line's worth of
/// the same token.
pub const THE_COMMANDS_DIVIDER_IS_LONG: u32 = 28;

/// The radius of the composer's own corners — `radius/md` in the design file.
pub const THE_COMPOSERS_CORNER: u32 = 12;

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

/// How big the initial in a fallback badge is drawn.
///
/// **The owner's ruling of 2026-10-11**, for an application with no usable
/// icon: *circle: 28 × 28 logical pixels, centred within the existing 48 × 48
/// hit target. Initial: Manrope SemiBold, 14px, optically centred.*
///
/// The circle is [`GLYPH`] — the badge **is** the artwork box, so a real icon
/// replaces a fallback without the slot moving or resizing, which is the last
/// line of that ruling. This is the letter inside it, and it is half the badge
/// rather than a number of its own: *half* is what makes a single initial read
/// as a mark rather than as cramped text.
///
/// **Manrope SemiBold is not repeated here.** `crate::painted_text`'s
/// `alos_own_text` is already that face at weight 600, in one place so that a
/// measurement and a drawing cannot disagree about which face they meant.
pub const AN_INITIAL_IS: u32 = 14;

/// The side of a utility symbol drawn inside [`ICON`] — search, overflow.
///
/// **Not [`GLYPH`], and the owner said so when they gave both.** Their table of
/// 2026-10-09 lists *application icon 32 × 32* and, on its own row, *utility
/// glyphs — search, overflow — 20 to 24, in the same 48 target*. A magnifying
/// glass is not an application: it is a mark that means an action, and it reads
/// at a smaller size than artwork meant to be recognised across a room.
///
/// **Twenty-four, the top of the range they gave**, because this is a target a
/// hand aims at as much as a mark an eye finds, and the larger end of a range
/// given for legibility is the end to take when nothing argues for the other.
/// The range is recorded here rather than the choice alone so that a later
/// change knows what it is moving inside.
pub const A_UTILITY_GLYPH: u32 = 24;

/// The room between an icon and its name, and between one dock item and the
/// next.
pub const GAP: u32 = 8;

/// The room between what the dock holds and each of the dock's two faces.
pub const MARGIN: u32 = 8;

/// How wide *More apps* is.
///
/// Measured off `More apps / Popover` — `406:26881`, read from the live file on
/// 2026-10-11 and captured in `docs/design/figma-snapshot/406-26881.xml`. Every
/// one of its six states is 448 wide.
///
/// **This replaced 248, which was a different surface.** That figure came from
/// `More apps / opens inward`: a narrow list of the applications that
/// overflowed. The owner's ruling of 2026-10-11 (ADR 0103) made this an
/// overview of every pinned and open application, drawn as a grid with a search
/// field, and gave it a design of its own. The old frame is not an earlier
/// version of this one and nothing may be measured off it.
pub const MORE_APPS_IS_WIDE: u32 = 448;

/// The most *More apps* may be, top to bottom, heading and footer included.
///
/// **A cap, not a height.** Stated by the owner on 2026-10-11, separately from
/// the design file: *these are caps, not heights that increase with application
/// count. Sixty applications must scroll inside the same bounded viewport.*
///
/// Every state in the design is drawn at this number, which is exactly why the
/// file cannot show that it is a maximum — so it is said here instead.
pub const MORE_APPS_IS_AT_MOST_TALL: u32 = 536;

/// The most the scrolling list of applications inside it may be.
///
/// The other half of the same ruling, and the one that does the work: the
/// heading, the search field and the footer are fixed, and **only this
/// scrolls**. On an output too short for the whole popover this shrinks further
/// so that the rest still fits.
pub const THE_APP_LIST_IS_AT_MOST_TALL: u32 = 336;

/// Between a list the Dock opens and the bar it opens from.
///
/// Measured off `01 · More apps / Bottom · With minimised windows`
/// (`408:28177`): the popover sits at y 312 and is 536 tall, so its lower edge
/// is at 848; `Dock / Bottom` begins at 860. **Twelve between them**, which is
/// also what the owner states in words — *keep a 12px gap from the Dock*.
///
/// It is not [`GAP`]. That is 8 and is the room between things **inside** one
/// surface; this is the room between two surfaces, and they were the same
/// number by accident until the popover was measured.
pub const BESIDE_THE_DOCK: u32 = 12;

/// The room inside *More apps*, on every side.
///
/// Measured: the first child sits at 25 in a panel with a 1-wide border, so the
/// padding inside the border is 24 — which is `space/6` in the design's own
/// tokens.
pub const INSIDE_MORE_APPS: u32 = 24;

/// Between the heading, the search field, the list and the footer.
///
/// Measured: 73 − 57, 137 − 121 and 489 − 473 are each 16, which is `space/4`.
pub const DOWN_MORE_APPS: u32 = 16;

/// How tall the heading of *More apps* is.
///
/// Measured: `Heading`, 400 × **32**, holding *More apps* at 22/30.
pub const THE_MORE_APPS_HEADING_IS_TALL: u32 = 32;

/// How tall its search field is.
///
/// Measured: `Search pinned and open applications`, 400 × **48** — the same 48
/// every control on the Dock is built to.
pub const THE_SEARCH_FIELD_IS_TALL: u32 = 48;

/// How tall the line of words under the list is.
///
/// **22, which is its box and not its ink.** The capture reports the text as
/// 207 × 16 — that is the glyphs at 11/16. The box it sits in is what the
/// layout uses, and the frame gives it exactly: the footer starts at 489, the
/// popover is 536, and the bottom border and padding are 25, so 536 − 25 − 489
/// is 22.
///
/// Worth the four lines because taking the 16 makes the whole popover 530 and
/// the owner's cap is 536, and a reader checking the sum would find it 6 short
/// with every individual number right.
pub const THE_MORE_APPS_FOOTER_IS_TALL: u32 = 22;

/// How tall a window's row in the picker is.
///
/// Measured: `Window / Launch website`, 400 × **106**, with 8 between rows.
pub const A_WINDOW_ROW_IS_TALL: u32 = 106;

/// How tall the picker's heading is, which is taller than the overview's.
///
/// Measured: `Heading`, 400 × **48** — it holds a 48 × 48 back control beside
/// the title, where the overview's holds only a line of text.
pub const THE_PICKERS_HEADING_IS_TALL: u32 = 48;

/// How tall the line saying how many windows there are is.
///
/// Measured: `Window count and selection instruction`, 400 × **48**.
pub const THE_WINDOW_COUNT_IS_TALL: u32 = 48;

/// How wide one application's cell in the grid is.
///
/// Measured: `More apps / Application`, **94** × 88, and the component's own
/// description says so — *94×88 grid cell; 48×48 icon area; 28×28 artwork.*
pub const AN_APP_CELL_IS_WIDE: u32 = 94;

/// How tall one is.
pub const AN_APP_CELL_IS_TALL: u32 = 88;

/// Between cells, between rows, and between a group's heading and its rows.
///
/// Measured: the cells sit at 0, 102, 204 and 306, so 8 between them; the rows
/// at 26 and 122 under an 18-high heading, so 8 again. One number because the
/// design uses one — `space/2`.
pub const BETWEEN_APP_CELLS: u32 = 8;

/// How tall a group's heading is — *Pinned*, *Other open apps*.
///
/// Measured: 44 × **18** and 104 × **18**, at 13/18.
pub const AN_APP_GROUP_HEADING_IS_TALL: u32 = 18;

/// The room either side of a name in a list the Dock opens.
///
/// Measured off the old `More apps / opens inward`: the rows sit at x 18 and
/// the divider at x 17, in a panel 248 wide. Eighteen for both sides,
/// symmetrised because the file's own two numbers differ by one.
///
/// **Still used, and no longer by *More apps*.** The window chooser is a list
/// of rows and this is its inset; the overview is a grid and uses
/// [`INSIDE_MORE_APPS`].
pub const BESIDE_A_NAME_IN_A_LIST: u32 = 18;

/// How thick a rule under a heading is.
///
/// Measured: `Divider`, 214 × **1**.
pub const A_DIVIDER: u32 = 1;

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

/// The artwork fits inside the target, centred, with the quiet it leaves.
///
/// **This asserted `(ICON - GLYPH) / 2 == MARGIN && == GAP` until ADR 0101**,
/// which was true while the artwork was 32 and was a **coincidence** rather
/// than a system: one spacing unit happened to be what 48 less 32, halved,
/// came to. The owner's ruling moved the artwork to 28 and the coincidence
/// broke, which is the right way for a false relationship to be found.
///
/// What is held now is what is actually true: the artwork is smaller than the
/// target, it is centred in it, and the quiet is even. The quiet's **value** is
/// `AROUND_THE_ARTWORK`'s own business and is deliberately not compared to a
/// spacing unit here.
const _: () = assert!(
    GLYPH < ICON && (ICON - GLYPH).is_multiple_of(2) && AROUND_THE_ARTWORK * 2 + GLYPH == ICON,
    "the artwork no longer sits centred inside its target with even quiet on each side"
);

/// A Dock holding one application is 64 logical pixels thick./// A Dock holding one application is 64 logical pixels thick.
///
/// The owner gave 64 as its own figure as well as the three it is built from,
/// so both are held here and a change to a spacing unit says what it costs.
const _: () = assert!(
    MARGIN + ICON + MARGIN == 64,
    "the Dock is no longer 64 logical pixels thick"
);

/// The initial is half its badge, and the badge is the artwork box.
///
/// Both halves of the owner's ruling of 2026-10-11, held at compile time: a
/// change to [`GLYPH`] that forgot this would give a badge the initial no
/// longer suits, and a badge that is not [`GLYPH`] would move the slot when a
/// real icon arrived.
const _: () = assert!(
    AN_INITIAL_IS * 2 == GLYPH,
    "the initial in a fallback badge is no longer half the badge it sits in"
);

/// A utility symbol is inside the owner's range and inside the target.
///
/// Twenty to twenty-four, *in the same 48 target* — so it is smaller than the
/// artwork an application gets and smaller still than the slot both sit in.
const _: () = assert!(
    A_UTILITY_GLYPH >= 20 && A_UTILITY_GLYPH <= 24 && A_UTILITY_GLYPH < GLYPH,
    "a utility symbol is outside the 20-to-24 the owner gave, or is no longer smaller than an \
     application's artwork"
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
