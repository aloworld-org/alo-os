//! The pairings as they are written down, and read back believed or refused.
//!
//! A pairing is a row two people made (ADR 0003) and the key the two
//! machines agreed (ADR 0031), and until this file existed both lived in
//! memory: a machine switched off at night was paired with nothing in the
//! morning, and the person who had walked to the other room to read six
//! digits aloud did it again. This is the shape a machine keeps them in
//! between restarts.
//!
//! ```toml
//! format = 1
//!
//! [[pairing]]
//! with = "0f1e2d3c4b5a69788796a5b4c3d2e1f0"
//! may = ["models"]
//! made = 1760000000
//! ends = 1760086400
//! key = "…sixty-four lowercase hexadecimal characters…"
//! ```
//!
//! # What is written is exactly the row two people made
//!
//! Five fields, and every one of them is something the two people were shown
//! or agreed: the other machine, the enumerated list, when, until when, and
//! the key both machines derived over those terms. Nothing about a pairing is
//! widened by being written down — there is no field for an address, for a
//! name a person gave the machine, or for *trusted*, because a row with any
//! of those would be a pairing on terms nobody confirmed. A key nobody
//! declared is refused rather than read around, for `reading.rs`'s reason.
//!
//! # Every row is made again on the way in
//!
//! Nothing here deserialises a [`Pairing`]. Each row is handed to the same
//! constructor [`Deliberating::agreed`](crate::Deliberating::agreed) uses,
//! after the checks a proposal is held to: the list must permit something and
//! name only arms this crate has, the duration must be whole seconds within
//! [`AT_MOST`], the key must be exactly the bytes a key is.
//! A file hand-edited into a longer pairing or a wider list is refused whole,
//! by the crate that owns the rule.
//!
//! # A row that has ended is not on the list that comes back
//!
//! [`read`] takes the moment and drops every row whose end is at or before it
//! — before anybody holds the list, the way `alo-remembering` drops an expired
//! grant — and [`written`] writes only what stands at the moment it is called.
//! That is what makes *a pairing's expiry survives a restart* a property of
//! the file rather than a filter somebody has to remember: a pairing that
//! ended while the machine was off is gone when the machine wakes, and a key
//! never outlives the row it was made for.
//!
//! # The key is in the file, and this is the decision
//!
//! The plan asked which store the key goes in, with the care a credential
//! gets. It goes here, in a file the person owns and nobody else can read or
//! write, beside the grants and the machine's own identity — and not in the
//! keyring a provider's key lives in (ADR 0022). Three reasons, in order of
//! weight. The identity the key is paired **to** is already a file in that
//! folder (`machine-id`), and a key kept somewhere the identity is not would
//! be half a pairing in each of two stores. A keyring can be locked when the
//! service starts, and a machine that reads its pairings at start would then
//! be paired with nothing until somebody typed a password — which is the
//! *paired with nothing in the morning* this file exists to end, wearing a
//! prompt. And what the key lets its holder do is bounded on the other
//! machine by that machine's own grants and by this row's expiry, so it is
//! exactly as sensitive as the grants file beside it and no more: whoever can
//! read this file could already rewrite what this machine's agent may reach.
//! The file is `0600`, refused if anybody else could write it, and refused
//! whole if any row does not hold — `alo-remembering` holds it to the grants
//! file's three rules.

use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::deliberating::AT_MOST;
use crate::hexing;
use crate::keying::PairingKey;
use crate::machine::MachineId;
use crate::pairing::{NotPaired, Pairing, Pairings};
use crate::permitting::MayAskIts;

/// Which shape of pairings file this alo OS writes and reads.
pub const THE_PAIRINGS_FORMAT: u32 = 1;

