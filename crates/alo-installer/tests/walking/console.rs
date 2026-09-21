//! What the guest said, read off its serial line.
//!
//! **A machine that is still running is not evidence that anything happened.**
//! Everything this walk believes about the guest comes from a line the guest
//! itself printed on `COM1`, which QEMU writes to a file on the host: the
//! session table and the shell's own process for *it reached a desktop*, the
//! manifest's digest for *its files are what they were*, and what Windows' own
//! tools printed for everything else.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// The line the guest's every-start script prints first.
pub const BEGINS: &str = "ALOWALK-BEGIN";

/// The line it prints when the boot has done what it was told.
pub const DONE: &str = "ALOWALK-DONE";

/// The line it prints when it found nothing to do.
pub const NOTHING_TO_DO: &str = "ALOWALK-NO-INSTRUCTION";

/// The serial line, as a file on the host.
#[derive(Debug, Clone)]
pub struct Console(PathBuf);

impl Console {
    /// The line kept at this path, emptied first so nothing an earlier boot
    /// said can be read as this one's.
    ///
    /// # Panics
    /// When the file cannot be emptied.
    #[must_use]
    pub fn fresh(at: &Path) -> Self {
        let _ = std::fs::remove_file(at);
        std::fs::write(at, b"").expect("an empty console");
        Self(at.to_path_buf())
    }

    /// Where it is, for QEMU.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Everything said so far, with the firmware's terminal control taken out
    /// so a line can be matched.
    #[must_use]
    pub fn said(&self) -> String {
        let bytes = std::fs::read(&self.0).unwrap_or_default();
        let text = String::from_utf8_lossy(&bytes);
        let mut plain = String::with_capacity(text.len());
        let mut in_escape = false;
        for character in text.chars() {
            match character {
                '\u{1b}' => in_escape = true,
                _ if in_escape => {
                    if character.is_ascii_alphabetic() {
                        in_escape = false;
                    }
                }
                '\0' => {}
                _ => plain.push(character),
            }
        }
        plain
    }

    /// Wait until the guest says one of these, and say which — or nothing when
    /// the wait runs out.
    #[must_use]
    pub fn wait_for(&self, any_of: &[&str], patience: Duration) -> Option<String> {
        let until = Instant::now() + patience;
        while Instant::now() < until {
            let said = self.said();
            if let Some(found) = any_of.iter().find(|line| said.contains(**line)) {
                return Some((*found).to_owned());
            }
            std::thread::sleep(Duration::from_secs(2));
        }
        None
    }

    /// The one line beginning with this, and what follows it.
    #[must_use]
    pub fn after(&self, beginning: &str) -> Option<String> {
        self.said()
            .lines()
            .find_map(|line| line.trim().strip_prefix(beginning).map(str::to_owned))
    }

    /// Whether the guest said this, anywhere.
    #[must_use]
    pub fn contains(&self, line: &str) -> bool {
        self.said().contains(line)
    }
}
