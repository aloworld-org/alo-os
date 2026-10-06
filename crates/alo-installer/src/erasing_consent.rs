//! The second typed consent, on the road that erases a disk: the disk's name
//! **and** the word that names what is lost.
//!
//! Installer plan task 7: *Replace Windows* asks twice, and the second time the
//! person types the name of the disk and the word that names what is lost, in
//! their own language. [`crate::consent`] holds the first consent, which names a
//! disk; this holds the second, which is a different question with a different
//! refusal, and the two live apart so that neither can quietly become the other.
//!
//! # Why both halves, and why in one answer
//!
//! A disk's name on its own can be typed by mistake — it is on the screen, and a
//! person who has just read it can reproduce it without deciding anything. The
//! word cannot be arrived at by copying: it has to be read and meant. Asking for
//! both in one answer means there is no sequence of plausible keystrokes that
//! reaches the destructive road, and no second prompt whose answer could
//! disagree with the first.
//!
//! # The word is translated, and is never English written into the program
//!
//! What arrives here is `ERASING_WORD`, from this crate's `words`, as the
//! reader's language renders it -- resolved by the caller from the vocabulary. A person typing an
//! English word they do not read has not consented to anything, and a European
//! installer that only accepts `erase` has refused everybody who does not speak
//! English.
//!
//! # Forgiving in form, exact in substance, and silent about which half
//!
//! Case and spacing do not matter, exactly as they do not for the first consent —
//! and by the same code, because two ideas of *forgiving* that could drift apart
//! would be a bug. Everything else is a refusal, and the refusal never says which
//! half was wrong: telling a person which half to fix is walking them towards the
//! answer that erases their disk.
//!
//! The order is the order the question asked for — the name, then the word. This
//! is the one place in the installer that is deliberately not tolerant of a
//! reasonable variation, because *strict in what we accept* is the right side to
//! err on when the operation cannot be undone.

use crate::consent::normalised;
use crate::deciding::ForAloOs;
use crate::ended::Refusal;

/// Whether the person typed this disk's name and the word, in that order.
///
/// `word` is `ERASING_WORD`, from this crate's `words`, in the reader's
/// language. It is a private item, so this names it rather than linking to it.
///
/// # Errors
/// [`Refusal::NotAgreed`] for nothing typed — a person who pressed Enter on its
/// own stopped, and stopping is not a mistake to be reported as one.
/// [`Refusal::NotTheWord`] for anything else, whichever half is wrong.
pub fn erasing(typed: &str, disk: &ForAloOs, word: &str) -> Result<(), Refusal> {
    let typed = normalised(typed);
    if typed.is_empty() {
        return Err(Refusal::NotAgreed);
    }
    if typed == normalised(&format!("{} {}", disk.shown, word)) {
        Ok(())
    } else {
        Err(Refusal::NotTheWord {
            disk: disk.shown.clone(),
            word: word.to_owned(),
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_installing::DiskName;

    use super::*;
    use crate::identities::DiskNumber;

    /// The disk this machine would erase.
    fn disk() -> ForAloOs {
        ForAloOs {
            number: DiskNumber(0),
            shown: "Samsung SSD 870 EVO".to_owned(),
            after_the_restart: DiskName::named("ata-Samsung_SSD_870_EVO_S5Y1").unwrap(),
            beside: None,
        }
    }

    /// **The name and the word together are the consent**, whatever their case
    /// and spacing.
    #[test]
    fn the_name_and_the_word_together_are_the_consent() {
        for typed in [
            "Samsung SSD 870 EVO erase",
            "samsung ssd 870 evo ERASE",
            "  Samsung   SSD 870   EVO   erase  \r\n",
        ] {
            assert_eq!(erasing(typed, &disk(), "erase"), Ok(()), "{typed}");
        }
    }

    /// **The word is whatever this language's word is**, and English is not
    /// special.
    ///
    /// A Dutch reader types the Dutch word, and `erase` is then no longer the
    /// consent — which is the point: the sentence asked for a word they read.
    #[test]
    fn the_word_is_this_languages_word_and_english_is_not_special() {
        assert_eq!(
            erasing("Samsung SSD 870 EVO wissen", &disk(), "wissen"),
            Ok(())
        );
        assert!(erasing("Samsung SSD 870 EVO erase", &disk(), "wissen").is_err());
    }

    /// **Nothing typed is a person who stopped**, not a person who got it wrong.
    #[test]
    fn nothing_typed_is_a_person_who_stopped() {
        assert_eq!(erasing("", &disk(), "erase"), Err(Refusal::NotAgreed));
        assert_eq!(erasing("  \r\n", &disk(), "erase"), Err(Refusal::NotAgreed));
    }

    /// **Either half wrong is a refusal, and it never says which.**
    ///
    /// Every one of these is the same refusal carrying the same two fields: the
    /// name alone, the word alone, the word before the name, a near miss of each
    /// half, and the answers a person gives a machine they expect to nag them.
    #[test]
    fn either_half_wrong_is_one_refusal_that_never_says_which() {
        let disk = disk();
        for typed in [
            "Samsung SSD 870 EVO",
            "erase",
            "erase Samsung SSD 870 EVO",
            "Samsung SSD 870 erase",
            "Samsung SSD 870 EVO Plus erase",
            "Samsung SSD 870 EVO erases",
            "Samsung SSD 870 EVO delete",
            "Samsung SSD 870 EVO, erase",
            "yes",
            "y",
            "I agree",
            "Samsung SSD 870 EVO erase erase",
        ] {
            assert_eq!(
                erasing(typed, &disk, "erase"),
                Err(Refusal::NotTheWord {
                    disk: "Samsung SSD 870 EVO".to_owned(),
                    word: "erase".to_owned(),
                }),
                "{typed}"
            );
        }
    }
}