/// Why the pairings a machine kept were not read, or could not be written.
///
/// English, with a `Display`, for `alo-remembering`'s reason: nobody using
/// the machine reads these. They are read out of a service log by whoever is
/// standing a machine up and has found a pairings file that does not hold —
/// and every one of them means **no pairings were read**, because a list
/// that silently dropped the row a person is looking for, or silently kept a
/// key it could not check, would be a list that lies about who this machine
/// is paired with.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NotWrittenDown {
    /// The text is not the shape a machine's pairings take.
    #[error("the pairings a machine keeps are not the shape they take: {why}")]
    NotTheShape {
        /// What would not parse, in the parser's words.
        why: String,
    },

    /// Pairings written for an alo OS this is not.
    ///
    /// Refused before any row is looked at, the machine description's rule
    /// kept for its reason.
    #[error(
        "these pairings say format {format}, and this alo OS reads only format {}",
        THE_PAIRINGS_FORMAT
    )]
    AnotherFormat {
        /// The number the file says.
        format: u32,
    },

    /// A row naming something that is not a machine's identity.
    #[error(
        "the pairing in row {row} names `{said}` as a machine, which is not a machine's identity"
    )]
    NotAMachine {
        /// Which row, counting from one.
        row: usize,
        /// What it said.
        said: String,
    },

    /// A row permitting something no pairing can permit.
    ///
    /// The list is closed ([`crate::EVERYTHING_A_PAIRING_MAY_PERMIT`]), and
    /// a word that is not on it is refused rather than widened.
    #[error(
        "the pairing in row {row} permits `{said}`, which is not something a pairing may permit"
    )]
    NotSomethingAPairingMayPermit {
        /// Which row, counting from one.
        row: usize,
        /// What it said.
        said: String,
    },

    /// A row whose key is not the bytes a key is.
    ///
    /// The length is carried rather than the text, because a key has no
    /// business in a sentence even when it is wrong.
    #[error("the pairing in row {row} holds {length} characters as its key, which is not a key")]
    NotAKey {
        /// Which row, counting from one.
        row: usize,
        /// How many characters were there.
        length: usize,
    },

    /// A row this crate would not make a pairing from.
    ///
    /// The list permits nothing, the pairing lasts no time or longer than
    /// [`AT_MOST`], or its end cannot be represented — each in this crate's
    /// own words for it, since this crate owns the rule.
    #[error("the pairing in row {row} is not one this machine would make: {said}")]
    NotAPairing {
        /// Which row, counting from one.
        row: usize,
        /// What this crate says about it, in the language this code is
        /// written in — nobody reading a service log has a vocabulary loaded.
        said: &'static str,
    },

    /// Two rows about one machine.
    ///
    /// A list a person cannot act on: revoking the one they can see would
    /// leave the one they cannot. [`Pairings::keep`] replaces rather than
    /// joins for the same reason, so a file with two is a file nothing here
    /// wrote.
    #[error("the pairings name `{with}` twice, and a machine is paired with another once")]
    TwoRowsForOneMachine {
        /// The machine named twice.
        with: String,
    },

    /// A pairing timed before 1970, which cannot be written down.
    #[error(
        "the pairing with `{with}` is timed before 1970 and cannot be written down; this \
         machine's clock is wrong rather than its pairings"
    )]
    NotAMoment {
        /// The machine it is with.
        with: String,
    },
}

/// The file's shape, as serde sees it.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Kept {
    /// Which shape this file is in.
    format: u32,
    /// The pairings, one table each.
    #[serde(rename = "pairing", default)]
    pairings: Vec<KeptPairing>,
}

/// One pairing's table in the file: the five things two people agreed.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KeptPairing {
    /// The other machine, by its identity.
    with: String,
    /// What it may ask this machine for, each as the wire spells it.
    may: Vec<String>,
    /// When the two people agreed, in seconds since 1970.
    made: u64,
    /// When it stops, in seconds since 1970.
    ends: u64,
    /// The key both machines hold, as lowercase hexadecimal.
    key: String,
}

impl KeptPairing {
    /// This row as the pairing it means, checked the way a proposal is.
    fn as_a_pairing(&self, row: usize) -> Result<Pairing, NotWrittenDown> {
        let with = MachineId::read(&self.with).map_err(|_| NotWrittenDown::NotAMachine {
            row,
            said: self.with.clone(),
        })?;
        let mut may = Vec::with_capacity(self.may.len());
        for arm in &self.may {
            may.push(MayAskIts::read(arm).ok_or_else(|| {
                NotWrittenDown::NotSomethingAPairingMayPermit {
                    row,
                    said: arm.clone(),
                }
            })?);
        }
        if may.is_empty() {
            return Err(NotWrittenDown::NotAPairing {
                row,
                said: NotPaired::NothingAsked.word().says(),
            });
        }
        may.sort_unstable();
        may.dedup();
        let mut bytes = [0_u8; PairingKey::LENGTH];
        if !hexing::read(&self.key, &mut bytes) {
            return Err(NotWrittenDown::NotAKey {
                row,
                length: self.key.chars().count(),
            });
        }
        // Saturating rather than checked: a row that ends before it begins
        // lasts no time, and *no time* is this crate's own sentence for it.
        let lasting = Duration::from_secs(self.ends.saturating_sub(self.made));
        if lasting.is_zero() {
            return Err(NotWrittenDown::NotAPairing {
                row,
                said: NotPaired::NoTime.word().says(),
            });
        }
        if lasting > AT_MOST {
            return Err(NotWrittenDown::NotAPairing {
                row,
                said: NotPaired::TooLong.word().says(),
            });
        }
        Pairing::between(
            with,
            &may,
            at_second(self.made),
            lasting,
            PairingKey::of(bytes),
        )
        .map_err(|why| NotWrittenDown::NotAPairing {
            row,
            said: why.word().says(),
        })
    }
}

