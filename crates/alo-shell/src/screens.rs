//! Several screens at once: which outputs there are, where each one is drawn,
//! how large, and what each wears.
//!
//! # Every decision here is `alo-displays`'
//!
//! Which screen is which, where each sits, how large it draws, which is the
//! main one, and where the windows of a screen that was unplugged belong are
//! all `alo_displays::Attached`'s answers. This file asks them once and keeps
//! the answers in the shape a compositor draws from; it arranges nothing, moves
//! nothing and rounds nothing. `tests/screens_source.rs` reads these files and
//! refuses a road to an `Arrangement` or a `Placed` made here.
//!
//! What each screen wears — the background behind its windows, the edge its
//! dock sits on, and how warm it is drawn — is `alo_displays::Wearing`, which
//! is itself `alo-appearance`'s background, `alo-dock`'s edge and night light's
//! warmth for that one screen. Night light is one decision for the machine and
//! is applied **per screen**, beside that screen's own background, because a
//! single tinted sheet over the whole desk would have to decide what to do
//! about a screen plugged in halfway through the evening.
//!
//! # Where a screen is, and how large
//!
//! [`ScreenPlace::at`] is the screen's corner on the desk the arrangement
//! makes, in the arrangement's own units, and [`ScreenPlace::room`] is what a
//! surface has to lay itself out in: the screen's pixels at the size it is
//! **drawn** at, which is the size it was given unless the machine cannot draw
//! that one and `alo-displays` rounded it. Laying a dock out for the size a
//! screen was given rather than the one it draws at is the bug that puts a
//! dock half off a 4K panel.

use alo_appearance::{Appearance, DisplayId};
use alo_displays::{
    Arrangement, Attached, CameBack, Changes, Identity, Moved, NotAttached, Note, Position,
    Reported, Resolution, Scale, Socket, Tonight, Warming, Wearing,
};

/// One screen as the compositor draws on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScreenPlace {
    /// Which screen this is tomorrow.
    identity: Identity,
    /// The name the shell and `alo-appearance` know it by.
    name: DisplayId,
    /// Its corner on the desk the arrangement makes.
    at: Position,
    /// How many pixels it has.
    pixels: Resolution,
    /// What a surface on it has to lay itself out in: its pixels at the size it
    /// is drawn at.
    room: TheRoom,
    /// The size it is drawn at, which is the size it was given unless the
    /// machine cannot draw that one.
    scale: Scale,
    /// Whether a new window opens here.
    is_main: bool,
    /// Its background, its dock's edge and how warm it is drawn.
    wearing: Wearing,
}

/// A screen's room: **its pixels divided by the scale it is drawn at, once.**
///
/// # Why this is a type and not a pair of numbers
///
/// `TheRoom::from_pixels` is the one place in this crate where a display's
/// scale is divided out. Until 2026-10-02 it was a free function, `room_at`,
/// returning a bare `(i32, i32)`. **A
/// conversion that throws away the type of its own answer leaves every reader
/// downstream to guess**, and they guessed `Physical` — smithay's marker for
/// framebuffer pixels, which this is the opposite of.
///
/// The cost of that guess, measured rather than feared:
///
/// - two lanes spent an hour on 2026-09-30 diagnosing a unit mismatch that did
///   not exist, recorded in `canvas_fixed_controls`'s module header;
/// - a confident bug report was filed on 2026-10-02 against a pointer road that
///   was correct, and withdrawn only because the other lane asked for a
///   measurement instead of accepting it;
/// - a drag-handle floor was converted by the display scale on the same day and
///   **landed**, protecting 88 × 48 where the owner's ruling promises 44 × 24,
///   with the one test able to detect it asserting the fault as the promise.
///
/// Three wrong readings of the same eight lines. The type is here so the fourth
/// one is a compiler error instead of an afternoon.
///
/// # What it is not
///
/// **`Physical` is not renamed and nothing is replaced mechanically.** It is
/// smithay's marker, it is correct wherever a value really is in framebuffer
/// pixels, and the owner's direction is explicit that the usages are audited by
/// what they represent rather than swept. This names the boundary; the audit
/// follows it outwards, with evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheRoom {
    /// Side to side, in laid-out units.
    across: i32,
    /// Top to bottom, in laid-out units.
    along: i32,
}

impl TheRoom {
    /// The room a screen of `pixels` offers when drawn at `scale`.
    ///
    /// **The only constructor**, so a `TheRoom` cannot be made from numbers that
    /// never went through the division. A pair that was never converted is
    /// exactly the thing this type exists to be distinguishable from.
    fn from_pixels(scale: Scale, pixels: Resolution) -> Self {
        let side = |pixels: u32| {
            i32::try_from(scale.laid_out(pixels))
                .unwrap_or(i32::MAX)
                .max(1)
        };
        Self {
            across: side(pixels.width()),
            along: side(pixels.height()),
        }
    }

    /// Across and along, in laid-out units.
    ///
    /// Named rather than a field access or a `From`, because **this is where a
    /// caller stops being told what the numbers are** and it should be visible
    /// at the call site that it happened.
    #[must_use]
    pub const fn across_and_along(self) -> (i32, i32) {
        (self.across, self.along)
    }
}

impl ScreenPlace {
    /// Which screen this is.
    #[must_use]
    pub const fn identity(&self) -> &Identity {
        &self.identity
    }

    /// The name `alo-appearance` holds a background against.
    #[must_use]
    pub const fn name(&self) -> &DisplayId {
        &self.name
    }

    /// Its corner on the desk.
    #[must_use]
    pub const fn at(&self) -> Position {
        self.at
    }

    /// How many pixels it has.
    #[must_use]
    pub const fn pixels(&self) -> Resolution {
        self.pixels
    }

