//! What a long program complained of, kept to its last lines.
//!
//! The writer runs for a quarter of an hour and prints to its standard error
//! as it goes — progress, warnings, and, if it fails, why. All of it goes to the
//! machine's log as it arrives. What is **kept** is the end, because a program
//! that stops says why last, and a machine that holds a whole quarter of an
//! hour of it in memory is holding it in the memory the install needs.

use std::collections::VecDeque;

/// How many of the last lines are kept.
///
/// A failure is said in a line or two; the lines before it are what the program
/// was doing when it stopped, which is the next thing anybody reading it asks.
pub const THE_LAST_LINES: usize = 12;

/// The last lines a program complained of.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Complaint {
    /// At most [`THE_LAST_LINES`], oldest first.
    kept: VecDeque<String>,
}

impl Complaint {
    /// Nothing complained of yet.
    #[must_use]
    pub fn nothing() -> Self {
        Self::default()
    }

    /// One more line, as the program printed it.
    ///
    /// Its line ending is taken off, a blank line is not kept, and once
    /// [`THE_LAST_LINES`] are held the oldest is let go.
    pub fn heard(&mut self, line: &str) {
        let line = line.trim_end();
        if line.is_empty() {
            return;
        }
        if self.kept.len() == THE_LAST_LINES {
            drop(self.kept.pop_front());
        }
        self.kept.push_back(line.to_owned());
    }

    /// The lines kept, one to a line, oldest first.
    #[must_use]
    pub fn said(&self) -> String {
        self.kept.iter().cloned().collect::<Vec<_>>().join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The end is kept, in order, and nothing blank.
    #[test]
    fn the_last_lines_are_kept_in_order() {
        let mut complaint = Complaint::nothing();
        assert_eq!(complaint.said(), "");
        for at in 0..30 {
            complaint.heard(&format!("line {at}\r\n"));
            complaint.heard("   \n");
        }
        complaint.heard("error: Installing to disk: it stopped");
        let said = complaint.said();
        let lines: Vec<&str> = said.lines().collect();
        assert_eq!(lines.len(), THE_LAST_LINES);
        assert_eq!(lines.first(), Some(&"line 19"));
        assert_eq!(lines.last(), Some(&"error: Installing to disk: it stopped"));
    }
}
