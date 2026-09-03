//! Where something an agent caused is going.
//!
//! Law 1 says nothing leaves silently, and a thing that leaves silently is
//! usually a thing nobody could have named. A base URL cannot be shown to a
//! person at the moment it matters — `https://…` is equally an appliance in the
//! next room and a service on another continent — so a destination says which
//! kind of place it is, in the same three kinds [`InferenceSource`] already
//! uses, widened to everything else an agent can reach.
//!
//! **The widening is the point of this file.** `alo-models` answers where a
//! *question* went. This answers where *anything* went, because the indicator a
//! person watches cannot have a gap in it for the egress somebody had not
//! thought of yet.
//!
//! **An address is checked before it can become a destination.** A paired
//! machine's name and a provider's name were written by a person, but a host is
//! whatever a verb's argument said, and the indicator has to be readable in one
//! line: text carrying a newline or an escape code can make the line say
//! something other than what is happening. That is the same reason
//! [`alo_capability::Arg`] refuses control characters, and the same refusal.
//!
//! **Neither of the two things here that a person reads has a `Display`**
//! (item 9h, following `alo-models` in 9f). A `Display` is one `to_string()`
//! away from a screen whose author had no reason to think about language, so
//! the only road to words is [`Destination::shown`] and
//! [`DestinationError::said`], both of which need the strings the reader in
//! front of the machine actually reads.
//!
//! **And a place goes inside a sentence through [`Destination::fills`]**
//! (item 15), never as a bare `String`. Both sentences a place appears in — the
//! indicator line law 1 shows while something is leaving, and the refusal a
//! policy makes — are only as translated as the place named in the middle of
//! them, and a gap holding text alone could not say so.

use alo_models::{InferenceSource, Region};
use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// The most characters an address may be.
///
/// A hostname cannot be longer than this and still be a hostname, so the bound
/// costs nothing real and stops the indicator being handed a paragraph.
const LONGEST: usize = 253;

/// Why something could not become a destination.
///
/// The sentences say what to do about them. They are read by whoever is holding
/// an egress that was never shown — somebody writing an adapter, or somebody
/// reading a refusal — and "invalid destination" would tell neither anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestinationError {
    /// Nothing, or only spaces.
    Nameless,
    /// A character that cannot be read on one line.
    NotPrintable,
    /// Longer than an address can be.
    TooLong {
        /// The most characters an address may be.
        ///
        /// A number beside the sentence rather than inside it, because a
        /// sentence that counted characters would be English's two plural
        /// shapes standing in for Polish's three — `words.rs` has the
        /// reasoning. Whoever draws this writes the number the way the reader's
        /// region writes one.
        longest: usize,
    },
    /// A source that is this machine, which is not a departure at all.
    NothingLeaves,
}

