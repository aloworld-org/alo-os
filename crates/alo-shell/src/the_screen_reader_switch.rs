//! **The *Screen reader* switch, carried from Settings to the desktop and the
//! desktop's answer carried back** —
//! [ADR 0151](../../../docs/decisions/0151-alo-ships-a-screen-reader-and-its-voices-stay-on-the-machine.md),
//! the owner's *"Ship Orca"* of 2026-11-12.
//!
//! Settings keeps the switch in the person's `access.toml`
//! (`alo_access::Setting::ScreenReader`). This compositor starts nothing itself:
//! starting a program in the person's session is the desktop's
//! (`alo-desktop`'s `starting_the_screen_reader`), so the switch is held here
//! until the desktop takes it on the next frame, as *Apps describe themselves*
//! is (`crate::apps_describing_themselves`). It is held **at sign-in too**, with
//! whatever the person's file says, so a person who left it on is read to from
//! the first frame.
//!
//! The desktop answers on its own thread, and an answer that is not *done* is
//! said under the switch, in the person's language
//! ([`TheScreenReaderAnswered::said`]). **The switch stays as the person set
//! it** whatever the answer — law 5: a reader that could not start is tried
//! again at the next sign-in, and a person who turned it on is never quietly
//! turned off.

use alo_strings::Strings;

use crate::the_settings_words as w;

impl crate::Server {
    /// Whether the screen reader is to run, held for the desktop.
    pub(crate) const fn the_screen_reader_is_now(&mut self, on: bool) {
        self.screen_reader_to_tell = Some(on);
    }

    /// Whether the screen reader is to run, taken: [`None`] when nothing was
    /// said since the desktop last asked.
    pub(crate) const fn the_screen_reader_to_tell(&mut self) -> Option<bool> {
        self.screen_reader_to_tell.take()
    }
}

/// **What the desktop answered when it was asked to start or stop the screen
/// reader.**
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheScreenReaderAnswered {
    /// It is running and stayed running.
    Started,
    /// It is not running.
    Stopped,
    /// It was asked to start, and the reader or its unit is not on this
    /// computer — a fault in how the machine was made.
    NotOnThisComputer,
    /// It was asked to start, and did not, or fell over at once.
    DidNotStart,
    /// It was asked to stop, and is still running.
    DidNotStop,
}

impl TheScreenReaderAnswered {
    /// **What Settings says under the switch**, in the person's language:
    /// [`None`] where it did what it was asked, because a switch that worked
    /// needs no sentence beside it.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Option<String> {
        let word = match self {
            Self::Started | Self::Stopped => return None,
            Self::NotOnThisComputer => w::SCREEN_READER_NOT_HERE,
            Self::DidNotStart => w::SCREEN_READER_DID_NOT_START,
            Self::DidNotStop => w::SCREEN_READER_DID_NOT_STOP,
        };
        Some(crate::settings_page::said(strings, word))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **A switch said is handed over once**, and nothing is handed where
    /// nothing was said.
    #[cfg(target_os = "linux")]
    #[test]
    fn the_switch_is_handed_to_the_desktop_once() {
        use std::os::unix::fs::PermissionsExt as _;
        let Ok(directory) = tempfile::tempdir() else {
            unreachable!("no private directory");
        };
        assert!(
            std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
                .is_ok()
        );
        let Ok(mut server) = crate::Server::bind(directory.path(), "reading-aloud") else {
            unreachable!("no bound socket");
        };
        assert_eq!(server.the_screen_reader_to_tell(), None);
        server.the_screen_reader_is_now(true);
        server.the_screen_reader_is_now(false);
        assert_eq!(
            server.the_screen_reader_to_tell(),
            Some(false),
            "the last thing said is what the desktop does"
        );
        assert_eq!(server.the_screen_reader_to_tell(), None, "handed once");
    }

    /// **What worked says nothing, and every way it did not work says so** —
    /// each in its own sentence, and none naming the engine underneath.
    #[test]
    fn every_failure_is_said_and_success_is_silent() {
        let strings = crate::settings_testing::settings_words();
        assert_eq!(TheScreenReaderAnswered::Started.said(&strings), None);
        assert_eq!(TheScreenReaderAnswered::Stopped.said(&strings), None);
        let failures = [
            TheScreenReaderAnswered::NotOnThisComputer,
            TheScreenReaderAnswered::DidNotStart,
            TheScreenReaderAnswered::DidNotStop,
        ];
        let said: Vec<String> = failures
            .iter()
            .map(|answer| answer.said(&strings).unwrap_or_default())
            .collect();
        for (answer, sentence) in failures.iter().zip(&said) {
            assert!(!sentence.is_empty(), "{answer:?} says nothing");
            let lower = sentence.to_lowercase();
            for never in ["orca", "espeak", "speech-dispatcher", "at-spi", "systemd"] {
                assert!(!lower.contains(never), "{answer:?} names {never}");
            }
        }
        assert_ne!(said[0], said[1]);
        assert_ne!(said[1], said[2]);
        // **A reader that did not start is one the person is told is still
        // switched on**, so they know it will be tried again.
        assert!(said[1].contains("switch stays on"));
    }
}
