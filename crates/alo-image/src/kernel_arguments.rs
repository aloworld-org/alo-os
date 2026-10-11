//! The kernel arguments a deployment boots with, and whether a boot can speak.
//!
//! `bootc` reads every `*.toml` under `/usr/lib/bootc/kargs.d` and adds what
//! they name to the kernel command line of each deployment it writes. This
//! reads the same files to answer one question: **can somebody read what
//! happened to this machine, or only photograph it?**
//!
//! # Why this is a check and not a comment
//!
//! Two boots on 2026-10-10 and 2026-10-11 were diagnosed from screenshots.
//! GRUB writes to a serial line of its own accord, so the log ended at the
//! entry it was about to boot and everything Linux did afterwards went to the
//! screen alone. A machine that cannot say what happened to it is one whose
//! faults are found by looking at pixels, and the line that fixes it is one
//! line in one file - exactly the kind that is deleted by somebody tidying up
//! who does not know what it was for.
//!
//! # Read leniently
//!
//! A file that is not TOML, or names no arguments, is an image
//! `crate::checking` has a sentence about rather than a directory that is no
//! image at all - the same rule the recipe is read under.

use serde::Deserialize;

/// What `bootc` is told to put on the kernel command line.
#[derive(Debug, Default, Deserialize)]
struct Written {
    /// The arguments this file adds.
    #[serde(default)]
    kargs: Vec<String>,
}

/// The kernel arguments this image's deployments carry.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TheKernelArguments {
    /// Every argument named, in the order the file names them.
    arguments: Vec<String>,
}

impl TheKernelArguments {
    /// Read one `kargs.d` file.
    ///
    /// Anything that is not TOML, and TOML that names nothing, both answer an
    /// empty set rather than a failure.
    #[must_use]
    pub fn read(text: &str) -> Self {
        let written: Written = toml::from_str(text).unwrap_or_default();
        Self {
            arguments: written.kargs,
        }
    }

    /// Every argument, as written.
    #[must_use]
    pub fn every_one(&self) -> &[String] {
        &self.arguments
    }

    /// The consoles named, in the order they are given.
    ///
    /// Order is the whole of what matters here, which is why this answers a
    /// list rather than a set: the kernel makes the **last** `console=` its
    /// primary one, and the others receive a copy.
    #[must_use]
    pub fn consoles(&self) -> Vec<&str> {
        self.arguments
            .iter()
            .filter_map(|one| one.strip_prefix("console="))
            .collect()
    }

    /// Whether anything but the screen is given the kernel's messages.
    ///
    /// A serial console is the one a reader can keep. `tty0` is the screen and
    /// `ttyS0` is a line; this asks for at least one that is not the screen.
    #[must_use]
    pub fn a_reader_can_keep_the_boot(&self) -> bool {
        self.consoles().iter().any(|one| !one.starts_with("tty0"))
    }

    /// The console the kernel makes primary, which is the last one named.
    #[must_use]
    pub fn the_one_a_person_watches(&self) -> Option<&str> {
        self.consoles().last().copied()
    }

    /// Whether the whole journal reaches the console, or only early boot.
    ///
    /// **A serial console alone is half the answer**, which the first boot
    /// carrying one measured: systemd writes unit messages to the console
    /// until `systemd-journald` takes over, and after that - 95 seconds in,
    /// on that machine - the console hears nothing about any unit. The
    /// greeter, the desktop, the portal and the application install all start
    /// after it, so a log without this carries the kernel and the first
    /// minute and a half, and nothing about the surfaces this machine exists
    /// to draw.
    #[must_use]
    pub fn the_whole_journal_is_forwarded(&self) -> bool {
        self.arguments
            .iter()
            .any(|one| one == "systemd.journald.forward_to_console=1")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The pair this image ships**, read as `bootc` reads it.
    #[test]
    fn the_shipped_pair_gives_a_reader_a_line_and_a_person_the_screen() {
        let read =
            TheKernelArguments::read("kargs = [\"console=ttyS0,115200\", \"console=tty0\"]\n");

        assert_eq!(read.consoles(), vec!["ttyS0,115200", "tty0"]);
        assert!(read.a_reader_can_keep_the_boot());
        assert_eq!(read.the_one_a_person_watches(), Some("tty0"));
    }

    /// **The screen alone is a boot nobody can keep**, which is the state
    /// every alo OS deployment was in until 2026-10-11.
    #[test]
    fn the_screen_alone_cannot_be_kept() {
        let read = TheKernelArguments::read("kargs = [\"console=tty0\"]\n");

        assert!(!read.a_reader_can_keep_the_boot());
    }

    /// **No arguments at all is the same answer**, and is what an image with
    /// no `kargs.d` file in it has.
    #[test]
    fn nothing_named_is_nothing_to_read() {
        let read = TheKernelArguments::read("");

        assert!(read.every_one().is_empty());
        assert!(!read.a_reader_can_keep_the_boot());
        assert_eq!(read.the_one_a_person_watches(), None);
    }

    /// **The order is kept**, because the order is the decision: a pair
    /// written the other way round makes the serial line primary and the
    /// screen the copy, which is the opposite of what a person watching wants.
    #[test]
    fn the_order_the_file_writes_is_the_order_read_back() {
        let read =
            TheKernelArguments::read("kargs = [\"console=tty0\", \"console=ttyS0,115200\"]\n");

        assert_eq!(read.the_one_a_person_watches(), Some("ttyS0,115200"));
    }

    /// **Something that is not TOML is an empty answer and not a panic**, for
    /// the reason the header gives: the recipe is read the same way.
    #[test]
    fn text_that_is_not_toml_answers_nothing() {
        let read = TheKernelArguments::read("console=ttyS0 this is not toml");

        assert!(read.every_one().is_empty());
    }

    /// **An argument that is not a console is kept and not mistaken for one.**
    #[test]
    fn an_argument_that_is_not_a_console_is_not_counted_as_one() {
        let read = TheKernelArguments::read(
            "kargs = [\"quiet\", \"console=ttyS0,115200\", \"rd.shell=0\"]\n",
        );

        assert_eq!(read.every_one().len(), 3);
        assert_eq!(read.consoles(), vec!["ttyS0,115200"]);
    }
}
