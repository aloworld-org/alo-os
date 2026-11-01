//! Reading the part of SVG the design file exports for small artwork.
//!
//! # The rules
//!
//! - **One `<svg>`**, with a `viewBox` that starts at `0 0` and has a positive
//!   width and height. Its `width`, `height`, `xmlns`, `preserveAspectRatio`,
//!   `overflow`, `style` and `id` are presentation for a browser and are read
//!   past; its `fill` may only be `none`, which is what the export writes.
//! - **`<g>` groups**, carrying an `id` and nothing else. A group with a
//!   `transform` is refused, because this crate does not apply one.
//! - **`<path/>` elements**, each with `d`, and filled, stroked or both, in one
//!   colour: `#rrggbb`, `white` or `black`. A stroke has a `stroke-width`
//!   (one, if not given), a `stroke-linecap` of `butt`, `round` or `square` and
//!   a `stroke-linejoin` of `miter`, `round` or `bevel` — SVG's defaults are
//!   `butt` and `miter`.
//!
//! - **A clip that clips nothing**: a `<g>` may carry `clip-path="url(#id)"`
//!   when `id` is a `<clipPath>` in `<defs>` holding only `<rect>`s that cover
//!   the whole `viewBox`. That is what the design file exports around a
//!   frame's own artwork — the alo mark is drawn so (`549:195`) — and it
//!   changes nothing that is drawn. A clip that would cut anything is refused,
//!   because this crate does not apply one. *Added 2026-11-01 for the mark.*
//!
//! **Everything else is refused, by name.** A file that needs more than this is
//! a file this crate cannot draw truthfully, and it says so rather than drawing
//! part of it.
//!
//! **Path data is checked, not parsed, here**: it must start with a move and
//! hold only SVG's command letters, numbers and separators. The rasteriser
//! parses it, and [`AShape::read`] draws every path once at its own size to
//! refuse one that comes out empty.
//!
//! **Attribute values may not contain `>`**, which none of SVG's attributes this
//! crate reads ever needs to.

/// Artwork read from an SVG file, ready to draw at any size.
#[derive(Debug, Clone, PartialEq)]
pub struct AShape {
    /// The `viewBox`'s width, in the file's own units.
    pub(crate) wide: f32,
    /// The `viewBox`'s height.
    pub(crate) tall: f32,
    /// Every path, in the order the file draws them.
    pub(crate) parts: Vec<APart>,
}

/// One painted path.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct APart {
    /// The path data, as the file wrote it.
    pub(crate) d: String,
    /// Whether it is filled or stroked, and how.
    pub(crate) paint: Paint,
    /// The colour the file painted it in.
    pub(crate) colour: [u8; 3],
}

/// How one path is painted.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Paint {
    /// Its inside, by SVG's default non-zero rule.
    Filled,
    /// Its outline.
    Stroked {
        /// How wide, in the file's own units.
        width: f32,
        /// How its open ends finish.
        cap: Cap,
        /// How its corners finish.
        join: Join,
    },
}

/// `stroke-linecap`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Cap {
    /// `butt`, SVG's default.
    Butt,
    /// `round`.
    Round,
    /// `square`.
    Square,
}

/// `stroke-linejoin`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Join {
    /// `miter`, SVG's default.
    Miter,
    /// `round`.
    Round,
    /// `bevel`.
    Bevel,
}

/// Why a file could not be read as artwork this crate can draw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotAShape {
    /// Something other than whitespace sits between two tags.
    TextOutsideATag,
    /// A `<` with no `>` after it.
    ATagNeverEnds,
    /// An element this crate does not draw, by its name.
    AnElementNotDrawn(String),
    /// An attribute this crate does not read, on the element it was found on.
    AnAttributeNotRead {
        /// The element.
        element: String,
        /// The attribute.
        attribute: String,
    },
    /// An attribute written without a quoted value.
    AnAttributeWithNoValue(String),
    /// A closing tag that does not close what is open.
    AClosingTagOutOfPlace(String),
    /// The file ended with an element still open.
    AnElementNeverClosed(String),
    /// The `<svg>` has no `viewBox`, or a second `<svg>` was found.
    NotOneSvgWithAViewBox,
    /// The `viewBox` does not start at `0 0`, or is not a positive size.
    AViewBoxThisCannotDraw(String),
    /// A value that should be a number and is not.
    NotANumber(String),
    /// A paint that is not `none`, `#rrggbb`, `white` or `black`.
    NotAColour(String),
    /// A cap or join SVG does not name.
    NotACapOrJoin(String),
    /// A path with no `d`, or one whose data is not SVG path data.
    NotPathData(String),
    /// A path that is neither filled nor stroked.
    APathNotPainted,
    /// No path at all.
    NothingDrawn,
    /// A clip this crate cannot honour by drawing nothing differently: one
    /// that is not defined, not a `<rect>`, or smaller than the `viewBox`.
    AClipThisCannotDraw(String),
}

