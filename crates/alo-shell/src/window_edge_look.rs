//! How a window's card looks — the band and the window, as one card — and
//! the one difference a window without the keyboard has.
//!
//! **The owner's decisions of 2026-11-21** (`docs/design/the-window-controls.md`),
//! quoted as the question and the label chosen:
//!
//! - *"For other apps' windows (VSCodium, Firefox...), should they also become
//!   one card?"* — **"One card for all windows"**, in the 2026-11-09 look,
//!   `11:30` of `XbSmsBYix0TrcQ0Qzaup64`: corner **16**, one line round it,
//!   shadow `0 16 40` at 10 % of `#102a43`, no seam between band and content;
//! - a window without the keyboard — **"Only the title softens"**: the same
//!   card and the same shadow for every window, its title in `text/muted`.
//!
//! | | every window | the window the keys go to | every other |
//! |---|---|---|---|
//! | the band's ground | `bg/surface` | | |
//! | the card's line | `border/default` | | |
//! | the shadow | 16 down, 40 blur, no spread, 10 % of `text/primary` `#102a43` | | |
//! | title and controls | | `text/primary` | `text/muted` |
//!
//! **The colours are the coordinator's reading, for the owner to overrule.**
//! `11:30` binds no variables: its line `#d9dfd9` and ground `#fdfefc` are
//! literals, drawn as the roles the 2026-11-09 reading mapped them to, so
//! nothing outside windows changes.
//!
//! # What this replaced
//!
//! **The round-2 reading of 2026-11-01** (`557:456`, `554:56`): a strip with
//! corners of 10; `bg/surface` and `border/strong` for the focused window and
//! `bg/canvas` and `border/default` for every other; `Elevation/Floating`
//! (16, 40, −4, 14 %) and `Elevation/Quiet` (8, 24, 8 %) shadows. One card for
//! every window, whatever has the keyboard, replaced all of it.
//!
//! `border/strong` stays named here for the terminal's own frame
//! (`crate::the_terminals_frame`), which is not yet drawn by the card.

use alo_appearance::Role;

/// `border/strong`, `#cbd5e1`: the terminal's own frame's line while it has
/// the keyboard, until the card draws that frame.
pub(crate) const BORDER_STRONG: [u8; 3] = [0xCB, 0xD5, 0xE1];

/// How round the card's corners are: `11:30`'s 16.
pub(crate) const THE_STRIPS_CORNER: u32 = 16;

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
        // **One card for every window; only the title softens.**
        EdgeColours {
            strip: rgb(Role::BgSurface),
            edge: rgb(Role::BorderDefault),
            ink: rgb(match self {
                Self::Focused => Role::TextPrimary,
                Self::NotFocused => Role::TextMuted,
            }),
        }
    }

    /// The shadow under this window's body.
    #[must_use]
    pub(crate) const fn shadow(self) -> AShadow {
        // **The same shadow for every window**: `11:30`'s `0 16 40` at 10 %.
        let _ = self;
        AShadow {
            falls: 16,
            blurs: 40,
            spreads: 0,
            darkness: 0.10,
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

    /// **One card for every window, and only the title softens** — the owner,
    /// 2026-11-21. The ground and the line are the same whatever has the
    /// keyboard; the ink is `text/primary` for the window the keys go to and
    /// `text/muted` for every other.
    #[test]
    fn every_window_wears_one_card_and_only_its_title_softens() {
        let rgb = |role: Role| {
            let it = role.colour();
            [it.red(), it.green(), it.blue()]
        };
        let (focused, quiet) = (
            WhichLook::Focused.colours(),
            WhichLook::NotFocused.colours(),
        );
        assert_eq!((focused.strip, focused.edge), (quiet.strip, quiet.edge));
        assert_eq!(focused.strip, rgb(Role::BgSurface));
        assert_eq!(focused.edge, rgb(Role::BorderDefault));
        assert_eq!(focused.ink, rgb(Role::TextPrimary));
        assert_eq!(quiet.ink, rgb(Role::TextMuted));
    }

    /// **The same shadow for every window**, `11:30`'s `0 16 40` at 10 %, and
    /// the card's corner of 16.
    #[test]
    fn every_window_has_the_cards_shadow_and_corner() {
        for look in [WhichLook::Focused, WhichLook::NotFocused] {
            let shadow = look.shadow();
            assert_eq!((shadow.falls, shadow.blurs, shadow.spreads), (16, 40, 0));
            assert!((shadow.darkness - 0.10).abs() < f32::EPSILON);
        }
        assert_eq!(THE_STRIPS_CORNER, 16);
    }

    /// **Only the window with the keyboard is focused.**
    #[test]
    fn the_keyboard_decides_the_look() {
        assert_eq!(WhichLook::of(true), WhichLook::Focused);
        assert_eq!(WhichLook::of(false), WhichLook::NotFocused);
    }
}