impl DestinationError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::Nameless => words::NOWHERE_NAMED,
            Self::NotPrintable => words::NOT_SHOWABLE,
            Self::TooLong { .. } => words::TOO_LONG,
            Self::NothingLeaves => words::NOTHING_LEAVES,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::egress_words`] answers with the
    /// key, marked, and `Said::is_a_bug`.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// Where something is going.
///
/// Read back as well as written down, because "where did that go?" is a
/// question asked at the end of a week and not only in the second the indicator
/// lit up. Reading one back reaches nothing and permits nothing: it names a
/// place, and naming a place is all it does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Destination {
    /// A machine on this network, paired deliberately
    /// ([ADR 0003](../../../docs/decisions/0003-the-network-is-not-authority.md)).
    PairedMachine {
        /// The machine's name, as a person named it when pairing.
        machine: String,
    },
    /// A named service, and where it says it runs.
    Provider {
        /// The provider, as a person would say it — "alo", not a hostname.
        provider: String,
        /// Where that provider says it runs.
        region: Region,
    },
    /// A host an argument named, with nothing declared about it.
    ///
    /// This is the honest kind. Nobody paired it and nobody wrote down where it
    /// runs, so it satisfies no policy that names a region — see
    /// [`Destination::is_in`].
    Address {
        /// The host, as it was named.
        host: String,
    },
}

impl Destination {
    /// A machine on this network, by the name it was paired under.
    ///
    /// # Errors
    /// [`DestinationError`] when the name could not be shown on one line.
    pub fn paired(machine: &str) -> Result<Self, DestinationError> {
        Ok(Self::PairedMachine {
            machine: readable(machine)?,
        })
    }

    /// A named service, and the region it says it runs in.
    ///
    /// # Errors
    /// [`DestinationError`] when the name could not be shown on one line.
    pub fn provider(provider: &str, region: Region) -> Result<Self, DestinationError> {
        Ok(Self::Provider {
            provider: readable(provider)?,
            region,
        })
    }

    /// A host, as an argument named it.
    ///
    /// # Errors
    /// [`DestinationError`] when the host could not be shown on one line.
    pub fn at(host: &str) -> Result<Self, DestinationError> {
        Ok(Self::Address {
            host: readable(host)?,
        })
    }

    /// Where a question goes, when it is answered somewhere other than here.
    ///
    /// # Errors
    /// [`DestinationError::NothingLeaves`] for
    /// [`InferenceSource::ThisMachine`], because an answer given here never
    /// departs and so has nothing to show. That is a refusal rather than a
    /// silent empty destination: a caller building an egress for a local answer
    /// has made a mistake, and the indicator would otherwise carry a line about
    /// a departure that did not happen.
    pub fn of(source: &InferenceSource) -> Result<Self, DestinationError> {
        match source {
            InferenceSource::ThisMachine => Err(DestinationError::NothingLeaves),
            InferenceSource::PairedMachine { machine } => Self::paired(machine),
            InferenceSource::Hosted { provider, region } => {
                Self::provider(provider, region.clone())
            }
        }
    }

    /// Whether this is inside the organisation's own building or network.
    ///
    /// Only a paired machine is. A host nobody paired is outside it even if it
    /// answers on the same wire, because "it is only our internal network" is
    /// exactly the assumption
    /// [ADR 0003](../../../docs/decisions/0003-the-network-is-not-authority.md)
    /// refuses to make.
    #[must_use]
    pub fn stays_in_the_building(&self) -> bool {
        matches!(self, Self::PairedMachine { .. })
    }

    /// Whether this satisfies a policy naming a region.
    ///
    /// A paired machine qualifies wherever the customer is: their machines are
    /// in their region by definition. A provider qualifies only where it has
    /// **declared** that region, and an address nobody declared anything about
    /// never does — guessing a region from a hostname is how a customer ends up
    /// in breach while looking at a reassuring label.
    #[must_use]
    pub fn is_in(&self, region: &str) -> bool {
        match self {
            Self::PairedMachine { .. } => true,
            Self::Provider {
                region: declared, ..
            } => declared.is(region),
            Self::Address { .. } => false,
        }
    }

    /// The string this crate declares for describing this place, where there
    /// is one.
    ///
    /// [`None`] for a host a verb's argument named, and that is the honest
    /// answer rather than a gap: `alo.example` is somebody's data, shown as it
    /// was written like a filename in `alo-files`, and a translation of it
    /// would be an invention.
    #[must_use]
    pub fn word(&self) -> Option<words::Word> {
        match self {
            Self::PairedMachine { .. } => Some(words::A_PAIRED_MACHINE),
            Self::Provider {
                region: Region::Declared(_),
                ..
            } => Some(words::A_PROVIDER),
            Self::Provider {
                region: Region::Unknown,
                ..
            } => Some(words::A_PROVIDER_SOMEWHERE),
            Self::Address { .. } => None,
        }
    }

    /// This place as a clause, carrying where its words came from.
    ///
    /// [`None`] for a host a verb's argument named, for the reason
    /// [`Destination::word`] answers `None`: `alo.example` is somebody's
    /// address, there is nothing in it anybody could have translated, and a
    /// [`Said`] about it would be a claim that somebody should have.
    ///
    /// **Anything putting this inside another sentence wants
    /// [`fills`](Self::fills)**, which is this and the host together in one
    /// door.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        let (word, filling) = match self {
            Self::PairedMachine { machine } => (
                words::A_PAIRED_MACHINE,
                Filling::of("machine", machine.clone()),
            ),
            Self::Provider { provider, region } => {
                let named = Filling::of("provider", provider.clone());
                match region {
                    Region::Declared(where_) => {
                        (words::A_PROVIDER, named.and("region", where_.clone()))
                    }
                    Region::Unknown => (words::A_PROVIDER_SOMEWHERE, named),
                }
            }
            Self::Address { .. } => return None,
        };
        Some(strings.say(&word.key(), &filling))
    }

    /// What to show a person, at the moment it is happening.
    ///
    /// A phrase rather than a sentence, because the sentence is
    /// [`crate::Leaving::said`]'s and there is one of those, in one place, for
    /// whoever translates it.
    ///
    /// [`said`](Self::said) with the provenance dropped, for a caller putting
    /// the clause on a line of its own. **One rendering, not two**: a place
    /// described here and described again inside a refusal would be two things
    /// that can disagree.
    #[must_use]
    pub fn shown(&self, strings: &Strings) -> String {
        self.said(strings)
            .map_or_else(|| self.as_named().to_owned(), Said::into_text)
    }

    /// This place, put into the gap named `name` of another sentence.
    ///
    /// The one door for a line with a place in it, because the branch is this
    /// type's own and not its callers': a place this crate has words for
    /// carries where those words came from, so the line around it is only as
    /// translated as the place inside it, and a host carries none because there
    /// is nothing in it to translate. Two callers deciding that for themselves
    /// would be two chances to put an English clause inside a German line with
    /// nothing anywhere counting it.
    #[must_use]
    pub fn fills(&self, name: &str, filling: Filling, strings: &Strings) -> Filling {
        match self.said(strings) {
            Some(said) => filling.and_said(name, &said),
            None => filling.and(name, self.as_named().to_owned()),
        }
    }

    /// This place as it was named, with no words around it.
    ///
    /// What [`shown`](Self::shown) falls back to for the one kind that has no
    /// words. It went through [`readable`] before it could become a destination
    /// at all, so it is something a person can be shown whichever kind it is.
    fn as_named(&self) -> &str {
        match self {
            Self::PairedMachine { machine } => machine,
            Self::Provider { provider, .. } => provider,
            Self::Address { host } => host,
        }
    }
}

/// Text with something in it, nothing in it that cannot be shown, and short
/// enough to be a line.
fn readable(text: &str) -> Result<String, DestinationError> {
    let text = text.trim();
    if text.is_empty() {
        return Err(DestinationError::Nameless);
    }
    if text.chars().any(char::is_control) {
        return Err(DestinationError::NotPrintable);
    }
    if text.chars().count() > LONGEST {
        return Err(DestinationError::TooLong { longest: LONGEST });
    }
    Ok(text.to_owned())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    fn eu() -> Region {
        Region::Declared("the EU".to_owned())
    }

    /// Every kind of source that leaves becomes a destination, and the one that
    /// does not leave is refused rather than becoming an empty one.
    #[test]
    fn a_source_that_leaves_becomes_a_destination_and_one_that_does_not_is_refused() {
        assert_eq!(
            Destination::of(&InferenceSource::PairedMachine {
                machine: "the studio workstation".to_owned()
            })
            .unwrap(),
            Destination::paired("the studio workstation").unwrap()
        );
        assert_eq!(
            Destination::of(&InferenceSource::Hosted {
                provider: "alo".to_owned(),
                region: eu()
            })
            .unwrap(),
            Destination::provider("alo", eu()).unwrap()
        );
        assert_eq!(
            Destination::of(&InferenceSource::ThisMachine).unwrap_err(),
            DestinationError::NothingLeaves
        );
    }

    /// **Law 1's exception, refused here as it is in `alo-models`.** A machine
    /// on the same network was reached across a wire, and the only reason it is
    /// treated differently at all is that a person paired it deliberately.
    #[test]
    fn a_host_nobody_paired_is_outside_the_building_even_on_the_same_wire() {
        assert!(
            Destination::paired("the studio workstation")
                .unwrap()
                .stays_in_the_building()
        );
        assert!(
            !Destination::at("files.local")
                .unwrap()
                .stays_in_the_building()
        );
        assert!(
            !Destination::provider("alo", eu())
                .unwrap()
                .stays_in_the_building()
        );
    }

    /// A region is satisfied by a declaration, never by a hostname that looks
    /// European. An address nobody declared anything about satisfies none.
    #[test]
    fn only_a_declared_region_satisfies_one_and_an_address_never_does() {
        assert!(Destination::provider("alo", eu()).unwrap().is_in("the EU"));
        assert!(
            !Destination::provider("someone", Region::Unknown)
                .unwrap()
                .is_in("the EU")
        );
        assert!(!Destination::at("mail.eu").unwrap().is_in("the EU"));
        assert!(
            Destination::paired("the studio workstation")
                .unwrap()
                .is_in("Singapore"),
            "a person's own machine is in their region wherever that is"
        );
    }

    /// **The indicator is one line, and a destination cannot rewrite it.** The
    /// host came from a verb's argument, which came from a model that may have
    /// been talked into something.
    #[test]
    fn an_address_that_could_rewrite_the_indicator_never_becomes_a_destination() {
        for attempt in [
            "alo.example\nand nothing is leaving",
            "\u{1b}[2Kalo.example",
            "alo.example\u{7}",
        ] {
            assert_eq!(
                Destination::at(attempt).unwrap_err(),
                DestinationError::NotPrintable,
                "{attempt:?}"
            );
        }
        assert_eq!(
            Destination::at("   ").unwrap_err(),
            DestinationError::Nameless
        );
        assert_eq!(
            Destination::at(&"x".repeat(LONGEST + 1)).unwrap_err(),
            DestinationError::TooLong { longest: LONGEST }
        );
        assert!(Destination::at(&"x".repeat(LONGEST)).is_ok());
    }

    /// The refusals say what to do, not only what went wrong — and the one
    /// that carries a number keeps it beside the sentence rather than inside
    /// it, so nobody writes English's plural shapes into a language with more.
    #[test]
    fn the_refusals_say_what_to_do() {
        let strings = in_english();
        let nowhere = DestinationError::Nameless.said(&strings);
        assert!(
            nowhere.text().contains("say where this is going"),
            "{nowhere}"
        );
        let here = DestinationError::NothingLeaves.said(&strings);
        assert!(here.text().contains("causes egress"), "{here}");

        let long = DestinationError::TooLong { longest: LONGEST };
        assert!(long.said(&strings).text().contains("hostname"), "{long:?}");
        assert!(
            !long
                .said(&strings)
                .text()
                .chars()
                .any(|char| char.is_ascii_digit()),
            "the number is a field, not a word in the sentence"
        );
        assert_eq!(long, DestinationError::TooLong { longest: 253 });
    }

    /// What a person reads says the uncomfortable thing plainly: a provider
    /// that has not said where it runs says so rather than sounding settled.
    #[test]
    fn an_undeclared_provider_says_so_rather_than_sounding_safe() {
        let strings = in_english();
        let said = Destination::provider("someone", Region::Unknown)
            .unwrap()
            .shown(&strings);
        assert!(said.contains("has not said where it runs"), "{said}");
        assert_eq!(
            Destination::provider("alo", eu()).unwrap().shown(&strings),
            "alo, in the EU"
        );
        assert_eq!(
            Destination::paired("the studio workstation")
                .unwrap()
                .shown(&strings),
            "the studio workstation, on your network"
        );
    }

    /// **A host is data, not a string.** It is shown exactly as it was written
    /// — like a filename in `alo-files` — and there is nothing here for a
    /// translator to be handed, which is what [`Destination::word`] answering
    /// [`None`] says.
    #[test]
    fn a_host_nobody_declared_anything_about_is_shown_as_it_was_written() {
        let address = Destination::at("alo.example").unwrap();
        assert_eq!(address.word(), None);
        assert_eq!(address.shown(&in_english()), "alo.example");
        assert_eq!(
            address.shown(&Strings::of(alo_strings::Vocabulary::empty())),
            "alo.example",
            "a host does not stop being shown because nothing was declared"
        );
    }

    /// **The word a destination names is the string it is shown by.** They are
    /// written in two places — one answers *which string*, the other fills it
    /// — and a test rather than a comment is what keeps them the same.
    #[test]
    fn the_string_a_destination_names_is_the_one_it_is_shown_by() {
        for (destination, german) in [
            (
                Destination::paired("the studio workstation").unwrap(),
                "anderswo: {machine}",
            ),
            (
                Destination::provider("alo", eu()).unwrap(),
                "anderswo: {provider}, in {region}",
            ),
            (
                Destination::provider("someone", Region::Unknown).unwrap(),
                "anderswo: {provider}",
            ),
        ] {
            let word = destination.word().unwrap();
            let strings = translated(&[(word, german)]);
            assert!(
                destination.shown(&strings).starts_with("anderswo:"),
                "{}: {}",
                word.named(),
                destination.shown(&strings)
            );
        }
    }

    /// Where something went outlives the moment it went, so a destination has
    /// to survive being written down and read back — still saying the same
    /// thing about the building and about the region.
    #[test]
    fn a_destination_survives_being_written_down_and_read_back() {
        for destination in [
            Destination::paired("the studio workstation").unwrap(),
            Destination::provider("alo", eu()).unwrap(),
            Destination::provider("someone", Region::Unknown).unwrap(),
            Destination::at("alo.example").unwrap(),
        ] {
            let written = serde_json::to_string(&destination).unwrap();
            let read = serde_json::from_str::<Destination>(&written).ok();
            assert_eq!(read.as_ref(), Some(&destination), "{written}");
            assert_eq!(
                read.map(|read| read.stays_in_the_building()),
                Some(destination.stays_in_the_building()),
                "{written}"
            );
        }
    }
}