impl core::fmt::Display for NotAShape {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TextOutsideATag => write!(f, "text outside a tag"),
            Self::ATagNeverEnds => write!(f, "a tag never ends"),
            Self::AnElementNotDrawn(name) => write!(f, "<{name}> is not drawn by this crate"),
            Self::AnAttributeNotRead { element, attribute } => {
                write!(f, "{attribute} on <{element}> is not read by this crate")
            }
            Self::AnAttributeWithNoValue(name) => write!(f, "{name} has no quoted value"),
            Self::AClosingTagOutOfPlace(name) => write!(f, "</{name}> closes nothing open"),
            Self::AnElementNeverClosed(name) => write!(f, "<{name}> is never closed"),
            Self::NotOneSvgWithAViewBox => write!(f, "not one <svg> with a viewBox"),
            Self::AViewBoxThisCannotDraw(value) => write!(f, "viewBox {value:?} cannot be drawn"),
            Self::NotANumber(value) => write!(f, "{value:?} is not a number"),
            Self::NotAColour(value) => write!(f, "{value:?} is not a colour this crate reads"),
            Self::NotACapOrJoin(value) => write!(f, "{value:?} is not a cap or a join"),
            Self::NotPathData(value) => write!(f, "{value:?} is not path data"),
            Self::APathNotPainted => write!(f, "a path is neither filled nor stroked"),
            Self::NothingDrawn => write!(f, "nothing is drawn"),
            Self::AClipThisCannotDraw(what) => write!(f, "the clip {what:?} would cut something"),
        }
    }
}

impl std::error::Error for NotAShape {}

impl AShape {
    /// Read artwork from the text of an SVG file.
    ///
    /// # Errors
    /// [`NotAShape`], naming the first thing in the file this crate does not
    /// draw — see this module's rules.
    pub fn read(svg: &str) -> Result<Self, NotAShape> {
        let mut reader = Reader::default();
        let mut rest = svg;
        loop {
            rest = rest.trim_start();
            if rest.is_empty() {
                break;
            }
            let opened = rest.strip_prefix('<').ok_or(NotAShape::TextOutsideATag)?;
            let (tag, after) = opened.split_once('>').ok_or(NotAShape::ATagNeverEnds)?;
            reader.a_tag(tag)?;
            rest = after;
        }
        reader.finished()
    }

    /// The colour of every path, in the order the file draws them.
    #[must_use]
    pub fn colours(&self) -> Vec<[u8; 3]> {
        self.parts.iter().map(|part| part.colour).collect()
    }

    /// How many painted paths it has: one for each path filled or stroked,
    /// two for one that is both.
    #[must_use]
    pub fn how_many_parts(&self) -> usize {
        self.parts.len()
    }

    /// The `viewBox`'s width and height, in the file's own units.
    #[must_use]
    pub const fn its_box(&self) -> (f32, f32) {
        (self.wide, self.tall)
    }
}

/// What the reader has seen so far.
#[derive(Debug, Default)]
struct Reader {
    /// The `viewBox`, once the `<svg>` has been read.
    view: Option<(f32, f32)>,
    /// The elements open, outermost first.
    open: Vec<String>,
    /// The paths read.
    parts: Vec<APart>,
    /// Every clip a group names, by its id.
    clips_used: Vec<String>,
    /// Every clip defined as covering the whole `viewBox`, by its id.
    clips_that_cut_nothing: Vec<String>,
    /// The `<clipPath>` being read, while one is open.
    clip_being_read: Option<String>,
}