/// The moment this many seconds after 1970.
fn at_second(seconds: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
}

/// The whole second this moment is in, or a refusal naming the pairing.
///
/// Downwards, which is `alo-remembering`'s rounding for the same reason: a
/// pairing read back ends at or before the moment it was made to end, never
/// after.
fn as_seconds(moment: SystemTime, with: &MachineId) -> Result<u64, NotWrittenDown> {
    moment
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|since| since.as_secs())
        .map_err(|_before_the_epoch| NotWrittenDown::NotAMoment {
            with: with.as_str().to_owned(),
        })
}

/// The pairings this text holds, believed only whole, with the ended ones
/// already gone.
///
/// `now` is what *ended* is measured against, and dropping them here is the
/// point rather than housekeeping: what comes back is the pairings, not the
/// pairings and a filter somebody has to remember to apply.
///
/// # Errors
///
/// [`NotWrittenDown::NotTheShape`] for text that is not this file,
/// [`NotWrittenDown::AnotherFormat`] for pairings from an alo OS this is not
/// — answered before any row is looked at — and every way one row is not a
/// pairing this crate would make, each naming the row.
pub fn read(text: &str, now: SystemTime) -> Result<Pairings, NotWrittenDown> {
    let kept: Kept = toml::from_str(text).map_err(|why| NotWrittenDown::NotTheShape {
        why: why.to_string(),
    })?;
    if kept.format != THE_PAIRINGS_FORMAT {
        return Err(NotWrittenDown::AnotherFormat {
            format: kept.format,
        });
    }
    let mut pairings = Pairings::none();
    for (at, one) in kept.pairings.iter().enumerate() {
        let pairing = one.as_a_pairing(at.saturating_add(1))?;
        if pairings
            .every()
            .iter()
            .any(|already| already.with() == pairing.with())
        {
            return Err(NotWrittenDown::TwoRowsForOneMachine {
                with: pairing.with().as_str().to_owned(),
            });
        }
        // Before anybody has it: a pairing that ended while the machine was
        // off is not on the list the machine wakes with.
        if now < pairing.ends() {
            pairings.keep(pairing);
        }
    }
    Ok(pairings)
}

