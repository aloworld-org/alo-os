//! Write a frame a fixture drew to a file somebody can open, and nothing else.
//!
//! # This is not the screenshot promise, and the distinction is the scope gate
//!
//! `docs/features.md` carries, at `[v0.5]`, *Screenshots: whole screen, one
//! window, a selected region — to a file or the clipboard*. **This file is not
//! the beginning of that and must not be read as it.** The difference is not
//! the format or the code, which would look much the same; it is who can ask:
//!
//! - nothing in `alo-shell`'s public surface calls this. It is an example's
//!   helper, compiled only into `desktop_check` and only when that fixture is
//!   run by hand with `--save-to`;
//! - no agent verb, no D-Bus method, no server request and no key reaches it.
//!   `alo-shell`'s readback says of the pixels it hands back that *it
//!   exposes no server request, agent verb, screenshot or background capture*,
//!   and putting those pixels in a file here does not give anybody a way to
//!   ask for them;
//! - a signed-in session never takes the readback at all.
//!   `alo_shell::Nested::keep_each_frame` is off until a fixture turns it on.
//!
//! The screenshot promise, when it is built, is a person's own capture with a
//! surface, a grant and a record. This is a developer looking at what the
//! renderer drew.
//!
//! # What the bytes are
//!
//! `alo_shell::ScanoutPixels` is tightly packed `B,G,R,0`, rows top to bottom
//! — the row order already resolved by whoever did the readback, which is why
//! this takes the frame rather than a buffer and a guess. PNG wants `R,G,B`, so
//! each pixel is reordered and the fourth byte dropped: alpha is discarded
//! after rendering, so there is none to keep.

/// Where a fixture was asked to put the frames, if it was asked at all.
///
/// Parsed from the process arguments rather than an environment variable so
/// that a person who gets it wrong is told, at the start, by a fixture that
/// then does nothing. An environment variable nobody spelled right stays
/// silent and the run looks like it worked.
pub struct SaveTo {
    /// The folder each frame is written into. Created if it is not there.
    folder: std::path::PathBuf,
    /// How many frames have been written, so each gets its own number and the
    /// files sort into the order they were drawn in.
    written: usize,
}

impl SaveTo {
    /// Read `--save-to <folder>` out of the arguments this process was given.
    ///
    /// [`None`] when the flag is absent, which is every ordinary run.
    ///
    /// # Errors
    /// The flag with nothing after it, an unknown argument, or a folder that
    /// cannot be made. Each is a person's mistake at the start of a run, and
    /// being told beats a fixture that draws for two minutes and saves nothing.
    pub fn from_arguments() -> Result<Option<Self>, Box<dyn std::error::Error>> {
        let mut arguments = std::env::args().skip(1);
        let mut folder = None;
        while let Some(argument) = arguments.next() {
            match argument.as_str() {
                "--save-to" => {
                    let named = arguments
                        .next()
                        .ok_or("--save-to needs a folder after it")?;
                    folder = Some(std::path::PathBuf::from(named));
                }
                other => {
                    return Err(format!("unknown argument {other}; try --save-to <folder>").into());
                }
            }
        }
        let Some(folder) = folder else {
            return Ok(None);
        };
        std::fs::create_dir_all(&folder)?;
        Ok(Some(Self { folder, written: 0 }))
    }

    /// The folder being written into, for a fixture that wants to name it.
    pub fn folder(&self) -> &std::path::Path {
        &self.folder
    }

    /// How many frames have been written so far.
    pub fn written(&self) -> usize {
        self.written
    }

    /// Write one drawn frame, named after what it shows.
    ///
    /// The name is `NN-what-it-shows.png`, numbered from one in the order the
    /// frames were drawn, because a folder of thirty-odd pictures is only
    /// useful if it sorts into the order a person would walk them in.
    ///
    /// # Errors
    /// A frame whose extent does not fit its bytes, and any failure to write.
    pub fn frame(
        &mut self,
        shows: &str,
        frame: &alo_shell::ScanoutPixels,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let (width, height) = frame.size();
        let rgb = rgb_from_bgrx(frame.pixels(), width, height)?;
        self.written += 1;
        let at = self
            .folder
            .join(format!("{:02}-{}.png", self.written, named(shows)));
        image::save_buffer(&at, &rgb, width, height, image::ColorType::Rgb8)?;
        Ok(at)
    }
}

/// Three bytes a PNG can carry from four a renderer handed back.
///
/// Refuses a length that is not this extent's rather than writing a picture
/// made partly of whatever came next, which is the failure that looks like a
/// rendering bug for an afternoon.
fn rgb_from_bgrx(
    pixels: &[u8],
    width: u32,
    height: u32,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let count = (width as usize)
        .checked_mul(height as usize)
        .ok_or("a frame extent that does not fit in memory")?;
    if pixels.len() != count * 4 {
        return Err(format!(
            "{width}x{height} wants {} bytes and the frame has {}",
            count * 4,
            pixels.len()
        )
        .into());
    }
    let mut rgb = Vec::with_capacity(count * 3);
    for [blue, green, red, _] in pixels.as_chunks::<4>().0 {
        rgb.extend_from_slice(&[*red, *green, *blue]);
    }
    Ok(rgb)
}

/// A file name out of a sentence: lower case, words joined by hyphens.
///
/// Anything that is not a letter or a digit becomes a hyphen, and runs of them
/// collapse, so *the dock with a question leaving, 9am* becomes
/// `the-dock-with-a-question-leaving-9am` on every file system this runs on.
fn named(shows: &str) -> String {
    let mut name = String::new();
    for character in shows.chars() {
        if character.is_ascii_alphanumeric() {
            name.extend(character.to_lowercase());
        } else if !name.ends_with('-') {
            name.push('-');
        }
    }
    name.trim_matches('-').to_owned()
}
