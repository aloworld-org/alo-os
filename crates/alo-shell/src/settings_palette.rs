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

/// How strongly it is the accent, dark: `rgba(138,160,230,0.22)` in `550:588`.
const THE_CHOSEN_GROUND_IS_DARK: f32 = 0.22;

/// The dark frames' colours, the accent and the chosen grounds left to the
/// person's accent. **The design's values, and not the roles'**:
/// `alo_appearance::Role` is the light theme only and says so, and these were
/// read with `get_variable_defs` on `550:580` and `550:1025` on 2026-10-31.
const THE_DARK_FRAME: SettingsPalette = SettingsPalette {
    surface: [0x10, 0x2A, 0x43],
    canvas: [0x07, 0x13, 0x1F],
    cool: [0x18, 0x3B, 0x56],
    edge: [0x27, 0x4C, 0x68],
    ink: [0xFF, 0xFC, 0xF8],
    secondary: [0xCB, 0xD5, 0xE1],
    muted: [0xA4, 0xB4, 0xC2],
    warning: [0xE8, 0xAD, 0x50],
    positive: [0x7E, 0xD3, 0xA6],
    accent: [0, 0, 0],
    chosen: [0, 0, 0],
    chosen_on_surface: [0, 0, 0],
    dark: true,
};

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
    /// Whether these are the dark frame's colours, for what has a light and a
    /// dark value of its own: an accent shown as a colour.
    pub(crate) dark: bool,
    /// A chosen entry's ground in a list: the accent over `bg/surface`
    /// (`551:853`).
    pub(crate) chosen_on_surface: [u8; 3],
}

/// A role's colour as three bytes.
fn rgb_of(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// The dark frame's side column, its chosen category Indigo at 22 per cent:
/// what `550:588` fills it with.
#[cfg(test)]
pub(crate) fn the_dark_chosen_ground() -> [u8; 3] {
    mixed(
        THE_DARK_FRAME.canvas,
        [0x8A, 0xA0, 0xE6],
        THE_CHOSEN_GROUND_IS_DARK,
    )
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
        // **Dark is the frame's own values** (`550:580`, `550:1025`), read from
        // the dark frames and recorded in `docs/design/the-settings-window.md`.
        // High contrast keeps its own palette in either scheme.
        if look.scheme == Scheme::Dark && look.contrast == crate::Contrast::AsDesigned {
            let dark = THE_DARK_FRAME;
            return Ok(Self {
                accent: desktop.accent,
                chosen: mixed(dark.canvas, desktop.accent, THE_CHOSEN_GROUND_IS_DARK),
                chosen_on_surface: mixed(dark.surface, desktop.accent, THE_CHOSEN_GROUND_IS_DARK),
                ..dark
            });
        }
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
            chosen_on_surface: mixed(desktop.dock, desktop.accent, THE_CHOSEN_GROUND_IS),
            chosen: mixed(canvas, desktop.accent, THE_CHOSEN_GROUND_IS),
            dark: look.scheme == Scheme::Dark,
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