/// What is paired at this moment, as the file that keeps it.
///
/// A pairing that has ended is not written: it permits nothing, it is dropped
/// the moment the file is read anyway, and a key kept past its row would be a
/// credential outliving the thing it was for.
///
/// # Errors
///
/// [`NotWrittenDown::NotAMoment`] for a pairing timed before 1970, and
/// [`NotWrittenDown::NotTheShape`] if the value would not serialise, which
/// five strings and two numbers cannot cause.
pub fn written(pairings: &Pairings, now: SystemTime) -> Result<String, NotWrittenDown> {
    let mut rows = Vec::new();
    for pairing in pairings.every() {
        if now >= pairing.ends() {
            continue;
        }
        let made = as_seconds(pairing.made(), pairing.with())?;
        let ends = as_seconds(pairing.ends(), pairing.with())?;
        // A pairing whose whole-second end is not after its whole-second start
        // ends inside the second this is being written in; writing it would
        // produce a row the reader refuses as lasting no time.
        if ends <= made {
            continue;
        }
        rows.push(KeptPairing {
            with: pairing.with().as_str().to_owned(),
            may: pairing
                .may()
                .iter()
                .map(|arm| arm.said().to_owned())
                .collect(),
            made,
            ends,
            key: hexing::said(pairing.key().bytes()),
        });
    }
    let kept = Kept {
        format: THE_PAIRINGS_FORMAT,
        pairings: rows,
    };
    toml::to_string(&kept).map_err(|why| NotWrittenDown::NotTheShape {
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::{NotWrittenDown, read, written};
    use crate::pairing::Pairings;
    use crate::permitting::MayAskIts;
    use crate::proof::Proof;
    use crate::proven::Proven;
    use crate::replaying::Seen;
    use crate::testing::{a_moment, paired, reception, studio};

    /// A day, which is how long the fixture's pairing lasts.
    fn a_day() -> Duration {
        Duration::from_secs(86_400)
    }

    /// The studio's pairings: one row, with reception, made at the fixture's
    /// moment for a day — and reception's own row of the same pairing, for
    /// making proofs with.
    fn the_studios() -> (Pairings, crate::Pairing) {
        let (on_reception, on_studio) = paired();
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        (pairings, on_reception)
    }

    /// **What was written reads back and still holds the key**: a proof
    /// reception makes with its own row verifies against the row the studio
    /// read back off the file, which is the whole of what a pairing surviving
    /// a restart means.
    #[test]
    fn what_was_written_reads_back_and_the_key_still_proves_the_other_machine() {
        let (pairings, on_reception) = the_studios();
        let text = written(&pairings, a_moment()).unwrap();
        assert!(text.contains("format = 1"), "{text}");
        assert!(text.contains("[[pairing]]"), "{text}");
        assert!(text.contains(reception().as_str()), "{text}");
        assert!(text.contains("may = [\"models\"]"), "{text}");

        let back = read(&text, a_moment()).unwrap();
        assert_eq!(back.every().len(), 1);
        let row = back.every().first().unwrap();
        assert_eq!(row.with(), &reception());
        assert_eq!(row.may(), &[MayAskIts::Models]);
        assert!(back.permits(&reception(), MayAskIts::Models, a_moment()));

        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        assert!(
            Proven::checked(
                &back,
                &studio(),
                &proof,
                b"a question",
                a_moment(),
                &mut Seen::nothing()
            )
            .is_ok(),
            "the key did not survive the file"
        );
    }

    /// **The expiry survives with it, and a row that has ended is not on the
    /// list that comes back** — read one second before it ends and it is
    /// there; read at the moment it ends and it is gone, before anybody holds
    /// the list.
    #[test]
    fn the_expiry_survives_and_a_row_that_ended_is_not_read_back() {
        let (pairings, _) = the_studios();
        let text = written(&pairings, a_moment()).unwrap();

        let just_before = read(&text, a_moment() + a_day() - Duration::from_secs(2)).unwrap();
        assert_eq!(just_before.every().len(), 1);
        // The file keeps whole seconds and rounds the end downwards, so the
        // row read back ends at or before the moment it was made to end.
        let row = just_before.every().first().unwrap();
        assert!(row.ends() <= a_moment() + a_day());
        assert!(row.ends() > a_moment() + a_day() - Duration::from_secs(2));

        let at_the_end = read(&text, a_moment() + a_day()).unwrap();
        assert!(at_the_end.every().is_empty(), "a row that ended came back");
        assert!(!at_the_end.paired_with(&reception(), a_moment() + a_day()));
    }

    /// **A row that has ended is not written down either**, so a key never
    /// outlives the row it was made for.
    #[test]
    fn a_pairing_that_has_ended_is_not_written_down() {
        let (pairings, _) = the_studios();
        let text = written(&pairings, a_moment() + a_day()).unwrap();
        assert!(!text.contains("[[pairing]]"), "{text}");
        assert!(!text.contains("key"), "{text}");
        assert!(read(&text, a_moment()).unwrap().every().is_empty());
    }

    /// A machine paired with nothing keeps an empty list, and reads it back
    /// as one rather than as a failure.
    #[test]
    fn a_machine_paired_with_nothing_round_trips() {
        let text = written(&Pairings::none(), a_moment()).unwrap();
        assert!(read(&text, a_moment()).unwrap().every().is_empty());
    }

    /// **The key is in the file, and nowhere in a sentence.** The row spells it
    /// as sixty-four lowercase hexadecimal characters, and a key that is not
    /// exactly that is refused with its length rather than its text.
    #[test]
    fn a_key_that_is_not_a_key_is_refused_by_its_length_and_not_quoted() {
        let (pairings, _) = the_studios();
        let text = written(&pairings, a_moment()).unwrap();
        let key = text
            .lines()
            .find_map(|line| line.strip_prefix("key = \""))
            .map(|rest| rest.trim_end_matches('"'))
            .unwrap();
        assert_eq!(key.len(), 64);
        assert!(
            key.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        );

        for wrong in ["", "00", &key.to_uppercase(), &format!("{key}00")] {
            let edited = text.replace(key, wrong);
            let refused = read(&edited, a_moment()).unwrap_err();
            assert!(
                matches!(refused, NotWrittenDown::NotAKey { row: 1, .. }),
                "{refused}"
            );
            assert!(
                !refused.to_string().contains(key),
                "a key was quoted in a sentence: {refused}"
            );
        }
    }

    /// **A hand-edited row is refused whole, by the rule this crate owns**: a
    /// list widened to a word that is not an arm, an empty list, a pairing
    /// longer than the most a pairing may be, one that ends before it begins,
    /// and one naming something that is not a machine.
    #[test]
    fn a_row_this_crate_would_not_make_is_refused_whole() {
        let (pairings, _) = the_studios();
        let text = written(&pairings, a_moment()).unwrap();

        let widened = text.replace("may = [\"models\"]", "may = [\"models\", \"everything\"]");
        assert!(matches!(
            read(&widened, a_moment()).unwrap_err(),
            NotWrittenDown::NotSomethingAPairingMayPermit { row: 1, .. }
        ));

        let nothing = text.replace("may = [\"models\"]", "may = []");
        assert!(matches!(
            read(&nothing, a_moment()).unwrap_err(),
            NotWrittenDown::NotAPairing { row: 1, .. }
        ));

        let ends = text
            .lines()
            .find(|line| line.starts_with("ends = "))
            .unwrap()
            .to_owned();
        let too_long = text.replace(&ends, "ends = 9999999999");
        let refused = read(&too_long, a_moment()).unwrap_err();
        assert!(
            matches!(refused, NotWrittenDown::NotAPairing { row: 1, .. }),
            "{refused}"
        );
        let backwards = text.replace(&ends, "ends = 1");
        assert!(matches!(
            read(&backwards, a_moment()).unwrap_err(),
            NotWrittenDown::NotAPairing { row: 1, .. }
        ));

        let nobody = text.replace(reception().as_str(), "disan-laptop");
        assert!(matches!(
            read(&nobody, a_moment()).unwrap_err(),
            NotWrittenDown::NotAMachine { row: 1, .. }
        ));
    }

    /// **A field nobody declared is refused, not skipped** — there is no field
    /// for an address, a name or *trusted*, and a file that grew one is a file
    /// nothing here wrote.
    #[test]
    fn a_field_nobody_declared_is_refused() {
        let (pairings, _) = the_studios();
        let text = written(&pairings, a_moment()).unwrap();
        for extra in [
            "trusted = true",
            "address = \"192.168.1.20\"",
            "name = \"the studio\"",
        ] {
            let with_extra = format!("{text}\n{extra}\n");
            assert!(
                matches!(
                    read(&with_extra, a_moment()),
                    Err(NotWrittenDown::NotTheShape { .. })
                ),
                "`{extra}` was read around"
            );
        }
    }

    /// Text that is not this file, and a file from an alo OS this is not,
    /// are each refused as that, before any row is looked at.
    #[test]
    fn what_is_not_a_list_of_pairings_is_refused() {
        for wrong in ["=", "format = \"one\"", "just some text"] {
            assert!(matches!(
                read(wrong, a_moment()),
                Err(NotWrittenDown::NotTheShape { .. })
            ));
        }
        let (pairings, _) = the_studios();
        let newer = written(&pairings, a_moment())
            .unwrap()
            .replace("format = 1", "format = 2");
        assert!(matches!(
            read(&newer, a_moment()),
            Err(NotWrittenDown::AnotherFormat { format: 2 })
        ));
    }

    /// **Two rows about one machine are refused whole**: revoking the row a
    /// person can see would leave the one they cannot.
    #[test]
    fn two_rows_about_one_machine_are_refused() {
        let (pairings, _) = the_studios();
        let text = written(&pairings, a_moment()).unwrap();
        let table = text
            .split("[[pairing]]")
            .nth(1)
            .map(|rest| format!("[[pairing]]{rest}"))
            .unwrap();
        let doubled = format!("{text}\n{table}");
        assert!(matches!(
            read(&doubled, a_moment()),
            Err(NotWrittenDown::TwoRowsForOneMachine { .. })
        ));
    }

    /// A revoked pairing written again is gone from the file, so it does not
    /// come back at the next restart.
    #[test]
    fn a_revoked_pairing_is_not_in_the_file_written_afterwards() {
        let (mut pairings, _) = the_studios();
        assert!(pairings.revoke(&reception()));
        let text = written(&pairings, a_moment()).unwrap();
        assert!(!text.contains(reception().as_str()), "{text}");
        assert!(read(&text, a_moment()).unwrap().every().is_empty());
    }
}
