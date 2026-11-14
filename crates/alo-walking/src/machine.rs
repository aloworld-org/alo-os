//! **What a walk needs of a machine**, and nothing about how it is had.
//!
//! The walk is written against this trait so that everything above the wire —
//! the waiting, the settling, the checks and their evidence — is tested with a
//! machine made of pictures and log lines, and only `crate::qemu` touches a
//! real one.

use std::time::Duration;

use crate::frame::Frame;
use crate::strokes::Stroke;

/// What went wrong with the machine itself, as a sentence.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{0}")]
pub struct MachineError(pub String);

/// A machine a walk can press keys at, look at and read.
pub trait Machine {
    /// Send one edge of one key.
    ///
    /// # Errors
    ///
    /// When the machine did not take it.
    fn stroke(&mut self, stroke: &Stroke) -> Result<(), MachineError>;

    /// A picture of the screen now, to compare.
    ///
    /// # Errors
    ///
    /// When no picture could be taken or read.
    fn frame(&mut self) -> Result<Frame, MachineError>;

    /// Keep a picture of the screen now as evidence, named `name`, and say the
    /// file it was kept in.
    ///
    /// # Errors
    ///
    /// When no picture could be kept.
    fn keep_picture(&mut self, name: &str) -> Result<String, MachineError>;

    /// Everything the machine's serial port has said since it started.
    ///
    /// # Errors
    ///
    /// When the log cannot be read.
    fn log(&mut self) -> Result<Vec<u8>, MachineError>;

    /// Let time pass.
    fn wait(&mut self, for_how_long: Duration);

    /// How long since the machine started.
    fn elapsed(&self) -> Duration;
}
