//! **A screen with alo's pointer on it**, for the tests: patches of colour, each
//! asking for a cursor, and the pointer drawn over them by the real artwork
//! where it was put — a little off, as a guest's own mapping from qemu's range
//! may put it.
//!
//! It can also be made wrong in the ways the walk has to catch: drawing no
//! pointer at all, as a capture without the cursor plane would show, or
//! drawing another look than the person chose.

use std::time::Duration;

use alo_cursors::CursorIcon;

use crate::a_pointer_drawn_over::drawn_over;
use crate::frame::Frame;
use crate::machine::{Machine, MachineError};
use crate::strokes::Stroke;
use crate::the_pointer_seen::{Drawn, Look};

/// A patch of the screen: where, its colour, and the cursor it asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Patch {
    /// Left, top, width and height.
    pub area: (usize, usize, usize, usize),
    /// Its colour.
    pub rgb: [u8; 3],
    /// The cursor drawn over it.
    pub icon: CursorIcon,
}

/// The fake machine.
#[derive(Debug, Clone)]
pub struct AScreenWithAPointer {
    /// Width and height.
    pub size: (usize, usize),
    /// What is under everything.
    pub background: [u8; 3],
    /// The patches, the last on top.
    pub patches: Vec<Patch>,
    /// Patches that appear when a key is first pressed, as a window opens.
    pub on_a_key: Vec<Patch>,
    /// The look the pointer is drawn in.
    pub look: Look,
    /// Whether the pointer is in the picture at all.
    pub draws_the_pointer: bool,
    /// Where the hotspot is.
    pub at: (i64, i64),
    /// Where the pointer lands, as a percentage of where it was put.
    pub lands_at: i64,
    /// Every place the pointer was put, in order.
    pub pointed: Vec<(usize, usize)>,
    /// Every edge of the left button, down or up, in order.
    pub buttons: Vec<bool>,
    /// Every stroke sent, in order.
    pub strokes: Vec<Stroke>,
    /// The names of the pictures kept.
    pub kept: Vec<String>,
    /// The clock.
    pub now: Duration,
}

impl AScreenWithAPointer {
    /// A grey screen 640 by 400 with the pointer in the shipped look at
    /// `(100, 100)`, landing where it is put.
    #[must_use]
    pub fn grey() -> Self {
        Self {
            size: (640, 400),
            background: [128, 128, 128],
            patches: Vec::new(),
            on_a_key: Vec::new(),
            look: Look::SHIPPED,
            draws_the_pointer: true,
            at: (100, 100),
            lands_at: 100,
            pointed: Vec::new(),
            buttons: Vec::new(),
            strokes: Vec::new(),
            kept: Vec::new(),
            now: Duration::ZERO,
        }
    }

    /// And a patch.
    #[must_use]
    pub fn with(mut self, patch: Patch) -> Self {
        self.patches.push(patch);
        self
    }

    /// Where the hotspot is, on the screen.
    fn hotspot(&self) -> (usize, usize) {
        let clamp = |it: i64, most: usize| {
            usize::try_from(it.max(0))
                .unwrap_or(0)
                .min(most.saturating_sub(1))
        };
        (clamp(self.at.0, self.size.0), clamp(self.at.1, self.size.1))
    }

    /// The cursor asked for where the hotspot is.
    fn icon_here(&self) -> CursorIcon {
        let (x, y) = self.hotspot();
        self.patches
            .iter()
            .rev()
            .find(|it| {
                let (left, top, width, height) = it.area;
                (left..left + width).contains(&x) && (top..top + height).contains(&y)
            })
            .map_or(CursorIcon::Default, |it| it.icon)
    }
}

impl Machine for AScreenWithAPointer {
    fn stroke(&mut self, stroke: &Stroke) -> Result<(), MachineError> {
        self.strokes.push(stroke.clone());
        self.patches.append(&mut self.on_a_key);
        Ok(())
    }

    fn point_at(&mut self, at: (usize, usize)) -> Result<(), MachineError> {
        self.pointed.push(at);
        let landed = |it: usize| i64::try_from(it).unwrap_or(i64::MAX) * self.lands_at / 100;
        self.at = (landed(at.0), landed(at.1));
        Ok(())
    }

    fn button(&mut self, down: bool) -> Result<(), MachineError> {
        self.buttons.push(down);
        Ok(())
    }

    fn frame(&mut self) -> Result<Frame, MachineError> {
        let (width, height) = self.size;
        let mut frame = Frame::filled(width, height, self.background);
        for patch in &self.patches {
            let (left, top, w, h) = patch.area;
            for y in top..(top + h).min(height) {
                for x in left..(left + w).min(width) {
                    frame = frame.with_pixel(x, y, patch.rgb);
                }
            }
        }
        if self.draws_the_pointer {
            let drawn = Drawn {
                icon: self.icon_here(),
                look: self.look,
                scale: 100,
            };
            frame = drawn_over(frame, drawn, self.hotspot());
        }
        Ok(frame)
    }

    fn keep_picture(&mut self, name: &str) -> Result<String, MachineError> {
        self.kept.push(name.to_owned());
        Ok(format!("{name}.png"))
    }

    fn log(&mut self) -> Result<Vec<u8>, MachineError> {
        Ok(Vec::new())
    }

    fn wait(&mut self, for_how_long: Duration) {
        self.now += for_how_long;
    }

    fn elapsed(&self) -> Duration {
        self.now
    }
}