impl Reader {
    /// One tag, without its `<` and `>`.
    fn a_tag(&mut self, tag: &str) -> Result<(), NotAShape> {
        if let Some(closing) = tag.strip_prefix('/') {
            let name = closing.trim();
            return match self.open.pop() {
                Some(open) if open == name => {
                    if name == "clipPath" {
                        self.clip_being_read = None;
                    }
                    Ok(())
                }
                _ => Err(NotAShape::AClosingTagOutOfPlace(name.to_owned())),
            };
        }
        let (body, closes_itself) = match tag.strip_suffix('/') {
            Some(body) => (body, true),
            None => (tag, false),
        };
        let body = body.trim();
        let (name, attributes) = body.split_once(char::is_whitespace).unwrap_or((body, ""));
        let attributes = attributes_of(attributes)?;
        match name {
            "svg" => self.an_svg(&attributes)?,
            "g" => self.a_group(&attributes)?,
            "path" => self.a_path(&attributes)?,
            "defs" => self.a_defs(&attributes)?,
            "clipPath" => self.a_clip_path(&attributes)?,
            "rect" => self.a_clips_rect(&attributes)?,
            other => return Err(NotAShape::AnElementNotDrawn(other.to_owned())),
        }
        if !closes_itself {
            self.open.push(name.to_owned());
        }
        Ok(())
    }

    /// The `<svg>` itself.
    fn an_svg(&mut self, attributes: &[(&str, &str)]) -> Result<(), NotAShape> {
        if self.view.is_some() || !self.open.is_empty() {
            return Err(NotAShape::NotOneSvgWithAViewBox);
        }
        let mut view = None;
        for &(name, value) in attributes {
            match name {
                "viewBox" => view = Some(a_view_box(value)?),
                "fill" if value == "none" => {}
                "xmlns"
                | "width"
                | "height"
                | "preserveAspectRatio"
                | "overflow"
                | "style"
                | "id" => {}
                other => return Err(not_read("svg", other)),
            }
        }
        self.view = Some(view.ok_or(NotAShape::NotOneSvgWithAViewBox)?);
        Ok(())
    }

    /// A `<g>`, which only groups.
    fn a_group(&mut self, attributes: &[(&str, &str)]) -> Result<(), NotAShape> {
        self.inside_the_svg()?;
        self.not_inside_the_defs("g")?;
        for &(name, value) in attributes {
            match name {
                "id" => {}
                // **Checked when the file ends**, because the export writes the
                // `<defs>` after the group that names it.
                "clip-path" => {
                    let id = value
                        .strip_prefix("url(#")
                        .and_then(|it| it.strip_suffix(')'))
                        .ok_or_else(|| NotAShape::AClipThisCannotDraw(value.to_owned()))?;
                    self.clips_used.push(id.to_owned());
                }
                other => return Err(not_read("g", other)),
            }
        }
        Ok(())
    }

    /// `<defs>`, which may hold clips and nothing that is drawn.
    fn a_defs(&self, attributes: &[(&str, &str)]) -> Result<(), NotAShape> {
        self.inside_the_svg()?;
        match attributes.first() {
            Some(&(name, _)) => Err(not_read("defs", name)),
            None => Ok(()),
        }
    }

    /// A `<clipPath>`, directly inside `<defs>`.
    fn a_clip_path(&mut self, attributes: &[(&str, &str)]) -> Result<(), NotAShape> {
        if self.open.last().map(String::as_str) != Some("defs") {
            return Err(NotAShape::AnElementNotDrawn("clipPath".to_owned()));
        }
        let mut id = None;
        for &(name, value) in attributes {
            match name {
                "id" => id = Some(value.to_owned()),
                other => return Err(not_read("clipPath", other)),
            }
        }
        self.clip_being_read =
            Some(id.ok_or_else(|| NotAShape::AClipThisCannotDraw(String::new()))?);
        Ok(())
    }

