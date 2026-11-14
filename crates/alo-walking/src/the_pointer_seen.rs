//! **Finding alo's pointer in a picture of the screen**, by the pixels alo draws
//! it with.
//!
//! The reference is never a screenshot somebody looked at. It is
//! `alo_cursors::draw_in` — the function the compositor draws its own pointer
//! with (`alo_shell::the_pointer_drawn`), and the one `alo-cursor-theme` writes
//! the cursor theme from for applications that draw their own — asked for the
//! same size the compositor asks for: `alo_cursors::measures::pixels_at` of the
//! display's scale times the size the person chose. So a change to the artwork
//! moves the reference with it, and the walk cannot be holding the product to
//! a picture the product no longer draws.
//!
//! # Only the pixels that do not depend on what is under the pointer
//!
//! The pointer's soft edge and its shadow are blended with whatever it is over,
//! so they say nothing about the pointer. Its **solid pixels** — alpha 255 —
//! are the same over any background: the fill, the keyline, and the pixels
//! where the two meet. Those are what is compared, each within [`TOLERANCE`] on
//! every channel.
//!
//! # Both inks must be there
//!
//! A navy block is navy wherever the fill is, so the fill alone would find a
//! pointer in every navy title on the screen. A match needs nine in ten of the
//! fill's solid pixels **and** nine in ten of the rest — the keyline and where
//! it meets the fill. That is also what tells the shapes apart: the familiar
//! arrow's fill falls where the kite has its keyline or nothing, a white
//! pointer's fill is where a navy one's is navy, and a pointer at 24 pixels has
//! its keyline where one at 36 is still filled.
//!
//! # What would have to be true for this to find a pointer that is not there
//!
//! Something else on the screen drawn with the same solid pixels at the same
//! size — a picture of alo's pointer, in a Settings preview or on a web page.
//! The walk does not take a match on its own word: it moves the pointer and
//! finds it again where it went (`crate::walking`'s pointer checks), which a
//! picture of a pointer does not do.
//!
//! # And what would have to be true for it to miss one that is there
//!
//! A renderer that changed the pointer's solid pixels — scaled the picture, or
//! drew it between two pixels. Then the check fails and says what it did find
//! instead, which is the opposite of a silent pass.

use alo_appearance::{PointerColour, PointerShape, PointerSize};
use alo_cursors::{CursorIcon, Silhouette};

use crate::frame::Frame;

/// How far a pixel on the screen may be from the reference's, on each channel.
pub const TOLERANCE: u8 = 16;

/// The fewest solid pixels of each kind a reference needs for a match to mean
/// anything.
pub const FEWEST_SOLID: usize = 3;

/// How far apart two matches must be to be two pointers rather than one.
const ONE_POINTER: usize = 3;

/// What the person chose their pointer to look like: the three settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Look {
    /// Which arrow.
    pub shape: PointerShape,
    /// Which colour.
    pub colour: PointerColour,
    /// How large.
    pub size: PointerSize,
}

impl Look {
    /// As a fresh machine draws it: the navy kite at the size every pointer is.
    pub const SHIPPED: Self = Self {
        shape: PointerShape::Kite,
        colour: PointerColour::Navy,
        size: PointerSize::Default,
    };

    /// Every look a person can choose.
    #[must_use]
    pub fn every() -> Vec<Self> {
        let mut every = Vec::new();
        for shape in PointerShape::ALL {
            for colour in PointerColour::ALL {
                for size in PointerSize::ALL {
                    every.push(Self {
                        shape,
                        colour,
                        size,
                    });
                }
            }
        }
        every
    }

    /// How a report says it: *navy kite, default size*.
    #[must_use]
    pub fn said(self) -> String {
        format!(
            "{} {}, {}",
            colour_said(self.colour),
            shape_said(self.shape),
            size_said(self.size)
        )
    }
}

/// One cursor as alo draws it: a shape in a look, on a display at a scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Drawn {
    /// The shape, by the name an application asks for.
    pub icon: CursorIcon,
    /// The person's look.
    pub look: Look,
    /// The display's scale, in per cent.
    pub scale: u16,
}

