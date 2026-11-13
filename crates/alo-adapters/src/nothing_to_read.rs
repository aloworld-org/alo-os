//! **What a person is told about a window that shows no text** — and which
//! switch is the likeliest reason.
//!
//! One sentence for a person's own `alo-app … read` and for alo's read in a
//! turn, so a person is never told something alo would not be. VSCodium's
//! document is readable only in its screen reader mode, which alo turns on for
//! it only where the person let alo work fully inside apps like it
//! (`docs/quirks/vscodium-is-pressed-and-read-only-when-described-completely.md`);
//! for any other application, the likeliest reason is that it is not describing
//! itself at all. Which of the two applies is [`WorkingFully`], the same value
//! a refused press is worded by, decided by the caller that knows the
//! application and reads the person's settings.

use alo_strings::{Filling, Said, Strings};

use crate::fallback_words as words;
use crate::not_pressed::WorkingFully;

/// What a person is told about `window`, of `application`, which shows no text.
#[must_use]
pub fn nothing_to_read(
    window: &str,
    application: &str,
    fully: WorkingFully,
    strings: &Strings,
) -> Said {
    let titled = Filling::of(words::WINDOW, window.to_owned());
    match fully {
        WorkingFully::CouldBeTurnedOn => strings.say(
            &words::NOTHING_TO_READ_UNTIL_ALO_WORKS_FULLY.key(),
            &titled.and(words::APPLICATION, application.to_owned()),
        ),
        WorkingFully::NothingMoreToTurnOn => strings.say(&words::NOTHING_TO_READ.key(), &titled),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    fn in_english() -> Strings {
        Strings::of(crate::words::adapter_words().unwrap())
    }

    /// **Each names the switch that would help, and only that one.**
    #[test]
    fn each_names_the_one_switch_that_would_help() {
        let strings = in_english();
        let fully = nothing_to_read(
            "README.md",
            "com.vscodium.codium",
            WorkingFully::CouldBeTurnedOn,
            &strings,
        );
        assert!(fully.text().contains("“README.md”"), "{}", fully.text());
        assert!(
            fully
                .text()
                .contains("“Let alo work fully inside apps like VSCodium”"),
            "{}",
            fully.text()
        );
        assert!(!fully.text().contains("Apps describe themselves"));
        let described = nothing_to_read(
            "Welcome",
            "org.gnome.TextEditor",
            WorkingFully::NothingMoreToTurnOn,
            &strings,
        );
        assert!(
            described.text().contains("“Welcome”"),
            "{}",
            described.text()
        );
        assert!(
            described.text().contains("Apps describe themselves"),
            "{}",
            described.text()
        );
        assert!(!described.text().contains("Let alo work fully"));
    }
}