    /// The room a surface on it lays itself out in.
    ///
    /// Returns [`TheRoom`] rather than a pair, so a caller cannot mistake it for
    /// the screen's pixels — which is [`Self::pixels`], directly above, and was
    /// the same shape of value with a different meaning.
    #[must_use]
    pub const fn room(&self) -> TheRoom {
        self.room
    }

    /// The size it is drawn at.
    #[must_use]
    pub const fn scale(&self) -> Scale {
        self.scale
    }

    /// Whether a new window opens here.
    #[must_use]
    pub const fn is_main(&self) -> bool {
        self.is_main
    }

    /// Its background, its dock's edge and how warm it is drawn.
    #[must_use]
    pub const fn wearing(&self) -> &Wearing {
        &self.wearing
    }

    /// What night light does to each of this screen's three channels.
    #[must_use]
    pub const fn warming(&self) -> Warming {
        self.wearing.warming()
    }
}

/// The screens plugged in now, in the shape a compositor draws from.
#[derive(Debug, Clone, PartialEq)]
pub struct Screens {
    /// `alo-displays`' own answer about this set.
    attached: Attached,
    /// One place per screen, in the order the machine reported them.
    places: Vec<ScreenPlace>,
}

impl Screens {
    /// The screens plugged in now, each with what it wears.
    #[must_use]
    pub fn of(attached: Attached, appearance: &Appearance, tonight: &Tonight) -> Self {
        let places = laid_out(&attached, appearance, tonight);
        Self { attached, places }
    }

    /// Ask again what each screen wears, without touching the arrangement.
    ///
    /// Night light coming on and a person choosing another background both
    /// reach the screens this way: the set and its places are unchanged and only
    /// what each screen wears is asked again.
    ///
    /// **A dock moved to another edge used to reach them here too.** ADR 0076
    /// fixed the dock to the bottom edge of every screen, so there is nothing
    /// about it left for a screen to wear and this no longer takes one.
    pub fn wearing_again(&mut self, appearance: &Appearance, tonight: &Tonight) {
        self.places = laid_out(&self.attached, appearance, tonight);
    }

    /// Every screen, in the order the machine reported them.
    pub fn each(&self) -> impl Iterator<Item = &ScreenPlace> {
        self.places.iter()
    }

    /// How many screens are plugged in.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.places.len()
    }

    /// One screen, if it is plugged in.
    #[must_use]
    pub fn on(&self, screen: &Identity) -> Option<&ScreenPlace> {
        self.places.iter().find(|place| &place.identity == screen)
    }

    /// The screen a new window opens on.
    #[must_use]
    pub const fn main_screen(&self) -> &Identity {
        self.attached.main_screen()
    }

    /// The arrangement in force, which is what [`Changes::remember`] is handed
    /// when a person is done moving their screens about.
    #[must_use]
    pub const fn arrangement(&self) -> &Arrangement {
        self.attached.arrangement()
    }

    /// What a person is told about how these screens were set up — that they
    /// are as they left them, or that two of them are told apart by their
    /// sockets.
    #[must_use]
    pub fn notes(&self) -> &[Note] {
        self.attached.notes()
    }

    /// Every screen that has been unplugged and whose windows are sitting
    /// somewhere else: the screen that went, then the one they are on.
    pub fn windows_away(&self) -> impl Iterator<Item = (&Identity, &Identity)> {
        self.attached.windows_away()
    }

    /// A screen has been unplugged. Says where what was on it belongs, and
    /// lays the screens that remain out again.
    ///
    /// The compositor moves those windows to [`Moved::onto`]; where they go is
    /// `alo-displays`' answer and never this crate's.
    ///
    /// # Errors
    /// Every refusal `alo_displays::Attached::unplugged` makes: a socket with
    /// no screen in it, the only screen the machine has — what is on it would
    /// have nowhere to go, so nothing changes — and a set that does not
    /// arrange.
    pub fn unplugged(
        &mut self,
        socket: &Socket,
        remembered: &Changes,
        appearance: &Appearance,
        tonight: &Tonight,
    ) -> Result<Moved, NotAttached> {
        let moved = self.attached.unplugged(socket, remembered)?;
        self.places = laid_out(&self.attached, appearance, tonight);
        Ok(moved)
    }

    /// A screen has been plugged in. Says what goes back to it, and lays the
    /// whole set out again — as the person left it, when they have arranged
    /// exactly this set before.
    ///
    /// # Errors
    /// Every refusal `alo_displays::Attached::plugged_in` makes: a socket that
    /// already has a screen in it, and a set that does not arrange.
    pub fn plugged_in(
        &mut self,
        arriving: Reported,
        remembered: &Changes,
        appearance: &Appearance,
        tonight: &Tonight,
    ) -> Result<CameBack, NotAttached> {
        let back = self.attached.plugged_in(arriving, remembered)?;
        self.places = laid_out(&self.attached, appearance, tonight);
        Ok(back)
    }
}

/// One place per attached screen, each asked of the crates that own what it
/// wears.
fn laid_out(attached: &Attached, appearance: &Appearance, tonight: &Tonight) -> Vec<ScreenPlace> {
    attached
        .each()
        .map(|on| {
            let reported = on.reported();
            let drawn_at = on.drawn_at();
            ScreenPlace {
                identity: on.identity().clone(),
                name: reported.named_for_the_shell().clone(),
                at: on.placed().position(),
                pixels: reported.pixels(),
                room: TheRoom::from_pixels(drawn_at, reported.pixels()),
                scale: drawn_at,
                is_main: on.placed().is_the_main_screen(),
                wearing: Wearing::of(reported, appearance, tonight),
            }
        })
        .collect()
}

#[cfg(test)]
#[path = "screens_tests.rs"]
mod tests;