impl Drawn {
    /// How many pixels across it is drawn: the compositor's own sum, the
    /// display's scale times the person's size, through
    /// `alo_cursors::measures::pixels_at`.
    #[must_use]
    pub fn pixels(self) -> u32 {
        let per_cent = u32::from(self.scale) * u32::from(self.look.size.per_cent()) / 100;
        alo_cursors::measures::pixels_at(u16::try_from(per_cent).unwrap_or(u16::MAX))
    }

    /// How a report says it: *the navy kite, default size, 24 pixels*.
    #[must_use]
    pub fn said(self) -> String {
        let what = if self.icon == CursorIcon::Default {
            shape_said(self.look.shape).to_owned()
        } else {
            format!("{:?} cursor", self.icon.name())
        };
        format!(
            "the {} {what}, {}, {} pixels",
            colour_said(self.look.colour),
            size_said(self.look.size),
            self.pixels()
        )
    }
}

/// One solid pixel of a reference: where it is in the picture, and its colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Solid {
    /// Across, from the picture's left.
    x: usize,
    /// Down, from the picture's top.
    y: usize,
    /// Its colour, the same over any background.
    rgb: [u8; 3],
}

/// One cursor's solid pixels, ready to be looked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// What it is.
    pub drawn: Drawn,
    /// The pixel that clicks, from the picture's top-left.
    hotspot: (usize, usize),
    /// The solid pixels in the fill's ink.
    fill: Vec<Solid>,
    /// Every other solid pixel: the keyline, and where it meets the fill.
    rest: Vec<Solid>,
}

/// Why a cursor could not be made into something to look for.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NoReference {
    /// `alo-cursors` would not draw it.
    #[error("alo-cursors did not draw {0}: {1}")]
    NotDrawn(String, String),
    /// It has too few solid pixels of one kind for a match to mean anything.
    #[error("{0} has {1} solid fill and {2} other solid pixels, too few to be told apart")]
    TooFewSolid(String, usize, usize),
}

impl Reference {
    /// The solid pixels of `drawn`, from `alo_cursors::draw_in`.
    ///
    /// # Errors
    ///
    /// [`NoReference`] when it is not drawn, or has fewer than [`FEWEST_SOLID`]
    /// solid pixels of either kind.
    pub fn of(drawn: Drawn) -> Result<Self, NoReference> {
        let picture = alo_cursors::draw_in(
            drawn.icon,
            Silhouette::from(drawn.look.shape),
            drawn.look.colour,
            drawn.pixels(),
        )
        .map_err(|why| NoReference::NotDrawn(drawn.said(), why.to_string()))?;
        let ink = alo_cursors::inks::inks(drawn.look.colour).fill;
        let size = usize::try_from(picture.size).unwrap_or(0);
        let mut fill = Vec::new();
        let mut rest = Vec::new();
        for (at, pixel) in picture.pixels.as_chunks::<4>().0.iter().enumerate() {
            if let &[red, green, blue, 255] = pixel {
                let solid = Solid {
                    x: at % size.max(1),
                    y: at / size.max(1),
                    rgb: [red, green, blue],
                };
                if near(solid.rgb, ink, 2) {
                    fill.push(solid);
                } else {
                    rest.push(solid);
                }
            }
        }
        if fill.len() < FEWEST_SOLID || rest.len() < FEWEST_SOLID {
            return Err(NoReference::TooFewSolid(
                drawn.said(),
                fill.len(),
                rest.len(),
            ));
        }
        let hotspot = (
            usize::try_from(picture.hotspot.0).unwrap_or(0),
            usize::try_from(picture.hotspot.1).unwrap_or(0),
        );
        Ok(Self {
            drawn,
            hotspot,
            fill,
            rest,
        })
    }