    /// A `<rect/>`, which is read only as a clip and only when it covers the
    /// whole `viewBox`.
    fn a_clips_rect(&mut self, attributes: &[(&str, &str)]) -> Result<(), NotAShape> {
        let (Some(id), Some((wide, tall))) = (self.clip_being_read.clone(), self.view) else {
            return Err(NotAShape::AnElementNotDrawn("rect".to_owned()));
        };
        let (mut x, mut y, mut w, mut h) = (0.0, 0.0, None, None);
        for &(name, value) in attributes {
            match name {
                "x" => x = a_number(value)?,
                "y" => y = a_number(value)?,
                "width" => w = Some(a_number(value)?),
                "height" => h = Some(a_number(value)?),
                // A clip's own paint is never drawn.
                "fill" | "id" => {}
                other => return Err(not_read("rect", other)),
            }
        }
        let covers = |from: f32, size: Option<f32>, whole: f32| {
            from <= 0.0 && size.is_some_and(|size| from + size >= whole)
        };
        if covers(x, w, wide) && covers(y, h, tall) {
            self.clips_that_cut_nothing.push(id);
            Ok(())
        } else {
            Err(NotAShape::AClipThisCannotDraw(id))
        }
    }

    /// Refuse a drawn element inside `<defs>`, where nothing is drawn.
    fn not_inside_the_defs(&self, element: &str) -> Result<(), NotAShape> {
        if self.open.iter().any(|open| open == "defs") {
            Err(NotAShape::AnElementNotDrawn(element.to_owned()))
        } else {
            Ok(())
        }
    }

    /// A `<path/>`.
    fn a_path(&mut self, attributes: &[(&str, &str)]) -> Result<(), NotAShape> {
        self.inside_the_svg()?;
        self.not_inside_the_defs("path")?;
        let mut d = None;
        let mut fill = None;
        let mut stroke = None;
        let mut width = 1.0;
        let mut cap = Cap::Butt;
        let mut join = Join::Miter;
        for &(name, value) in attributes {
            match name {
                "d" => d = Some(path_data(value)?),
                "fill" => fill = a_paint(value)?,
                "stroke" => stroke = a_paint(value)?,
                "stroke-width" => width = a_positive_number(value)?,
                "stroke-linecap" => {
                    cap = match value {
                        "butt" => Cap::Butt,
                        "round" => Cap::Round,
                        "square" => Cap::Square,
                        other => return Err(NotAShape::NotACapOrJoin(other.to_owned())),
                    };
                }
                "stroke-linejoin" => {
                    join = match value {
                        "miter" => Join::Miter,
                        "round" => Join::Round,
                        "bevel" => Join::Bevel,
                        other => return Err(NotAShape::NotACapOrJoin(other.to_owned())),
                    };
                }
                "id" => {}
                other => return Err(not_read("path", other)),
            }
        }
        let d = d.ok_or_else(|| NotAShape::NotPathData(String::new()))?;
        if fill.is_none() && stroke.is_none() {
            return Err(NotAShape::APathNotPainted);
        }
        if let Some(colour) = fill {
            self.parts.push(APart {
                d: d.clone(),
                paint: Paint::Filled,
                colour,
            });
        }
        if let Some(colour) = stroke {
            self.parts.push(APart {
                d,
                paint: Paint::Stroked { width, cap, join },
                colour,
            });
        }
        Ok(())
    }

    /// Refuse an element that is not inside the `<svg>`.
    fn inside_the_svg(&self) -> Result<(), NotAShape> {
        if self.view.is_some() && !self.open.is_empty() {
            Ok(())
        } else {
            Err(NotAShape::NotOneSvgWithAViewBox)
        }
    }

    /// The file has ended: everything is closed and something is drawn.
    fn finished(self) -> Result<AShape, NotAShape> {
        if let Some(open) = self.open.last() {
            return Err(NotAShape::AnElementNeverClosed(open.clone()));
        }
        let (wide, tall) = self.view.ok_or(NotAShape::NotOneSvgWithAViewBox)?;
        if let Some(cuts) = self
            .clips_used
            .iter()
            .find(|used| !self.clips_that_cut_nothing.contains(used))
        {
            return Err(NotAShape::AClipThisCannotDraw(cuts.clone()));
        }
        if self.parts.is_empty() {
            return Err(NotAShape::NothingDrawn);
        }
        let shape = AShape {
            wide,
            tall,
            parts: self.parts,
        };
        // **Each path drawn once, at its own size**, so path data the
        // rasteriser cannot follow is refused here rather than drawn as
        // nothing on somebody's screen.
        for part in &shape.parts {
            if crate::drawing::coverage_of_one(&shape, part, 64, 64)
                .iter()
                .all(|covered| *covered == 0)
            {
                return Err(NotAShape::NotPathData(part.d.clone()));
            }
        }
        Ok(shape)
    }
}

