//! Where everything on the sign-in and unlock screen goes, in logical pixels.
//!
//! Read from `docs/design/figma-snapshot/176-10686.xml`, which was captured
//! from the live file on 2026-10-12 against the owner's seven links. Every
//! number below appears there; none is invented here, and the ones the owner
//! stated independently — the group's 400, the monogram's 64, the entry's
//! 304 × 56 and its 18 radius — agree with it.
//!
//! # Geometry only, so it can be checked against the file
//!
//! Nothing here knows a colour, a font or a frame. That is `crate::sign_in_raster`'s,
//! and the split is what lets a test assert *the entry is 304 × 56 with its
//! submit target inside it* against the capture rather than against a picture
//! somebody looked at. `crate::window_edge` is the same arrangement for the
//! same reason.
//!
//! # The reference is 1440 × 960 and no machine is that
//!
//! The design is drawn at one size; a screen is whatever the person has. So
//! the numbers are **anchors, not positions**:
//!
//! | what | how it adapts |
//! |---|---|
//! | the authentication group | 400 wide until the viewport is narrower, then the viewport less a margin each side |
//! | the essential actions | pinned to the bottom edge, 40 in from each side, never scrolled away |
//! | *Back to lock* | pinned to the top-left corner at its own inset |
//! | the entry's submit target | **44 × 44 always** — it is the one hard floor |
//!
//! The submit target never shrinks because *usable targets* is the owner's
//! word and 44 is the floor every other surface in this repository holds to.
//! A narrow screen takes the width out of the entry's text, which can scroll,
//! rather than out of the thing a finger has to hit.

use smithay::utils::{Logical, Point, Rectangle, Size};

/// The width the design draws the authentication group at.
pub const THE_GROUP_IS_WIDE: i32 = 400;

/// How tall the whole group is, greeting through the row below the entry.
pub const THE_GROUP_IS_TALL: i32 = 268;

/// The monogram's square, and the account initial inside it.
pub const THE_MONOGRAM_IS: i32 = 64;

/// The entry field: `304 × 56`, radius [`THE_ENTRY_CORNER_IS`].
pub const THE_ENTRY_IS: (i32, i32) = (304, 56);

/// The entry's corner, `radius/lg`.
pub const THE_ENTRY_CORNER_IS: i32 = 18;

/// The submit target inside the entry, and the floor it never goes below.
pub const THE_SUBMIT_TARGET_IS: i32 = 44;

/// The arrow drawn inside that target.
pub const THE_ARROW_IS: i32 = 20;

/// How tall the row of essential actions is, and how far in from each side.
pub const THE_ACTIONS_ROW_IS_TALL: i32 = 48;

/// The margin that row keeps from the left and right edges.
pub const THE_ACTIONS_INSET_IS: i32 = 40;

/// How far the actions row's top sits above the bottom edge at the reference
/// size: `960 - 884 = 76`.
const THE_ACTIONS_SIT_ABOVE_THE_EDGE_BY: i32 = 76;

/// *Back to lock*: a 48 × 48 target at `(32, 28)`.
pub const THE_BACK_TARGET_IS: i32 = 48;

/// Where that target's corner sits.
const THE_BACK_SITS_AT: (i32, i32) = (32, 28);

/// How far the group's own top sits below the screen's top at the reference
/// size, as a fraction of the height: `266 / 960`.
///
/// **A fraction rather than 266**, because the group must stay optically where
/// the design puts it — a little above centre — on a screen of any height. At
/// 960 this gives 266 back exactly.
const THE_GROUP_SITS_DOWN_BY: (i32, i32) = (266, 960);

/// The smallest screen this screen lays out on.
///
/// **Derived rather than chosen.** The group is 268 tall and the actions row
/// sits 76 above the bottom edge, so a screen shorter than 344 has nowhere to
/// put both and the two would overlap. 360 is that floor with a little room.
/// The first figure here was 320 x 240, and the test that lays every part out
/// and asks whether it is on the screen caught it: at 240 the group hung 28
/// pixels off the bottom and sat on the actions row.
///
/// A caller is told rather than given a layout that overlaps itself.
pub const THE_SMALLEST_SCREEN_IS: (i32, i32) = (360, 360);

/// Everything on the screen, placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SignInLayout {
    /// The whole screen, which the wallpaper fills.
    pub screen: Rectangle<i32, Logical>,
    /// The authentication group: monogram, greeting, entry, the row below it.
    pub group: Rectangle<i32, Logical>,
    /// The monogram's square.
    pub monogram: Rectangle<i32, Logical>,
    /// The line that greets the person by name.
    pub account: Rectangle<i32, Logical>,
    /// The entry field.
    pub entry: Rectangle<i32, Logical>,
    /// The submit target, **inside** the entry.
    pub submit: Rectangle<i32, Logical>,
    /// The arrow drawn inside the submit target.
    pub arrow: Rectangle<i32, Logical>,
    /// Where an authentication failure is said, between the entry and the row
    /// below it.
    pub feedback: Rectangle<i32, Logical>,
    /// *Other ways to sign in*.
    pub other_ways: Rectangle<i32, Logical>,
    /// The row of essential actions along the bottom.
    pub actions: Rectangle<i32, Logical>,
    /// *Accessibility*, at that row's left.
    pub accessibility: Rectangle<i32, Logical>,
    /// *Switch person*, at its right.
    pub switch_person: Rectangle<i32, Logical>,
    /// The power target, at the far right.
    pub power: Rectangle<i32, Logical>,
    /// *Back to lock*, at the top left.
    pub back: Rectangle<i32, Logical>,
}

