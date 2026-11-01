//! The colours a task window draws in, mapped from the design's tokens to this
//! machine's roles.
//!
//! The design names seven colours. **None of them is a hex at a call site**:
//! each is an `alo_appearance::Role`, so a person's contrast setting reaches
//! them and a change to the palette reaches this window with everything else.
//!
//! | design token | role |
//! |---|---|
//! | `bg/surface` | the window's ground |
//! | `bg/cool` | [`Role::BgCool`] — bubbles, chips, control grounds |
//! | `border/default` | [`Role::BorderDefault`] — the hairline |
//! | `text/primary` | the ink |
//! | `text/secondary` | [`Role::TextSecondary`] |
//! | `text/muted` | [`Role::TextMuted`] |
//! | `accent/default` | **deep teal, and never the person's accent** |
//!
//! # Why alo's colour is not the accent, and is not allowed to become it
//!
//! [ADR 0067](../../../docs/decisions/0067-the-agents-colour-is-deep-teal-and-the-colour-it-vacates-is-given-back.md)
//! reserves deep teal `#0F6B72` for alo acting on a person's behalf, and
//! `alo_appearance::Accent::of_colour` **refuses** it: it is not a colour a
//! person can pick. So `accent/default` here is the reserved token, read
//! directly, and `DesktopPalette::accent` — which *is* whatever the person
//! chose — is deliberately not used.
//!
//! The window would look right either way on a machine whose owner happened to
//! choose a teal-ish accent, which is exactly why this is held by a test rather
//! than by care: `alos_colour_is_never_the_accent_a_person_chose` draws the
//! palette for every accent there is and holds the teal still.
//!
//! # `ground` is not a window's ground, whatever its name says
//!
//! `DesktopPalette::ground` is documented as *a window's ground* and resolves
//! to `Token::Cream` `#FAF7F2`, which is `Role::BgCanvas` — **the canvas behind
//! windows**. `DesktopPalette::dock` resolves to `Token::Porcelain` `#FFFFFF`,
//! which is `Role::BgSurface` — a panel's own ground, and what the design calls
//! `bg/surface`. The Dock's own panel fills with `palette.dock` for that
//! reason.
//!
//! So a task window's surface is **`dock`**, and this file used `ground` until a
//! test caught it: the contrast table below said teal measured 6.24:1 and the
//! test measured 5.84:1, which is teal on Cream. The figure was right about
//! white and the code was drawing on the wrong colour, so **the check that
//! found it was a figure disagreeing with a figure** rather than anything
//! visible — the window would have looked very slightly warm and nothing would
//! have said why.
//!
//! The names are the trap and they are not this file's to fix. Flagged here
//! because the next reader to want a white panel will reach for `ground`.
//!
//! # Teal survives high contrast, and that is measured
//!
//! `DesktopPalette` substitutes the ink for `edge` and `ink_quiet` at high
//! contrast, because a hairline and a placeholder are quiet by design and a
//! person asking for contrast is asking for them to stop being. **Teal is not
//! substituted**, and the reason is a measurement rather than a preference:
//!
//! | on | ratio | AA 4.5 |
//! |---|---|---|
//! | `#0F6B72` on white | **6.24:1** | passes |
//! | `#0F6B72` on `bg/cool` | **5.54:1** | passes |
//! | `#274C68` on white | 9.05:1 | passes |
//! | `#596B78` on white | 5.53:1 | passes |
//! | `#596B78` on `bg/cool` | **4.91:1** | passes, and is the narrowest here |
//!
//! Measured on 2026-10-12 with the WCAG relative-luminance formula. Teal fails
//! AAA at 7.0 and that is accepted: substituting the ink for it would remove
//! the one mark that says **alo is acting** rather than make it easier to read,
//! and a signal nobody can distinguish is worse than one that meets AA.
//!
//! `#596B78` on `bg/cool` at 4.91:1 is the narrowest pair in the window — the
//! shared-file count on a chip. It passes, and it is named here because it is
//! the first thing a change to either role would break.
//!
//! **This is a measurement, not a standing property.** Nothing re-runs it. A
//! change to `Role::AccentDefault`, `Role::TextMuted` or `Role::BgCool` makes
//! every figure above stale, and no test will say so.

use crate::desktop_look::{DesktopLook, DesktopPalette};
use alo_appearance::{AccentError, Role};

/// The colours a task window draws in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TaskPalette {
    /// The window's own ground — `bg/surface`.
    pub(crate) surface: [u8; 3],
    /// A bubble's, a chip's and a control's ground — `bg/cool`.
    pub(crate) cool: [u8; 3],
    /// The hairline round the window and its reply field — `border/default`.
    pub(crate) edge: [u8; 3],
    /// Words a person reads first — `text/primary`.
    pub(crate) ink: [u8; 3],
    /// The task's kind, and *Activity details* — `text/secondary`.
    pub(crate) ink_secondary: [u8; 3],
    /// A count, a placeholder, where the processing happens — `text/muted`.
    pub(crate) ink_quiet: [u8; 3],
    /// **Alo acting**: its name on a reply, and the status while it works.
    ///
    /// Deep teal, reserved by ADR 0067, and never the person's accent.
    pub(crate) alo: [u8; 3],
    /// The ground of the one Send-or-Stop button, which is `text/primary`.
    ///
    /// The design fills it with the ink colour and draws its arrow in the
    /// surface colour — the only place in the window where that inversion
    /// happens, and what marks it as the one thing a person presses to act.
    pub(crate) the_one_button: [u8; 3],
    /// The arrow or square on that button.
    pub(crate) on_the_one_button: [u8; 3],
    /// The approval card's row of choices — `bg/canvas`, the ground behind
    /// windows, which is `DesktopPalette::ground` (see the note above about
    /// that name).
    pub(crate) canvas: [u8; 3],
}

/// A role's colour as three bytes.
fn rgb_of(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

impl TaskPalette {
    /// The colours for this look.
    ///
    /// # Errors
    /// [`AccentError`] when the person's accent is not one they may choose —
    /// passed straight through from [`DesktopPalette`] rather than swallowed,
    /// because a window that drew itself anyway would be drawing on a machine
    /// whose appearance file is not valid.
    pub(crate) fn of(look: DesktopLook) -> Result<Self, AccentError> {
        let desktop: DesktopPalette = look.palette()?;
        Ok(Self {
            // **`dock`, not `ground`** — see the note above. This is a panel's
            // own white surface, which is what the design calls `bg/surface`.
            surface: desktop.dock,
            // `bg/cool` and the fallback badge's ground are the same role, and
            // `DesktopPalette` already makes the high-contrast substitution.
            cool: desktop.badge,
            edge: desktop.edge,
            ink: desktop.ink,
            // **Not substituted at high contrast**, because 9.05:1 on white is
            // already past AAA — there is nothing for a substitution to gain.
            ink_secondary: rgb_of(Role::TextSecondary),
            ink_quiet: desktop.ink_quiet,
            // **The reserved token, read directly.** Never `desktop.accent`.
            alo: rgb_of(Role::AccentDefault),
            the_one_button: desktop.ink,
            on_the_one_button: desktop.dock,
            canvas: desktop.ground,
        })
    }
}

#[cfg(test)]
#[path = "task_look_tests.rs"]
mod tests;