    /// Whether it is drawn with its hotspot at `(x, y)`.
    ///
    /// **Where the screen's edge cuts it off**, as the compositor clips a
    /// pointer pushed to an edge, the pixels off the screen are not counted —
    /// but at least half of each kind must be on it, and nine in ten of those
    /// must agree.
    #[must_use]
    pub fn is_at(&self, frame: &Frame, (x, y): (usize, usize)) -> bool {
        let (width, height) = frame.size();
        let left = signed(x) - signed(self.hotspot.0);
        let top = signed(y) - signed(self.hotspot.1);
        let on_the_screen = |solid: &Solid| {
            let px = usize::try_from(left + signed(solid.x)).ok()?;
            let py = usize::try_from(top + signed(solid.y)).ok()?;
            (px < width && py < height).then_some((px, py))
        };
        let agrees = |(px, py): (usize, usize), solid: &Solid| {
            frame
                .rgb(px, py)
                .is_some_and(|seen| near(seen, solid.rgb, TOLERANCE))
        };
        holds_for(&self.fill, &on_the_screen, &agrees)
            && holds_for(&self.rest, &on_the_screen, &agrees)
    }
}

/// Where on the screen a search looks: the hotspot within `reach` of a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Around {
    /// Across.
    pub x: usize,
    /// Down.
    pub y: usize,
    /// How far either way.
    pub reach: usize,
}

/// Every place `reference` is drawn on `frame`, as the hotspot's position —
/// anywhere, or only `around` a place. Matches within a few pixels of each
/// other are one pointer, said once.
#[must_use]
pub fn found_in(
    frame: &Frame,
    reference: &Reference,
    around: Option<Around>,
) -> Vec<(usize, usize)> {
    let (width, height) = frame.size();
    if width == 0 || height == 0 {
        return Vec::new();
    }
    let (xs, ys) = match around {
        None => ((0, width - 1), (0, height - 1)),
        Some(around) => (
            (
                around.x.saturating_sub(around.reach),
                (around.x + around.reach).min(width - 1),
            ),
            (
                around.y.saturating_sub(around.reach),
                (around.y + around.reach).min(height - 1),
            ),
        ),
    };
    let mut found: Vec<(usize, usize)> = Vec::new();
    for y in ys.0..=ys.1 {
        for x in xs.0..=xs.1 {
            if !reference.is_at(frame, (x, y)) {
                continue;
            }
            let already = found
                .iter()
                .any(|it| it.0.abs_diff(x) <= ONE_POINTER && it.1.abs_diff(y) <= ONE_POINTER);
            if !already {
                found.push((x, y));
            }
        }
    }
    found
}

/// Whether at least half of `solids` are on the screen, and all but a tenth of
/// those agree — stopping as soon as they cannot.
fn holds_for(
    solids: &[Solid],
    on_the_screen: &impl Fn(&Solid) -> Option<(usize, usize)>,
    agrees: &impl Fn((usize, usize), &Solid) -> bool,
) -> bool {
    let mut inside = 0;
    let mut differed = 0;
    let may_differ = solids.len() / 10;
    for solid in solids {
        let Some(at) = on_the_screen(solid) else {
            continue;
        };
        inside += 1;
        if !agrees(at, solid) {
            differed += 1;
            if differed > may_differ {
                return false;
            }
        }
    }
    inside * 2 >= solids.len() && differed <= inside / 10
}

/// A position as a signed number, to subtract.
fn signed(it: usize) -> i64 {
    i64::try_from(it).unwrap_or(i64::MAX)
}

/// Whether two colours are within `by` on every channel.
fn near(one: [u8; 3], two: [u8; 3], by: u8) -> bool {
    one.iter().zip(two).all(|(a, b)| a.abs_diff(b) <= by)
}

/// A shape as a report says it.
fn shape_said(shape: PointerShape) -> &'static str {
    match shape {
        PointerShape::Kite => "kite",
        PointerShape::Classic => "familiar arrow",
    }
}

/// A colour as a report says it.
fn colour_said(colour: PointerColour) -> &'static str {
    match colour {
        PointerColour::Navy => "navy",
        PointerColour::White => "white",
        PointerColour::Black => "black",
    }
}

/// A size as a report says it.
fn size_said(size: PointerSize) -> &'static str {
    match size {
        PointerSize::Default => "default size",
        PointerSize::Large => "large",
        PointerSize::Larger => "larger",
    }
}

#[cfg(test)]
#[path = "the_pointer_seen_tests.rs"]
mod tests;
