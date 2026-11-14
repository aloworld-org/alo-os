//! **A machine made of pictures and log lines**, for the tests: it shows the
//! pictures it was given in turn, says the log lines it was given at the times
//! it was given, and remembers every stroke sent to it. Its clock moves only
//! when somebody waits.

use std::collections::VecDeque;
use std::time::Duration;

use crate::frame::Frame;
use crate::machine::{Machine, MachineError};
use crate::strokes::Stroke;

/// The fake machine.
#[derive(Debug, Default)]
pub struct PicturesAndLines {
    /// The pictures still to show; the last is shown for ever once reached.
    pictures: VecDeque<Frame>,
    /// The picture shown when no more are queued.
    showing: Option<Frame>,
    /// Log lines, each with when it is said.
    lines: Vec<(Duration, String)>,
    /// Every stroke sent, in order.
    pub strokes: Vec<Stroke>,
    /// The names of the pictures kept, in order.
    pub kept: Vec<String>,
    /// The clock.
    now: Duration,
}

impl PicturesAndLines {
    /// A machine that shows these pictures in turn.
    #[must_use]
    pub fn showing(pictures: Vec<Frame>) -> Self {
        Self {
            pictures: pictures.into(),
            ..Self::default()
        }
    }

    /// And says this log line once its clock reaches `at`.
    #[must_use]
    pub fn saying(mut self, at: Duration, line: &str) -> Self {
        self.lines.push((at, line.to_owned()));
        self
    }

    /// The keys pressed down, in order — the strokes with `down`.
    #[must_use]
    pub fn keys_pressed(&self) -> Vec<&'static str> {
        self.strokes
            .iter()
            .filter(|it| it.down)
            .map(|it| it.key)
            .collect()
    }
}

impl Machine for PicturesAndLines {
    fn stroke(&mut self, stroke: &Stroke) -> Result<(), MachineError> {
        self.strokes.push(stroke.clone());
        Ok(())
    }

    fn frame(&mut self) -> Result<Frame, MachineError> {
        if let Some(next) = self.pictures.pop_front() {
            self.showing = Some(next);
        }
        self.showing
            .clone()
            .ok_or_else(|| MachineError("no picture to show".to_owned()))
    }

    fn keep_picture(&mut self, name: &str) -> Result<String, MachineError> {
        self.kept.push(name.to_owned());
        Ok(format!("{name}.png"))
    }

    fn log(&mut self) -> Result<Vec<u8>, MachineError> {
        let mut said = String::new();
        for (at, line) in &self.lines {
            if *at <= self.now {
                said.push_str(line);
                said.push('\n');
            }
        }
        Ok(said.into_bytes())
    }

    fn wait(&mut self, for_how_long: Duration) {
        self.now += for_how_long;
    }

    fn elapsed(&self) -> Duration {
        self.now
    }
}
