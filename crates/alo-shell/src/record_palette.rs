//! The colours the record window draws in, from the design's tokens through
//! this machine's roles — and the dark frame's own values where the roles have
//! none.
//!
//! | design token | here | light | dark (`560:490`) |
//! |---|---|---|---|
//! | `bg/surface` | [`RecordPalette::surface`] | `#ffffff` | `#102a43` |
//! | `bg/cool` | [`RecordPalette::cool`] — an icon's tile, a count | `#eef2f4` | `#183b56` |
//! | `border/default` | [`RecordPalette::edge`] — cards, rules, the footer | `#e7ebef` | `#274c68` |
//! | `border/strong` | [`RecordPalette::frame`] — the window's own edge | `#cbd5e1` | `#738294` |
//! | `text/primary` | [`RecordPalette::ink`] | `#102a43` | `#fffcf8` |
//! | `text/secondary` | [`RecordPalette::secondary`] | `#274c68` | `#cbd5e1` |
//! | `text/muted` | [`RecordPalette::muted`] | `#596b78` | `#a4b4c2` |
//! | `accent/default` | [`RecordPalette::agent`] — the agent's name | `#0f6b72` | `#77c8c6` |
//! | `status/positive` | [`RecordPalette::allowed`] | `#146c43` | `#7ed3a6` |
//! | `status/danger` | [`RecordPalette::refused`] | `#b42318` | `#ff8d86` |
//!
//! # Deep teal is the agent's name, and nothing else
//!
//! Until the owner approved `560:142` on 2026-11-01 this window drew no teal at
//! all — *it means the agent acting, and this window is a person reading*. The
//! approved design draws it in exactly one place: the name of the agent an
//! entry was done under, which **is** the agent acting. So teal is drawn there
//! and nowhere else, and `record_raster_tests` holds it to that.
//!
//! # High contrast is two colours
//!
//! `crate::access_contrast` decides the ground and the ink, and every role
//! here becomes one of the two — the window this was before the redesign, in
//! the redesign's shapes.

use alo_appearance::{Role, Scheme};

use crate::Contrast;

/// The colours the window draws in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RecordPalette {
    /// The window, its strip and its cards.
    pub(crate) surface: [u8; 3],
    /// An entry's icon tile, and the ground of today's counts.
    pub(crate) cool: [u8; 3],
    /// A card's edge, the rules between entries, the footer's line.
    pub(crate) edge: [u8; 3],
    /// The window's own edge and its strip's.
    pub(crate) frame: [u8; 3],
    /// The heading, the title, a clause, a day.
    pub(crate) ink: [u8; 3],
    /// The introduction, the machine's own sentences, a time.
    pub(crate) secondary: [u8; 3],
    /// A date, *Already sent*, the footer.
    pub(crate) muted: [u8; 3],
    /// The name of the agent an entry was done under.
    pub(crate) agent: [u8; 3],
    /// That the person said yes.
    pub(crate) allowed: [u8; 3],
    /// That it was refused.
    pub(crate) refused: [u8; 3],
    /// The confirmation's ground: `bg/canvas` (`560:763`).
    pub(crate) canvas: [u8; 3],
}

/// A role's colour as three bytes.
fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// `border/strong`, `#cbd5e1`, which is not one of the twelve roles:
/// `crate::settings_palette` says why it is named rather than added.
const BORDER_STRONG: [u8; 3] = [0xCB, 0xD5, 0xE1];

/// The dark frame's colours, read off `560:523` on 2026-11-01. **The design's
/// values and not the roles'**: `alo_appearance::Role` is the light theme only
/// and says so.
const THE_DARK_FRAME: RecordPalette = RecordPalette {
    surface: [0x10, 0x2A, 0x43],
    cool: [0x18, 0x3B, 0x56],
    edge: [0x27, 0x4C, 0x68],
    frame: [0x73, 0x82, 0x94],
    ink: [0xFF, 0xFC, 0xF8],
    secondary: [0xCB, 0xD5, 0xE1],
    muted: [0xA4, 0xB4, 0xC2],
    agent: [0x77, 0xC8, 0xC6],
    allowed: [0x7E, 0xD3, 0xA6],
    refused: [0xFF, 0x8D, 0x86],
    canvas: [0x07, 0x13, 0x1F],
};

impl RecordPalette {
    /// The colours for this scheme and contrast.
    pub(crate) fn of(scheme: Scheme, contrast: Contrast) -> Self {
        if contrast == Contrast::High {
            let (ground, ink) = (contrast.ground(scheme), contrast.ink(scheme));
            return Self {
                surface: ground,
                cool: ground,
                edge: ink,
                frame: ink,
                ink,
                secondary: ink,
                muted: ink,
                agent: ink,
                allowed: ink,
                refused: ink,
                canvas: ground,
            };
        }
        if scheme == Scheme::Dark {
            return THE_DARK_FRAME;
        }
        Self {
            surface: rgb(Role::BgSurface),
            cool: rgb(Role::BgCool),
            edge: rgb(Role::BorderDefault),
            frame: BORDER_STRONG,
            ink: rgb(Role::TextPrimary),
            secondary: rgb(Role::TextSecondary),
            muted: rgb(Role::TextMuted),
            agent: rgb(Role::AccentDefault),
            allowed: rgb(Role::StatusPositive),
            refused: rgb(Role::StatusDanger),
            canvas: rgb(Role::BgCanvas),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The light window is the frame's colours**, token for token.
    #[test]
    fn the_light_window_is_the_frames_colours() {
        let light = RecordPalette::of(Scheme::Light, Contrast::AsDesigned);
        assert_eq!(light.surface, [0xFF, 0xFF, 0xFF]);
        assert_eq!(light.frame, [0xCB, 0xD5, 0xE1]);
        assert_eq!(light.ink, [0x10, 0x2A, 0x43]);
        assert_eq!(light.secondary, [0x27, 0x4C, 0x68]);
        assert_eq!(light.muted, [0x59, 0x6B, 0x78]);
        assert_eq!(light.agent, [0x0F, 0x6B, 0x72]);
        assert_eq!(light.allowed, [0x14, 0x6C, 0x43]);
        assert_eq!(light.refused, [0xB4, 0x23, 0x18]);
    }

    /// **High contrast is two colours, in either scheme**, so nothing in the
    /// window is told apart by colour alone there.
    #[test]
    fn high_contrast_is_two_colours() {
        for scheme in [Scheme::Light, Scheme::Dark] {
            let high = RecordPalette::of(scheme, Contrast::High);
            let two = [high.surface, high.ink];
            for colour in [
                high.cool,
                high.edge,
                high.frame,
                high.secondary,
                high.muted,
                high.agent,
                high.allowed,
                high.refused,
            ] {
                assert!(two.contains(&colour), "{scheme:?}: a third colour");
            }
            assert_ne!(high.surface, high.ink);
        }
    }
}
