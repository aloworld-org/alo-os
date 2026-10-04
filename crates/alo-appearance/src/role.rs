//! What a colour is *for*, as code — the twelve roles the design file names.
//!
//! [`crate::token`] carries colour *names*, because a person choosing a plain
//! colour to put behind their windows is choosing a name. This carries colour
//! *roles*, because a compositor drawing a frame is not choosing anything: it
//! needs to know that this rectangle is a canvas and that one is a card.
//!
//! The two were one type until 2026-10-04, and the cost of that showed up in
//! `porcelain`: four consumers used it and meant four different things — the
//! workspace canvas, a light ground to measure contrast against, a card
//! surface, and a recessed area. **A colour name cannot be wrong about its role
//! because it never claimed one.** ADR 0092 is the decision; the owner's
//! direction was to *use semantic roles rather than forcing twelve design roles
//! into six colour names*.
//!
//! # Where the values come from
//!
//! `docs/design/palette.toml`, which takes them from the design file.
//! `docs/design/figma-snapshot/variables.toml` records the measurement —
//! which node, which day, which method — so that
//! `tests/a_palette_with_one_source.rs` can hold these constants to the design
//! **without live Figma access**, which two of the three machines working on
//! this repository do not have.
//!
//! These constants exist at all because a compositor cannot parse a file at the
//! moment it draws a frame. They are a copy, the copy is legitimate, and the
//! gate is what stops it drifting.
//!
//! # This is the light theme
//!
//! The design binds the same twelve names to different values in a dark mode,
//! where [`Role::AccentDefault`] measures `#77C8C6` rather than `#0F6B72`.
//! **None of that is here**, because dark-theme parity needs its own measured
//! evidence before anything claims it. [`crate::scheme`] and
//! [`crate::contrast`] still decide ground and readability, and
//! [`crate::token::Token::Charcoal`] is still the dark ground.
//!
//! # Nothing here is said to a person
//!
//! A [`Role`] has no [`alo_strings::Word`], no vocabulary key and no
//! translation, and that is the deliberate opposite of [`crate::token`]'s
//! choice. Nobody reads `bg/cool`. [`Role::key`] returns the design file's own
//! variable name, which is an identifier shared with a design tool rather than
//! English shown on a screen — it is what the gate matches on, and what a
//! failure names so a reader can find the row in `palette.toml`.

use crate::colour::Colour;

/// What a colour is for.
///
/// Twelve, which is the design file's set and not a number chosen here. Adding
/// a thirteenth means measuring it in the design file first: `bg/subtle` was
/// measured and deliberately not adopted, and is recorded in
/// `docs/design/figma-snapshot/variables.toml` so that *absent from the
/// palette* stays distinguishable from *absent from the design*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    /// Structure and body text.
    TextPrimary,
    /// Text that is still text rather than metadata: a subtitle, a second line.
    TextSecondary,
    /// Metadata.
    TextMuted,
    /// The workspace canvas — the desktop a person's windows sit on.
    BgCanvas,
    /// Documents, cards, panel surfaces: anything that sits *on* the canvas.
    ///
    /// The light rail's base is this, by the owner's direction of 2026-10-04.
    /// **The rail's glass treatment is not here**: a base colour, an opacity, a
    /// blur, a border and a shadow are five measurements and this is one of
    /// them. The other four are to be read from the design file's actual fills
    /// and effects rather than invented from a hex.
    BgSurface,
    /// A recessed or secondary surface — a well, a sunken row.
    BgCool,
    /// The ordinary dividing line.
    ///
    /// **A divider drawn in this may never be the only thing conveying
    /// structure or state.** It measures 1.12:1 against [`Self::BgCanvas`],
    /// which is far below the 3.0 a meaningful shape needs — that is what a
    /// hairline is, and it is acceptable only because a hairline is decoration.
    /// A control that needs a visible boundary uses [`Self::TextMuted`] or
    /// darker. ADR 0092 records the measurement.
    BorderDefault,
    /// alo — present, acting, proposing, or waiting for an approval, and
    /// nothing else.
    ///
    /// About five percent of any screen, so a person can tell at a glance
    /// whether the machine is doing something on their behalf. **It is not the
    /// ordinary selection or button colour**, which is exactly what a semantic
    /// name invites a reader to assume, and is why this paragraph is here.
    ///
    /// It never appears alone: the four-corner mark or the word comes with it,
    /// because [`Self::TextPrimary`] and this are both dark desaturated blues
    /// and a person must be able to say who is acting without naming a colour.
    /// ADR 0010, amended by ADR 0067, which reserved this value and which ADR
    /// 0092 deliberately does not touch.
    AccentDefault,
    /// The quiet ground behind alo's own surfaces.
    ///
    /// Not a second accent and not a selection colour.
    AccentSoft,
    /// It finished, it is allowed, it is on.
    StatusPositive,
    /// It needs attention and nothing is broken.
    StatusWarning,
    /// It failed, it is refused, it would destroy something.
    StatusDanger,
}