/// Lay the sign-in screen out for a screen of this size.
///
/// [`None`] for a screen smaller than [`THE_SMALLEST_SCREEN_IS`], which is the
/// refusal rather than a layout whose parts overlap. A caller that drew one
/// anyway would put the submit arrow under the account's name.
#[must_use]
pub fn sign_in_laid_out(screen: Size<i32, Logical>) -> Option<SignInLayout> {
    if screen.w < THE_SMALLEST_SCREEN_IS.0 || screen.h < THE_SMALLEST_SCREEN_IS.1 {
        return None;
    }
    let whole = Rectangle::from_size(screen);

    // **The group, centred, and never wider than the screen less a margin.**
    let margin = THE_ACTIONS_INSET_IS.min(screen.w / 8);
    let group_width = THE_GROUP_IS_WIDE.min(screen.w - 2 * margin);
    let group_left = (screen.w - group_width) / 2;
    // The design's own fraction of the height, so the group stays optically
    // where it was drawn rather than at a fixed distance from the top.
    let group_top = (screen.h * THE_GROUP_SITS_DOWN_BY.0 / THE_GROUP_SITS_DOWN_BY.1)
        .min(screen.h - THE_GROUP_IS_TALL - THE_ACTIONS_SIT_ABOVE_THE_EDGE_BY)
        .max(0);
    let group = Rectangle::new(
        Point::from((group_left, group_top)),
        Size::from((group_width, THE_GROUP_IS_TALL)),
    );

    // Inside the group, every offset is the capture's.
    let monogram = Rectangle::new(
        Point::from((group_left + (group_width - THE_MONOGRAM_IS) / 2, group_top)),
        Size::from((THE_MONOGRAM_IS, THE_MONOGRAM_IS)),
    );
    // `176:10998` is 156 wide at x=122 in a 400 group — centred, 44 tall, at
    // y=76. Centred here too, at the group's full width, because a name is as
    // long as it is and the design's 156 is one sample.
    let account = Rectangle::new(
        Point::from((group_left, group_top + 76)),
        Size::from((group_width, 44)),
    );

    // `360:25699` is at (48, 144) and 304 wide in a 400 group: a 48 inset each
    // side. Kept as an inset so a narrower group narrows the entry with it.
    let entry_inset = ((group_width - THE_ENTRY_IS.0) / 2).max(0);
    let entry_width = (group_width - 2 * entry_inset).max(THE_SUBMIT_TARGET_IS);
    let entry = Rectangle::new(
        Point::from((group_left + entry_inset, group_top + 144)),
        Size::from((entry_width, THE_ENTRY_IS.1)),
    );
    // `364:25560` at (253, 6) inside a 304 × 56 entry: six in from the right
    // and six down, which is where a 44 target sits centred in a 56 field.
    let submit = Rectangle::new(
        Point::from((
            // Seven in from the right, which is what the capture gives: the
            // entry is 304 and `364:25560` sits at 253, so 304 - 44 - 253 = 7.
            // Six was the guess, and the assertion against the file caught it.
            entry.loc.x + entry.size.w - THE_SUBMIT_TARGET_IS - 7,
            entry.loc.y + (THE_ENTRY_IS.1 - THE_SUBMIT_TARGET_IS) / 2,
        )),
        Size::from((THE_SUBMIT_TARGET_IS, THE_SUBMIT_TARGET_IS)),
    );
    let arrow = Rectangle::new(
        Point::from((
            submit.loc.x + (THE_SUBMIT_TARGET_IS - THE_ARROW_IS) / 2,
            submit.loc.y + (THE_SUBMIT_TARGET_IS - THE_ARROW_IS) / 2,
        )),
        Size::from((THE_ARROW_IS, THE_ARROW_IS)),
    );

    // `364:25566` at (0, 208), 400 × 22, and `364:25564` at (80, 224) — the
    // feedback sits between the entry and the row below it.
    let feedback = Rectangle::new(
        Point::from((group_left, group_top + 208)),
        Size::from((group_width, 22)),
    );
    let other_ways = Rectangle::new(
        Point::from((group_left, group_top + 224)),
        Size::from((group_width, 44)),
    );

    // The essential actions, pinned to the bottom rather than placed at 884.
    let actions_inset = THE_ACTIONS_INSET_IS.min(screen.w / 8);
    let actions = Rectangle::new(
        Point::from((
            actions_inset,
            screen.h - THE_ACTIONS_SIT_ABOVE_THE_EDGE_BY,
        )),
        Size::from((
            (screen.w - 2 * actions_inset).max(0),
            THE_ACTIONS_ROW_IS_TALL,
        )),
    );
    let accessibility = Rectangle::new(
        Point::from((actions.loc.x, actions.loc.y + 2)),
        Size::from((160.min(actions.size.w), 44)),
    );
    // `360:25707` is 200 wide at the row's right: 136 of name then a 48 target
    // at 152.
    let power = Rectangle::new(
        Point::from((
            actions.loc.x + actions.size.w - THE_BACK_TARGET_IS,
            actions.loc.y,
        )),
        Size::from((THE_BACK_TARGET_IS, THE_BACK_TARGET_IS)),
    );
    let switch_person = Rectangle::new(
        Point::from((power.loc.x - 16 - 136, actions.loc.y + 2)),
        Size::from((136, 44)),
    );

    let back = Rectangle::new(
        Point::from(THE_BACK_SITS_AT),
        Size::from((THE_BACK_TARGET_IS, THE_BACK_TARGET_IS)),
    );

    Some(SignInLayout {
        screen: whole,
        group,
        monogram,
        account,
        entry,
        submit,
        arrow,
        feedback,
        other_ways,
        actions,
        accessibility,
        switch_person,
        power,
        back,
    })
}

#[cfg(test)]
#[path = "sign_in_layout_tests.rs"]
mod tests;
