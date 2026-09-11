//! The list a surface would show, derived from the machine's own grants.
//!
//! ADR 0001 §3 makes visibility part of the grant model itself: grants are
//! *enumerated, visible where the person can find them, revocable*, and a
//! grant a person cannot find is a grant they cannot revoke. `alo-capability`
//! has held the enumeration from the start — [`Grants::active_at`] is the
//! person's question, asked with the same clock the daemon's `permits` is
//! asked with — and nothing had ever shaped its answer for a screen.
//!
//! # Derived, at one moment, and never assembled
//!
//! [`Listing::of`] is the only door, and it takes the machine's [`Grants`] —
//! the same value the daemon asks, whether it was granted this session or
//! read back off the disk by `alo-remembering`. Every row is a
//! [`crate::Seen`], which has no constructor a caller can reach, so a listing
//! cannot be padded with a grant the machine does not hold; and the rows are
//! taken through `alo_capability::Grant::expires_in` at one `now`, so an
//! expired grant is never shown as live — not because a filter remembered to
//! run, but because an expired grant cannot become a row at all.
//!
//! `now` is passed in rather than read from the clock, as everywhere else in
//! this repository: what the person is shown and what the daemon enforces
//! must not be able to disagree about when something was true.
//!
//! # *Nothing granted* is a sentence
//!
//! An empty list beside a settings heading reads as a screen that failed to
//! load, and the state it stands for — the machine holds no reach at all — is
//! the single most reassuring fact this surface can show. So a listing with
//! no rows answers [`Listing::word`] with [`crate::words::NOTHING_GRANTED`],
//! and [`Listing::said`] renders it in the language the person reads.

use std::time::SystemTime;

use alo_capability::Grants;
use alo_strings::{Filling, Said, Strings};

use crate::seen::Seen;
use crate::words::{self, Word};

/// What a surface showing the grants would show, at one moment.
///
/// ```
/// use alo_capability::Grants;
/// use alo_granted::Listing;
/// use std::time::{Duration, SystemTime};
///
/// let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
/// let listing = Listing::of(&Grants::default(), noon);
/// assert!(listing.is_nothing_granted());
/// assert!(listing.rows().is_empty());
/// ```
///
/// A listing cannot be assembled from rows somebody made up — it has no
/// reachable parts, and a [`crate::Seen`] has no constructor:
///
/// ```compile_fail
/// let listing = alo_granted::Listing { rows: Vec::new() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// The active grants, in the order they were made — the order a person
    /// will look for them in. Filled by [`Listing::of`] and by nothing else.
    rows: Vec<Seen>,
}

impl Listing {
    /// What the machine's grants read as, right now.
    ///
    /// The only way to a [`Listing`]. Reading it never changes the grants:
    /// `grants` is borrowed, and asking a hundred times leaves the list
    /// exactly as it was.
    #[must_use]
    pub fn of(grants: &Grants, now: SystemTime) -> Self {
        Self {
            rows: grants
                .active_at(now)
                .filter_map(|held| Seen::of(held, now))
                .collect(),
        }
    }

    /// The rows, in the order the grants were made. Empty when nothing is
    /// granted — in which case [`Listing::said`] is what a person reads
    /// instead.
    #[must_use]
    pub fn rows(&self) -> &[Seen] {
        &self.rows
    }

    /// Whether the machine holds no reach at all — the state every machine
    /// starts in.
    #[must_use]
    pub fn is_nothing_granted(&self) -> bool {
        self.rows.is_empty()
    }

    /// The string this crate declares for the whole list, where it has one:
    /// the *nothing granted* sentence. [`None`] while there are rows, each of
    /// which says itself.
    #[must_use]
    pub fn word(&self) -> Option<Word> {
        self.is_nothing_granted().then_some(words::NOTHING_GRANTED)
    }