impl Role {
    /// All twelve, in the order `docs/design/palette.toml` lists them.
    pub const ALL: [Self; 12] = [
        Self::TextPrimary,
        Self::TextSecondary,
        Self::TextMuted,
        Self::BgCanvas,
        Self::BgSurface,
        Self::BgCool,
        Self::BorderDefault,
        Self::AccentDefault,
        Self::AccentSoft,
        Self::StatusPositive,
        Self::StatusWarning,
        Self::StatusDanger,
    ];

    /// The colour itself, as the design file measures it in its light mode.
    #[must_use]
    pub const fn colour(self) -> Colour {
        match self {
            Self::TextPrimary => Colour::of(0x10, 0x2A, 0x43),
            Self::TextSecondary => Colour::of(0x27, 0x4C, 0x68),
            Self::TextMuted => Colour::of(0x59, 0x6B, 0x78),
            Self::BgCanvas => Colour::of(0xFA, 0xF7, 0xF2),
            Self::BgSurface => Colour::of(0xFF, 0xFF, 0xFF),
            Self::BgCool => Colour::of(0xEE, 0xF2, 0xF4),
            Self::BorderDefault => Colour::of(0xE7, 0xEB, 0xEF),
            Self::AccentDefault => Colour::of(0x0F, 0x6B, 0x72),
            Self::AccentSoft => Colour::of(0xE8, 0xF4, 0xF2),
            Self::StatusPositive => Colour::of(0x14, 0x6C, 0x43),
            Self::StatusWarning => Colour::of(0x8C, 0x5A, 0x0A),
            Self::StatusDanger => Colour::of(0xB4, 0x23, 0x18),
        }
    }

    /// The design file's own variable name for this role.
    ///
    /// The key `docs/design/palette.toml` writes it under, and what the
    /// consistency gate matches on. It is an identifier shared with a design
    /// tool, not a string anybody is shown — which is why it is spelled the
    /// design file's way, slash and all, rather than translated.
    #[must_use]
    pub const fn key(self) -> &'static str {
        match self {
            Self::TextPrimary => "text/primary",
            Self::TextSecondary => "text/secondary",
            Self::TextMuted => "text/muted",
            Self::BgCanvas => "bg/canvas",
            Self::BgSurface => "bg/surface",
            Self::BgCool => "bg/cool",
            Self::BorderDefault => "border/default",
            Self::AccentDefault => "accent/default",
            Self::AccentSoft => "accent/soft",
            Self::StatusPositive => "status/positive",
            Self::StatusWarning => "status/warning",
            Self::StatusDanger => "status/danger",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every role is a distinct colour**, because two roles with one value is
    /// either a decision nobody wrote down or a copy-paste, and the twelve are
    /// twelve by construction.
    #[test]
    fn no_two_roles_are_the_same_colour() {
        for (at, role) in Role::ALL.iter().enumerate() {
            for other in Role::ALL.iter().skip(at.saturating_add(1)) {
                assert_ne!(
                    role.colour(),
                    other.colour(),
                    "{} and {} are the same colour",
                    role.key(),
                    other.key()
                );
            }
        }
    }

    /// **And every role has a distinct key**, which the gate relies on: two
    /// roles answering `bg/canvas` would make one of them unreachable from
    /// `palette.toml` and the gate would pass while a colour went unchecked.
    #[test]
    fn no_two_roles_share_a_key() {
        for (at, role) in Role::ALL.iter().enumerate() {
            for other in Role::ALL.iter().skip(at.saturating_add(1)) {
                assert_ne!(role.key(), other.key());
            }
        }
    }

    /// **`ALL` is all of them**, which no derive enforces: a thirteenth variant
    /// added without a row in `ALL` is a colour the gate never reads and the
    /// compositor can still draw with.
    ///
    /// Matching exhaustively is what makes this bite — adding a variant stops
    /// compiling here rather than passing quietly.
    #[test]
    fn all_is_every_role() {
        for role in Role::ALL {
            match role {
                Role::TextPrimary
                | Role::TextSecondary
                | Role::TextMuted
                | Role::BgCanvas
                | Role::BgSurface
                | Role::BgCool
                | Role::BorderDefault
                | Role::AccentDefault
                | Role::AccentSoft
                | Role::StatusPositive
                | Role::StatusWarning
                | Role::StatusDanger => {}
            }
        }
        assert_eq!(Role::ALL.len(), 12, "the design file names twelve");
    }

    /// **The agent's colour is the value ADR 0067 reserved**, asserted here as
    /// well as in the gate, because this is the constant a frame is drawn with
    /// and 0067 is the reason it may not move.
    #[test]
    fn the_agents_colour_is_the_one_adr_0067_reserved() {
        assert_eq!(
            Role::AccentDefault.colour(),
            Colour::of(0x0F, 0x6B, 0x72),
            "ADR 0067 reserves deep teal #0F6B72 for the agent"
        );
    }
}
