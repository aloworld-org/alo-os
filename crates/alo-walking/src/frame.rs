//! **One picture of the screen**, as qemu's `screendump` writes it — a binary
//! PPM (`P6`) — and how many pixels two pictures differ by.
//!
//! The comparison is of pixels, not of files. The shell walks learned that
//! equal PNG sizes are suggestive and not proof, and that counting ink is the
//! wrong measure entirely (a ring on row one and on row two have the same
//! area); a count of differing pixels is neither.

/// A picture: its size and its pixels, three bytes each.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    /// Width in pixels.
    width: usize,
    /// Height in pixels.
    height: usize,
    /// Red, green, blue for each pixel, row by row.
    pixels: Vec<u8>,
}

/// A file that is not a picture this reads.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("not a binary PPM qemu writes: {0}")]
pub struct NotAPicture(pub &'static str);

impl Frame {
    /// A picture of one colour, for tests and for nothing else.
    #[must_use]
    pub fn filled(width: usize, height: usize, rgb: [u8; 3]) -> Self {
        Self {
            width,
            height,
            pixels: rgb.repeat(width * height),
        }
    }

    /// The same picture with one pixel changed.
    #[must_use]
    pub fn with_pixel(mut self, x: usize, y: usize, rgb: [u8; 3]) -> Self {
        let at = (y * self.width + x) * 3;
        if let Some(pixel) = self.pixels.get_mut(at..at + 3) {
            pixel.copy_from_slice(&rgb);
        }
        self
    }

    /// Read a binary PPM with a maximum value of 255, as qemu writes one.
    ///
    /// # Errors
    ///
    /// [`NotAPicture`] when the header is not `P6 <w> <h> 255` or the pixels
    /// are fewer than it says.
    pub fn from_ppm(bytes: &[u8]) -> Result<Self, NotAPicture> {
        let mut fields = Vec::new();
        let mut at = 0;
        while fields.len() < 4 {
            while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                at += 1;
            }
            let start = at;
            while bytes.get(at).is_some_and(|it| !it.is_ascii_whitespace()) {
                at += 1;
            }
            let field = bytes.get(start..at).ok_or(NotAPicture("a short header"))?;
            if field.is_empty() {
                return Err(NotAPicture("a short header"));
            }
            fields.push(field);
        }
        // Exactly one whitespace byte separates the header from the pixels.
        at += 1;
        let [magic, width, height, most] = fields.as_slice() else {
            return Err(NotAPicture("a short header"));
        };
        if *magic != b"P6" {
            return Err(NotAPicture("not P6"));
        }
        if *most != b"255" {
            return Err(NotAPicture("not eight bits a channel"));
        }
        let width = number(width)?;
        let height = number(height)?;
        let length = width * height * 3;
        let pixels = bytes
            .get(at..at + length)
            .ok_or(NotAPicture("fewer pixels than the header says"))?
            .to_vec();
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    /// Width and height.
    #[must_use]
    pub fn size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// How many pixels differ from another picture — every pixel, when the
    /// two are not the same size, because a screen that changed size changed.
    #[must_use]
    pub fn differs_from(&self, other: &Self) -> usize {
        if self.size() != other.size() {
            return self.width.max(other.width) * self.height.max(other.height);
        }
        self.pixels
            .chunks_exact(3)
            .zip(other.pixels.chunks_exact(3))
            .filter(|(one, two)| one != two)
            .count()
    }
}

/// A decimal number in a header.
fn number(field: &[u8]) -> Result<usize, NotAPicture> {
    std::str::from_utf8(field)
        .ok()
        .and_then(|text| text.parse().ok())
        .filter(|it| *it > 0 && *it <= 16_384)
        .ok_or(NotAPicture("a size that is not a number"))
}

#[cfg(test)]
#[path = "frame_tests.rs"]
mod tests;
