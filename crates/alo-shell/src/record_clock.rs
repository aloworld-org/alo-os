//! **When the record is read from, and how its times are written.**
//!
//! Frame `82:887` *History / Open* puts the time of day at the head of every
//! entry — *17:38  alo shortened the introduction* — and names the day above
//! them: *European launch · Today*. The owner's decision of 2026-10-28 carries
//! that to every day the record holds: a heading wherever the day changes,
//! *Today*, *Yesterday*, or the date.
//!
//! Both are the person's: the time as their region writes one, the day where
//! they are, and *today* measured from the moment this frame was drawn. So the
//! window is handed the person's [`TheirFormats`] and that moment, and decides
//! neither — `alo_formats::TheirFormats` is the one answer the status area's
//! clock asks too, so the clock and the record cannot disagree about what time
//! something happened.

use std::time::SystemTime;

use alo_formats::TheirFormats;

/// The person's way of writing a moment, and the moment the record is read at.
#[derive(Debug, Clone, Copy)]
pub struct RecordClock<'a> {
    /// How this person writes a time and a day, and where they are.
    pub formats: &'a TheirFormats,
    /// Now, which is what *today* and *yesterday* are measured from.
    pub now: SystemTime,
}