    /// What stands where the list would be when nothing is granted, in the
    /// language the person reads — and [`None`] while there are rows, so no
    /// caller can put the empty sentence above a list that is not empty.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        self.word()
            .map(|word| strings.say(&word.key(), &Filling::nothing()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{grants_of_one_folder, hour, in_english, noon, translated};
    use alo_capability::{Grant, Reach};

    /// **A machine where nothing is granted says so in a sentence**, rather
    /// than standing an empty list where reassurance belongs.
    #[test]
    fn nothing_granted_is_a_sentence_rather_than_an_empty_list() {
        let listing = Listing::of(&Grants::default(), noon());
        assert!(listing.is_nothing_granted());
        assert!(listing.rows().is_empty());
        assert_eq!(listing.word(), Some(words::NOTHING_GRANTED));

        let said = listing.said(&in_english()).unwrap();
        assert!(!said.is_a_bug(), "the sentence is not declared");
        assert!(said.text().starts_with("Nothing is granted"), "{said}");
    }

    /// **A list with rows has no empty sentence**, so no caller can show
    /// both.
    #[test]
    fn a_list_with_rows_has_no_empty_sentence() {
        let listing = Listing::of(&grants_of_one_folder(), noon());
        assert!(!listing.is_nothing_granted());
        assert_eq!(listing.word(), None);
        assert!(listing.said(&in_english()).is_none());
        assert_eq!(listing.rows().len(), 1);
    }

    /// **An expired grant is never shown as live.** The grant is still on the
    /// machine's list — nothing has swept it — and the listing derived after
    /// its end holds no row for it, because an expired grant cannot become a
    /// row at all.
    #[test]
    fn an_expired_grant_is_never_shown_as_live() {
        let grants = grants_of_one_folder();
        let after = noon() + hour();
        assert_eq!(
            grants.len(),
            1,
            "the grant was swept, so this proves nothing"
        );

        let listing = Listing::of(&grants, after);
        assert!(listing.rows().is_empty());
        assert!(listing.is_nothing_granted());
        let said = listing.said(&in_english()).unwrap();
        assert!(said.text().starts_with("Nothing is granted"), "{said}");
    }

    /// **The rows come in the order the grants were made**, which is the
    /// order a person will look for them in — `alo-capability`'s own order,
    /// kept rather than re-decided.
    #[test]
    fn the_rows_come_in_the_order_the_grants_were_made() {
        let mut grants = grants_of_one_folder();
        grants.grant(
            Grant::checked(
                "@blender",
                Reach::Application("org.blender.Blender".to_owned()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let listing = Listing::of(&grants, noon());
        let named: Vec<&str> = listing.rows().iter().map(Seen::to).collect();
        assert_eq!(named, ["@files", "@blender"]);
    }

    /// **Reading the list never widens anything.** A hundred derivations
    /// leave the grants byte for byte as they were — `alo-capability`'s
    /// *never widened by use*, seen from the screen that reads it.
    #[test]
    fn reading_the_list_leaves_the_grants_where_they_were() {
        let grants = grants_of_one_folder();
        let before = serde_json::to_string(&grants).unwrap();
        for _ in 0..100 {
            assert_eq!(Listing::of(&grants, noon()).rows().len(), 1);
        }
        assert_eq!(serde_json::to_string(&grants).unwrap(), before);
    }

    /// **The empty sentence arrives in the language the person reads**, which
    /// is the whole of what declaring it through `alo-strings` buys.
    #[test]
    fn the_empty_sentence_is_read_in_the_readers_own_language() {
        let strings = translated(&[(
            words::NOTHING_GRANTED,
            "Zurzeit ist nichts gewährt. Kein Agent erreicht irgendeinen Ordner, irgendeine Datei \
             oder Anwendung auf diesem Gerät, und es gibt hier nichts zu widerrufen",
        )]);
        let said = Listing::of(&Grants::default(), noon())
            .said(&strings)
            .unwrap();
        assert!(said.is_translated());
        assert!(said.text().starts_with("Zurzeit"), "{said}");
    }
}
