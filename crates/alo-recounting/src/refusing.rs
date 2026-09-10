//! What a person reads when they asked what their machine did and were not
//! answered.
//!
//! There is no silent failure anywhere in this crate, and here the reason is
//! sharper than usual: **an unanswered question about a record looks exactly
//! like a record with nothing in it.** Somebody who asks what the agent has
//! been doing and is shown an empty screen has been told something about their
//! machine, and on a machine whose record is missing that something is false.
//!
//! Like everything else this repository says to a person, the sentences are
//! declared in [`crate::words`] and answered through [`NotRecounted::said`];
//! there is no `Display` that would put English on a screen by accident.
//!
//! # Most of this is somebody else's words
//!
//! Everything about the record itself is `alo_keeping::NotKept`, carried whole
//! and asked for the sentence. That crate already decided what *there is no
//! record here* means and worded it — including the half of the sentence that
//! matters most, *a machine with no record is not a machine that has done
//! nothing* — and a second wording of it here would be a machine able to
//! describe one fact two ways, with the milder one on the screen.
//!
//! What is left is this crate's own, and nobody else knows them: there was
//! nowhere to put the account, the compositor refused it, and this machine
//! could not say where it keeps a record in the first place. The last three are
//! `where_it_is.rs`' refusals, worded here because they are read by the person
//! who asked the question rather than by whoever wrote the description.

use alo_keeping::NotKept;
use alo_strings::{Filling, Said, Strings};

use crate::surface::SurfaceRefused;
use crate::where_it_is::NotSaid;
use crate::words;

/// Why what the machine did was not put in front of the person.
///
/// Four shapes of *not answered*: no compositor at all, a compositor that
/// refused, a record that could not be read, and a machine that could not say
/// where its record is. The first two are kept apart
/// because the person is told different things — one is *the desktop is not
/// running*, the other is a fact the compositor knows, like a machine with no
/// screen — and because they are fixed by different actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotRecounted {
    /// There is no compositor to ask: nothing on this machine is drawing a
    /// screen at all.
    NoCompositor,
    /// The compositor was asked and refused, for the reason carried.
    Surface(SurfaceRefused),
    /// The record could not be read — `alo-keeping`'s own refusal, carried
    /// whole and worded by the crate that made it.
    Record(NotKept),
    /// This machine could not say where it keeps a record at all, so nothing
    /// was looked for.
    ///
    /// Apart from [`NotRecounted::Record`] because it is a different machine:
    /// one has a record that could not be read, the other cannot say whether it
    /// has one. Both are refusals, and neither is an empty day.
    Nowhere(NotSaid),
}

