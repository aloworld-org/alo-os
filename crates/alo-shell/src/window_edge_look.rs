//! How a window's edge and body look, focused and not: the colours and the
//! figures that tell one window from another.
//!
//! **The owner's answer of 2026-11-01**, from the board `557:456` (*How a person
//! tells windows apart*) and the edge `554:56` (*Proposed / Window edge (title
//! kept visible)*), each measured off the live file that day:
//!
//! | | focused — the window your typing goes to | not focused |
//! |---|---|---|
//! | strip | `bg/surface` `#ffffff` | `bg/canvas` `#faf7f2` |
//! | strip's edge | `border/strong` `#cbd5e1` | `border/default` `#e7ebef` |
//! | title, icon, controls | `text/primary` `#102a43` | `text/muted` `#596b78` |
//! | body's edge | `border/strong` | `border/default` |
//! | body's shadow | `Elevation/Floating`: 16 down, 40 blur, −4 spread, 14 % | `Elevation/Quiet`: 8 down, 24 blur, 8 % |
//!
//! The strip is 32 tall with its top corners rounded at **10** (`554:25`). The
//! shadow's colour is `#07131f` in both, which is `crate::dock_surface`'s.
//!
//! **Application content is never dimmed** — the owner's words, and the board's:
//! *the application's own content is never dimmed, cropped or recoloured.* So
//! every difference here is in what alo draws round a window, and nothing here
//! touches a pixel the application drew.
//!
//! # `border/strong` is not one of the twelve roles
//!
//! `docs/design/figma-snapshot/variables.toml` records it as seen and
//! unadopted, and `alo_appearance::Role` holds the twelve the owner approved on
//! 2026-10-04. The owner approved the frames that use it on 2026-11-01, which
//! is approval of this use rather than a thirteenth role; so it is named here,
//! once, the way `crate::settings_palette` already names it, and not added to
//! the palette.

use alo_appearance::Role;

/// `border/strong`, `#cbd5e1`: the focused window's edges.
pub(crate) const BORDER_STRONG: [u8; 3] = [0xCB, 0xD5, 0xE1];

/// How round the strip's top corners are: `554:25`'s 10.
pub(crate) const THE_STRIPS_CORNER: u32 = 10;

/// How thick every edge line is: one logical unit, scaled at the boundary.
pub(crate) const A_LINE_IS: i32 = 1;

/// Which look a window wears.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WhichLook {
    /// The window the person's typing goes to.
    Focused,
    /// Every other window.
    #[default]
    NotFocused,
}

impl WhichLook {
    /// The look of a window that does, or does not, have the keyboard.
    #[must_use]
    pub const fn of(has_the_keyboard: bool) -> Self {
        if has_the_keyboard {
            Self::Focused
        } else {
            Self::NotFocused
        }
    }

    /// The colours for this look.
    #[must_use]
    pub fn colours(self) -> EdgeColours {
        let rgb = |role: Role| {
            let it = role.colour();
            [it.red(), it.green(), it.blue()]
        };
        match self {
            Self::Focused => EdgeColours {
                strip: rgb(Role::BgSurface),
                edge: BORDER_STRONG,
                ink: rgb(Role::TextPrimary),
            },
            Self::NotFocused => EdgeColours {
                strip: rgb(Role::BgCanvas),
                edge: rgb(Role::BorderDefault),
                ink: rgb(Role::TextMuted),
            },
        }
    }

    /// The shadow under this window's body.
    #[must_use]
    pub(crate) const fn shadow(self) -> AShadow {
        match self {
            // `Elevation/Floating`, as `557:213` draws it.
            Self::Focused => AShadow {
                falls: 16,
                blurs: 40,
                spreads: -4,
                darkness: 36.0 / 255.0,
            },
            // `Elevation/Quiet`, as `557:149` draws it.
            Self::NotFocused => AShadow {
                falls: 8,
                blurs: 24,
                spreads: 0,
                darkness: 0.08,
            },
        }
    }
}

/// The three colours an edge is drawn in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EdgeColours {
    /// The strip's ground, which the title's letters are inked on.
    pub strip: [u8; 3],
    /// The strip's line and the body's.
    pub edge: [u8; 3],
    /// The title, the icon's neighbours and the controls.
    pub ink: [u8; 3],
}

/// A shadow, in logical units.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct AShadow {
    /// How far below the body it falls.
    pub(crate) falls: i32,
    /// How wide its fade is.
    pub(crate) blurs: i32,
    /// How much bigger than the body it is before it fades; negative tucks it in.
    pub(crate) spreads: i32,
    /// How dark it is at its darkest, nought to one.
    pub(crate) darkness: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The two looks are the board's, colour for colour.**
    #[test]
    fn the_two_looks_are_the_ones_the_owner_approved() {
        let focused = WhichLook::Focused.colours();
        assert_eq!(focused.strip, [0xFF, 0xFF, 0xFF]);
        assert_eq!(focused.edge, [0xCB, 0xD5, 0xE1]);
        assert_eq!(focused.ink, [0x10, 0x2A, 0x43]);
        let quiet = WhichLook::NotFocused.colours();
        assert_eq!(quiet.strip, [0xFA, 0xF7, 0xF2]);
        assert_eq!(quiet.edge, [0xE7, 0xEB, 0xEF]);
        assert_eq!(quiet.ink, [0x59, 0x6B, 0x78]);
    }

    /// **A focused window is raised higher than one that is not**, which is the
    /// whole of what the shadow is for.
    #[test]
    fn the_focused_window_stands_further_off_the_canvas() {
        let (high, low) = (WhichLook::Focused.shadow(), WhichLook::NotFocused.shadow());
        assert!(high.falls > low.falls && high.blurs > low.blurs);
        assert!(high.darkness > low.darkness);
        assert_eq!((high.falls, high.blurs, high.spreads), (16, 40, -4));
        assert_eq!((low.falls, low.blurs, low.spreads), (8, 24, 0));
    }

    /// **Only the window with the keyboard is focused.**
    #[test]
    fn the_keyboard_decides_the_look() {
        assert_eq!(WhichLook::of(true), WhichLook::Focused);
        assert_eq!(WhichLook::of(false), WhichLook::NotFocused);
    }
}
