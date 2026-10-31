//! The colours the Settings window draws in, from the design's tokens through
//! this machine's roles — so a person's contrast setting and their accent reach
//! every one.
//!
//! | design token | here |
//! |---|---|
//! | `bg/surface` | [`SettingsPalette::surface`] — the page and its cards |
//! | `bg/canvas` | [`SettingsPalette::canvas`] — the side column, a risk box |
//! | `bg/cool` | [`SettingsPalette::cool`] — a box that says something, a switch that is off |
//! | `border/default` | [`SettingsPalette::edge`] |
//! | `text/primary` | [`SettingsPalette::ink`], and the keyboard's outline |
//! | `text/secondary` | [`SettingsPalette::secondary`] |
//! | `text/muted` | [`SettingsPalette::muted`] |
//! | `status/warning` | [`SettingsPalette::warning`] |
//! | the person's accent | [`SettingsPalette::accent`] |
//!
//! # Selections and switches follow the person's accent
//!
//! The owner's decision of 2026-10-31. A chosen ring, a switch that is on and
//! the chosen category's ground are the accent a person picked, which is never
//! deep teal (`alo_appearance::Accent` cannot be). The chosen category's ground
//! is the accent at **12 per cent** over the side column, as frame `550:105`
//! fills it — `rgba(58,90,168,0.12)` is Indigo's light value at that opacity.
//!
//! # The keyboard's outline is not the accent
//!
//! It is `text/primary`, as the task window's is (*focus uses a visible navy
//! outline*): where the keyboard is must read the same whatever colour a person
//! chose, and an outline in a pale accent on a pale ground would not.

use alo_appearance::{AccentError, Role, Scheme};

use crate::SettingsLook;
use crate::desktop_look::DesktopPalette;

/// How strongly the chosen category's ground is the accent, light.
const THE_CHOSEN_GROUND_IS: f32 = 0.12;

/// The colours the window draws in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SettingsPalette {
    /// The page and its cards.
    pub(crate) surface: [u8; 3],
    /// The side column and a risk box.
    pub(crate) canvas: [u8; 3],
    /// A box that says something, and a switch that is off.
    pub(crate) cool: [u8; 3],
    /// Every hairline.
    pub(crate) edge: [u8; 3],
    /// Names and the words a person reads first.
    pub(crate) ink: [u8; 3],
    /// Headings, an introduction, a category not chosen.
    pub(crate) secondary: [u8; 3],
    /// A row's line, a footnote, a ring not chosen.
    pub(crate) muted: [u8; 3],
    /// A note that asks for attention, and what a choice risks.
    pub(crate) warning: [u8; 3],
    /// *Always on*: `status/positive`.
    pub(crate) positive: [u8; 3],
    /// The person's accent.
    pub(crate) accent: [u8; 3],
    /// The chosen category's ground: the accent over the side column.
    pub(crate) chosen: [u8; 3],
}

/// A role's colour as three bytes.
fn rgb_of(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// `over` at `amount` on `under`.
fn mixed(under: [u8; 3], over: [u8; 3], amount: f32) -> [u8; 3] {
    let mut out = [0; 3];
    for ((out, under), over) in out.iter_mut().zip(under).zip(over) {
        let it = f32::from(over) * amount + f32::from(under) * (1.0 - amount);
        *out = it.round().clamp(0.0, 255.0) as u8;
    }
    out
}

impl SettingsPalette {
    /// The colours for this look.
    ///
    /// # Errors
    /// [`AccentError`] when the person's accent is not one they may choose,
    /// passed through rather than swallowed: a window that drew itself anyway
    /// would be drawing on a machine whose appearance file is not valid.
    pub(crate) fn of(look: SettingsLook) -> Result<Self, AccentError> {
        let desktop = DesktopPalette::of(look.scheme, look.accent, look.contrast)?;
        let quiet_or_ink = |role: Role| match look.contrast {
            crate::Contrast::High => desktop.ink,
            crate::Contrast::AsDesigned => rgb_of(role),
        };
        let canvas = desktop.ground;
        Ok(Self {
            surface: desktop.dock,
            canvas,
            cool: desktop.badge,
            edge: desktop.edge,
            ink: desktop.ink,
            secondary: quiet_or_ink(Role::TextSecondary),
            muted: desktop.ink_quiet,
            warning: quiet_or_ink(Role::StatusWarning),
            positive: quiet_or_ink(Role::StatusPositive),
            accent: desktop.accent,
            chosen: match look.scheme {
                Scheme::Light | Scheme::Dark => mixed(canvas, desktop.accent, THE_CHOSEN_GROUND_IS),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Indigo at twelve per cent over the side column is what `550:105`
    /// fills the chosen category with**, to the byte.
    #[test]
    fn the_chosen_ground_is_the_accent_at_twelve_per_cent() {
        // rgba(58,90,168,0.12) over #FAF7F2.
        assert_eq!(
            mixed([0xFA, 0xF7, 0xF2], [0x3A, 0x5A, 0xA8], 0.12),
            [227, 228, 233]
        );
    }
}