impl NotRecounted {
    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put in front of the person, and where it
    /// came from is on the [`Said`]. A `Strings` that was never given
    /// [`crate::recounting_words`] answers with the key, marked
    /// `Said::is_a_bug` — the honest answer to *the shell forgot to declare
    /// what this crate can say*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::NoCompositor => strings.say(&words::NO_COMPOSITOR.key(), &Filling::nothing()),
            Self::Surface(SurfaceRefused::NothingToShowOn) => {
                strings.say(&words::NOTHING_TO_SHOW_ON.key(), &Filling::nothing())
            }
            Self::Record(why) => why.said(strings),
            Self::Nowhere(NotSaid::NoDescription { .. }) => {
                strings.say(&words::NO_DESCRIPTION.key(), &Filling::nothing())
            }
            Self::Nowhere(NotSaid::NotADescription { .. }) => {
                strings.say(&words::NOT_A_DESCRIPTION.key(), &Filling::nothing())
            }
            Self::Nowhere(NotSaid::NotRead { .. }) => {
                strings.say(&words::DESCRIPTION_NOT_READ.key(), &Filling::nothing())
            }
        }
    }

    /// Whether this is the machine having nowhere to put the account, rather
    /// than there being no account to put.
    ///
    /// A shell does two different things about them: the first is worth saying
    /// out loud to whoever is standing the machine up, and the second is about
    /// the record itself and belongs in front of the person who asked.
    #[must_use]
    pub fn is_nowhere_to_show_it(&self) -> bool {
        matches!(self, Self::NoCompositor | Self::Surface(_))
    }

    /// Whether there is no record on this machine at all.
    ///
    /// **The one a caller must not treat as an empty answer.** A machine that
    /// has done nothing and a machine whose record has been deleted are
    /// indistinguishable from a screen, and only one of them is innocent; this
    /// is `alo_keeping::Reading`'s refusal to answer *nothing happened*,
    /// carried up to the surface where somebody would otherwise draw a blank
    /// list.
    #[must_use]
    pub fn there_is_no_record(&self) -> bool {
        matches!(self, Self::Record(NotKept::NotThere { .. }))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way an account can fail to reach somebody.
    fn every_refusal() -> Vec<NotRecounted> {
        vec![
            NotRecounted::NoCompositor,
            NotRecounted::Surface(SurfaceRefused::NothingToShowOn),
            NotRecounted::Record(NotKept::NotThere {
                path: "/var/lib/alo/record.jsonl".to_owned(),
            }),
            NotRecounted::Record(NotKept::NotARecord {
                path: "/var/lib/alo/record.jsonl".to_owned(),
            }),
            NotRecounted::Record(NotKept::FromANewerAlo {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                format: 4,
            }),
            NotRecounted::Record(NotKept::NotRead {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                why: "permission denied".to_owned(),
            }),
            NotRecounted::Record(NotKept::ALink {
                path: "/var/lib/alo/record.jsonl".to_owned(),
            }),
            NotRecounted::Record(NotKept::SomebodyElses {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                owner: 1001,
            }),
            NotRecounted::Record(NotKept::WritableByOthers {
                path: "/var/lib/alo/record.jsonl".to_owned(),
                mode: 0o666,
            }),
            NotRecounted::Nowhere(NotSaid::NoDescription {
                path: "/etc/alo/agentd.toml".to_owned(),
            }),
            NotRecounted::Nowhere(NotSaid::NotADescription {
                path: "/etc/alo/agentd.toml".to_owned(),
            }),
            NotRecounted::Nowhere(NotSaid::NotRead {
                path: "/etc/alo/agentd.toml".to_owned(),
                why: "permission denied".to_owned(),
            }),
        ]
    }

    /// **Every refusal says something a person could read**, and no two ways of
    /// failing read the same — somebody told the same sentence for a missing
    /// desktop and a missing record would go and fix the wrong one, and one of
    /// those two is evidence going missing.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert!(!said.text().is_empty(), "{refusal:?} says nothing");
            assert!(!said.is_a_bug(), "{refusal:?} is not declared");
            assert!(
                said.unfilled().is_empty(),
                "{refusal:?} left {:?} with nothing in it",
                said.unfilled()
            );
            assert!(!seen.contains(&said.text().to_owned()), "two say {said}");
            seen.push(said.text().to_owned());
        }
    }

    /// **A missing record is never an empty answer.** It is the one refusal a
    /// caller has to be able to tell from every other, because drawing an empty
    /// list for it would tell somebody their agent has done nothing.
    #[test]
    fn a_record_that_is_not_there_is_answerable_without_matching_everything() {
        let missing = NotRecounted::Record(NotKept::NotThere {
            path: "/var/lib/alo/record.jsonl".to_owned(),
        });
        assert!(missing.there_is_no_record());
        assert!(!missing.is_nowhere_to_show_it());

        // And the sentence is `alo-keeping`'s, which is the one that says why a
        // missing record is worth finding out about.
        let said = missing.said(&in_english());
        assert!(said.text().contains("has done nothing"), "{said}");

        for other in every_refusal().into_iter().filter(|why| why != &missing) {
            assert!(!other.there_is_no_record(), "{other:?}");
        }
        assert!(NotRecounted::NoCompositor.is_nowhere_to_show_it());
        assert!(NotRecounted::Surface(SurfaceRefused::NothingToShowOn).is_nowhere_to_show_it());
    }

    /// **Nothing about the record itself is worded here.** With nothing but
    /// this crate's list loaded, the five that are ours read — the two about a
    /// surface and the three about a machine that cannot say where its record
    /// is — and every refusal `alo-keeping` made is a key nothing declares,
    /// which is what says the rest are somebody else's words rather than copies
    /// of them.
    #[test]
    fn the_only_sentences_this_crate_says_of_its_own_are_its_own() {
        let ours = Strings::of(crate::words::recounting_words().unwrap());
        let said_by_us = every_refusal()
            .iter()
            .filter(|refusal| !refusal.said(&ours).is_a_bug())
            .count();
        assert_eq!(
            said_by_us, 5,
            "this surface has started saying something somebody else already says"
        );
        for about_the_record in every_refusal()
            .iter()
            .filter(|refusal| matches!(refusal, NotRecounted::Record(_)))
        {
            assert!(
                about_the_record.said(&ours).is_a_bug(),
                "{about_the_record:?}"
            );
        }
    }

    /// **A machine that cannot say where its record is has not said nothing
    /// happened.** It is refused apart from every refusal about a record,
    /// because the two send somebody to different parts of their machine.
    #[test]
    fn a_machine_that_cannot_say_where_its_record_is_is_refused_apart() {
        let nowhere = NotRecounted::Nowhere(NotSaid::NoDescription {
            path: "/etc/alo/agentd.toml".to_owned(),
        });
        assert!(!nowhere.there_is_no_record());
        assert!(!nowhere.is_nowhere_to_show_it());

        let said = nowhere.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("does not say where"), "{said}");
    }

    /// **A refusal arrives in the language the person reads** when somebody has
    /// translated it, and says so.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let german = translated(&[(
            words::NO_COMPOSITOR,
            "Was dieses Gerät getan hat, kann Ihnen nicht gezeigt werden: der Schreibtisch läuft \
             nicht. Melden Sie sich am Schreibtisch an und fragen Sie erneut",
        )]);
        let said = NotRecounted::NoCompositor.said(&german);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().starts_with("Was dieses Gerät"));

        // The one nobody translated is still English, and says it is.
        let untranslated = NotRecounted::Surface(SurfaceRefused::NothingToShowOn).said(&german);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug(), "{untranslated}");
    }
}