/// An attribute this crate does not read.
fn not_read(element: &str, attribute: &str) -> NotAShape {
    NotAShape::AnAttributeNotRead {
        element: element.to_owned(),
        attribute: attribute.to_owned(),
    }
}

/// `name="value"` pairs, in order.
fn attributes_of(mut rest: &str) -> Result<Vec<(&str, &str)>, NotAShape> {
    let mut pairs = Vec::new();
    loop {
        rest = rest.trim_start();
        if rest.is_empty() {
            return Ok(pairs);
        }
        let (name, after) = rest
            .split_once('=')
            .ok_or_else(|| NotAShape::AnAttributeWithNoValue(rest.trim().to_owned()))?;
        let name = name.trim();
        let after = after.trim_start();
        let quote = after
            .chars()
            .next()
            .filter(|quote| *quote == '"' || *quote == '\'')
            .ok_or_else(|| NotAShape::AnAttributeWithNoValue(name.to_owned()))?;
        let inside = after.get(quote.len_utf8()..).unwrap_or_default();
        let (value, after) = inside
            .split_once(quote)
            .ok_or_else(|| NotAShape::AnAttributeWithNoValue(name.to_owned()))?;
        pairs.push((name, value));
        rest = after;
    }
}

/// `0 0 width height`.
fn a_view_box(value: &str) -> Result<(f32, f32), NotAShape> {
    let numbers = value
        .split(|c: char| c.is_whitespace() || c == ',')
        .filter(|one| !one.is_empty())
        .map(a_number)
        .collect::<Result<Vec<_>, _>>()?;
    match numbers.as_slice() {
        [x, y, wide, tall] if *x == 0.0 && *y == 0.0 && *wide > 0.0 && *tall > 0.0 => {
            Ok((*wide, *tall))
        }
        _ => Err(NotAShape::AViewBoxThisCannotDraw(value.to_owned())),
    }
}

/// A finite number.
fn a_number(value: &str) -> Result<f32, NotAShape> {
    value
        .trim()
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or_else(|| NotAShape::NotANumber(value.to_owned()))
}

/// A finite number above nought.
fn a_positive_number(value: &str) -> Result<f32, NotAShape> {
    let number = a_number(value)?;
    if number > 0.0 {
        Ok(number)
    } else {
        Err(NotAShape::NotANumber(value.to_owned()))
    }
}

/// A paint: [`None`] for `none`, or one colour.
fn a_paint(value: &str) -> Result<Option<[u8; 3]>, NotAShape> {
    let not = || NotAShape::NotAColour(value.to_owned());
    match value {
        "none" => Ok(None),
        "white" => Ok(Some([255, 255, 255])),
        "black" => Ok(Some([0, 0, 0])),
        hex => {
            let digits = hex.strip_prefix('#').ok_or_else(not)?;
            if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
                return Err(not());
            }
            let byte = |at: usize| {
                digits
                    .get(at..at + 2)
                    .and_then(|two| u8::from_str_radix(two, 16).ok())
                    .ok_or_else(not)
            };
            Ok(Some([byte(0)?, byte(2)?, byte(4)?]))
        }
    }
}

/// Path data: starts with a move, and holds only what SVG path data can.
fn path_data(value: &str) -> Result<String, NotAShape> {
    let trimmed = value.trim();
    let starts_with_a_move = trimmed.starts_with('M') || trimmed.starts_with('m');
    let only_path_data = trimmed.chars().all(|c| {
        c.is_ascii_digit()
            || c.is_whitespace()
            || matches!(c, '.' | ',' | '-' | '+' | 'e' | 'E')
            || "MmLlHhVvCcSsQqTtAaZz".contains(c)
    });
    if starts_with_a_move && only_path_data {
        Ok(trimmed.to_owned())
    } else {
        Err(NotAShape::NotPathData(value.to_owned()))
    }
}

#[cfg(test)]
#[path = "reading_tests.rs"]
mod tests;
