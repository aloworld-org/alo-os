//! **What the pointer is inked in**: the fill, and the edge round it.
//!
//! The owner's yes of 2026-11-06 to three choices, each a shape in one colour
//! with an edge in the opposite one, so it stays visible on any background:
//!
//! | Choice | Fill | Edge |
//! |---|---|---|
//! | Navy, as it ships | `text/primary` #102A43 | `bg/surface` white |
//! | White, for dark backgrounds | `bg/surface` white | `text/primary` #102A43 |
//! | Black | black #000000 | `bg/surface` white |
//!
//! **Black is not a palette token**: the palette has no pure black, and the owner
//! chose black by name, so it is written here as what it is. **Deep teal is never
//! one of them** — it is the agent's (ADR 0067) — and
//! `the_persons_pointer_is_never_the_agents_colour` holds every choice to that.

use alo_appearance::PointerColour;
use alo_appearance::role::Role;

/// Black, which the palette does not name.
pub const BLACK: [u8; 3] = [0, 0, 0];

/// The fill and the edge of one colour choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inks {
    /// The shape itself.
    pub fill: [u8; 3],
    /// The keyline round it.
    pub edge: [u8; 3],
}

/// The inks for `colour`.
#[must_use]
pub fn inks(colour: PointerColour) -> Inks {
    let navy = rgb(Role::TextPrimary);
    let white = rgb(Role::BgSurface);
    match colour {
        PointerColour::Navy => Inks {
            fill: navy,
            edge: white,
        },
        PointerColour::White => Inks {
            fill: white,
            edge: navy,
        },
        PointerColour::Black => Inks {
            fill: BLACK,
            edge: white,
        },
    }
}

/// A palette role's colour as three bytes.
fn rgb(role: Role) -> [u8; 3] {
    let colour = role.colour();
    [colour.red(), colour.green(), colour.blue()]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Each choice is the shape in one colour and its edge in the opposite
    /// one**, so none of them disappears on a background of its own colour.
    #[test]
    fn each_colour_is_edged_in_its_opposite() {
        let navy = [0x10, 0x2A, 0x43];
        let white = [0xFF, 0xFF, 0xFF];
        assert_eq!(
            inks(PointerColour::Navy),
            Inks {
                fill: navy,
                edge: white
            }
        );
        assert_eq!(
            inks(PointerColour::White),
            Inks {
                fill: white,
                edge: navy
            }
        );
        assert_eq!(
            inks(PointerColour::Black),
            Inks {
                fill: BLACK,
                edge: white
            }
        );
        for colour in PointerColour::ALL {
            let them = inks(colour);
            assert_ne!(them.fill, them.edge, "{colour:?}");
        }
    }
}
